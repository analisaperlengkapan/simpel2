//! Role management HTTP handlers

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
use authenc_types::{AuthencError, RealmId, RoleId, UserId};


/// Helper to get role_service or return error
fn require_role_service(
    state: &IamApiState,
) -> std::result::Result<&authenc_core::services::RoleManagementServiceImpl, crate::error::ApiError> {
    state
        .role_service
        .as_ref()
        .map(|s| s.as_ref())
        .ok_or_else(|| {
            crate::error::ApiError(AuthencError::not_implemented(
                "Role management service not configured",
            ))
        })
}


/// Role response DTO
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub realm_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create role request
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub realm_id: Uuid,
}

/// Update role request
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// GET /api/v1/iam/roles - List roles
pub async fn list_roles(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    let svc = require_role_service(&state)?;
    let roles = svc.list_roles(None).await.map_err(crate::error::ApiError)?;

    let responses = roles.into_iter().map(|r| RoleResponse {
        id: *r.id.as_uuid(),
        name: r.name,
        description: r.description,
        realm_id: *r.realm_id.as_uuid(),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }).collect();

    Ok(Json(responses))
}

/// POST /api/v1/iam/roles - Create role
pub async fn create_role(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateRoleRequest>,
) -> ApiResult<(StatusCode, Json<RoleResponse>)> {
    let svc = require_role_service(&state)?;
    let role = svc.create_role(&req.name, req.description.as_deref(), RealmId::from_uuid(req.realm_id)).await.map_err(crate::error::ApiError)?;

    Ok((
        StatusCode::CREATED,
        Json(RoleResponse {
            id: *role.id.as_uuid(),
            name: role.name,
            description: role.description,
            realm_id: *role.realm_id.as_uuid(),
            created_at: role.created_at,
            updated_at: role.updated_at,
        })
    ))
}

/// GET /api/v1/iam/roles/{id} - Get role by ID
pub async fn get_role(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RoleResponse>> {
    let svc = require_role_service(&state)?;
    let role = svc.get_role(RoleId::from_uuid(id)).await.map_err(crate::error::ApiError)?
        .ok_or_else(|| crate::error::ApiError(AuthencError::not_found("Role not found")))?;

    Ok(Json(RoleResponse {
        id: *role.id.as_uuid(),
        name: role.name,
        description: role.description,
        realm_id: *role.realm_id.as_uuid(),
        created_at: role.created_at,
        updated_at: role.updated_at,
    }))
}

/// PUT /api/v1/iam/roles/{id} - Update role
pub async fn update_role(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> ApiResult<Json<RoleResponse>> {
    let svc = require_role_service(&state)?;
    let desc_opt = match &req.description {
        Some(d) => Some(Some(d.as_str())),
        None => None
    };

    let role = svc.update_role(RoleId::from_uuid(id), req.name.as_deref(), desc_opt).await.map_err(crate::error::ApiError)?;

    Ok(Json(RoleResponse {
        id: *role.id.as_uuid(),
        name: role.name,
        description: role.description,
        realm_id: *role.realm_id.as_uuid(),
        created_at: role.created_at,
        updated_at: role.updated_at,
    }))
}

/// DELETE /api/v1/iam/roles/{id} - Delete role
pub async fn delete_role(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let svc = require_role_service(&state)?;
    svc.delete_role(RoleId::from_uuid(id)).await.map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{user_id}/roles/{role_id} - Assign role to user
pub async fn assign_role_to_user(
    State(state): State<Arc<IamApiState>>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let svc = require_role_service(&state)?;
    svc.assign_role(UserId::from_uuid(user_id), RoleId::from_uuid(role_id)).await.map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/iam/users/{user_id}/roles/{role_id} - Remove role from user
pub async fn remove_role_from_user(
    State(state): State<Arc<IamApiState>>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let svc = require_role_service(&state)?;
    svc.remove_role_from_user(UserId::from_uuid(user_id), RoleId::from_uuid(role_id)).await.map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}
