use olx_scrapper::*;

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    match fetch_listings(&client, "RTX 3070").await {
        Ok(listings) => {
            let json = serde_json::to_string(&listings)?;

            println!("{}", json);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
