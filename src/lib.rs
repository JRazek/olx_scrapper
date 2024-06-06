pub mod db;
pub mod histogram;

use chrono::prelude::*;
use scraper::selectable::Selectable;
use scraper::selector::*;
use scraper::ElementRef;
use thiserror::Error;

use reqwest::Client;
use reqwest::Error as ReqwestError;
use reqwest::Url;

use scraper::Html;
use serde::Serialize;

const OLX_URL: &str = "https://www.olx.pl";

#[derive(Debug, Serialize)]
pub struct Price {
    pub value: u32,
    pub negotiable: bool,
}

#[derive(Debug, Serialize)]
pub struct Listing {
    pub title: String,
    pub price: Price,
    pub location: String,
    pub date_posted: DateTime<Utc>,
    pub url: String,
}

#[derive(Debug, Error)]
pub struct FieldParsingError {
    error_type: String,
    message: String,
}

impl std::fmt::Display for FieldParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error_type: {}, message: {}",
            self.error_type, self.message
        )
    }
}

#[derive(Debug, Error)]
pub struct MissingFieldError(String);

impl std::fmt::Display for MissingFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing field: {}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum ScrapperError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] ReqwestError),

    #[error("Field parse error: {0}")]
    FieldParsingError(#[from] FieldParsingError),

    #[error("Missing field: {0}")]
    MissingFieldError(#[from] MissingFieldError),
}

fn parse_date(date: &str) -> Result<DateTime<Utc>, FieldParsingError> {
    let date = date.to_lowercase();
    let is_today = date.find("dzisiaj").is_some();

    if is_today {
        let time_regex = regex::Regex::new(r"(\d+:\d+)").unwrap();
        let time = time_regex
            .find(&date)
            .ok_or(FieldParsingError {
                error_type: "TimeParsingError".to_owned(),
                message: date.clone(),
            })?
            .as_str();

        let time = NaiveTime::parse_from_str(time, "%H:%M").map_err(|_| FieldParsingError {
            error_type: "TimeParsingError".to_owned(),
            message: date.clone(),
        })?;

        let now = Utc::now().with_time(time).unwrap();

        return Ok(now);
    }

    const MONTHS: [&str; 12] = [
        "stycznia",
        "lutego",
        "marca",
        "kwietnia",
        "maja",
        "czerwca",
        "lipca",
        "sierpnia",
        "września",
        "października",
        "listopada",
        "grudnia",
    ];

    let date_regex = regex::Regex::new(r"(\d+) (\w+) (\d{4})").unwrap();

    let captures = date_regex.captures(&date).ok_or(FieldParsingError {
        error_type: "DateParsingError".to_owned(),
        message: date.clone(),
    })?;

    let day = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();

    let month = captures.get(2).unwrap().as_str();

    let month = MONTHS
        .iter()
        .position(|&m| m == month)
        .ok_or(FieldParsingError {
            error_type: "DateParsingError".to_owned(),
            message: date.clone(),
        })? as u32
        + 1;

    let year = captures.get(3).unwrap().as_str().parse::<i32>().unwrap();

    let date = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();

    Ok(date)
}

fn get_location_date_from_raw_text(
    raw_text: impl ToOwned<Owned = String>,
) -> Result<(String, DateTime<Utc>), FieldParsingError> {
    let text = raw_text.to_owned();

    let lines: [&str; 2] = text
        .split(" - ")
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| FieldParsingError {
            error_type: "LocationDateParsingError".to_owned(),
            message: text.clone(),
        })?;

    let location = lines[0].to_owned();

    let date = parse_date(lines[1])?;

    Ok((location, date))
}

fn get_price_from_raw_text(
    raw_text: impl ToOwned<Owned = String>,
) -> Result<Price, FieldParsingError> {
    let text = raw_text.to_owned();
    let price_regex = regex::Regex::new(r"(\d+(?: \d+)*)(?:,(\d{2}))?").unwrap();

    let matched = price_regex.captures(&text).ok_or(FieldParsingError {
        error_type: "PriceParsingError".to_owned(),
        message: "No price found".to_owned(),
    })?;

    let map_error = |e: std::num::ParseIntError| FieldParsingError {
        error_type: "PriceParsingError".to_owned(),
        message: e.to_string(),
    };

    let integer = matched
        .get(1)
        .unwrap()
        .as_str()
        .replace(" ", "")
        .parse::<u32>()
        .map_err(map_error);

    let decimal = matched
        .get(2)
        .map(|m| m.as_str())
        .unwrap_or("00")
        .replace(" ", "")
        .parse::<u32>()
        .map_err(map_error);

    let value = integer.unwrap() * 100 + decimal.unwrap();

    let negotiable = text.find("do negocjacji").is_some();

    Ok(Price { value, negotiable })
}

fn parse_listing(listing: ElementRef) -> Result<Listing, ScrapperError> {
    let ad_card_title = listing
        .select(&Selector::parse(r#"[data-cy="ad-card-title"]"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("ad-card-title missing".to_owned()))?
        .select(&Selector::parse(r#"a"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("ad-card-title missing".to_owned()))?
        .select(&Selector::parse(r#"h6"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("ad-card-title missing".to_owned()))?
        .text()
        .collect::<String>();

    let price_raw_text = listing
        .select(&Selector::parse(r#"[data-testid="ad-price"]"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("ad-price missing".to_owned()))?
        .text()
        .collect::<String>();

    let price = get_price_from_raw_text(price_raw_text)?;

    let location_date = listing
        .select(&Selector::parse(r#"[data-testid="location-date"]"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("location-date missing".to_owned()))?
        .text()
        .collect::<String>();

    let (location, date) = get_location_date_from_raw_text(location_date)?;

    // slash is already added by the OLX.
    let url = listing
        .select(&Selector::parse(r#"[data-cy="ad-card-title"]"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("ad-card-title missing".to_owned()))?
        .select(&Selector::parse(r#"a"#).unwrap())
        .nth(0)
        .ok_or(MissingFieldError("a param missing".to_owned()))?
        .value()
        .attr("href")
        .ok_or(MissingFieldError("href missing".to_owned()))?
        .to_owned();

    Ok(Listing {
        title: ad_card_title,
        price,
        location,
        date_posted: date,
        url,
    })
}

pub async fn fetch_listings(
    client: &Client,
    search_term: &str,
) -> Result<Vec<Listing>, ScrapperError> {
    let url = Url::parse(&format!("{}/q-{}", OLX_URL, search_term)).unwrap();

    let response = client.get(url).send().await?;

    let body = response.text().await?;

    let html = Html::parse_document(&body);

    let listing_grid_selector = Selector::parse(r#"[data-testid="listing-grid"]"#).unwrap();

    let listing_grid =
        html.select(&listing_grid_selector)
            .nth(0)
            .ok_or(ScrapperError::FieldParsingError(FieldParsingError {
                error_type: "ListingGridNotFound".to_owned(),
                message: "Listing grid not found".to_owned(),
            }))?;

    let listings_selector = Selector::parse(r#"[data-testid="l-card"]"#).unwrap();
    let listings = listing_grid.select(&listings_selector).collect::<Vec<_>>();

    let mut listings_data = Vec::new();

    for listing in listings {
        match parse_listing(listing) {
            Ok(listing) => {
                listings_data.push(listing);
            }
            Err(e) => {
                println!("Error while parsing listing: {:?}", e);
                println!("skipping...");
            }
        }
    }

    Ok(listings_data)
}
