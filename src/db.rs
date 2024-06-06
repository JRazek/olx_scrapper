use tokio_postgres::Client;

use crate::Listing;

pub async fn insert_results_db(
    client: &Client,
    listings: Vec<Listing>,
) -> Result<(), Box<dyn std::error::Error>> {
    for listing in listings {
        client
            .execute(
                "insert into listings (
                    url,
                    category,
                    title,
                    query,
                    price,
                    negotiable,
                    location,
                    date_posted
            ) values ($1, $2, $3, $4, $5, $6, $7, $8)
                on conflict (url) do update set
                    last_seen = now()
                ",
                &[
                    &listing.url,
                    &0i32,
                    &"dummy query",
                    &listing.title,
                    &(listing.price.value as i32),
                    &listing.price.negotiable,
                    &listing.location,
                    &listing.date_posted.naive_utc(),
                ],
            )
            .await?;
    }

    Ok(())
}
