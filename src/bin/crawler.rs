use db::fetch_categories;
use olx_scrapper::db::insert_results_db;
use olx_scrapper::*;

use tokio_postgres::NoTls;

use reqwest::Client;

async fn crawl_query(
    client: &Client,
    search_term: &str,
    page: u32,
) -> Result<Vec<Listing>, Box<dyn std::error::Error>> {
    let mut listings = Vec::new();

    let mut page = page;

    loop {
        match fetch_listings(client, search_term, page).await {
            Ok(new_listings) => {
                listings.extend(new_listings);

                page += 1;
            }
            //once one reaches a page # that is not existent, olx redirects to the last page
            Err(ScrapperError::Redirected(_)) => {
                eprintln!("redirected, stopping");
                break;
            }
            Err(e) => Err(e)?,
        }
    }

    Ok(listings)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (psql_client, psql_connection) = tokio_postgres::connect(
        "host=localhost user=olx_scrapper_root password=pass dbname=olx_data",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = psql_connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let categories = fetch_categories(&psql_client).await?;

    println!("Fetched categories: {:?}", categories);

    let reqwest_client = Client::new();

    eprintln!("Crawling categories...");

    for category in categories {
        let listings = crawl_query(&reqwest_client, &category.default_query, 1).await?;
        insert_results_db(&psql_client, listings).await?;
    }

    println!("Done!");

    Ok(())
}
