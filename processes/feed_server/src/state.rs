use std::sync::Arc;

use axum::extract::FromRef;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::Database;

use super::feeds::Feeds;

#[derive(Clone)]
pub struct FeedServerState {
    pub database: Arc<Database>,
    pub config: Arc<Config>,
    pub feeds: Arc<Feeds>,
}

impl FromRef<FeedServerState> for Arc<Database> {
    fn from_ref(state: &FeedServerState) -> Arc<Database> {
        state.database.clone()
    }
}

impl FromRef<FeedServerState> for Arc<Config> {
    fn from_ref(state: &FeedServerState) -> Arc<Config> {
        state.config.clone()
    }
}

impl FromRef<FeedServerState> for Arc<Feeds> {
    fn from_ref(state: &FeedServerState) -> Arc<Feeds> {
        state.feeds.clone()
    }
}
