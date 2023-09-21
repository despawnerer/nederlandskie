use std::sync::Arc;

use crate::algos::Algos;
use crate::config::Config;
use crate::services::Database;

#[derive(Clone)]
pub struct FeedServerState {
    pub database: Arc<Database>,
    pub config: Arc<Config>,
    pub algos: Arc<Algos>,
}
