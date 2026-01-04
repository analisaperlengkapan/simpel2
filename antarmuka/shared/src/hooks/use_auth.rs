//! Authentication hook for microfrontends
//!
//! Provides session management and authentication state for microfrontend applications

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// User session data (matches portal UserSession structure)
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

impl UserSession {
    /// Check if session is valid (not expired)
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = chrono::Utc::now().timestamp();
            expires_at > now
        } else {
            // If no expiration, consider valid
            true
        }
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| {
            p == permission || p.ends_with(":*") && permission.starts_with(&p[..p.len() - 1])
        })
    }
}

/// Authentication context for microfrontends
#[derive(Clone, Copy)]
pub struct AuthContext {
    /// Current user session (None if not authenticated)
    pub session: RwSignal<Option<UserSession>>,
}

impl AuthContext {
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.session.with(|s| s.is_some())
    }

    /// Get current user session
    pub fn get_session(&self) -> Option<UserSession> {
        self.session.get()
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.session.with(|s| {
            s.as_ref()
                .map(|session| session.has_permission(permission))
                .unwrap_or(false)
        })
    }

    /// Check if user has admin role
    pub fn is_admin(&self) -> bool {
        self.session.with(|s| {
            s.as_ref()
                .map(|session| session.role.is_admin())
                .unwrap_or(false)
        })
    }

    /// Logout user and redirect to portal
    pub fn logout(&self) {
        // Clear session immediately for responsive UI
        self.session.set(None);

        // Clear localStorage
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
            let _ = storage.remove_item("user_session");
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("refresh_token");
        }

        // Call backend logout endpoint
        #[cfg(target_arch = "wasm32")]
        {
            spawn_local(async move {
                let authenc_url = std::env::var("AUTHENC_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string());

                let portal_url = get_portal_url();
                let redirect_uri = format!("{}/logged-out", portal_url);

                let logout_url = format!(
                    "{}/v1/oidc/logout?post_logout_redirect_uri={}",
                    authenc_url,
                    urlencoding::encode(&redirect_uri)
                );

                // Make request with credentials to include SSO cookie
                if let Some(window) = window() {
                    use wasm_bindgen::JsValue;
                    use web_sys::{Request, RequestCredentials, RequestInit, RequestMode};

                    let mut opts = RequestInit::new();
                    opts.method("GET");
                    opts.mode(RequestMode::Cors);
                    opts.credentials(RequestCredentials::Include);

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

    /// Redirect to portal login with return URL
    pub fn redirect_to_login(&self) {
        let portal_url = get_portal_url();
        if let Some(window) = window() {
            // Get current URL for return_url
            if let Ok(current_url) = window.location().href() {
                let login_url = format!(
                    "{}/login?return_url={}",
                    portal_url,
                    urlencoding::encode(&current_url)
                );
                let _ = window.location().set_href(&login_url);
            }
        }
    }
}

/// Hook to access authentication context
///
/// # Example
/// ```rust
/// use shared_microfrontend::hooks::use_auth::use_auth;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn MyComponent() -> impl IntoView {
///     let auth = use_auth();
///
///     view! {
///         <Show when=move || auth.is_authenticated()>
///             <p>"Welcome, " {move || auth.get_session().map(|s| s.name).unwrap_or_default()}</p>
///         </Show>
///     }
/// }
/// ```
pub fn use_auth() -> AuthContext {
    // Try to get existing context
    if let Some(ctx) = use_context::<AuthContext>() {
        return ctx;
    }

    // Create new context if not exists
    let session = RwSignal::new(load_session_from_storage());

    // Setup storage event listener for cross-tab sync
    setup_storage_listener(session);

    let ctx = AuthContext { session };

    // Provide context for child components
    provide_context(ctx);

    ctx
}

/// Load session from localStorage
fn load_session_from_storage() -> Option<UserSession> {
    use crate::hooks::load_from_storage;
    load_from_storage("user_session")
}

/// Setup storage event listener for cross-tab session synchronization
fn setup_storage_listener(session: RwSignal<Option<UserSession>>) {
    let closure = Closure::wrap(Box::new(move |event: web_sys::StorageEvent| {
        if let Some(key) = event.key() {
            match key.as_str() {
                "user_session" => {
                    // Session changed in another tab
                    let new_session = load_session_from_storage();
                    session.set(new_session);
                }
                "logout_event" => {
                    // Logout triggered in another tab
                    session.set(None);
                }
                _ => {}
            }
        }
    }) as Box<dyn FnMut(_)>);

    if let Some(window) = window() {
        let _ =
            window.add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref());
    }

    closure.forget(); // Keep the closure alive
}

/// Get portal URL from environment or default
pub fn get_portal_url() -> String {
    // In production, this comes from environment variable or config
    // For development, default to localhost
    std::env::var("PORTAL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Get current microfrontend app name from environment
pub fn get_app_name() -> String {
    std::env::var("APP_NAME").unwrap_or_else(|_| "Microfrontend".to_string())
}
