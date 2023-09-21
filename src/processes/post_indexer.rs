use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::com::atproto::sync::subscribe_repos::Commit;
use log::info;

use crate::algos::Algos;
use crate::config::Config;
use crate::services::bluesky::{Bluesky, Operation, OperationProcessor};
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
            .fetch_subscription_cursor(&self.config.service_did)
            .await?;

        if cursor.is_none() {
            self.database
                .create_subscription_state(&self.config.service_did)
                .await?;
        }

        info!("Subscribing with cursor {:?}", cursor);

        Ok(self.bluesky.subscribe_to_operations(self, cursor).await?)
    }
}

#[async_trait]
impl OperationProcessor for PostIndexer {
    async fn process_operation(&self, operation: &Operation, commit: &Commit) -> Result<()> {
        match operation {
            Operation::CreatePost {
                author_did,
                cid,
                uri,
                languages,
                text,
            } => {
                if self
                    .algos
                    .iter_all()
                    .any(|a| a.should_index_post(author_did, languages, text))
                {
                    info!("Received insertable post from {author_did}: {text}");

                    self.database
                        .insert_profile_if_it_doesnt_exist(&author_did)
                        .await?;
                    self.database.insert_post(&author_did, &cid, &uri).await?;
                }
            }
            Operation::DeletePost { uri } => {
                info!("Received a post to delete: {uri}");

                // TODO: Delete posts from db
                // self.database.delete_post(&self.db_connection_pool, &uri).await?;
            }
        };

        if commit.seq % 20 == 0 {
            info!(
                "Updating cursor for {} to {}",
                self.config.service_did, commit.seq
            );
            self.database
                .update_subscription_cursor(&self.config.service_did, commit.seq)
                .await?;
        }

        Ok(())
    }
}
