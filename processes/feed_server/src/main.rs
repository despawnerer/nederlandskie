extern crate nederlandskie_feed_server;

use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use log::info;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::Database;
use nederlandskie_feed_server::{feeds::initialize_all_feeds, FeedServer};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Loading configuration");

    let config = Arc::new(Config::load()?);

    info!("Connecting to the database");

    let database = Arc::new(Database::connect(&config.database_url).await?);

    info!("Initializing feeds");

    let feeds = Arc::new(initialize_all_feeds());

    let feed_server = FeedServer::new(database.clone(), config.clone(), feeds.clone());

    info!("Starting Feed Server");

    feed_server.serve().await
}
