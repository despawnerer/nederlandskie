mod database;
mod frames;
mod streaming;

use anyhow::Result;
use async_trait::async_trait;

use crate::database::{
    insert_post, insert_profile_if_it_doesnt_exist, make_connection_pool, ConnectionPool,
};
use crate::streaming::{start_processing_operations_with, Operation, OperationProcessor};

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

                insert_profile_if_it_doesnt_exist(&self.db_connection_pool, &author_did).await?;
                insert_post(&self.db_connection_pool, &author_did, &cid, &uri).await?;
            }
            Operation::DeletePost { uri } => {
                println!("received a post do delete: {uri}");

                // TODO: Delete posts from db
                // delete_post(&self.db_connection_pool, &uri).await?;
            }
        };

        Ok(())
    }
}
