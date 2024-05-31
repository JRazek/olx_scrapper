use olx_scrapper::*;
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //requires geckodriver to be running
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;

    let mut driver = WebDriver::new("http://localhost:4444", caps).await?;

    match fetch_listings(&mut driver, "RTX 3070").await {
        Ok(listings) => {
            let json = serde_json::to_string(&listings)?;

            println!("{}", json);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    driver.quit().await?;

    Ok(())
}
