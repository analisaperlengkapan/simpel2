//! Authentication handlers
//!
//! REST API handlers that proxy auth requests to Authenc via gRPC

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::services::AuthencError;
use crate::state::AppState;

/// Login request body
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub captcha_token: Option<String>,
    #[serde(default)]
    pub mfa_code: Option<String>,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_token: Option<String>,
    pub mfa_required: bool,
    pub mfa_setup_required: bool,
    pub message: String,
    pub expires_in: Option<i64>,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Token refresh response
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

/// Token validation request
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub token: String,
}

/// MFA setup response
#[derive(Debug, Serialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

/// MFA verify request
#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub code: String,
    pub temp_token: Option<String>,
}

/// MFA verify response
#[derive(Debug, Serialize)]
pub struct MfaVerifyResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ErrorResponse {
    fn from_authenc_error(err: AuthencError) -> (StatusCode, Json<Self>) {
        let (status, code) = match &err {
            AuthencError::AuthenticationFailed(_) => (StatusCode::UNAUTHORIZED, "AUTH_FAILED"),
            AuthencError::Unauthorized(_) => (StatusCode::FORBIDDEN, "UNAUTHORIZED"),
            AuthencError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AuthencError::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE"),
            AuthencError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST"),
            AuthencError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        (
            status,
            Json(ErrorResponse {
                error: err.to_string(),
                code: code.to_string(),
            }),
        )
    }
}

/// POST /api/auth/login - Authenticate user
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    info!("Login attempt for user: {}", request.username);

    // TODO: Validate CAPTCHA token via secreton encryption
    // For now, just proceed with authentication

    match state
        .authenc
        .authenticate(
            &request.username,
            &request.password,
            request.mfa_code.as_deref(),
        )
        .await
    {
        Ok(auth_result) => {
            info!("Login successful for user: {}", request.username);

            let response = LoginResponse {
                access_token: Some(auth_result.access_token),
                refresh_token: Some(auth_result.refresh_token),
                temp_token: None,
                mfa_required: auth_result.mfa_required,
                mfa_setup_required: auth_result.mfa_setup_required,
                message: "Login successful".to_string(),
                expires_in: Some(auth_result.expires_in),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            error!("Login failed for user {}: {}", request.username, err);

            // Check if this is an MFA-related response
            // In practice, the gRPC response would indicate MFA requirements
            let (status, error_response) = ErrorResponse::from_authenc_error(err);
            (status, error_response).into_response()
        }
    }
}

/// POST /api/auth/refresh - Refresh access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshRequest>,
) -> impl IntoResponse {
    info!("Token refresh request");

    match state.authenc.refresh_token(&request.refresh_token).await {
        Ok((access_token, refresh_token, expires_in)) => {
            info!("Token refresh successful");

            let response = RefreshResponse {
                access_token,
                refresh_token,
                expires_in,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            error!("Token refresh failed: {}", err);
            let (status, error_response) = ErrorResponse::from_authenc_error(err);
            (status, error_response).into_response()
        }
    }
}

/// POST /api/auth/logout - Revoke session
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LogoutRequest>,
) -> impl IntoResponse {
    info!("Logout request");

    let mut success = true;

    // Revoke access token if provided
    if let Some(access_token) = &request.access_token {
        if let Err(err) = state.authenc.revoke_token(access_token, false).await {
            error!("Failed to revoke access token: {}", err);
            success = false;
        }
    }

    // Revoke refresh token if provided
    if let Some(refresh_token) = &request.refresh_token {
        if let Err(err) = state.authenc.revoke_token(refresh_token, true).await {
            error!("Failed to revoke refresh token: {}", err);
            success = false;
        }
    }

    if success {
        info!("Logout successful");
        (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": "Logged out successfully"})),
        )
            .into_response()
    } else {
        (StatusCode::PARTIAL_CONTENT, Json(serde_json::json!({"success": false, "message": "Partial logout - some tokens could not be revoked"}))).into_response()
    }
}

/// POST /api/auth/validate - Validate token
pub async fn validate_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ValidateRequest>,
) -> impl IntoResponse {
    info!("Token validation request");

    match state.authenc.validate_token(&request.token).await {
        Ok(validation) => {
            info!("Token validation result: {}", validation.valid);
            (StatusCode::OK, Json(validation)).into_response()
        }
        Err(err) => {
            error!("Token validation failed: {}", err);
            let (status, error_response) = ErrorResponse::from_authenc_error(err);
            (status, error_response).into_response()
        }
    }
}

/// POST /api/auth/mfa/setup - Initiate MFA setup
pub async fn mfa_setup(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user_id): axum::extract::Extension<String>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
) -> impl IntoResponse {
    info!("MFA setup request for user: {}", user_id);

    match state.authenc.enable_mfa(&token, &user_id).await {
        Ok(result) => {
            info!("MFA setup initiated for user: {}", user_id);

            let response = MfaSetupResponse {
                secret: result.secret,
                qr_code_url: result.qr_code_url,
                backup_codes: result.backup_codes,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            error!("MFA setup failed for user {}: {}", user_id, err);
            let (status, error_response) = ErrorResponse::from_authenc_error(err);
            (status, error_response).into_response()
        }
    }
}

/// POST /api/auth/mfa/verify - Verify MFA code
pub async fn mfa_verify(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user_id): axum::extract::Extension<String>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    Json(request): Json<MfaVerifyRequest>,
) -> impl IntoResponse {
    info!("MFA verification request for user: {}", user_id);

    match state
        .authenc
        .verify_mfa(&token, &user_id, &request.code)
        .await
    {
        Ok(valid) => {
            info!("MFA verification result for user {}: {}", user_id, valid);

            // If valid, could return new tokens here
            let response = MfaVerifyResponse {
                valid,
                access_token: None, // Would be populated after successful MFA
                refresh_token: None,
                message: if valid {
                    Some("MFA verification successful".to_string())
                } else {
                    Some("Invalid MFA code".to_string())
                },
            };

            if valid {
                (StatusCode::OK, Json(response)).into_response()
            } else {
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
        }
        Err(err) => {
            error!("MFA verification failed for user {}: {}", user_id, err);
            let (status, error_response) = ErrorResponse::from_authenc_error(err);
            (status, error_response).into_response()
        }
    }
}

/// Routes for auth handlers
pub fn routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::post;

    axum::Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/validate", post(validate_token))
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/verify", post(mfa_verify))
}
