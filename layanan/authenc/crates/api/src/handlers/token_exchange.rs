//! Token exchange handlers (RFC 8693)
//!
//! Handles OAuth2 token exchange flows for exchanging tokens between
//! different types and audiences.

use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Token exchange request per RFC 8693
#[derive(Debug, Deserialize)]
pub struct TokenExchangeRequest {
    /// The grant type (must be "urn:ietf:params:oauth:grant-type:token-exchange")
    pub grant_type: String,
    /// The subject token to exchange
    pub subject_token: String,
    /// The type of the subject token
    pub subject_token_type: String,
    /// Optional: the actor token
    pub actor_token: Option<String>,
    /// Optional: the type of the actor token
    pub actor_token_type: Option<String>,
    /// Optional: requested token type
    pub requested_token_type: Option<String>,
    /// Optional: target audience
    pub audience: Option<String>,
    /// Optional: requested scopes
    pub scope: Option<String>,
    /// Optional: resource URI
    pub resource: Option<String>,
}

/// Token exchange response per RFC 8693
#[derive(Debug, Serialize)]
pub struct TokenExchangeResponse {
    /// The issued token
    pub access_token: String,
    /// The token type (typically "Bearer")
    pub token_type: String,
    /// Expiration in seconds
    pub expires_in: u64,
    /// The type of the issued token
    pub issued_token_type: String,
    /// Granted scopes (space-delimited)
    pub scope: Option<String>,
    /// Optional refresh token
    pub refresh_token: Option<String>,
}

/// Token exchange error response
#[derive(Debug, Serialize)]
pub struct TokenExchangeError {
    pub error: String,
    pub error_description: Option<String>,
}

// ===== Routes =====

/// Create token exchange routes
pub fn create_token_exchange_routes() -> Router<Arc<ApiState>> {
    Router::new().route("/oauth2/token/exchange", post(exchange_token))
}

// ===== Handlers =====

/// Exchange a token per RFC 8693
async fn exchange_token(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<TokenExchangeRequest>,
) -> impl IntoResponse {
    // TODO: Validate subject token
    // TODO: Check permissions for token exchange
    // TODO: Issue new token with requested type/audience
    Json(serde_json::json!({
        "error": "unsupported_grant_type",
        "error_description": "Token exchange not yet implemented (stub)"
    }))
}
