use axum::{extract::State, Json};
use serde::Serialize;

use crate::processes::feed_server::state::FeedServerState;

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
    service_endpoint: String,
}

pub async fn did_json(State(state): State<FeedServerState>) -> Json<Did> {
    Json(Did {
        context: vec!["https://www.w3.org/ns/did/v1".to_owned()],
        id: state.config.service_did.clone(),
        service: vec![Service {
            id: "#bsky_fg".to_owned(),
            type_: "BskyFeedGenerator".to_owned(),
            service_endpoint: format!("https://{}", state.config.hostname),
        }],
    })
}
