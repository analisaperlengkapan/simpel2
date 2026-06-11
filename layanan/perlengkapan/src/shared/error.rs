//! # Application Errors
//!
//! Centralized error handling for the Perlengkapan service

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Excel error: {0}")]
    Excel(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),
}

// Implement From for common error types
impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        AppError::Excel(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
    pub error_code: String,
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database operation failed".to_string(),
                    None,
                )
            }
            AppError::Validation(e) => {
                let details = validation_errors_to_json(e);
                (
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Validation failed".to_string(),
                    Some(details),
                )
            }
            AppError::Authentication(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
                msg.clone(),
                None,
            ),
            AppError::Authorization(msg) => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                msg.clone(),
                None,
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone(), None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone(), None),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone(), None)
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal server error occurred".to_string(),
                    None,
                )
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {:?}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    "JWT_ERROR",
                    "Invalid token".to_string(),
                    None,
                )
            }
            AppError::Uuid(e) => {
                tracing::error!("UUID error: {:?}", e);
                (
                    StatusCode::BAD_REQUEST,
                    "UUID_ERROR",
                    "Invalid UUID format".to_string(),
                    None,
                )
            }
            AppError::Parse(msg) => (StatusCode::BAD_REQUEST, "PARSE_ERROR", msg.clone(), None),
            AppError::Io(e) => {
                tracing::error!("IO error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "IO operation failed".to_string(),
                    None,
                )
            }
            AppError::Excel(msg) => {
                tracing::error!("Excel error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "EXCEL_ERROR",
                    "Excel operation failed".to_string(),
                    None,
                )
            }
            AppError::WorkflowError(msg) => {
                tracing::error!("Workflow error: {}", msg);
                (StatusCode::BAD_REQUEST, "WORKFLOW_ERROR", msg.clone(), None)
            }
        };

        let body = ErrorResponse {
            success: false,
            message,
            error_code: error_code.to_string(),
            details,
        };

        (status, Json(body)).into_response()
    }
}

fn validation_errors_to_json(errors: &ValidationErrors) -> serde_json::Value {
    use serde_json::{Map, Value, json};

    let mut error_map = Map::new();

    for (field, field_errors) in errors.field_errors() {
        let field_error_messages: Vec<String> = field_errors
            .iter()
            .map(|error| {
                error
                    .message
                    .as_ref()
                    .map(|msg| msg.to_string())
                    .unwrap_or_else(|| error.code.to_string())
            })
            .collect();

        error_map.insert(
            field.to_string(),
            Value::Array(
                field_error_messages
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }

    json!(error_map)
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

// Helper function to create 404 errors
pub fn not_found(resource: &str, id: &str) -> AppError {
    AppError::NotFound(format!("{} with id '{}' not found", resource, id))
}

// Helper function to create conflict errors
pub fn conflict(message: &str) -> AppError {
    AppError::Conflict(message.to_string())
}

// Helper function to create bad request errors
pub fn bad_request(message: &str) -> AppError {
    AppError::BadRequest(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_internal_error_is_generic() {
        let err = AppError::Internal("Sensitive database details".to_string());
        let response = err.into_response();
        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.message, "An internal server error occurred");
        assert!(json.error_code == "INTERNAL_ERROR");
    }

    #[tokio::test]
    async fn test_uuid_error_is_generic() {
        let uuid_err = uuid::Uuid::parse_str("invalid").unwrap_err();
        let err = AppError::Uuid(uuid_err);
        let response = err.into_response();
        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.message, "Invalid UUID format");
        assert!(json.error_code == "UUID_ERROR");
    }

    #[tokio::test]
    async fn test_io_error_is_generic() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AppError::Io(io_err);
        let response = err.into_response();
        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.message, "IO operation failed");
        assert!(json.error_code == "IO_ERROR");
    }
}
