use anyhow::Result;
use chrono::{DateTime, Utc};
use scooby::postgres::{insert_into, select, update, Parameters};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::query;
use sqlx::Row;

pub struct Post {
    indexed_at: DateTime<Utc>,
    author_did: String,
    cid: String,
    uri: String,
}

pub struct Profile {
    first_seen_at: DateTime<Utc>,
    did: String,
    has_been_processed: bool,
    likely_country_of_living: Option<String>,
}

pub struct SubscriptionState {
    service: String,
    cursor: i64,
}

pub struct Database {
    connection_pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            connection_pool: PgPoolOptions::new().max_connections(5).connect(url).await?,
        })
    }

    pub async fn insert_post(&self, author_did: &str, cid: &str, uri: &str) -> Result<()> {
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
        .execute(&self.connection_pool)
        .await
        .map(|_| ())?)
    }

    pub async fn insert_profile_if_it_doesnt_exist(&self, did: &str) -> Result<bool> {
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
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected() > 0)?)
    }

    pub async fn fetch_unprocessed_profile_dids(&self) -> Result<Vec<String>> {
        Ok(query(
            &select("did")
                .from("Profile")
                .where_("has_been_processed = FALSE")
                .to_string(),
        )
        .map(|r: PgRow| r.get(0))
        .fetch_all(&self.connection_pool)
        .await?)
    }

    pub async fn store_profile_details(
        &self,
        did: &str,
        likely_country_of_living: &str,
    ) -> Result<bool> {
        let mut params = Parameters::new();

        Ok(query(
            &update("Profile")
                .set("has_been_processed", "TRUE")
                .set("likely_country_of_living", params.next())
                .where_(format!("did = {}", params.next()))
                .to_string(),
        )
        .bind(likely_country_of_living)
        .bind(did)
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected() > 0)?)
    }
}
