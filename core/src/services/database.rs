use anyhow::Result;
use chrono::{DateTime, Utc};
use scooby::postgres::{
    delete_from, insert_into, select, update, Aliasable, Joinable, Orderable, Parameters,
};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::query;
use sqlx::Row;

pub struct Post {
    pub indexed_at: DateTime<Utc>,
    pub author_did: String,
    pub cid: String,
    pub uri: String,
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

    pub async fn fetch_posts_by_authors_country(
        &self,
        author_country: &str,
        limit: usize,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<Post>> {
        let mut params = Parameters::new();
        let mut sql_builder = select(("p.indexed_at", "p.author_did", "p.cid", "p.uri"))
            .from(
                "Post"
                    .as_("p")
                    .inner_join("Profile".as_("pr"))
                    .on("pr.did = p.author_did"),
            )
            .where_(format!("pr.likely_country_of_living = {}", params.next()))
            .order_by(("p.indexed_at".desc(), "p.cid".desc()))
            .limit(limit);

        if earlier_than.is_some() {
            sql_builder = sql_builder
                .where_(format!("p.indexed_at <= {}", params.next()))
                .where_(format!("p.cid < {}", params.next()));
        }

        let sql_string = sql_builder.to_string();

        let mut query_object = query(&sql_string).bind(author_country);

        if let Some((last_indexed_at, last_cid)) = earlier_than {
            query_object = query_object.bind(last_indexed_at).bind(last_cid);
        }

        Ok(query_object
            .map(|r: PgRow| Post {
                indexed_at: r.get("indexed_at"),
                author_did: r.get("author_did"),
                cid: r.get("cid"),
                uri: r.get("uri"),
            })
            .fetch_all(&self.connection_pool)
            .await?)
    }

    pub async fn delete_post(&self, uri: &str) -> Result<bool> {
        let mut params = Parameters::new();

        Ok(query(
            &delete_from("Post")
                .where_(format!("uri = {}", params.next()))
                .to_string(),
        )
        .bind(uri)
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected() > 0)?)
    }

    pub async fn delete_old_posts(&self, earlier_than: &DateTime<Utc>) -> Result<u64> {
        let mut params = Parameters::new();

        Ok(query(
            &delete_from("Post")
                .where_(format!("indexed_at < {}", params.next()))
                .to_string(),
        )
        .bind(earlier_than)
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected())?)
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

    pub async fn force_profile_country(
        &self,
        did: &str,
        likely_country_of_living: &str,
    ) -> Result<bool> {
        let mut transaction = self.connection_pool.begin().await?;

        {
            let mut params = Parameters::new();

            query(
                &insert_into("Profile")
                    .columns(("did",))
                    .values([params.next()])
                    .on_conflict()
                    .do_nothing()
                    .to_string(),
            )
            .bind(did)
            .execute(&mut *transaction)
            .await?;
        }

        {
            let mut params = Parameters::new();
            query(
                &update("Profile")
                    .set("has_been_processed", "TRUE")
                    .set("likely_country_of_living", params.next())
                    .where_(format!("did = {}", params.next()))
                    .to_string(),
            )
            .bind(likely_country_of_living)
            .bind(did)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(true)
    }

    pub async fn is_profile_in_this_country(
        &self,
        did: &str,
        country: &str,
    ) -> Result<Option<bool>> {
        let mut params = Parameters::new();

        Ok(query(
            &select("likely_country_of_living")
                .from("Profile")
                .where_(format!("did = {}", params.next()))
                .where_("has_been_processed = TRUE")
                .to_string(),
        )
        .bind(did)
        .map(|r: PgRow| r.get("likely_country_of_living"))
        .map(|c: String| c == country)
        .fetch_optional(&self.connection_pool)
        .await?)
    }

    pub async fn fetch_subscription_cursor(&self, host: &str, did: &str) -> Result<Option<i64>> {
        let mut params = Parameters::new();

        Ok(query(
            &select("cursor")
                .from("SubscriptionState")
                .where_(format!("service = {}", params.next()))
                .where_(format!("host = {}", params.next()))
                .to_string(),
        )
        .bind(did)
        .bind(host)
        .map(|r: PgRow| r.get("cursor"))
        .fetch_optional(&self.connection_pool)
        .await?)
    }

    pub async fn create_subscription_state(&self, host: &str, did: &str) -> Result<bool> {
        let mut params = Parameters::new();

        Ok(query(
            &insert_into("SubscriptionState")
                .columns(("service", "cursor", "host"))
                .values([params.next_array()])
                .to_string(),
        )
        .bind(did)
        .bind(0)
        .bind(host)
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected() > 0)?)
    }

    pub async fn update_subscription_cursor(
        &self,
        host: &str,
        did: &str,
        cursor: i64,
    ) -> Result<bool> {
        let mut params = Parameters::new();

        Ok(query(
            &update("SubscriptionState")
                .set("cursor", params.next())
                .where_(format!("service = {}", params.next()))
                .where_(format!("host = {}", params.next()))
                .to_string(),
        )
        .bind(cursor)
        .bind(did)
        .bind(host)
        .execute(&self.connection_pool)
        .await
        .map(|result| result.rows_affected() > 0)?)
    }
}
