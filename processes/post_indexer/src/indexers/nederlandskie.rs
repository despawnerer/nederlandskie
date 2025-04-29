use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use lingua::Language::Russian;
use lingua::LanguageDetector;

use super::Indexer;

use nederlandskie_core::services::bluesky;
use nederlandskie_core::services::database::Database;

/// An indexer that indexes posts that are either in Russian, or made by profiles residing in Netherlands
pub struct NederlandskieIndexer {
    language_detector: Arc<LanguageDetector>,
    database: Arc<Database>,
}

impl NederlandskieIndexer {
    pub fn new(language_detector: Arc<LanguageDetector>, database: Arc<Database>) -> Self {
        Self {
            language_detector,
            database,
        }
    }
}

impl NederlandskieIndexer {
    fn is_post_in_russian(&self, post: &bluesky::PostRecord) -> bool {
        self.language_detector.detect_language_of(&post.text) == Some(Russian)
    }

    async fn is_profile_residing_in_netherlands(&self, did: &str) -> Result<bool> {
        Ok(self.database.is_profile_in_this_country(did, "nl").await? == Some(true))
    }
}

#[async_trait]
impl Indexer for NederlandskieIndexer {
    async fn should_index_post(
        &self,
        author_did: &str,
        post: &bluesky::PostRecord,
    ) -> Result<bool> {
        Ok(self.is_post_in_russian(post)
            || self.is_profile_residing_in_netherlands(author_did).await?)
    }
}
