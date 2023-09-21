use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lingua::Language::Russian;
use lingua::LanguageDetector;

use super::Algo;

use crate::services::{database::Post, Database};

pub struct Nederlandskie {
    language_detector: Arc<LanguageDetector>,
}

impl Nederlandskie {
    pub fn new(language_detector: Arc<LanguageDetector>) -> Self {
        Self { language_detector }
    }
}

/// An algorithm that serves posts written in Russian by people living in Netherlands
#[async_trait]
impl Algo for Nederlandskie {
    async fn should_index_post(
        &self,
        _author_did: &str,
        _languages: &HashSet<String>,
        text: &str,
    ) -> Result<bool> {
        Ok(self.language_detector.detect_language_of(text) == Some(Russian))
    }

    async fn fetch_posts(
        &self,
        database: &Database,
        limit: i32,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<Post>> {
        Ok(database
            .fetch_posts_by_authors_country("nl", limit as usize, earlier_than)
            .await?)
    }
}
