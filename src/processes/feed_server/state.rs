use std::sync::Arc;

use axum::extract::FromRef;

use crate::algos::Algos;
use crate::config::Config;
use crate::services::Database;

#[derive(Clone)]
pub struct FeedServerState {
    pub database: Arc<Database>,
    pub config: Arc<Config>,
    pub algos: Arc<Algos>,
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

impl FromRef<FeedServerState> for Arc<Algos> {
    fn from_ref(state: &FeedServerState) -> Arc<Algos> {
        state.algos.clone()
    }
}
