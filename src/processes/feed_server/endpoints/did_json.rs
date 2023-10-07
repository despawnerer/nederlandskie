use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::config::Config;

#[derive(Serialize)]
pub struct Did {
    #[serde(rename = "@context")]
    context: Vec<String>,
    id: String,
    service: Vec<Service>,
}

#[derive(Serialize)]
pub struct Service {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "serviceEndpoint")]
    service_endpoint: String,
}

pub async fn did_json(State(config): State<Arc<Config>>) -> Json<Did> {
    Json(Did {
        context: vec!["https://www.w3.org/ns/did/v1".to_owned()],
        id: config.feed_generator_did.clone(),
        service: vec![Service {
            id: "#bsky_fg".to_owned(),
            type_: "BskyFeedGenerator".to_owned(),
            service_endpoint: format!("https://{}", config.feed_generator_hostname),
        }],
    })
}
