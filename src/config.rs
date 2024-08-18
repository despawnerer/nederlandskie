use anyhow::Result;
use atrium_api::types::string::Did;
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub chat_gpt_api_key: String,
    pub database_url: String,
    pub feed_generator_did: Did,
    pub publisher_did: Did,
    pub feed_generator_hostname: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv()?;

        Ok(Self {
            chat_gpt_api_key: env::var("CHAT_GPT_API_KEY")?,
            database_url: env::var("DATABASE_URL")?,
            feed_generator_hostname: env::var("FEED_GENERATOR_HOSTNAME")?,
            feed_generator_did: format!("did:web:{}", env::var("FEED_GENERATOR_HOSTNAME")?)
                .parse()
                .map_err(anyhow::Error::msg)?,
            publisher_did: env::var("PUBLISHER_DID")?
                .parse()
                .map_err(anyhow::Error::msg)?,
        })
    }
}
