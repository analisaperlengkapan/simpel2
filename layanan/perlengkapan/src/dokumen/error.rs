use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("Database pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Virus detected: {0}")]
    Virus(String),
    #[error("AI service error: {0}")]
    Ai(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(_) | AppError::Pool(_) | AppError::Io(_) | AppError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Validation(msg) | AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Virus(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Ai(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };
        error!(error = ?self, "AppError");
        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}
