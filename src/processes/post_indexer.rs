use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::info;

use crate::algos::Algos;
use crate::services::bluesky::{Bluesky, Operation, OperationProcessor};
use crate::services::Database;

pub struct PostIndexer {
    database: Arc<Database>,
    bluesky: Arc<Bluesky>,
    algos: Arc<Algos>,
}

impl PostIndexer {
    pub fn new(database: Arc<Database>, bluesky: Arc<Bluesky>, algos: Arc<Algos>) -> Self {
        Self {
            database,
            bluesky,
            algos,
        }
    }
}

impl PostIndexer {
    pub async fn start(&self) -> Result<()> {
        info!("Starting");
        Ok(self.bluesky.subscribe_to_operations(self).await?)
    }
}

#[async_trait]
impl OperationProcessor for PostIndexer {
    async fn process_operation(&self, operation: &Operation) -> Result<()> {
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

        Ok(())
    }
}
