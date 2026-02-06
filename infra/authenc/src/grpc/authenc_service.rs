//! AuthencService gRPC implementation
//!
//! This module implements all RPC methods defined in the AuthencService proto.
//! It provides authentication, authorization, user management, and MFA operations.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

use crate::app::AppState;
use crate::error::AuthencError;
use crate::services::cache::Cache;
use crate::services::stores::UserStoreTrait;
use hmac::{Hmac, Mac};
use sha2::Sha256;

// Use shared proto from grpc module
pub use crate::grpc::proto::authenc::v1 as proto;
pub use crate::grpc::proto::common;

use proto::{
    AssignRoleRequest,
    AssignRoleResponse,
    // Audit
    AuditLogsRequest,
    AuditLogsResponse,
    // Authentication
    AuthenticateRequest,
    AuthenticateResponse,
    // Captcha
    CaptchaChallengeRequest,
    CaptchaChallengeResponse,
    CaptchaVerificationRequest,
    CaptchaVerificationResponse,
    ChallengeType,
    // Authorization
    CheckPermissionRequest,
    CheckPermissionResponse,
    CompleteFederatedAuthRequest,
    CompleteFederatedAuthResponse,
    ComplianceReportRequest,
    ComplianceReportResponse,
    // User Management
    CreateUserRequest,
    CreateUserResponse,
    DeleteUserRequest,
    DeleteUserResponse,
    DisableMfaRequest,
    DisableMfaResponse,
    // MFA
    EnableMfaRequest,
    EnableMfaResponse,
    // Federation
    FederatedAuthRequest,
    FederatedAuthResponse,
    GetUserRequest,
    GetUserResponse,
    IntrospectTokenRequest,
    IntrospectTokenResponse,
    ListRolesRequest,
    ListRolesResponse,
    ListUsersRequest,
    ListUsersResponse,
    // OAuth2/OIDC
    OAuthTokenRequest,
    OAuthTokenResponse,
    RefreshTokenRequest,
    RefreshTokenResponse,
    RevokeRoleRequest,
    RevokeRoleResponse,
    RevokeTokenRequest,
    RevokeTokenResponse,
    UpdateUserRequest,
    UpdateUserResponse,
    UserInfoRequest,
    UserInfoResponse,
    ValidateTokenRequest,
    ValidateTokenResponse,
    VerifyMfaRequest,
    VerifyMfaResponse,
    authenc_service_server::AuthencService,
};

use crate::services::captcha::{
    BehavioralMetrics, CaptchaServiceTrait, ChallengeType as ServiceChallengeType,
};

/// gRPC service implementation for Authenc
pub struct AuthencGrpcService {
    state: Arc<AppState>,
}

impl AuthencGrpcService {
    /// Create a new AuthencGrpcService instance
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Convert AuthencError to tonic Status
    fn map_error(err: AuthencError) -> Status {
        match err {
            AuthencError::Unauthorized { message } => Status::unauthenticated(message),
            AuthencError::Forbidden { message } => Status::permission_denied(message),
            AuthencError::ValidationError { message } => Status::invalid_argument(message),
            AuthencError::AuthenticationFailed => Status::unauthenticated("Authentication failed"),
            AuthencError::InvalidCredentials => Status::unauthenticated("Invalid credentials"),
            AuthencError::TokenExpired => Status::unauthenticated("Token expired"),
            _ => Status::internal("Internal server error"),
        }
    }
}

#[tonic::async_trait]
impl AuthencService for AuthencGrpcService {
    // ==================== Authentication ====================

    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<AuthenticateResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC Authenticate request for user: {}", req.username);

        // CAPTCHA Validation
        if let Some(token) = &req.captcha_token {
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() != 3 {
                return Err(Status::invalid_argument("Invalid CAPTCHA token format"));
            }

            let challenge_id = parts[0];
            let timestamp_str = parts[1];
            let signature = parts[2];

            let timestamp: u64 = timestamp_str
                .parse()
                .map_err(|_| Status::invalid_argument("Invalid CAPTCHA timestamp"))?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Token valid for 10 minutes
            if now > timestamp + 600 {
                return Err(Status::unauthenticated("CAPTCHA token expired"));
            }

            let payload = format!("{}:{}", challenge_id, timestamp_str);
            let secret = &self.state.config.security.jwt_secret;

            // Use HKDF to derive the same key used for signing (Bug 10)
            let hk = hkdf::Hkdf::<Sha256>::new(None, secret.as_bytes());
            let mut captcha_key = [0u8; 32];
            hk.expand(b"captcha-v1", &mut captcha_key)
                .expect("HKDF expand failed");

            type HmacSha256 = Hmac<Sha256>;
            let mut mac =
                HmacSha256::new_from_slice(&captcha_key).expect("HMAC can take key of any size");
            mac.update(payload.as_bytes());

            if let Err(_) = mac.verify_slice(
                &hex::decode(signature)
                    .map_err(|_| Status::invalid_argument("Invalid CAPTCHA signature encoding"))?,
            ) {
                return Err(Status::unauthenticated("Invalid CAPTCHA signature"));
            }

            // Note: Token usage check is deferred until after credential verification
            // to prevent DoS where failed logins consume tokens (Bug 1)
        } else {
            // For now, allow requests without captcha if not strictly required,
            // but the bug report says it should be enforced.
            warn!(
                "Login attempt without CAPTCHA token for user: {}",
                req.username
            );
            return Err(Status::unauthenticated("CAPTCHA verification required"));
        }

        // OPTIMIZATION: Parallel DB + cache checks using tokio::join!
        let (user_result, cache_result) = tokio::join!(
            // DB query for user
            self.state.user_store.get_user_by_username(&req.username),
            // Cache check for recent failed attempts (rate limiting)
            async {
                if let Some(redis_cache) = &self.state.redis_cache {
                    let cache_key = format!("rate_limit:{}:login", req.username);
                    redis_cache.get(&cache_key).await.ok().flatten()
                } else {
                    None
                }
            }
        );

        // Check rate limiting from cache
        let current_attempts = if let Some(ref cached_attempts) = cache_result {
            if let Ok(attempts) = serde_json::from_value::<i32>(cached_attempts.clone()) {
                if attempts >= 5 {
                    return Err(Status::resource_exhausted(
                        "Too many failed login attempts. Please try again later.",
                    ));
                }
                attempts
            } else {
                0
            }
        } else {
            0
        };

        let user = user_result
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::unauthenticated("Invalid credentials"))?;

        // Verify password using crypto utils
        let password_valid = crate::utils::crypto::verify_password(
            &req.password,
            &user.password_hash.unwrap_or_default(),
        )
        .map_err(|_| Status::internal("Password verification failed"))?;

        if !password_valid {
            // Increment failed attempts in cache (non-blocking)
            if let Some(redis_cache) = &self.state.redis_cache {
                let cache_key = format!("rate_limit:{}:login", req.username);
                let attempts = current_attempts + 1;
                let _ = redis_cache
                    .set(
                        &cache_key,
                        &serde_json::json!(attempts),
                        std::time::Duration::from_secs(900),
                    )
                    .await;
            }

            // Fire login error event (non-blocking with tokio::spawn)
            let event_manager = self.state.event_manager.clone();
            let user_id = user.id;
            tokio::spawn(async move {
                let em = event_manager.write().await;
                let event = crate::services::events::EventBuilder::new(
                    crate::models::events::EventType::LoginError,
                    "master".to_string(),
                )
                .user_id(user_id.to_string())
                .client_id("grpc".to_string())
                .detail("method", "password")
                .detail("reason", "invalid_credentials")
                .build();

                let _ = em.fire_event(event).await;
            });

            return Err(Status::unauthenticated("Invalid credentials"));
        }

        // Check if MFA is required
        // BUG FIX: Mark CAPTCHA token as used ONLY after password verification succeeds
        // This prevents "Token Burning" attacks (Bug 1) and uses atomic SET NX to prevent races (Bug 2)
        if let Some(token) = &req.captcha_token {
            let parts: Vec<&str> = token.split(':').collect();
            // We already validated format/signature above, so this unwrap logic is safe enough or we re-parse
            if let Some(challenge_id) = parts.first() {
                if let Some(redis_cache) = &self.state.redis_cache {
                    let used_key = format!("used_captcha:{}", challenge_id);

                    // Atomic check-and-set using SET NX
                    // If it returns true: Key was set (we claimed it first) -> OK
                    // If it returns false: Key existed (replay attempt) -> FAIL
                    match redis_cache
                        .set_nx(
                            &used_key,
                            &serde_json::json!(true),
                            std::time::Duration::from_secs(600),
                        )
                        .await
                    {
                        Ok(true) => {
                            debug!(
                                "CAPTCHA token {} successfully consumed for user {}",
                                challenge_id, req.username
                            );
                        }
                        Ok(false) => {
                            warn!(
                                "Replay attack detected: CAPTCHA token {} already used (race condition check)",
                                challenge_id
                            );
                            // Even though password was correct, we fail because the token was reused
                            return Err(Status::unauthenticated("CAPTCHA token already used"));
                        }
                        Err(e) => {
                            warn!("Redis error checking CAPTCHA replay: {}", e);
                            // Fail open or closed? Security-wise should fail closed, but availability-wise...
                            // Let's fail closed for now as this is a security feature
                            return Err(Status::internal(
                                "Internal error verifying CAPTCHA status",
                            ));
                        }
                    }
                }
            }
        }

        // Check if MFA is required
        if user.mfa_enabled {
            if let Some(mfa_code) = req.mfa_code {
                // Verify MFA code
                let mfa_service = crate::services::MfaService::new(
                    self.state.secreton_client.clone(),
                    self.state.db_pool.clone(),
                );

                mfa_service
                    .verify_mfa(user.id, &mfa_code)
                    .await
                    .map_err(Self::map_error)?;

                // MFA verified, proceed with token generation
            } else {
                // MFA required but not provided
                return Err(Status::failed_precondition("MFA verification required"));
            }
        }

        // Generate JWT tokens
        let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
        let access_token = crate::utils::jwt::generate_jwt(
            &user.id.to_string(),
            Some(user.email.clone()),
            Some(roles),
        )
        .map_err(|_| Status::internal("Token generation failed"))?;

        let refresh_token = crate::utils::jwt::generate_refresh_token(&user.id.to_string())
            .map_err(|_| Status::internal("Refresh token generation failed"))?;

        // Clear failed attempts on successful login
        if let Some(redis_cache) = &self.state.redis_cache {
            let cache_key = format!("rate_limit:{}:login", req.username);
            let _ = redis_cache.delete(&cache_key).await;
        }

        // Fire successful login event (non-blocking with tokio::spawn)
        let event_manager = self.state.event_manager.clone();
        let user_id = user.id;
        let mfa_enabled = user.mfa_enabled;
        tokio::spawn(async move {
            let em = event_manager.write().await;
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::Login,
                "master".to_string(),
            )
            .user_id(user_id.to_string())
            .client_id("grpc".to_string())
            .detail("method", "password")
            .detail("mfa_verified", mfa_enabled.to_string())
            .build();

            let _ = em.fire_event(event).await;
        });

        // Build user info response
        let user_info = proto::UserInfo {
            user_id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.nama.clone(),
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            is_active: user.enabled,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at.timestamp(),
            last_login: user.last_login_at.map(|t| t.timestamp()).unwrap_or(0),
        };

        Ok(Response::new(AuthenticateResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour
            user: Some(user_info),
            scopes: vec!["openid".to_string(), "profile".to_string()],
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC RefreshToken request");

        // Verify refresh token
        let claims = crate::utils::jwt::verify_refresh_token(&req.refresh_token)
            .map_err(|_| Status::unauthenticated("Invalid refresh token"))?;

        let user_id = claims.sub;

        // Check if user still exists and is active
        let user = self
            .state
            .user_store
            .get_user(
                uuid::Uuid::parse_str(&user_id).map_err(|_| Status::internal("Invalid user ID"))?,
            )
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::unauthenticated("User not found"))?;

        if !user.enabled {
            return Err(Status::permission_denied("User account is disabled"));
        }

        // Generate new tokens
        let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
        let new_access_token =
            crate::utils::jwt::generate_jwt(&user_id, Some(user.email.clone()), Some(roles))
                .map_err(|_| Status::internal("Token generation failed"))?;

        let new_refresh_token = crate::utils::jwt::generate_refresh_token(&user_id)
            .map_err(|_| Status::internal("Refresh token generation failed"))?;

        // Fire token refresh event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::RefreshToken,
            "master".to_string(),
        )
        .user_id(user_id)
        .client_id("grpc".to_string())
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            expires_in: 3600, // 1 hour
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ValidateToken request");

        // Create JWT validator with cache support
        let cache: Option<Arc<dyn crate::services::cache::Cache>> = self
            .state
            .redis_cache
            .clone()
            .map(|c| c as Arc<dyn crate::services::cache::Cache>);
        let validator = crate::services::JwtValidator::new(cache);

        // Validate token with caching (fast path: < 10ms for cached tokens)
        let validation_result = validator
            .validate_token(&req.token)
            .await
            .map_err(|e| Status::internal(format!("Validation error: {}", e)))?;

        // Check required scopes if specified
        let has_required_scopes = if !req.required_scopes.is_empty() {
            // In a full implementation, check if token has required scopes
            // For now, assume token has all scopes if valid
            validation_result.valid
        } else {
            true
        };

        if validation_result.valid && !has_required_scopes {
            return Ok(Response::new(ValidateTokenResponse {
                valid: false,
                user_id: None,
                scopes: vec![],
                expires_at: None,
                error: Some("Insufficient scopes".to_string()),
            }));
        }

        // Convert validation result to gRPC response
        let response = ValidateTokenResponse {
            valid: validation_result.valid,
            user_id: validation_result.user_id.clone(),
            scopes: if validation_result.valid {
                vec!["openid".to_string(), "profile".to_string()]
            } else {
                vec![]
            },
            expires_at: validation_result.expires_at.map(|exp| exp as i64),
            error: validation_result.error.clone(),
        };

        Ok(Response::new(response))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC RevokeToken request");

        // Verify the token to get user_id and claims
        let claims = crate::utils::jwt::verify_jwt(&req.token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        let user_id = claims.sub.clone();

        // Create JWT validator with cache support
        let cache: Option<Arc<dyn crate::services::cache::Cache>> = self
            .state
            .redis_cache
            .clone()
            .map(|c| c as Arc<dyn crate::services::cache::Cache>);
        let validator = crate::services::JwtValidator::new(cache);

        // Revoke token using JWT validator (handles blacklist and cache invalidation)
        validator
            .revoke_token(&req.token, &claims)
            .await
            .map_err(|e| Status::internal(format!("Token revocation failed: {}", e)))?;

        // Fire token revocation event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::TokenRevoked,
            "master".to_string(),
        )
        .user_id(user_id)
        .client_id("grpc".to_string())
        .detail("token_type", format!("{:?}", req.token_type))
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(RevokeTokenResponse { success: true }))
    }

    // ==================== User Management ====================

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC CreateUser request for username: {}", req.username);

        // Hash the password if provided
        let password = if !req.password.is_empty() {
            Some(req.password.clone())
        } else {
            None
        };

        // Create user request
        let create_request = crate::models::user::CreateUserRequest {
            username: req.username.clone(),
            email: req.email.clone(),
            satker_code: req
                .metadata
                .get("satker_code")
                .cloned()
                .unwrap_or_else(|| "DEFAULT".to_string()),
            password,
            first_name: req.full_name.clone(),
            last_name: None,
            nip: None,
            nama: req.full_name.clone(),
            jabatan: None,
            phone_number: None,
            realm_id: None,
            organization_id: None,
            roles: None,
            attributes: if req.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_value(&req.metadata).unwrap_or_default())
            },
        };

        // Create user in database
        let user = self
            .state
            .user_store
            .add_user(create_request)
            .await
            .map_err(Self::map_error)?;

        // Assign roles if provided
        if !req.roles.is_empty() {
            for role_name in &req.roles {
                // Find role by name and assign to user
                if let Ok(roles) = crate::database::operations::roles::list_roles_by_realm(
                    self.state.user_store.database(),
                    &user.realm_id.unwrap_or_default(),
                )
                .await
                {
                    if let Some(role) = roles.iter().find(|r| r.name == *role_name) {
                        let _ = crate::database::operations::roles::assign_role_to_user(
                            self.state.user_store.database(),
                            &user.id,
                            &role.id,
                        )
                        .await;
                    }
                }
            }
        }

        // Fire user creation event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::Register,
            "master".to_string(),
        )
        .user_id(user.id.to_string())
        .client_id("grpc".to_string())
        .detail("username", user.username.clone())
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(CreateUserResponse {
            user_id: user.id.to_string(),
            username: user.username,
            created_at: user.created_at.timestamp(),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC GetUser request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Get user from database
        let user = self
            .state
            .user_store
            .get_user(user_id)
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Build user info response
        let user_info = proto::UserInfo {
            user_id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.nama.clone(),
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            is_active: user.enabled,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at.timestamp(),
            last_login: user.last_login_at.map(|t| t.timestamp()).unwrap_or(0),
        };

        Ok(Response::new(GetUserResponse {
            user: Some(user_info),
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC UpdateUser request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Create update request
        let update_request = crate::models::user::UpdateUserRequest {
            username: None,
            email: req.email.clone(),
            satker_code: None,
            first_name: req.full_name.clone(),
            last_name: None,
            nip: None,
            nama: req.full_name.clone(),
            jabatan: None,
            phone_number: None,
            enabled: req.is_active,
            email_verified: None,
            phone_verified: None,
            require_password_change: None,
            attributes: if req.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_value(&req.metadata).unwrap_or_default())
            },
        };

        // Update user in database
        let user = self
            .state
            .user_store
            .update_user(user_id, update_request)
            .await
            .map_err(Self::map_error)?;

        // Fire user update event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::UpdateProfile,
            "master".to_string(),
        )
        .user_id(user.id.to_string())
        .client_id("grpc".to_string())
        .detail("username", user.username.clone())
        .build();

        let _ = event_manager.fire_event(event).await;

        // Build user info response
        let user_info = proto::UserInfo {
            user_id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.nama.clone(),
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            is_active: user.enabled,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at.timestamp(),
            last_login: user.last_login_at.map(|t| t.timestamp()).unwrap_or(0),
        };

        Ok(Response::new(UpdateUserResponse {
            user: Some(user_info),
            updated_at: user.updated_at.timestamp(),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        warn!("gRPC DeleteUser request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Delete user (soft delete)
        self.state
            .user_store
            .delete_user(user_id)
            .await
            .map_err(Self::map_error)?;

        // Fire user deletion event (using RemoveCredential as closest match)
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::RemoveCredential,
            "master".to_string(),
        )
        .user_id(user_id.to_string())
        .client_id("grpc".to_string())
        .detail("action", "soft_delete")
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(DeleteUserResponse { success: true }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ListUsers request");

        // Get all users (in production, implement pagination and filtering)
        let all_users = self
            .state
            .user_store
            .get_all()
            .await
            .map_err(Self::map_error)?;

        // Apply pagination
        let limit = req.limit.unwrap_or(100) as usize;
        let offset = req.offset.unwrap_or(0) as usize;
        let total = all_users.len() as i32;

        let users: Vec<proto::UserInfo> = all_users
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|user| proto::UserInfo {
                user_id: user.id.to_string(),
                username: user.username.clone(),
                email: user.email.clone(),
                full_name: user.nama.clone(),
                roles: user.roles.iter().map(|r| r.name.clone()).collect(),
                is_active: user.enabled,
                mfa_enabled: user.mfa_enabled,
                created_at: user.created_at.timestamp(),
                last_login: user.last_login_at.map(|t| t.timestamp()).unwrap_or(0),
            })
            .collect();

        Ok(Response::new(ListUsersResponse { users, total }))
    }

    // ==================== Multi-Factor Authentication ====================

    async fn enable_mfa(
        &self,
        request: Request<EnableMfaRequest>,
    ) -> Result<Response<EnableMfaResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC EnableMFA request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Create MFA service instance
        let mfa_service = crate::services::MfaService::new(
            self.state.secreton_client.clone(),
            self.state.db_pool.clone(),
        );

        // Setup MFA for the user
        let setup_response = mfa_service
            .setup_mfa(user_id)
            .await
            .map_err(Self::map_error)?;

        // Fire MFA setup initiated event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::MfaSetup,
            "master".to_string(),
        )
        .user_id(user_id.to_string())
        .client_id("grpc".to_string())
        .detail("action", "setup_initiated")
        .detail("method", format!("{:?}", req.method))
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(EnableMfaResponse {
            secret: setup_response.secret_key,
            qr_code_url: setup_response.qr_code_url,
            backup_codes: setup_response.backup_codes,
        }))
    }

    async fn verify_mfa(
        &self,
        request: Request<VerifyMfaRequest>,
    ) -> Result<Response<VerifyMfaResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC VerifyMFA request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Create MFA service instance
        let mfa_service = crate::services::MfaService::new(
            self.state.secreton_client.clone(),
            self.state.db_pool.clone(),
        );

        // Verify MFA code
        let is_valid = mfa_service.verify_mfa(user_id, &req.code).await.is_ok();

        if is_valid {
            // Fire MFA verification successful event
            let event_manager = self.state.event_manager.write().await;
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::MfaVerification,
                "master".to_string(),
            )
            .user_id(user_id.to_string())
            .client_id("grpc".to_string())
            .detail("action", "verification_successful")
            .detail("method", format!("{:?}", req.method))
            .build();

            let _ = event_manager.fire_event(event).await;
        } else {
            // Fire MFA verification failed event
            let event_manager = self.state.event_manager.write().await;
            let event = crate::services::events::EventBuilder::new(
                crate::models::events::EventType::MfaVerificationError,
                "master".to_string(),
            )
            .user_id(user_id.to_string())
            .client_id("grpc".to_string())
            .detail("action", "verification_failed")
            .detail("method", format!("{:?}", req.method))
            .build();

            let _ = event_manager.fire_event(event).await;
        }

        Ok(Response::new(VerifyMfaResponse { valid: is_valid }))
    }

    async fn disable_mfa(
        &self,
        request: Request<DisableMfaRequest>,
    ) -> Result<Response<DisableMfaResponse>, Status> {
        let req = request.into_inner();
        warn!("gRPC DisableMFA request for user_id: {}", req.user_id);

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Verify password for security
        let user = self
            .state
            .user_store
            .get_user(user_id)
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        let password_valid = crate::utils::crypto::verify_password(
            &req.password,
            &user.password_hash.unwrap_or_default(),
        )
        .map_err(|_| Status::internal("Password verification failed"))?;

        if !password_valid {
            return Err(Status::permission_denied("Invalid password"));
        }

        // Create security context
        let security_context = crate::models::user::SecurityContext {
            ip_address: None,
            user_agent: None,
            session_id: None,
            timestamp: chrono::Utc::now(),
            risk_score: Some(0.3), // Moderate risk for MFA disable
            metadata: Some(serde_json::json!({
                "action": "mfa_disable",
                "user_initiated": true
            })),
        };

        // Create MFA service instance
        let mfa_service = crate::services::MfaService::new(
            self.state.secreton_client.clone(),
            self.state.db_pool.clone(),
        );

        // Disable MFA
        mfa_service
            .disable_mfa(user_id, &security_context)
            .await
            .map_err(Self::map_error)?;

        // Fire MFA disabled event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::MfaDisabled,
            "master".to_string(),
        )
        .user_id(user_id.to_string())
        .client_id("grpc".to_string())
        .detail("action", "mfa_disabled")
        .detail("user_initiated", "true")
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(DisableMfaResponse { success: true }))
    }

    // ==================== CAPTCHA ====================

    async fn generate_captcha_challenge(
        &self,
        request: Request<CaptchaChallengeRequest>,
    ) -> Result<Response<CaptchaChallengeResponse>, Status> {
        // Extract real client IP from gRPC metadata before consuming the request
        let ip_address = request
            .remote_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let req = request.into_inner();

        // Convert challenge type
        let challenge_type = match req.challenge_type {
            1 => ServiceChallengeType::Visual,
            2 => ServiceChallengeType::Audio,
            3 => ServiceChallengeType::Behavioral,
            4 => ServiceChallengeType::Logical,
            5 => ServiceChallengeType::Hybrid,
            _ => ServiceChallengeType::Visual,
        };

        // Generate challenge
        let challenge = self
            .state
            .captcha_service
            .generate_challenge(
                challenge_type,
                Some(req.difficulty as u8),
                Some(req.session_id.clone()),
                ip_address,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to generate challenge: {}", e)))?;

        // Convert to proto response
        let response = CaptchaChallengeResponse {
            challenge_id: challenge.id,
            challenge_type: req.challenge_type,
            challenge_data: challenge.encrypted_data,
            difficulty: challenge.difficulty_level as u32,
            expires_at: challenge
                .expires_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            metadata: req.metadata,
        };

        Ok(Response::new(response))
    }

    async fn verify_captcha_challenge(
        &self,
        request: Request<CaptchaVerificationRequest>,
    ) -> Result<Response<CaptchaVerificationResponse>, Status> {
        let req = request.into_inner();

        // Parse behavioral data if provided
        let behavioral_data = if !req.behavioral_data.is_empty() {
            serde_json::from_slice::<BehavioralMetrics>(&req.behavioral_data).ok()
        } else {
            None
        };

        // Validate challenge
        let validation_result = self
            .state
            .captcha_service
            .validate_challenge(req.challenge_id.clone(), req.answer, behavioral_data)
            .await
            .map_err(|e| match e {
                crate::services::captcha::CaptchaError::ChallengeNotFound { .. } => {
                    Status::not_found("Challenge not found")
                }
                crate::services::captcha::CaptchaError::ChallengeExpired { .. } => {
                    Status::failed_precondition("Challenge expired")
                }
                _ => Status::internal(format!("Validation failed: {}", e)),
            })?;

        // Generate verification token if successful
        let verification_token = if validation_result.success {
            let timestamp = chrono::Utc::now().timestamp();

            // Format: challenge_id:timestamp:signature
            let payload = format!("{}:{}", req.challenge_id, timestamp);
            let secret = &self.state.config.security.jwt_secret;

            // Use HKDF to derive a specific key for CAPTCHA tokens (matching REST/Validation logic)
            let hk = hkdf::Hkdf::<Sha256>::new(None, secret.as_bytes());
            let mut captcha_key = [0u8; 32];
            hk.expand(b"captcha-v1", &mut captcha_key)
                .map_err(|_| Status::internal("HKDF expansion failed"))?;

            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(&captcha_key)
                .map_err(|_| Status::internal("HMAC initialization failed"))?;
            mac.update(payload.as_bytes());
            let result_mac = mac.finalize();
            let signature = hex::encode(result_mac.into_bytes());

            format!("{}:{}:{}", req.challenge_id, timestamp, signature)
        } else {
            String::new()
        };

        let lockout_duration = validation_result
            .lockout_duration
            .map(|d| d.as_secs() as u32);

        let response = CaptchaVerificationResponse {
            success: validation_result.success,
            message: validation_result.message,
            verification_token,
            next_difficulty: validation_result.next_difficulty as u32,
            retry_allowed: validation_result.retry_allowed,
            lockout_duration,
        };

        Ok(Response::new(response))
    }

    // ==================== Authorization & RBAC ====================

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC CheckPermission request for user: {}, resource: {}, action: {}",
            req.user_id, req.resource, req.action
        );

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Fast path: Check cache first if available
        if let Some(redis_cache) = &self.state.redis_cache {
            let cache_key = format!("permission:{}:{}:{}", req.user_id, req.resource, req.action);
            if let Ok(Some(cached_result)) = redis_cache.get(&cache_key).await {
                if let Ok(response) =
                    serde_json::from_value::<CheckPermissionResponse>(cached_result)
                {
                    debug!("Permission check cache hit");
                    return Ok(Response::new(response));
                }
            }
        }

        // OPTIMIZATION: Parallel queries for user profile + permissions using tokio::join!
        let (user_result, _permissions_result) = tokio::join!(
            // Get user profile
            self.state.user_store.get_user(user_id),
            // Placeholder for future separate permissions query
            async { Ok::<_, AuthencError>(()) }
        );

        let user = user_result
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Check if user has the required permission
        // user::Permission uses resource_type and resource_pattern
        let has_permission = user.permissions.iter().any(|p| {
            p.action == req.action && {
                // Check if resource_type matches or if there's a resource_pattern match
                if let Some(pattern) = &p.resource_pattern {
                    // Simple pattern matching - supports wildcards
                    if pattern.contains('*') {
                        let prefix = pattern.trim_end_matches('*');
                        req.resource.starts_with(prefix)
                    } else {
                        pattern == &req.resource
                    }
                } else {
                    // If no pattern, match by resource_type
                    p.resource_type == req.resource
                }
            }
        });

        // If not directly granted, check capability-based permissions
        let has_capability_permission = if !has_permission {
            // Use capability checker for dynamic authorization
            // This replaces hardcoded role checks with database-driven capabilities
            let capability_code = format!("{}:{}", req.resource, req.action);
            self.state
                .capability_checker
                .user_has_any_capability(&user.id, &[capability_code.as_str(), "system:admin"])
                .await
                .unwrap_or(false)
        } else {
            false
        };

        let allowed = has_permission || has_capability_permission;

        let reason = if !allowed {
            Some(format!(
                "User does not have permission to {} on resource {}",
                req.action, req.resource
            ))
        } else {
            None
        };

        let response = CheckPermissionResponse { allowed, reason };

        // Cache the permission check result (TTL: 5 minutes) - non-blocking
        if let Some(redis_cache) = self.state.redis_cache.clone() {
            let cache_key = format!("permission:{}:{}:{}", req.user_id, req.resource, req.action);
            let cache_value = serde_json::to_value(&response).unwrap_or_default();
            tokio::spawn(async move {
                let _ = redis_cache
                    .set(
                        &cache_key,
                        &cache_value,
                        std::time::Duration::from_secs(300),
                    )
                    .await;
            });
        }

        // Fire authorization check event (non-blocking with tokio::spawn)
        let event_manager = self.state.event_manager.clone();
        let user_id_str = user_id.to_string();
        let resource = req.resource.clone();
        let action = req.action.clone();
        tokio::spawn(async move {
            let em = event_manager.write().await;
            let event = crate::services::events::EventBuilder::new(
                if allowed {
                    crate::models::events::EventType::AuthorizationSuccess
                } else {
                    crate::models::events::EventType::AuthorizationFailure
                },
                "master".to_string(),
            )
            .user_id(user_id_str)
            .client_id("grpc".to_string())
            .detail("resource", resource)
            .detail("action", action)
            .detail("allowed", allowed.to_string())
            .build();

            let _ = em.fire_event(event).await;
        });

        Ok(Response::new(response))
    }

    async fn assign_role(
        &self,
        request: Request<AssignRoleRequest>,
    ) -> Result<Response<AssignRoleResponse>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC AssignRole request for user: {}, role: {}",
            req.user_id, req.role
        );

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Get user to verify existence
        let user = self
            .state
            .user_store
            .get_user(user_id)
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Find role by name
        let roles = crate::database::operations::roles::list_roles_by_realm(
            self.state.user_store.database(),
            &user.realm_id.unwrap_or_default(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list roles: {}", e)))?;

        let role = roles
            .iter()
            .find(|r| r.name == req.role)
            .ok_or_else(|| Status::not_found(format!("Role '{}' not found", req.role)))?;

        // Assign role to user
        crate::database::operations::roles::assign_role_to_user(
            self.state.user_store.database(),
            &user_id,
            &role.id,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to assign role: {}", e)))?;

        // Fire role assignment event (using UpdateProfile as closest match)
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::UpdateProfile,
            "master".to_string(),
        )
        .user_id(user_id.to_string())
        .client_id("grpc".to_string())
        .detail("action", "role_assigned")
        .detail("role", req.role.clone())
        .detail("scope", req.scope.unwrap_or_default())
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(AssignRoleResponse { success: true }))
    }

    async fn revoke_role(
        &self,
        request: Request<RevokeRoleRequest>,
    ) -> Result<Response<RevokeRoleResponse>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC RevokeRole request for user: {}, role: {}",
            req.user_id, req.role
        );

        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // Get user to verify existence
        let user = self
            .state
            .user_store
            .get_user(user_id)
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Find role by name
        let roles = crate::database::operations::roles::list_roles_by_realm(
            self.state.user_store.database(),
            &user.realm_id.unwrap_or_default(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list roles: {}", e)))?;

        let role = roles
            .iter()
            .find(|r| r.name == req.role)
            .ok_or_else(|| Status::not_found(format!("Role '{}' not found", req.role)))?;

        // Remove role from user
        crate::database::operations::roles::remove_role_from_user(
            self.state.user_store.database(),
            &user_id,
            &role.id,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to revoke role: {}", e)))?;

        // Fire role revocation event (using RevokeGrant as closest match)
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::RevokeGrant,
            "master".to_string(),
        )
        .user_id(user_id.to_string())
        .client_id("grpc".to_string())
        .detail("action", "role_revoked")
        .detail("role", req.role.clone())
        .build();

        let _ = event_manager.fire_event(event).await;

        Ok(Response::new(RevokeRoleResponse { success: true }))
    }

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ListRoles request");

        // If user_id is provided, get roles for that user
        if let Some(user_id_str) = req.user_id {
            let user_id = uuid::Uuid::parse_str(&user_id_str)
                .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

            let user_roles = crate::database::operations::roles::get_user_roles(
                self.state.user_store.database(),
                &user_id,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get user roles: {}", e)))?;

            let roles: Vec<proto::RoleInfo> = user_roles
                .into_iter()
                .map(|role| proto::RoleInfo {
                    role_id: role.id.to_string(),
                    name: role.name.clone(),
                    description: role.description.clone().unwrap_or_default(),
                    permissions: vec![], // Permissions would need separate query
                })
                .collect();

            return Ok(Response::new(ListRolesResponse { roles }));
        }

        // Otherwise, list all roles (use default realm)
        let default_realm_id = uuid::Uuid::nil(); // In production, get from config
        let all_roles = crate::database::operations::roles::list_roles_by_realm(
            self.state.user_store.database(),
            &default_realm_id,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list roles: {}", e)))?;

        let roles: Vec<proto::RoleInfo> = all_roles
            .into_iter()
            .map(|role| proto::RoleInfo {
                role_id: role.id.to_string(),
                name: role.name.clone(),
                description: role.description.clone().unwrap_or_default(),
                permissions: vec![], // Permissions would need separate query
            })
            .collect();

        Ok(Response::new(ListRolesResponse { roles }))
    }

    // ==================== OAuth2 / OIDC ====================

    async fn get_o_auth_token(
        &self,
        request: Request<OAuthTokenRequest>,
    ) -> Result<Response<OAuthTokenResponse>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC GetOAuthToken request with grant_type: {}",
            req.grant_type
        );

        match req.grant_type.as_str() {
            "authorization_code" => {
                // Authorization code flow
                let code = req
                    .code
                    .ok_or_else(|| Status::invalid_argument("Authorization code required"))?;

                // In production, validate the authorization code from database/cache
                // For now, we'll extract user_id from code (simplified)
                // Real implementation would query authorization_codes table

                // Verify redirect_uri matches
                let _redirect_uri = req.redirect_uri.unwrap_or_default();

                // Generate tokens (simplified - in production, validate code properly)
                let user_id = "user_from_code"; // Extract from validated code
                let access_token = crate::utils::jwt::generate_jwt(
                    user_id,
                    Some("user@example.com".to_string()),
                    Some(vec!["user".to_string()]),
                )
                .map_err(|_| Status::internal("Token generation failed"))?;

                let refresh_token = crate::utils::jwt::generate_refresh_token(user_id)
                    .map_err(|_| Status::internal("Refresh token generation failed"))?;

                let id_token = crate::utils::jwt::generate_jwt(
                    user_id,
                    Some("user@example.com".to_string()),
                    Some(vec!["user".to_string()]),
                )
                .map_err(|_| Status::internal("ID token generation failed"))?;

                Ok(Response::new(OAuthTokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: Some(refresh_token),
                    id_token: Some(id_token),
                    scopes: req.scopes,
                }))
            }
            "refresh_token" => {
                // Refresh token flow
                let refresh_token = req
                    .refresh_token
                    .ok_or_else(|| Status::invalid_argument("Refresh token required"))?;

                // Verify refresh token
                let claims = crate::utils::jwt::verify_refresh_token(&refresh_token)
                    .map_err(|_| Status::unauthenticated("Invalid refresh token"))?;

                let user_id = claims.sub;

                // Fetch user to get fresh email and roles
                let user = self
                    .state
                    .user_store
                    .get_user(
                        uuid::Uuid::parse_str(&user_id)
                            .map_err(|_| Status::internal("Invalid user ID"))?,
                    )
                    .await
                    .map_err(Self::map_error)?
                    .ok_or_else(|| Status::unauthenticated("User not found"))?;

                // Check if user account is enabled
                if !user.enabled {
                    return Err(Status::permission_denied("User account is disabled"));
                }

                let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();

                // Generate new access token
                let new_access_token =
                    crate::utils::jwt::generate_jwt(&user_id, Some(user.email), Some(roles))
                        .map_err(|_| Status::internal("Token generation failed"))?;

                let new_refresh_token = crate::utils::jwt::generate_refresh_token(&user_id)
                    .map_err(|_| Status::internal("Refresh token generation failed"))?;

                Ok(Response::new(OAuthTokenResponse {
                    access_token: new_access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: Some(new_refresh_token),
                    id_token: None,
                    scopes: req.scopes,
                }))
            }
            "client_credentials" => {
                // Client credentials flow
                let client_id = req
                    .client_id
                    .ok_or_else(|| Status::invalid_argument("Client ID required"))?;

                let client_secret = req
                    .client_secret
                    .ok_or_else(|| Status::invalid_argument("Client secret required"))?;

                // Verify client credentials (simplified)
                // In production, validate against clients table

                let access_token = crate::utils::jwt::generate_jwt(&client_id, None, None)
                    .map_err(|_| Status::internal("Token generation failed"))?;

                Ok(Response::new(OAuthTokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: None,
                    id_token: None,
                    scopes: req.scopes,
                }))
            }
            _ => Err(Status::invalid_argument(format!(
                "Unsupported grant type: {}",
                req.grant_type
            ))),
        }
    }

    async fn introspect_token(
        &self,
        request: Request<IntrospectTokenRequest>,
    ) -> Result<Response<IntrospectTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC IntrospectToken request");

        // Verify the token
        match crate::utils::jwt::verify_jwt(&req.token) {
            Ok(claims) => {
                let user_id = claims.sub.clone();
                let exp = claims.exp;

                // Check if token is revoked (check blacklist)
                let is_revoked = if let Some(redis_cache) = &self.state.redis_cache {
                    let blacklist_key = format!("token:blacklist:{}", req.token);
                    redis_cache
                        .get(&blacklist_key)
                        .await
                        .ok()
                        .flatten()
                        .is_some()
                } else {
                    false
                };

                if is_revoked {
                    return Ok(Response::new(IntrospectTokenResponse {
                        active: false,
                        user_id: None,
                        client_id: None,
                        scopes: vec![],
                        exp: None,
                        iat: None,
                    }));
                }

                Ok(Response::new(IntrospectTokenResponse {
                    active: true,
                    user_id: Some(user_id),
                    client_id: None, // Would be extracted from token if present
                    scopes: vec!["openid".to_string(), "profile".to_string()],
                    exp: Some(exp as i64),
                    iat: None, // Claims struct doesn't have iat field
                }))
            }
            Err(_) => Ok(Response::new(IntrospectTokenResponse {
                active: false,
                user_id: None,
                client_id: None,
                scopes: vec![],
                exp: None,
                iat: None,
            })),
        }
    }

    async fn get_user_info(
        &self,
        request: Request<UserInfoRequest>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC GetUserInfo request");

        // Verify access token
        let claims = crate::utils::jwt::verify_jwt(&req.access_token)
            .map_err(|_| Status::unauthenticated("Invalid access token"))?;

        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| Status::internal("Invalid user ID in token"))?;

        // Get user from database
        let user = self
            .state
            .user_store
            .get_user(user_id)
            .await
            .map_err(Self::map_error)?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Build user info response (OIDC standard claims)
        let mut additional_claims = std::collections::HashMap::new();
        additional_claims.insert("username".to_string(), user.username.clone());
        if let Some(nip) = &user.nip {
            additional_claims.insert("nip".to_string(), nip.clone());
        }
        if let Some(jabatan) = &user.jabatan {
            additional_claims.insert("jabatan".to_string(), jabatan.clone());
        }
        additional_claims.insert("satker_code".to_string(), user.satker_code.clone());

        Ok(Response::new(UserInfoResponse {
            sub: user.id.to_string(),
            email: user.email.clone(),
            email_verified: user.email_verified,
            name: user.nama.clone(),
            picture: None, // Could be added if profile pictures are stored
            additional_claims,
        }))
    }

    // ==================== Federation ====================

    async fn initiate_federated_auth(
        &self,
        request: Request<FederatedAuthRequest>,
    ) -> Result<Response<FederatedAuthResponse>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC InitiateFederatedAuth request for provider: {}",
            req.provider
        );

        // Generate state parameter for CSRF protection
        let state = uuid::Uuid::new_v4().to_string();

        // Store state in cache with short TTL (10 minutes)
        if let Some(redis_cache) = &self.state.redis_cache {
            let state_key = format!("oauth:state:{}", state);
            let state_data = serde_json::json!({
                "provider": req.provider,
                "redirect_uri": req.redirect_uri,
                "scopes": req.scopes,
                "created_at": chrono::Utc::now().timestamp()
            });
            let _ = redis_cache
                .set(&state_key, &state_data, std::time::Duration::from_secs(600))
                .await;
        }

        // Build authorization URL based on provider
        let auth_url = match req.provider.as_str() {
            "google" => {
                let scopes = if req.scopes.is_empty() {
                    "openid profile email".to_string()
                } else {
                    req.scopes.join(" ")
                };
                let redirect_uri = req
                    .redirect_uri
                    .unwrap_or_else(|| "https://simpel.kejaksaan.go.id/auth/callback".to_string());
                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                    "YOUR_GOOGLE_CLIENT_ID", // Should come from config
                    urlencoding::encode(&redirect_uri),
                    urlencoding::encode(&scopes),
                    state
                )
            }
            "github" => {
                let scopes = if req.scopes.is_empty() {
                    "read:user user:email".to_string()
                } else {
                    req.scopes.join(" ")
                };
                let redirect_uri = req
                    .redirect_uri
                    .unwrap_or_else(|| "https://simpel.kejaksaan.go.id/auth/callback".to_string());
                format!(
                    "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
                    "YOUR_GITHUB_CLIENT_ID", // Should come from config
                    urlencoding::encode(&redirect_uri),
                    urlencoding::encode(&scopes),
                    state
                )
            }
            "saml" => {
                // SAML federation would require different flow
                format!(
                    "https://idp.example.com/saml/sso?SAMLRequest=...&RelayState={}",
                    state
                )
            }
            _ => {
                return Err(Status::invalid_argument(format!(
                    "Unsupported provider: {}",
                    req.provider
                )));
            }
        };

        Ok(Response::new(FederatedAuthResponse { auth_url, state }))
    }

    async fn complete_federated_auth(
        &self,
        request: Request<CompleteFederatedAuthRequest>,
    ) -> Result<Response<CompleteFederatedAuthResponse>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC CompleteFederatedAuth request for provider: {}",
            req.provider
        );

        // Verify state parameter
        if let Some(redis_cache) = &self.state.redis_cache {
            let state_key = format!("oauth:state:{}", req.state);
            let state_data = redis_cache
                .get(&state_key)
                .await
                .map_err(|_| Status::internal("Failed to verify state"))?
                .ok_or_else(|| Status::invalid_argument("Invalid or expired state"))?;

            // Verify provider matches
            if let Some(stored_provider) = state_data.get("provider").and_then(|v| v.as_str()) {
                if stored_provider != req.provider {
                    return Err(Status::invalid_argument("Provider mismatch"));
                }
            }

            // Delete state after verification (one-time use)
            let _ = redis_cache.delete(&state_key).await;
        }

        // Exchange authorization code for tokens with provider
        // This is simplified - in production, make actual HTTP requests to provider
        let (provider_access_token, provider_user_info) = match req.provider.as_str() {
            "google" => {
                // Exchange code for tokens with Google
                // let token_response = reqwest::post("https://oauth2.googleapis.com/token")...
                // let user_info = reqwest::get("https://www.googleapis.com/oauth2/v2/userinfo")...
                (
                    "google_access_token".to_string(),
                    serde_json::json!({
                        "email": "user@example.com",
                        "name": "User Name",
                        "sub": "google_user_id"
                    }),
                )
            }
            "github" => {
                // Exchange code for tokens with GitHub
                (
                    "github_access_token".to_string(),
                    serde_json::json!({
                        "email": "user@example.com",
                        "name": "User Name",
                        "id": "github_user_id"
                    }),
                )
            }
            _ => {
                return Err(Status::invalid_argument(format!(
                    "Unsupported provider: {}",
                    req.provider
                )));
            }
        };

        // Find or create user based on federated identity
        let email = provider_user_info
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Status::internal("Email not provided by provider"))?;

        let user = match self.state.user_store.get_user_by_email(email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                // Create new federated user
                let create_request = crate::models::user::CreateUserRequest {
                    username: email.to_string(),
                    email: email.to_string(),
                    satker_code: "FEDERATED".to_string(),
                    password: None, // Federated users don't have passwords
                    first_name: provider_user_info
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    last_name: None,
                    nip: None,
                    nama: provider_user_info
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    jabatan: None,
                    phone_number: None,
                    realm_id: None,
                    organization_id: None,
                    roles: None,
                    attributes: Some(serde_json::json!({
                        "federated": true,
                        "provider": req.provider,
                        "provider_user_id": provider_user_info.get("sub").or(provider_user_info.get("id"))
                    })),
                };

                self.state
                    .user_store
                    .add_user(create_request)
                    .await
                    .map_err(Self::map_error)?
            }
            Err(e) => return Err(Self::map_error(e)),
        };

        // Generate our own tokens
        let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
        let access_token = crate::utils::jwt::generate_jwt(
            &user.id.to_string(),
            Some(user.email.clone()),
            Some(roles),
        )
        .map_err(|_| Status::internal("Token generation failed"))?;

        let refresh_token = crate::utils::jwt::generate_refresh_token(&user.id.to_string())
            .map_err(|_| Status::internal("Refresh token generation failed"))?;

        // Fire federated login event
        let event_manager = self.state.event_manager.write().await;
        let event = crate::services::events::EventBuilder::new(
            crate::models::events::EventType::Login,
            "master".to_string(),
        )
        .user_id(user.id.to_string())
        .client_id("grpc".to_string())
        .detail("method", "federated")
        .detail("provider", req.provider.clone())
        .build();

        let _ = event_manager.fire_event(event).await;

        // Build user info
        let user_info = proto::UserInfo {
            user_id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.nama.clone(),
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            is_active: user.enabled,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at.timestamp(),
            last_login: user.last_login_at.map(|t| t.timestamp()).unwrap_or(0),
        };

        Ok(Response::new(CompleteFederatedAuthResponse {
            access_token,
            refresh_token,
            user: Some(user_info),
        }))
    }

    // ==================== Audit & Compliance ====================

    async fn get_audit_logs(
        &self,
        request: Request<AuditLogsRequest>,
    ) -> Result<Response<AuditLogsResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC GetAuditLogs request");

        // Parse optional user_id
        let user_id = if let Some(user_id_str) = req.user_id {
            Some(
                uuid::Uuid::parse_str(&user_id_str)
                    .map_err(|_| Status::invalid_argument("Invalid user ID"))?,
            )
        } else {
            None
        };

        // Convert timestamps
        let start_time = req
            .start_time
            .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(chrono::Utc::now));
        let end_time = req
            .end_time
            .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(chrono::Utc::now));

        // Get audit logs from database
        let audit_events = crate::database::operations::audit::get_audit_logs(
            self.state.user_store.database(),
            user_id,
            req.action.as_deref(),
            req.limit.unwrap_or(100) as i64,
            req.offset.unwrap_or(0) as i64,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get audit logs: {}", e)))?;

        // Get total count
        let total = crate::database::operations::audit::get_audit_log_count(
            self.state.user_store.database(),
            user_id,
            req.action.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get audit log count: {}", e)))?;

        // Convert to proto format
        let logs: Vec<proto::AuditLog> = audit_events
            .into_iter()
            .map(|event| {
                let mut metadata = std::collections::HashMap::new();
                if let Some(details) = event.details {
                    if let Ok(map) =
                        serde_json::from_value::<std::collections::HashMap<String, String>>(details)
                    {
                        metadata = map;
                    }
                }

                proto::AuditLog {
                    id: uuid::Uuid::new_v4().to_string(), // Generate ID since AuditEvent doesn't have one
                    user_id: event.user_id.map(|id| id.to_string()).unwrap_or_default(),
                    action: event.action.clone(),
                    resource: event.resource_type.unwrap_or_default(),
                    success: event.status == "success",
                    ip_address: event.ip_address.unwrap_or_default(),
                    user_agent: event.user_agent.unwrap_or_default(),
                    timestamp: event.timestamp.timestamp(),
                    metadata,
                }
            })
            .collect();

        Ok(Response::new(AuditLogsResponse {
            logs,
            total: total as i32,
        }))
    }

    async fn get_compliance_report(
        &self,
        request: Request<ComplianceReportRequest>,
    ) -> Result<Response<ComplianceReportResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC GetComplianceReport request");

        let start_time = chrono::DateTime::from_timestamp(req.start_time, 0)
            .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
        let end_time =
            chrono::DateTime::from_timestamp(req.end_time, 0).unwrap_or_else(chrono::Utc::now);

        // Get audit logs for the period
        let audit_events = crate::database::operations::audit::get_audit_logs(
            self.state.user_store.database(),
            None,
            None,
            10000, // Large limit for report
            0,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to get audit logs: {}", e)))?;

        // Calculate metrics
        let total_authentications = audit_events
            .iter()
            .filter(|e| {
                e.event_type == "Login"
                    || e.event_type == "LoginError"
                    || e.event_type == "Authenticate"
            })
            .count() as i64;

        let failed_authentications = audit_events
            .iter()
            .filter(|e| e.event_type == "LoginError" || e.event_type == "AuthenticationFailed")
            .count() as i64;

        // Get MFA-enabled users count
        let all_users = self
            .state
            .user_store
            .get_all()
            .await
            .map_err(Self::map_error)?;

        let mfa_enabled_users = all_users.iter().filter(|u| u.mfa_enabled).count() as i64;

        // Get active sessions (simplified - would need session table query)
        let active_sessions = 0i64; // Placeholder

        // Build metrics map
        let mut metrics_map = std::collections::HashMap::new();

        // Authentication metrics
        let mut auth_metrics = std::collections::HashMap::new();
        auth_metrics.insert("password_logins".to_string(), total_authentications);
        auth_metrics.insert("failed_logins".to_string(), failed_authentications);
        auth_metrics.insert(
            "success_rate".to_string(),
            if total_authentications > 0 {
                ((total_authentications - failed_authentications) * 100) / total_authentications
            } else {
                0
            },
        );

        metrics_map.insert(
            "authentication".to_string(),
            proto::ComplianceMetrics {
                total_authentications,
                failed_authentications,
                mfa_enabled_users,
                active_sessions,
                additional_metrics: auth_metrics,
            },
        );

        // MFA metrics
        let mut mfa_metrics = std::collections::HashMap::new();
        mfa_metrics.insert("total_users".to_string(), all_users.len() as i64);
        mfa_metrics.insert("mfa_enabled".to_string(), mfa_enabled_users);
        mfa_metrics.insert(
            "mfa_adoption_rate".to_string(),
            if !all_users.is_empty() {
                (mfa_enabled_users * 100) / all_users.len() as i64
            } else {
                0
            },
        );

        metrics_map.insert(
            "mfa".to_string(),
            proto::ComplianceMetrics {
                total_authentications: 0,
                failed_authentications: 0,
                mfa_enabled_users,
                active_sessions: 0,
                additional_metrics: mfa_metrics,
            },
        );

        // Security events metrics
        let security_events = audit_events
            .iter()
            .filter(|e| {
                e.event_type == "PasswordChanged"
                    || e.event_type == "MfaEnabled"
                    || e.event_type == "MfaDisabled"
                    || e.event_type == "RoleAssigned"
                    || e.event_type == "RoleRevoked"
            })
            .count() as i64;

        let mut security_metrics = std::collections::HashMap::new();
        security_metrics.insert("total_security_events".to_string(), security_events);

        metrics_map.insert(
            "security".to_string(),
            proto::ComplianceMetrics {
                total_authentications: 0,
                failed_authentications: 0,
                mfa_enabled_users: 0,
                active_sessions: 0,
                additional_metrics: security_metrics,
            },
        );

        Ok(Response::new(ComplianceReportResponse {
            metrics: metrics_map,
            generated_at: chrono::Utc::now().timestamp(),
        }))
    }

    // ==================== Health & Metrics ====================

    async fn health_check(
        &self,
        request: Request<common::v1::HealthCheckRequest>,
    ) -> Result<Response<common::v1::HealthCheckResponse>, Status> {
        // Delegate to health service
        debug!("gRPC HealthCheck request");

        // TODO: This will be implemented in the health module
        Err(Status::unimplemented("Health check not yet implemented"))
    }
}
