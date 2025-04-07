use postgres_derive::{FromSql, ToSql};
use tokio_postgres::Client;

use tokio_postgres::types::FromSql;

use crate::Listing;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub default_query: String,
}

pub async fn fetch_categories(client: &Client) -> Result<Vec<Category>> {
    let rows = client
        .query("select name, default_query from categories", &[])
        .await?;

    let mut categories = Vec::new();

    for row in rows {
        let name: String = row.try_get(0)?;
        let default_query: String = row.try_get(1)?;

        categories.push(Category {
            id: 0, //todo
            name,
            default_query,
        });
    }

    Ok(categories)
}

pub async fn insert_results_db(client: &Client, listing: &Listing) -> Result<()> {
    client
        .execute(
            "insert into listings (
                    url,
                    category,
                    title,
                    price,
                    negotiable,
                    location,
                    date_posted
            ) values ($1, $2, $3, $4, $5, $6, $7)
                on conflict (url) do update set
                    last_seen = now()
                ",
            &[
                &listing.url,
                &1i32, //dummy for now
                &listing.title,
                &(listing.price.value as i32),
                &listing.price.negotiable,
                &listing.location,
                &listing.date_posted.naive_utc(),
            ],
        )
        .await?;

    Ok(())
}
