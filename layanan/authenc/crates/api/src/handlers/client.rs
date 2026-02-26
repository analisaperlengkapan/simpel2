//! Client management handlers
//!
//! Handles OAuth2 client CRUD operations for admin users.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Query parameters for listing clients
#[derive(Debug, Deserialize)]
pub struct ListClientsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
}

/// Request to create a new OAuth2 client
#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub scope: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
}

/// Response after creating a client
#[derive(Debug, Serialize)]
pub struct CreateClientResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub created_at: String,
}

/// Request to update an existing client
#[derive(Debug, Deserialize)]
pub struct UpdateClientRequest {
    pub client_name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub scope: Option<String>,
}

/// Client information response
#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub client_id: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub scope: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// Response for listing clients
#[derive(Debug, Serialize)]
pub struct ListClientsResponse {
    pub clients: Vec<ClientResponse>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

// ===== Handlers =====

/// List all OAuth2 clients (admin only)
pub async fn list_clients_handler(
    State(_state): State<Arc<ApiState>>,
    Query(_query): Query<ListClientsQuery>,
) -> impl IntoResponse {
    // TODO: Query clients from client_service
    Json(ListClientsResponse {
        clients: vec![],
        total: 0,
        page: 1,
        per_page: 20,
    })
}

/// Create a new OAuth2 client (admin only)
pub async fn create_client_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<CreateClientRequest>,
) -> impl IntoResponse {
    // TODO: Create client via client_service
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "not_implemented",
            "message": "Create client (stub)"
        })),
    )
}

/// Get a specific OAuth2 client by ID
pub async fn get_client_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Fetch client from client_service
    Json(serde_json::json!({
        "status": "not_found",
        "message": "Client not found (stub)"
    }))
}

/// Update an existing OAuth2 client (admin only)
pub async fn update_client_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
    Json(_req): Json<UpdateClientRequest>,
) -> impl IntoResponse {
    // TODO: Update client via client_service
    Json(serde_json::json!({
        "status": "ok",
        "message": "Client updated (stub)"
    }))
}

/// Delete an OAuth2 client (admin only)
pub async fn delete_client_handler(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Delete client via client_service
    StatusCode::NO_CONTENT
}
