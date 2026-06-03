//! gRPC service implementation

use crate::proto::authenc::v1::authenc_service_server::AuthencService;
use crate::proto::authenc::v1::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, error};

use authenc_core::services::{
    AuditService, AuthenticationServiceImpl, OAuth2ServiceImpl, RealmManagementServiceImpl,
    RoleManagementServiceImpl, UserManagementServiceImpl,
};
use authenc_crypto::jwt::JwtService;
use authenc_federation::service::{
    CompleteFederatedAuthRequest as DomainCompleteFederatedAuthRequest,
    FederatedAuthRequest as DomainFederatedAuthRequest, FederationService,
};
use authenc_storage::PostgresRevocationStore;
use authenc_types::traits::{
    AuthenticationService as AuthenticationServiceTrait, OAuth2Service as OAuth2ServiceTrait,
};
use authenc_types::{
    AuthResult, CreateUserRequest as DomainCreateUserRequest, Credentials, RealmId, RoleId,
    SessionId, UpdateUserRequest as DomainUpdateUserRequest, UserId,
};
use uuid::Uuid;

/// Build an `invalid` gRPC validation response carrying a generic error message
/// and no identity claims. Shared by the verification-failed and revoked paths.
fn invalid_validation_response(error: &str) -> ValidateTokenResponse {
    ValidateTokenResponse {
        valid: false,
        user_id: None,
        scopes: vec![],
        expires_at: None,
        error: Some(error.to_string()),
        username: None,
        name: None,
        nip: None,
        jabatan: None,
        satker_code: None,
        realm_roles: vec![],
    }
}

/// AuthencGrpcService implementation
///
/// This service provides gRPC endpoints for service-to-service communication.
/// It wraps the core business logic services and exposes them via gRPC.
#[derive(Clone)]
pub struct AuthencGrpcService {
    /// Authentication service for login/logout operations
    auth_service: Arc<AuthenticationServiceImpl>,
    /// User management service for CRUD operations
    user_service: Arc<UserManagementServiceImpl>,
    /// OAuth2 service for token operations
    oauth2_service: Arc<OAuth2ServiceImpl>,
    /// Realm management service
    #[allow(dead_code)]
    realm_service: Arc<RealmManagementServiceImpl>,
    /// Role management service for RBAC/ABAC
    role_service: Arc<RoleManagementServiceImpl>,
    /// JWT service for token generation and validation
    jwt_service: Arc<JwtService>,
    /// MFA service facade (optional, for MFA operations)
    mfa_service: Option<Arc<dyn MfaServiceFacade>>,
    /// Federation service for SSO and external IdP integration
    federation_service: Arc<FederationService>,
    /// Audit service for audit logs and compliance reporting
    audit_service: Arc<AuditService>,
    /// Token revocation list (F2H) — consulted by `validate_token` after JWT
    /// verification so revoked tokens are rejected before their `exp`. Shared
    /// with the REST surface so both channels enforce the same revocations.
    revocation_store: Arc<PostgresRevocationStore>,
}

/// Facade trait for MFA operations
///
/// This trait abstracts MFA operations to avoid generic complexity in the gRPC service.
/// Implementations should integrate with authenc-mfa crate services.
#[async_trait::async_trait]
pub trait MfaServiceFacade: Send + Sync {
    /// Setup TOTP for a user
    async fn setup_totp(
        &self,
        user_id: UserId,
        username: &str,
    ) -> authenc_types::Result<MfaSetupResponse>;

    /// Verify TOTP code
    async fn verify_totp(&self, user_id: UserId, code: &str) -> authenc_types::Result<bool>;

    /// Disable TOTP for a user
    async fn disable_totp(&self, user_id: UserId) -> authenc_types::Result<()>;
}

/// MFA setup response
#[derive(Debug, Clone)]
pub struct MfaSetupResponse {
    /// Base32-encoded TOTP secret
    pub secret: String,
    /// QR code as SVG or data URL
    pub qr_code: String,
    /// Backup codes for recovery
    pub backup_codes: Vec<String>,
}

impl AuthencGrpcService {
    /// Create new gRPC service instance
    ///
    /// # Arguments
    ///
    /// * `auth_service` - Authentication service implementation
    /// * `user_service` - User management service implementation
    /// * `oauth2_service` - OAuth2 service implementation
    /// * `realm_service` - Realm management service implementation
    /// * `role_service` - Role management service implementation
    /// * `jwt_service` - JWT service for token generation
    /// * `federation_service` - Federation service for SSO and external IdP integration
    /// * `audit_service` - Audit service for audit logs and compliance reporting
    /// * `revocation_store` - Token revocation list checked during validate_token
    pub fn new(
        auth_service: Arc<AuthenticationServiceImpl>,
        user_service: Arc<UserManagementServiceImpl>,
        oauth2_service: Arc<OAuth2ServiceImpl>,
        realm_service: Arc<RealmManagementServiceImpl>,
        role_service: Arc<RoleManagementServiceImpl>,
        jwt_service: Arc<JwtService>,
        federation_service: Arc<FederationService>,
        audit_service: Arc<AuditService>,
        revocation_store: Arc<PostgresRevocationStore>,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            oauth2_service,
            realm_service,
            role_service,
            jwt_service,
            mfa_service: None,
            federation_service,
            audit_service,
            revocation_store,
        }
    }

    /// Set MFA service (optional)
    pub fn with_mfa_service(mut self, mfa_service: Arc<dyn MfaServiceFacade>) -> Self {
        self.mfa_service = Some(mfa_service);
        self
    }

    /// Helper method to get role ID by name
    async fn get_role_id_by_name(&self, role_name: &str) -> authenc_types::Result<RoleId> {
        // Query database for role by name
        let _query = "SELECT id FROM roles WHERE name = $1 LIMIT 1";

        // Access database through role_service
        // For now, we'll return an error indicating the role wasn't found
        // In a real implementation, this would query the database
        Err(authenc_types::error::AuthencError::NotFound(format!(
            "Role '{}' not found. Please use role UUID instead.",
            role_name
        )))
    }

    /// Convert AuthencError to gRPC Status
    fn error_to_status(error: authenc_types::error::AuthencError) -> Status {
        use authenc_types::error::AuthencError;

        match error {
            AuthencError::UserNotFound(msg) => Status::not_found(msg),
            AuthencError::NotFound(msg) => Status::not_found(msg),
            AuthencError::UserDisabled => Status::permission_denied("User account is disabled"),
            AuthencError::InvalidCredentials => Status::unauthenticated("Invalid credentials"),
            AuthencError::TokenExpired => Status::unauthenticated("Token expired"),
            AuthencError::InvalidToken(msg) => {
                Status::unauthenticated(format!("Invalid token: {}", msg))
            }
            AuthencError::SessionNotFound(_) => Status::not_found("Session not found"),
            AuthencError::AccountLocked { .. } => Status::permission_denied("Account locked"),
            AuthencError::ValidationError(msg) => Status::invalid_argument(msg),
            AuthencError::UsernameAlreadyExists(username) => {
                Status::already_exists(format!("Username {} already exists", username))
            }
            AuthencError::EmailAlreadyExists(email) => {
                Status::already_exists(format!("Email {} already exists", email))
            }
            AuthencError::Unauthorized(msg) => Status::permission_denied(msg),
            AuthencError::AuthorizationFailed(msg) => Status::permission_denied(msg),
            _ => Status::internal(error.to_string()),
        }
    }
}

#[tonic::async_trait]
impl AuthencService for AuthencGrpcService {
    /// Authenticate user with username/password
    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<AuthenticateResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC Authenticate request for user: {}", req.username);

        // For now, use default realm (master)
        let realm_id = RealmId::new(); // TODO: Parse from request metadata

        // Create credentials
        let credentials = Credentials {
            username: req.username.clone(),
            password: req.password,
        };

        // Authenticate
        let auth_result = self
            .auth_service
            .authenticate(credentials, realm_id)
            .await
            .map_err(Self::error_to_status)?;

        // Convert result to gRPC response
        match auth_result {
            AuthResult::Success {
                user_id,
                session_id,
            } => {
                // Get user info
                let user = self
                    .user_service
                    .get_user(user_id)
                    .await
                    .map_err(Self::error_to_status)?;

                // Generate JWT access token
                let access_token = self
                    .jwt_service
                    .generate_access_token(
                        &user_id.to_string(),
                        Some("master".to_string()), // TODO: Use actual realm
                        Some("openid profile".to_string()),
                        Some(session_id.to_string()),
                    )
                    .map_err(|e| {
                        Status::internal(format!("Failed to generate access token: {}", e))
                    })?;

                // Generate refresh token
                let refresh_token = self
                    .jwt_service
                    .generate_refresh_token(&user_id.to_string(), &session_id.to_string())
                    .map_err(|e| {
                        Status::internal(format!("Failed to generate refresh token: {}", e))
                    })?;

                Ok(Response::new(AuthenticateResponse {
                    access_token,
                    refresh_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 900, // 15 minutes (from JWT service config)
                    user: Some(UserInfo {
                        user_id: user.id.to_string(),
                        username: user.username,
                        email: user.email,
                        full_name: None,
                        roles: vec![],
                        is_active: user.enabled,
                        mfa_enabled: user.mfa_enabled,
                        created_at: user.created_at.timestamp(),
                        last_login: 0, // TODO: Track last login
                    }),
                    scopes: vec!["openid".to_string(), "profile".to_string()],
                }))
            }
            AuthResult::MfaRequired { .. } => {
                Err(Status::failed_precondition("MFA verification required"))
            }
            AuthResult::Failed { reason } => Err(Status::unauthenticated(format!(
                "Authentication failed: {:?}",
                reason
            ))),
        }
    }
    /// Refresh access token
    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC RefreshToken request");

        // Verify refresh token
        let claims = self
            .jwt_service
            .verify_token(&req.refresh_token)
            .map_err(|e| Status::unauthenticated(format!("Invalid refresh token: {}", e)))?;

        // Verify it's a refresh token (has refresh_token scope)
        if claims.scope != Some("refresh_token".to_string()) {
            return Err(Status::invalid_argument("Token is not a refresh token"));
        }

        // Extract user_id and session_id from claims
        let user_id = claims.sub;
        let session_id = claims
            .sid
            .ok_or_else(|| Status::invalid_argument("Refresh token missing session ID"))?;

        // Generate new access token
        let access_token = self
            .jwt_service
            .generate_access_token(
                &user_id,
                claims.realm,
                Some("openid profile".to_string()),
                Some(session_id.clone()),
            )
            .map_err(|e| Status::internal(format!("Failed to generate access token: {}", e)))?;

        // Generate new refresh token (token rotation)
        let new_refresh_token = self
            .jwt_service
            .generate_refresh_token(&user_id, &session_id)
            .map_err(|e| Status::internal(format!("Failed to generate refresh token: {}", e)))?;

        Ok(Response::new(RefreshTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: 900, // 15 minutes
        }))
    }

    /// Validate JWT token
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ValidateToken request");

        // Verify JWT token
        match self.jwt_service.verify_token(&req.token) {
            Ok(claims) => {
                // Revocation check — a signature-valid, unexpired token may
                // still have been revoked (logout, role change, deactivation).
                // Fail closed: a revoked token, or a store error, is reported as
                // invalid rather than risking an authenticated verdict.
                match self
                    .revocation_store
                    .is_revoked(&claims.jti, claims.sid.as_deref(), &claims.sub, claims.iat)
                    .await
                {
                    Ok(false) => {}
                    Ok(true) => {
                        return Ok(Response::new(invalid_validation_response(
                            "Token has been revoked",
                        )));
                    }
                    Err(e) => {
                        error!("Revocation check failed: {}", e);
                        return Ok(Response::new(invalid_validation_response(
                            "Token validation failed",
                        )));
                    }
                }

                // Extract scopes from claims
                let scopes = claims
                    .scope
                    .as_ref()
                    .map(|s| s.split_whitespace().map(String::from).collect())
                    .unwrap_or_else(Vec::new);

                // Pull first-class identity claims out of the flattened
                // `custom` map so callers no longer have to parse prefixed
                // scope entries (`username:...`, `satker:...`, etc.).
                let pick = |key: &str| -> Option<String> {
                    claims
                        .custom
                        .get(key)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                };
                let username = pick("username");
                let name = pick("name").or_else(|| pick("nama"));
                let nip = pick("nip");
                let jabatan = pick("jabatan");
                let satker_code = pick("satker_code").or_else(|| pick("satker"));
                let realm_roles = claims
                    .custom
                    .get("realm_access")
                    .and_then(|v| v.get("roles"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|r| r.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                Ok(Response::new(ValidateTokenResponse {
                    valid: true,
                    user_id: Some(claims.sub),
                    scopes,
                    expires_at: Some(claims.exp),
                    error: None,
                    username,
                    name,
                    nip,
                    jabatan,
                    satker_code,
                    realm_roles,
                }))
            }
            Err(e) => {
                debug!("Token validation failed: {}", e);
                Ok(Response::new(invalid_validation_response(
                    "Token validation failed",
                )))
            }
        }
    }

    /// Revoke token
    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC RevokeToken request");

        // Treat token as session_id
        let session_id = SessionId::from_string(&req.token)
            .map_err(|e| Status::invalid_argument(format!("Invalid token format: {}", e)))?;

        // Record a session-wide revocation so every access token carrying this
        // sid is rejected by validate_token before its `exp` — not just removed
        // from the session store. Bounded at the refresh lifetime.
        let expires_at = chrono::Utc::now() + self.jwt_service.refresh_token_ttl();
        if let Err(e) = self
            .revocation_store
            .revoke_session(&req.token, expires_at, Some("grpc_revoke"))
            .await
        {
            error!("Failed to record session revocation: {}", e);
        }

        // Logout (invalidate session)
        self.auth_service
            .logout(session_id)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(RevokeTokenResponse { success: true }))
    }

    /// Create new user
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC CreateUser request for username: {}", req.username);

        // Use default realm for now
        let realm_id = RealmId::new(); // TODO: Parse from request

        // Create user request using domain type
        let create_request = DomainCreateUserRequest {
            username: req.username.clone(),
            email: req.email,
            satker_code: String::new(),
            password: Some(req.password),
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            realm_id: Some(realm_id.into()),
            organization_id: None,
            roles: None,
            attributes: None,
            enabled: None,
        };

        // Create user
        let user = self
            .user_service
            .create_user(create_request)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(CreateUserResponse {
            user_id: user.id.to_string(),
            username: user.username,
            created_at: user.created_at.timestamp(),
        }))
    }

    /// Get user by ID
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC GetUser request for user_id: {}", req.user_id);

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Get user
        let user = self
            .user_service
            .get_user(user_id)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(GetUserResponse {
            user: Some(UserInfo {
                user_id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: None,
                roles: vec![],
                is_active: user.enabled,
                mfa_enabled: user.mfa_enabled,
                created_at: user.created_at.timestamp(),
                last_login: 0, // TODO: Track last login
            }),
        }))
    }

    /// Update user
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC UpdateUser request for user_id: {}", req.user_id);

        // Parse user_id from the request message
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Create update request using domain type - map proto fields to domain fields
        let update_request = DomainUpdateUserRequest {
            username: None,
            email: req.email,
            satker_code: None,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            phone_number: None,
            password: None, // Password updates should go through separate endpoint
            enabled: req.is_active, // Map is_active to enabled
            email_verified: None,
            phone_verified: None,
            require_password_change: None,
            mfa_enabled: None,
            attributes: None,
        };

        // Update user
        let user = self
            .user_service
            .update_user(user_id, update_request)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(UpdateUserResponse {
            user: Some(UserInfo {
                user_id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: None,
                roles: vec![],
                is_active: user.enabled,
                mfa_enabled: user.mfa_enabled,
                created_at: user.created_at.timestamp(),
                last_login: 0,
            }),
            updated_at: user.updated_at.timestamp(),
        }))
    }

    /// Delete user
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC DeleteUser request for user_id: {}", req.user_id);

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Delete user (soft delete)
        self.user_service
            .delete_user(user_id)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(DeleteUserResponse { success: true }))
    }

    /// List users
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ListUsers request");

        // Use default realm
        let realm_id = RealmId::new(); // TODO: Parse from request

        let offset = req.offset.unwrap_or(0) as usize;
        let limit = req.limit.unwrap_or(20) as usize;

        // List users
        let users = self
            .user_service
            .list_users(realm_id, offset, limit)
            .await
            .map_err(Self::error_to_status)?;

        // Convert to proto users
        let proto_users: Vec<UserInfo> = users
            .into_iter()
            .map(|user| UserInfo {
                user_id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: None,
                roles: vec![],
                is_active: user.enabled,
                mfa_enabled: user.mfa_enabled,
                created_at: user.created_at.timestamp(),
                last_login: 0,
            })
            .collect();

        Ok(Response::new(ListUsersResponse {
            users: proto_users,
            total: 0, // TODO: Implement total count
        }))
    }

    // MFA methods
    async fn enable_mfa(
        &self,
        request: Request<EnableMfaRequest>,
    ) -> Result<Response<EnableMfaResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC EnableMFA request for user_id: {}", req.user_id);

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Check if MFA service is available
        let mfa_service = self
            .mfa_service
            .as_ref()
            .ok_or_else(|| Status::unimplemented("MFA service not configured"))?;

        // Get user to retrieve username
        let user = self
            .user_service
            .get_user(user_id)
            .await
            .map_err(Self::error_to_status)?;

        // Setup TOTP and generate backup codes
        let mfa_setup = mfa_service
            .setup_totp(user_id, &user.username)
            .await
            .map_err(Self::error_to_status)?;

        // Enable MFA flag on user
        self.user_service
            .enable_mfa(user_id)
            .await
            .map_err(Self::error_to_status)?;

        debug!("MFA enabled successfully for user_id: {}", req.user_id);

        Ok(Response::new(EnableMfaResponse {
            secret: mfa_setup.secret,
            qr_code_url: mfa_setup.qr_code,
            backup_codes: mfa_setup.backup_codes,
        }))
    }
    async fn verify_mfa(
        &self,
        request: Request<VerifyMfaRequest>,
    ) -> Result<Response<VerifyMfaResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC VerifyMFA request for user_id: {}", req.user_id);

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Verify MFA
        let auth_result = self
            .auth_service
            .verify_mfa(user_id, req.code)
            .await
            .map_err(Self::error_to_status)?;

        match auth_result {
            AuthResult::Success { .. } => Ok(Response::new(VerifyMfaResponse { valid: true })),
            _ => Ok(Response::new(VerifyMfaResponse { valid: false })),
        }
    }

    async fn disable_mfa(
        &self,
        request: Request<DisableMfaRequest>,
    ) -> Result<Response<DisableMfaResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC DisableMFA request for user_id: {}", req.user_id);

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Ensure MFA service is configured
        let mfa_service = self.mfa_service.as_ref().ok_or_else(|| {
            error!("MFA service not configured");
            Status::unimplemented("MFA service not configured")
        })?;

        // Disable MFA
        self.user_service
            .disable_mfa(user_id)
            .await
            .map_err(Self::error_to_status)?;

        // Clean up TOTP secret
        mfa_service
            .disable_totp(user_id)
            .await
            .map_err(Self::error_to_status)?;

        Ok(Response::new(DisableMfaResponse { success: true }))
    }

    // Authorization methods
    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC CheckPermission request for user {} on resource {} with action {}",
            req.user_id, req.resource, req.action
        );

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Check permission
        let allowed = self
            .role_service
            .check_permission(user_id, &req.resource, &req.action, &req.context)
            .await
            .map_err(Self::error_to_status)?;

        let reason = if allowed {
            None
        } else {
            Some(format!(
                "User {} does not have permission to {} on {}",
                req.user_id, req.action, req.resource
            ))
        };

        Ok(Response::new(CheckPermissionResponse { allowed, reason }))
    }

    async fn assign_role(
        &self,
        request: Request<AssignRoleRequest>,
    ) -> Result<Response<AssignRoleResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC AssignRole request: assigning role {} to user {}",
            req.role, req.user_id
        );

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Parse role as role_id (UUID) or look up by name
        let role_id = if let Ok(uuid) = Uuid::parse_str(&req.role) {
            RoleId::from_uuid(uuid)
        } else {
            // Look up role by name
            self.get_role_id_by_name(&req.role)
                .await
                .map_err(Self::error_to_status)?
        };

        // Assign role
        self.role_service
            .assign_role(user_id, role_id)
            .await
            .map_err(Self::error_to_status)?;

        debug!(
            "Role {} assigned to user {} successfully",
            req.role, req.user_id
        );
        Ok(Response::new(AssignRoleResponse { success: true }))
    }

    async fn revoke_role(
        &self,
        request: Request<RevokeRoleRequest>,
    ) -> Result<Response<RevokeRoleResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC RevokeRole request: revoking role {} from user {}",
            req.role, req.user_id
        );

        // Parse user_id
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Parse role as role_id (UUID) or look up by name
        let role_id = if let Ok(uuid) = Uuid::parse_str(&req.role) {
            RoleId::from_uuid(uuid)
        } else {
            // Look up role by name
            self.get_role_id_by_name(&req.role)
                .await
                .map_err(Self::error_to_status)?
        };

        // Revoke role
        self.role_service
            .revoke_role(user_id, role_id)
            .await
            .map_err(Self::error_to_status)?;

        debug!(
            "Role {} revoked from user {} successfully",
            req.role, req.user_id
        );
        Ok(Response::new(RevokeRoleResponse { success: true }))
    }

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC ListRoles request for user: {:?}", req.user_id);

        // Parse user_id if provided
        let user_id = if let Some(uid_str) = req.user_id {
            Some(
                UserId::from_string(&uid_str)
                    .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?,
            )
        } else {
            None
        };

        // List roles
        let roles = self
            .role_service
            .list_roles(user_id)
            .await
            .map_err(Self::error_to_status)?;

        // Convert to proto roles
        let proto_roles: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                role_id: role.id.to_string(),
                name: role.name,
                description: role.description.unwrap_or_default(),
                permissions: role
                    .permissions
                    .into_iter()
                    .map(|p| format!("{}:{}", p.permission, p.resource.unwrap_or_default()))
                    .collect(),
            })
            .collect();

        debug!("Found {} roles", proto_roles.len());
        Ok(Response::new(ListRolesResponse { roles: proto_roles }))
    }

    // OAuth2/OIDC methods
    async fn get_o_auth_token(
        &self,
        request: Request<OAuthTokenRequest>,
    ) -> Result<Response<OAuthTokenResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC GetOAuthToken request with grant_type: {}",
            req.grant_type
        );

        // Use default realm for now
        let realm_id = RealmId::new(); // TODO: Parse from request metadata

        // Build TokenRequest from proto message
        let token_request = authenc_types::TokenRequest {
            grant_type: req.grant_type.clone(),
            code: req.code,
            redirect_uri: req.redirect_uri,
            code_verifier: None, // PKCE code_verifier should be provided in authorization_code flow
            client_id: req.client_id.unwrap_or_default(),
            client_secret: req.client_secret,
            refresh_token: req.refresh_token,
            scope: if req.scopes.is_empty() {
                None
            } else {
                Some(req.scopes.join(" "))
            },
            realm_id,
        };

        // Call OAuth2 service
        let token_response = self
            .oauth2_service
            .token(token_request)
            .await
            .map_err(Self::error_to_status)?;

        // Convert to proto response
        Ok(Response::new(OAuthTokenResponse {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            expires_in: token_response.expires_in as i64,
            refresh_token: token_response.refresh_token,
            id_token: None, // TODO: Generate ID token for OIDC
            scopes: token_response
                .scope
                .split_whitespace()
                .map(String::from)
                .collect(),
        }))
    }

    async fn introspect_token(
        &self,
        request: Request<IntrospectTokenRequest>,
    ) -> Result<Response<IntrospectTokenResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC IntrospectToken request");

        // Verify JWT token
        match self.jwt_service.verify_token(&req.token) {
            Ok(claims) => {
                // Extract scopes from claims
                let scopes = claims
                    .scope
                    .as_ref()
                    .map(|s| s.split_whitespace().map(String::from).collect())
                    .unwrap_or_else(Vec::new);

                // Check if token is expired
                let now = chrono::Utc::now().timestamp();
                let active = claims.exp > now;

                Ok(Response::new(IntrospectTokenResponse {
                    active,
                    user_id: Some(claims.sub),
                    client_id: None, // TODO: Extract client_id from claims if present
                    scopes,
                    exp: Some(claims.exp),
                    iat: Some(claims.iat),
                }))
            }
            Err(e) => {
                debug!("Token introspection failed: {}", e);
                // Return inactive token response (not an error)
                Ok(Response::new(IntrospectTokenResponse {
                    active: false,
                    user_id: None,
                    client_id: None,
                    scopes: vec![],
                    exp: None,
                    iat: None,
                }))
            }
        }
    }

    async fn get_user_info(
        &self,
        request: Request<UserInfoRequest>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC GetUserInfo request");

        // Verify access token
        let claims = self
            .jwt_service
            .verify_token(&req.access_token)
            .map_err(|e| Status::unauthenticated(format!("Invalid access token: {}", e)))?;

        // Parse user_id from claims
        let user_id = UserId::from_string(&claims.sub)
            .map_err(|e| Status::internal(format!("Invalid user_id in token: {}", e)))?;

        // Get user from database
        let user = self
            .user_service
            .get_user(user_id)
            .await
            .map_err(Self::error_to_status)?;

        // Build UserInfo response (OIDC standard claims)
        let mut additional_claims = std::collections::HashMap::new();

        // Add realm if present in claims
        if let Some(realm) = claims.realm {
            additional_claims.insert("realm".to_string(), realm);
        }

        // Add session_id if present
        if let Some(sid) = claims.sid {
            additional_claims.insert("sid".to_string(), sid);
        }

        Ok(Response::new(UserInfoResponse {
            sub: user.id.to_string(),
            email: user.email.clone(),
            email_verified: user.email_verified,
            name: None,    // TODO: Add full_name field to User model
            picture: None, // TODO: Add profile picture support
            additional_claims,
        }))
    }

    // Federation methods
    async fn initiate_federated_auth(
        &self,
        request: Request<FederatedAuthRequest>,
    ) -> Result<Response<FederatedAuthResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC InitiateFederatedAuth request for provider: {}",
            req.provider
        );

        // Use default realm for now
        let realm_id = RealmId::new(); // TODO: Parse from request metadata

        // Build domain request
        let domain_request = DomainFederatedAuthRequest {
            provider: req.provider,
            redirect_uri: req.redirect_uri,
            scopes: req.scopes,
            realm_id,
        };

        // Initiate federated authentication
        let response = self
            .federation_service
            .initiate_federated_auth(domain_request)
            .await
            .map_err(Self::error_to_status)?;

        debug!(
            "Federated auth initiated successfully, auth_url: {}",
            response.auth_url
        );

        Ok(Response::new(FederatedAuthResponse {
            auth_url: response.auth_url,
            state: response.state,
        }))
    }

    async fn complete_federated_auth(
        &self,
        request: Request<CompleteFederatedAuthRequest>,
    ) -> Result<Response<CompleteFederatedAuthResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC CompleteFederatedAuth request for provider: {}",
            req.provider
        );

        // Use default realm for now
        let realm_id = RealmId::new(); // TODO: Parse from request metadata

        // Build domain request
        let domain_request = DomainCompleteFederatedAuthRequest {
            provider: req.provider,
            code: req.code,
            state: req.state,
            realm_id,
        };

        // Complete federated authentication
        let response = self
            .federation_service
            .complete_federated_auth(domain_request)
            .await
            .map_err(Self::error_to_status)?;

        // Get user info
        let user = self
            .user_service
            .get_user(response.user_id)
            .await
            .map_err(Self::error_to_status)?;

        debug!(
            "Federated auth completed successfully for user: {}",
            response.user_id
        );

        Ok(Response::new(CompleteFederatedAuthResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            user: Some(UserInfo {
                user_id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: None,
                roles: vec![],
                is_active: user.enabled,
                mfa_enabled: user.mfa_enabled,
                created_at: user.created_at.timestamp(),
                last_login: 0,
            }),
        }))
    }

    // Audit methods - TODO: Implement with audit service
    async fn get_audit_logs(
        &self,
        request: Request<AuditLogsRequest>,
    ) -> Result<Response<AuditLogsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC GetAuditLogs request: user_id={:?}, action={:?}",
            req.user_id, req.action
        );

        // Parse user_id if provided
        let user_id = if let Some(uid_str) = req.user_id {
            if !uid_str.is_empty() {
                Some(
                    Uuid::parse_str(&uid_str)
                        .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Parse action if provided
        let action = req.action.filter(|a| !a.is_empty());

        // Get audit logs from service
        let (logs, total) = self
            .audit_service
            .get_audit_logs(
                user_id,
                action,
                req.start_time,
                req.end_time,
                req.limit.unwrap_or(100),
                req.offset.unwrap_or(0),
            )
            .await
            .map_err(Self::error_to_status)?;

        // Convert domain AuditLog to proto AuditLog
        let proto_logs = logs
            .into_iter()
            .map(|log| AuditLog {
                id: log.id.to_string(),
                user_id: log.user_id.map(|u| u.to_string()).unwrap_or_default(),
                action: format!("{:?}", log.operation),
                resource: log.resource_type.unwrap_or_default(),
                success: matches!(
                    log.result,
                    authenc_types::domain::audit::AuditResult::Success
                ),
                ip_address: log.ip_address.unwrap_or_default(),
                user_agent: log.user_agent.unwrap_or_default(),
                timestamp: log.timestamp.timestamp(),
                metadata: log
                    .details
                    .and_then(|d| {
                        serde_json::from_value::<std::collections::HashMap<String, String>>(d).ok()
                    })
                    .unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(AuditLogsResponse {
            logs: proto_logs,
            total,
        }))
    }

    async fn get_compliance_report(
        &self,
        request: Request<ComplianceReportRequest>,
    ) -> Result<Response<ComplianceReportResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC GetComplianceReport request: start_time={}, end_time={}",
            req.start_time, req.end_time
        );

        // Get compliance report from service
        let metrics_map = self
            .audit_service
            .get_compliance_report(req.start_time, req.end_time, req.report_types)
            .await
            .map_err(Self::error_to_status)?;

        // Convert domain ComplianceMetrics to proto ComplianceMetrics
        let proto_metrics: std::collections::HashMap<String, ComplianceMetrics> = metrics_map
            .into_iter()
            .map(|(key, metrics)| {
                (
                    key,
                    ComplianceMetrics {
                        total_authentications: metrics.total_logins as i64,
                        failed_authentications: metrics.failed_logins as i64,
                        mfa_enabled_users: metrics.mfa_enabled_users as i64,
                        active_sessions: metrics.active_sessions as i64,
                        additional_metrics: {
                            let mut m = std::collections::HashMap::new();
                            m.insert("active_users".to_string(), metrics.active_users as i64);
                            m.insert(
                                "locked_accounts".to_string(),
                                metrics.locked_accounts as i64,
                            );
                            m.insert(
                                "password_changes".to_string(),
                                metrics.password_changes as i64,
                            );
                            m.insert(
                                "security_incidents".to_string(),
                                metrics.security_incidents as i64,
                            );
                            m
                        },
                    },
                )
            })
            .collect();

        Ok(Response::new(ComplianceReportResponse {
            metrics: proto_metrics,
            generated_at: chrono::Utc::now().timestamp(),
        }))
    }

    // Health check
    async fn health_check(
        &self,
        request: Request<crate::proto::common::v1::HealthCheckRequest>,
    ) -> Result<Response<crate::proto::common::v1::HealthCheckResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC HealthCheck request for service: {}", req.service);

        Ok(Response::new(
            crate::proto::common::v1::HealthCheckResponse {
                info: Some(crate::proto::common::v1::HealthInfo {
                    service_name: "authenc-grpc".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    status: crate::proto::common::v1::HealthStatus::Healthy as i32,
                    uptime_seconds: 0, // TODO: Track actual uptime
                    dependencies: std::collections::HashMap::new(),
                }),
            },
        ))
    }

    // CAPTCHA methods - TODO: Implement with CAPTCHA service
    async fn generate_captcha_challenge(
        &self,
        _request: Request<CaptchaChallengeRequest>,
    ) -> Result<Response<CaptchaChallengeResponse>, Status> {
        Err(Status::unimplemented(
            "CAPTCHA not yet implemented - requires CAPTCHA service",
        ))
    }

    async fn verify_captcha_challenge(
        &self,
        _request: Request<CaptchaVerificationRequest>,
    ) -> Result<Response<CaptchaVerificationResponse>, Status> {
        Err(Status::unimplemented(
            "CAPTCHA not yet implemented - requires CAPTCHA service",
        ))
    }
}
