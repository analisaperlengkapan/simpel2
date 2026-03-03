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
use authenc_types::domain::user::{
    CreateUserRequest as CoreCreateUserRequest, UpdateUserRequest as CoreUpdateUserRequest,
};
use authenc_types::{AuthencError, RealmId, User, UserId};

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
        return Err(crate::error::ApiError(AuthencError::validation(
            "page_size must be greater than 0",
        )));
    }

    let realm_id = params.realm_id.unwrap_or(Uuid::nil());
    let offset = (params.page.saturating_sub(1) * params.page_size) as usize;

    // TODO: Replace with proper database-level pagination once implemented in Core Service.
    // Currently fetching up to 1000 items into memory for client-side filtering if `enabled` is filtered.
    // This is technical debt that limits correct pagination for realms with >1000 users.
    // A clean approach without adding new trait methods is fetching larger bounds.
    let needs_memory_pagination = params.enabled.is_some() || params.search.is_some();

    let fetch_limit = if needs_memory_pagination {
        1000
    } else {
        params.page_size as usize
    };
    let fetch_offset = if needs_memory_pagination { 0 } else { offset };

    let mut users = if let Some(ref search_term) = params.search {
        state
            .user_service
            .search_users(RealmId::from_uuid(realm_id), search_term, fetch_limit)
            .await
            .map_err(crate::error::ApiError)?
    } else {
        state
            .user_service
            .list_users(RealmId::from_uuid(realm_id), fetch_offset, fetch_limit)
            .await
            .map_err(crate::error::ApiError)?
    };

    if let Some(enabled_filter) = params.enabled {
        users.retain(|u| u.enabled == enabled_filter);
    }

    if needs_memory_pagination {
        // Manual pagination for post-filtered results
        users = users
            .into_iter()
            .skip(offset)
            .take(params.page_size as usize)
            .collect();
    }

    let total_returned = users.len() as u64;
    let user_responses: Vec<UserResponse> = users.into_iter().map(to_user_response).collect();

    let total = if total_returned < params.page_size as u64 && params.page == 1 {
        total_returned
    } else {
        total_returned + offset as u64
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
                Err(_) => {
                    return Err(crate::error::ApiError(AuthencError::validation(format!(
                        "Invalid UUID for role: {}",
                        r
                    ))));
                }
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

    let mut user = state
        .user_service
        .create_user(core_req)
        .await
        .map_err(crate::error::ApiError)?;

    // Core CreateUserRequest doesn't support setting 'enabled' directly, it uses default=true.
    // If the request explicitly asked to create a disabled user, update them.
    if let Some(false) = req.enabled {
        let disable_req = CoreUpdateUserRequest {
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
            enabled: Some(false),
            email_verified: None,
            phone_verified: None,
            require_password_change: None,
            password: None,
            mfa_enabled: None,
            clear_totp_secret: None,
            totp_secret: None,
        };
        user = state
            .user_service
            .update_user(UserId::from_uuid(user.id), disable_req)
            .await
            .map_err(crate::error::ApiError)?;
    }

    Ok((StatusCode::CREATED, Json(to_user_response(user))))
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
        clear_totp_secret: None,
    };

    let user = state
        .user_service
        .update_user(UserId::from_uuid(id), core_req)
        .await
        .map_err(crate::error::ApiError)?;
    Ok(Json(to_user_response(user)))
}

/// DELETE /api/v1/iam/users/{id} - Delete user
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
        clear_totp_secret: None,
    };

    state
        .user_service
        .update_user(UserId::from_uuid(id), core_req)
        .await
        .map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/iam/users/{id}/mfa/enable - Enable MFA for user
pub async fn enable_user_mfa(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EnableMfaResponse>> {
    let user_id = UserId::from_uuid(id);

    let user = state
        .user_service
        .get_user(user_id)
        .await
        .map_err(crate::error::ApiError)?;

    if let Some(mfa_service) = &state.mfa_service {
        // Use the proper MFA service facade to setup TOTP
        let mfa_setup = mfa_service
            .setup_totp(user_id, &user.username)
            .await
            .map_err(crate::error::ApiError)?;

        // Ensure user is marked as having MFA enabled in the core service
        state
            .user_service
            .enable_mfa(user_id, mfa_setup.secret.clone())
            .await
            .map_err(crate::error::ApiError)?;

        return Ok(Json(EnableMfaResponse {
            secret: mfa_setup.secret,
            qr_code: mfa_setup.qr_code,
        }));
    }

    // Fallback if no MFA service is configured (e.g. tests)
    let mut secret_bytes = [0u8; 20];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret_bytes);
    let encoded_secret = data_encoding::BASE32_NOPAD.encode(&secret_bytes);

    state
        .user_service
        .enable_mfa(user_id, encoded_secret.clone())
        .await
        .map_err(crate::error::ApiError)?;

    // URL-encode the username for the fallback QR URI
    let encoded_username = urlencoding::encode(&user.username);
    let qr_string = format!(
        "otpauth://totp/SIMPEL:{}?secret={}&issuer=SIMPEL",
        encoded_username, encoded_secret
    );

    // Using an embedded simulated Base64 SVG payload for the QR URI for demonstration purposes
    let simulated_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="200" height="200" fill="white"/><text x="10" y="100" font-family="Arial" font-size="12" fill="black">QR: {}</text></svg>"#,
        qr_string
    );
    let simulated_svg_b64 = data_encoding::BASE64.encode(simulated_svg.as_bytes());
    let qr_code_data_uri = format!("data:image/svg+xml;base64,{}", simulated_svg_b64);

    Ok(Json(EnableMfaResponse {
        secret: encoded_secret,
        qr_code: qr_code_data_uri,
    }))
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
