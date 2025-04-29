extern crate nederlandskie_post_indexer;

use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use lingua::LanguageDetectorBuilder;
use log::info;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::{Bluesky, Database};

use nederlandskie_post_indexer::{indexers::initialize_all_indexers, PostIndexer};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Loading configuration");

    let config = Arc::new(Config::load()?);

    info!("Initializing service clients");

    let bluesky = Arc::new(Bluesky::unauthenticated());
    let database = Arc::new(Database::connect(&config.database_url).await?);

    info!("Initializing language detector");

    let language_detector = Arc::new(
        LanguageDetectorBuilder::from_all_languages_with_cyrillic_script()
            .with_preloaded_language_models()
            .build(),
    );

    let indexers = Arc::new(initialize_all_indexers(
        language_detector.clone(),
        database.clone(),
    ));

    let post_indexer = PostIndexer::new(
        database.clone(),
        bluesky.clone(),
        indexers.clone(),
        config.clone(),
    );

    info!("Starting Post Indexer");

    post_indexer.start().await
}
