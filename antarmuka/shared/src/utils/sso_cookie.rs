//! SSO Cookie Management for Frontend
//!
//! Provides utilities to read and validate SSO cookies set by Authenc backend.
//! This module is designed to work in sync with the backend SSO cookie implementation.

use leptos::prelude::Set;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{HtmlDocument, window};

/// SSO session data (matches backend SsoSession structure)
///
/// This structure mirrors the backend implementation in:
/// `infra/authenc/src/utils/sso_cookie.rs`
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SsoSession {
    /// Session ID (UUID)
    pub session_id: String,

    /// User ID
    pub user_id: String,

    /// Username
    pub username: String,

    /// User email
    pub email: Option<String>,

    /// User roles
    pub roles: Vec<String>,

    /// Session creation timestamp (ISO 8601)
    pub created_at: String,

    /// Session expiration timestamp (ISO 8601)
    pub expires_at: String,

    /// Client IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,
}

impl SsoSession {
    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        // Parse expires_at timestamp
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            let now = chrono::Utc::now();
            expires.with_timezone(&chrono::Utc) < now
        } else {
            // If parsing fails, consider expired for safety
            true
        }
    }

    /// Check if the session is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// SSO Cookie Reader for frontend
///
/// Reads and validates SSO cookies set by the Authenc backend.
/// Cookie name: AUTHENC_SSO (configurable in backend)
pub struct SsoCookieReader {
    cookie_name: String,
}

impl SsoCookieReader {
    /// Create a new SSO cookie reader with default cookie name
    pub fn new() -> Self {
        Self {
            cookie_name: "AUTHENC_SSO".to_string(),
        }
    }

    /// Create a new SSO cookie reader with custom cookie name
    pub fn with_name(cookie_name: String) -> Self {
        Self { cookie_name }
    }

    /// Read SSO session from cookie
    ///
    /// Returns None if:
    /// - Cookie doesn't exist
    /// - Cookie value is invalid
    /// - Session is expired
    pub fn read_session(&self) -> Option<SsoSession> {
        let cookie_value = self.get_cookie_value()?;

        // Decode base64
        let decoded = self.decode_base64(&cookie_value).ok()?;

        // Parse JSON
        let session: SsoSession = serde_json::from_str(&decoded).ok()?;

        // Validate session
        if session.is_valid() {
            Some(session)
        } else {
            None
        }
    }

    /// Check if SSO session exists and is valid
    pub fn has_valid_session(&self) -> bool {
        self.read_session().is_some()
    }

    /// Get cookie value from browser
    fn get_cookie_value(&self) -> Option<String> {
        let window = window()?;
        let document = window.document()?;
        let html_doc = document.dyn_ref::<HtmlDocument>()?;

        let cookies = html_doc.cookie().ok()?;

        // Parse cookies
        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix(&format!("{}=", self.cookie_name)) {
                return Some(value.to_string());
            }
        }

        None
    }

    /// Decode base64 string
    fn decode_base64(&self, encoded: &str) -> Result<String, String> {
        use base64::{Engine as _, engine::general_purpose};

        let decoded_bytes = general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        String::from_utf8(decoded_bytes).map_err(|e| format!("UTF-8 decode error: {}", e))
    }
}

impl Default for SsoCookieReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert SsoSession to UserSession (for use_auth hook)
///
/// This function bridges the SSO cookie data with the existing
/// authentication system in shared microfrontend.
impl From<SsoSession> for crate::hooks::use_auth::UserSession {
    fn from(sso: SsoSession) -> Self {
        use crate::hooks::use_auth::UserRole;

        // Determine role from roles array
        let role = if sso.has_role("admin") {
            UserRole::Admin
        } else if sso.has_role("supervisor") {
            UserRole::Supervisor
        } else if sso.has_role("guest") {
            UserRole::Guest
        } else {
            UserRole::User
        };

        // Parse expiration timestamp
        let expires_at = chrono::DateTime::parse_from_rfc3339(&sso.expires_at)
            .ok()
            .map(|dt| dt.timestamp());

        Self {
            id: sso.user_id.clone(),
            username: sso.username.clone(),
            role,
            name: sso.username.clone(), // Use username as display name
            email: sso.email.unwrap_or_default(),
            avatar: None,
            division: String::new(), // Not available in SSO session
            captcha_validated: true, // Assume validated if SSO session exists
            mfa_enabled: false,      // Not available in SSO session
            mfa_setup_required: false,
            created_at: Some(sso.created_at.clone()),
            access_token: None, // Not stored in cookie for security
            refresh_token: None,
            expires_at,
            permissions: sso.roles.clone(), // Use roles as permissions
        }
    }
}

/// Initialize authentication from SSO cookie
///
/// This function should be called during app initialization to
/// restore session from SSO cookie if available.
///
/// # Example
/// ```rust
/// use shared_microfrontend::utils::sso_cookie::init_auth_from_sso_cookie;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     // Initialize auth from SSO cookie on mount
///     create_effect(move |_| {
///         init_auth_from_sso_cookie();
///     });
///
///     view! {
///         // Your app content
///     }
/// }
/// ```
pub fn init_auth_from_sso_cookie() {
    use crate::hooks::use_auth::use_auth;

    let auth = use_auth();

    // Check if already authenticated
    if auth.is_authenticated() {
        return;
    }

    // Try to read SSO session from cookie
    let reader = SsoCookieReader::new();
    if let Some(sso_session) = reader.read_session() {
        // Convert to UserSession and set in auth context
        let user_session = crate::hooks::use_auth::UserSession::from(sso_session);

        // Also save to localStorage for persistence
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
            if let Ok(json) = serde_json::to_string(&user_session) {
                let _ = storage.set_item("user_session", &json);
            }
        }

        // Set in auth context (after localStorage to avoid move issue)
        auth.session.set(Some(user_session));
    }
}

/// Check SSO session validity periodically
///
/// This function sets up a periodic check to validate SSO session
/// and logout if session expires.
///
/// # Example
/// ```rust
/// use shared_microfrontend::utils::sso_cookie::setup_sso_session_monitor;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     // Setup SSO session monitoring on mount
///     create_effect(move |_| {
///         setup_sso_session_monitor(60000); // Check every 60 seconds
///     });
///
///     view! {
///         // Your app content
///     }
/// }
/// ```
pub fn setup_sso_session_monitor(interval_ms: i32) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let callback = Closure::wrap(Box::new(move || {
        let reader = SsoCookieReader::new();

        // Check if SSO session is still valid
        if !reader.has_valid_session() {
            // Session expired or invalid, logout
            use crate::hooks::use_auth::use_auth;
            let auth = use_auth();
            auth.logout();
        }
    }) as Box<dyn FnMut()>);

    if let Some(window) = window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            interval_ms,
        );
    }

    callback.forget(); // Keep the closure alive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_session_validation() {
        let now = chrono::Utc::now();
        let future = now + chrono::Duration::hours(1);
        let past = now - chrono::Duration::hours(1);

        // Valid session
        let valid_session = SsoSession {
            session_id: "test-session-id".to_string(),
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["user".to_string()],
            created_at: now.to_rfc3339(),
            expires_at: future.to_rfc3339(),
            ip_address: None,
            user_agent: None,
        };

        assert!(valid_session.is_valid());
        assert!(!valid_session.is_expired());

        // Expired session
        let expired_session = SsoSession {
            expires_at: past.to_rfc3339(),
            ..valid_session.clone()
        };

        assert!(!expired_session.is_valid());
        assert!(expired_session.is_expired());
    }

    #[test]
    fn test_sso_session_role_check() {
        let session = SsoSession {
            session_id: "test-session-id".to_string(),
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["user".to_string(), "admin".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            ip_address: None,
            user_agent: None,
        };

        assert!(session.has_role("user"));
        assert!(session.has_role("admin"));
        assert!(!session.has_role("supervisor"));
    }

    #[test]
    fn test_sso_to_user_session_conversion() {
        let sso_session = SsoSession {
            session_id: "test-session-id".to_string(),
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["admin".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            ip_address: None,
            user_agent: None,
        };

        let user_session = crate::hooks::use_auth::UserSession::from(sso_session.clone());

        assert_eq!(user_session.id, sso_session.user_id);
        assert_eq!(user_session.username, sso_session.username);
        assert_eq!(user_session.email, sso_session.email.unwrap());
        assert!(user_session.role.is_admin());
    }
}
