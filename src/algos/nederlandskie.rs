use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::Algo;

use crate::services::{database::Post, Database};

pub struct Nederlandskie;

/// An algorithm that serves posts written in Russian by people living in Netherlands
#[async_trait]
impl Algo for Nederlandskie {
    fn should_index_post(
        &self,
        _author_did: &str,
        languages: &HashSet<String>,
        _text: &str,
    ) -> bool {
        // BlueSky gets confused a lot about Russian vs Ukrainian, so skip posts
        // that may be in Ukrainian regardless of whether Russian is in the list
        languages.contains("ru") && !languages.contains("uk")
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
