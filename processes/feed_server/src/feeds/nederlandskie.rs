use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::Feed;

use nederlandskie_core::services::database::{self, Database};

/// A feed that serves posts written in Russian by people living in Netherlands
pub struct NederlandskieFeed;

impl Default for NederlandskieFeed {
    fn default() -> Self {
        Self::new()
    }
}

impl NederlandskieFeed {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Feed for NederlandskieFeed {
    async fn fetch_posts(
        &self,
        database: &Database,
        limit: u8,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<database::Post>> {
        Ok(database
            .fetch_posts_by_authors_country("nl", limit as usize, earlier_than)
            .await?)
    }
}
