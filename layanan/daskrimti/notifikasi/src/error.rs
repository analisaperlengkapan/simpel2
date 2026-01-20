use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found")]
    #[allow(dead_code)]
    NotFound,
    #[error("Forbidden")]
    #[allow(dead_code)]
    Forbidden,
    #[error("Validation error: {0}")]
    #[allow(dead_code)]
    Validation(Box<str>),
    #[error("Rate limit exceeded")]
    #[allow(dead_code)]
    RateLimit,
    #[error("Email error: {0}")]
    #[allow(dead_code)]
    Email(Box<str>),
    #[error("WhatsApp error: {0}")]
    #[allow(dead_code)]
    WhatsApp(Box<str>),
    #[error("Push error: {0}")]
    #[allow(dead_code)]
    Push(Box<str>),
    #[error("Unauthorized")]
    #[allow(dead_code)]
    Unauthorized,
    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(Box<str>),
    #[error("Internal server error")]
    Internal,
    #[error("Redis error: {0}")]
    Redis(redis::RedisError),
}

impl From<redis::RedisError> for AppError {
    fn from(_err: redis::RedisError) -> Self {
        AppError::Redis(_err)
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
            AppError::Db(_) | AppError::Io(_) | AppError::Internal | AppError::Redis(_) => {
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
