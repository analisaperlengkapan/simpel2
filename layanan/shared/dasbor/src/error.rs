use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Database error: {0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found")]
    NotFound,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limit exceeded")]
    RateLimit,
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
    #[error("Aggregator error: {0}")]
    Aggregator(String),
    #[error("Chart error: {0}")]
    Chart(String),
    #[error("Real-time error: {0}")]
    RealTime(String),
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> Response {
        let status = match self {
            DashboardError::Db(_)
            | DashboardError::Pool(_)
            | DashboardError::PoolConfig(_)
            | DashboardError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DashboardError::NotFound => StatusCode::NOT_FOUND,
            DashboardError::Forbidden => StatusCode::FORBIDDEN,
            DashboardError::Validation(_)
            | DashboardError::BadRequest(_)
            | DashboardError::Json(_) => StatusCode::BAD_REQUEST,
            DashboardError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            DashboardError::Prometheus(_) | DashboardError::Utf8(_) | DashboardError::Redis(_) => {
                StatusCode::BAD_GATEWAY
            }
            DashboardError::Unauthorized => StatusCode::UNAUTHORIZED,
            DashboardError::Aggregator(_)
            | DashboardError::Chart(_)
            | DashboardError::RealTime(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        error!(error = ?self, "DashboardError");
        (status, format!("{{\"error\":\"{}\"}}", self)).into_response()
    }
}
