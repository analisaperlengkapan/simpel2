//! Authentication and authorization handlers.
//!
//! Provides endpoints for user login, token management, MFA,
//! and OAuth2 integration.

use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Json,
    routing::{delete, get, post},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

// Use canonical types from core
use secreton_core::models::{LoginRequest, LoginResponse, RefreshTokenRequest, UserInfo};

use crate::{
    ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser, handlers::AppState,
    helpers::extract_client_ip, services::ServiceContainer,
};

/// Create authentication routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/token/refresh", post(refresh_token))
        .route("/token/verify", post(verify_token))
        .route("/mfa/setup", post(setup_mfa))
        .route("/mfa/verify", post(verify_mfa))
        .route("/mfa/disable", post(disable_mfa))
        .route("/oauth/{provider}", get(oauth_login))
        .route("/oauth/{provider}/callback", get(oauth_callback))
        .route("/sessions", get(list_sessions))
        .route("/sessions/{session_id}", delete(revoke_session))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::middleware::RequestContext;
    use crate::services::ServiceContainer;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use std::sync::Arc;

    async fn mock_auth_middleware(
        req: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let mut req = req;
        if let Some(user_id) = req.headers().get("X-Test-User-Id")
            && let Ok(user_id_str) = user_id.to_str()
        {
            let context = RequestContext {
                request_id: "test-req".to_string(),
                user_id: Some(user_id_str.to_string()),
                user_email: None,
                user_roles: vec![],
                user_permissions: vec![],
                start_time: std::time::Instant::now(),
                jwt_claims: None,
                auth_token: None,
                client_ip: None,
                user_agent: None,
                policy_names: vec![],
            };
            req.extensions_mut().insert(context);
        }
        next.run(req).await
    }

    async fn create_test_server() -> TestServer {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        // Create test user 'alice'
        let _ = services
            .auth
            .create_user(
                "alice",
                "alice@example.com",
                "password123",
                None,
                vec!["user".to_string()],
                None,
                true,
            )
            .await;

        let app = create_routes()
            .with_state(services)
            .layer(axum::middleware::from_fn(mock_auth_middleware));

        TestServer::new(app)
    }

    #[tokio::test]
    async fn test_login_endpoint_returns_tokens() {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        // Create test user
        services
            .auth
            .create_user(
                "alice",
                "alice@example.com",
                "password123",
                None,
                vec!["user".to_string()],
                None,
                true,
            )
            .await
            .expect("Failed to create user");

        let app = create_routes()
            .with_state(services)
            .layer(axum::middleware::from_fn(mock_auth_middleware));

        let server = TestServer::new(app);

        let request = LoginRequest {
            username: "alice".to_string(),
            password: "password123".to_string(),
            mfa_code: None,
            remember_me: Some(true),
        };

        let response = server.post("/login").json(&request).await;
        response.assert_status_ok();

        let body: ApiResponse<LoginResponse> = response.json();
        assert!(body.success);
        let data = body.data.expect("login response");
        assert_eq!(data.token_type, "Bearer");
        assert_eq!(data.user.username, "alice");
        assert!(!data.mfa_required);
    }

    #[tokio::test]
    async fn test_mfa_setup_sms_success() {
        let server = create_test_server().await;
        let request = MfaSetupRequest {
            method: "sms".to_string(),
            phone_number: Some("+1234567890".to_string()),
            email: None,
        };

        let response = server
            .post("/mfa/setup")
            .add_header("X-Test-User-Id", uuid::Uuid::new_v4().to_string())
            .json(&request)
            .await;

        response.assert_status_ok();
        let body: ApiResponse<MfaSetupResponse> = response.json();
        assert!(body.success);
        let data = body.data.expect("setup response");
        assert_eq!(data.method, "sms");
        assert_eq!(data.backup_codes.len(), 10);
    }

    #[tokio::test]
    async fn test_mfa_setup_sms_validation_failure() {
        let server = create_test_server().await;
        let request = MfaSetupRequest {
            method: "sms".to_string(),
            phone_number: None,
            email: None,
        };

        let response = server
            .post("/mfa/setup")
            .add_header("X-Test-User-Id", uuid::Uuid::new_v4().to_string())
            .json(&request)
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);
        let body: ApiResponse<serde_json::Value> = response.json();
        assert!(!body.success);
        let error = body.error.expect("error payload");
        assert_eq!(error.code, "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn test_mfa_setup_rejects_unsupported_method() {
        let server = create_test_server().await;
        let request = MfaSetupRequest {
            method: "webauthn".to_string(),
            phone_number: None,
            email: None,
        };

        let response = server
            .post("/mfa/setup")
            .add_header("X-Test-User-Id", uuid::Uuid::new_v4().to_string())
            .json(&request)
            .await;

        response.assert_status(StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_oauth_login_returns_authorization_url() {
        use crate::config::{OAuth2Config, OAuth2Provider};

        let mut config = ApiConfig::default();
        config.auth.oauth2 = Some(OAuth2Config {
            providers: vec![OAuth2Provider {
                name: "github".to_string(),
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                user_info_url: "https://api.github.com/user".to_string(),
            }],
            redirect_url: "http://localhost:8200/auth/callback".to_string(),
            scopes: vec!["read:user".to_string()],
        });

        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_routes()
            .with_state(services)
            .layer(axum::middleware::from_fn(mock_auth_middleware));

        let server = TestServer::new(app);

        let response = server.get("/oauth/github").await;
        response.assert_status_ok();

        let body: ApiResponse<serde_json::Value> = response.json();
        assert!(body.success);
        let data = body.data.expect("oauth payload");
        assert_eq!(data["provider"], "github");

        let auth_url = data["auth_url"].as_str().expect("auth_url string");
        assert!(auth_url.starts_with("https://github.com/login/oauth/authorize"));
        assert!(auth_url.contains("client_id=test_client_id"));
        assert!(auth_url.contains("state="));
        assert!(auth_url.contains("scope=read%3Auser"));
    }

    #[tokio::test]
    async fn test_oauth_callback_invalid_state() {
        let server = create_test_server().await;

        let response = server
            .get("/oauth/github/callback")
            .add_query_param("state", "invalid_state")
            .add_query_param("code", "some_code")
            .await;

        // Should be 400 because state is invalid/expired (Validation error)
        response.assert_status(StatusCode::BAD_REQUEST);

        let body: ApiResponse<serde_json::Value> = response.json();
        assert!(!body.success);
        let error = body.error.expect("error payload");
        assert_eq!(error.code, "INVALID_REQUEST");
    }
}

// LoginRequest, LoginResponse, RefreshTokenRequest, UserInfo now imported from secreton_core::models

/// Token verification request
#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

/// MFA setup request
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupRequest {
    pub method: String, // "totp", "sms", "email", "webauthn"
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

/// MFA setup response
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub method: String,
    pub secret: Option<String>,  // For TOTP
    pub qr_code: Option<String>, // For TOTP
    pub backup_codes: Vec<String>,
}

/// MFA verification request
#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub method: String,
    pub code: String,
    pub backup_code: Option<String>,
}

/// Session information
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_current: bool,
}

/// User login endpoint
pub async fn login(
    State(state): State<Arc<ServiceContainer>>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    // Extract client info for session tracking
    let ip_address = extract_client_ip(&headers).unwrap_or_else(|| "unknown".to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Authenticate using AuthService
    let auth_result = state
        .auth
        .authenticate(
            &request.username,
            &request.password,
            request.mfa_code.as_deref(),
            &ip_address,
            user_agent,
        )
        .await;

    match auth_result {
        Ok(auth_token) => {
            let policies = state
                .auth
                .get_user_policies(&auth_token.user)
                .await
                .unwrap_or_else(|_| vec!["default".to_string()]);

            // Convert User to UserInfo
            let user_info = UserInfo {
                username: auth_token.user.username.clone(),
                email: Some(auth_token.user.email.clone()),
                display_name: auth_token.user.full_name.clone(),
                groups: auth_token.user.roles.iter().cloned().collect(),
                policies,
                metadata: HashMap::new(),
            };

            let response = LoginResponse {
                access_token: auth_token.access_token,
                refresh_token: auth_token.refresh_token,
                token_type: auth_token.token_type,
                expires_in: auth_token.expires_in,
                user: user_info.clone(),
                mfa_required: false,
            };

            // Audit: authentication success
            let audit_entry = secreton_core::audit::AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "authentication_success".to_string(),
                actor: Some(user_info.username.clone()),
                resource_type: "auth".to_string(),
                resource_id: user_info.username.clone(),
                status: secreton_core::audit::AuditStatus::Success,
                ip: Some(ip_address.clone()),
                user_agent: Some(user_agent.to_string()),
                namespace: Some(auth_token.user.namespace),
                metadata: HashMap::new(),
            };
            let _ = state.audit.log(audit_entry).await;

            Ok(Json(ApiResponse::success(response)))
        }
        Err(crate::services::auth::AuthError::MfaRequired) => {
            // MFA is required but not provided
            let response = LoginResponse {
                access_token: String::new(),
                refresh_token: String::new(),
                token_type: "Bearer".to_string(),
                expires_in: 0,
                user: UserInfo {
                    username: request.username.clone(),
                    email: None,
                    display_name: None,
                    groups: vec![],
                    policies: vec![],
                    metadata: HashMap::new(),
                },
                mfa_required: true,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            // Audit: authentication failure
            let audit_entry = secreton_core::audit::AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "authentication_failure".to_string(),
                actor: Some(request.username.clone()),
                resource_type: "auth".to_string(),
                resource_id: request.username.clone(),
                status: secreton_core::audit::AuditStatus::Failure,
                ip: Some(ip_address.clone()),
                user_agent: Some(user_agent.to_string()),
                namespace: None,
                metadata: [("error".to_string(), e.to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            };
            let _ = state.audit.log(audit_entry).await;

            Err(ApiError::Authentication {
                message: e.to_string(),
            })
        }
    }
}

/// User logout endpoint
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // Extract token from Authorization header
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("");

    // Extract client info
    let ip_address = extract_client_ip(&headers).unwrap_or_else(|| "unknown".to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Attempt to validate token to get user info for audit
    let username = if !token.is_empty() {
        state
            .auth
            .validate_token(token)
            .await
            .ok()
            .map(|u| u.username)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    // Invalidate token
    if !token.is_empty()
        && let Err(e) = state.auth.revoke_token(token).await
    {
        tracing::warn!("Failed to revoke token during logout: {}", e);
    }

    // Audit log
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "logout".to_string(),
        actor: Some(username),
        resource_type: "auth".to_string(),
        resource_id: "session".to_string(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: Some(ip_address.clone()),
        user_agent: Some(user_agent.to_string()),
        namespace: None,
        metadata: HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    let data = serde_json::json!({
        "message": "Successfully logged out"
    });

    Ok(Json(ApiResponse::success(data)))
}

/// Refresh access token
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RefreshTokenRequest>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    // Use AuthService to refresh token
    match state.auth.refresh_token(&request.refresh_token).await {
        Ok(auth_token) => {
            let policies = state
                .auth
                .get_user_policies(&auth_token.user)
                .await
                .unwrap_or_else(|_| vec!["default".to_string()]);

            let user_info = UserInfo {
                username: auth_token.user.username.clone(),
                email: Some(auth_token.user.email.clone()),
                display_name: auth_token.user.full_name.clone(),
                groups: auth_token.user.roles.iter().cloned().collect(),
                policies,
                metadata: HashMap::new(),
            };

            let response = LoginResponse {
                access_token: auth_token.access_token,
                refresh_token: auth_token.refresh_token,
                token_type: auth_token.token_type,
                expires_in: auth_token.expires_in,
                user: user_info.clone(),
                mfa_required: false,
            };

            // Audit: token refresh
            let audit_entry = secreton_core::audit::AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "token_refresh".to_string(),
                actor: Some(user_info.username.clone()),
                resource_type: "auth".to_string(),
                resource_id: user_info.username.clone(),
                status: secreton_core::audit::AuditStatus::Success,
                ip: extract_client_ip(&headers),
                user_agent: None,
                namespace: Some(auth_token.user.namespace),
                metadata: HashMap::new(),
            };
            let _ = state.audit.log(audit_entry).await;

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(ApiError::Authentication {
            message: format!("Token refresh failed: {}", e),
        }),
    }
}

/// Verify token validity
pub async fn verify_token(
    State(state): State<Arc<ServiceContainer>>,
    Json(request): Json<VerifyTokenRequest>,
) -> ApiResult<Json<ApiResponse<UserInfo>>> {
    // Validate token using AuthService
    match state.auth.validate_token(&request.token).await {
        Ok(user) => {
            let policies = state
                .auth
                .get_user_policies(&user)
                .await
                .unwrap_or_else(|_| vec!["default".to_string()]);

            let user_info = UserInfo {
                username: user.username,
                email: Some(user.email),
                display_name: user.full_name,
                groups: user.roles.iter().cloned().collect(),
                policies,
                metadata: HashMap::new(),
            };
            Ok(Json(ApiResponse::success(user_info)))
        }
        Err(e) => Err(ApiError::Authentication {
            message: format!("Invalid token: {}", e),
        }),
    }
}

/// Setup MFA for user
pub async fn setup_mfa(
    State(state): State<Arc<ServiceContainer>>,
    headers: HeaderMap,
    user: AuthenticatedUser,
    Json(request): Json<MfaSetupRequest>,
) -> ApiResult<Json<ApiResponse<MfaSetupResponse>>> {
    // 1. Extract user ID from AuthenticatedUser (extracted from JWT by middleware)
    let user_id = user.id.to_string();
    let user_label = user
        .email
        .unwrap_or_else(|| format!("{}@kejaksaan.go.id", user.username));

    // 2. Generate MFA configuration based on method
    let response = match request.method.as_str() {
        "totp" => {
            // Generate TOTP configuration
            let totp_config = state
                .mfa
                .enable_totp(&user_id, "Secreton Engine".to_string(), user_label)
                .await
                .map_err(|e| ApiError::Internal {
                    message: format!("Failed to setup TOTP: {}", e),
                })?;

            // Get recovery codes (from MfaConfig)
            let mfa_config =
                state
                    .mfa
                    .get_config(&user_id)
                    .await
                    .ok_or_else(|| ApiError::Internal {
                        message: "Failed to retrieve MFA configuration".to_string(),
                    })?;

            MfaSetupResponse {
                method: "totp".to_string(),
                secret: Some(totp_config.secret.clone()),
                qr_code: Some(totp_config.qr_code_url.clone()),
                backup_codes: mfa_config.recovery_codes.clone(),
            }
        }
        "email" => {
            if let Some(email) = &request.email {
                // Initiate Email setup
                let _code = state
                    .mfa
                    .initiate_email_setup(&user_id, email.clone())
                    .await
                    .map_err(|e| ApiError::Internal {
                        message: format!("Failed to initiate Email setup: {}", e),
                    })?;

                // Get recovery codes
                let mfa_config =
                    state
                        .mfa
                        .get_config(&user_id)
                        .await
                        .ok_or_else(|| ApiError::Internal {
                            message: "Failed to retrieve MFA configuration".to_string(),
                        })?;

                MfaSetupResponse {
                    method: "email".to_string(),
                    secret: None,
                    qr_code: None,
                    backup_codes: mfa_config.recovery_codes.clone(),
                }
            } else {
                return Err(ApiError::Validation {
                    message: "Email is required for email MFA method".to_string(),
                    field: Some("email".to_string()),
                    details: None,
                });
            }
        }
        "sms" => {
            if let Some(phone) = &request.phone_number {
                // Initiate SMS setup
                let _code = state
                    .mfa
                    .initiate_sms_setup(&user_id, phone.clone())
                    .await
                    .map_err(|e| ApiError::Internal {
                        message: format!("Failed to initiate SMS setup: {}", e),
                    })?;

                // Get recovery codes
                let mfa_config =
                    state
                        .mfa
                        .get_config(&user_id)
                        .await
                        .ok_or_else(|| ApiError::Internal {
                            message: "Failed to retrieve MFA configuration".to_string(),
                        })?;

                MfaSetupResponse {
                    method: "sms".to_string(),
                    secret: None,
                    qr_code: None,
                    backup_codes: mfa_config.recovery_codes.clone(),
                }
            } else {
                return Err(ApiError::Validation {
                    message: "Phone number is required for SMS MFA method".to_string(),
                    field: Some("phone_number".to_string()),
                    details: None,
                });
            }
        }
        "webauthn" => {
            // TODO: Implement WebAuthn MFA setup
            return Err(ApiError::NotImplemented(
                "WebAuthn MFA not yet implemented".to_string(),
            ));
        }
        _ => {
            return Err(ApiError::Validation {
                message: format!("Unsupported MFA method: {}", request.method),
                field: Some("method".to_string()),
                details: Some(HashMap::from([(
                    "supported_methods".to_string(),
                    "totp, email, sms, webauthn".to_string(),
                )])),
            });
        }
    };

    // 3. Audit log the MFA setup
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "mfa_setup".to_string(),
        actor: Some(user_id.to_string()),
        resource_type: "mfa".to_string(),
        resource_id: request.method.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: extract_client_ip(&headers),
        user_agent: headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    tracing::info!(
        user_id = %user_id,
        method = %request.method,
        "MFA setup initiated"
    );

    Ok(Json(ApiResponse::success(response)))
}

/// Verify MFA code
pub async fn verify_mfa(
    State(state): State<Arc<ServiceContainer>>,
    headers: HeaderMap,
    user: AuthenticatedUser,
    Json(request): Json<MfaVerifyRequest>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. Extract user ID from AuthenticatedUser
    let user_id = user.id.to_string();

    // 2. Verify MFA code based on method
    let is_valid = match request.method.as_str() {
        "totp" => {
            // Verify TOTP code
            state
                .mfa
                .verify_totp(&user_id, &request.code)
                .await
                .map_err(|e| ApiError::Authentication {
                    message: format!("Failed to verify TOTP code: {}", e),
                })?
        }
        "sms" => {
            // Verify SMS code
            state
                .mfa
                .verify_sms_setup(&user_id, &request.code)
                .await
                .map_err(|e| ApiError::Authentication {
                    message: format!("Failed to verify SMS code: {}", e),
                })?
        }
        "email" => {
            // Verify Email code
            state
                .mfa
                .verify_email_setup(&user_id, &request.code)
                .await
                .map_err(|e| ApiError::Authentication {
                    message: format!("Failed to verify Email code: {}", e),
                })?
        }
        "recovery" => {
            // Use recovery code
            if let Some(backup_code) = &request.backup_code {
                state
                    .mfa
                    .verify_recovery_code(&user_id, backup_code)
                    .await
                    .map_err(|e| ApiError::Authentication {
                        message: format!("Invalid recovery code: {}", e),
                    })?;
                true
            } else {
                return Err(ApiError::Validation {
                    message: "Recovery code is required for recovery method".to_string(),
                    field: Some("backup_code".to_string()),
                    details: None,
                });
            }
        }
        _ => {
            return Err(ApiError::Validation {
                message: format!("Unsupported verification method: {}", request.method),
                field: Some("method".to_string()),
                details: None,
            });
        }
    };

    // 3. Check verification result
    if !is_valid {
        // Audit failed verification
        let audit_entry = secreton_core::audit::AuditLog {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: "mfa_verification_failed".to_string(),
            actor: Some(user_id.to_string()),
            resource_type: "mfa".to_string(),
            resource_id: request.method.clone(),
            status: secreton_core::audit::AuditStatus::Failure,
            ip: extract_client_ip(&headers),
            user_agent: headers
                .get("User-Agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            namespace: None,
            metadata: std::collections::HashMap::new(),
        };
        let _ = state.audit.log(audit_entry).await;

        return Err(ApiError::Authentication {
            message: "Invalid MFA code".to_string(),
        });
    }

    // 4. MFA verification successful - audit log
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "mfa_verified".to_string(),
        actor: Some(user_id.to_string()),
        resource_type: "mfa".to_string(),
        resource_id: request.method.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: extract_client_ip(&headers),
        user_agent: headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    tracing::info!(
        user_id = %user_id,
        method = %request.method,
        "MFA verification successful"
    );

    let data = serde_json::json!({
        "message": "MFA successfully verified and enabled",
        "method": request.method,
        "verified": true
    });

    Ok(Json(ApiResponse::success(data)))
}

/// Disable MFA for user
pub async fn disable_mfa(
    State(state): State<AppState>,
    headers: HeaderMap,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. Extract user ID from AuthenticatedUser
    let user_id = user.id.to_string();

    // 2. Check if MFA is configured for this user
    let mfa_config = state.mfa.get_config(&user_id).await;

    if mfa_config.is_none() {
        return Err(ApiError::NotFound {
            resource: "MFA configuration for this user".to_string(),
        });
    }

    // 3. Disable MFA for the user (disable TOTP specifically)
    state
        .mfa
        .disable_totp(&user_id)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to disable MFA: {}", e),
        })?;

    // 4. Audit log the MFA disable action
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "mfa_disabled".to_string(),
        actor: Some(user_id.clone()),
        resource_type: "mfa".to_string(),
        resource_id: user_id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: extract_client_ip(&headers),
        user_agent: headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    tracing::warn!(
        user_id = %user_id,
        "MFA disabled for user - security reduced"
    );

    let data = serde_json::json!({
        "message": "MFA successfully disabled",
        "warning": "Your account security has been reduced. Consider re-enabling MFA."
    });

    Ok(Json(ApiResponse::success(data)))
}

/// OAuth login redirect
pub async fn oauth_login(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. Validate provider
    let oauth_config = state
        .config
        .auth
        .oauth2
        .as_ref()
        .ok_or_else(|| ApiError::NotFound {
            resource: "OAuth2 configuration".to_string(),
        })?;

    let provider_config = oauth_config
        .providers
        .iter()
        .find(|p| p.name == provider)
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("OAuth provider '{}'", provider),
        })?;

    // 2. Generate OAuth state
    let state_string = uuid::Uuid::new_v4().to_string();

    // 3. Store state for callback verification
    let state_path = format!("sys/oauth/states/{}", state_string);

    // Encrypt state data containing provider info
    let state_data = serde_json::json!({
        "provider": provider,
        "created_at": chrono::Utc::now()
    });
    let state_bytes = serde_json::to_vec(&state_data).map_err(|e| ApiError::Internal {
        message: format!("Serialization failed: {}", e),
    })?;

    let encrypted_data =
        state
            .crypto
            .encrypt_simple(&state_bytes)
            .map_err(|e| ApiError::Internal {
                message: format!("Encryption failed: {}", e),
            })?;

    let entry = secreton_storage::SecretEntry::new(
        state_path,
        encrypted_data,
        serde_json::json!({
            "method": "simple",
            "provider": provider
        }),
        secreton_storage::SecurityLevel::Internal,
        "system".to_string(),
    )
    .with_expiration(chrono::Utc::now() + chrono::Duration::minutes(10));

    state
        .storage
        .store(&entry)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to store OAuth state: {}", e),
        })?;

    // 4. Build authorization URL
    let mut url = url::Url::parse(&provider_config.auth_url).map_err(|e| ApiError::Internal {
        message: e.to_string(),
    })?;

    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("client_id", &provider_config.client_id);
        pairs.append_pair("redirect_uri", &oauth_config.redirect_url);
        pairs.append_pair("response_type", "code");
        if !oauth_config.scopes.is_empty() {
            pairs.append_pair("scope", &oauth_config.scopes.join(" "));
        }
        pairs.append_pair("state", &state_string);
    }

    // Store provider name in metadata/data so we know who to verify against in callback if needed
    let data = serde_json::json!({
        "provider": provider,
        "auth_url": url.to_string(),
        "state": state_string
    });

    Ok(Json(ApiResponse::success(data)))
}

/// OAuth callback handler
pub async fn oauth_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    // Check for OAuth error response first
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("Authorization failed");
        return Err(ApiError::Authentication {
            message: format!("OAuth error: {} - {}", error, description),
        });
    }

    // 1. Verify state parameter
    let code = params.get("code").ok_or(ApiError::Validation {
        message: "Missing 'code' parameter".to_string(),
        field: Some("code".to_string()),
        details: None,
    })?;

    let state_param = params.get("state").ok_or(ApiError::Validation {
        message: "Missing 'state' parameter".to_string(),
        field: Some("state".to_string()),
        details: None,
    })?;

    let state_path = format!("sys/oauth/states/{}", state_param);
    let state_entry =
        state
            .storage
            .get_by_path(&state_path)
            .await
            .map_err(|e| ApiError::Internal {
                message: format!("Failed to verify state: {}", e),
            })?;

    let state_entry = state_entry.ok_or(ApiError::Validation {
        message: "Invalid or expired state parameter".to_string(),
        field: Some("state".to_string()),
        details: None,
    })?;

    // Check expiration
    if state_entry.is_expired() {
        let _ = state.storage.delete_by_path(&state_path).await;
        return Err(ApiError::Validation {
            message: "Invalid or expired state parameter".to_string(),
            field: Some("state".to_string()),
            details: None,
        });
    }

    // Verify provider matches (decrypt state data)
    let decrypted_bytes = state
        .crypto
        .decrypt_simple(&state_entry.encrypted_data)
        .map_err(|_| ApiError::Authentication {
            message: "Invalid state data".to_string(),
        })?;

    let state_data: serde_json::Value =
        serde_json::from_slice(&decrypted_bytes).map_err(|_| ApiError::Authentication {
            message: "Invalid state data".to_string(),
        })?;

    if state_data["provider"].as_str() != Some(provider.as_str()) {
        let _ = state.storage.delete_by_path(&state_path).await;
        return Err(ApiError::Validation {
            message: "Provider mismatch in OAuth state".to_string(),
            field: Some("state".to_string()),
            details: None,
        });
    }

    // Delete used state
    let _ = state.storage.delete_by_path(&state_path).await;

    // 2. Get provider config
    let oauth_config = state
        .config
        .auth
        .oauth2
        .as_ref()
        .ok_or_else(|| ApiError::NotFound {
            resource: "OAuth2 configuration".to_string(),
        })?;

    let provider_config = oauth_config
        .providers
        .iter()
        .find(|p| p.name == provider)
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("OAuth provider '{}'", provider),
        })?;

    // 3. Exchange code for access token
    let client = HTTP_CLIENT.get_or_init(reqwest::Client::new);
    let token_params = [
        ("client_id", provider_config.client_id.as_str()),
        ("client_secret", provider_config.client_secret.as_str()),
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", oauth_config.redirect_url.as_str()),
    ];

    let token_res = client
        .post(&provider_config.token_url)
        .form(&token_params)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to request access token: {}", e),
        })?;

    if !token_res.status().is_success() {
        let error_text = token_res.text().await.unwrap_or_default();
        return Err(ApiError::Authentication {
            message: format!("OAuth provider rejected token request: {}", error_text),
        });
    }

    let token_data: serde_json::Value = token_res.json().await.map_err(|e| ApiError::Internal {
        message: format!("Failed to parse token response: {}", e),
    })?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or(ApiError::Authentication {
            message: "Missing access_token in provider response".to_string(),
        })?;

    // 4. Fetch user information
    let user_info_res = client
        .get(&provider_config.user_info_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "Secreton-Engine")
        .send()
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to request user info: {}", e),
        })?;

    if !user_info_res.status().is_success() {
        let error_text = user_info_res.text().await.unwrap_or_default();
        return Err(ApiError::Authentication {
            message: format!("OAuth provider rejected user info request: {}", error_text),
        });
    }

    let user_data: serde_json::Value =
        user_info_res.json().await.map_err(|e| ApiError::Internal {
            message: format!("Failed to parse user info: {}", e),
        })?;

    // Extract email and name
    let email = user_data["email"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::Authentication {
            message: "Email not provided by OAuth provider".to_string(),
        })?;

    let name = user_data["name"]
        .as_str()
        .or_else(|| user_data["login"].as_str()) // GitHub fallback
        .map(|s| s.to_string());

    // Metadata from provider
    let mut metadata = HashMap::new();
    metadata.insert("provider".to_string(), provider.clone());
    if let Some(sub) = user_data["sub"].as_str() {
        metadata.insert("provider_id".to_string(), sub.to_string());
    } else if let Some(id) = user_data["id"].as_i64() {
        metadata.insert("provider_id".to_string(), id.to_string());
    }

    // 5. Create/update user account
    let ip_address = extract_client_ip(&headers).unwrap_or_else(|| "unknown".to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let auth_token = state
        .auth
        .login_external(&email, name, metadata, &ip_address, user_agent)
        .await
        .map_err(|e| ApiError::Authentication {
            message: e.to_string(),
        })?;

    // 6. Generate response
    let policies = state
        .auth
        .get_user_policies(&auth_token.user)
        .await
        .unwrap_or_else(|_| vec!["default".to_string()]);

    let user_info = UserInfo {
        username: auth_token.user.username.clone(),
        email: Some(auth_token.user.email.clone()),
        display_name: auth_token.user.full_name.clone(),
        groups: auth_token.user.roles.iter().cloned().collect(),
        policies,
        metadata: HashMap::new(),
    };

    let response = LoginResponse {
        access_token: auth_token.access_token,
        refresh_token: auth_token.refresh_token,
        token_type: auth_token.token_type,
        expires_in: auth_token.expires_in,
        user: user_info.clone(),
        mfa_required: false,
    };

    // Audit: authentication success
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "oauth_login_success".to_string(),
        actor: Some(user_info.username.clone()),
        resource_type: "auth".to_string(),
        resource_id: provider.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: Some(ip_address.clone()),
        user_agent: Some(user_agent.to_string()),
        namespace: Some(auth_token.user.namespace),
        metadata: HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

/// List user sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<Vec<SessionInfo>>>> {
    // Fetch user sessions from storage
    let sessions = state
        .auth
        .list_user_sessions(&user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: e.to_string(),
        })?;

    // Determine current session from token
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("");

    let current_jti = state.auth.get_token_id(token).unwrap_or_default();

    let session_infos = sessions
        .into_iter()
        .map(|s| SessionInfo {
            id: s.id.clone(),
            user_id: s.user_id,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            created_at: s.created_at,
            last_accessed: s.last_accessed,
            expires_at: s.expires_at,
            is_current: s.id == current_jti,
        })
        .collect();

    Ok(Json(ApiResponse::success(session_infos)))
}

/// Revoke a user session
pub async fn revoke_session(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. Validate session belongs to current user
    let session = state
        .auth
        .get_session(&session_id)
        .await
        .map_err(|_| ApiError::NotFound {
            resource: "Session".to_string(),
        })?;

    // Check ownership
    // Note: session.user_id stores username
    if session.user_id != user.username {
        // Allow admins to revoke any session? For now strict ownership.
        return Err(ApiError::Forbidden);
    }

    // 2. Remove session from storage
    state
        .auth
        .revoke_session(&session_id)
        .await
        .map_err(|e| ApiError::Internal {
            message: e.to_string(),
        })?;

    // 3. Audit log
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "revoke_session".to_string(),
        actor: Some(user.username.clone()),
        resource_type: "auth".to_string(),
        resource_id: session_id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: extract_client_ip(&headers),
        user_agent: headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        namespace: None,
        metadata: HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    let data = serde_json::json!({
        "message": "Session successfully revoked",
        "session_id": session_id
    });

    Ok(Json(ApiResponse::success(data)))
}
