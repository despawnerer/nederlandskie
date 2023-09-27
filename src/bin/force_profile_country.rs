extern crate nederlandskie;

use std::env;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dotenv::dotenv;

use nederlandskie::services::{Bluesky, Database};

#[derive(Parser, Debug)]
struct Args {
    /// Handle of the user to force the country for
    #[arg(long)]
    handle: String,

    /// Country to use, two letters
    #[arg(long)]
    country: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let args = Args::parse();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

    let bluesky = Bluesky::new("https://bsky.social");

    let did = bluesky
        .resolve_handle(&args.handle)
        .await?
        .ok_or_else(|| anyhow!("No such user: {}", args.handle))?;

    println!("Resolved handle '{}' to did '{}'", args.handle, did);

    let database = Database::connect(&database_url).await?;

    database.force_profile_country(&did, &args.country).await?;

    println!(
        "Stored '{}' as the country for profile with did '{}'",
        args.country, did
    );

    Ok(())
}
