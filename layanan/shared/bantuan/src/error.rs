use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[allow(dead_code)]
    #[error("Database error: {0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[allow(dead_code)]
    #[error("Not found")]
    NotFound,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[allow(dead_code)]
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("AI error: {0}")]
    Ai(String),
    #[allow(dead_code)]
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Pool config error: {0}")]
    PoolConfig(String),
    #[error("Prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Db(_) | AppError::Pool(_) | AppError::PoolConfig(_) | AppError::Io(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::Ai(_) | AppError::Prometheus(_) | AppError::Utf8(_) | AppError::Redis(_) => {
                StatusCode::BAD_GATEWAY
            }
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        };
        error!(error = ?self, "AppError");
        (status, format!("{{\"error\":\"{}\"}}", self)).into_response()
    }
}
