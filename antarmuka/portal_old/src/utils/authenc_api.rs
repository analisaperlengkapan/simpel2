//! Authenc API Client
//!
//! Type-safe, centralized API client for communicating directly with Authenc REST API.
//! Eliminates the need for layanan-portal by calling authenc-api and authenc-iam-api directly.
//!
//! ## Architecture
//!
//! ```text
//! Portal (WASM) → AuthencApiClient → Authenc REST API
//!                                   ├── /api/v1/auth/*      (authenc-api)
//!                                   ├── /api/v1/oauth2/*    (authenc-api)
//!                                   └── /api/v1/iam/*       (authenc-iam-api)
//! ```

use serde::{Deserialize, Serialize};

// ─── Request / Response Types ────────────────────────────────────────────────

/// Login request body
#[derive(Clone, Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub realm: String,
    pub captcha_token: Option<String>,
}

/// Login response from authenc
#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: Option<String>,
    pub temp_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub mfa_required: bool,
    pub mfa_setup_required: bool,
    pub message: String,
}

/// Token refresh request
#[derive(Clone, Debug, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Token response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

/// Current user info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub division: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub mfa_enabled: bool,
    pub email_verified: bool,
    pub created_at: Option<String>,
}

/// Profile update request
#[derive(Clone, Debug, Serialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Password change request
#[derive(Clone, Debug, Serialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Password reset request
#[derive(Clone, Debug, Serialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub captcha_token: Option<String>,
}

/// Password reset confirm request
#[derive(Clone, Debug, Serialize)]
pub struct PasswordResetConfirmRequest {
    pub token: String,
    pub new_password: String,
}

// ─── WebAuthn Types ──────────────────────────────────────────────────────────

/// WebAuthn start response (registration or authentication)
///
/// The backend returns `{ challenge: <WebAuthnOptions>, session_id: <String> }`.
/// The `challenge` is passed to the browser WebAuthn API;
/// the `session_id` is sent back in the finish request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebAuthnStartResponse {
    /// WebAuthn options to pass to navigator.credentials.create()/get()
    pub challenge: serde_json::Value,
    /// Session ID for the finish request
    pub session_id: String,
}

/// WebAuthn finish request wrapper
///
/// Sent as the body to register/finish and authenticate/finish endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct WebAuthnFinishRequest {
    /// Session ID from the start response
    pub session_id: String,
    /// Credential/assertion from the browser WebAuthn API
    pub credential: serde_json::Value,
}

/// Registered passkey/credential info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PasskeyInfo {
    pub id: String,
    pub nickname: Option<String>,
    pub credential_type: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub aaguid: Option<String>,
    pub is_platform: bool,
}

/// Update passkey nickname
#[derive(Clone, Debug, Serialize)]
pub struct UpdatePasskeyRequest {
    pub nickname: String,
}

// ─── MFA Types ───────────────────────────────────────────────────────────────

/// TOTP enable response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TotpEnableResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

/// TOTP verify request
#[derive(Clone, Debug, Serialize)]
pub struct TotpVerifyRequest {
    pub code: String,
    pub temp_token: Option<String>,
}

// ─── Session Types ───────────────────────────────────────────────────────────

/// Active session info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
    pub last_active: String,
    pub is_current: bool,
    pub location: Option<String>,
}

// ─── IAM Admin Types ─────────────────────────────────────────────────────────

/// IAM User (admin view)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IamUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: bool,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: String,
    pub last_login: Option<String>,
    pub roles: Vec<String>,
}

/// Create user request
#[derive(Clone, Debug, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: bool,
}

/// Update user request
#[derive(Clone, Debug, Serialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: Option<bool>,
}

/// Realm info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RealmInfo {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub user_count: Option<u64>,
    pub created_at: String,
}

/// Create realm request
#[derive(Clone, Debug, Serialize)]
pub struct CreateRealmRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub enabled: bool,
}

/// OAuth2 Client info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    pub id: String,
    pub client_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub client_type: String,
    pub redirect_uris: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

/// Create client request
#[derive(Clone, Debug, Serialize)]
pub struct CreateClientRequest {
    pub client_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub client_type: String,
    pub redirect_uris: Vec<String>,
    pub enabled: bool,
}

/// Role info
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub user_count: Option<u64>,
    pub created_at: String,
}

/// Create role request
#[derive(Clone, Debug, Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// Federation identity provider
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityProviderInfo {
    pub id: String,
    pub alias: String,
    pub display_name: Option<String>,
    pub provider_type: String,
    pub enabled: bool,
    pub created_at: String,
}

/// Create IdP request
#[derive(Clone, Debug, Serialize)]
pub struct CreateIdentityProviderRequest {
    pub alias: String,
    pub display_name: Option<String>,
    pub provider_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
}

/// Group info from IAM API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupInfo {
    pub id: String,
    pub realm_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub attributes: serde_json::Value,
    pub member_count: i64,
    pub subgroup_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Create group request
#[derive(Clone, Debug, Serialize)]
pub struct CreateGroupApiRequest {
    pub realm_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

/// Audit log entry
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub details: Option<String>,
    pub realm: Option<String>,
    pub timestamp: String,
    pub success: bool,
}

/// Audit log query params
#[derive(Clone, Debug, Serialize)]
pub struct AuditLogQuery {
    pub event_type: Option<String>,
    pub user_id: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// Admin statistics
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminStats {
    pub total_users: u64,
    pub active_users: u64,
    pub total_sessions: u64,
    pub total_realms: u64,
    pub total_clients: u64,
    pub mfa_enabled_users: u64,
    pub recent_login_count: u64,
    pub failed_login_count: u64,
}

/// Paginated response wrapper
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Generic API error
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status: Option<u16>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ─── API Client ──────────────────────────────────────────────────────────────

/// Centralized Authenc API Client
///
/// Provides type-safe, grouped methods for all Authenc REST API endpoints.
/// Communicates directly with authenc-api and authenc-iam-api without layanan-portal proxy.
#[derive(Clone, Debug)]
pub struct AuthencApiClient {
    base_url: String,
}

impl Default for AuthencApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthencApiClient {
    /// Create a new API client using the current origin
    pub fn new() -> Self {
        let base_url = {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::window()
                    .and_then(|w| w.location().origin().ok())
                    .unwrap_or_else(|| "http://localhost:3000".to_string())
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                "http://localhost:3000".to_string()
            }
        };
        Self { base_url }
    }

    /// Create client with custom base URL
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Get the stored JWT token from localStorage
    fn get_token() -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("simpel_access_token").ok().flatten())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    // ─── Internal HTTP helpers ───────────────────────────────────────────────

    #[cfg(target_arch = "wasm32")]
    async fn get(&self, path: &str) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = gloo_net::http::Request::get(&url);
        if let Some(token) = Self::get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        req.send()
            .await
            .map_err(|e| format!("Network error: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req =
            gloo_net::http::Request::post(&url).header("Content-Type", "application/json");
        if let Some(token) = Self::get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        req.json(body)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    async fn put<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = gloo_net::http::Request::put(&url).header("Content-Type", "application/json");
        if let Some(token) = Self::get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        req.json(body)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    async fn patch<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req =
            gloo_net::http::Request::patch(&url).header("Content-Type", "application/json");
        if let Some(token) = Self::get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        req.json(body)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    async fn delete(&self, path: &str) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = gloo_net::http::Request::delete(&url);
        if let Some(token) = Self::get_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        req.send()
            .await
            .map_err(|e| format!("Network error: {}", e))
    }

    /// Parse a JSON response or return error
    #[cfg(target_arch = "wasm32")]
    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: gloo_net::http::Response,
    ) -> Result<T, String> {
        if resp.ok() {
            resp.json::<T>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {}: {}", status, body))
        }
    }

    /// Check if response is successful (for empty-body responses)
    #[cfg(target_arch = "wasm32")]
    async fn check_ok(resp: gloo_net::http::Response) -> Result<(), String> {
        if resp.ok() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {}: {}", status, body))
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Authentication endpoints (/api/v1/auth/)
    // ═════════════════════════════════════════════════════════════════════════

    /// Login with username/password
    #[cfg(target_arch = "wasm32")]
    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, String> {
        let resp = self.post("/api/v1/auth/login", req).await?;
        Self::parse_response(resp).await
    }

    /// Logout (invalidate session)
    #[cfg(target_arch = "wasm32")]
    pub async fn logout(&self) -> Result<(), String> {
        let resp = self
            .post("/api/v1/auth/logout", &serde_json::json!({}))
            .await?;
        Self::check_ok(resp).await
    }

    /// Refresh access token
    #[cfg(target_arch = "wasm32")]
    pub async fn refresh_token(&self, req: &RefreshTokenRequest) -> Result<TokenResponse, String> {
        let resp = self.post("/api/v1/auth/refresh", req).await?;
        Self::parse_response(resp).await
    }

    /// Get current user info
    #[cfg(target_arch = "wasm32")]
    pub async fn get_me(&self) -> Result<UserInfo, String> {
        let resp = self.get("/api/v1/auth/me").await?;
        Self::parse_response(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // WebAuthn/Passkey endpoints (/api/v1/auth/webauthn/)
    // ═════════════════════════════════════════════════════════════════════════

    /// Start passkey registration — returns WebAuthn options + session ID
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_register_start(&self) -> Result<WebAuthnStartResponse, String> {
        let resp = self
            .post(
                "/api/v1/auth/webauthn/register/start",
                &serde_json::json!({}),
            )
            .await?;
        Self::parse_response(resp).await
    }

    /// Finish passkey registration — sends session_id + browser credential
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_register_finish(
        &self,
        session_id: &str,
        credential: &serde_json::Value,
    ) -> Result<PasskeyInfo, String> {
        let body = WebAuthnFinishRequest {
            session_id: session_id.to_string(),
            credential: credential.clone(),
        };
        let resp = self
            .post("/api/v1/auth/webauthn/register/finish", &body)
            .await?;
        Self::parse_response(resp).await
    }

    /// Start passkey authentication — returns WebAuthn options + session ID
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_authenticate_start(&self) -> Result<WebAuthnStartResponse, String> {
        let resp = self
            .post(
                "/api/v1/auth/webauthn/authenticate/start",
                &serde_json::json!({}),
            )
            .await?;
        Self::parse_response(resp).await
    }

    /// Finish passkey authentication — sends session_id + browser assertion
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_authenticate_finish(
        &self,
        session_id: &str,
        assertion: &serde_json::Value,
    ) -> Result<TokenResponse, String> {
        let body = WebAuthnFinishRequest {
            session_id: session_id.to_string(),
            credential: assertion.clone(),
        };
        let resp = self
            .post("/api/v1/auth/webauthn/authenticate/finish", &body)
            .await?;
        Self::parse_response(resp).await
    }

    /// List registered passkeys
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_list_credentials(&self) -> Result<Vec<PasskeyInfo>, String> {
        let resp = self.get("/api/v1/auth/webauthn/credentials").await?;
        Self::parse_response(resp).await
    }

    /// Delete a passkey
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_delete_credential(&self, id: &str) -> Result<(), String> {
        let resp = self
            .delete(&format!("/api/v1/auth/webauthn/credentials/{}", id))
            .await?;
        Self::check_ok(resp).await
    }

    /// Update a passkey nickname
    #[cfg(target_arch = "wasm32")]
    pub async fn webauthn_update_credential(
        &self,
        id: &str,
        req: &UpdatePasskeyRequest,
    ) -> Result<(), String> {
        let resp = self
            .patch(&format!("/api/v1/auth/webauthn/credentials/{}", id), req)
            .await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // TOTP / MFA endpoints (/api/v1/auth/totp/)
    // ═════════════════════════════════════════════════════════════════════════

    /// Enable TOTP MFA
    #[cfg(target_arch = "wasm32")]
    pub async fn totp_enable(&self) -> Result<TotpEnableResponse, String> {
        let resp = self
            .post("/api/v1/auth/totp/enable", &serde_json::json!({}))
            .await?;
        Self::parse_response(resp).await
    }

    /// Disable TOTP MFA
    #[cfg(target_arch = "wasm32")]
    pub async fn totp_disable(&self) -> Result<(), String> {
        let resp = self
            .post("/api/v1/auth/totp/disable", &serde_json::json!({}))
            .await?;
        Self::check_ok(resp).await
    }

    /// Verify TOTP code
    #[cfg(target_arch = "wasm32")]
    pub async fn totp_verify(&self, req: &TotpVerifyRequest) -> Result<TokenResponse, String> {
        let resp = self.post("/api/v1/auth/totp/verify", req).await?;
        Self::parse_response(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Profile & Self-Service endpoints
    // ═════════════════════════════════════════════════════════════════════════

    /// Update user profile
    #[cfg(target_arch = "wasm32")]
    pub async fn update_profile(&self, req: &UpdateProfileRequest) -> Result<UserInfo, String> {
        let resp = self.put("/api/v1/auth/me", req).await?;
        Self::parse_response(resp).await
    }

    /// Change password
    #[cfg(target_arch = "wasm32")]
    pub async fn change_password(&self, req: &ChangePasswordRequest) -> Result<(), String> {
        let resp = self.post("/api/v1/auth/me/password", req).await?;
        Self::check_ok(resp).await
    }

    /// Request password reset
    #[cfg(target_arch = "wasm32")]
    pub async fn request_password_reset(&self, req: &PasswordResetRequest) -> Result<(), String> {
        let resp = self.post("/api/v1/auth/password/reset", req).await?;
        Self::check_ok(resp).await
    }

    /// Confirm password reset
    #[cfg(target_arch = "wasm32")]
    pub async fn confirm_password_reset(
        &self,
        req: &PasswordResetConfirmRequest,
    ) -> Result<(), String> {
        let resp = self
            .post("/api/v1/auth/password/reset/confirm", req)
            .await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Session management endpoints
    // ═════════════════════════════════════════════════════════════════════════

    /// List active sessions
    #[cfg(target_arch = "wasm32")]
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        let resp = self.get("/api/v1/iam/sessions").await?;
        Self::parse_response(resp).await
    }

    /// Terminate a specific session
    #[cfg(target_arch = "wasm32")]
    pub async fn terminate_session(&self, session_id: &str) -> Result<(), String> {
        let resp = self
            .delete(&format!("/api/v1/iam/sessions/{}", session_id))
            .await?;
        Self::check_ok(resp).await
    }

    /// Terminate all sessions except current
    #[cfg(target_arch = "wasm32")]
    pub async fn terminate_all_sessions(&self) -> Result<(), String> {
        let resp = self
            .post("/api/v1/iam/sessions/terminate-all", &serde_json::json!({}))
            .await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: User Management (/api/v1/iam/users/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List users
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_users(
        &self,
        page: u32,
        per_page: u32,
        search: Option<&str>,
    ) -> Result<PaginatedResponse<IamUser>, String> {
        let mut path = format!("/api/v1/iam/users?page={}&per_page={}", page, per_page);
        if let Some(q) = search {
            path.push_str(&format!("&search={}", urlencoding::encode(q)));
        }
        let resp = self.get(&path).await?;
        Self::parse_response(resp).await
    }

    /// Create user
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_user(&self, req: &CreateUserRequest) -> Result<IamUser, String> {
        let resp = self.post("/api/v1/iam/users", req).await?;
        Self::parse_response(resp).await
    }

    /// Get user by ID
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_get_user(&self, id: &str) -> Result<IamUser, String> {
        let resp = self.get(&format!("/api/v1/iam/users/{}", id)).await?;
        Self::parse_response(resp).await
    }

    /// Get client by ID
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_get_client(&self, id: &str) -> Result<ClientInfo, String> {
        let resp = self.get(&format!("/api/v1/iam/clients/{}", id)).await?;
        Self::parse_response(resp).await
    }

    /// Update user
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_update_user(
        &self,
        id: &str,
        req: &UpdateUserRequest,
    ) -> Result<IamUser, String> {
        let resp = self.put(&format!("/api/v1/iam/users/{}", id), req).await?;
        Self::parse_response(resp).await
    }

    /// Delete user
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_user(&self, id: &str) -> Result<(), String> {
        let resp = self.delete(&format!("/api/v1/iam/users/{}", id)).await?;
        Self::check_ok(resp).await
    }

    /// Reset user password (admin)
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_reset_user_password(&self, id: &str) -> Result<(), String> {
        let resp = self
            .post(
                &format!("/api/v1/iam/users/{}/password/reset", id),
                &serde_json::json!({}),
            )
            .await?;
        Self::check_ok(resp).await
    }

    /// Assign role to user
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_assign_role(&self, user_id: &str, role_id: &str) -> Result<(), String> {
        let resp = self
            .post(
                &format!("/api/v1/iam/users/{}/roles/{}", user_id, role_id),
                &serde_json::json!({}),
            )
            .await?;
        Self::check_ok(resp).await
    }

    /// Remove role from user
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_remove_role(&self, user_id: &str, role_id: &str) -> Result<(), String> {
        let resp = self
            .delete(&format!("/api/v1/iam/users/{}/roles/{}", user_id, role_id))
            .await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Realm Management (/api/v1/iam/realms/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List realms
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_realms(&self) -> Result<Vec<RealmInfo>, String> {
        let resp = self.get("/api/v1/iam/realms").await?;
        Self::parse_response(resp).await
    }

    /// Create realm
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_realm(&self, req: &CreateRealmRequest) -> Result<RealmInfo, String> {
        let resp = self.post("/api/v1/iam/realms", req).await?;
        Self::parse_response(resp).await
    }

    /// Delete realm
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_realm(&self, id: &str) -> Result<(), String> {
        let resp = self.delete(&format!("/api/v1/iam/realms/{}", id)).await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Client Management (/api/v1/iam/clients/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List OAuth2 clients
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_clients(&self) -> Result<Vec<ClientInfo>, String> {
        let resp = self.get("/api/v1/iam/clients").await?;
        Self::parse_response(resp).await
    }

    /// Create OAuth2 client
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_client(&self, req: &CreateClientRequest) -> Result<ClientInfo, String> {
        let resp = self.post("/api/v1/iam/clients", req).await?;
        Self::parse_response(resp).await
    }

    /// Delete OAuth2 client
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_client(&self, id: &str) -> Result<(), String> {
        let resp = self.delete(&format!("/api/v1/iam/clients/{}", id)).await?;
        Self::check_ok(resp).await
    }

    /// Regenerate client secret
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_regenerate_client_secret(
        &self,
        id: &str,
    ) -> Result<serde_json::Value, String> {
        let resp = self
            .post(
                &format!("/api/v1/iam/clients/{}/secret/regenerate", id),
                &serde_json::json!({}),
            )
            .await?;
        Self::parse_response(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Role Management (/api/v1/iam/roles/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List roles
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_roles(&self) -> Result<Vec<RoleInfo>, String> {
        let resp = self.get("/api/v1/iam/roles").await?;
        Self::parse_response(resp).await
    }

    /// Create role
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_role(&self, req: &CreateRoleRequest) -> Result<RoleInfo, String> {
        let resp = self.post("/api/v1/iam/roles", req).await?;
        Self::parse_response(resp).await
    }

    /// Delete role
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_role(&self, id: &str) -> Result<(), String> {
        let resp = self.delete(&format!("/api/v1/iam/roles/{}", id)).await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Federation / Identity Providers (/api/v1/iam/identity-providers/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List identity providers
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_identity_providers(&self) -> Result<Vec<IdentityProviderInfo>, String> {
        let resp = self.get("/api/v1/iam/identity-providers").await?;
        Self::parse_response(resp).await
    }

    /// Create identity provider
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_identity_provider(
        &self,
        req: &CreateIdentityProviderRequest,
    ) -> Result<IdentityProviderInfo, String> {
        let resp = self.post("/api/v1/iam/identity-providers", req).await?;
        Self::parse_response(resp).await
    }

    /// Delete identity provider
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_identity_provider(&self, id: &str) -> Result<(), String> {
        let resp = self
            .delete(&format!("/api/v1/iam/identity-providers/{}", id))
            .await?;
        Self::check_ok(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Audit Logs (/api/v1/iam/audit-logs/)
    // ═════════════════════════════════════════════════════════════════════════

    /// Query audit logs
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_query_audit_logs(
        &self,
        query: &AuditLogQuery,
    ) -> Result<PaginatedResponse<AuditLogEntry>, String> {
        let mut path = String::from("/api/v1/iam/audit-logs?");
        if let Some(ref et) = query.event_type {
            path.push_str(&format!("event_type={}&", urlencoding::encode(et)));
        }
        if let Some(ref uid) = query.user_id {
            path.push_str(&format!("user_id={}&", urlencoding::encode(uid)));
        }
        if let Some(ref from) = query.from_date {
            path.push_str(&format!("from_date={}&", urlencoding::encode(from)));
        }
        if let Some(ref to) = query.to_date {
            path.push_str(&format!("to_date={}&", urlencoding::encode(to)));
        }
        if let Some(p) = query.page {
            path.push_str(&format!("page={}&", p));
        }
        if let Some(pp) = query.per_page {
            path.push_str(&format!("per_page={}&", pp));
        }
        // Remove trailing &
        if path.ends_with('&') {
            path.pop();
        }
        let resp = self.get(&path).await?;
        Self::parse_response(resp).await
    }

    /// Export audit logs (returns CSV/JSON text)
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_export_audit_logs(&self, format: &str) -> Result<String, String> {
        let resp = self
            .get(&format!("/api/v1/iam/audit-logs/export?format={}", format))
            .await?;
        if resp.ok() {
            resp.text().await.map_err(|e| format!("Read error: {}", e))
        } else {
            let status = resp.status();
            Err(format!("HTTP {}", status))
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Dashboard & Stats (/api/v1/iam/admin/)
    // ═════════════════════════════════════════════════════════════════════════

    /// Get admin statistics
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_admin_stats(&self) -> Result<AdminStats, String> {
        let resp = self.get("/api/v1/iam/admin/stats").await?;
        Self::parse_response(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // IAM Admin: Group Management (/api/v1/iam/groups/)
    // ═════════════════════════════════════════════════════════════════════════

    /// List groups
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_list_groups(&self) -> Result<Vec<GroupInfo>, String> {
        let resp = self.get("/api/v1/iam/groups").await?;
        Self::parse_response(resp).await
    }

    /// Create group
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_create_group(&self, req: &CreateGroupApiRequest) -> Result<GroupInfo, String> {
        let resp = self.post("/api/v1/iam/groups", req).await?;
        Self::parse_response(resp).await
    }

    /// Get group by ID
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_get_group(&self, id: &str) -> Result<GroupInfo, String> {
        let resp = self.get(&format!("/api/v1/iam/groups/{}", id)).await?;
        Self::parse_response(resp).await
    }

    /// Delete group
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_delete_group(&self, id: &str) -> Result<(), String> {
        let resp = self.delete(&format!("/api/v1/iam/groups/{}", id)).await?;
        Self::check_ok(resp).await
    }

    /// Get group members
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_get_group_members(&self, id: &str) -> Result<Vec<String>, String> {
        let resp = self.get(&format!("/api/v1/iam/groups/{}/members", id)).await?;
        Self::parse_response(resp).await
    }

    /// Get subgroups
    #[cfg(target_arch = "wasm32")]
    pub async fn iam_get_subgroups(&self, id: &str) -> Result<Vec<GroupInfo>, String> {
        let resp = self.get(&format!("/api/v1/iam/groups/{}/subgroups", id)).await?;
        Self::parse_response(resp).await
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Non-WASM stubs (for compilation in non-WASM targets)
    // ═════════════════════════════════════════════════════════════════════════

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn login(&self, _req: &LoginRequest) -> Result<LoginResponse, String> {
        Ok(LoginResponse {
            access_token: Some("mock_token".to_string()),
            temp_token: None,
            refresh_token: Some("mock_refresh".to_string()),
            token_type: Some("Bearer".to_string()),
            expires_in: Some(3600),
            mfa_required: false,
            mfa_setup_required: false,
            message: "Mock login".to_string(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn logout(&self) -> Result<(), String> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn refresh_token(&self, _req: &RefreshTokenRequest) -> Result<TokenResponse, String> {
        Ok(TokenResponse {
            access_token: "mock_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("mock_refresh".to_string()),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_me(&self) -> Result<UserInfo, String> {
        Ok(UserInfo {
            id: "mock-id".to_string(),
            username: "admin".to_string(),
            email: "admin@kejaksaan.go.id".to_string(),
            name: Some("Admin User".to_string()),
            first_name: Some("Admin".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            avatar: None,
            division: Some("IT".to_string()),
            role: "admin".to_string(),
            permissions: vec!["admin:*".to_string()],
            mfa_enabled: false,
            email_verified: true,
            created_at: None,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_register_start(&self) -> Result<WebAuthnStartResponse, String> {
        Ok(WebAuthnStartResponse {
            challenge: serde_json::json!({}),
            session_id: "mock-session".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_register_finish(
        &self,
        _sid: &str,
        _c: &serde_json::Value,
    ) -> Result<PasskeyInfo, String> {
        Ok(PasskeyInfo {
            id: "mock".to_string(),
            nickname: None,
            credential_type: "public-key".to_string(),
            created_at: "2026-01-01".to_string(),
            last_used: None,
            aaguid: None,
            is_platform: true,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_authenticate_start(&self) -> Result<WebAuthnStartResponse, String> {
        Ok(WebAuthnStartResponse {
            challenge: serde_json::json!({}),
            session_id: "mock-session".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_authenticate_finish(
        &self,
        _sid: &str,
        _a: &serde_json::Value,
    ) -> Result<TokenResponse, String> {
        Ok(TokenResponse {
            access_token: "mock".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: None,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_list_credentials(&self) -> Result<Vec<PasskeyInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_delete_credential(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn webauthn_update_credential(
        &self,
        _id: &str,
        _r: &UpdatePasskeyRequest,
    ) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn totp_enable(&self) -> Result<TotpEnableResponse, String> {
        Ok(TotpEnableResponse {
            secret: "mock".to_string(),
            qr_code_url: "mock".to_string(),
            backup_codes: vec![],
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn totp_disable(&self) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn totp_verify(&self, _r: &TotpVerifyRequest) -> Result<TokenResponse, String> {
        Ok(TokenResponse {
            access_token: "mock".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: None,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn update_profile(&self, _r: &UpdateProfileRequest) -> Result<UserInfo, String> {
        self.get_me().await
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn change_password(&self, _r: &ChangePasswordRequest) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn request_password_reset(&self, _r: &PasswordResetRequest) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn confirm_password_reset(
        &self,
        _r: &PasswordResetConfirmRequest,
    ) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn terminate_session(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn terminate_all_sessions(&self) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_users(
        &self,
        _p: u32,
        _pp: u32,
        _s: Option<&str>,
    ) -> Result<PaginatedResponse<IamUser>, String> {
        Ok(PaginatedResponse {
            data: vec![],
            total: 0,
            page: 1,
            per_page: 20,
            total_pages: 0,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_user(&self, _r: &CreateUserRequest) -> Result<IamUser, String> {
        Ok(IamUser {
            id: "mock".to_string(),
            username: "mock".to_string(),
            email: "mock@test.com".to_string(),
            first_name: None,
            last_name: None,
            enabled: true,
            email_verified: false,
            mfa_enabled: false,
            created_at: "2026-01-01".to_string(),
            last_login: None,
            roles: vec![],
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_get_user(&self, _id: &str) -> Result<IamUser, String> {
        self.iam_create_user(&CreateUserRequest {
            username: "mock".to_string(),
            email: "mock".to_string(),
            password: "mock".to_string(),
            first_name: None,
            last_name: None,
            enabled: true,
        })
        .await
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_update_user(
        &self,
        _id: &str,
        _r: &UpdateUserRequest,
    ) -> Result<IamUser, String> {
        self.iam_get_user("mock").await
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_user(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_reset_user_password(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_assign_role(&self, _uid: &str, _rid: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_remove_role(&self, _uid: &str, _rid: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_realms(&self) -> Result<Vec<RealmInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_realm(&self, _r: &CreateRealmRequest) -> Result<RealmInfo, String> {
        Ok(RealmInfo {
            id: "mock".to_string(),
            name: "mock".to_string(),
            display_name: None,
            enabled: true,
            user_count: None,
            created_at: "2026-01-01".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_realm(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_clients(&self) -> Result<Vec<ClientInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_client(&self, _r: &CreateClientRequest) -> Result<ClientInfo, String> {
        Ok(ClientInfo {
            id: "mock".to_string(),
            client_id: "mock".to_string(),
            name: None,
            description: None,
            client_type: "public".to_string(),
            redirect_uris: vec![],
            enabled: true,
            created_at: "2026-01-01".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_client(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_regenerate_client_secret(
        &self,
        _id: &str,
    ) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"secret": "new_mock_secret"}))
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_roles(&self) -> Result<Vec<RoleInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_role(&self, _r: &CreateRoleRequest) -> Result<RoleInfo, String> {
        Ok(RoleInfo {
            id: "mock".to_string(),
            name: "mock".to_string(),
            description: None,
            permissions: vec![],
            user_count: None,
            created_at: "2026-01-01".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_role(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_get_client(&self, id: &str) -> Result<ClientInfo, String> {
        Ok(ClientInfo {
            id: id.to_string(),
            client_id: "mock_client".to_string(),
            name: None,
            description: None,
            client_type: "public".to_string(),
            redirect_uris: vec![],
            enabled: true,
            created_at: "2026-01-01".to_string(),
        })
    }


    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_identity_providers(&self) -> Result<Vec<IdentityProviderInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_identity_provider(
        &self,
        _r: &CreateIdentityProviderRequest,
    ) -> Result<IdentityProviderInfo, String> {
        Ok(IdentityProviderInfo {
            id: "mock".to_string(),
            alias: "mock".to_string(),
            display_name: None,
            provider_type: "oidc".to_string(),
            enabled: true,
            created_at: "2026-01-01".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_identity_provider(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_query_audit_logs(
        &self,
        _q: &AuditLogQuery,
    ) -> Result<PaginatedResponse<AuditLogEntry>, String> {
        Ok(PaginatedResponse {
            data: vec![],
            total: 0,
            page: 1,
            per_page: 20,
            total_pages: 0,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_export_audit_logs(&self, _f: &str) -> Result<String, String> {
        Ok(String::new())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_admin_stats(&self) -> Result<AdminStats, String> {
        Ok(AdminStats {
            total_users: 0,
            active_users: 0,
            total_sessions: 0,
            total_realms: 0,
            total_clients: 0,
            mfa_enabled_users: 0,
            recent_login_count: 0,
            failed_login_count: 0,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_list_groups(&self) -> Result<Vec<GroupInfo>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_create_group(&self, _r: &CreateGroupApiRequest) -> Result<GroupInfo, String> {
        Ok(GroupInfo {
            id: "mock".to_string(),
            realm_id: "mock".to_string(),
            name: "mock".to_string(),
            parent_id: None,
            description: None,
            attributes: serde_json::json!({}),
            member_count: 0,
            subgroup_count: 0,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_get_group(&self, _id: &str) -> Result<GroupInfo, String> {
        self.iam_create_group(&CreateGroupApiRequest {
            realm_id: "mock".to_string(),
            name: "mock".to_string(),
            parent_id: None,
            description: None,
        }).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_delete_group(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_get_group_members(&self, _id: &str) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn iam_get_subgroups(&self, _id: &str) -> Result<Vec<GroupInfo>, String> {
        Ok(vec![])
    }
}
