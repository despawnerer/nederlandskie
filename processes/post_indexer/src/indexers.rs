mod nederlandskie;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use lingua::LanguageDetector;

use nederlandskie_core::services::bluesky;
use nederlandskie_core::services::database::Database;

pub use self::nederlandskie::NederlandskieIndexer;

#[async_trait]
pub trait Indexer {
    async fn should_index_post(&self, author_did: &str, post: &bluesky::PostRecord)
        -> Result<bool>;
}

pub fn initialize_all_indexers(
    language_detector: Arc<LanguageDetector>,
    database: Arc<Database>,
) -> Indexers {
    IndexersBuilder::new()
        .add(
            "nederlandskie",
            NederlandskieIndexer::new(language_detector, database),
        )
        .build()
}

pub type AnyIndexer = Box<dyn Indexer + Sync + Send>;
type IndexersMap = HashMap<String, AnyIndexer>;

pub struct Indexers {
    indexers: IndexersMap,
}

impl Indexers {
    pub fn iter_names(&self) -> impl Iterator<Item = &str> {
        self.indexers.keys().map(String::as_str)
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &AnyIndexer> {
        self.indexers.values()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&AnyIndexer> {
        self.indexers.get(name)
    }
}

#[derive(Default)]
pub struct IndexersBuilder {
    indexers: IndexersMap,
}

impl IndexersBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<T: Indexer + Send + Sync + 'static>(mut self, name: &str, indexer: T) -> Self {
        self.indexers.insert(name.to_owned(), Box::new(indexer));
        self
    }

    pub fn build(self) -> Indexers {
        Indexers {
            indexers: self.indexers,
        }
    }
}
