//! Consent UI handlers
//!
//! Handles user consent flows for OAuth2/OIDC authorization.
//! These endpoints render consent pages and process consent decisions.

use axum::{
    Json, Router,
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Query parameters for the consent page
#[derive(Debug, Deserialize)]
pub struct ConsentQuery {
    /// The OAuth2 client requesting consent
    pub client_id: Option<String>,
    /// Space-separated scopes being requested
    pub scope: Option<String>,
    /// Redirect URI to return to after consent
    pub redirect_uri: Option<String>,
    /// State parameter for CSRF protection
    pub state: Option<String>,
    /// Nonce for OIDC flows
    pub nonce: Option<String>,
    /// Response type (code, token, etc.)
    pub response_type: Option<String>,
}

/// The user's consent decision
#[derive(Debug, Deserialize)]
pub struct ConsentDecision {
    /// Whether consent was granted
    pub approved: bool,
    /// The scopes the user approved (subset of requested)
    pub approved_scopes: Option<Vec<String>>,
    /// The original client_id
    pub client_id: String,
    /// The redirect URI
    pub redirect_uri: Option<String>,
    /// The CSRF state
    pub state: Option<String>,
}

/// Grant information for displaying consent
#[derive(Debug, Serialize)]
pub struct ConsentGrant {
    /// Client application name
    pub client_name: String,
    /// Client application description
    pub client_description: Option<String>,
    /// Requested scopes with descriptions
    pub scopes: Vec<ScopeInfo>,
    /// Whether this is a re-consent
    pub previously_granted: bool,
}

/// Scope information returned in consent grants
#[derive(Debug, Serialize)]
pub struct ScopeInfo {
    /// Scope name
    pub name: String,
    /// Human-readable description
    pub description: String,
}

/// Response for consent status checks
#[derive(Debug, Serialize)]
pub struct ConsentStatusResponse {
    /// Whether consent has already been granted
    pub consented: bool,
    /// Previously granted scopes
    pub granted_scopes: Vec<String>,
}

// ===== Routes =====

/// Create consent UI routes
pub fn create_consent_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/consent", get(show_consent_page))
        .route("/consent", post(process_consent_decision))
        .route("/consent/status", get(check_consent_status))
        .route("/consent/revoke", post(revoke_consent))
        .route("/consent/grants", get(list_consent_grants))
}

// ===== Handlers =====

/// Show the consent page for an OAuth2 authorization request
async fn show_consent_page(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<ConsentQuery>,
) -> impl IntoResponse {
    // TODO: Look up client info from state.client_service
    // TODO: Build consent page HTML with client name, requested scopes
    Html("<html><body><h1>Consent Required</h1><p>TODO: Implement consent UI</p></body></html>")
}

/// Process the user's consent decision
async fn process_consent_decision(
    State(_state): State<Arc<ApiState>>,
    Json(_decision): Json<ConsentDecision>,
) -> impl IntoResponse {
    // TODO: Store consent decision
    // TODO: Generate authorization code if approved
    // TODO: Redirect to client redirect_uri
    Json(serde_json::json!({
        "status": "ok",
        "message": "Consent processed (stub)"
    }))
}

/// Check if consent has already been granted for a client/scope combination
async fn check_consent_status(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<ConsentQuery>,
) -> impl IntoResponse {
    // TODO: Check stored consents for user/client pair
    Json(ConsentStatusResponse {
        consented: false,
        granted_scopes: vec![],
    })
}

/// Revoke previously granted consent for a client
async fn revoke_consent(
    State(_state): State<Arc<ApiState>>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    // TODO: Remove stored consent
    Json(serde_json::json!({
        "status": "ok",
        "message": "Consent revoked (stub)"
    }))
}

/// List all active consent grants for the current user
async fn list_consent_grants(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Fetch consent grants from storage
    let grants: Vec<ConsentGrant> = vec![];
    Json(grants)
}
