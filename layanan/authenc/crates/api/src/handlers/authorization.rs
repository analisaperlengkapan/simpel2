//! Authorization handlers
//!
//! Handles authorization checks and permission evaluation.

use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Authorization check request
#[derive(Debug, Deserialize)]
pub struct AuthzCheckRequest {
    pub subject: String,
    pub resource: String,
    pub action: String,
    pub context: Option<serde_json::Value>,
}

/// Authorization check response
#[derive(Debug, Serialize)]
pub struct AuthzCheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Batch authorization check request
#[derive(Debug, Deserialize)]
pub struct BatchAuthzRequest {
    pub checks: Vec<AuthzCheckRequest>,
}

/// Batch authorization check response
#[derive(Debug, Serialize)]
pub struct BatchAuthzResponse {
    pub results: Vec<AuthzCheckResponse>,
}

// ===== Routes =====

/// Create authorization routes
pub fn create_authorization_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/authz/check", post(check_authorization))
        .route("/authz/batch", post(batch_check_authorization))
}

// ===== Handlers =====

/// Check if a subject is authorized to perform an action on a resource
async fn check_authorization(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<AuthzCheckRequest>,
) -> impl IntoResponse {
    // TODO: Evaluate authorization policy
    Json(AuthzCheckResponse {
        allowed: false,
        reason: Some("Authorization not implemented (stub)".to_string()),
    })
}

/// Batch check authorization for multiple requests
async fn batch_check_authorization(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<BatchAuthzRequest>,
) -> impl IntoResponse {
    // TODO: Evaluate batch authorization
    Json(BatchAuthzResponse { results: vec![] })
}
