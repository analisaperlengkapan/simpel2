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

/// Register a new client dynamically (RFC 7591)
pub async fn register_client_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<ClientRegistrationRequest>,
) -> impl IntoResponse {
    // TODO: Validate request, create client, return registration response
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "error": "not_implemented",
            "error_description": "Dynamic client registration (stub)"
        })),
    )
}

/// Get client configuration (RFC 7592)
pub async fn get_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Return client configuration
    Json(serde_json::json!({
        "error": "not_found",
        "error_description": "Client not found (stub)"
    }))
}

/// Update client configuration (RFC 7592)
pub async fn update_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
    Json(_req): Json<ClientUpdateRequest>,
) -> impl IntoResponse {
    // TODO: Update client configuration
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "Client configuration update (stub)"
    }))
}

/// Delete client configuration (RFC 7592)
pub async fn delete_client_configuration_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_client_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Delete client registration
    StatusCode::NO_CONTENT
}
