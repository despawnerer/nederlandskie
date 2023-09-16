use anyhow::{anyhow, Result};
use atrium_api::app::bsky::feed::defs::SkeletonFeedPost;
use atrium_api::app::bsky::feed::get_feed_skeleton::{
    Output as FeedSkeleton, Parameters as FeedSkeletonQuery,
};
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, TimeZone, Utc};

use crate::processes::feed_server::state::FeedServerState;

pub async fn get_feed_skeleton(
    State(state): State<FeedServerState>,
    query: Query<FeedSkeletonQuery>,
) -> Json<FeedSkeleton> {
    let limit = query.limit.unwrap_or(20) as usize;
    let earlier_than = query
        .cursor
        .as_deref()
        .map(parse_cursor)
        .transpose()
        .unwrap(); // TODO: handle error

    let posts = state
        .database
        .fetch_posts_by_authors_country("ru", limit, earlier_than)
        .await
        .unwrap();

    let feed = posts
        .iter()
        .map(|p| SkeletonFeedPost {
            post: p.uri.clone(),
            reason: None,
        })
        .collect();

    let cursor = posts.last().map(|p| make_cursor(&p.indexed_at, &p.cid));

    Json(FeedSkeleton { cursor, feed })
}

fn make_cursor(date: &DateTime<Utc>, cid: &str) -> String {
    format!("{}::{}", date.timestamp() * 1000, cid)
}

fn parse_cursor(cursor: &str) -> Result<(DateTime<Utc>, &str)> {
    let mut parts = cursor.split("::");

    let indexed_at = parts.next().ok_or_else(|| anyhow!("Malformed cursor"))?;
    let cid = parts.next().ok_or_else(|| anyhow!("Malformed cursor"))?;

    if parts.next().is_some() {
        return Err(anyhow!("Malformed cursor"));
    }

    let indexed_at: i64 = indexed_at.parse()?;
    let indexed_at = Utc.timestamp_opt(indexed_at / 1000, 0).unwrap(); // TODO: handle error

    Ok((indexed_at, cid))
}
