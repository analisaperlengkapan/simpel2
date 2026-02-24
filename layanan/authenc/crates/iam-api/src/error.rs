//! IAM API error handling

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use authenc_types::AuthencError;

/// Error response JSON structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// API error wrapper that implements IntoResponse
pub struct ApiError(pub AuthencError);

impl From<AuthencError> for ApiError {
    fn from(err: AuthencError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            AuthencError::AuthenticationFailed(_) => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::AuthorizationFailed(_) => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthencError::UserNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthencError::RealmNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthencError::ClientNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthencError::SessionNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthencError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::UserDisabled => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthencError::AccountLocked { .. } => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthencError::RealmDisabled => (StatusCode::FORBIDDEN, self.0.to_string()),
            AuthencError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::TokenExpired => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::InvalidMfaCode => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::MfaRequired => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthencError::CryptoError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthencError::ConfigError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthencError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AuthencError::Conflict(_) => (StatusCode::CONFLICT, self.0.to_string()),
            AuthencError::UsernameAlreadyExists(_) => (StatusCode::CONFLICT, self.0.to_string()),
            AuthencError::EmailAlreadyExists(_) => (StatusCode::CONFLICT, self.0.to_string()),
            AuthencError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.0.to_string()),
            AuthencError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthencError::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            AuthencError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            AuthencError::WebAuthnError(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AuthencError::NotImplemented(_) => (StatusCode::NOT_IMPLEMENTED, self.0.to_string()),
            AuthencError::OAuth2Error(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
        };

        let error_response = ErrorResponse {
            error: format!("{:?}", self.0),
            message,
        };

        (status, Json(error_response)).into_response()
    }
}

/// Result type alias for IAM API handlers
pub type ApiResult<T> = std::result::Result<T, ApiError>;
