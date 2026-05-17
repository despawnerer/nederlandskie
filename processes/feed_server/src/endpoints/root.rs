use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::response::Html;

use crate::errors::AppError;
use nederlandskie_core::services::Database;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    total_posts: i64,
    nl_profiles: i64,
}

pub async fn root(State(database): State<Arc<Database>>) -> Result<Html<String>, AppError> {
    let total_posts = database.count_posts().await?;
    let nl_profiles = database.count_profiles_in_country("nl").await?;

    let rendered = IndexTemplate {
        total_posts,
        nl_profiles,
    }
    .render()?;

    Ok(Html(rendered))
}
