use std::sync::Arc;

use atrium_api::app::bsky::feed::describe_feed_generator::{
    Feed, Output as FeedGeneratorDescription,
};
use axum::{extract::State, Json};

use crate::{algos::Algos, config::Config};

pub async fn describe_feed_generator(
    State(config): State<Arc<Config>>,
    State(algos): State<Arc<Algos>>,
) -> Json<FeedGeneratorDescription> {
    Json(FeedGeneratorDescription {
        did: config.feed_generator_did.clone(),
        feeds: algos
            .iter_names()
            .map(|name| Feed {
                uri: format!(
                    "at://{}/app.bsky.feed.generator/{}",
                    config.publisher_did, name
                ),
            })
            .collect(),
        links: None,
    })
}
