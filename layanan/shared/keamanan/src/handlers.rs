use axum::extract::FromRef;
use axum::{
    extract::{ConnectInfo, Path, Query, State, TypedHeader},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::email::send_email;
use crate::{
    auth::AuthService,
    error::AppError,
    models::{
        AppState, AuditLogQuery, CreateRoleRequest, LoginRequest, LoginResponse, MfaSetupRequest,
        MfaSetupResponse, MfaVerifyRequest, UserInfo,
    },
    vault::VaultClient,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub nip: Option<String>,
    pub nik: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub email: Option<String>,
    pub nip: Option<String>,
    pub nik: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaRecoveryRequest {
    pub email: String,
    pub recovery_code: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaDisableRequest {
    pub password: String,
    pub mfa_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RotateSecretRequest {
    pub path: String,
}

fn validate_password_policy(password: &str, username: &str, email: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password minimal 8 karakter".to_string());
    }
    if password.eq_ignore_ascii_case(username) || password.eq_ignore_ascii_case(email) {
        return Err("Password tidak boleh sama dengan username/email".to_string());
    }
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    if !(has_upper && has_lower && has_digit && has_symbol) {
        return Err("Password harus kombinasi huruf besar, kecil, angka, dan simbol".to_string());
    }
    Ok(())
}

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn login(
    State(state): State<AppState>,
    Json(login_req): Json<LoginRequest>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<LoginResponse>, AppError> {
    let auth_service = AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut details = json!({"username": login_req.username});
    // Cek lockout
    let client = state.pool.get().await?;
    let lockout_rows = client
        .query(
            "SELECT attempt_count, last_attempt FROM keamanan.failed_logins WHERE username = $1 AND ip_address = $2",
            &[&login_req.username, &ip_address]
        )
        .await?;

    if let Some(row) = lockout_rows.first() {
        let attempt_count: i32 = row.get("attempt_count");
        let last_attempt: chrono::DateTime<chrono::Utc> = row.get("last_attempt");
        if attempt_count >= 5 && last_attempt > chrono::Utc::now() - chrono::Duration::minutes(15) {
            state
                .insert_audit_log(
                    None,
                    "login_lockout",
                    "auth",
                    None,
                    &json!({"status": "lockout", "username": login_req.username, "ip": ip_address}),
                    &ip_address,
                    user_agent,
                )
                .await
                .ok();
            return Err(AppError::Validation(
                "Account locked due to too many failed login attempts. Please try again later."
                    .to_string(),
            ));
        }
    }

    // Get user from database
    let user_rows = client
        .query(
            "SELECT id, username, password_hash, email, is_active, mfa_enabled, mfa_secret, created_at, updated_at
             FROM keamanan.users WHERE username = $1 AND is_active = true",
            &[&login_req.username]
        )
        .await?;

    let user = if let Some(row) = user_rows.first() {
        Some(crate::models::User::from(row))
    } else {
        None
    };

    if user.is_none() {
        // Update/insert failed login
        let _ = client
            .execute(
                "INSERT INTO keamanan.failed_logins (username, ip_address, user_agent, attempt_count, first_attempt, last_attempt) VALUES ($1, $2, $3, 1, NOW(), NOW()) \
                ON CONFLICT (username, ip_address) DO UPDATE SET attempt_count = failed_logins.attempt_count + 1, last_attempt = NOW()",
                &[&login_req.username, &ip_address, &user_agent]
            )
            .await?;
        return Err(AppError::InvalidCredentials);
    }
    let user = match user {
        Some(u) => u,
        None => return Err(AppError::NotFound),
    };
    // Verify password
    if !auth_service.verify_password(&login_req.password, &user.password_hash)? {
        // Update/insert failed login
        let _ = client
            .execute(
                "INSERT INTO keamanan.failed_logins (username, ip_address, user_agent, attempt_count, first_attempt, last_attempt) VALUES ($1, $2, $3, 1, NOW(), NOW()) \
                ON CONFLICT (username, ip_address) DO UPDATE SET attempt_count = failed_logins.attempt_count + 1, last_attempt = NOW()",
                &[&login_req.username, &ip_address, &user_agent]
            )
            .await?;
        state
            .insert_audit_log(
                Some(user.id),
                "login",
                "auth",
                None,
                &json!({"status": "failed", "error": "Invalid password"}),
                &ip_address,
                user_agent,
            )
            .await
            .ok();
        return Err(AppError::InvalidCredentials);
    }
    // Reset failed login counter jika sukses
    let _ = client
        .execute(
            "DELETE FROM keamanan.failed_logins WHERE username = $1 AND ip_address = $2",
            &[&login_req.username, &ip_address],
        )
        .await?;
    // Verify MFA if enabled
    if user.mfa_enabled {
        if let Some(mfa_code) = &login_req.mfa_code {
            if !auth_service.verify_mfa(&user.mfa_secret.clone().unwrap_or_default(), mfa_code)? {
                state
                    .insert_audit_log(
                        Some(user.id),
                        "login",
                        "auth",
                        None,
                        &json!({"status": "failed", "error": "Invalid MFA code"}),
                        &ip_address,
                        user_agent,
                    )
                    .await
                    .ok();
                return Err(AppError::InvalidMfaCode);
            }
        } else {
            state
                .insert_audit_log(
                    Some(user.id),
                    "login",
                    "auth",
                    None,
                    &json!({"status": "failed", "error": "MFA required"}),
                    &ip_address,
                    user_agent,
                )
                .await
                .ok();
            return Err(AppError::MfaRequired);
        }
    }
    // Get user roles and permissions
    let roles = auth_service.get_user_roles(user.id).await?;
    let permissions = auth_service.get_user_permissions(user.id).await?;
    // Generate tokens
    let access_token = auth_service.generate_access_token(&user, &roles, &permissions)?;
    let refresh_token = auth_service.generate_refresh_token(&user)?;
    let user_info = crate::models::UserInfo {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        roles,
        permissions,
    };
    let response = crate::models::LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: user_info,
    };
    state
        .insert_audit_log(
            Some(user.id),
            "login",
            "auth",
            None,
            &json!({"status": "success"}),
            &ip_address,
            user_agent,
        )
        .await
        .ok();
    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Result<StatusCode, AppError> {
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Extract user_id dari JWT jika ada
    let user_id = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
        .and_then(|token| {
            let auth_service =
                crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
            auth_service.verify_token(token).ok()
        })
        .map(|claims| claims.sub);
    state
        .insert_audit_log(
            user_id,
            "logout",
            "auth",
            None,
            &json!({"status": "success"}),
            &ip_address,
            user_agent,
        )
        .await
        .ok();
    info!("User logged out");
    Ok(StatusCode::OK)
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(token_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let auth_service = AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
    let refresh_token = token_data["refresh_token"]
        .as_str()
        .ok_or(AppError::Validation("Missing refresh token".to_string()))?;
    let claims = auth_service.verify_token(refresh_token)?;
    let user_id = claims.sub;
    let refresh_token_hash = auth_service.hash_token(refresh_token);
    // Cek apakah token sudah pernah dipakai (revoked)
    if state
        .is_refresh_token_revoked(&refresh_token_hash)
        .await
        .unwrap_or(true)
    {
        // Reuse detected, revoke semua session user
        let _ = state.revoke_all_sessions_for_user(user_id).await;
        return Err(AppError::Validation(
            "Refresh token reuse detected. All sessions revoked.".to_string(),
        ));
    }
    // Revoke token lama
    let _ = state.revoke_refresh_token(&refresh_token_hash).await;
    // Generate token baru
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_one("SELECT id, username, email, password_hash, nip, nik, is_active, created_at, updated_at FROM keamanan.users WHERE id = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user = crate::models::User::from(&row);
    let roles = auth_service.get_user_roles(user.id).await?;
    let permissions = auth_service.get_user_permissions(user.id).await?;
    let new_access_token = auth_service.generate_access_token(&user, &roles, &permissions)?;
    let new_refresh_token = auth_service.generate_refresh_token(&user, &roles, &permissions)?;
    let new_refresh_token_hash = auth_service.hash_token(&new_refresh_token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    let _ = state
        .save_new_refresh_token(user.id, &new_refresh_token_hash, expires_at)
        .await;
    Ok(Json(json!({
        "access_token": new_access_token,
        "refresh_token": new_refresh_token,
        "token_type": "Bearer",
        "expires_in": 3600
    })))
}

pub async fn setup_mfa(
    State(state): State<AppState>,
    Json(mfa_req): Json<MfaSetupRequest>,
) -> Result<Json<MfaSetupResponse>, AppError> {
    use base32::Alphabet;
    use totp_lite::{totp_custom, Algorithm, Sha1};

    // Generate MFA secret
    let secret = base32::encode(
        Alphabet::RFC4648 { padding: true },
        &rand::random::<[u8; 20]>(),
    );

    // Generate QR code
    let otpauth_url = format!(
        "otpauth://totp/SIMPelv2:{}?secret={}&issuer=SIMPelv2",
        mfa_req.user_id, secret
    );

    let qr_code = qrcode::QrCode::new(&otpauth_url)?;
    let qr_code_svg = qr_code.to_svg_string(
        qrcode::render::svg::Color::Black,
        qrcode::render::svg::Color::White,
    )?;

    // Store secret in database
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    client
        .execute("UPDATE keamanan.users SET mfa_secret = $1, mfa_enabled = true WHERE id = $2", &[&secret, &mfa_req.user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("MFA setup completed for user: {}", mfa_req.user_id);

    Ok(Json(MfaSetupResponse {
        qr_code: qr_code_svg,
        secret,
    }))
}

pub async fn verify_mfa(
    State(state): State<AppState>,
    Json(mfa_req): Json<MfaVerifyRequest>,
) -> Result<StatusCode, AppError> {
    use totp_lite::{totp_custom, Algorithm, Sha1};

    // Get user's MFA secret
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_one("SELECT mfa_secret FROM keamanan.users WHERE id = $1", &[&mfa_req.user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let mfa_secret: Option<String> = row.get(0);

    let secret = mfa_secret
        .ok_or(AppError::Validation("MFA not set up".to_string()))?;

    // Verify MFA code
    let totp = totp_custom!(secret.as_bytes(), 30, 6, Sha1::sha1());

    if totp != mfa_req.code {
        return Err(AppError::InvalidMfaCode);
    }

    info!("MFA verified for user: {}", mfa_req.user_id);
    Ok(StatusCode::OK)
}

pub async fn update_profile(
    State(state): State<AppState>,
    Json(req): Json<UpdateProfileRequest>,
    headers: HeaderMap,
) -> Result<Json<UserInfo>, AppError> {
    // Extract user_id dari JWT (asumsi sudah ada middleware auth)
    let claims = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
        .and_then(|token| {
            let auth_service =
                crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
            auth_service.verify_token(token).ok()
        })
        .ok_or(AppError::Unauthorized)?;
    let user_id = claims.sub;
    // Update user di DB
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_one("UPDATE keamanan.users SET email = COALESCE($1, email), nip = COALESCE($2, nip), nik = COALESCE($3, nik), updated_at = NOW() WHERE id = $4 RETURNING id, username, email, password_hash, nip, nik, is_active, created_at, updated_at", &[&req.email, &req.nip, &req.nik, &user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user = crate::models::User::from(&row);
    Ok(Json(UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        nip: user.nip,
        nik: user.nik,
        roles: vec![],
        permissions: vec![],
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    // In a real implementation, you'd extract the JWT token from headers
) -> Result<Json<UserInfo>, AppError> {
    // This is a placeholder - in real implementation, extract user from JWT
    let user_info = UserInfo {
        id: Uuid::new_v4(),
        username: "admin".to_string(),
        email: "admin@simpelv2.go.id".to_string(),
        roles: vec!["admin".to_string()],
        permissions: vec!["*".to_string()],
    };

    Ok(Json(user_info))
}

pub async fn get_roles(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::Role>>, AppError> {
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let rows = client
        .query("SELECT id, name, description, created_at, updated_at FROM keamanan.roles ORDER BY name", &[])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let roles: Vec<crate::models::Role> = rows.iter().map(|row| crate::models::Role::from(row)).collect();

    Ok(Json(roles))
}

pub async fn create_role(
    State(state): State<AppState>,
    Json(role_req): Json<CreateRoleRequest>,
) -> Result<Json<crate::models::Role>, AppError> {
    let role_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_one("INSERT INTO keamanan.roles (id, name, description, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, name, description, created_at, updated_at", &[&role_id, &role_req.name, &role_req.description, &role_req.permissions, &now, &now])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let role = crate::models::Role::from(&row);

    info!("Role created: {}", role.name);
    Ok(Json(role))
}

pub async fn get_permissions(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::Permission>>, AppError> {
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let rows = client
        .query("SELECT id, name, description, created_at, updated_at FROM keamanan.permissions ORDER BY name", &[])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let permissions: Vec<crate::models::Permission> = rows.iter().map(|row| crate::models::Permission::from(row)).collect();

    Ok(Json(permissions))
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<crate::models::AuditLog>>, AppError> {
    let mut sql = "SELECT * FROM keamanan.audit_logs WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];
    let mut param_count = 1;

    if let Some(user_id) = query.user_id {
        sql.push_str(&format!(" AND user_id = ${}", param_count));
        params.push(user_id.to_string());
        param_count += 1;
    }

    if let Some(action) = query.action {
        sql.push_str(&format!(" AND action = ${}", param_count));
        params.push(action);
        param_count += 1;
    }

    if let Some(resource) = query.resource {
        sql.push_str(&format!(" AND resource = ${}", param_count));
        params.push(resource);
        param_count += 1;
    }

    if let Some(start_date) = query.start_date {
        sql.push_str(&format!(" AND timestamp >= ${}", param_count));
        params.push(start_date.to_rfc3339());
        param_count += 1;
    }

    if let Some(end_date) = query.end_date {
        sql.push_str(&format!(" AND timestamp <= ${}", param_count));
        params.push(end_date.to_rfc3339());
        param_count += 1;
    }

    sql.push_str(" ORDER BY timestamp DESC");

    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    // Note: This is a simplified implementation
    // In a real implementation, you'd use proper parameterized queries
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let rows = client
        .query(&sql, &[])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let logs: Vec<crate::models::AuditLog> = rows.iter().map(|row| crate::models::AuditLog::from(row)).collect();

    Ok(Json(logs))
}

pub async fn mfa_recovery(
    State(state): State<AppState>,
    Json(req): Json<MfaRecoveryRequest>,
) -> Result<StatusCode, AppError> {
    // Implementasi recovery code/token secure
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_opt("SELECT id, username, email, password_hash, nip, nik, is_active, created_at, updated_at FROM keamanan.users WHERE email = $1", &[&req.email])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::NotFound)?;
    let user = crate::models::User::from(&row);
    // Cari recovery code di DB
    let rec_row = client
        .query_opt("SELECT code, expires_at, used FROM keamanan.mfa_recovery WHERE user_id = $1 ORDER BY expires_at DESC LIMIT 1", &[&user.id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::Validation("No recovery code found".to_string()))?;
    
    let code: String = rec_row.get(0);
    let expires_at: chrono::DateTime<chrono::Utc> = rec_row.get(1);
    let used: bool = rec_row.get(2);
    if used || expires_at < chrono::Utc::now() {
        return Err(AppError::Validation(
            "Recovery code expired or already used".to_string(),
        ));
    }
    if req.recovery_code != code {
        return Err(AppError::Validation("Invalid recovery code".to_string()));
    }
    // Disable MFA
    client
        .execute("UPDATE keamanan.users SET mfa_enabled = false, mfa_secret = NULL WHERE id = $1", &[&user.id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    // Tandai recovery code sudah dipakai
    client
        .execute("UPDATE keamanan.mfa_recovery SET used = TRUE WHERE user_id = $1 AND code = $2", &[&user.id, &code])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    state
        .insert_audit_log(
            Some(user.id),
            "mfa_recovery",
            "auth",
            None,
            &json!({"status": "success"}),
            "-",
            "-",
        )
        .await
        .ok();
    Ok(StatusCode::OK)
}

pub async fn mfa_disable(
    State(state): State<AppState>,
    Json(req): Json<MfaDisableRequest>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    // Extract user_id dari JWT
    let claims = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
        .and_then(|token| {
            let auth_service =
                crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
            auth_service.verify_token(token).ok()
        })
        .ok_or(AppError::Unauthorized)?;
    let user_id = claims.sub;
    // Get user
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_one("SELECT id, username, email, password_hash, nip, nik, is_active, created_at, updated_at FROM keamanan.users WHERE id = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user = crate::models::User::from(&row);
    // Verifikasi password
    let auth_service =
        crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
    if !auth_service.verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }
    // Jika MFA masih aktif, verifikasi MFA code jika ada
    if user.mfa_enabled {
        if let Some(mfa_code) = &req.mfa_code {
            if !auth_service.verify_mfa(&user.mfa_secret.clone().unwrap_or_default(), mfa_code)? {
                return Err(AppError::InvalidMfaCode);
            }
        }
    }
    // Disable MFA
    client
        .execute("UPDATE keamanan.users SET mfa_enabled = false, mfa_secret = NULL WHERE id = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    state
        .insert_audit_log(
            Some(user_id),
            "mfa_disable",
            "auth",
            None,
            &json!({"status": "success"}),
            "-",
            "-",
        )
        .await
        .ok();
    Ok(StatusCode::OK)
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<StatusCode, AppError> {
    // Cari user
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let row = client
        .query_opt("SELECT id, username, email, password_hash, nip, nik, is_active, created_at, updated_at FROM keamanan.users WHERE email = $1", &[&req.email])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::NotFound)?;
    let user = crate::models::User::from(&row);
    // Generate token
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
    // Simpan token ke DB
    client
        .execute("INSERT INTO keamanan.password_resets (user_id, token, expires_at) VALUES ($1, $2, $3)", &[&user.id, &token, &expires_at])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    // Kirim email notification
    let subject = "Password Reset Request";
    let body = format!(
        "Gunakan token berikut untuk reset password Anda: {} (berlaku sampai {})",
        token, expires_at
    );
    let _ = send_email(&user.email, subject, &body).await;
    state
        .insert_audit_log(
            Some(user.id),
            "request_password_reset",
            "auth",
            None,
            &json!({"status": "success"}),
            "-",
            "-",
        )
        .await
        .ok();
    Ok(StatusCode::OK)
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetConfirm>,
) -> Result<StatusCode, AppError> {
    // Cari token
    let client = state.pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;
    let rec_row = client
        .query_opt("SELECT user_id, expires_at, used FROM keamanan.password_resets WHERE token = $1", &[&req.token])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::Validation("Invalid or expired token".to_string()))?;
    
    let user_id: uuid::Uuid = rec_row.get(0);
    let expires_at: chrono::DateTime<chrono::Utc> = rec_row.get(1);
    let used: bool = rec_row.get(2);
    
    if used || expires_at < chrono::Utc::now() {
        return Err(AppError::Validation(
            "Token expired or already used".to_string(),
        ));
    }
    // Update password user
    // Tandai token sudah used
    client
        .execute("UPDATE keamanan.password_resets SET used = TRUE WHERE token = $1", &[&req.token])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let hash = crate::auth::hash_password(&req.new_password)?;
    client
        .execute("UPDATE keamanan.users SET password_hash = $1 WHERE id = $2", &[&hash, &user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    state
        .insert_audit_log(
            Some(user_id),
            "reset_password",
            "auth",
            None,
            &json!({"status": "success"}),
            "-",
            "-",
        )
        .await
        .ok();
    Ok(StatusCode::OK)
}

pub async fn rotate_vault_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RotateSecretRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Cek role admin dari JWT
    let claims = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
        .and_then(|token| {
            let auth_service =
                crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone());
            auth_service.verify_token(token).ok()
        })
        .ok_or(AppError::Unauthorized)?;
    let roles = crate::auth::AuthService::new(state.pool.clone(), state.config.jwt_secret.clone())
        .get_user_roles(claims.sub)
        .await?;
    if !roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden);
    }
    // Trigger secret rotation
    let new_secret = state.vault_client.rotate_secret(&req.path).await?;
    state
        .insert_audit_log(
            Some(claims.sub),
            "vault_rotate_secret",
            "vault",
            Some(&req.path),
            &json!({"status": "success"}),
            "-",
            "-",
        )
        .await
        .ok();
    Ok(Json(json!({"new_secret": new_secret})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt; // for `oneshot` method

    #[tokio::test]
    async fn test_login_fail_wrong_password() {
        // Setup dummy state dan user
        let pool_config = deadpool_postgres::Config {
            user: Some("user".to_string()),
            password: Some("password".to_string()),
            host: Some("localhost".to_string()),
            dbname: Some("testdb".to_string()),
            ..Default::default()
        };
        let pool = pool_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap();
        let config = crate::config::Config::load().unwrap();
        let vault_client = crate::vault::VaultClient::new("http://localhost:8200", "test").unwrap();
        let state = crate::models::AppState {
            pool,
            vault_client,
            config,
        };
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "salah".to_string(),
            mfa_code: None,
        };
        let headers = axum::http::HeaderMap::new();
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 12345));
        let result = login(
            State(state),
            axum::Json(req),
            headers,
            axum::extract::ConnectInfo(addr),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_success() {
        // Setup dummy state dan user (harus ada user admin:admin di DB test)
        let pool_config = deadpool_postgres::Config {
            user: Some("user".to_string()),
            password: Some("password".to_string()),
            host: Some("localhost".to_string()),
            dbname: Some("testdb".to_string()),
            ..Default::default()
        };
        let pool = pool_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap();
        let config = crate::config::Config::load().unwrap();
        let vault_client = crate::vault::VaultClient::new("http://localhost:8200", "test").unwrap();
        let state = crate::models::AppState {
            pool,
            vault_client,
            config,
        };
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
            mfa_code: None,
        };
        let headers = axum::http::HeaderMap::new();
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 12345));
        let result = login(
            State(state),
            axum::Json(req),
            headers,
            axum::extract::ConnectInfo(addr),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mfa_fail() {
        // Setup dummy state dan user dengan MFA aktif
        let pool_config = deadpool_postgres::Config {
            user: Some("user".to_string()),
            password: Some("password".to_string()),
            host: Some("localhost".to_string()),
            dbname: Some("testdb".to_string()),
            ..Default::default()
        };
        let pool = pool_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap();
        let config = crate::config::Config::load().unwrap();
        let vault_client = crate::vault::VaultClient::new("http://localhost:8200", "test").unwrap();
        let state = crate::models::AppState {
            pool,
            vault_client,
            config,
        };
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
            mfa_code: Some("salah".to_string()),
        };
        let headers = axum::http::HeaderMap::new();
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 12345));
        let result = login(
            State(state),
            axum::Json(req),
            headers,
            axum::extract::ConnectInfo(addr),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_fail_token() {
        // Setup dummy state
        let pool_config = deadpool_postgres::Config {
            user: Some("user".to_string()),
            password: Some("password".to_string()),
            host: Some("localhost".to_string()),
            dbname: Some("testdb".to_string()),
            ..Default::default()
        };
        let pool = pool_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap();
        let config = crate::config::Config::load().unwrap();
        let vault_client = crate::vault::VaultClient::new("http://localhost:8200", "test").unwrap();
        let state = crate::models::AppState {
            pool,
            vault_client,
            config,
        };
        let req = PasswordResetConfirm {
            token: "token_salah".to_string(),
            new_password: "PasswordBaru123!".to_string(),
        };
        let result = reset_password(State(state), axum::Json(req)).await;
        assert!(result.is_err());
    }

    // Tambahkan test lain sesuai kebutuhan
}
