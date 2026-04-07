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
///
/// Returns the Ed25519 public key used to sign access tokens and ID tokens,
/// enabling relying parties to verify token signatures.
async fn jwks_handler(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let public_key_b64 = state.jwt_service.get_public_key_base64();

    Json(JwksResponse {
        keys: vec![Jwk {
            kty: "OKP".to_string(),
            crv: Some("Ed25519".to_string()),
            x: Some(public_key_b64),
            n: None,
            e: None,
            kid: "authenc-ed25519-key".to_string(),
            use_: "sig".to_string(),
            alg: "EdDSA".to_string(),
        }],
    })
}
