use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{error, info};

use crate::services::Bluesky;
use crate::services::Database;
use crate::services::AI;

pub struct ProfileClassifier {
    database: Arc<Database>,
    ai: Arc<AI>,
    bluesky: Arc<Bluesky>,
}

impl ProfileClassifier {
    pub fn new(database: Arc<Database>, ai: Arc<AI>, bluesky: Arc<Bluesky>) -> Self {
        Self {
            database,
            ai,
            bluesky,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting");
        loop {
            // TODO: Don't just exit this function when an error happens, just wait a minute or so?
            self.classify_unclassified_profiles().await?;
        }
    }

    async fn classify_unclassified_profiles(&self) -> Result<()> {
        // TODO: Maybe streamify this so that each thing is processed in parallel

        let dids = self.database.fetch_unprocessed_profile_dids().await?;
        if dids.is_empty() {
            info!("No profiles to process: waiting 10 seconds");
        } else {
            info!("Classifying {} new profiles", dids.len());
            for did in &dids {
                match self.fill_in_profile_details(did).await {
                    Ok(()) => continue,
                    Err(e) => error!("Could not classify profile: {:?}", e)
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;

        Ok(())
    }

    async fn fill_in_profile_details(&self, did: &str) -> Result<()> {
        let details = self.bluesky.fetch_profile_details(did).await?;
        let country = self
            .ai
            .infer_country_of_living(&details.display_name, &details.description)
            .await?;
        self.database.store_profile_details(did, &country).await?;
        info!("Stored inferred country of living for {did}: {country}");
        Ok(())
    }
}
