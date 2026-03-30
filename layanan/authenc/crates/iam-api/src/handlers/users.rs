//! User management HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::ApiResult, state::IamApiState};
use authenc_types::{RealmId, UserId};

/// Master realm UUID constant (from migration 001)
const MASTER_REALM_ID: Uuid = Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// Search query (username, email, nip, nama)
    pub search: Option<String>,

    /// Filter by enabled status
    pub enabled: Option<bool>,

    /// Filter by realm ID
    pub realm_id: Option<Uuid>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}

/// Paginated user list response
#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// User response DTO (enriched with pegawai fields)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub satker_code: String,
    pub require_password_change: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub realm_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub enabled: Option<bool>,
    pub realm_id: Option<Uuid>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub satker_code: Option<String>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub satker_code: Option<String>,
    pub require_password_change: Option<bool>,
}

/// Reset password request
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

/// Enable MFA response
#[derive(Debug, Serialize)]
pub struct EnableMfaResponse {
    pub secret: String,
    pub qr_code: String,
}

/// Convert domain User to API UserResponse DTO
fn user_to_response(user: &authenc_types::domain::user::User) -> UserResponse {
    UserResponse {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        enabled: user.enabled,
        email_verified: user.email_verified,
        mfa_enabled: user.mfa_enabled,
        nip: user.nip.clone(),
        nama: user.nama.clone(),
        jabatan: user.jabatan.clone(),
        satker_code: user.satker_code.clone(),
        require_password_change: user.require_password_change,
        last_login_at: user.last_login_at,
        realm_id: user.realm_id,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

/// GET /api/v1/iam/users - List users with pagination and search
pub async fn list_users(
    State(state): State<Arc<IamApiState>>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<PaginatedUsers>> {
    let realm_id = RealmId::from_uuid(params.realm_id.unwrap_or(MASTER_REALM_ID));
    let page_size = params.page_size.min(100).max(1);
    let page = params.page.max(1);
    let offset = ((page - 1) * page_size) as usize;
    let limit = page_size as usize;

    let (users, total) = if let Some(ref search) = params.search {
        state
            .user_service
            .search_users_paginated(realm_id, search, params.enabled, offset, limit)
            .await
            .map_err(crate::error::ApiError)?
    } else {
        let total = state
            .user_service
            .count_users_filtered(realm_id, params.enabled)
            .await
            .map_err(crate::error::ApiError)?;
        let users = state
            .user_service
            .list_users_filtered(realm_id, params.enabled, offset, limit)
            .await
            .map_err(crate::error::ApiError)?;
        (users, total)
    };

    let total_pages = if total > 0 {
        ((total as u32) + page_size - 1) / page_size
    } else {
        0
    };

    let response = PaginatedUsers {
        users: users.iter().map(|u| user_to_response(u)).collect(),
        total,
        page,
        page_size,
        total_pages,
    };

    Ok(Json(response))
}

/// POST /api/v1/iam/users - Create user
pub async fn create_user(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<(StatusCode, Json<UserResponse>)> {
    let create_req = authenc_types::domain::CreateUserRequest {
        username: req.username,
        email: req.email,
        password: req.password,
        realm_id: Some(req.realm_id.unwrap_or(MASTER_REALM_ID)),
        satker_code: req.satker_code.unwrap_or_default(),
        first_name: None,
        last_name: None,
        nip: req.nip,
        nama: req.nama,
        jabatan: req.jabatan,
        phone_number: None,
        organization_id: None,
        roles: None,
        attributes: None,
        enabled: req.enabled,
    };

    let user = state
        .user_service
        .create_user(create_req)
        .await
        .map_err(crate::error::ApiError)?;

    Ok((StatusCode::CREATED, Json(user_to_response(&user))))
}

/// GET /api/v1/iam/users/{id} - Get user details
pub async fn get_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    let user = state
        .user_service
        .get_user(UserId::from_uuid(id))
        .await
        .map_err(crate::error::ApiError)?;

    Ok(Json(user_to_response(&user)))
}

/// PUT /api/v1/iam/users/{id} - Update user
pub async fn update_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    let update_req = authenc_types::domain::UpdateUserRequest {
        username: None,
        email: req.email,
        satker_code: req.satker_code,
        first_name: None,
        last_name: None,
        nip: req.nip,
        nama: req.nama,
        jabatan: req.jabatan,
        phone_number: None,
        phone_verified: None,
        require_password_change: req.require_password_change,
        password: None,
        enabled: req.enabled,
        email_verified: None,
        mfa_enabled: None,
        attributes: None,
    };

    let user = state
        .user_service
        .update_user(UserId::from_uuid(id), update_req)
        .await
        .map_err(crate::error::ApiError)?;

    Ok(Json(user_to_response(&user)))
}

/// DELETE /api/v1/iam/users/{id} - Delete user (soft delete)
pub async fn delete_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .user_service
        .delete_user(UserId::from_uuid(id))
        .await
        .map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{id}/password/reset - Reset user password
pub async fn reset_user_password(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> ApiResult<StatusCode> {
    let update_req = authenc_types::domain::UpdateUserRequest {
        username: None,
        email: None,
        satker_code: None,
        first_name: None,
        last_name: None,
        nip: None,
        nama: None,
        jabatan: None,
        phone_number: None,
        phone_verified: None,
        require_password_change: Some(true),
        password: Some(req.new_password),
        enabled: None,
        email_verified: None,
        mfa_enabled: None,
        attributes: None,
    };

    state
        .user_service
        .update_user(UserId::from_uuid(id), update_req)
        .await
        .map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{id}/mfa/enable - Enable MFA for user
pub async fn enable_user_mfa(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .user_service
        .enable_mfa(UserId::from_uuid(id))
        .await
        .map_err(crate::error::ApiError)?;

    Ok(Json(serde_json::json!({
        "message": "MFA enabled successfully"
    })))
}

/// POST /api/v1/iam/users/{id}/mfa/disable - Disable MFA for user
pub async fn disable_user_mfa(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .user_service
        .disable_mfa(UserId::from_uuid(id))
        .await
        .map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}
