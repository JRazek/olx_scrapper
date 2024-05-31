use chrono::prelude::*;
use thirtyfour::prelude::*;
use thiserror::Error;

const OLX_URL: &str = "https://www.olx.pl";

async fn try_accept_cookies(driver: &mut WebDriver) -> WebDriverResult<()> {
    match driver.find(By::Id("onetrust-accept-btn-handler")).await {
        Ok(elem) => {
            elem.click().await?;
        }
        Err(WebDriverError::NoSuchElement(..)) => {
            println!("No cookies to accept.");
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Price {
    value: u32,
    negotiable: bool,
}

#[derive(Debug, Serialize)]
pub struct Listing {
    title: String,
    price: Price,
    location: String,
    date: DateTime<Utc>,
    url: String,
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
pub enum ScrapperError {
    #[error("WebDriver error: {0}")]
    WebDriverError(#[from] WebDriverError),

    #[error("Price parse error: {0}")]
    FieldParsingError(#[from] FieldParsingError),
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
    let price_regex = regex::Regex::new(r"(\d+(?: \d+)*)").unwrap();

    let matched = price_regex.find(&text).ok_or(FieldParsingError {
        error_type: "PriceParsingError".to_owned(),
        message: "No price found".to_owned(),
    })?;

    let value = matched
        .as_str()
        .replace(" ", "")
        .parse::<u32>()
        .map_err(|_| FieldParsingError {
            error_type: "PriceParsingError".to_owned(),
            message: "Failed to parse price".to_owned(),
        })?;

    let negotiable = text.find("do negocjacji").is_some();

    Ok(Price { value, negotiable })
}

async fn parse_listing(listing: WebElement) -> Result<Listing, ScrapperError> {
    let ad_card_title = listing
        .find(By::Css(r#"[data-cy="ad-card-title"]"#))
        .await?
        .find(By::XPath(r#"a/h6"#))
        .await?
        .text()
        .await?;

    let price_raw_text = listing
        .find(By::Css(r#"[data-testid="ad-price"]"#))
        .await?
        .text()
        .await?;

    let price = get_price_from_raw_text(price_raw_text)?;

    let location_date = listing
        .find(By::Css(r#"[data-testid="location-date"]"#))
        .await?
        .text()
        .await?;

    let (location, date) = get_location_date_from_raw_text(location_date)?;

    // slash is already added by the OLX.
    let url = format!(
        "{}{}",
        OLX_URL,
        listing
            .find(By::Css(r#"[data-cy="ad-card-title"]"#))
            .await?
            .find(By::XPath(r#"a"#))
            .await?
            .attr("href")
            .await?
            .unwrap()
    );

    Ok(Listing {
        title: ad_card_title,
        price,
        location,
        date,
        url,
    })
}

pub async fn fetch_listings(
    driver: &mut WebDriver,
    search_term: &str,
) -> Result<Vec<Listing>, ScrapperError> {
    driver.goto(OLX_URL).await?;

    driver
        .set_implicit_wait_timeout(std::time::Duration::from_secs(10))
        .await?;

    try_accept_cookies(driver).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let elem_form = driver
        .find(By::Css(r#"[data-testid="search-form"]"#))
        .await?;

    // Find element from element.
    let elem_text = elem_form.find(By::Id("search")).await?;

    // Type in the search terms.
    elem_text.send_keys(search_term).await?;

    // Click the search button.
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    // Look for header to implicitly wait for the page to load.
    let listing_grid = driver
        .find(By::Css(r#"[data-testid="listing-grid"]"#))
        .await?;

    let listings = listing_grid
        .find_all(By::Css(r#"[data-testid="l-card"]"#))
        .await?;

    let mut listings_data = Vec::new();

    for listing in listings {
        match parse_listing(listing).await {
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
