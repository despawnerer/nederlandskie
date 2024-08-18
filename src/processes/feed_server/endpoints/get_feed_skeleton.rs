use std::sync::Arc;

use anyhow::anyhow;
use atrium_api::app::bsky::feed::defs::SkeletonFeedPostData;
use atrium_api::app::bsky::feed::get_feed_skeleton::{
    OutputData as FeedSkeleton, ParametersData as FeedSkeletonQuery,
};
use atrium_api::types::{LimitedNonZeroU8, Object};
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, TimeZone, Utc};

use crate::algos::Algos;
use crate::processes::feed_server::errors::AppError;
use crate::services::Database;

pub async fn get_feed_skeleton(
    State(algos): State<Arc<Algos>>,
    State(database): State<Arc<Database>>,
    query: Query<FeedSkeletonQuery>,
) -> Result<Json<FeedSkeleton>, AppError> {
    let feed_name = query
        .feed
        .split('/')
        .last()
        .ok_or_else(|| anyhow!("Invalid feed URI"))?;

    let algo = algos
        .get_by_name(feed_name)
        .ok_or_else(|| AppError::FeedNotFound(feed_name.to_owned()))?;

    let limit = query
        .limit
        .unwrap_or(LimitedNonZeroU8::try_from(20).expect("this default limit should always work"));
    let earlier_than = query.cursor.as_deref().map(parse_cursor).transpose()?;

    let posts = algo
        .fetch_posts(&database, limit.into(), earlier_than)
        .await?;

    let feed = posts
        .iter()
        .map(|p| SkeletonFeedPostData {
            post: p.uri.clone(),
            feed_context: None,
            reason: None,
        })
        .map(Object::from)
        .collect();

    let cursor = posts.last().map(|p| make_cursor(&p.indexed_at, &p.cid));

    Ok(Json(FeedSkeleton { cursor, feed }))
}

fn make_cursor(date: &DateTime<Utc>, cid: &str) -> String {
    format!("{}::{}", date.timestamp() * 1000, cid)
}

fn parse_cursor(cursor: &str) -> anyhow::Result<(DateTime<Utc>, &str)> {
    let mut parts = cursor.split("::");

    let indexed_at = parts
        .next()
        .ok_or_else(|| anyhow!("Malformed cursor: {cursor}"))?;
    let cid = parts
        .next()
        .ok_or_else(|| anyhow!("Malformed cursor: {cursor}"))?;

    if parts.next().is_some() {
        return Err(anyhow!("Malformed cursor: {cursor}"));
    }

    let indexed_at: i64 = indexed_at.parse()?;
    let indexed_at = Utc.timestamp_opt(indexed_at / 1000, 0).unwrap(); // TODO: handle error

    Ok((indexed_at, cid))
}
