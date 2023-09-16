mod nederlandskie;

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

use crate::services::database::{Database, Post};

use self::nederlandskie::Nederlandskie;

#[async_trait]
pub trait Algo {
    fn should_index_post(&self, author_did: &str, languages: &HashSet<String>, text: &str) -> bool;

    async fn fetch_posts(
        &self,
        database: &Database,
        limit: i32,
        earlier_than: Option<(DateTime<Utc>, &str)>,
    ) -> Result<Vec<Post>>;
}

pub type AnyAlgo = Box<dyn Algo + Sync + Send>;
pub type AlgosMap = HashMap<&'static str, AnyAlgo>;

static ALL_ALGOS: Lazy<AlgosMap> = Lazy::new(|| {
    let mut m = AlgosMap::new();
    m.insert("nederlandskie", Box::new(Nederlandskie));
    m
});

pub fn iter_names() -> impl Iterator<Item = &'static str> {
    ALL_ALGOS.keys().map(|s| *s)
}

pub fn iter_all() -> impl Iterator<Item = &'static AnyAlgo> {
    ALL_ALGOS.values()
}

pub fn get_by_name(name: &str) -> Option<&'static AnyAlgo> {
    ALL_ALGOS.get(name)
}
