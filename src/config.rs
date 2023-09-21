use anyhow::Result;
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub chat_gpt_api_key: String,
    pub database_url: String,
    pub service_did: String,
    pub publisher_did: String,
    pub hostname: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv()?;

        Ok(Self {
            chat_gpt_api_key: env::var("CHAT_GPT_API_KEY")?,
            database_url: env::var("DATABASE_URL")?,
            hostname: env::var("HOSTNAME")?,
            service_did: format!("did:web:{}", env::var("HOSTNAME")?),
            publisher_did: "".to_owned(), // TODO
        })
    }
}
