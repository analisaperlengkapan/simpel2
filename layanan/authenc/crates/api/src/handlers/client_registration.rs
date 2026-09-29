//! Dynamic Client Registration handlers (RFC 7591/7592)
//!
//! Handles OAuth2 Dynamic Client Registration and Configuration endpoints.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Dynamic Client Registration request (RFC 7591)
#[derive(Debug, Deserialize)]
pub struct ClientRegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub scope: Option<String>,
    pub software_id: Option<String>,
    pub software_version: Option<String>,
}

/// Dynamic Client Registration response
#[derive(Debug, Serialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_id_issued_at: u64,
    pub client_secret_expires_at: u64,
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub token_endpoint_auth_method: String,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub registration_access_token: Option<String>,
    pub registration_client_uri: Option<String>,
}

/// Client configuration update request (RFC 7592)
#[derive(Debug, Deserialize)]
pub struct ClientUpdateRequest {
    pub redirect_uris: Option<Vec<String>>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub scope: Option<String>,
}

/// DCR error response
#[derive(Debug, Serialize)]
pub struct DcrErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

// ===== Handlers =====
//
// RFC 7591/7592 Dynamic Client Registration is NOT implemented. These routes used
// to answer with success-shaped responses — `201 Created` carrying an error body,
// `200` with "stub", and `204 No Content` for a DELETE that deleted nothing — so a
// client (or an operator's script) could conclude it had registered or removed a
// client when the server had done neither. They now say what is true: 501.
//
// Registering a client is an administrative act (`/api/v1/clients`, admin-only);
// opening it to unauthenticated callers would be a design decision needing an
// initial-access-token model, not a stub to be filled in.

fn not_implemented() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "not_implemented",
            "error_description":
                "Dynamic client registration (RFC 7591/7592) is not supported; \
                 clients are registered by an administrator via /api/v1/clients"
        })),
    )
}

/// Register a new client dynamically (RFC 7591) — not implemented.
pub async fn register_client_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<ClientRegistrationRequest>,
) -> impl IntoResponse {
    not_implemented()
}

/// Get client configuration (RFC 7592) — not implemented.
pub async fn get_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
) -> impl IntoResponse {
    not_implemented()
}

/// Update client configuration (RFC 7592) — not implemented.
pub async fn update_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
    Json(_req): Json<ClientUpdateRequest>,
) -> impl IntoResponse {
    not_implemented()
}

/// Delete client configuration (RFC 7592) — not implemented.
pub async fn delete_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
) -> impl IntoResponse {
    not_implemented()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stub that answers success-shaped responses is worse than a missing
    /// endpoint: `201`/`204` tell a caller it registered/removed a client when
    /// the server did nothing.
    #[test]
    fn every_dcr_route_says_it_is_not_implemented() {
        let (status, Json(body)) = not_implemented();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"], "not_implemented");
        assert!(
            body["error_description"]
                .as_str()
                .unwrap()
                .contains("/api/v1/clients"),
            "the response should point at the real (admin) route"
        );
    }
}
