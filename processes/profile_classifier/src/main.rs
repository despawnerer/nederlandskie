extern crate nederlandskie_profile_classifier;

use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use log::info;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::{Bluesky, Database, AI};

use nederlandskie_profile_classifier::ProfileClassifier;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Loading configuration");

    let config = Arc::new(Config::load()?);

    info!("Initializing service clients");

    let ai = Arc::new(AI::new(&config.chat_gpt_api_key, "https://api.openai.com"));
    let bluesky = Arc::new(Bluesky::unauthenticated());

    info!("Connecting to the database");
    let database = Arc::new(Database::connect(&config.database_url).await?);

    let profile_classifier = ProfileClassifier::new(database.clone(), ai.clone(), bluesky.clone());

    info!("Starting Profile Classifier");

    profile_classifier.start().await
}
