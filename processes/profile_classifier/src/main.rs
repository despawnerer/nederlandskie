extern crate nederlandskie_profile_classifier;

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

    let config = Config::load()?;

    info!("Initializing service clients");

    let ai = AI::new(&config.anthropic_api_key);
    let bluesky = Bluesky::unauthenticated();

    info!("Connecting to the database");
    let database = Database::connect(&config.database_url).await?;

    let profile_classifier = ProfileClassifier::new(database, ai, bluesky);

    info!("Starting Profile Classifier");

    profile_classifier.start().await
}
