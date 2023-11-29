use std::sync::Arc;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use log::info;

use crate::algos::Algos;
use crate::config::Config;
use crate::services::Database;

use super::endpoints::{describe_feed_generator, did_json, get_feed_skeleton, root};
use super::state::FeedServerState;

pub struct FeedServer {
    database: Arc<Database>,
    config: Arc<Config>,
    algos: Arc<Algos>,
}

impl FeedServer {
    pub fn new(database: Arc<Database>, config: Arc<Config>, algos: Arc<Algos>) -> Self {
        Self {
            database,
            config,
            algos,
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
                algos: self.algos,
            });

        let addr = "0.0.0.0:3030";
        info!("Serving feed on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
