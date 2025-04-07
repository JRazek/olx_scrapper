use db::fetch_categories;
use olx_scrapper::db::insert_results_db;
use olx_scrapper::*;

use reqwest::redirect::Policy;
use tokio_postgres::NoTls;

use reqwest::Client;

use anyhow::{Context, Result};

async fn crawl_query(client: &Client, search_term: &str) -> Result<Vec<Listing>> {
    let mut listings = Vec::new();

    let mut page = 1;

    loop {
        println!("Fetching page: {}", page);

        match fetch_listings(client, search_term, page).await {
            Ok(new_listings) => {
                println!("Fetched {} listings", new_listings.len());
                listings.extend(new_listings);
            }
            //once one reaches a page # that is not existent, olx redirects to the last page
            Err(ScrapperError::Redirected(_)) => {
                eprintln!("redirected, stopping");

                break;
            }
            Err(e) => {
                eprintln!("Error while fetching listings: {:?}", e);

                Err(e)?;
            }
        }

        page += 1;
    }

    Ok(listings)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("connecting to db..");
    let (psql_client, psql_connection) = tokio_postgres::connect(
        "host=localhost user=olx_scrapper_root password=pass dbname=olx_data",
        NoTls,
    )
    .await
    .context("Failed to connect to database")?;

    tokio::spawn(async move {
        if let Err(e) = psql_connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Connected to db");

    println!("Fetching categories...");
    let categories = fetch_categories(&psql_client).await?;

    println!("Fetched categories: {:?}", categories);

    let reqwest_client = Client::builder().redirect(Policy::none()).build()?;

    eprintln!("Crawling categories...");

    for category in categories {
        println!("Fetching listings for category: {}", category.name);
        let listings = crawl_query(&reqwest_client, &category.default_query).await?;

        for listing in listings.iter() {
            insert_results_db(&psql_client, listing).await?;
        }

        println!(
            "Inserted {}, listings for category: {}",
            listings.len(),
            category.name
        );
    }

    println!("Done!");

    Ok(())
}
