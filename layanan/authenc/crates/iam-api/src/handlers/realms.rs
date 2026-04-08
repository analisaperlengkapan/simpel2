//! Realm management HTTP handlers

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::IamApiState;
pub use authenc_types::domain::realm::RealmResponse;
use authenc_types::{AuthencError, RealmId};

/// Create realm request — scoped to fields the service actually supports.
#[derive(Debug, Deserialize)]
pub struct CreateRealmRequest {
    pub name: String,
    pub display_name: Option<String>,
}

/// Update realm request — scoped to fields the service actually supports.
#[derive(Debug, Deserialize)]
pub struct UpdateRealmRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
}

/// GET /api/v1/iam/realms - List realms
pub async fn list_realms(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RealmResponse>>> {
    let realms = state
        .realm_service
        .list_realms()
        .await
        .map_err(AuthencError::from)
        .map_err(ApiError)?;

    let response = realms.into_iter().map(RealmResponse::from).collect();

    Ok(Json(response))
}

/// POST /api/v1/iam/realms - Create realm
pub async fn create_realm(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateRealmRequest>,
) -> ApiResult<(StatusCode, Json<RealmResponse>)> {
    let realm = state
        .realm_service
        .create_realm(req.name.clone(), req.display_name.unwrap_or_else(|| req.name))
        .await
        .map_err(AuthencError::from)
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(RealmResponse::from(realm))))
}

/// GET /api/v1/iam/realms/{id} - Get realm details
pub async fn get_realm(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RealmResponse>> {
    let realm = state
        .realm_service
        .get_realm(RealmId::from_uuid(id))
        .await
        .map_err(AuthencError::from)
        .map_err(ApiError)?;

    Ok(Json(RealmResponse::from(realm)))
}

/// PUT /api/v1/iam/realms/{id} - Update realm
pub async fn update_realm(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRealmRequest>,
) -> ApiResult<Json<RealmResponse>> {
    let realm = state
        .realm_service
        .update_realm(RealmId::from_uuid(id), req.display_name, req.enabled)
        .await
        .map_err(AuthencError::from)
        .map_err(ApiError)?;

    Ok(Json(RealmResponse::from(realm)))
}

/// DELETE /api/v1/iam/realms/{id} - Delete realm
pub async fn delete_realm(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .realm_service
        .delete_realm(RealmId::from_uuid(id))
        .await
        .map_err(AuthencError::from)
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}
