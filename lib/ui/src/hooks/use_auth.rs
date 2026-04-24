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
    /// NIP (Nomor Induk Pegawai)
    #[serde(default)]
    pub nip: Option<String>,
    /// Jabatan (position/title)
    #[serde(default)]
    pub jabatan: Option<String>,
    /// Kode Satker (work unit code)
    #[serde(default)]
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
        // Read refresh token BEFORE clearing localStorage
        #[cfg(target_arch = "wasm32")]
        let saved_refresh_token = window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("refresh_token").ok().flatten())
            .unwrap_or_default();

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
                let origin = window()
                    .and_then(|w| w.location().origin().ok())
                    .unwrap_or_else(|| "http://localhost:8080".to_string());

                let logout_url = format!("{}/api/v1/auth/logout", origin);

                let refresh_token = saved_refresh_token;

                // POST to /api/v1/auth/logout with the refresh token
                if let Some(window) = window() {
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
/// use lib_ui::hooks::use_auth::use_auth;
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
    use crate::utils::storage::load_from_storage;
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
    // For development, default to current origin + /portal
    std::env::var("PORTAL_URL").unwrap_or_else(|_| {
        if let Some(window) = window() {
            format!(
                "{}/portal",
                window
                    .location()
                    .origin()
                    .unwrap_or_else(|_| "http://localhost:8080".to_string())
            )
        } else {
            "http://localhost:8080".to_string()
        }
    })
}

/// Get current microfrontend app name
pub fn get_app_name() -> String {
    // Try environment variable (SSR or build-time injection)
    if let Ok(name) = std::env::var("APP_NAME") {
        return name;
    }

    // Try global config object on window (CSR injection)
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = window() {
        if let Ok(value) = js_sys::Reflect::get(&window, &JsValue::from_str("__SIMPEL_CONFIG")) {
            if !value.is_undefined() && !value.is_null() {
                if let Ok(name) = js_sys::Reflect::get(&value, &JsValue::from_str("appName")) {
                    if let Some(s) = name.as_string() {
                        return s;
                    }
                }
            }
        }
    }

    "Microfrontend".to_string()
}

/// Get current microfrontend app description
pub fn get_app_description() -> String {
    // Try environment variable
    if let Ok(desc) = std::env::var("APP_DESCRIPTION") {
        return desc;
    }

    // Try global config object on window
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = window() {
        if let Ok(value) = js_sys::Reflect::get(&window, &JsValue::from_str("__SIMPEL_CONFIG")) {
            if !value.is_undefined() && !value.is_null() {
                if let Ok(desc) = js_sys::Reflect::get(&value, &JsValue::from_str("appDescription"))
                {
                    if let Some(s) = desc.as_string() {
                        return s;
                    }
                }
            }
        }
    }

    "Sistem Informasi Manajemen Perkara Elektronik".to_string()
}
