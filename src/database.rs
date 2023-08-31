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
            .columns(("author_did", "cid", "uri"))
            .values([params.next_array()])
            .to_string(),
    )
    .bind(author_did)
    .bind(cid)
    .bind(uri)
    .execute(db)
    .await
    .map(|_| ())?)
}

pub async fn insert_profile_if_it_doesnt_exist(db: &ConnectionPool, did: &str) -> Result<bool> {
    let mut params = Parameters::new();

    Ok(query(
        &insert_into("Profile")
            .columns(("did",))
            .values([params.next()])
            .on_conflict()
            .do_nothing()
            .to_string(),
    )
    .bind(did)
    .execute(db)
    .await
    .map(|result| result.rows_affected() > 0)?)
}
