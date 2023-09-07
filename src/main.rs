mod processes;
mod services;

use std::env;

use anyhow::Result;
use dotenv::dotenv;

use crate::processes::post_saver::PostSaver;
use crate::processes::profile_classifier::ProfileClassifier;
use crate::services::ai::AI;
use crate::services::bluesky::Bluesky;
use crate::services::database::Database;

struct Config {
    chat_gpt_api_key: String,
    database_url: String,
}

impl Config {
    fn load() -> Result<Self> {
        dotenv()?;

        Ok(Self {
            chat_gpt_api_key: env::var("CHAT_GPT_API_KEY")?,
            database_url: env::var("DATABASE_URL")?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let ai = AI::new(&config.chat_gpt_api_key, "https://api.openai.com");
    let bluesky = Bluesky::new("https://bsky.social");
    let database = Database::connect(&config.database_url).await?;

    let post_saver = PostSaver::new(&database, &bluesky);
    let profile_classifier = ProfileClassifier::new(&database, &ai, &bluesky);

    tokio::try_join!(post_saver.start(), profile_classifier.start())?;

    Ok(())
}
