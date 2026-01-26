use crate::app::AppState;
use crate::error::{AuthencError, Result};
use crate::services::webauthn::WebAuthnService;
use crate::spi::credential::webauthn::AttestationPreference;
use axum::{Router, extract::State, response::Json, routing::post};
use std::sync::Arc;
use webauthn_rs_proto::{PublicKeyCredential, RegisterPublicKeyCredential};

/// Create WebAuthn routes with full attestation support
pub fn create_webauthn_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/challenge", post(register_challenge))
        .route("/register/verify", post(register_verify))
        .route("/authenticate/challenge", post(authenticate_challenge))
        .route("/authenticate/verify", post(authenticate_verify))
}

/// WebAuthn registration challenge handler with attestation support
pub async fn register_challenge(
    State(state): State<Arc<AppState>>,
    Json(request): Json<crate::services::webauthn::WebAuthnRegistrationRequest>,
) -> Result<Json<webauthn_rs::prelude::CreationChallengeResponse>> {
    // TODO: Get these from configuration
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name =
        std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "SIMPelv2 Authenc".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "https://10.1.7.121/api/auth".to_string());

    // Determine attestation preference from request or config
    let attestation_pref = AttestationPreference::None; // Can be configured per request

    let webauthn_service = WebAuthnService::new_with_attestation(
        state.database.clone(),
        rp_id,
        rp_name,
        rp_origin,
        attestation_pref,
    )?;

    webauthn_service
        .generate_registration_challenge(request)
        .await
}

/// WebAuthn registration verification handler with full attestation validation
pub async fn register_verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // Extract username and registration response from payload
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthencError::validation("Missing username"))?;

    let response: RegisterPublicKeyCredential = serde_json::from_value(
        payload
            .get("response")
            .cloned()
            .ok_or_else(|| AuthencError::validation("Missing registration response"))?,
    )
    .map_err(|e| AuthencError::validation(&format!("Invalid registration response: {}", e)))?;

    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name =
        std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "SIMPelv2 Authenc".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "https://10.1.7.121/api/auth".to_string());

    let attestation_pref = AttestationPreference::None;

    let webauthn_service = WebAuthnService::new_with_attestation(
        state.database.clone(),
        rp_id,
        rp_name,
        rp_origin,
        attestation_pref,
    )?;

    webauthn_service
        .verify_registration(username, response)
        .await
}

/// WebAuthn authentication challenge handler
pub async fn authenticate_challenge(
    State(state): State<Arc<AppState>>,
    Json(request): Json<crate::services::webauthn::WebAuthnAuthenticationRequest>,
) -> Result<Json<webauthn_rs::prelude::RequestChallengeResponse>> {
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name =
        std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "SIMPelv2 Authenc".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "https://10.1.7.121/api/auth".to_string());

    let webauthn_service = WebAuthnService::new(state.database.clone(), rp_id, rp_name, rp_origin)?;

    webauthn_service
        .generate_authentication_challenge(request)
        .await
}

/// WebAuthn authentication verification handler with full validation
pub async fn authenticate_verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // Extract username and authentication response from payload
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthencError::validation("Missing username"))?;

    let response: PublicKeyCredential = serde_json::from_value(
        payload
            .get("response")
            .cloned()
            .ok_or_else(|| AuthencError::validation("Missing authentication response"))?,
    )
    .map_err(|e| AuthencError::validation(&format!("Invalid authentication response: {}", e)))?;

    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name =
        std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "SIMPelv2 Authenc".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "https://10.1.7.121/api/auth".to_string());

    let webauthn_service = WebAuthnService::new(state.database.clone(), rp_id, rp_name, rp_origin)?;

    webauthn_service
        .verify_authentication(username, response)
        .await
}
