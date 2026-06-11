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
        let status = match &self {
            AppError::Db(_) | AppError::Pool(_) | AppError::Io(_) | AppError::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Virus(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Ai(_) => StatusCode::BAD_GATEWAY,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        };
        error!(error = ?self, "AppError");
        (status, format!("{{\"error\":\"{}\"}}", self)).into_response()
    }
}
