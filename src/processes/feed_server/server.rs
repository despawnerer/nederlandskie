use std::net::SocketAddr;

use anyhow::Result;
use axum::routing::get;
use axum::{Router, Server};

use crate::config::Config;
use crate::services::Database;

use super::endpoints::{describe_feed_generator, did_json, get_feed_skeleton, root};
use super::state::FeedServerState;

pub struct FeedServer<'a> {
    database: &'a Database,
    config: &'a Config,
}

impl<'a> FeedServer<'a> {
    pub fn new(database: &'a Database, config: &'a Config) -> Self {
        Self { database, config }
    }

    pub async fn serve(self) -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/.well-known/did.json", get(did_json))
            .route(
                "/xrpc/app.bsky.feed.describeFeedGenerator",
                get(describe_feed_generator),
            )
            .route(
                "/xrpc/app.bsky.feed.getFeedSkeleton",
                get(get_feed_skeleton),
            )
            .with_state(FeedServerState {
                database: self.database.clone(),
                config: self.config.clone(),
            });

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        Server::bind(&addr).serve(app.into_make_service()).await?;
        Ok(())
    }
}
