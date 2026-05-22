//! Single source of truth for Perlengkapan session state.
//!
//! The only durable identity artifact is the JWT stored in `auth_token`.
//! Everything we know about the user — username, name, NIP, satker, roles —
//! is decoded from that JWT on demand. We intentionally no longer read the
//! legacy `perlengkapan_user_session`, `user_session`, or `active_role`
//! localStorage keys: they race with the portal, drift out of sync, and
//! produced the "ghost session" class of bugs. If the JWT is missing or
//! expired, the user is simply logged out.

use lib_core::encoding::base64_decode_url;
use lib_core::jwt_claims::Claims;
use serde::{Deserialize, Serialize};

pub const AUTH_TOKEN_KEY: &str = "auth_token";
pub const REFRESH_TOKEN_KEY: &str = "refresh_token";
pub const LOGOUT_EVENT_KEY: &str = "logout_event";

/// Session derived from JWT claims. Never constructed from localStorage JSON —
/// always a projection of `Claims`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSession {
    pub user_id: String,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub nip: Option<String>,
    pub jabatan: Option<String>,
    pub satker_code: Option<String>,
    /// Satker UUID — projected from the matching JWT claim. Required for
    /// any request payload that ties a record back to the satker (permit
    /// creation, penghapusan submission, etc.). `None` means the user is
    /// not yet attached to a satker on the authenc side.
    pub satker_id: Option<String>,
    /// Display-only satker name. `None` when authenc has not populated it.
    pub satker_nama: Option<String>,
    /// All realm roles from the JWT (`realm_access.roles`). Order matches the
    /// issuer — no filtering, no aliasing. Callers ask about specific roles.
    pub roles: Vec<String>,
    /// Primary role picked for UI surfaces that still need a single label.
    pub role: String,
    /// JWT expiry as a Unix timestamp (seconds). `None` means the token had
    /// no `exp` claim, which we treat as "already expired".
    pub expires_at: Option<i64>,
}

impl UserSession {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has_role(r))
    }

    /// An admin is anyone holding an explicit admin realm role. No more
    /// `starts_with("admin_")` string matching — that was a frequent source
    /// of false positives for roles like `admin_master_read_only`.
    pub fn is_admin(&self) -> bool {
        const ADMIN_ROLES: &[&str] = &[
            "admin",
            "super_admin",
            "admin_pusat",
            "admin_wilayah",
            "admin_satker",
        ];
        self.has_any_role(ADMIN_ROLES)
    }

    pub fn is_validator_pusat(&self) -> bool {
        self.has_role("validator_pusat")
    }

    pub fn is_validator_wilayah(&self) -> bool {
        self.has_role("validator_wilayah")
    }

    pub fn is_operator_satker(&self) -> bool {
        self.has_role("operator_satker")
    }

    /// Whether the JWT is still within its expiry window. A missing `exp`
    /// is treated as expired to avoid accepting malformed tokens.
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(exp) => (exp as f64) > current_unix_seconds(),
            None => false,
        }
    }

    /// True if the token expires within `seconds` — used to trigger refresh.
    pub fn expires_within(&self, seconds: i64) -> bool {
        match self.expires_at {
            Some(exp) => {
                let now = current_unix_seconds() as i64;
                exp - now <= seconds
            }
            None => true,
        }
    }
}

fn current_unix_seconds() -> f64 {
    js_sys::Date::now() / 1000.0
}

fn pick_primary_role(roles: &[String]) -> String {
    const PRIORITY: &[&str] = &[
        "super_admin",
        "admin",
        "admin_pusat",
        "admin_wilayah",
        "admin_satker",
        "validator_pusat",
        "validator_wilayah",
        "operator_satker",
    ];
    for candidate in PRIORITY {
        if roles.iter().any(|r| r == candidate) {
            return (*candidate).to_string();
        }
    }
    roles
        .first()
        .cloned()
        .unwrap_or_else(|| "operator_satker".to_string())
}

pub struct AuthService;

impl AuthService {
    /// Load the current session by decoding the JWT from `auth_token`.
    /// Returns `None` if the token is missing, malformed, or expired.
    pub fn load_session() -> Option<UserSession> {
        let token = Self::read_token()?;
        let session = Self::session_from_token(&token).ok()?;
        if session.is_active() {
            Some(session)
        } else {
            None
        }
    }

    /// Best-effort access to the raw token. Callers that need the token for
    /// outbound HTTP calls should prefer `api::client::require_auth_token`
    /// which produces a typed error when it's missing.
    pub fn read_token() -> Option<String> {
        let storage = web_sys::window()?.local_storage().ok().flatten()?;
        storage
            .get_item(AUTH_TOKEN_KEY)
            .ok()
            .flatten()
            .filter(|t| !t.is_empty())
    }

    pub fn read_refresh_token() -> Option<String> {
        let storage = web_sys::window()?.local_storage().ok().flatten()?;
        storage
            .get_item(REFRESH_TOKEN_KEY)
            .ok()
            .flatten()
            .filter(|t| !t.is_empty())
    }

    /// Decode the JWT payload and project its claims into a `UserSession`.
    /// WASM-safe: no signature verification (the gateway already does that).
    pub fn session_from_token(token: &str) -> Result<UserSession, String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }
        let payload_bytes =
            base64_decode_url(parts[1]).map_err(|e| format!("base64 decode: {e}"))?;
        let payload = String::from_utf8(payload_bytes).map_err(|e| format!("utf8 decode: {e}"))?;
        let claims: Claims =
            serde_json::from_str(&payload).map_err(|e| format!("claim decode: {e}"))?;

        let roles = claims
            .realm_access
            .as_ref()
            .map(|ra| ra.roles.clone())
            .unwrap_or_default();
        let primary = pick_primary_role(&roles);
        let username = claims
            .preferred_username
            .clone()
            .unwrap_or_else(|| claims.sub.clone());
        let name = claims.name.clone().unwrap_or_else(|| username.clone());

        Ok(UserSession {
            user_id: claims.sub,
            username,
            name,
            email: claims.email,
            nip: claims.nip,
            jabatan: claims.jabatan,
            satker_code: claims.satker_code,
            satker_id: claims.satker_id,
            satker_nama: claims.satker_nama,
            roles,
            role: primary,
            expires_at: Some(claims.exp as i64),
        })
    }

    /// Persist the token and emit a storage event so other tabs / the portal
    /// can react. We deliberately do **not** store a serialized session — it
    /// would duplicate the JWT and drift out of sync.
    pub fn store_token(token: &str) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(AUTH_TOKEN_KEY, token);
        }
    }

    pub fn store_refresh_token(token: &str) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(REFRESH_TOKEN_KEY, token);
        }
    }

    pub fn clear_session() {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item(AUTH_TOKEN_KEY);
            let _ = storage.remove_item(REFRESH_TOKEN_KEY);
            // Best-effort cleanup of legacy keys so stale data from older
            // builds doesn't confuse diagnostics.
            let _ = storage.remove_item("perlengkapan_user_session");
            let _ = storage.remove_item("user_session");
            let _ = storage.remove_item("active_role");
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn logout() {
        let refresh = Self::read_refresh_token().unwrap_or_default();

        Self::broadcast_logout();
        Self::clear_session();

        wasm_bindgen_futures::spawn_local(async move {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "http://localhost:8080".to_string());
            let url = format!("{origin}/api/v1/auth/logout");
            let body = serde_json::json!({ "refresh_token": refresh }).to_string();

            use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode};
            let Ok(headers) = Headers::new() else {
                return;
            };
            let _ = headers.set("Content-Type", "application/json");

            let opts = RequestInit::new();
            opts.set_method("POST");
            opts.set_mode(RequestMode::Cors);
            opts.set_credentials(RequestCredentials::Include);
            opts.set_headers(&headers);
            opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

            if let Some(window) = web_sys::window()
                && let Ok(request) = Request::new_with_str_and_init(&url, &opts)
            {
                let _ =
                    wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await;
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn broadcast_logout() {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let timestamp = js_sys::Date::now().to_string();
            let _ = storage.set_item(LOGOUT_EVENT_KEY, &timestamp);
            let _ = storage.remove_item(LOGOUT_EVENT_KEY);
        }
    }

    /// Ask the authenc backend to mint a new access token. Returns the
    /// refreshed session if successful. Failures propagate so the caller
    /// can decide whether to fall back to a hard logout.
    #[cfg(target_arch = "wasm32")]
    pub async fn try_refresh() -> Result<UserSession, String> {
        let refresh = Self::read_refresh_token().ok_or_else(|| "no refresh token".to_string())?;
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        let url = format!("{origin}/api/v1/auth/refresh");
        let body = serde_json::json!({ "refresh_token": refresh }).to_string();

        let resp = gloo_net::http::Request::post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !(200..300).contains(&resp.status()) {
            return Err(format!("refresh failed with status {}", resp.status()));
        }

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            #[serde(default)]
            refresh_token: Option<String>,
        }

        let parsed: RefreshResponse = resp.json().await.map_err(|e| e.to_string())?;
        Self::store_token(&parsed.access_token);
        if let Some(new_refresh) = parsed.refresh_token {
            Self::store_refresh_token(&new_refresh);
        }
        Self::session_from_token(&parsed.access_token)
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
                    AUTH_TOKEN_KEY => {
                        on_session_change(Self::load_session());
                    }
                    LOGOUT_EVENT_KEY => {
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
