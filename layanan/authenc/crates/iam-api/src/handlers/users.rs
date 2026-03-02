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
use authenc_types::{AuthencError, RealmId, UserId, User};
use authenc_types::domain::user::{CreateUserRequest as CoreCreateUserRequest, UpdateUserRequest as CoreUpdateUserRequest};

fn to_user_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        enabled: user.enabled,
        email_verified: user.email_verified,
        mfa_enabled: user.mfa_enabled,
        realm_id: user.realm_id.unwrap_or(Uuid::nil()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}


/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// Search query (username, email, name)
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
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// User response DTO
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub realm_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub satker_code: String,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub phone_number: Option<String>,
    pub organization_id: Option<Uuid>,
    pub roles: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub realm_id: Uuid,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub roles: Option<Vec<String>>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub satker_code: Option<String>,
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

/// GET /api/v1/iam/users - List users with pagination
pub async fn list_users(
    State(state): State<Arc<IamApiState>>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<PaginatedUsers>> {
    if params.page_size == 0 {
        return Err(crate::error::ApiError(AuthencError::validation("page_size must be greater than 0")));
    }

    let realm_id = params.realm_id.unwrap_or(Uuid::nil());
    let offset = (params.page.saturating_sub(1) * params.page_size) as usize;

    // In a real implementation we would do searching and getting a total count.
    // Here we just wrap the existing `list_users`
    let users = state.user_service.list_users(RealmId::from_uuid(realm_id), offset, params.page_size as usize).await.map_err(crate::error::ApiError)?;

    let total_returned = users.len() as u64;
    let user_responses: Vec<UserResponse> = users.into_iter().map(to_user_response).collect();

    // In a full implementation, we'd query the actual total from the database.
    // Since UserManagementServiceImpl doesn't expose a count method yet,
    // we provide a heuristic based on what we fetched.
    let total = if total_returned < params.page_size as u64 && params.page == 1 {
        total_returned
    } else {
        total_returned + offset as u64 // Minimum possible total
    };

    let total_pages = (total as f64 / params.page_size as f64).ceil() as u32;

    Ok(Json(PaginatedUsers {
        users: user_responses,
        total,
        page: params.page,
        page_size: params.page_size,
        total_pages: std::cmp::max(1, total_pages),
    }))
}

/// POST /api/v1/iam/users - Create user
pub async fn create_user(
    State(state): State<Arc<IamApiState>>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<(StatusCode, Json<UserResponse>)> {
    let mut attrs_map = serde_json::Map::new();
    if let Some(enabled) = req.enabled {
        attrs_map.insert("enabled".to_string(), serde_json::Value::Bool(enabled));
    }

    let mut roles_uuids = None;
    if let Some(rs) = req.roles {
        let mut parsed = Vec::new();
        for r in rs {
            match Uuid::parse_str(&r) {
                Ok(uuid) => parsed.push(uuid),
                Err(_) => return Err(crate::error::ApiError(AuthencError::validation(format!("Invalid UUID for role: {}", r)))),
            }
        }
        roles_uuids = Some(parsed);
    }

    let core_req = CoreCreateUserRequest {
        username: req.username,
        email: req.email,
        satker_code: req.satker_code,
        password: req.password,
        first_name: req.first_name,
        last_name: req.last_name,
        nip: req.nip,
        nama: req.nama,
        jabatan: req.jabatan,
        phone_number: req.phone_number,
        organization_id: req.organization_id,
        roles: roles_uuids,
        realm_id: Some(req.realm_id),
        attributes: Some(serde_json::Value::Object(attrs_map)),
    };

    let user = state.user_service.create_user(core_req).await.map_err(crate::error::ApiError)?;

    // We would ideally set the password too if user_service supported it directly or via another call

    Ok((StatusCode::CREATED, Json(to_user_response(user))))
}

/// GET /api/v1/iam/users/{id} - Get user details
pub async fn get_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    let user = state.user_service.get_user(UserId::from_uuid(id)).await.map_err(crate::error::ApiError)?;
    Ok(Json(to_user_response(user)))
}

/// PUT /api/v1/iam/users/{id} - Update user
pub async fn update_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    let core_req = CoreUpdateUserRequest {
        username: None,
        email: req.email,
        first_name: req.first_name,
        last_name: req.last_name,
        phone_number: req.phone_number,
        nip: req.nip,
        nama: req.nama,
        jabatan: req.jabatan,
        satker_code: req.satker_code,
        attributes: None,
        enabled: req.enabled,
        email_verified: None,
        phone_verified: None,
        require_password_change: None,
        password: None,
        mfa_enabled: None,
        totp_secret: None,
    };

    let user = state.user_service.update_user(UserId::from_uuid(id), core_req).await.map_err(crate::error::ApiError)?;
    Ok(Json(to_user_response(user)))
}

/// DELETE /api/v1/iam/users/{id} - Delete user
pub async fn delete_user(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.user_service.delete_user(UserId::from_uuid(id)).await.map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{id}/password/reset - Reset user password
pub async fn reset_user_password(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> ApiResult<StatusCode> {
    // Using UpdateUserRequest as a proxy to reset the password through user_service
    let core_req = CoreUpdateUserRequest {
        username: None,
        email: None,
        first_name: None,
        last_name: None,
        phone_number: None,
        nip: None,
        nama: None,
        jabatan: None,
        satker_code: None,
        attributes: None,
        enabled: None,
        email_verified: None,
        phone_verified: None,
        require_password_change: None,
        password: Some(req.new_password),
        mfa_enabled: None,
        totp_secret: None,
    };

    state.user_service.update_user(UserId::from_uuid(id), core_req).await.map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{id}/mfa/enable - Enable MFA for user
pub async fn enable_user_mfa(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EnableMfaResponse>> {
    // Generate a random TOTP secret (at least 20 bytes, base32 encoded) using Uuid since rand is not linked
    let mut secret_bytes = [0u8; 20];
    let uuid_bytes = Uuid::new_v4().into_bytes();
    secret_bytes[0..16].copy_from_slice(&uuid_bytes);
    secret_bytes[16..20].copy_from_slice(&[1, 2, 3, 4]);

    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut encoded_secret = String::with_capacity((secret_bytes.len() * 8 + 4) / 5);
    let mut buffer = 0u32;
    let mut bits_left = 0;

    for &byte in &secret_bytes {
        buffer = (buffer << 8) | (byte as u32);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = (buffer >> bits_left) & 0x1F;
            encoded_secret.push(ALPHABET[index as usize] as char);
        }
    }
    if bits_left > 0 {
        let index = (buffer << (5 - bits_left)) & 0x1F;
        encoded_secret.push(ALPHABET[index as usize] as char);
    }

    let user = state.user_service.enable_mfa(UserId::from_uuid(id), encoded_secret.clone()).await.map_err(crate::error::ApiError)?;

    let qr_code = format!("otpauth://totp/SIMPEL:{}?secret={}&issuer=SIMPEL", user.username, encoded_secret);

    Ok(Json(EnableMfaResponse {
        secret: encoded_secret,
        qr_code,
    }))
}

/// POST /api/v1/iam/users/{id}/mfa/disable - Disable MFA for user
pub async fn disable_user_mfa(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.user_service.disable_mfa(UserId::from_uuid(id)).await.map_err(crate::error::ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}
