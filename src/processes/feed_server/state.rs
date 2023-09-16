use crate::config::Config;
use crate::services::Database;

#[derive(Clone)]
pub struct FeedServerState {
    pub database: Database,
    pub config: Config,
}
