use olx_scrapper::db::insert_results_db;
use olx_scrapper::histogram::plot_histogram;
use olx_scrapper::*;

use tokio_postgres::NoTls;
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

    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=olx_scrapper_root password=pass dbname=olx_data",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    insert_results_db(&client, listings).await?;

    println!("Done!");

    Ok(())
}
