//! MFA (Multi-Factor Authentication) API handlers
//!
//! These handlers implement the MFA login flow endpoints that the portal
//! microfrontend calls during authentication:
//!
//! - `POST /api/v1/auth/mfa/setup` — Generate TOTP secret and QR code
//! - `POST /api/v1/auth/mfa/verify-setup` — Verify initial TOTP code to confirm setup
//! - `POST /api/v1/auth/mfa/verify` — Verify TOTP code during login (returns JWT)
//! - `GET /api/v1/auth/mfa/status` — Check MFA enrollment status
//!
//! ## Architecture
//!
//! The handlers use the `MfaApiService` trait to abstract MFA operations.
//! This avoids a circular dependency with `authenc-mfa` (which depends on `authenc-api`).
//! The concrete implementation is injected via `ApiState` at startup.
//!
//! ## Authentication Flow
//!
//! 1. User logs in with password → server returns `temp_token` if MFA required
//! 2. Frontend calls `/mfa/verify` with `temp_token` + TOTP code
//! 3. Server validates TOTP → returns full `access_token` + `refresh_token`

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

// =============================================================================
// MFA API Service Trait (Dependency Inversion)
// =============================================================================

/// Trait for MFA operations consumed by REST handlers.
///
/// Implementations must be `Send + Sync` for use in async handlers.
/// The concrete implementation (using `authenc-mfa` crate) is injected
/// into `ApiState` during application startup.
#[async_trait::async_trait]
pub trait MfaApiService: Send + Sync {
    /// Set up TOTP for a user — generates secret, QR code, and backup codes.
    async fn setup_totp(&self, user_id: Uuid, username: &str) -> Result<MfaSetupData, MfaApiError>;

    /// Verify the initial TOTP code during setup confirmation.
    async fn verify_setup(&self, user_id: Uuid, code: &str) -> Result<bool, MfaApiError>;

    /// Verify a TOTP code (during login or re-authentication).
    async fn verify_code(&self, user_id: Uuid, code: &str) -> Result<bool, MfaApiError>;

    /// Disable TOTP for a user.
    async fn disable_totp(&self, user_id: Uuid) -> Result<(), MfaApiError>;

    /// Get MFA enrollment status for a user.
    async fn get_status(&self, user_id: Uuid) -> Result<MfaStatusData, MfaApiError>;
}

// =============================================================================
// Request / Response Types
// =============================================================================

/// MFA setup data returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupData {
    /// QR code data URL for scanning with authenticator apps
    pub qr_code_url: String,
    /// Base32-encoded secret for manual entry
    pub secret_key: String,
    /// One-time backup/recovery codes
    pub backup_codes: Vec<String>,
}

/// MFA status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaStatusData {
    /// Whether MFA is currently enabled
    pub enabled: bool,
    /// When MFA was set up (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_at: Option<String>,
    /// Number of unused backup codes remaining
    pub backup_codes_remaining: i32,
    /// Last time MFA was used for authentication (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// Request body for MFA verify-setup and verify endpoints
#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    /// 6-digit TOTP code from authenticator app
    pub code: String,
    /// Temporary token (used during login flow)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_token: Option<String>,
}

/// Response for MFA verify during login — returns JWT tokens
#[derive(Debug, Serialize)]
pub struct MfaVerifyResponse {
    /// Access token (JWT)
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// Refresh token for session renewal
    pub refresh_token: String,
}

/// MFA API error
#[derive(Debug, Serialize)]
pub struct MfaApiError {
    pub error: String,
    pub message: String,
}

impl MfaApiError {
    pub fn not_configured() -> Self {
        Self {
            error: "mfa_not_configured".to_string(),
            message: "MFA service is not configured on this server".to_string(),
        }
    }

    pub fn invalid_code() -> Self {
        Self {
            error: "invalid_code".to_string(),
            message: "The provided TOTP code is invalid or expired".to_string(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            error: "internal_error".to_string(),
            message: msg.into(),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            error: "unauthorized".to_string(),
            message: msg.into(),
        }
    }
}

impl IntoResponse for MfaApiError {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "unauthorized" => StatusCode::UNAUTHORIZED,
            "mfa_not_configured" => StatusCode::SERVICE_UNAVAILABLE,
            "invalid_code" => StatusCode::BAD_REQUEST,
            "not_found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

// =============================================================================
// Handler Functions
// =============================================================================

/// POST /api/v1/auth/mfa/setup
///
/// Generate TOTP secret, QR code, and backup codes for the authenticated user.
/// Requires a valid JWT (Bearer token) in the Authorization header.
///
/// # Response
/// ```json
/// {
///   "qr_code_url": "data:image/svg+xml;base64,...",
///   "secret_key": "JBSWY3DPEHPK3PXP",
///   "backup_codes": ["12345678", "87654321", ...]
/// }
/// ```
pub async fn mfa_setup_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<MfaSetupData>, MfaApiError> {
    // Extract user from JWT
    let user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    // Get MFA service
    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    // Use user_id as username for TOTP URI (could also look up real username)
    let username = user_id.to_string();

    let setup_data = mfa_service.setup_totp(user_id, &username).await?;

    Ok(Json(setup_data))
}

/// POST /api/v1/auth/mfa/verify-setup
///
/// Verify the initial TOTP code to confirm MFA setup.
/// User must enter a code from their authenticator app to prove it's working.
///
/// # Request
/// ```json
/// { "code": "123456" }
/// ```
pub async fn mfa_verify_setup_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<MfaVerifyRequest>,
) -> Result<StatusCode, MfaApiError> {
    let user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    let valid = mfa_service.verify_setup(user_id, &request.code).await?;

    if valid {
        Ok(StatusCode::OK)
    } else {
        Err(MfaApiError::invalid_code())
    }
}

/// POST /api/v1/auth/mfa/verify
///
/// Verify TOTP code during login flow. On success, exchanges the temporary
/// token for a full access token + refresh token.
///
/// # Request
/// ```json
/// { "code": "123456", "temp_token": "eyJ..." }
/// ```
///
/// # Response
/// ```json
/// {
///   "access_token": "eyJ...",
///   "token_type": "Bearer",
///   "expires_in": 3600,
///   "refresh_token": "eyJ..."
/// }
/// ```
pub async fn mfa_verify_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, MfaApiError> {
    // Extract user from temp_token (passed in body or Authorization header)
    let user_id = if let Some(ref temp_token) = request.temp_token {
        auth_helpers::verify_token_from_body(&state, temp_token)
            .await
            .map_err(|e| MfaApiError::unauthorized(e.message))?
    } else {
        auth_helpers::extract_user_from_token(&state, &headers)
            .await
            .map_err(|e| MfaApiError::unauthorized(e.message))?
    };

    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    // Verify TOTP code
    let valid = mfa_service.verify_code(user_id, &request.code).await?;

    if !valid {
        return Err(MfaApiError::invalid_code());
    }

    // Generate full JWT tokens (MFA verification succeeded)
    let session_id = Uuid::new_v4().to_string();

    // Fetch user to build custom claims
    let user = state.user_service.get_user(authenc_types::UserId::from_uuid(user_id)).await.ok();

    if user.is_none() {
        tracing::warn!("Failed to fetch user details for MFA token claims mapping");
    }

    // Build custom claims
    let custom_claims = crate::handlers::auth_helpers::build_user_custom_claims(user.as_ref());

    let access_token = state
        .jwt_service
        .generate_access_token_with_claims(
            &user_id.to_string(),
            None,
            Some("openid profile email".to_string()),
            Some(session_id.clone()),
            custom_claims,
        )
        .map_err(|e| MfaApiError::internal(format!("Failed to generate access token: {}", e)))?;

    let refresh_token = state
        .jwt_service
        .generate_refresh_token(&user_id.to_string(), &session_id)
        .map_err(|e| MfaApiError::internal(format!("Failed to generate refresh token: {}", e)))?;

    Ok(Json(MfaVerifyResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
    }))
}

/// GET /api/v1/auth/mfa/status
///
/// Check MFA enrollment status for the authenticated user.
///
/// # Response
/// ```json
/// {
///   "enabled": true,
///   "setup_at": "2024-01-15T10:30:00Z",
///   "backup_codes_remaining": 8,
///   "last_used": "2024-06-01T14:22:00Z"
/// }
/// ```
pub async fn mfa_status_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<MfaStatusData>, MfaApiError> {
    let user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    let status = mfa_service.get_status(user_id).await?;

    Ok(Json(status))
}

// =============================================================================
// Alias Handlers for /totp/* endpoints (AuthencApiClient compatibility)
// =============================================================================

/// POST /api/v1/auth/totp/enable — alias for mfa_setup_handler
pub async fn totp_enable_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<MfaSetupData>, MfaApiError> {
    mfa_setup_handler(State(state), headers).await
}

/// POST /api/v1/auth/totp/disable — disable TOTP for authenticated user
pub async fn totp_disable_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<StatusCode, MfaApiError> {
    let user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    mfa_service.disable_totp(user_id).await?;

    Ok(StatusCode::OK)
}

/// POST /api/v1/auth/totp/verify — alias for mfa_verify_handler
pub async fn totp_verify_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    body: Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, MfaApiError> {
    mfa_verify_handler(State(state), headers, body).await
}

// =============================================================================
// Backup Codes & Recovery Handlers
// =============================================================================

/// Request body for backup codes operations
#[derive(Debug, Deserialize)]
pub struct BackupCodesRequest {
    /// Action: "generate" or "list"
    pub action: String,
    /// Optional token for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Optional code for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Response for backup codes operations
#[derive(Debug, Serialize)]
pub struct BackupCodesResponse {
    /// Generated or remaining backup codes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codes: Option<Vec<String>>,
    /// Number of remaining codes
    pub remaining: i32,
    /// Success message
    pub message: String,
}

/// Recovery code verification request
#[derive(Debug, Deserialize)]
pub struct RecoveryVerifyRequest {
    /// Recovery/backup code
    pub code: String,
}

/// POST /api/v1/auth/mfa/backup-codes — Generate or list backup codes
pub async fn mfa_backup_codes_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<BackupCodesRequest>,
) -> Result<Json<BackupCodesResponse>, MfaApiError> {
    let _user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    let _mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    match request.action.as_str() {
        "generate" => {
            // TODO: Generate backup codes via MFA service
            Err(MfaApiError::internal(
                "Backup code generation not yet implemented",
            ))
        }
        "list" => {
            // TODO: List remaining backup codes status
            Err(MfaApiError::internal(
                "Backup code listing not yet implemented",
            ))
        }
        _ => Err(MfaApiError {
            error: "invalid_action".to_string(),
            message: format!(
                "Unknown action: {}. Use 'generate' or 'list'",
                request.action
            ),
        }),
    }
}

/// POST /api/v1/auth/mfa/verify-recovery — Verify a recovery/backup code during login
pub async fn mfa_verify_recovery_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<RecoveryVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, MfaApiError> {
    // Extract user from temp_token in header
    let user_id = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| MfaApiError::unauthorized(e.message))?;

    let mfa_service = state
        .mfa_service
        .as_ref()
        .ok_or_else(MfaApiError::not_configured)?;

    // Verify recovery code (uses same verify path — backup codes are checked as fallback)
    let valid = mfa_service.verify_code(user_id, &request.code).await?;

    if !valid {
        return Err(MfaApiError {
            error: "invalid_recovery_code".to_string(),
            message: "The provided recovery code is invalid or already used".to_string(),
        });
    }

    // Generate full JWT tokens
    let session_id = Uuid::new_v4().to_string();

    // Fetch user to build custom claims
    let user = state.user_service.get_user(authenc_types::UserId::from_uuid(user_id)).await.ok();

    if user.is_none() {
        tracing::warn!("Failed to fetch user details for MFA token claims mapping");
    }

    // Build custom claims
    let custom_claims = crate::handlers::auth_helpers::build_user_custom_claims(user.as_ref());

    let access_token = state
        .jwt_service
        .generate_access_token_with_claims(
            &user_id.to_string(),
            None,
            Some("openid profile email".to_string()),
            Some(session_id.clone()),
            custom_claims,
        )
        .map_err(|e| MfaApiError::internal(format!("Failed to generate access token: {}", e)))?;

    let refresh_token = state
        .jwt_service
        .generate_refresh_token(&user_id.to_string(), &session_id)
        .map_err(|e| MfaApiError::internal(format!("Failed to generate refresh token: {}", e)))?;

    Ok(Json(MfaVerifyResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
    }))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfa_setup_data_serialization() {
        let data = MfaSetupData {
            qr_code_url: "data:image/svg+xml;base64,abc123".to_string(),
            secret_key: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["12345678".to_string(), "87654321".to_string()],
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("qr_code_url"));
        assert!(json.contains("secret_key"));
        assert!(json.contains("backup_codes"));
    }

    #[test]
    fn test_mfa_status_data_serialization() {
        let status = MfaStatusData {
            enabled: true,
            setup_at: Some("2024-01-15T10:30:00Z".to_string()),
            backup_codes_remaining: 8,
            last_used: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"backup_codes_remaining\":8"));
        // last_used should be skipped when None
        assert!(!json.contains("last_used"));
    }

    #[test]
    fn test_mfa_verify_request_deserialization() {
        let json = r#"{"code":"123456"}"#;
        let request: MfaVerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.code, "123456");
        assert!(request.temp_token.is_none());
    }

    #[test]
    fn test_mfa_verify_request_with_temp_token() {
        let json = r#"{"code":"123456","temp_token":"eyJ..."}"#;
        let request: MfaVerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.code, "123456");
        assert_eq!(request.temp_token.as_deref(), Some("eyJ..."));
    }

    #[test]
    fn test_mfa_verify_response_serialization() {
        let response = MfaVerifyResponse {
            access_token: "jwt_access".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: "jwt_refresh".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("Bearer"));
        assert!(json.contains("3600"));
    }

    #[test]
    fn test_mfa_api_error_variants() {
        let err = MfaApiError::not_configured();
        assert_eq!(err.error, "mfa_not_configured");

        let err = MfaApiError::invalid_code();
        assert_eq!(err.error, "invalid_code");

        let err = MfaApiError::internal("something broke");
        assert_eq!(err.error, "internal_error");

        let err = MfaApiError::unauthorized("bad token");
        assert_eq!(err.error, "unauthorized");
    }
}
