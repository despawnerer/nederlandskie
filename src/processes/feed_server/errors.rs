use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

pub enum AppError {
    FeedNotFound(String),
    Other(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::FeedNotFound(name) => (StatusCode::NOT_FOUND, format!("Feed not found: {}", name)),
            Self::Other(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            )
        }.into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Other(err.into())
    }
}
