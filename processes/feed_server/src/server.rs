use std::sync::Arc;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use log::info;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::Database;

use super::endpoints::{describe_feed_generator, did_json, get_feed_skeleton, root};
use super::feeds::Feeds;
use super::state::FeedServerState;

pub struct FeedServer {
    database: Arc<Database>,
    config: Arc<Config>,
    feeds: Arc<Feeds>,
}

impl FeedServer {
    pub fn new(database: Arc<Database>, config: Arc<Config>, feeds: Arc<Feeds>) -> Self {
        Self {
            database,
            config,
            feeds,
        }
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
                database: self.database,
                config: self.config,
                feeds: self.feeds,
            });

        let addr = "0.0.0.0:3030";
        info!("Serving feed on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
