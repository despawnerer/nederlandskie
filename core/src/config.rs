use anyhow::Result;
use atrium_api::types::string::Did;
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub anthropic_api_key: String,
    pub database_url: String,
    pub feed_generator_did: Did,
    pub publisher_did: Did,
    pub feed_generator_hostname: String,
    pub metrics_enabled: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv()?;

        Ok(Self {
            anthropic_api_key: env::var("ANTHROPIC_API_KEY")?,
            database_url: env::var("DATABASE_URL")?,
            feed_generator_hostname: env::var("FEED_GENERATOR_HOSTNAME")?,
            feed_generator_did: format!("did:web:{}", env::var("FEED_GENERATOR_HOSTNAME")?)
                .parse()
                .map_err(anyhow::Error::msg)?,
            publisher_did: env::var("PUBLISHER_DID")?
                .parse()
                .map_err(anyhow::Error::msg)?,
            metrics_enabled: env::var("METRICS_ENABLED")
                .map(|v| v != "false")
                .unwrap_or(true),
        })
    }
}
