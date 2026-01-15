use thiserror::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Error, Debug)]
pub enum AuthencError {
    #[error("Invalid OTP code")]
    InvalidOtpCode,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Authorization error: {0}")]
    AuthError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account locked: {reason}")]
    AccountLocked { reason: String, locked_until: std::time::Instant },
}

impl AuthencError {
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, AuthencError>;

impl IntoResponse for AuthencError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthencError::InvalidOtpCode => (StatusCode::UNAUTHORIZED, "Invalid OTP code".to_string()),
            AuthencError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            AuthencError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg)),
            AuthencError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AuthencError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthencError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthencError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AuthencError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication failed".to_string()),
            AuthencError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            AuthencError::AccountLocked { reason, .. } => (StatusCode::FORBIDDEN, format!("Account locked: {}", reason)),
        };
        (status, message).into_response()
    }
}
