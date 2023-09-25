use anyhow::anyhow;
use atrium_api::app::bsky::feed::defs::SkeletonFeedPost;
use atrium_api::app::bsky::feed::get_feed_skeleton::{
    Output as FeedSkeleton, Parameters as FeedSkeletonQuery,
};
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, TimeZone, Utc};

use crate::processes::feed_server::state::FeedServerState;
use crate::processes::feed_server::errors::AppError;

pub async fn get_feed_skeleton(
    State(state): State<FeedServerState>,
    query: Query<FeedSkeletonQuery>,
) -> Result<Json<FeedSkeleton>, AppError> {
    let algo = state
        .algos
        .get_by_name(&query.feed)
        .ok_or_else(|| AppError::FeedNotFound(query.feed.clone()))?;

    let limit = query.limit.unwrap_or(20);
    let earlier_than = query
        .cursor
        .as_deref()
        .map(parse_cursor)
        .transpose()?;

    let posts = algo
        .fetch_posts(&state.database, limit, earlier_than)
        .await?;

    let feed = posts
        .iter()
        .map(|p| SkeletonFeedPost {
            post: p.uri.clone(),
            reason: None,
        })
        .collect();

    let cursor = posts.last().map(|p| make_cursor(&p.indexed_at, &p.cid));

    Ok(Json(FeedSkeleton { cursor, feed }))
}

fn make_cursor(date: &DateTime<Utc>, cid: &str) -> String {
    format!("{}::{}", date.timestamp() * 1000, cid)
}

fn parse_cursor(cursor: &str) -> anyhow::Result<(DateTime<Utc>, &str)> {
    let mut parts = cursor.split("::");

    let indexed_at = parts.next().ok_or_else(|| anyhow!("Malformed cursor: {cursor}"))?;
    let cid = parts.next().ok_or_else(|| anyhow!("Malformed cursor: {cursor}"))?;

    if parts.next().is_some() {
        return Err(anyhow!("Malformed cursor: {cursor}"));
    }

    let indexed_at: i64 = indexed_at.parse()?;
    let indexed_at = Utc.timestamp_opt(indexed_at / 1000, 0).unwrap(); // TODO: handle error

    Ok((indexed_at, cid))
}
