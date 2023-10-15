extern crate nederlandskie;

use std::env;

use anyhow::{anyhow, Context, Result};
use dotenv::dotenv;

use nederlandskie::services::Bluesky;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let handle = env::var("PUBLISHER_BLUESKY_HANDLE")
        .context("PUBLISHER_BLUESKY_HANDLE environment variable must be set")?;

    let password = env::var("PUBLISHER_BLUESKY_PASSWORD")
        .context("PUBLISHER_BLUESKY_PASSWORD environment variable must be set")?;

    let bluesky = Bluesky::login(&handle, &password).await?;
    let session = bluesky
        .session()
        .ok_or_else(|| anyhow!("Could not log in"))?;

    println!("{}", session.did);

    Ok(())
}
