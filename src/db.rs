use tokio_postgres::Client;

use crate::Listing;

#[derive(Debug)]
pub struct Category {
    pub name: String,
    pub default_query: String,
}

pub async fn fetch_categories(
    client: &Client,
) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
    let rows = client.query("select name from categories", &[]).await?;

    let mut categories = Vec::new();

    for row in rows {
        let name: String = row.get(0);
        let default_query: String = row.get(1);

        categories.push(Category {
            name,
            default_query,
        });
    }

    Ok(categories)
}

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
