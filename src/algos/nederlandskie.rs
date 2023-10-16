use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lingua::Language::Russian;
use lingua::LanguageDetector;

use super::Algo;

use crate::services::bluesky;
use crate::services::database::{self, Database};

/// An algorithm that serves posts written in Russian by people living in Netherlands
pub struct Nederlandskie {
    language_detector: Arc<LanguageDetector>,
    database: Arc<Database>,
}

impl Nederlandskie {
    pub fn new(language_detector: Arc<LanguageDetector>, database: Arc<Database>) -> Self {
        Self {
            language_detector,
            database,
        }
    }
}

impl Nederlandskie {
    fn is_post_in_russian(&self, post: &bluesky::PostRecord) -> bool {
        self.language_detector.detect_language_of(&post.text) == Some(Russian)
    }

    async fn is_profile_residing_in_netherlands(&self, did: &str) -> Result<bool> {
        Ok(self.database.is_profile_in_this_country(did, "nl").await? == Some(true))
    }
}

#[async_trait]
impl Algo for Nederlandskie {
    async fn should_index_post(
        &self,
        author_did: &str,
        post: &bluesky::PostRecord,
    ) -> Result<bool> {
        Ok(self.is_post_in_russian(&post)
            || self.is_profile_residing_in_netherlands(author_did).await?)
    }

    async fn fetch_posts(
        &self,
        database: &Database,
        limit: i32,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<database::Post>> {
        Ok(database
            .fetch_posts_by_authors_country("nl", limit as usize, earlier_than)
            .await?)
    }
}
