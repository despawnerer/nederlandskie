mod database;
mod frames;
mod streaming;

use crate::database::ConnectionPool;
use anyhow::Result;
use async_trait::async_trait;
use database::insert_post;

use streaming::{Operation, OperationProcessor};

use crate::database::make_connection_pool;
use crate::streaming::start_processing_operations_with;

#[tokio::main]
async fn main() -> Result<()> {
    let db_connection_pool = make_connection_pool().await?;

    // FIXME: This struct shouldn't really exist, but I couldn't find a way to replace
    // this whole nonsense with a closure, which is what this whole thing should be in
    // first place.
    let post_saver = PostSaver { db_connection_pool };

    start_processing_operations_with(post_saver).await?;

    Ok(())
}

struct PostSaver {
    db_connection_pool: ConnectionPool,
}

#[async_trait]
impl OperationProcessor for PostSaver {
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

                insert_post(&self.db_connection_pool, &author_did, &cid, &uri).await?;

                // TODO: Insert profile if it doesn't exist yet
                // insert_profile_if_it_doesnt_exist(&self.db_connection_pool, &author_did).await?;
            }
            Operation::DeletePost { uri: _ } => {
                // TODO: Delete posts from db
                // delete_post(&self.db_connection_pool, &uri).await?;
            }
        };

        Ok(())
    }
}
