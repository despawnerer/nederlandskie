use std::time::Duration;

use anyhow::Result;

use crate::services::ai::AI;
use crate::services::bluesky::Bluesky;
use crate::services::database::Database;

pub struct ProfileClassifier<'a> {
    database: &'a Database,
    ai: &'a AI,
    bluesky: &'a Bluesky,
}

impl<'a> ProfileClassifier<'a> {
    pub fn new(database: &'a Database, ai: &'a AI, bluesky: &'a Bluesky) -> Self {
        Self {
            database,
            ai,
            bluesky,
        }
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            // TODO: Don't just exit this function when an error happens, just wait a minute or so?
            self.classify_unclassified_profiles().await?;
        }
    }

    async fn classify_unclassified_profiles(&self) -> Result<()> {
        // TODO: Maybe streamify this so that each thing is processed in parallel

        let dids = self.database.fetch_unprocessed_profile_dids().await?;
        if dids.is_empty() {
            println!("No profiles to process: waiting 10 seconds");
            tokio::time::sleep(Duration::from_secs(10)).await;
        } else {
            for did in &dids {
                self.fill_in_profile_details(did).await?;
            }
        }

        Ok(())
    }

    async fn fill_in_profile_details(&self, did: &str) -> Result<()> {
        let details = self.bluesky.fetch_profile_details(did).await?;
        let country = self
            .ai
            .infer_country_of_living(&details.display_name, &details.description)
            .await?;
        self.database.store_profile_details(did, &country).await?;
        println!("Stored inferred country of living for {did}: {country}");
        Ok(())
    }
}
