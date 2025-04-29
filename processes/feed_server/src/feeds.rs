mod nederlandskie;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use nederlandskie_core::services::database::{self, Database};

pub use self::nederlandskie::NederlandskieFeed;

#[async_trait]
pub trait Feed {
    async fn fetch_posts(
        &self,
        database: &Database,
        limit: u8,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<database::Post>>;
}

pub fn initialize_all_feeds() -> Feeds {
    FeedsBuilder::new()
        .add("nederlandskie", NederlandskieFeed::new())
        .build()
}

pub type AnyFeed = Box<dyn Feed + Sync + Send>;
type FeedsMap = HashMap<String, AnyFeed>;

pub struct Feeds {
    feeds: FeedsMap,
}

impl Feeds {
    pub fn iter_names(&self) -> impl Iterator<Item = &str> {
        self.feeds.keys().map(String::as_str)
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &AnyFeed> {
        self.feeds.values()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&AnyFeed> {
        self.feeds.get(name)
    }
}

#[derive(Default)]
pub struct FeedsBuilder {
    feeds: FeedsMap,
}

impl FeedsBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<T: Feed + Send + Sync + 'static>(mut self, name: &str, feed: T) -> Self {
        self.feeds.insert(name.to_owned(), Box::new(feed));
        self
    }

    pub fn build(self) -> Feeds {
        Feeds { feeds: self.feeds }
    }
}
