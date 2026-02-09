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
    #[error("Database pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound,
    #[error("Internal server error")]
    Internal,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
}

impl From<(StatusCode, String)> for AppError {
    fn from((status, msg): (StatusCode, String)) -> Self {
        match status {
            StatusCode::UNAUTHORIZED => AppError::Authentication(msg),
            StatusCode::FORBIDDEN => AppError::Authorization(msg),
            StatusCode::BAD_REQUEST => AppError::BadRequest(msg),
            _ => AppError::Internal,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Pool(e) => {
                error!("Pool error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database pool error".to_string(),
                )
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Authorization(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        };

        let api_error = lib_common::error::ApiError {
            message,
            code: None, // We can add specific error codes later
        };

        (status, axum::Json(api_error)).into_response()
    }
}
