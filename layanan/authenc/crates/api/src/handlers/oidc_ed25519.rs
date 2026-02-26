//! OIDC Ed25519 handlers
//!
//! Handles OIDC operations using Ed25519 key pairs for token signing.
//! Provides JWKS endpoints and token verification with EdDSA.

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Ed25519 JWKS response
#[derive(Debug, Serialize)]
pub struct Ed25519JwksResponse {
    pub keys: Vec<Ed25519Jwk>,
}

/// Individual Ed25519 JWK
#[derive(Debug, Serialize)]
pub struct Ed25519Jwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
    pub kid: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
}

/// Request to rotate Ed25519 keys
#[derive(Debug, Deserialize)]
pub struct RotateKeysRequest {
    pub reason: Option<String>,
}

/// Key rotation response
#[derive(Debug, Serialize)]
pub struct RotateKeysResponse {
    pub new_kid: String,
    pub rotated_at: String,
    pub previous_kid: Option<String>,
}

/// Token verification request
#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

/// Token verification response
#[derive(Debug, Serialize)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub claims: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ===== Routes =====

/// Create OIDC Ed25519 routes
pub fn create_oidc_ed25519_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/oidc/ed25519/jwks", get(ed25519_jwks))
        .route("/oidc/ed25519/verify", post(verify_ed25519_token))
        .route("/oidc/ed25519/rotate", post(rotate_ed25519_keys))
}

// ===== Handlers =====

/// Get Ed25519 JWKS (JSON Web Key Set)
async fn ed25519_jwks(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Return current Ed25519 public keys from jwt_service
    Json(Ed25519JwksResponse { keys: vec![] })
}

/// Verify a token signed with Ed25519
async fn verify_ed25519_token(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    // TODO: Verify Ed25519-signed JWT using jwt_service
    Json(VerifyTokenResponse {
        valid: false,
        claims: None,
        error: Some("Not implemented (stub)".to_string()),
    })
}

/// Rotate Ed25519 signing keys
async fn rotate_ed25519_keys(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<RotateKeysRequest>,
) -> impl IntoResponse {
    // TODO: Generate new Ed25519 key pair, archive old key
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "Key rotation (stub)"
    }))
}
