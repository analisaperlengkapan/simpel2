use crate::error::AuthencError;
use crate::services::authorization::capabilities;
use crate::services::mfa_service::MfaService;
use crate::services::stores::user_store::UserStoreTrait;
use crate::utils::jwt;
use crate::utils::validation::{sanitize_string, sanitize_username};
use axum::{
    Router,
    extract::{ConnectInfo, State},
    response::Json,
    routing::post,
};
use garde::Validate;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

/// Create authentication routes
pub fn create_auth_routes() -> Router<Arc<crate::app::AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/test-login", post(test_login))
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/verify-setup", post(mfa_verify_setup))
        .route("/mfa/verify", post(mfa_verify))
        .route("/mfa/status", post(mfa_status))
        .route("/mfa/disable", post(mfa_disable))
        .route("/mfa/reset", post(mfa_reset))
        .route("/mfa/backup-codes", post(mfa_backup_codes))
}

#[derive(Deserialize, Validate)]
/// Request payload for user login
pub struct LoginRequest {
    /// The username for authentication
    #[garde(length(min = 3, max = 50))]
    #[garde(pattern(r"^[a-zA-Z0-9_-]+$"))]
    pub username: String,

    /// The password for authentication
    #[garde(length(min = 8, max = 128))]
    pub password: String,

    /// The realm the user belongs to (optional, defaults to "master")
    #[garde(skip)]
    #[serde(default = "default_realm")]
    pub realm: String,

    /// Optional CAPTCHA token for verification
    #[garde(skip)]
    pub captcha_token: Option<String>,
}

/// Default realm value
fn default_realm() -> String {
    "master".to_string()
}

impl LoginRequest {
    /// Sanitize the request fields
    pub fn sanitize(&mut self) {
        self.username = sanitize_username(&self.username);
        self.realm = sanitize_string(&self.realm, 100);
    }
}

#[derive(Serialize)]
/// Response payload for login attempts
pub struct LoginResponse {
    /// JWT access token for authenticated requests (only present after full authentication)
    pub access_token: Option<String>,
    /// Temporary token for MFA verification (present when MFA verification needed)
    pub temp_token: Option<String>,
    /// Whether MFA verification is required
    pub mfa_required: bool,
    /// Whether MFA setup is required for first-time users
    pub mfa_setup_required: bool,
    /// Response message
    pub message: String,
}

#[derive(Deserialize, Validate)]
/// Request payload for MFA verification
pub struct MfaVerifyRequest {
    /// Temporary token from login response
    #[garde(length(min = 10, max = 1000))]
    pub temp_token: String,

    /// 6-digit OTP code from authenticator app
    #[garde(length(min = 6, max = 6))]
    #[garde(pattern(r"^\d{6}$"))]
    pub code: String,
}

#[derive(Deserialize, Validate)]
/// Request payload for MFA setup verification
pub struct MfaSetupVerifyRequest {
    /// Temporary token from login response
    #[garde(length(min = 10, max = 1000))]
    pub temp_token: String,

    /// 6-digit OTP code from authenticator app
    #[garde(length(min = 6, max = 6))]
    #[garde(pattern(r"^\d{6}$"))]
    pub code: String,
}

#[derive(Serialize)]
/// Response payload for MFA operations
pub struct MfaResponse {
    /// Success message
    pub message: String,
    /// Access token (for successful verification)
    pub access_token: Option<String>,
}

#[derive(Deserialize)]
/// Request payload for MFA disable (admin only)
pub struct MfaDisableRequest {
    /// User ID to disable MFA for
    pub user_id: String,
    /// Admin authorization token
    pub admin_token: String,
    /// Reason for disabling MFA
    pub reason: Option<String>,
}

#[derive(Deserialize)]
/// Request payload for MFA reset
pub struct MfaResetRequest {
    /// User ID to reset MFA for (admin) or token for self-reset
    pub user_id: Option<String>,
    /// Authorization token (admin token for admin reset, user token for self-reset)
    pub token: String,
    /// Reason for resetting MFA
    pub reason: Option<String>,
}

#[derive(Deserialize)]
/// Request payload for backup codes management
pub struct MfaBackupCodesRequest {
    /// Authorization token
    pub token: String,
    /// Action to perform (generate, list, verify)
    pub action: String,
    /// Backup code to verify (for verify action)
    pub code: Option<String>,
}

#[derive(Serialize)]
/// Response payload for backup codes operations
pub struct MfaBackupCodesResponse {
    /// Success message
    pub message: String,
    /// Backup codes (for generate action)
    pub codes: Option<Vec<String>>,
    /// Number of remaining codes (for list action)
    pub remaining: Option<i32>,
}

/// Authenticate a user with username and password
pub async fn login(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthencError> {
    // Get user by username from database
    let user = state
        .user_store
        .get_user_by_username(&req.username)
        .await?
        .ok_or_else(|| AuthencError::unauthorized("Invalid credentials"))?;

    // Check if user belongs to the requested realm
    // For now, just check if the user has a realm_id that matches the master realm
    let master_realm_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")
        .map_err(|_| AuthencError::internal("Invalid master realm ID"))?;
    if user.realm_id != Some(master_realm_id) {
        return Err(AuthencError::unauthorized("Invalid credentials"));
    }

    // Verify password against stored hash
    let password_valid = if let Some(password_hash) = &user.password_hash {
        tracing::debug!(
            "Verifying password for user {} with hash prefix: {}",
            user.username,
            &password_hash[..20]
        );
        let result = crate::utils::crypto::password::verify_password(password_hash, &req.password)
            .map_err(|e| {
                tracing::error!("Password verification error for user {}: {}", user.username, e);
                AuthencError::internal(format!("Password verification error: {}", e))
            })?;
        tracing::debug!("Password verification result for user {}: {}", user.username, result);
        result
    } else {
        // No password hash stored - reject authentication
        tracing::warn!("No password hash stored for user {}", user.username);
        false
    };

    if !password_valid {
        tracing::warn!("Invalid password for user {}", user.username);
        // Fire login error event
        // ... (events)
        return Err(AuthencError::unauthorized("Invalid credentials"));
    }

    // CAPTCHA Validation
    if let Some(token) = &req.captcha_token {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 3 {
            return Err(AuthencError::validation("Invalid CAPTCHA token format"));
        }

        let challenge_id = parts[0];
        let timestamp_str = parts[1];
        let signature = parts[2];

        // 1. Check expiration (e.g., 10 minutes)
        let timestamp: u64 = timestamp_str
            .parse()
            .map_err(|_| AuthencError::validation("Invalid CAPTCHA timestamp"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > timestamp + 600 {
            return Err(AuthencError::unauthorized("CAPTCHA token expired"));
        }

        // 2. Verify signature
        let payload = format!("{}:{}", challenge_id, timestamp_str);
        let secret = &state.config.security.jwt_secret;

        // Use HKDF to derive key (matching gRPC and Validation logic)
        let hk = hkdf::Hkdf::<Sha256>::new(None, secret.as_bytes());
        let mut captcha_key = [0u8; 32];
        hk.expand(b"captcha-v1", &mut captcha_key)
            .map_err(|_| AuthencError::internal("HKDF expansion failed"))?;

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(&captcha_key)
            .map_err(|_| AuthencError::internal("HMAC initialization failed"))?;
        mac.update(payload.as_bytes());
        let result_mac = mac.finalize();
        let expected_signature = hex::encode(result_mac.into_bytes());

        if signature != expected_signature {
            return Err(AuthencError::unauthorized("Invalid CAPTCHA signature"));
        }
    } else {
        // Policy: Require CAPTCHA for all login attempts (skip in non-production)
        let app_env = std::env::var("APP_ENVIRONMENT").unwrap_or_default();
        if app_env != "staging" && app_env != "development" && app_env != "test" {
            return Err(AuthencError::unauthorized("CAPTCHA verification required"));
        }
        tracing::debug!("CAPTCHA check skipped in {} environment", app_env);
    }

    // Password is valid, now check MFA status
    if user.mfa_enabled {
        // User has MFA enabled - require verification
        let temp_token = jwt::generate_temp_jwt(&user.id.to_string())
            .map_err(|_| AuthencError::internal("Temp token generation failed"))?;

        // Fire partial login event (password successful, MFA pending)
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::Login,
            req.realm.clone(),
        )
        .user_id(user.id.to_string())
        .client_id("api".to_string())
        .detail("method", "password")
        .detail("mfa_status", "verification_required")
        .build();

        if let Err(e) = state.event_manager.write().await.fire_event(event).await {
            tracing::error!("Failed to fire partial login event: {}", e);
        }

        return Ok(Json(LoginResponse {
            access_token: None,
            temp_token: Some(temp_token),
            mfa_required: true,
            mfa_setup_required: false,
            message: "MFA verification required".to_string(),
        }));
    } else {
        // Check if MFA should be required for this user (policy-based)
        let mfa_required = should_require_mfa(&user).await?;

        if mfa_required {
            // First-time MFA setup required
            let temp_token = jwt::generate_temp_jwt(&user.id.to_string())
                .map_err(|_| AuthencError::internal("Temp token generation failed"))?;

            // Fire partial login event (password successful, MFA setup required)
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::Login,
                req.realm.clone(),
            )
            .user_id(user.id.to_string())
            .client_id("api".to_string())
            .detail("method", "password")
            .detail("mfa_status", "setup_required")
            .build();

            if let Err(e) = state.event_manager.write().await.fire_event(event).await {
                tracing::error!("Failed to fire partial login event: {}", e);
            }

            return Ok(Json(LoginResponse {
                access_token: None,
                temp_token: Some(temp_token),
                mfa_required: false,
                mfa_setup_required: true,
                message: "MFA setup required".to_string(),
            }));
        } else {
            // Normal login without MFA (for users not requiring MFA)
            let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
            let token =
                jwt::generate_jwt(&user.id.to_string(), Some(user.email.clone()), Some(roles))
                    .map_err(|_| AuthencError::internal("Token generation failed"))?;

            // Fire successful login event
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::Login,
                req.realm.clone(),
            )
            .user_id(user.id.to_string())
            .client_id("api".to_string())
            .detail("method", "password")
            .detail("mfa_status", "not_required")
            .build();

            if let Err(e) = state.event_manager.write().await.fire_event(event).await {
                tracing::error!("Failed to fire login event: {}", e);
            }

            return Ok(Json(LoginResponse {
                access_token: Some(token),
                temp_token: None,
                mfa_required: false,
                mfa_setup_required: false,
                message: "Login successful".to_string(),
            }));
        }
    }
}

/// Check if MFA should be required for a user based on policy
async fn should_require_mfa(_user: &crate::models::user::User) -> Result<bool, AuthencError> {
    // Policy-based MFA requirement logic
    // For government employees, MFA might be required based on:
    // - Role (admin, sensitive positions)
    // - Satker (certain organizational units)
    // - Security level of accessed resources

    // For now, require MFA for all users (government security policy)
    // In production, this could be more granular based on user roles/satker
    Ok(true)
}

/// Test login endpoint for development - creates a test user if it doesn't exist
pub async fn test_login(
    State(state): State<Arc<crate::app::AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<LoginResponse>, AuthencError> {
    use crate::services::stores::user_store::UserStoreTrait;

    let username = body["username"]
        .as_str()
        .unwrap_or("testuser");
    let password = body["password"]
        .as_str()
        .unwrap_or("");

    // Check if user exists
    let test_user = state
        .user_store
        .get_user_by_username(username)
        .await?
        .ok_or_else(|| AuthencError::internal("Test user not found"))?;

    // Verify password if provided
    if !password.is_empty() {
        let valid = lib_common::crypto::password::verify_password(
            test_user.password_hash.as_deref().unwrap_or(""),
            password,
        )
        .unwrap_or(false);
        if !valid {
            return Err(AuthencError::unauthorized("Invalid password"));
        }
    }

    // Generate JWT token
    let roles: Vec<String> = test_user.roles.iter().map(|r| r.name.clone()).collect();
    let token = jwt::generate_jwt(
        &test_user.id.to_string(),
        Some(test_user.email.clone()),
        Some(roles),
    )
    .map_err(|_| AuthencError::internal("Token generation failed"))?;
    let message = format!("Test login successful for user {}", test_user.username);

    // Fire successful login event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::Login,
        "test-realm".to_string(),
    )
    .user_id(test_user.id.to_string())
    .client_id("api".to_string())
    .detail("method", "test")
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire test login event: {}", e);
    }

    Ok(Json(LoginResponse {
        access_token: Some(token),
        temp_token: None,
        mfa_required: false,
        mfa_setup_required: false,
        message,
    }))
}

/// Setup MFA for a user (requires temporary token from login)
pub async fn mfa_setup(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<crate::services::mfa_service::MfaSetupResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Extract and verify temporary token
    let temp_token = req["temp_token"]
        .as_str()
        .ok_or_else(|| AuthencError::unauthorized("Temporary token required"))?;

    let claims = jwt::verify_jwt(temp_token)
        .map_err(|_| AuthencError::unauthorized("Invalid temporary token"))?;

    // Verify token purpose
    if claims.purpose.as_deref() != Some("mfa_verification") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Create MFA rate limiter (in production, this would be shared state)
    let mfa_rate_limiter =
        crate::middleware::MfaRateLimiterState::new(state.config.mfa_rate_limit.clone());

    // Create MFA security monitor
    let mfa_security_monitor = crate::services::MfaSecurityMonitor::new(
        crate::services::MfaSecurityMonitorConfig::default(),
        state.event_manager.clone(),
    );

    // Check MFA setup rate limits
    mfa_rate_limiter.check_mfa_setup_rate_limit(&ip).await?;

    // Record MFA setup for security monitoring
    if let Err(e) = mfa_security_monitor.record_mfa_setup(&ip, user_id).await {
        tracing::error!("Failed to record MFA setup: {}", e);
    }

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Setup MFA for the user
    let setup_response = mfa_service.setup_mfa(user_id).await?;

    // Fire MFA setup initiated event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::MfaSetup,
        "master".to_string(),
    )
    .user_id(user_id.to_string())
    .client_id("api".to_string())
    .detail("action", "setup_initiated")
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire MFA setup event: {}", e);
    }

    Ok(Json(setup_response))
}

/// Verify MFA setup with initial OTP code
pub async fn mfa_verify_setup(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<MfaSetupVerifyRequest>,
) -> Result<Json<MfaResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Verify temporary token
    let claims = jwt::verify_jwt(&req.temp_token)
        .map_err(|_| AuthencError::unauthorized("Invalid temporary token"))?;

    // Verify token purpose
    if claims.purpose.as_deref() != Some("mfa_verification") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Create MFA rate limiter (in production, this would be shared state)
    let mfa_rate_limiter =
        crate::middleware::MfaRateLimiterState::new(state.config.mfa_rate_limit.clone());

    // Check rate limits before processing
    mfa_rate_limiter
        .check_mfa_verify_rate_limit(&ip, Some(user_id))
        .await?;

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Verify setup with OTP code
    match mfa_service.verify_setup(user_id, &req.code).await {
        Ok(_) => {
            // Reset progressive delay on successful verification
            mfa_rate_limiter.reset_progressive_delay(&ip);
        }
        Err(e) => {
            // Record failed attempt for rate limiting
            if let Err(lockout_err) = mfa_rate_limiter
                .record_failed_mfa_attempt(&ip, user_id)
                .await
            {
                return Err(lockout_err);
            }
            return Err(e);
        }
    }

    // Get user details for token generation
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::internal("User not found"))?;

    let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();

    // Generate full access token after successful MFA setup
    let access_token =
        jwt::generate_jwt(&user_id.to_string(), Some(user.email.clone()), Some(roles))
            .map_err(|_| AuthencError::internal("Token generation failed"))?;

    // Fire MFA setup completed event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::MfaSetup,
        "master".to_string(),
    )
    .user_id(user_id.to_string())
    .client_id("api".to_string())
    .detail("action", "setup_completed")
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire MFA setup completed event: {}", e);
    }

    Ok(Json(MfaResponse {
        message: "MFA setup completed successfully".to_string(),
        access_token: Some(access_token),
    }))
}

/// Verify MFA code during login
pub async fn mfa_verify(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Verify temporary token
    let claims = jwt::verify_jwt(&req.temp_token)
        .map_err(|_| AuthencError::unauthorized("Invalid temporary token"))?;

    // Verify token purpose
    if claims.purpose.as_deref() != Some("mfa_verification") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Create MFA rate limiter (in production, this would be shared state)
    let mfa_rate_limiter =
        crate::middleware::MfaRateLimiterState::new(state.config.mfa_rate_limit.clone());

    // Create MFA security monitor
    let mfa_security_monitor = crate::services::MfaSecurityMonitor::new(
        crate::services::MfaSecurityMonitorConfig::default(),
        state.event_manager.clone(),
    );

    // Check rate limits before processing
    mfa_rate_limiter
        .check_mfa_verify_rate_limit(&ip, Some(user_id))
        .await?;

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Verify MFA code
    match mfa_service.verify_mfa(user_id, &req.code).await {
        Ok(_) => {
            // Reset progressive delay on successful verification
            mfa_rate_limiter.reset_progressive_delay(&ip);

            // Record successful MFA for pattern learning
            if let Err(e) = mfa_security_monitor
                .record_successful_mfa(user_id, &ip, None)
                .await
            {
                tracing::error!("Failed to record successful MFA: {}", e);
            }
        }
        Err(e) => {
            // Record failed attempt for security monitoring
            if let Err(monitor_err) = mfa_security_monitor
                .record_failed_mfa_attempt(&ip, Some(user_id), None)
                .await
            {
                tracing::error!("Failed to record failed MFA attempt: {}", monitor_err);
            }

            // Record failed attempt for rate limiting and account lockout
            if let Err(lockout_err) = mfa_rate_limiter
                .record_failed_mfa_attempt(&ip, user_id)
                .await
            {
                // Also record the lockout event
                if let Err(monitor_err) = mfa_security_monitor
                    .record_account_lockout(user_id, &ip, 5)
                    .await
                {
                    tracing::error!("Failed to record account lockout: {}", monitor_err);
                }
                return Err(lockout_err);
            }
            return Err(e);
        }
    }

    // Get user details for token generation
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::internal("User not found"))?;

    let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();

    // Generate full access token after successful MFA verification
    let access_token =
        jwt::generate_jwt(&user_id.to_string(), Some(user.email.clone()), Some(roles))
            .map_err(|_| AuthencError::internal("Token generation failed"))?;

    // Fire MFA verification successful event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::MfaVerification,
        "master".to_string(),
    )
    .user_id(user_id.to_string())
    .client_id("api".to_string())
    .detail("action", "verification_successful")
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire MFA verification event: {}", e);
    }

    Ok(Json(MfaResponse {
        message: "MFA verification successful".to_string(),
        access_token: Some(access_token),
    }))
}

/// Get MFA status for a user
pub async fn mfa_status(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<crate::services::mfa_service::MfaStatus>, AuthencError> {
    // Extract and verify token (can be temp or full token)
    let token = req["token"]
        .as_str()
        .ok_or_else(|| AuthencError::unauthorized("Token required"))?;

    let claims = jwt::verify_jwt(token).map_err(|_| AuthencError::unauthorized("Invalid token"))?;

    // Verify token purpose (must be access token, not mfa_verification token)
    if claims.purpose.as_deref() != Some("access") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Get MFA status
    let status = mfa_service.get_mfa_status(user_id).await?;

    Ok(Json(status))
}

/// Disable MFA for a user (admin only)
pub async fn mfa_disable(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<MfaDisableRequest>,
) -> Result<Json<MfaResponse>, AuthencError> {
    // Verify admin token
    let claims = jwt::verify_jwt(&req.admin_token)
        .map_err(|_| AuthencError::unauthorized("Invalid admin token"))?;

    // Verify token purpose (must be access token, not mfa_verification token)
    if claims.purpose.as_deref() != Some("access") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let admin_user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid admin user ID in token"))?;

    // Check if user has MFA bypass capability (replaces hardcoded role check)
    state
        .capability_checker
        .require_any_capability(
            &admin_user_id,
            &[capabilities::MFA_BYPASS, capabilities::SYSTEM_ADMIN],
        )
        .await?;

    let target_user_id =
        Uuid::parse_str(&req.user_id).map_err(|_| AuthencError::validation("Invalid user ID"))?;

    // Create admin security context
    let admin_context = crate::models::user::SecurityContext {
        ip_address: None, // TODO: Extract from request
        user_agent: None, // TODO: Extract from request
        session_id: None,
        timestamp: chrono::Utc::now(),
        risk_score: Some(0.0), // Admin operation
        metadata: Some(serde_json::json!({
            "admin_user_id": admin_user_id.to_string(),
            "reason": req.reason.clone().unwrap_or_else(|| "Admin disable".to_string()),
            "action": "mfa_disable"
        })),
    };

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Disable MFA for the target user
    mfa_service
        .disable_mfa(target_user_id, &admin_context)
        .await?;

    // Fire MFA disabled event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::MfaDisabled,
        "master".to_string(),
    )
    .user_id(target_user_id.to_string())
    .client_id("api".to_string())
    .detail("admin_user_id", admin_user_id.to_string())
    .detail(
        "reason",
        req.reason.unwrap_or_else(|| "Admin disable".to_string()),
    )
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire MFA disabled event: {}", e);
    }

    Ok(Json(MfaResponse {
        message: "MFA disabled successfully".to_string(),
        access_token: None,
    }))
}

/// Reset MFA configuration for a user
pub async fn mfa_reset(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<MfaResetRequest>,
) -> Result<Json<MfaResponse>, AuthencError> {
    // Verify token
    let claims =
        jwt::verify_jwt(&req.token).map_err(|_| AuthencError::unauthorized("Invalid token"))?;

    // Verify token purpose (must be access token, not mfa_verification token)
    if claims.purpose.as_deref() != Some("access") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let requesting_user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    let target_user_id = if let Some(user_id_str) = &req.user_id {
        // Admin resetting another user's MFA
        let target_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AuthencError::validation("Invalid target user ID"))?;

        // Check if requesting user has MFA bypass capability
        state
            .capability_checker
            .require_any_capability(
                &requesting_user_id,
                &[capabilities::MFA_BYPASS, capabilities::SYSTEM_ADMIN],
            )
            .await?;

        target_id
    } else {
        // User resetting their own MFA
        requesting_user_id
    };

    // Create security context
    let security_context = crate::models::user::SecurityContext {
        ip_address: None, // TODO: Extract from request
        user_agent: None, // TODO: Extract from request
        session_id: None,
        timestamp: chrono::Utc::now(),
        risk_score: Some(if req.user_id.is_some() { 0.0 } else { 0.2 }), // Lower risk for admin operations
        metadata: Some(serde_json::json!({
            "requesting_user_id": requesting_user_id.to_string(),
            "target_user_id": target_user_id.to_string(),
            "reason": req.reason.clone().unwrap_or_else(|| "MFA reset".to_string()),
            "action": "mfa_reset",
            "is_admin_operation": req.user_id.is_some()
        })),
    };

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Reset MFA (disable and allow re-setup)
    mfa_service
        .disable_mfa(target_user_id, &security_context)
        .await?;

    // Fire MFA reset event
    let event = crate::services::events::EventBuilder::new(
        crate::models::events::EventType::MfaReset,
        "master".to_string(),
    )
    .user_id(target_user_id.to_string())
    .client_id("api".to_string())
    .detail("requesting_user_id", requesting_user_id.to_string())
    .detail(
        "reason",
        req.reason.unwrap_or_else(|| "MFA reset".to_string()),
    )
    .detail("is_admin_operation", req.user_id.is_some().to_string())
    .build();

    if let Err(e) = state.event_manager.write().await.fire_event(event).await {
        tracing::error!("Failed to fire MFA reset event: {}", e);
    }

    Ok(Json(MfaResponse {
        message: "MFA reset successfully. User can now set up MFA again.".to_string(),
        access_token: None,
    }))
}

/// Manage backup codes for MFA
pub async fn mfa_backup_codes(
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<MfaBackupCodesRequest>,
) -> Result<Json<MfaBackupCodesResponse>, AuthencError> {
    // Verify token
    let claims =
        jwt::verify_jwt(&req.token).map_err(|_| AuthencError::unauthorized("Invalid token"))?;

    // Verify token purpose (must be access token, not mfa_verification token)
    if claims.purpose.as_deref() != Some("access") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Create MFA service instance
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    match req.action.as_str() {
        "generate" => {
            // Generate new backup codes
            let new_codes = mfa_service.regenerate_recovery_codes(user_id).await?;

            // Fire backup codes generated event
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::MfaBackupCodesGenerated,
                "master".to_string(),
            )
            .user_id(user_id.to_string())
            .client_id("api".to_string())
            .detail("action", "generate")
            .detail("codes_count", new_codes.len().to_string())
            .build();

            if let Err(e) = state.event_manager.write().await.fire_event(event).await {
                tracing::error!("Failed to fire backup codes generated event: {}", e);
            }

            Ok(Json(MfaBackupCodesResponse {
                message: "New backup codes generated successfully".to_string(),
                codes: Some(new_codes),
                remaining: None,
            }))
        }
        "list" => {
            // Get current status (number of remaining codes)
            let status = mfa_service.get_mfa_status(user_id).await?;

            Ok(Json(MfaBackupCodesResponse {
                message: "Backup codes status retrieved".to_string(),
                codes: None,
                remaining: Some(status.backup_codes_remaining),
            }))
        }
        "verify" => {
            // Verify a backup code
            let code = req
                .code
                .ok_or_else(|| AuthencError::validation("Code required for verify action"))?;

            mfa_service.verify_recovery_code(user_id, &code).await?;

            // Fire backup code used event
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::MfaBackupCodeUsed,
                "master".to_string(),
            )
            .user_id(user_id.to_string())
            .client_id("api".to_string())
            .detail("action", "verify")
            .build();

            if let Err(e) = state.event_manager.write().await.fire_event(event).await {
                tracing::error!("Failed to fire backup code used event: {}", e);
            }

            Ok(Json(MfaBackupCodesResponse {
                message: "Backup code verified successfully".to_string(),
                codes: None,
                remaining: None,
            }))
        }
        _ => Err(AuthencError::validation(
            "Invalid action. Supported actions: generate, list, verify",
        )),
    }
}
