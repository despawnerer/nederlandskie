extern crate nederlandskie;

use std::env;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dotenv::dotenv;

use nederlandskie::services::{Bluesky, Database};

#[derive(Parser, Debug)]
struct Args {
    /// Handles of the users to force the country for, comma-separated
    #[arg(long, required(true), value_delimiter(','))]
    handle: Vec<String>,

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
    let database = Database::connect(&database_url).await?;

    for handle in &args.handle {
        let did = bluesky
            .resolve_handle(handle)
            .await?
            .ok_or_else(|| anyhow!("No such user: {}", handle))?;

        println!("Resolved handle '{}' to did '{}'", handle, did);

        database.force_profile_country(&did, &args.country).await?;

        println!(
            "Stored '{}' as the country for profile with did '{}'",
            args.country, did
        );
    }

    Ok(())
}
