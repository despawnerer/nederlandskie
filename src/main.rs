mod algos;
mod config;
mod processes;
mod services;

use anyhow::Result;

use crate::config::Config;
use crate::processes::FeedServer;
use crate::processes::PostIndexer;
use crate::processes::ProfileClassifier;
use crate::services::Bluesky;
use crate::services::Database;
use crate::services::AI;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let ai = AI::new(&config.chat_gpt_api_key, "https://api.openai.com");
    let bluesky = Bluesky::new("https://bsky.social");
    let database = Database::connect(&config.database_url).await?;

    let post_indexer = PostIndexer::new(&database, &bluesky);
    let profile_classifier = ProfileClassifier::new(&database, &ai, &bluesky);
    let feed_server = FeedServer::new(&database, &config);

    tokio::try_join!(
        post_indexer.start(),
        profile_classifier.start(),
        feed_server.serve(),
    )?;

    Ok(())
}
