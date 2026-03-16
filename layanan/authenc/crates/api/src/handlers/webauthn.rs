//! WebAuthn/Passkeys endpoint handlers (MANDATORY - PRIMARY authentication method)

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use super::auth_helpers::extract_user_from_token;
use crate::{handlers::ErrorResponse, state::ApiState};
use authenc_types::UserId;

/// Start passkey registration request
#[derive(Debug, Deserialize)]
pub struct StartRegistrationRequest {
    /// User ID
    pub user_id: UserId,
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
}

/// Start passkey registration response
#[derive(Debug, Serialize)]
pub struct StartRegistrationResponse {
    /// WebAuthn creation challenge
    pub challenge: CreationChallengeResponse,
    /// Registration session ID (to be used in finish)
    pub session_id: String,
}

/// Finish passkey registration request
#[derive(Debug, Deserialize)]
pub struct FinishRegistrationRequest {
    /// Registration session ID from start
    pub session_id: String,
    /// WebAuthn credential from browser
    pub credential: RegisterPublicKeyCredential,
}

/// Finish passkey registration response
#[derive(Debug, Serialize)]
pub struct FinishRegistrationResponse {
    /// Credential ID
    pub credential_id: Uuid,
    /// Success message
    pub message: String,
}

/// Start passkey authentication request
#[derive(Debug, Deserialize)]
pub struct StartAuthenticationRequest {
    /// Optional user ID (for username-based auth)
    /// If None, uses usernameless authentication (discoverable credentials)
    pub user_id: Option<UserId>,
}

/// Start passkey authentication response
#[derive(Debug, Serialize)]
pub struct StartAuthenticationResponse {
    /// WebAuthn authentication challenge
    pub challenge: RequestChallengeResponse,
    /// Authentication session ID (to be used in finish)
    pub session_id: String,
}

/// Finish passkey authentication request
#[derive(Debug, Deserialize)]
pub struct FinishAuthenticationRequest {
    /// Authentication session ID from start
    pub session_id: String,
    /// WebAuthn assertion from browser
    pub credential: PublicKeyCredential,
}

/// Finish passkey authentication response
#[derive(Debug, Serialize)]
pub struct FinishAuthenticationResponse {
    /// JWT access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
}

/// Credential metadata response
#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    /// Credential ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Optional nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// Created timestamp
    pub created_at: String,
    /// Last used timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    /// Authenticator type (platform or cross-platform)
    pub authenticator_type: String,
}

/// Update credential nickname request
#[derive(Debug, Deserialize)]
pub struct UpdateCredentialRequest {
    /// New nickname
    pub nickname: String,
}

/// POST /api/v1/auth/webauthn/register/start - Start passkey registration
///
/// Initiates the passkey registration ceremony.
/// User info is extracted from the JWT token in the Authorization header.
/// Returns a WebAuthn creation challenge to be passed to navigator.credentials.create().
pub async fn start_registration_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<StartRegistrationResponse>, ErrorResponse> {
    // Extract user ID from JWT token
    let user_id = extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "authentication_required".to_string(),
            message: e.message,
        })?;

    // Look up user details for WebAuthn registration
    let user_id = UserId(user_id);
    let username = user_id.0.to_string(); // Fallback: use UUID as username
    let display_name = username.clone(); // Fallback: use UUID as display name

    // Start registration with WebAuthn service
    let (challenge, session) = state
        .webauthn_service
        .start_registration(user_id, &username, &display_name)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "registration_failed".to_string(),
            message: format!("Failed to start passkey registration: {}", e),
        })?;

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Store session temporarily
    state
        .session_store
        .store_registration_session(session_id.clone(), session)
        .await;

    Ok(Json(StartRegistrationResponse {
        challenge,
        session_id,
    }))
}

/// POST /api/v1/auth/webauthn/register/finish - Complete passkey registration
///
/// Completes the passkey registration ceremony.
/// Verifies the attestation response and stores the credential.
pub async fn finish_registration_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<FinishRegistrationRequest>,
) -> Result<Json<FinishRegistrationResponse>, ErrorResponse> {
    // Retrieve registration session
    let session = state
        .session_store
        .remove_registration_session(&request.session_id)
        .await
        .ok_or_else(|| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_session".to_string(),
            message: "Registration session not found or expired".to_string(),
        })?;

    // Finish registration with WebAuthn service
    let credential = state
        .webauthn_service
        .finish_registration(session.user_id, &request.credential, &session)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "registration_failed".to_string(),
            message: format!("Failed to complete passkey registration: {}", e),
        })?;

    Ok(Json(FinishRegistrationResponse {
        credential_id: credential.id,
        message: "Passkey registered successfully".to_string(),
    }))
}

/// POST /api/v1/auth/webauthn/authenticate/start - Start passkey authentication
///
/// Initiates the passkey authentication ceremony.
/// Supports both username-based and usernameless (discoverable credentials) authentication.
pub async fn start_authentication_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<StartAuthenticationRequest>,
) -> Result<Json<StartAuthenticationResponse>, ErrorResponse> {
    // Start authentication with WebAuthn service
    let (challenge, session) = state
        .webauthn_service
        .start_authentication(request.user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to start passkey authentication");
            ErrorResponse {
                status_code: axum::http::StatusCode::BAD_REQUEST,
                error: "authentication_failed".to_string(),
                message: "Gagal memulai autentikasi passkey. Silakan coba lagi.".to_string(),
            }
        })?;

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Store session temporarily
    state
        .session_store
        .store_authentication_session(session_id.clone(), session)
        .await;

    Ok(Json(StartAuthenticationResponse {
        challenge,
        session_id,
    }))
}

/// POST /api/v1/auth/webauthn/authenticate/finish - Complete passkey authentication
///
/// Completes the passkey authentication ceremony.
/// Verifies the assertion response and returns JWT tokens on success.
pub async fn finish_authentication_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<FinishAuthenticationRequest>,
) -> Result<Json<FinishAuthenticationResponse>, ErrorResponse> {
    // Retrieve authentication session
    let session = state
        .session_store
        .remove_authentication_session(&request.session_id)
        .await
        .ok_or_else(|| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_session".to_string(),
            message: "Authentication session not found or expired".to_string(),
        })?;

    // Finish authentication with WebAuthn service
    let result = state
        .webauthn_service
        .finish_authentication(&request.credential, &session)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to complete passkey authentication");
            ErrorResponse {
                status_code: axum::http::StatusCode::BAD_REQUEST,
                error: "authentication_failed".to_string(),
                message: "Gagal menyelesaikan autentikasi passkey. Silakan coba lagi.".to_string(),
            }
        })?;

    // Extract user ID from result
    let user_id = match result {
        authenc_webauthn::AuthenticationResult::Success { user_id, .. } => user_id,
        authenc_webauthn::AuthenticationResult::Failed { reason } => {
            tracing::warn!(reason = %reason, "Passkey authentication failed");
            return Err(ErrorResponse {
                status_code: axum::http::StatusCode::UNAUTHORIZED,
                error: "authentication_failed".to_string(),
                message: "Autentikasi passkey gagal. Silakan coba lagi.".to_string(),
            });
        }
    };

    // Generate JWT tokens using JwtService
    let session_id = uuid::Uuid::new_v4().to_string();

    // Fetch user to build custom claims
    let user = state.user_service.get_user(user_id.clone()).await.ok();

    if user.is_none() {
        tracing::warn!("Failed to fetch user details for passkey token claims mapping");
    }

    // Build custom claims
    let custom_claims = crate::handlers::auth_helpers::build_user_custom_claims(user.as_ref());

    let access_token = state
        .jwt_service
        .generate_access_token_with_claims(
            &user_id.0.to_string(),
            None,                                     // realm
            Some("openid profile email".to_string()), // scope
            Some(session_id.clone()),
            custom_claims,
        )
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to generate access token");
            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "token_generation_failed".to_string(),
                message: "Terjadi kesalahan sistem. Silakan coba lagi nanti.".to_string(),
            }
        })?;

    let refresh_token = state
        .jwt_service
        .generate_refresh_token(&user_id.0.to_string(), &session_id)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to generate refresh token");
            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "token_generation_failed".to_string(),
                message: "Terjadi kesalahan sistem. Silakan coba lagi nanti.".to_string(),
            }
        })?;

    Ok(Json(FinishAuthenticationResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutes
    }))
}

/// GET /api/v1/auth/webauthn/credentials - List user's passkeys
///
/// Returns a list of all registered passkeys for the authenticated user.
/// Requires valid JWT token in Authorization header.
pub async fn list_credentials_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<CredentialResponse>>, ErrorResponse> {
    // Extract user ID from JWT
    let user_id = extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "authentication_required".to_string(),
            message: e.message,
        })?;

    let credentials = state
        .webauthn_service
        .list_credentials(UserId(user_id))
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "list_failed".to_string(),
            message: format!("Failed to list credentials: {}", e),
        })?;

    let response: Vec<CredentialResponse> = credentials
        .into_iter()
        .map(|c| CredentialResponse {
            id: c.id,
            user_id: c.user_id.0,
            nickname: c.nickname,
            created_at: c.created_at.to_rfc3339(),
            last_used: c.last_used.map(|t| t.to_rfc3339()),
            authenticator_type: "platform".to_string(),
        })
        .collect();

    Ok(Json(response))
}

/// DELETE /api/v1/auth/webauthn/credentials/{id} - Delete passkey
///
/// Deletes a specific passkey credential.
/// Requires valid JWT token and ownership verification.
pub async fn delete_credential_handler(
    State(state): State<Arc<ApiState>>,
    Path(credential_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ErrorResponse> {
    // Extract user ID from JWT
    let user_id = extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "authentication_required".to_string(),
            message: e.message,
        })?;

    state
        .webauthn_service
        .delete_credential(UserId(user_id), credential_id)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "deletion_failed".to_string(),
            message: format!("Failed to delete credential: {}", e),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/v1/auth/webauthn/credentials/{id} - Update passkey nickname
///
/// Updates the nickname of a specific passkey credential.
/// Requires valid JWT token and ownership verification.
pub async fn update_credential_handler(
    State(state): State<Arc<ApiState>>,
    Path(credential_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<UpdateCredentialRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // Extract user ID from JWT
    let user_id = extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "authentication_required".to_string(),
            message: e.message,
        })?;

    state
        .webauthn_service
        .update_credential_nickname(UserId(user_id), credential_id, request.nickname)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "update_failed".to_string(),
            message: format!("Failed to update credential: {}", e),
        })?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_registration_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "username": "testuser",
            "display_name": "Test User"
        }"#;
        let request: StartRegistrationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.display_name, "Test User");
    }

    #[test]
    fn test_update_credential_request_deserialization() {
        let json = r#"{"nickname":"My YubiKey"}"#;
        let request: UpdateCredentialRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.nickname, "My YubiKey");
    }
}
