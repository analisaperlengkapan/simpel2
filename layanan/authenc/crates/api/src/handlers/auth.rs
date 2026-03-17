//! Authentication endpoint handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::auth_helpers;
use crate::state::ApiState;

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
    /// Optional realm ID (defaults to master realm)
    pub realm_id: Option<Uuid>,
}

/// Login response payload
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// Temporary token for MFA flow (alias: mfa_token for portal compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_token: Option<String>,
    /// Temporary token alias used by portal frontend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_token: Option<String>,
    /// Whether MFA verification is required
    pub mfa_required: bool,
    /// Whether MFA setup is required (first-time)
    pub mfa_setup_required: bool,
    /// Whether password change is required before using the system
    pub require_password_change: bool,
    /// Status message
    pub message: String,
}

/// Logout request payload
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Refresh token to invalidate
    pub refresh_token: String,
}

/// Refresh token request payload
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Refresh token response payload
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New JWT access token
    pub access_token: String,
    /// New refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Display name (first + last)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// First name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Last name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// NIP (Nomor Induk Pegawai)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip: Option<String>,
    /// Nama lengkap pegawai
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nama: Option<String>,
    /// Jabatan pegawai
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jabatan: Option<String>,
    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Avatar URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Division / organizational unit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    /// Primary role
    pub role: String,
    /// Permissions list
    pub permissions: Vec<String>,
    /// Whether email is verified
    pub email_verified: bool,
    /// Whether MFA is enabled
    pub mfa_enabled: bool,
    /// Whether password change is required
    pub require_password_change: bool,
    /// Realm ID
    pub realm_id: Uuid,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Error message
    pub message: String,
    /// HTTP status code
    #[serde(skip)]
    pub status_code: StatusCode,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

/// POST /api/v1/auth/login - Username/password login
///
/// Authenticates a user with username and password.
/// Returns JWT access token and refresh token on success.
/// If MFA is enabled, returns mfa_token and requires MFA verification.
pub async fn login_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<LoginRequest>,
) -> impl axum::response::IntoResponse {
    use authenc_types::{
        AuthFailureReason, AuthResult, Credentials, RealmId, traits::AuthenticationService,
    };

    // Default to master realm if none provided (master realm = all-zeros UUID)
    let realm_id = RealmId::from_uuid(
        request
            .realm_id
            .unwrap_or_else(|| Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
    );

    let credentials = Credentials {
        username: request.username.clone(),
        password: request.password.clone(),
    };

    match state.auth_service.authenticate(credentials, realm_id).await {
        Ok(AuthResult::Success {
            user_id,
            session_id,
        }) => {
            let uid = user_id.as_uuid().to_string();
            let sid = session_id.0.to_string();

            // Check if user must change password
            let user = state.user_service.get_user(user_id).await.ok();
            let require_password_change = user
                .as_ref()
                .map(|u| u.require_password_change)
                .unwrap_or(false);

            if user.is_none() {
                tracing::warn!("Failed to fetch user details for token claims mapping");
            }

            // Enrich JWT with pegawai data
            let custom_claims = crate::handlers::auth_helpers::build_user_custom_claims(user.as_ref());

            let access_token = match state.jwt_service.generate_access_token_with_claims(
                &uid,
                None,
                Some("openid profile email".to_string()),
                Some(sid.clone()),
                custom_claims,
            ) {
                Ok(t) => t,
                Err(e) => {
                    return ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "token_error".to_string(),
                        message: e.to_string(),
                    }.into_response();
                }
            };

            let refresh_token = match state.jwt_service.generate_refresh_token(&uid, &sid) {
                Ok(t) => t,
                Err(e) => {
                    return ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "token_error".to_string(),
                        message: e.to_string(),
                    }.into_response();
                }
            };

            (
                axum::http::StatusCode::OK,
                axum::Json(LoginResponse {
                    access_token,
                    refresh_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 900,
                    mfa_token: None,
                    temp_token: None,
                    mfa_required: false,
                    mfa_setup_required: false,
                    require_password_change,
                    message: if require_password_change {
                        "Anda harus mengubah password sebelum melanjutkan.".to_string()
                    } else {
                        String::new()
                    },
                }),
            )
                .into_response()
        }

        Ok(AuthResult::MfaRequired {
            user_id: _,
            mfa_token,
        }) => (
            axum::http::StatusCode::OK,
            axum::Json(LoginResponse {
                access_token: String::new(),
                refresh_token: String::new(),
                token_type: "Bearer".to_string(),
                expires_in: 0,
                mfa_token: Some(mfa_token.clone()),
                temp_token: Some(mfa_token),
                mfa_required: true,
                mfa_setup_required: false,
                require_password_change: false,
                message: String::new(),
            }),
        )
            .into_response(),

        Ok(AuthResult::Failed { reason }) => {
            // Log actual reason server-side only — never expose to client
            tracing::warn!(
                username = %request.username,
                reason = %reason,
                "Authentication failed"
            );

            // Auto-provision: if username looks like NIP and the user does NOT
            // already exist locally, try fetching from MySIMKARI and create a
            // disabled account.  We always return the same generic error to
            // prevent NIP enumeration.
            if matches!(reason, AuthFailureReason::InvalidCredentials) {
                let username = &request.username;
                let is_nip = username.len() == 18 && username.chars().all(|c| c.is_ascii_digit());

                if is_nip {
                    // Only attempt provisioning when the user does not exist locally.
                    // This avoids unnecessary gRPC calls for existing users with wrong passwords.
                    let user_exists = state
                        .user_service
                        .get_user_by_username(username, realm_id)
                        .await
                        .is_ok();

                    if !user_exists {
                        if let Some(ref integrasi) = state.integrasi_client {
                            match integrasi.get_pegawai_by_nip(username).await {
                                Ok(Some(pegawai)) => {
                                    // Create a disabled user from pegawai data
                                    let email = if pegawai.email.is_empty() {
                                        format!("{}@kejaksaan.go.id", username)
                                    } else {
                                        pegawai.email.clone()
                                    };
                                    let create_req = authenc_types::CreateUserRequest {
                                        username: username.clone(),
                                        email,
                                        satker_code: pegawai.kode_satker.clone(),
                                        // Password needs uppercase, lowercase, digit, and length > 8
                                        password: Some(format!("Aa1{}", uuid::Uuid::new_v4().to_string())), // Random secure password
                                        first_name: None,
                                        last_name: None,
                                        nip: Some(username.clone()),
                                        nama: Some(pegawai.nama.clone()),
                                        jabatan: Some(pegawai.jabatan.clone()),
                                        phone_number: if pegawai.telepon.is_empty() {
                                            None
                                        } else {
                                            Some(pegawai.telepon.clone())
                                        },
                                        realm_id: Some(realm_id.0),
                                        organization_id: None,
                                        roles: None,
                                        attributes: None,
                                        enabled: Some(false), // Disable initially
                                    };
                                    match state.user_service.create_user(create_req).await {
                                        Ok(user) => {
                                            // Set require_password_change flag
                                            let update_req = authenc_types::UpdateUserRequest {
                                                require_password_change: Some(true),
                                                ..Default::default()
                                            };
                                            match state
                                                .user_service
                                                .update_user(
                                                    authenc_types::UserId::from_uuid(user.id),
                                                    update_req,
                                                )
                                                .await
                                            {
                                                Ok(_) => {
                                                    tracing::info!(
                                                        nip = %username,
                                                        user_id = %user.id,
                                                        "Auto-provisioned disabled account from MySIMKARI"
                                                    );
                                                }
                                                Err(e) => {
                                                    tracing::warn!(
                                                        nip = %username,
                                                        error = %e,
                                                        "Failed to set require_password_change for auto-provisioned user, rolling back creation"
                                                    );
                                                    // Fallback mechanism deletes the user softly.
                                                    // This is safe since the user was originally created as disabled, so
                                                    // they won't be able to login even if the update request failed.
                                                    let _ = state.user_service.delete_user(authenc_types::UserId::from_uuid(user.id)).await;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                nip = %username,
                                                error = %e,
                                                "Failed to auto-provision user from MySIMKARI"
                                            );
                                        }
                                    }
                                }
                                Ok(None) => {
                                    tracing::debug!(nip = %username, "NIP not found in MySIMKARI");
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        nip = %username,
                                        error = %e,
                                        "Failed to query MySIMKARI for auto-provisioning"
                                    );
                                }
                            }
                        }
                    }
                }
            }

            let message = match &reason {
                AuthFailureReason::InvalidCredentials => {
                    "Username atau password salah".to_string()
                }
                AuthFailureReason::UserDisabled => {
                    "Akun Anda telah dinonaktifkan. Silakan hubungi administrator.".to_string()
                }
                AuthFailureReason::AccountLocked => {
                    "Akun dikunci sementara karena terlalu banyak percobaan gagal. Silakan coba lagi nanti.".to_string()
                }
                AuthFailureReason::RealmDisabled => {
                    "Layanan autentikasi tidak tersedia saat ini.".to_string()
                }
                AuthFailureReason::InternalError(_) => {
                    "Terjadi kesalahan sistem. Silakan coba lagi nanti.".to_string()
                }
            };

            ErrorResponse {
                status_code: axum::http::StatusCode::UNAUTHORIZED,
                error: "authentication_failed".to_string(),
                message,
            }.into_response()
        }

        Err(e) => {
            // Log actual error server-side only — never expose to client
            tracing::error!(
                username = %request.username,
                error = %e,
                "Internal error during authentication"
            );

            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: "Terjadi kesalahan sistem. Silakan coba lagi nanti.".to_string(),
            }.into_response()
        }
    }
}

/// POST /api/v1/auth/logout - Session invalidation
///
/// Invalidates the provided refresh token and associated session.
pub async fn logout_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<LogoutRequest>,
) -> impl axum::response::IntoResponse {
    use authenc_types::{SessionId, traits::AuthenticationService};

    // Validate the refresh token to extract session_id
    let claims = match state.jwt_service.verify_token(&request.refresh_token) {
        Ok(c) => c,
        Err(_) => {
            // Token is invalid or expired - still return success (idempotent logout)
            return axum::http::StatusCode::NO_CONTENT.into_response();
        }
    };

    // Extract session_id from the token's sid claim
    if let Some(sid_str) = &claims.sid {
        if let Ok(session_uuid) = uuid::Uuid::parse_str(sid_str) {
            let session_id = SessionId(session_uuid);
            let _ = state.auth_service.logout(session_id).await;
        }
    }

    axum::http::StatusCode::NO_CONTENT.into_response()
}

/// POST /api/v1/auth/refresh - Refresh token exchange
///
/// Exchanges a refresh token for a new access token and refresh token.
pub async fn refresh_token_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> impl axum::response::IntoResponse {
    // Validate the refresh token
    let claims = match state.jwt_service.verify_token(&request.refresh_token) {
        Ok(c) => c,
        Err(_) => {
            return ErrorResponse {
                status_code: axum::http::StatusCode::UNAUTHORIZED,
                error: "invalid_token".to_string(),
                message: "Refresh token is invalid or expired".to_string(),
            }.into_response();
        }
    };

    // Extract user_id and session_id from token claims
    let uid = claims.sub.clone();
    let sid = claims.sid.as_deref().unwrap_or("").to_string();

    // Fetch user from DB to build the same custom claims as the login handler
    let mut custom_claims = std::collections::HashMap::new();
    if let Ok(user_uuid) = uid.parse::<Uuid>() {
        let user_id = authenc_types::UserId::from_uuid(user_uuid);
        let user = state.user_service.get_user(user_id).await.ok();

        if user.is_none() {
            tracing::warn!("Failed to fetch user details for refresh token claims mapping");
        }

        custom_claims = crate::handlers::auth_helpers::build_user_custom_claims(user.as_ref());
    }

    // Generate new access token with custom claims (keep same session_id)
    let access_token = match state.jwt_service.generate_access_token_with_claims(
        &uid,
        None,
        Some("openid profile email".to_string()),
        Some(sid.clone()),
        custom_claims,
    ) {
        Ok(t) => t,
        Err(e) => {
            return ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "token_error".to_string(),
                message: e.to_string(),
            }.into_response();
        }
    };

    // Generate new refresh token
    let refresh_token = match state.jwt_service.generate_refresh_token(&uid, &sid) {
        Ok(t) => t,
        Err(e) => {
            return ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "token_error".to_string(),
                message: e.to_string(),
            }.into_response();
        }
    };

    (
        axum::http::StatusCode::OK,
        axum::Json(RefreshTokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 900,
        }),
    )
        .into_response()
}

/// GET /api/v1/auth/me - Get current user profile
///
/// Returns the profile of the currently authenticated user.
/// Requires valid JWT token in Authorization header.
pub async fn get_current_user_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    use authenc_types::UserId;

    let user_uuid = match auth_helpers::extract_user_from_token(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            return ErrorResponse {
                status_code: axum::http::StatusCode::UNAUTHORIZED,
                error: "unauthorized".to_string(),
                message: e.message,
            }.into_response();
        }
    };

    let user_id = UserId::from_uuid(user_uuid);
    match state.user_service.get_user(user_id).await {
        Ok(user) => {
            let realm_id = user.realm_id.unwrap_or_else(|| {
                Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
            });
            // Build display name from first + last name
            let name = match (&user.first_name, &user.last_name) {
                (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                (Some(f), None) => Some(f.clone()),
                (None, Some(l)) => Some(l.clone()),
                _ => user.nama.clone(),
            };
            let division = match user.satker_code.as_str() {
                "" => None,
                "0100000" => Some("Bagian Umum".to_string()),
                code => Some(format!("Satuan Kerja {}", code)),
            };

            (
                axum::http::StatusCode::OK,
                axum::Json(UserProfileResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    name,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    nip: user.nip,
                    nama: user.nama,
                    jabatan: user.jabatan,
                    phone: user.phone_number,
                    avatar: None,
                    division,
                    role: user.roles.first().map(|r| r.name.clone()).unwrap_or_else(|| "user".to_string()),
                    permissions: Vec::new(),
                    email_verified: user.email_verified,
                    mfa_enabled: user.mfa_enabled,
                    require_password_change: user.require_password_change,
                    realm_id,
                }),
            )
                .into_response()
        }
        Err(e) => ErrorResponse {
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal_error".to_string(),
            message: e.to_string(),
        }.into_response(),
    }
}

// =============================================================================
// Profile & Password Management Handlers
// =============================================================================

/// Request body for profile update
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// Request body for password change
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password for verification
    pub current_password: String,
    /// New password
    pub new_password: String,
}

/// Request body for password reset request
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    /// Email address to send reset link
    pub email: String,
    /// Optional captcha token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_token: Option<String>,
}

/// Request body for password reset confirmation
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmRequest {
    /// Password reset token from email link
    pub token: String,
    /// New password
    pub new_password: String,
}

/// PUT /api/v1/auth/me - Update current user profile
pub async fn update_profile_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, ErrorResponse> {
    let user_uuid = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    let user_id = authenc_types::UserId::from_uuid(user_uuid);

    let update_req = authenc_types::UpdateUserRequest {
        first_name: request.display_name.clone(),
        email: request.email.clone(),
        phone_number: request.phone.clone(),
        ..Default::default()
    };

    let user = state
        .user_service
        .update_user(user_id, update_req)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "update_failed".to_string(),
            message: e.to_string(),
        })?;

    let realm_id = user
        .realm_id
        .unwrap_or_else(|| Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap());
    let name = match (&user.first_name, &user.last_name) {
        (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
        (Some(f), None) => Some(f.clone()),
        (None, Some(l)) => Some(l.clone()),
        _ => user.nama.clone(),
    };
    let division = match user.satker_code.as_str() {
        "" => None,
        "0100000" => Some("Bagian Umum".to_string()),
        code => Some(format!("Satuan Kerja {}", code)),
    };

    Ok(Json(UserProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        name,
        first_name: user.first_name,
        last_name: user.last_name,
        nip: user.nip,
        nama: user.nama,
        jabatan: user.jabatan,
        phone: user.phone_number,
        avatar: None,
        division,
        role: user.roles.first().map(|r| r.name.clone()).unwrap_or_else(|| "user".to_string()),
        permissions: Vec::new(),
        email_verified: user.email_verified,
        mfa_enabled: user.mfa_enabled,
        require_password_change: user.require_password_change,
        realm_id,
    }))
}

/// POST /api/v1/auth/me/password - Change password
pub async fn change_password_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ErrorResponse> {
    let user_uuid = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    let user_id = authenc_types::UserId::from_uuid(user_uuid);
    let user = state.user_service.get_user(user_id).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get user for password change");
        ErrorResponse {
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal_error".to_string(),
            message: "Gagal memproses perubahan password.".to_string(),
        }
    })?;

    // Verify current password
    let stored_hash = match user.password_hash.as_deref() {
        Some(h) if !h.is_empty() => h,
        _ => {
            return Err(ErrorResponse {
                status_code: axum::http::StatusCode::BAD_REQUEST,
                error: "no_password".to_string(),
                message: "Akun ini tidak memiliki password. Silakan gunakan metode autentikasi lain.".to_string(),
            });
        }
    };
    let valid = state
        .auth_service
        .verify_password(&request.current_password, stored_hash)
        .unwrap_or(false);

    if !valid {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_password".to_string(),
            message: "Password saat ini salah.".to_string(),
        });
    }

    // Update password and clear require_password_change flag
    let update_req = authenc_types::UpdateUserRequest {
        password: Some(request.new_password),
        require_password_change: Some(false),
        ..Default::default()
    };
    state
        .user_service
        .update_user(user_id, update_req)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to update password");
            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: "Gagal mengubah password.".to_string(),
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/auth/password/reset - Request password reset
pub async fn password_reset_request_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<PasswordResetRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // NOTE: No auth required — public endpoint
    // TODO: Implement password reset email flow
    Err(ErrorResponse {
        status_code: axum::http::StatusCode::NOT_IMPLEMENTED,
        error: "not_implemented".to_string(),
        message: "Password reset request endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/auth/password/reset/confirm - Confirm password reset
pub async fn password_reset_confirm_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<PasswordResetConfirmRequest>,
) -> Result<StatusCode, ErrorResponse> {
    // NOTE: No auth required — uses reset token
    // TODO: Implement password reset confirmation
    Err(ErrorResponse {
        status_code: axum::http::StatusCode::NOT_IMPLEMENTED,
        error: "not_implemented".to_string(),
        message: "Password reset confirm endpoint not yet implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username":"testuser","password":"testpass"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            access_token: "token123".to_string(),
            refresh_token: "refresh123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 900,
            mfa_token: None,
            temp_token: None,
            mfa_required: false,
            mfa_setup_required: false,
            require_password_change: false,
            message: "Login successful".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("Bearer"));
    }
}
