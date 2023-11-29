extern crate nederlandskie;

use std::env;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use dotenv::dotenv;

use nederlandskie::services::{Bluesky, Database};

#[derive(Parser, Debug)]
struct Args {
    /// Handles of the users to force the country for, comma-separated
    #[arg(long, value_delimiter(','))]
    handle: Vec<String>,

    /// DIDs of the users to force the country for, comma-separated
    #[arg(long, value_delimiter(','))]
    did: Vec<String>,

    /// Country to use, two letters
    #[arg(long)]
    country: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let args = Args::parse();

    if args.handle.is_empty() && args.did.is_empty() {
        bail!("Either --handle or --did must be supplied");
    }

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

    let bluesky = Bluesky::unauthenticated();
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

    for did in &args.did {
        database.force_profile_country(did, &args.country).await?;

        println!(
            "Stored '{}' as the country for profile with did '{}'",
            args.country, did
        );
    }

    Ok(())
}
