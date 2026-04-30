use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use env_logger::Env;
use log::info;
use metrics_exporter_prometheus::PrometheusBuilder;

use nederlandskie_core::config::Config;
use nederlandskie_core::services::Database;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Loading configuration");

    let config = Config::load()?;

    if config.metrics_enabled {
        PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], 9094))
            .install()
            .expect("failed to install metrics exporter");
    }

    info!("Connecting to the database");

    let database = Database::connect(&config.database_url).await?;

    loop {
        let now: DateTime<Utc> = Utc::now();
        let earlier_than = now - TimeDelta::days(150);

        info!("Deleting posts older than {}", earlier_than);

        let deleted_posts = database.delete_old_posts(&earlier_than).await?;

        if deleted_posts > 0 {
            metrics::counter!("posts_janitor_deleted_total").increment(deleted_posts as u64);
            info!("Deleted {}", deleted_posts);
        } else {
            info!("No posts to delete, waiting...");
        }

        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}
