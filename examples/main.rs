use olx_scrapper::histogram::plot_histogram;
use olx_scrapper::*;
//use polars::prelude::*;

use reqwest::Client;

use plotters::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let listings = match fetch_listings(&client, "RTX 3070").await {
        Ok(listings) => listings,
        Err(e) => {
            panic!("could not fetch entries: {}", e);
        }
    };

    let prices = listings
        .iter()
        .map(|listing| listing.price.value)
        .collect::<Vec<_>>();

    println!("Done!");

    Ok(())
}
