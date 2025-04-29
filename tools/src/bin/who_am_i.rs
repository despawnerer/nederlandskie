use std::env;

use anyhow::{Context, Result};
use dotenv::dotenv;

use nederlandskie_core::services::Bluesky;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let handle = env::var("PUBLISHER_BLUESKY_HANDLE")
        .context("PUBLISHER_BLUESKY_HANDLE environment variable must be set")?;

    let password = env::var("PUBLISHER_BLUESKY_PASSWORD")
        .context("PUBLISHER_BLUESKY_PASSWORD environment variable must be set")?;

    let bluesky = Bluesky::login(&handle, &password).await?;

    let did = bluesky
        .resolve_handle(&handle)
        .await?
        .expect("couldn't resolve our own handle, huh?");

    println!("{}", did);

    Ok(())
}
