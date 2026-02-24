//! SSO (Single Sign-On) handlers
//!
//! Handles SSO flows including session sharing and cookie-based SSO.

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

/// SSO initiation query
#[derive(Debug, Deserialize)]
pub struct SsoInitQuery {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
}

/// SSO session info
#[derive(Debug, Serialize)]
pub struct SsoSessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub authenticated: bool,
    pub clients: Vec<String>,
}

/// SSO logout request
#[derive(Debug, Deserialize)]
pub struct SsoLogoutRequest {
    pub session_id: Option<String>,
    pub logout_all: Option<bool>,
}

// ===== Routes =====

/// Create SSO routes
pub fn create_sso_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/sso/init", get(sso_init))
        .route("/sso/session", get(sso_session_info))
        .route("/sso/logout", post(sso_logout))
}

// ===== Handlers =====

/// Initialize a single sign-on session
async fn sso_init(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<SsoInitQuery>,
) -> impl IntoResponse {
    // TODO: Check existing SSO session, redirect if already authenticated
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "SSO init (stub)"
    }))
}

/// Get current SSO session information
async fn sso_session_info(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Look up SSO session from cookie
    Json(serde_json::json!({
        "authenticated": false,
        "message": "SSO session info (stub)"
    }))
}

/// Logout from SSO (single logout across all clients)
async fn sso_logout(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<SsoLogoutRequest>,
) -> impl IntoResponse {
    // TODO: Destroy SSO session, notify all clients
    Json(serde_json::json!({
        "status": "ok",
        "message": "SSO logout (stub)"
    }))
}
