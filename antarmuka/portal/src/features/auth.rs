//! Authentication types and service
//!
//! Provides authentication state management and user session handling

use serde::{Deserialize, Serialize};

/// Authentication state
#[derive(Clone, Debug, PartialEq)]
pub enum AuthState {
    /// User is not authenticated
    LoggedOut,
    /// User is authenticated
    LoggedIn,
    /// Authentication in progress
    Authenticating,
}

/// User session data
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserSession {
    /// User's unique identifier
    pub id: String,
    /// User's login username
    pub username: String,
    /// User's role in the system
    pub role: UserRole,
    /// User's display name
    pub name: String,
    /// User's email
    pub email: String,
    /// Profile picture URL
    pub avatar: Option<String>,
    /// NIP (Nomor Induk Pegawai)
    pub nip: Option<String>,
    /// Jabatan (position/title)
    pub jabatan: Option<String>,
    /// Kode Satker (work unit code)
    pub satker_code: Option<String>,
    /// Nama satuan kerja
    pub satuan_kerja: String,
    /// CAPTCHA validation status
    pub captcha_validated: bool,
    /// MFA enabled status
    pub mfa_enabled: bool,
    /// MFA setup required (true if user needs to setup MFA)
    pub mfa_setup_required: bool,
    /// Whether user must change password before using the system
    #[serde(default)]
    pub require_password_change: bool,
    /// Session creation timestamp
    pub created_at: Option<String>,
    /// JWT access token
    pub access_token: Option<String>,
    /// JWT refresh token
    pub refresh_token: Option<String>,
    /// Token expiration timestamp (Unix timestamp)
    pub expires_at: Option<i64>,
    /// User permissions
    pub permissions: Vec<String>,
}

pub use lib_core::auth::UserRole;

/// Login credentials
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// CAPTCHA token (optional, required after failed attempts)
    pub captcha_token: Option<String>,
}

/// Login result
#[derive(Clone, Debug)]
pub enum LoginResult {
    /// Login successful with session
    Success(Box<UserSession>),
    /// MFA setup required - contains temp token
    MfaSetupRequired(String), // temp_token
    /// MFA verification required - contains temp token
    MfaVerificationRequired(String), // temp_token
    /// Password change required before using the system
    PasswordChangeRequired(Box<UserSession>),
    /// Login failed with error message
    Error(String),
}

/// OAuth2 token response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Token expiration time in seconds
    pub expires_in: u64,
    /// Optional refresh token for token renewal
    pub refresh_token: Option<String>,
}

/// Login response from authenc (with MFA support)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Access token (only present after full authentication)
    pub access_token: Option<String>,
    /// Refresh token (only present after full authentication)
    pub refresh_token: Option<String>,
    /// Temporary token (present when MFA verification needed)
    pub temp_token: Option<String>,
    /// Whether MFA verification is required
    pub mfa_required: bool,
    /// Whether MFA setup is required
    pub mfa_setup_required: bool,
    /// Whether password change is required
    #[serde(default)]
    pub require_password_change: bool,
    /// Response message
    pub message: String,
}

/// Authentication service
pub struct AuthService;

impl AuthService {
    /// Get authenc API base URL from window.location.origin (same-origin pattern).
    /// This ensures the URL always matches the user's access URL, avoiding
    /// cross-origin or mixed-content issues from hardcoded config values.
    #[allow(dead_code)]
    fn get_api_url() -> String {
        #[cfg(target_arch = "wasm32")]
        {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());
            format!("{}/api/v1/auth", origin)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let config = crate::utils::config::get_config();
            config.authenc_url
        }
    }

    /// Validate login credentials via authenc API
    ///
    /// Calls authenc /api/auth/login endpoint with MFA support
    /// Returns appropriate LoginResult based on MFA status
    pub async fn login(credentials: LoginCredentials) -> LoginResult {
        // Validate input
        if credentials.username.trim().is_empty() {
            return LoginResult::Error("Username tidak boleh kosong".to_string());
        }

        if credentials.password.is_empty() {
            return LoginResult::Error("Password tidak boleh kosong".to_string());
        }

        // Call authenc login endpoint with MFA support
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            // api_url already includes /api/auth prefix
            let login_url = format!("{}/login", api_url);

            // Prepare JSON request body
            let body = serde_json::json!({
                "username": credentials.username,
                "password": credentials.password,
                "realm": "master",
                "captcha_token": credentials.captcha_token,
            });

            let request = match Request::post(&login_url)
                .header("Content-Type", "application/json")
                .json(&body)
            {
                Ok(req) => req,
                Err(e) => return LoginResult::Error(format!("Failed to build request: {}", e)),
            };

            match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        // Parse login response with MFA support
                        match response.json::<LoginResponse>().await {
                            Ok(login_resp) => {
                                // Check MFA status and return appropriate result
                                if login_resp.mfa_setup_required {
                                    // User needs to setup MFA
                                    if let Some(temp_token) = login_resp.temp_token {
                                        LoginResult::MfaSetupRequired(temp_token)
                                    } else {
                                        LoginResult::Error(
                                            "MFA setup required but no temp token provided"
                                                .to_string(),
                                        )
                                    }
                                } else if login_resp.mfa_required {
                                    // User needs to verify MFA
                                    if let Some(temp_token) = login_resp.temp_token {
                                        LoginResult::MfaVerificationRequired(temp_token)
                                    } else {
                                        LoginResult::Error(
                                            "MFA verification required but no temp token provided"
                                                .to_string(),
                                        )
                                    }
                                } else if let Some(access_token) =
                                    login_resp.access_token.filter(|t| !t.is_empty())
                                {
                                    // Full authentication complete
                                    Self::save_token(&access_token);
                                    if let Some(refresh_token) = &login_resp.refresh_token {
                                        Self::save_refresh_token(refresh_token);
                                    }

                                    // Decode JWT to extract user info
                                    match Self::decode_jwt_claims(&access_token) {
                                        Ok(mut session) => {
                                            if let Some(refresh) = &login_resp.refresh_token {
                                                session.refresh_token = Some(refresh.clone());
                                            }
                                            Self::save_session(&session);
                                            if login_resp.require_password_change {
                                                LoginResult::PasswordChangeRequired(Box::new(
                                                    session,
                                                ))
                                            } else {
                                                LoginResult::Success(Box::new(session))
                                            }
                                        }
                                        Err(e) => LoginResult::Error(format!(
                                            "Failed to decode token: {}",
                                            e
                                        )),
                                    }
                                } else {
                                    LoginResult::Error(
                                        "Invalid login response: no token or MFA status"
                                            .to_string(),
                                    )
                                }
                            }
                            Err(e) => {
                                LoginResult::Error(format!("Failed to parse response: {}", e))
                            }
                        }
                    } else {
                        let status = response.status();
                        // Parse error response JSON to extract user-friendly message
                        let error_msg = match response.json::<serde_json::Value>().await {
                            Ok(json) => json
                                .get("message")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    if status == 401 {
                                        "Username atau password salah".to_string()
                                    } else {
                                        "Terjadi kesalahan sistem. Silakan coba lagi nanti."
                                            .to_string()
                                    }
                                }),
                            Err(_) => {
                                if status == 401 {
                                    "Username atau password salah".to_string()
                                } else {
                                    "Terjadi kesalahan sistem. Silakan coba lagi nanti.".to_string()
                                }
                            }
                        };
                        LoginResult::Error(error_msg)
                    }
                }
                Err(e) => LoginResult::Error(format!("Network error: {}", e)),
            }
        }

        // Fallback for non-WASM (server-side rendering, testing)
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::create_mock_session(credentials)
        }
    }

    /// Create mock session for testing (non-WASM only)
    #[cfg(not(target_arch = "wasm32"))]
    fn create_mock_session(credentials: LoginCredentials) -> LoginResult {
        // Mock MFA flow based on username patterns for testing
        // - username ending with "_nomfa": Skip MFA (direct login)
        // - username ending with "_setup": Require MFA setup
        // - username ending with "_verify": Require MFA verification
        // - default: Skip MFA for demo simplicity

        let username_lower = credentials.username.to_lowercase();

        if username_lower.ends_with("_setup") {
            // Simulate MFA setup required
            return LoginResult::MfaSetupRequired("mock_temp_token_setup".to_string());
        } else if username_lower.ends_with("_verify") {
            // Simulate MFA verification required
            return LoginResult::MfaVerificationRequired("mock_temp_token_verify".to_string());
        }

        // Default: Create full session (no MFA for demo)
        let role = if credentials.username == "admin" {
            UserRole::Admin
        } else if credentials.username.starts_with("super") {
            UserRole::Supervisor
        } else {
            UserRole::User
        };

        // Determine permissions based on role's capabilities (not hardcoded variants)
        let permissions = if role.is_admin() {
            vec![
                "admin:*".to_string(),
                "user:read".to_string(),
                "user:write".to_string(),
            ]
        } else if matches!(role, UserRole::Supervisor) {
            vec!["user:read".to_string(), "user:write".to_string()]
        } else if matches!(role, UserRole::Guest) {
            vec![]
        } else {
            vec!["user:read".to_string()]
        };

        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::hours(8);

        let session = UserSession {
            id: uuid::Uuid::new_v4().to_string(),
            username: credentials.username.clone(),
            role,
            name: Self::generate_display_name(&credentials.username),
            email: format!("{}@kejaksaan.go.id", credentials.username),
            avatar: None,
            nip: Some(credentials.username.clone()),
            jabatan: Some("Kasubag Perlengkapan".to_string()),
            satker_code: Some("0100000".to_string()),
            satuan_kerja: "0100000".to_string(),
            captcha_validated: credentials.captcha_token.is_some(),
            mfa_enabled: username_lower.ends_with("_verify"), // MFA enabled if verification was required
            mfa_setup_required: false,                        // Setup complete in mock
            require_password_change: false,
            created_at: Some(now.to_rfc3339()),
            access_token: Some("mock_access_token".to_string()),
            refresh_token: Some("mock_refresh_token".to_string()),
            expires_at: Some(expires_at.timestamp()),
            permissions,
        };

        LoginResult::Success(Box::new(session))
    }

    /// Decode JWT token to extract user claims
    pub fn decode_jwt_claims(token: &str) -> Result<UserSession, String> {
        // Note: For WASM we just do basic base64 decode of payload since we don't have the secret
        // Full verification happens on the backend.
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }

        // JWT uses URL-safe Base64 encoding without padding
        use lib_core::encoding::base64_decode_url;
        let payload_bytes =
            base64_decode_url(parts[1]).map_err(|e| format!("Base64 decode error: {}", e))?;

        let payload_str =
            String::from_utf8(payload_bytes).map_err(|e| format!("UTF-8 decode error: {}", e))?;

        use lib_core::jwt_claims::Claims;
        let claims: Claims =
            serde_json::from_str(&payload_str).map_err(|e| format!("JSON parse error: {}", e))?;

        // Map roles
        let role = claims.get_primary_role();
        let username = claims.preferred_username.unwrap_or(claims.sub.clone());
        let permissions = claims.realm_access.map(|ra| ra.roles).unwrap_or_default();

        // Use the raw satker code as the satuan_kerja fallback.  The profile
        // page resolves the human-readable name via the API's
        // `resolve_satuan_kerja` helper; here we only have JWT claims.
        let satuan_kerja = match claims.satker_code.as_deref() {
            Some(code) if !code.is_empty() => code.to_string(),
            _ => String::new(),
        };

        Ok(UserSession {
            id: claims.sub,
            username: username.clone(),
            role,
            name: claims
                .name
                .unwrap_or_else(|| Self::generate_display_name(&username)),
            email: claims
                .email
                .unwrap_or_else(|| format!("{}@kejaksaan.go.id", username)),
            avatar: None,
            nip: claims.nip.clone(),
            jabatan: claims.jabatan.clone(),
            satker_code: claims.satker_code.clone(),
            satuan_kerja,
            captcha_validated: true,
            mfa_enabled: claims.mfa_enabled,
            mfa_setup_required: claims.mfa_setup_required,
            require_password_change: claims.require_password_change,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some(token.to_string()),
            refresh_token: None,
            expires_at: Some(claims.exp as i64),
            permissions,
        })
    }

    /// Save authentication token to localStorage
    pub fn save_token(token: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.set_item("auth_token", token);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = token; // Suppress unused warning
        }
    }

    /// Get stored authentication token
    pub fn get_token() -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|storage| storage.get_item("auth_token").ok())
                .flatten()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    /// Save refresh token to localStorage
    pub fn save_refresh_token(token: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.set_item("refresh_token", token);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = token;
        }
    }

    /// Save temporary token to localStorage (for MFA flow)
    pub fn save_temp_token(token: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.set_item("temp_token", token);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = token; // Suppress unused warning
        }
    }

    /// Get stored temporary token
    pub fn get_temp_token() -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|storage| storage.get_item("temp_token").ok())
                .flatten()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    /// Clear temporary token from localStorage
    pub fn clear_temp_token() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("temp_token");
            }
        }
    }

    /// Remember a deep-link redirect target across MFA/password-change
    /// detours. Stored in sessionStorage so it's scoped to the tab and
    /// cleared automatically when the tab closes. Only `/perlengkapan/…`
    /// paths are accepted to prevent open-redirect abuse.
    pub fn save_post_login_redirect(target: &str) {
        if !target.starts_with("/perlengkapan") {}
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.session_storage().ok())
                .flatten()
            {
                let _ = storage.set_item("post_login_redirect", target);
            }
        }
    }

    /// Consume any stored post-login redirect, clearing it so it fires once.
    pub fn take_post_login_redirect() -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage = web_sys::window()
                .and_then(|w| w.session_storage().ok())
                .flatten()?;
            let value = storage.get_item("post_login_redirect").ok().flatten()?;
            let _ = storage.remove_item("post_login_redirect");
            if value.starts_with("/perlengkapan") {
                Some(value)
            } else {
                None
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    /// Logout user - calls backend and clears local state
    pub fn logout() {
        // 1. Read refresh token BEFORE clearing localStorage
        #[cfg(target_arch = "wasm32")]
        let saved_refresh_token = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("refresh_token").ok().flatten())
            .unwrap_or_default();

        // 2. Clear localStorage immediately for responsive UI
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("user_session");
                let _ = storage.remove_item("auth_token");
                let _ = storage.remove_item("refresh_token");
                // Clear Perlengkapan token as well (Global Logout)
                let _ = storage.remove_item("jwt_token");
            }
        }

        // 3. Call backend logout endpoint asynchronously
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;

            spawn_local(async move {
                let origin = web_sys::window()
                    .and_then(|w| w.location().origin().ok())
                    .unwrap_or_else(|| "http://localhost:8080".to_string());

                // Use the standard auth logout endpoint which is routed
                // through Istio to authenc via the /api/v1/auth prefix.
                let logout_url = format!("{}/api/v1/auth/logout", origin);

                let refresh_token = saved_refresh_token;

                // POST to /api/v1/auth/logout with the refresh token
                if let Some(window) = web_sys::window() {
                    use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode};

                    let body = serde_json::json!({ "refresh_token": refresh_token }).to_string();

                    let headers = Headers::new().unwrap();
                    let _ = headers.set("Content-Type", "application/json");

                    let opts = RequestInit::new();
                    opts.set_method("POST");
                    opts.set_mode(RequestMode::Cors);
                    opts.set_credentials(RequestCredentials::Include);
                    opts.set_headers(&headers);
                    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

                    if let Ok(request) = Request::new_with_str_and_init(&logout_url, &opts) {
                        let _ = wasm_bindgen_futures::JsFuture::from(
                            window.fetch_with_request(&request),
                        )
                        .await;
                    }
                }
            });
        }
    }

    /// Load session from localStorage
    pub fn load_session() -> Option<UserSession> {
        #[cfg(target_arch = "wasm32")]
        {
            use lib_ui::utils::storage::load_from_storage;
            load_from_storage("user_session")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    /// Save session to localStorage
    pub fn save_session(session: &UserSession) {
        #[cfg(target_arch = "wasm32")]
        {
            use lib_ui::utils::storage::save_to_storage;
            let _ = save_to_storage("user_session", session);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = session; // Suppress unused warning in non-WASM
        }
    }

    /// Update existing session with MFA enabled
    pub fn update_session_mfa_enabled() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(mut session) = Self::load_session() {
                session.mfa_enabled = true;
                session.mfa_setup_required = false;
                Self::save_session(&session);
            }
        }
    }

    /// Generate display name from username
    fn generate_display_name(username: &str) -> String {
        // Capitalize first letter
        username
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().to_string()
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    /// Check if session is valid (not expired)
    pub fn is_session_valid(session: &UserSession) -> bool {
        if let Some(expires_at) = session.expires_at {
            let now = chrono::Utc::now().timestamp();
            expires_at > now
        } else {
            // If no expiration, consider valid
            true
        }
    }

    /// Check if token needs refresh (expires in less than 5 minutes)
    pub fn should_refresh_token(session: &UserSession) -> bool {
        if let Some(expires_at) = session.expires_at {
            let now = chrono::Utc::now().timestamp();
            let time_until_expiry = expires_at - now;
            time_until_expiry < 300 // Less than 5 minutes
        } else {
            false
        }
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(_refresh_token: &str) -> Result<TokenResponse, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            if api_url == "demo" || api_url == "mock" || api_url.is_empty() {
                // Mock refresh for demo
                return Ok(TokenResponse {
                    access_token: "mock_refreshed_access_token".to_string(),
                    token_type: "Bearer".to_string(),
                    expires_in: 28800, // 8 hours
                    refresh_token: Some(_refresh_token.to_string()),
                });
            }

            let token_url = format!("{}/refresh", api_url);

            let form_data = format!("{{\"refresh_token\":\"{}\"}}", _refresh_token);

            let request = Request::post(&token_url)
                .header("Content-Type", "application/json")
                .body(form_data)
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                response
                    .json::<TokenResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            } else {
                Err(format!("Token refresh failed: HTTP {}", response.status()))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Err("Token refresh not available in non-WASM environment".to_string())
        }
    }

    /// Update session with new token
    ///
    /// Re-decodes the JWT to refresh claim-derived fields (e.g.
    /// `require_password_change`, `nip`, `jabatan`, `roles`) so that
    /// route guards and UI components see up-to-date values after an
    /// automatic token refresh.
    pub fn update_session_token(token_response: &TokenResponse) {
        if let Some(mut session) = Self::load_session() {
            session.access_token = Some(token_response.access_token.clone());
            session.refresh_token = token_response.refresh_token.clone();

            // Calculate new expiration time
            let now = chrono::Utc::now();
            let expires_at = now + chrono::Duration::seconds(token_response.expires_in as i64);
            session.expires_at = Some(expires_at.timestamp());

            // Re-decode JWT claims so that claim-derived fields (name,
            // nip, jabatan, satker_code, require_password_change, roles,
            // mfa_enabled, etc.) are refreshed from the new token.
            // Without this, a stale `require_password_change: true` would
            // keep redirecting the user to the password-change page even
            // after the flag was cleared in the DB and the new JWT.
            if let Ok(decoded) = Self::decode_jwt_claims(&token_response.access_token) {
                session.name = decoded.name;
                session.email = decoded.email;
                session.role = decoded.role;
                session.nip = decoded.nip;
                session.jabatan = decoded.jabatan;
                session.satker_code = decoded.satker_code;
                session.satuan_kerja = decoded.satuan_kerja;
                session.mfa_enabled = decoded.mfa_enabled;
                session.mfa_setup_required = decoded.mfa_setup_required;
                session.require_password_change = decoded.require_password_change;
                session.permissions = decoded.permissions;
            }

            Self::save_session(&session);

            #[cfg(target_arch = "wasm32")]
            Self::save_token(&token_response.access_token);
        }
    }

    /// Validate JWT token structure (basic validation)
    pub fn validate_token_structure(token: &str) -> bool {
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3
    }

    /// Check if user has specific permission
    pub fn has_permission(session: &UserSession, permission: &str) -> bool {
        session.permissions.iter().any(|p| {
            p == permission || p.ends_with(":*") && permission.starts_with(&p[..p.len() - 1])
        })
    }

    /// Broadcast logout event to all tabs/windows
    #[cfg(target_arch = "wasm32")]
    pub fn broadcast_logout() {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            // Set a logout flag that other tabs can detect
            let _ = storage.set_item("logout_event", &chrono::Utc::now().timestamp().to_string());
            // Remove it immediately (the storage event will still fire)
            let _ = storage.remove_item("logout_event");
        }
    }

    /// Setup storage event listener for cross-tab session sync
    #[cfg(target_arch = "wasm32")]
    pub fn setup_storage_listener<F>(on_session_change: F)
    where
        F: Fn(Option<UserSession>) + 'static,
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        let closure = Closure::wrap(Box::new(move |event: web_sys::StorageEvent| {
            if let Some(key) = event.key() {
                match key.as_str() {
                    "user_session" => {
                        // Session changed in another tab
                        let session = Self::load_session();
                        on_session_change(session);
                    }
                    "logout_event" => {
                        // Logout triggered in another tab
                        on_session_change(None);
                    }
                    _ => {}
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref());
        }

        closure.forget(); // Keep the closure alive
    }

    // ========== MFA Methods ==========

    /// Setup MFA for the current user
    /// Calls authenc /api/auth/mfa/setup endpoint
    /// Returns MFA setup data including QR code and secret key
    pub async fn setup_mfa() -> Result<MfaSetupData, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            // api_url already includes /api/auth prefix
            let setup_url = format!("{}/mfa/setup", api_url);

            // Get temp token for authentication
            let token = Self::get_temp_token()
                .or_else(Self::get_token)
                .ok_or_else(|| "No authentication token available".to_string())?;

            let request = Request::post(&setup_url)
                .header("Authorization", &format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .build()
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                response
                    .json::<MfaSetupData>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "MFA setup failed: HTTP {} - {}",
                    status, error_text
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Err("MFA setup not available in non-WASM environment".to_string())
        }
    }

    /// Verify MFA setup with initial OTP code
    /// Calls authenc /api/auth/mfa/verify-setup endpoint
    pub async fn verify_mfa_setup(code: &str) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            // api_url already includes /api/auth prefix
            let verify_url = format!("{}/mfa/verify-setup", api_url);

            // Get temp token for authentication
            let token = Self::get_temp_token()
                .or_else(Self::get_token)
                .ok_or_else(|| "No authentication token available".to_string())?;

            let body = serde_json::json!({
                "code": code
            });

            let request = Request::post(&verify_url)
                .header("Authorization", &format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&body)
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                Ok(())
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "MFA setup verification failed: HTTP {} - {}",
                    status, error_text
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = code;
            Err("MFA setup verification not available in non-WASM environment".to_string())
        }
    }

    /// Verify MFA code during login
    /// Calls authenc /api/auth/mfa/verify endpoint
    /// Returns access token on successful verification
    pub async fn verify_mfa(code: &str) -> Result<String, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            // api_url already includes /api/auth prefix
            let verify_url = format!("{}/mfa/verify", api_url);

            // Get temp token for authentication
            let token =
                Self::get_temp_token().ok_or_else(|| "No temporary token available".to_string())?;

            let body = serde_json::json!({
                "code": code
            });

            let request = Request::post(&verify_url)
                .header("Authorization", &format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&body)
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                #[derive(Deserialize)]
                struct MfaVerifyResponse {
                    access_token: String,
                }

                let verify_response = response
                    .json::<MfaVerifyResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                Ok(verify_response.access_token)
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "MFA verification failed: HTTP {} - {}",
                    status, error_text
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = code;
            Err("MFA verification not available in non-WASM environment".to_string())
        }
    }

    /// Get MFA status for the current user
    /// Calls authenc /api/auth/mfa/status endpoint
    pub async fn get_mfa_status() -> Result<MfaStatus, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let api_url = Self::get_api_url();
            // api_url already includes /api/auth prefix
            let status_url = format!("{}/mfa/status", api_url);

            // Get token for authentication
            let token =
                Self::get_token().ok_or_else(|| "No authentication token available".to_string())?;

            let request = Request::get(&status_url)
                .header("Authorization", &format!("Bearer {}", token))
                .build()
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                response
                    .json::<MfaStatus>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "Failed to get MFA status: HTTP {} - {}",
                    status, error_text
                ))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Err("MFA status check not available in non-WASM environment".to_string())
        }
    }

    /// Update session with MFA state tracking
    /// Updates mfa_enabled, mfa_setup_required, and mfa_verification_required flags
    pub fn update_session_mfa_state(
        mfa_enabled: bool,
        mfa_setup_required: bool,
        _mfa_verification_required: bool,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(mut session) = Self::load_session() {
                session.mfa_enabled = mfa_enabled;
                session.mfa_setup_required = mfa_setup_required;
                // Note: mfa_verification_required is not stored in UserSession
                // It's a transient state during login flow
                Self::save_session(&session);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (mfa_enabled, mfa_setup_required, _mfa_verification_required);
        }
    }
}

/// MFA setup data returned from authenc
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MfaSetupData {
    /// QR code as data URL (data:image/png;base64,...)
    pub qr_code_url: String,
    /// Secret key for manual entry
    pub secret_key: String,
    /// Backup codes for recovery
    pub backup_codes: Vec<String>,
}

/// MFA status information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MfaStatus {
    /// Whether MFA is enabled for the user
    pub enabled: bool,
    /// When MFA was set up (ISO 8601 timestamp)
    pub setup_at: Option<String>,
    /// Number of remaining backup codes
    pub backup_codes_remaining: i32,
    /// Last time MFA was used (ISO 8601 timestamp)
    pub last_used: Option<String>,
}

// Internal structures for JWT parsing removed - using lib_core::jwt::Claims
