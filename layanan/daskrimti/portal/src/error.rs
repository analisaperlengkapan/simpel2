//! Error types for the portal service

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortalError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
}

impl IntoResponse for PortalError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            PortalError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            PortalError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            PortalError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
            PortalError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PortalError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PortalError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            PortalError::Grpc(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            PortalError::Pool(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, PortalError>;
