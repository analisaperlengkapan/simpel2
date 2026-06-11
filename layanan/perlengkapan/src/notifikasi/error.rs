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
    #[error("Internal server error: {0}")]
    Internal(Box<str>),
    #[error("Config error: {0}")]
    Config(Box<str>),
    #[error("Redis error: {0}")]
    Redis(redis::RedisError),
}

impl From<redis::RedisError> for AppError {
    fn from(_err: redis::RedisError) -> Self {
        AppError::Redis(_err)
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AppError::Internal(format!("Pool error: {}", err).into_boxed_str())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(format!("JSON error: {}", err).into_boxed_str())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(_)
            | AppError::Io(_)
            | AppError::Internal(_)
            | AppError::Redis(_)
            | AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Validation(msg) | AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            AppError::RateLimit => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            AppError::Email(msg) | AppError::WhatsApp(msg) | AppError::Push(msg) => {
                (StatusCode::BAD_GATEWAY, msg.to_string())
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };
        error!(error = ?self, "AppError");
        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}
