//! Authentication endpoint handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::auth_helpers;
use crate::state::ApiState;

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
    /// Optional realm ID (defaults to master realm)
    pub realm_id: Option<Uuid>,
}

/// Login response payload
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// Optional MFA token if MFA is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_token: Option<String>,
    /// Whether MFA is required
    pub mfa_required: bool,
}

/// Logout request payload
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Refresh token to invalidate
    pub refresh_token: String,
}

/// Refresh token request payload
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Refresh token response payload
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New JWT access token
    pub access_token: String,
    /// New refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// First name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Last name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Whether email is verified
    pub email_verified: bool,
    /// Whether MFA is enabled
    pub mfa_enabled: bool,
    /// Realm ID
    pub realm_id: Uuid,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Error message
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// POST /api/v1/auth/login - Username/password login
///
/// Authenticates a user with username and password.
/// Returns JWT access token and refresh token on success.
/// If MFA is enabled, returns mfa_token and requires MFA verification.
pub async fn login_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ErrorResponse> {
    // TODO: Implement authentication logic
    // 1. Call auth_service.authenticate(username, password, realm_id)
    // 2. Check if MFA is required
    // 3. If MFA required, return mfa_token
    // 4. Otherwise, generate JWT tokens and return

    // Placeholder implementation
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Login endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/logout - Session invalidation
///
/// Invalidates the provided refresh token and associated session.
pub async fn logout_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<LogoutRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // TODO: Implement logout logic
    // 1. Validate refresh token
    // 2. Invalidate session
    // 3. Revoke refresh token

    // Placeholder implementation
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Logout endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/refresh - Refresh token exchange
///
/// Exchanges a refresh token for a new access token and refresh token.
pub async fn refresh_token_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, ErrorResponse> {
    // TODO: Implement token refresh logic
    // 1. Validate refresh token
    // 2. Check if token is expired or revoked
    // 3. Generate new access token and refresh token
    // 4. Rotate refresh token (invalidate old one)

    // Placeholder implementation
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Token refresh endpoint not yet implemented".to_string(),
    })
}

/// GET /api/v1/auth/me - Get current user profile
///
/// Returns the profile of the currently authenticated user.
/// Requires valid JWT token in Authorization header.
pub async fn get_current_user_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<UserProfileResponse>, ErrorResponse> {
    let _user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    // TODO: Implement user profile retrieval from database
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Get current user endpoint not yet implemented".to_string(),
    })
}

// =============================================================================
// Profile & Password Management Handlers
// =============================================================================

/// Request body for profile update
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// Request body for password change
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password for verification
    pub current_password: String,
    /// New password
    pub new_password: String,
}

/// Request body for password reset request
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    /// Email address to send reset link
    pub email: String,
    /// Optional captcha token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_token: Option<String>,
}

/// Request body for password reset confirmation
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmRequest {
    /// Password reset token from email link
    pub token: String,
    /// New password
    pub new_password: String,
}

/// PUT /api/v1/auth/me - Update current user profile
pub async fn update_profile_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(_request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, ErrorResponse> {
    let _user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    // TODO: Implement profile update via user_service
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Profile update endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/me/password - Change password
pub async fn change_password_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(_request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ErrorResponse> {
    let _user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    // TODO: Implement password change via auth_service
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Password change endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/password/reset - Request password reset
pub async fn password_reset_request_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<PasswordResetRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // NOTE: No auth required — public endpoint
    // TODO: Implement password reset email flow
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Password reset request endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/password/reset/confirm - Confirm password reset
pub async fn password_reset_confirm_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<PasswordResetConfirmRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // NOTE: No auth required — uses reset token
    // TODO: Implement password reset confirmation
    Err(ErrorResponse {
        error: "not_implemented".to_string(),
        message: "Password reset confirm endpoint not yet implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username":"testuser","password":"testpass"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            access_token: "token123".to_string(),
            refresh_token: "refresh123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 900,
            mfa_token: None,
            mfa_required: false,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("Bearer"));
    }
}
