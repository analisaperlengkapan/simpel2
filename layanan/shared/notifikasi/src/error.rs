use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use deadpool_postgres;
use thiserror::Error;
use tokio_postgres::Error as PgError;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] PgError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found")]
    NotFound,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Email error: {0}")]
    Email(String),
    #[error("WhatsApp error: {0}")]
    WhatsApp(String),
    #[error("Push error: {0}")]
    Push(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error")]
    Internal,
}

impl From<redis::RedisError> for AppError {
    fn from(_err: redis::RedisError) -> Self {
        AppError::Internal
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(_err: deadpool_postgres::PoolError) -> Self {
        AppError::Internal
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Db(_) | AppError::Io(_) | AppError::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::Email(_) | AppError::WhatsApp(_) | AppError::Push(_) => {
                StatusCode::BAD_GATEWAY
            }
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        };
        error!(error = ?self, "AppError");
        (status, format!("{{\"error\":\"{}\"}}", self)).into_response()
    }
}
