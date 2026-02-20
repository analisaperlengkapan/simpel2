//! User management HTTP handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{state::IamApiState, error::ApiResult};

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

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 20 }

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
    pub password: String,
    pub enabled: Option<bool>,
    pub realm_id: Uuid,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub enabled: Option<bool>,
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
    State(_state): State<Arc<IamApiState>>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<PaginatedUsers>> {
    // TODO: Implement pagination and filtering in UserManagementService
    // For now, return empty list
    Ok(Json(PaginatedUsers {
        users: vec![],
        total: 0,
        page: params.page,
        page_size: params.page_size,
        total_pages: 0,
    }))
}

/// POST /api/v1/iam/users - Create user
pub async fn create_user(
    State(_state): State<Arc<IamApiState>>,
    Json(_req): Json<CreateUserRequest>,
) -> ApiResult<(StatusCode, Json<UserResponse>)> {
    // TODO: Call user_service.create_user()
    Err(crate::error::ApiError(AuthencError::NotImplemented("create_user not yet implemented".to_string()))))
}

/// GET /api/v1/iam/users/{id} - Get user details
pub async fn get_user(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    // TODO: Call user_service.get_user()
    Err(crate::error::ApiError(AuthencError::NotImplemented("get_user not yet implemented".to_string()))))
}

/// PUT /api/v1/iam/users/{id} - Update user
pub async fn update_user(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    // TODO: Call user_service.update_user()
    Err(crate::error::ApiError(AuthencError::NotImplemented("update_user not yet implemented".to_string()))))
}

/// DELETE /api/v1/iam/users/{id} - Delete user
pub async fn delete_user(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Call user_service.delete_user()
    Err(crate::error::ApiError(AuthencError::NotImplemented("delete_user not yet implemented".to_string()))))
}

/// POST /api/v1/iam/users/{id}/password/reset - Reset user password
pub async fn reset_user_password(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<ResetPasswordRequest>,
) -> ApiResult<StatusCode> {
    // TODO: Call user_service.reset_password()
    Err(crate::error::ApiError(AuthencError::NotImplemented("reset_password not yet implemented".to_string()))))
}

/// POST /api/v1/iam/users/{id}/mfa/enable - Enable MFA for user
pub async fn enable_user_mfa(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<EnableMfaResponse>> {
    // TODO: Call mfa_service.setup_totp()
    Err(crate::error::ApiError(AuthencError::NotImplemented("enable_mfa not yet implemented".to_string()))))
}
