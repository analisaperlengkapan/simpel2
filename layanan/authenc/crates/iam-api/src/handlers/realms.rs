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
use authenc_types::domain::realm::Realm;
use authenc_types::{AuthencError, RealmId};

fn to_realm_response(realm: Realm) -> RealmResponse {
    RealmResponse {
        id: realm.id,
        name: realm.name,
        display_name: realm.display_name,
        enabled: realm.enabled,
        created_at: realm.created_at,
        updated_at: realm.updated_at,
    }
}

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
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RealmResponse>>> {
    let realms = state
        .realm_service
        .list_realms()
        .await
        .map_err(crate::error::ApiError)?;
    let responses: Vec<RealmResponse> = realms.into_iter().map(to_realm_response).collect();
    Ok(Json(responses))
}

/// POST /api/v1/iam/realms - Create realm
pub async fn create_realm(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateRealmRequest>,
) -> ApiResult<(StatusCode, Json<RealmResponse>)> {
    let display_name = req.display_name.unwrap_or_else(|| req.name.clone());
    let realm = state
        .realm_service
        .create_realm(req.name, display_name)
        .await
        .map_err(crate::error::ApiError)?;

    // The create_realm does not let us set `enabled` initially, so update if specified.
    if let Some(false) = req.enabled {
        state
            .realm_service
            .disable_realm(RealmId::from_uuid(realm.id))
            .await
            .map_err(crate::error::ApiError)?;
    }

    // We fetch again to get the updated state
    let final_realm = state
        .realm_service
        .get_realm(RealmId::from_uuid(realm.id))
        .await
        .map_err(crate::error::ApiError)?;
    Ok((StatusCode::CREATED, Json(to_realm_response(final_realm))))
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
        .map_err(crate::error::ApiError)?;
    Ok(Json(to_realm_response(realm)))
}

/// PUT /api/v1/iam/realms/{id} - Update realm
pub async fn update_realm(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRealmRequest>,
) -> ApiResult<Json<RealmResponse>> {
    let mut realm = state
        .realm_service
        .get_realm(RealmId::from_uuid(id))
        .await
        .map_err(crate::error::ApiError)?;

    if let Some(display_name) = req.display_name {
        realm = state
            .realm_service
            .update_realm(RealmId::from_uuid(realm.id), Some(display_name), None)
            .await
            .map_err(crate::error::ApiError)?;
    }

    if let Some(enabled) = req.enabled {
        if enabled {
            realm = state
                .realm_service
                .enable_realm(RealmId::from_uuid(realm.id))
                .await
                .map_err(crate::error::ApiError)?;
        } else {
            realm = state
                .realm_service
                .disable_realm(RealmId::from_uuid(realm.id))
                .await
                .map_err(crate::error::ApiError)?;
        }
    }

    Ok(Json(to_realm_response(realm)))
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
        .map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}
