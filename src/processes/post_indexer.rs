use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};

use crate::algos::Algos;
use crate::config::Config;
use crate::services::bluesky::{Bluesky, CommitDetails, CommitProcessor, Operation};
use crate::services::Database;

pub struct PostIndexer {
    database: Arc<Database>,
    bluesky: Arc<Bluesky>,
    algos: Arc<Algos>,
    config: Arc<Config>,
}

impl PostIndexer {
    pub fn new(
        database: Arc<Database>,
        bluesky: Arc<Bluesky>,
        algos: Arc<Algos>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            database,
            bluesky,
            algos,
            config,
        }
    }
}

impl PostIndexer {
    pub async fn start(&self) -> Result<()> {
        info!("Starting");

        loop {
            if let Err(e) = self.process_from_last_point().await {
                error!("Stopped because of an error: {}", e);
            }

            info!("Waiting 10 seconds before reconnecting...");

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    async fn process_from_last_point(&self) -> Result<()> {
        let cursor = self
            .database
            .fetch_subscription_cursor(&self.config.feed_generator_did)
            .await?;

        if cursor.is_none() {
            self.database
                .create_subscription_state(&self.config.feed_generator_did)
                .await?;
        }

        info!("Subscribing with cursor {:?}", cursor);

        self.bluesky.subscribe_to_operations(self, cursor).await
    }
}

#[async_trait]
impl CommitProcessor for PostIndexer {
    async fn process_commit(&self, commit: &CommitDetails) -> Result<()> {
        for operation in &commit.operations {
            match operation {
                Operation::CreatePost {
                    author_did,
                    cid,
                    uri,
                    languages,
                    text,
                } => {
                    for algo in self.algos.iter_all() {
                        if algo.should_index_post(author_did, languages, text).await? {
                            info!("Received insertable post from {author_did}: {text}");

                            self.database
                                .insert_profile_if_it_doesnt_exist(author_did)
                                .await?;

                            self.database.insert_post(author_did, cid, uri).await?;

                            break;
                        }
                    }
                }
                Operation::DeletePost { uri } => {
                    info!("Received a post to delete: {uri}");

                    self.database.delete_post(uri).await?;
                }
                _ => continue,
            }
        }

        if commit.seq % 20 == 0 {
            info!(
                "Updating cursor for {} to {}",
                self.config.feed_generator_did, commit.seq
            );
            self.database
                .update_subscription_cursor(&self.config.feed_generator_did, commit.seq)
                .await?;
        }

        Ok(())
    }
}
