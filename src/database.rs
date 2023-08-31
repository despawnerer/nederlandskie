use anyhow::Result;
use chrono::{DateTime, Utc};
use scooby::postgres::{insert_into, Parameters};

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::query;

pub type ConnectionPool = PgPool;

pub struct Post {
    indexed_at: DateTime<Utc>,
    author_did: String,
    cid: String,
    uri: String,
}

pub struct Profile {
    first_seen_at: DateTime<Utc>,
    did: String,
    handle: Option<String>,
    likely_country_of_living: Option<String>,
}

pub struct SubscriptionState {
    service: String,
    cursor: i64,
}

pub async fn make_connection_pool() -> Result<ConnectionPool> {
    // TODO: get options from env vars
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/nederlandskie")
        .await?)
}

pub async fn insert_post(
    db: &ConnectionPool,
    author_did: &str,
    cid: &str,
    uri: &str,
) -> Result<()> {
    let mut params = Parameters::new();

    Ok(query(
        &insert_into("Post")
            .columns(("indexed_at", "author_did", "cid", "uri"))
            .values([[
                "now()".to_owned(),
                params.next(),
                params.next(),
                params.next(),
            ]])
            .to_string(),
    )
    .bind(author_did)
    .bind(cid)
    .bind(uri)
    .execute(db)
    .await
    .map(|_| ())?)
}
