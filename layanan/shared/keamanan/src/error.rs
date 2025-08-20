use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("MFA required")]
    MfaRequired,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Vault error: {0}")]
    VaultError(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Argon2 error: {0}")]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("Base32 error: {0}")]
    Base32(#[from] base32::DecodeError),

    #[error("QR code error: {0}")]
    QrCode(#[from] qrcode::types::QrError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::InvalidMfaCode => (StatusCode::UNAUTHORIZED, "Invalid MFA code"),
            AppError::MfaRequired => (StatusCode::UNAUTHORIZED, "MFA required"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::ConfigurationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::VaultError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AppError::Request(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Request error"),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "Invalid JSON"),
            AppError::Argon2(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Password hash error"),
            AppError::Base32(_) => (StatusCode::BAD_REQUEST, "Invalid base32 encoding"),
            AppError::QrCode(_) => (StatusCode::INTERNAL_SERVER_ERROR, "QR code generation error"),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
} 