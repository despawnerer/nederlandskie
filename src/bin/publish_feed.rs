extern crate nederlandskie;

use std::env;

use anyhow::{Context, Result};
use clap::Parser;
use dotenv::dotenv;

use nederlandskie::services::Bluesky;

#[derive(Parser, Debug)]
struct Args {
    /// Short name of the feed. Must match one of the defined algos.
    #[arg(long)]
    name: String,

    /// Name that will be displayed in Bluesky interface
    #[arg(long)]
    display_name: String,

    /// Description that will be displayed in Bluesky interface
    #[arg(long)]
    description: String,

    /// Filename of the avatar that will be displayed
    #[arg(long)]
    avatar_filename: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let args = Args::parse();

    let handle = env::var("PUBLISHER_BLUESKY_HANDLE")
        .context("PUBLISHER_BLUESKY_HANDLE environment variable must be set")?;

    let password = env::var("PUBLISHER_BLUESKY_PASSWORD")
        .context("PUBLISHER_BLUESKY_PASSWORD environment variable must be set")?;

    let feed_generator_did = format!("did:web:{}", env::var("FEED_GENERATOR_HOSTNAME")?);

    let bluesky = Bluesky::new("https://bsky.social");

    let session = bluesky.login(&handle, &password).await?;

    let mut avatar = None;
    if let Some(path) = args.avatar_filename {
        let bytes = std::fs::read(path)?;
        avatar = Some(bluesky.upload_blob(bytes).await?);
    }

    bluesky
        .publish_feed(
            &session.did,
            &feed_generator_did,
            &args.name,
            &args.display_name,
            &args.description,
            avatar,
        )
        .await?;

    Ok(())
}
