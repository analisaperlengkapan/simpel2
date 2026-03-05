//! Group management HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Query parameters for listing groups
#[derive(Debug, Deserialize)]
pub struct GroupListQuery {
    #[serde(default)]
    pub first: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
    pub search: Option<String>,
}

/// Create group request
#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub realm_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

/// Update group request
#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
    pub description: Option<Option<String>>,
    pub attributes: Option<serde_json::Value>,
}

/// Group response
#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub attributes: serde_json::Value,
    pub member_count: i64,
    pub subgroup_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Add member request
#[derive(Debug, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: Uuid,
}

/// Helper to get group_service or return error
fn require_group_service(
    state: &IamApiState,
) -> std::result::Result<&authenc_core::services::group_store::GroupStore, crate::error::ApiError> {
    state
        .group_service
        .as_ref()
        .map(|s| s.as_ref())
        .ok_or_else(|| {
            crate::error::ApiError(AuthencError::not_implemented(
                "Group management service not configured",
            ))
        })
}

/// POST /api/v1/iam/groups - Create a new group
pub async fn create_group(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateGroupRequest>,
) -> ApiResult<(StatusCode, Json<GroupResponse>)> {
    let svc = require_group_service(&state)?;
    let attrs = req.attributes.unwrap_or_else(|| serde_json::json!({}));
    let group = svc
        .create(
            req.realm_id,
            &req.name,
            req.parent_id,
            req.description.as_deref(),
            &attrs,
        )
        .await
        .map_err(crate::error::ApiError)?;

    let member_count = svc.member_count(group.id).await.unwrap_or(0);
    let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);

    Ok((
        StatusCode::CREATED,
        Json(GroupResponse {
            id: group.id,
            realm_id: group.realm_id,
            name: group.name,
            parent_id: group.parent_id,
            description: group.description,
            attributes: group.attributes,
            member_count,
            subgroup_count,
            created_at: group.created_at,
            updated_at: group.updated_at,
        }),
    ))
}

/// GET /api/v1/iam/groups - List all groups (alias for realm groups with default realm)
pub async fn list_groups(
    State(state): State<Arc<IamApiState>>,
    Query(query): Query<GroupListQuery>,
) -> ApiResult<Json<Vec<GroupResponse>>> {
    let svc = require_group_service(&state)?;
    // Use a default realm UUID if none specified - list first realm's groups
    let groups = svc
        .list_by_realm(Uuid::nil(), query.first, query.max)
        .await
        .map_err(crate::error::ApiError)?;

    let mut responses = Vec::new();
    for group in groups {
        let member_count = svc.member_count(group.id).await.unwrap_or(0);
        let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);
        responses.push(GroupResponse {
            id: group.id,
            realm_id: group.realm_id,
            name: group.name,
            parent_id: group.parent_id,
            description: group.description,
            attributes: group.attributes,
            member_count,
            subgroup_count,
            created_at: group.created_at,
            updated_at: group.updated_at,
        });
    }
    Ok(Json(responses))
}

/// GET /api/v1/iam/realms/{realm_id}/groups - Get all groups in a realm
pub async fn get_groups(
    State(state): State<Arc<IamApiState>>,
    Path(realm_id): Path<Uuid>,
    Query(query): Query<GroupListQuery>,
) -> ApiResult<Json<Vec<GroupResponse>>> {
    let svc = require_group_service(&state)?;
    let groups = svc
        .list_by_realm(realm_id, query.first, query.max)
        .await
        .map_err(crate::error::ApiError)?;

    let mut responses = Vec::new();
    for group in groups {
        let member_count = svc.member_count(group.id).await.unwrap_or(0);
        let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);
        responses.push(GroupResponse {
            id: group.id,
            realm_id: group.realm_id,
            name: group.name,
            parent_id: group.parent_id,
            description: group.description,
            attributes: group.attributes,
            member_count,
            subgroup_count,
            created_at: group.created_at,
            updated_at: group.updated_at,
        });
    }
    Ok(Json(responses))
}

/// GET /api/v1/iam/groups/{group_id} - Get a specific group by ID
pub async fn get_group_by_id(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
) -> ApiResult<Json<GroupResponse>> {
    let svc = require_group_service(&state)?;
    let group = svc
        .get(&group_id)
        .await
        .map_err(crate::error::ApiError)?
        .ok_or_else(|| crate::error::ApiError(AuthencError::not_found("Group not found")))?;

    let member_count = svc.member_count(group.id).await.unwrap_or(0);
    let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);

    Ok(Json(GroupResponse {
        id: group.id,
        realm_id: group.realm_id,
        name: group.name,
        parent_id: group.parent_id,
        description: group.description,
        attributes: group.attributes,
        member_count,
        subgroup_count,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }))
}

/// PUT /api/v1/iam/groups/{group_id} - Update a group
pub async fn update_group(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> ApiResult<Json<GroupResponse>> {
    let svc = require_group_service(&state)?;
    let group = svc
        .update(group_id, req.name, req.parent_id, req.description, req.attributes)
        .await
        .map_err(crate::error::ApiError)?;

    let member_count = svc.member_count(group.id).await.unwrap_or(0);
    let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);

    Ok(Json(GroupResponse {
        id: group.id,
        realm_id: group.realm_id,
        name: group.name,
        parent_id: group.parent_id,
        description: group.description,
        attributes: group.attributes,
        member_count,
        subgroup_count,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }))
}

/// DELETE /api/v1/iam/groups/{group_id} - Delete a group
pub async fn delete_group(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let svc = require_group_service(&state)?;
    svc.delete(&group_id)
        .await
        .map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/iam/groups/{group_id}/subgroups - Get subgroups of a group
pub async fn get_subgroups(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
    Query(_query): Query<GroupListQuery>,
) -> ApiResult<Json<Vec<GroupResponse>>> {
    let svc = require_group_service(&state)?;
    let groups = svc
        .subgroups(group_id)
        .await
        .map_err(crate::error::ApiError)?;

    let mut responses = Vec::new();
    for group in groups {
        let member_count = svc.member_count(group.id).await.unwrap_or(0);
        let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);
        responses.push(GroupResponse {
            id: group.id,
            realm_id: group.realm_id,
            name: group.name,
            parent_id: group.parent_id,
            description: group.description,
            attributes: group.attributes,
            member_count,
            subgroup_count,
            created_at: group.created_at,
            updated_at: group.updated_at,
        });
    }
    Ok(Json(responses))
}

/// POST /api/v1/iam/groups/{group_id}/members - Add user to group
pub async fn add_group_member(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<AddGroupMemberRequest>,
) -> ApiResult<StatusCode> {
    let svc = require_group_service(&state)?;
    svc.add_member(group_id, req.user_id)
        .await
        .map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/iam/groups/{group_id}/members/{user_id} - Remove user from group
pub async fn remove_group_member(
    State(state): State<Arc<IamApiState>>,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let svc = require_group_service(&state)?;
    svc.remove_member(group_id, user_id)
        .await
        .map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/iam/groups/{group_id}/members - Get members of a group
pub async fn get_group_members(
    State(state): State<Arc<IamApiState>>,
    Path(group_id): Path<Uuid>,
    Query(query): Query<GroupListQuery>,
) -> ApiResult<Json<Vec<Uuid>>> {
    let svc = require_group_service(&state)?;
    let members = svc
        .members(group_id, query.first, query.max)
        .await
        .map_err(crate::error::ApiError)?;
    Ok(Json(members))
}

/// GET /api/v1/iam/users/{user_id}/groups - Get user's groups
pub async fn get_user_groups(
    State(state): State<Arc<IamApiState>>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Json<Vec<GroupResponse>>> {
    let svc = require_group_service(&state)?;
    let groups = svc
        .user_groups(user_id)
        .await
        .map_err(crate::error::ApiError)?;

    let mut responses = Vec::new();
    for group in groups {
        let member_count = svc.member_count(group.id).await.unwrap_or(0);
        let subgroup_count = svc.subgroup_count(group.id).await.unwrap_or(0);
        responses.push(GroupResponse {
            id: group.id,
            realm_id: group.realm_id,
            name: group.name,
            parent_id: group.parent_id,
            description: group.description,
            attributes: group.attributes,
            member_count,
            subgroup_count,
            created_at: group.created_at,
            updated_at: group.updated_at,
        });
    }
    Ok(Json(responses))
}
