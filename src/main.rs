mod config;
mod processes;
mod services;

use anyhow::Result;

use crate::config::Config;
use crate::processes::feed_server::FeedServer;
use crate::processes::post_saver::PostSaver;
use crate::processes::profile_classifier::ProfileClassifier;
use crate::services::ai::AI;
use crate::services::bluesky::Bluesky;
use crate::services::database::Database;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let ai = AI::new(&config.chat_gpt_api_key, "https://api.openai.com");
    let bluesky = Bluesky::new("https://bsky.social");
    let database = Database::connect(&config.database_url).await?;

    let post_saver = PostSaver::new(&database, &bluesky);
    let profile_classifier = ProfileClassifier::new(&database, &ai, &bluesky);
    let feed_server = FeedServer::new(&database, &config);

    tokio::try_join!(
        post_saver.start(),
        profile_classifier.start(),
        feed_server.serve(),
    )?;

    Ok(())
}
