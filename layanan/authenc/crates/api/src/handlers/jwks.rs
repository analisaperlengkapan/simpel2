//! JWKS (JSON Web Key Set) handlers
//!
//! Provides endpoints for serving JSON Web Key Sets for token verification.

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// JWKS response containing public keys
#[derive(Debug, Serialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

/// Individual JSON Web Key
#[derive(Debug, Serialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    pub kid: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
}

// ===== Routes =====

/// Create JWKS routes
pub fn create_jwks_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/jwks", get(jwks_handler))
        .route("/.well-known/jwks.json", get(jwks_handler))
}

// ===== Handlers =====

/// Serve the JSON Web Key Set
async fn jwks_handler(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Return actual public keys from jwt_service
    Json(JwksResponse { keys: vec![] })
}
