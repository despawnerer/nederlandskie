mod algos;
mod config;
mod processes;
mod services;

use std::sync::Arc;

use anyhow::Result;
use lingua::LanguageDetectorBuilder;

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
    let config = Arc::new(Config::load()?);

    let ai = Arc::new(AI::new(&config.chat_gpt_api_key, "https://api.openai.com"));
    let bluesky = Arc::new(Bluesky::new("https://bsky.social"));
    let database = Arc::new(Database::connect(&config.database_url).await?);
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

    let post_indexer = PostIndexer::new(database.clone(), bluesky.clone(), algos.clone());
    let profile_classifier = ProfileClassifier::new(database.clone(), ai.clone(), bluesky.clone());
    let feed_server = FeedServer::new(database.clone(), config.clone(), algos.clone());

    tokio::try_join!(
        post_indexer.start(),
        profile_classifier.start(),
        feed_server.serve(),
    )?;

    Ok(())
}
