extern crate nederlandskie;

use std::env;

use anyhow::{Context, Result};
use dotenv::dotenv;

use nederlandskie::services::Bluesky;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let bluesky = Bluesky::new("https://bsky.social");

    let handle = env::var("PUBLISHER_BLUESKY_HANDLE")
        .context("PUBLISHER_BLUESKY_HANDLE environment variable must be set")?;

    let password = env::var("PUBLISHER_BLUESKY_PASSWORD")
        .context("PUBLISHER_BLUESKY_PASSWORD environment variable must be set")?;

    let session = bluesky.login(&handle, &password).await?;

    println!("{}", session.did);

    Ok(())
}
