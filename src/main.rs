mod algos;
mod config;
mod processes;
mod services;

use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use lingua::LanguageDetectorBuilder;
use log::info;

use crate::algos::AlgosBuilder;
use crate::algos::Nederlandskie;
use crate::config::Config;
use crate::processes::FeedServer;
use crate::processes::PostIndexer;
use crate::processes::ProfileClassifier;
use crate::services::Bluesky;
use crate::services::Database;
use crate::services::AI;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Loading configuration");

    let config = Arc::new(Config::load()?);

    info!("Initializing service clients");

    let ai = Arc::new(AI::new(&config.chat_gpt_api_key, "https://api.openai.com"));
    let bluesky = Arc::new(Bluesky::new("https://bsky.social"));
    let database = Arc::new(Database::connect(&config.database_url).await?);

    info!("Initializing language detector");

    let language_detector = Arc::new(
        LanguageDetectorBuilder::from_all_languages()
            .with_preloaded_language_models()
            .build(),
    );

    let algos = Arc::new(
        AlgosBuilder::new()
            .add("nederlandskie", Nederlandskie::new(language_detector))
            .build(),
    );

    let post_indexer = PostIndexer::new(
        database.clone(),
        bluesky.clone(),
        algos.clone(),
        config.clone(),
    );
    let profile_classifier = ProfileClassifier::new(database.clone(), ai.clone(), bluesky.clone());
    let feed_server = FeedServer::new(database.clone(), config.clone(), algos.clone());

    info!("Starting everything up");

    tokio::try_join!(
        post_indexer.start(),
        profile_classifier.start(),
        feed_server.serve(),
    )?;

    Ok(())
}
