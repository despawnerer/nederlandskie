use crate::config::Config;
use crate::services::database::Database;

#[derive(Clone)]
pub struct FeedServerState {
    pub database: Database,
    pub config: Config,
}
