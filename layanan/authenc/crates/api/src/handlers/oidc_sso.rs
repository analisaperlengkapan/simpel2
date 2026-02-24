//! OIDC SSO handlers
//!
//! Handles OpenID Connect-based Single Sign-On flows.

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// OIDC SSO login parameters
#[derive(Debug, Deserialize)]
pub struct OidcSsoLoginQuery {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub prompt: Option<String>,
}

/// OIDC SSO status response
#[derive(Debug, Serialize)]
pub struct OidcSsoStatusResponse {
    pub active: bool,
    pub user_id: Option<String>,
    pub session_expiry: Option<String>,
}

/// OIDC SSO callback parameters
#[derive(Debug, Deserialize)]
pub struct OidcSsoCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

// ===== Routes =====

/// Create OIDC SSO routes
pub fn create_oidc_sso_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/oidc/sso/login", get(oidc_sso_login))
        .route("/oidc/sso/callback", get(oidc_sso_callback))
        .route("/oidc/sso/status", get(oidc_sso_status))
        .route("/oidc/sso/logout", post(oidc_sso_logout))
}

// ===== Handlers =====

/// Initiate OIDC SSO login
async fn oidc_sso_login(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<OidcSsoLoginQuery>,
) -> impl IntoResponse {
    // TODO: Build auth request URL, redirect to IdP
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "OIDC SSO login (stub)"
    }))
}

/// OIDC SSO callback handler
async fn oidc_sso_callback(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<OidcSsoCallbackQuery>,
) -> impl IntoResponse {
    // TODO: Exchange code, validate tokens, create session
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "OIDC SSO callback (stub)"
    }))
}

/// Check OIDC SSO status
async fn oidc_sso_status(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    Json(OidcSsoStatusResponse {
        active: false,
        user_id: None,
        session_expiry: None,
    })
}

/// OIDC SSO logout
async fn oidc_sso_logout(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Destroy SSO session, notify RPs
    Json(serde_json::json!({
        "status": "ok",
        "message": "OIDC SSO logout (stub)"
    }))
}
