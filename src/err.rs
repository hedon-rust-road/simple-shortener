use axum::response::IntoResponse;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub(crate) enum ShortenError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Duplicate ID: {0}")]
    DuplicateId(String),
}

impl IntoResponse for ShortenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::DatabaseError(e) => {
                warn!("Database error: {:?}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Self::DuplicateId(_) => {
                unreachable!("Duplicate ID should be handled by the AppState")
            }
        }
    }
}
