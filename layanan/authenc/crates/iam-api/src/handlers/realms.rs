//! Realm management HTTP handlers

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Realm response DTO
#[derive(Debug, Serialize)]
pub struct RealmResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create realm request
#[derive(Debug, Deserialize)]
pub struct CreateRealmRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
}

/// Update realm request
#[derive(Debug, Deserialize)]
pub struct UpdateRealmRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
}

/// GET /api/v1/iam/realms - List realms
pub async fn list_realms(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RealmResponse>>> {
    // TODO: Call realm_service.list_realms()
    Ok(Json(vec![]))
}

/// POST /api/v1/iam/realms - Create realm
pub async fn create_realm(
    State(_state): State<Arc<IamApiState>>,
    Json(_req): Json<CreateRealmRequest>,
) -> ApiResult<(StatusCode, Json<RealmResponse>)> {
    // TODO: Call realm_service.create_realm()
    Err(crate::error::ApiError(AuthencError::NotImplemented(
        "create_realm not yet implemented".to_string(),
    )))
}

/// GET /api/v1/iam/realms/{id} - Get realm details
pub async fn get_realm(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<RealmResponse>> {
    // TODO: Call realm_service.get_realm()
    Err(crate::error::ApiError(AuthencError::NotImplemented(
        "get_realm not yet implemented".to_string(),
    )))
}

/// PUT /api/v1/iam/realms/{id} - Update realm
pub async fn update_realm(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateRealmRequest>,
) -> ApiResult<Json<RealmResponse>> {
    // TODO: Call realm_service.update_realm()
    Err(crate::error::ApiError(AuthencError::NotImplemented(
        "update_realm not yet implemented".to_string(),
    )))
}

/// DELETE /api/v1/iam/realms/{id} - Delete realm
pub async fn delete_realm(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Call realm_service.delete_realm()
    Err(crate::error::ApiError(AuthencError::NotImplemented(
        "delete_realm not yet implemented".to_string(),
    )))
}
