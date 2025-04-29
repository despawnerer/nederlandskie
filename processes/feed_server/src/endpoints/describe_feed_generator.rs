use std::sync::Arc;

use atrium_api::app::bsky::feed::describe_feed_generator::{
    FeedData, OutputData as FeedGeneratorDescription,
};
use atrium_api::types::Object;
use axum::{extract::State, Json};

use nederlandskie_core::config::Config;

use crate::feeds::Feeds;

pub async fn describe_feed_generator(
    State(config): State<Arc<Config>>,
    State(feeds): State<Arc<Feeds>>,
) -> Json<FeedGeneratorDescription> {
    Json(FeedGeneratorDescription {
        did: config.feed_generator_did.clone(),
        feeds: feeds
            .iter_names()
            .map(|name| FeedData {
                uri: format!(
                    "at://{}/app.bsky.feed.generator/{}",
                    config.publisher_did.as_ref(),
                    name
                ),
            })
            .map(Object::from)
            .collect(),
        links: None,
    })
}
