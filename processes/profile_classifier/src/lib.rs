pub mod metrics;

use std::time::Duration;

use anyhow::{Context, Result};
use log::{error, info};

use nederlandskie_core::services::{AI, Bluesky, Database};

pub struct ProfileClassifier {
    database: Database,
    ai: AI,
    bluesky: Bluesky,
}

impl ProfileClassifier {
    pub fn new(database: Database, ai: AI, bluesky: Bluesky) -> Self {
        Self {
            database,
            ai,
            bluesky,
        }
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting");

        loop {
            if let Err(e) = self.classify_unclassified_profiles().await {
                metrics::profiles_classification_failed("fetch_profiles");
                error!("Problem with classifying profiles: {}", e)
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    async fn classify_unclassified_profiles(&self) -> Result<()> {
        // TODO: Maybe streamify this so that each thing is processed in parallel

        let dids = self.database.fetch_unprocessed_profile_dids().await?;

        metrics::profiles_pending(dids.len());

        if dids.is_empty() {
            info!("No profiles to process");
        } else {
            info!("Classifying {} new profiles", dids.len());
            for did in &dids {
                match self.fill_in_profile_details(did).await {
                    Ok(()) => {
                        metrics::profiles_classified();
                    }
                    Err(e) => error!("Could not classify profile with did {}: {:?}", did, e),
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        Ok(())
    }

    async fn fill_in_profile_details(&self, did: &str) -> Result<()> {
        let details = self
            .bluesky
            .fetch_profile_details(did)
            .await
            .inspect_err(|_| metrics::profiles_classification_failed("fetch_profile"))
            .context("Could not fetch profile details")?;

        let country = match details {
            Some(details) => self
                .ai
                .infer_country_of_living(
                    details.display_name.as_deref().unwrap_or_default(),
                    details.description.as_deref().unwrap_or_default(),
                )
                .await
                .inspect_err(|_| metrics::profiles_classification_failed("infer_country"))
                .context("Could not infer country of living")?,
            None => "xx".to_owned(),
        };

        self.database
            .store_profile_details(did, &country)
            .await
            .inspect_err(|_| metrics::profiles_classification_failed("store_profile"))?;
        info!("Stored inferred country of living for {did}: {country}");
        Ok(())
    }
}
