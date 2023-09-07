use anyhow::Result;
use async_trait::async_trait;

use crate::services::bluesky::{Bluesky, Operation, OperationProcessor};
use crate::services::database::Database;

pub struct PostSaver<'a, 'b> {
    database: &'a Database,
    bluesky: &'b Bluesky,
}

impl<'a, 'b> PostSaver<'a, 'b> {
    pub fn new(database: &'a Database, bluesky: &'b Bluesky) -> Self {
        Self { database, bluesky }
    }
}

impl<'a, 'b> PostSaver<'a, 'b> {
    pub async fn start(&self) -> Result<()> {
        Ok(self.bluesky.subscribe_to_operations(self).await?)
    }
}

#[async_trait]
impl<'a, 'b> OperationProcessor for PostSaver<'a, 'b> {
    async fn process_operation(&self, operation: &Operation) -> Result<()> {
        match operation {
            Operation::CreatePost {
                author_did,
                cid,
                uri,
                languages,
                text,
            } => {
                // TODO: Configure this via env vars
                if !languages.contains("ru") {
                    return Ok(());
                }

                // BlueSky gets confused a lot about Russian vs Ukrainian, so skip posts
                // that may be in Ukrainian regardless of whether Russian is in the list
                // TODO: Configure this via env vars
                if languages.contains("uk") {
                    return Ok(());
                }

                println!("received insertable post from {author_did}: {text}");

                self.database
                    .insert_profile_if_it_doesnt_exist(&author_did)
                    .await?;
                self.database.insert_post(&author_did, &cid, &uri).await?;
            }
            Operation::DeletePost { uri } => {
                println!("received a post do delete: {uri}");

                // TODO: Delete posts from db
                // self.database.delete_post(&self.db_connection_pool, &uri).await?;
            }
        };

        Ok(())
    }
}
