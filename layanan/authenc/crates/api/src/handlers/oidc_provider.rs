//! OIDC provider handlers
//!
//! Handles OpenID Connect provider operations including discovery,
//! authorization, and session management.

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// OIDC authorization request parameters
#[derive(Debug, Deserialize)]
pub struct OidcAuthorizationQuery {
    pub response_type: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<u64>,
    pub login_hint: Option<String>,
}

/// OIDC provider configuration (OpenID Connect Discovery)
#[derive(Debug, Serialize)]
pub struct OidcProviderConfig {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

/// OIDC session information
#[derive(Debug, Serialize)]
pub struct OidcSessionResponse {
    pub session_id: String,
    pub client_id: String,
    pub user_id: String,
    pub scopes: Vec<String>,
    pub authenticated_at: String,
}

// ===== Routes =====

/// Create OIDC provider routes
pub fn create_oidc_provider_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/.well-known/openid-configuration", get(oidc_discovery))
        .route("/oidc/authorize", get(oidc_authorize))
        .route(
            "/oidc/end-session",
            get(oidc_end_session).post(oidc_end_session),
        )
        .route("/oidc/check-session", get(oidc_check_session))
}

// ===== Handlers =====

/// OIDC Discovery endpoint
async fn oidc_discovery(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Build from actual server configuration
    Json(OidcProviderConfig {
        issuer: "https://authenc.example.com".to_string(),
        authorization_endpoint: "https://authenc.example.com/oidc/authorize".to_string(),
        token_endpoint: "https://authenc.example.com/api/v1/oauth2/token".to_string(),
        userinfo_endpoint: "https://authenc.example.com/api/v1/oauth2/userinfo".to_string(),
        jwks_uri: "https://authenc.example.com/api/v1/oauth2/jwks".to_string(),
        registration_endpoint: None,
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        response_types_supported: vec!["code".to_string(), "id_token".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["EdDSA".to_string(), "RS256".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        claims_supported: vec![
            "sub".to_string(),
            "name".to_string(),
            "email".to_string(),
            "iss".to_string(),
        ],
        code_challenge_methods_supported: vec!["S256".to_string()],
    })
}

/// OIDC Authorization endpoint
async fn oidc_authorize(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<OidcAuthorizationQuery>,
) -> impl IntoResponse {
    // TODO: Validate client_id, redirect_uri, generate auth code
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "OIDC authorization (stub)"
    }))
}

/// OIDC End Session endpoint (RP-Initiated Logout)
async fn oidc_end_session(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Handle RP-initiated logout
    Json(serde_json::json!({
        "status": "ok",
        "message": "OIDC end session (stub)"
    }))
}

/// OIDC Check Session iframe endpoint
async fn oidc_check_session(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Return session check iframe HTML
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "OIDC check session (stub)"
    }))
}
