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
    /// User's division/unit
    pub division: String,
    /// CAPTCHA validation status
    pub captcha_validated: bool,
    /// MFA enabled status
    pub mfa_enabled: bool,
    /// MFA setup required (true if user needs to setup MFA)
    pub mfa_setup_required: bool,
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

/// User role enum
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum UserRole {
    /// System administrator
    Admin,
    /// Regular user
    #[default]
    User,
    /// Supervisor
    Supervisor,
    /// Guest (read-only)
    Guest,
}

impl UserRole {
    /// Get role display name in Indonesian
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Admin => "Administrator",
            Self::User => "Pengguna",
            Self::Supervisor => "Supervisor",
            Self::Guest => "Tamu",
        }
    }

    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    /// Check if role can manage users
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin | Self::Supervisor)
    }
}

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
    Success(UserSession),
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

/// Authentication service
pub struct AuthService;

impl AuthService {
    /// Get authenc API base URL from environment or default
    fn get_api_url() -> String {
        // In production, this comes from environment variable or config
        // For development, default to localhost:3000 (authenc default port)
        std::env::var("AUTHENC_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
    }

    /// Validate login credentials via authenc API
    ///
    /// Calls authenc /realms/{realm}/protocol/openid-connect/token endpoint
    /// Returns JWT access token and user session on success
    pub async fn login(credentials: LoginCredentials) -> LoginResult {
        // Validate input
        if credentials.username.trim().is_empty() {
            return LoginResult::Error("Username tidak boleh kosong".to_string());
        }

        if credentials.password.is_empty() {
            return LoginResult::Error("Password tidak boleh kosong".to_string());
        }

        // Check for demo/mock mode
        let api_url = Self::get_api_url();
        if api_url == "demo" || api_url == "mock" || api_url.is_empty() {
            return Self::create_mock_session(credentials);
        }

        // Call authenc token endpoint
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;
            use web_sys::RequestMode;

            let api_url = Self::get_api_url();
            let token_url = format!("{}/realms/simpel/protocol/openid-connect/token", api_url);

            // Prepare form data for token request
            let mut form_data = format!(
                "grant_type=password&client_id=portal&username={}&password={}",
                urlencoding::encode(&credentials.username),
                urlencoding::encode(&credentials.password)
            );

            // Add CAPTCHA token if provided
            if let Some(captcha_token) = &credentials.captcha_token {
                form_data.push_str(&format!(
                    "&captcha_token={}",
                    urlencoding::encode(captcha_token)
                ));
            }

            let request = match Request::post(&token_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(form_data)
            {
                Ok(req) => req,
                Err(e) => return LoginResult::Error(format!("Failed to build request: {}", e)),
            };

            match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        // Parse token response
                        match response.json::<TokenResponse>().await {
                            Ok(token_resp) => {
                                // Store token
                                Self::save_token(&token_resp.access_token);

                                // Decode JWT to extract user info
                                match Self::decode_jwt_claims(&token_resp.access_token) {
                                    Ok(session) => {
                                        Self::save_session(&session);
                                        LoginResult::Success(session)
                                    }
                                    Err(e) => {
                                        LoginResult::Error(format!("Failed to decode token: {}", e))
                                    }
                                }
                            }
                            Err(e) => {
                                LoginResult::Error(format!("Failed to parse response: {}", e))
                            }
                        }
                    } else {
                        let status = response.status();
                        LoginResult::Error(format!("Login failed: HTTP {}", status))
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

    /// Create mock session for demo/testing
    fn create_mock_session(credentials: LoginCredentials) -> LoginResult {
        let role = if credentials.username == "admin" {
            UserRole::Admin
        } else if credentials.username.starts_with("super") {
            UserRole::Supervisor
        } else {
            UserRole::User
        };

        let permissions = match role {
            UserRole::Admin => vec![
                "admin:*".to_string(),
                "user:read".to_string(),
                "user:write".to_string(),
            ],
            UserRole::Supervisor => vec!["user:read".to_string(), "user:write".to_string()],
            UserRole::User => vec!["user:read".to_string()],
            UserRole::Guest => vec![],
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
            division: "Bagian Umum".to_string(),
            captcha_validated: credentials.captcha_token.is_some(),
            mfa_enabled: false,       // Mock: assume MFA not enabled initially
            mfa_setup_required: true, // Mock: require MFA setup for all users
            created_at: Some(now.to_rfc3339()),
            access_token: Some("mock_access_token".to_string()),
            refresh_token: Some("mock_refresh_token".to_string()),
            expires_at: Some(expires_at.timestamp()),
            permissions,
        };

        LoginResult::Success(session)
    }

    /// Decode JWT token to extract user claims
    #[cfg(target_arch = "wasm32")]
    pub fn decode_jwt_claims(token: &str) -> Result<UserSession, String> {
        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }

        // Decode base64 payload (part[1])
        use base64::{Engine as _, engine::general_purpose};
        let payload_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        let payload_str =
            String::from_utf8(payload_bytes).map_err(|e| format!("UTF-8 decode error: {}", e))?;

        // Parse JSON claims
        #[derive(Deserialize)]
        struct Claims {
            sub: String,
            preferred_username: Option<String>,
            name: Option<String>,
            email: Option<String>,
            realm_access: Option<RealmAccess>,
        }

        #[derive(Deserialize)]
        struct RealmAccess {
            roles: Vec<String>,
        }

        let claims: Claims =
            serde_json::from_str(&payload_str).map_err(|e| format!("JSON parse error: {}", e))?;

        // Map roles
        let role = claims
            .realm_access
            .as_ref()
            .and_then(|ra| {
                if ra.roles.contains(&"admin".to_string()) {
                    Some(UserRole::Admin)
                } else if ra.roles.contains(&"supervisor".to_string()) {
                    Some(UserRole::Supervisor)
                } else {
                    None
                }
            })
            .unwrap_or(UserRole::User);

        let username = claims.preferred_username.unwrap_or(claims.sub.clone());

        let permissions = claims.realm_access.map(|ra| ra.roles).unwrap_or_default();

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
            division: "Bagian Umum".to_string(),
            captcha_validated: true, // JWT tokens from authenc indicate successful CAPTCHA validation
            mfa_enabled: false,      // TODO: Extract from JWT claims when available
            mfa_setup_required: true, // TODO: Extract from JWT claims when available
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some(token.to_string()),
            refresh_token: None, // Will be set separately if available
            expires_at: None,    // TODO: Extract exp claim from JWT
            permissions,
        })
    }

    /// Save authentication token to localStorage
    #[cfg(target_arch = "wasm32")]
    fn save_token(token: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("auth_token", token);
        }
    }

    /// Get stored authentication token
    #[cfg(target_arch = "wasm32")]
    pub fn get_token() -> Option<String> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|storage| storage.get_item("auth_token").ok())
            .flatten()
    }

    /// Logout user
    pub fn logout() {
        // Clear localStorage
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("user_session");
                let _ = storage.remove_item("auth_token");
            }
        }
    }

    /// Load session from localStorage
    pub fn load_session() -> Option<UserSession> {
        #[cfg(target_arch = "wasm32")]
        {
            use shared_microfrontend::hooks::load_from_storage;
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
            use shared_microfrontend::hooks::save_to_storage;
            save_to_storage("user_session", session);
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

            let token_url = format!("{}/realms/simpel/protocol/openid-connect/token", api_url);

            let form_data = format!(
                "grant_type=refresh_token&client_id=portal&refresh_token={}",
                urlencoding::encode(_refresh_token)
            );

            let request = Request::post(&token_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
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
    pub fn update_session_token(token_response: &TokenResponse) {
        if let Some(mut session) = Self::load_session() {
            session.access_token = Some(token_response.access_token.clone());
            session.refresh_token = token_response.refresh_token.clone();

            // Calculate new expiration time
            let now = chrono::Utc::now();
            let expires_at = now + chrono::Duration::seconds(token_response.expires_in as i64);
            session.expires_at = Some(expires_at.timestamp());

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
}
