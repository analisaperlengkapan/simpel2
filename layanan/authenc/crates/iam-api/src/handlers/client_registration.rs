//! Dynamic Client Registration HTTP handlers (RFC 7591/7592)

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Client registration request (RFC 7591)
#[derive(Debug, Deserialize)]
pub struct ClientRegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub scope: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub jwks_uri: Option<String>,
    pub jwks: Option<serde_json::Value>,
    pub software_id: Option<String>,
    pub software_version: Option<String>,
    pub software_statement: Option<String>,
    #[serde(flatten)]
    pub additional_metadata: serde_json::Value,
}

/// Client registration response (RFC 7591)
#[derive(Debug, Serialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_id_issued_at: i64,
    pub client_secret_expires_at: Option<i64>,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub scope: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub jwks_uri: Option<String>,
    pub jwks: Option<serde_json::Value>,
    pub software_id: Option<String>,
    pub software_version: Option<String>,
    pub registration_access_token: String,
    pub registration_client_uri: String,
}

/// Client update request (RFC 7592)
#[derive(Debug, Deserialize)]
pub struct ClientUpdateRequest {
    pub redirect_uris: Option<Vec<String>>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub scope: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub jwks_uri: Option<String>,
    pub jwks: Option<serde_json::Value>,
}

/// Client registration error (RFC 7591)
#[derive(Debug, Serialize)]
pub struct ClientRegistrationError {
    pub error: String,
    pub error_description: Option<String>,
}

/// POST /api/v1/oauth2/register - Register a new OAuth 2.0 client (RFC 7591)
pub async fn register_client(
    State(_state): State<Arc<IamApiState>>,
    _headers: HeaderMap,
    Json(_request): Json<ClientRegistrationRequest>,
) -> ApiResult<(StatusCode, Json<ClientRegistrationResponse>)> {
    // TODO: Implement client registration
    // - Validate initial access token from Authorization header
    // - Validate redirect URIs
    // - Generate client_id and client_secret
    // - Store client in database
    // - Generate registration access token
    // - Return client configuration
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Dynamic Client Registration not yet implemented. Requires client_registration_service in IamApiState.",
    )))
}

/// GET /api/v1/oauth2/register/{client_id} - Get client configuration (RFC 7592)
pub async fn get_client_configuration(
    State(_state): State<Arc<IamApiState>>,
    _headers: HeaderMap,
    Path(_client_id): Path<String>,
) -> ApiResult<Json<ClientRegistrationResponse>> {
    // TODO: Implement get client configuration
    // - Validate registration access token
    // - Retrieve client from database
    // - Return client configuration
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Dynamic Client Registration not yet implemented. Requires client_registration_service in IamApiState.",
    )))
}

/// PUT /api/v1/oauth2/register/{client_id} - Update client configuration (RFC 7592)
pub async fn update_client_configuration(
    State(_state): State<Arc<IamApiState>>,
    _headers: HeaderMap,
    Path(_client_id): Path<String>,
    Json(_request): Json<ClientUpdateRequest>,
) -> ApiResult<Json<ClientRegistrationResponse>> {
    // TODO: Implement update client configuration
    // - Validate registration access token
    // - Update client in database
    // - Return updated client configuration
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Dynamic Client Registration not yet implemented. Requires client_registration_service in IamApiState.",
    )))
}

/// DELETE /api/v1/oauth2/register/{client_id} - Delete client registration (RFC 7592)
pub async fn delete_client_registration(
    State(_state): State<Arc<IamApiState>>,
    _headers: HeaderMap,
    Path(_client_id): Path<String>,
) -> ApiResult<StatusCode> {
    // TODO: Implement delete client registration
    // - Validate registration access token
    // - Delete client from database
    // - Revoke all tokens
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Dynamic Client Registration not yet implemented. Requires client_registration_service in IamApiState.",
    )))
}
