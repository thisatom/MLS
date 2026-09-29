//! Error types for MLS relay server

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

/// Server error type
#[derive(Debug, Error)]
pub enum ServerError {
    /// Invalid or missing authentication key
    #[error("Invalid or missing authentication key")]
    InvalidAuthKey,

    /// Unauthorized access
    #[error("Unauthorized access")]
    Unauthorized,

    /// Invalid request format
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Blob not found
    #[error("Blob not found")]
    BlobNotFound,

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            ServerError::InvalidAuthKey | ServerError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::DatabaseError(_) | ServerError::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ServerError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ServerError::BlobNotFound => StatusCode::NOT_FOUND,
        };

        (status, self.to_string()).into_response()
    }
}
