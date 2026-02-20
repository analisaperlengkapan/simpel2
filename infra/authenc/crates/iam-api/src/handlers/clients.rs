//! OAuth2 client management HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::IamApiState;
use authenc_types::AuthencError;
use crate::error::ApiResult;

/// OAuth2 client response DTO
#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: Uuid,
    pub client_id: String,
    pub client_name: String,
    pub client_type: String, // "public" or "confidential"
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub enabled: bool,
    pub realm_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create client request
#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub client_name: String,
    pub client_type: String, // "public" or "confidential"
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub realm_id: Uuid,
}

/// Update client request
#[derive(Debug, Deserialize)]
pub struct UpdateClientRequest {
    pub client_name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub allowed_scopes: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// Client secret response
#[derive(Debug, Serialize)]
pub struct ClientSecretResponse {
    pub client_id: String,
    pub client_secret: String,
}

/// GET /api/v1/iam/clients - List clients
pub async fn list_clients(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<ClientResponse>>> {
    // TODO: Call client_service.list_clients()
    Ok(Json(vec![]))
}

/// POST /api/v1/iam/clients - Create client
pub async fn create_client(
    State(_state): State<Arc<IamApiState>>,
    Json(_req): Json<CreateClientRequest>,
) -> ApiResult<(StatusCode, Json<ClientResponse>)> {
    // TODO: Call client_service.create_client()
    Err(crate::error::ApiError(AuthencError::NotImplemented("create_client not yet implemented".to_string()))))
}

/// GET /api/v1/iam/clients/{id} - Get client details
pub async fn get_client(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<ClientResponse>> {
    // TODO: Call client_service.get_client()
    Err(crate::error::ApiError(AuthencError::NotImplemented("get_client not yet implemented".to_string()))))
}

/// PUT /api/v1/iam/clients/{id} - Update client
pub async fn update_client(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateClientRequest>,
) -> ApiResult<Json<ClientResponse>> {
    // TODO: Call client_service.update_client()
    Err(crate::error::ApiError(AuthencError::NotImplemented("update_client not yet implemented".to_string()))))
}

/// DELETE /api/v1/iam/clients/{id} - Delete client
pub async fn delete_client(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Call client_service.delete_client()
    Err(crate::error::ApiError(AuthencError::NotImplemented("delete_client not yet implemented".to_string()))))
}

/// POST /api/v1/iam/clients/{id}/secret/regenerate - Regenerate client secret
pub async fn regenerate_client_secret(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<ClientSecretResponse>> {
    // TODO: Call client_service.regenerate_secret()
    Err(crate::error::ApiError(AuthencError::NotImplemented("regenerate_secret not yet implemented".to_string()))))
}
