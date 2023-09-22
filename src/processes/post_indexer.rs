use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::info;

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

                    // TODO: Delete posts from db
                    // self.database.delete_post(&self.db_connection_pool, &uri).await?;
                }
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
