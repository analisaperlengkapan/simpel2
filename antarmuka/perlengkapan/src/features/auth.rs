use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserSession {
    pub username: String,
    pub role: String,
    pub access_token: Option<String>,
}

impl UserSession {
    pub fn is_admin(&self) -> bool {
        let role = self.role.to_ascii_lowercase();
        role == "admin" || role == "super_admin" || role.starts_with("admin_")
    }
}

pub struct AuthService;

impl AuthService {
    const SESSION_KEY: &'static str = "perlengkapan_user_session";
    const PORTAL_SESSION_KEY: &'static str = "user_session";
    const AUTH_TOKEN_KEY: &'static str = "auth_token";
    const REFRESH_TOKEN_KEY: &'static str = "refresh_token";
    const ACTIVE_ROLE_KEY: &'static str = "active_role";
    const LOGOUT_EVENT_KEY: &'static str = "logout_event";

    pub fn load_session() -> Option<UserSession> {
        let storage = web_sys::window()?.local_storage().ok().flatten()?;

        if let Some(raw_session) = storage.get_item(Self::SESSION_KEY).ok().flatten() {
            if let Ok(session) = serde_json::from_str::<UserSession>(&raw_session) {
                return Some(session);
            }
        }

        // Compatibility fallback: read portal session payload if available.
        if let Some(raw_portal_session) = storage.get_item(Self::PORTAL_SESSION_KEY).ok().flatten()
        {
            let token = storage.get_item(Self::AUTH_TOKEN_KEY).ok().flatten();
            let role = storage
                .get_item(Self::ACTIVE_ROLE_KEY)
                .ok()
                .flatten()
                .unwrap_or_else(|| "operator_satker".to_string());

            if let Some(session) = Self::from_portal_session_payload(&raw_portal_session, role, token)
            {
                return Some(session);
            }
        }

        // Fallback: bootstrap lightweight session from legacy keys.
        let token = storage.get_item(Self::AUTH_TOKEN_KEY).ok().flatten();
        token.as_ref()?;

        let role = storage
            .get_item(Self::ACTIVE_ROLE_KEY)
            .ok()
            .flatten()
            .unwrap_or_else(|| "operator_satker".to_string());

        Some(UserSession {
            username: "Pengguna".to_string(),
            role,
            access_token: token,
        })
    }

    fn from_portal_session_payload(
        raw: &str,
        fallback_role: String,
        fallback_token: Option<String>,
    ) -> Option<UserSession> {
        let value: serde_json::Value = serde_json::from_str(raw).ok()?;

        let username = value
            .get("username")
            .and_then(|v| v.as_str())
            .or_else(|| value.get("name").and_then(|v| v.as_str()))
            .unwrap_or("Pengguna")
            .to_string();

        let role = value
            .get("role")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .unwrap_or(fallback_role);

        let access_token = value
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .or(fallback_token);

        Some(UserSession {
            username,
            role,
            access_token,
        })
    }

    pub fn save_session(session: &UserSession) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(serialized) = serde_json::to_string(session) {
                let _ = storage.set_item(Self::SESSION_KEY, &serialized);
            }
            if let Some(token) = &session.access_token {
                let _ = storage.set_item(Self::AUTH_TOKEN_KEY, token);
            }
            let _ = storage.set_item(Self::ACTIVE_ROLE_KEY, &session.role);
        }
    }

    pub fn clear_session() {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item(Self::SESSION_KEY);
            let _ = storage.remove_item(Self::PORTAL_SESSION_KEY);
            let _ = storage.remove_item(Self::AUTH_TOKEN_KEY);
            let _ = storage.remove_item(Self::REFRESH_TOKEN_KEY);
            let _ = storage.remove_item(Self::ACTIVE_ROLE_KEY);
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn logout() {
        // Read refresh token before local cleanup.
        let saved_refresh_token = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(Self::REFRESH_TOKEN_KEY).ok().flatten())
            .unwrap_or_default();

        // Trigger cross-tab/app logout first.
        Self::broadcast_logout();

        // Clear local state for responsive UX.
        Self::clear_session();

        // Inform backend to revoke refresh token asynchronously.
        wasm_bindgen_futures::spawn_local(async move {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            let logout_url = format!("{}/api/v1/auth/logout", origin);
            let body = serde_json::json!({ "refresh_token": saved_refresh_token }).to_string();

            use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode};

            let headers = match Headers::new() {
                Ok(h) => h,
                Err(_) => return,
            };
            let _ = headers.set("Content-Type", "application/json");

            let opts = RequestInit::new();
            opts.set_method("POST");
            opts.set_mode(RequestMode::Cors);
            opts.set_credentials(RequestCredentials::Include);
            opts.set_headers(&headers);
            opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

            if let Some(window) = web_sys::window()
                && let Ok(request) = Request::new_with_str_and_init(&logout_url, &opts)
            {
                let _ = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                    .await;
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn broadcast_logout() {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let timestamp = js_sys::Date::now().to_string();
            let _ = storage.set_item(Self::LOGOUT_EVENT_KEY, &timestamp);
            let _ = storage.remove_item(Self::LOGOUT_EVENT_KEY);
        }
    }

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
                    Self::PORTAL_SESSION_KEY | Self::SESSION_KEY | Self::AUTH_TOKEN_KEY => {
                        on_session_change(Self::load_session());
                    }
                    Self::LOGOUT_EVENT_KEY => {
                        Self::clear_session();
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

        closure.forget();
    }
}
