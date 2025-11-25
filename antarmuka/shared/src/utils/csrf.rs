// CSRF (Cross-Site Request Forgery) protection utilities
use gloo::timers::callback::Timeout;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::window;

const CSRF_TOKEN_KEY: &str = "csrf_token";
const CSRF_TOKEN_EXPIRY_KEY: &str = "csrf_token_expiry";
const CSRF_TOKEN_LIFETIME_MS: u64 = 3600000; // 1 hour

/// CSRF token manager
#[derive(Clone, Debug)]
pub struct CsrfToken {
    token: String,
    expires_at: i64,
}

impl CsrfToken {
    /// Generate a new CSRF token
    pub fn generate() -> Self {
        let token = Self::generate_random_token();
        let expires_at = js_sys::Date::now() as i64 + CSRF_TOKEN_LIFETIME_MS as i64;

        Self { token, expires_at }
    }

    /// Generate a cryptographically secure random token
    fn generate_random_token() -> String {
        let mut bytes = [0u8; 32];

        // Use crypto.getRandomValues for secure random generation
        if let Some(window) = window()
            && let Ok(crypto) = window.crypto()
            && crypto.get_random_values_with_u8_array(&mut bytes).is_ok()
        {
            // Convert bytes to hex string
            return bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
        }

        // Fallback to Math.random (less secure, but better than nothing)
        (0..32)
            .map(|_| format!("{:02x}", (js_sys::Math::random() * 255.0) as u8))
            .collect::<String>()
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        js_sys::Date::now() as i64 > self.expires_at
    }

    /// Get token value
    pub fn value(&self) -> &str {
        &self.token
    }

    /// Store token in localStorage
    pub fn store(&self) -> Result<(), JsValue> {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
            storage.set_item(CSRF_TOKEN_KEY, &self.token)?;
            storage.set_item(CSRF_TOKEN_EXPIRY_KEY, &self.expires_at.to_string())?;
            Ok(())
        } else {
            Err(JsValue::from_str("localStorage not available"))
        }
    }

    /// Load token from localStorage
    pub fn load() -> Option<Self> {
        let storage = window()?.local_storage().ok()??;

        let token = storage.get_item(CSRF_TOKEN_KEY).ok()??;
        let expires_at = storage
            .get_item(CSRF_TOKEN_EXPIRY_KEY)
            .ok()??
            .parse::<i64>()
            .ok()?;

        let csrf_token = Self { token, expires_at };

        // Return None if expired
        if csrf_token.is_expired() {
            None
        } else {
            Some(csrf_token)
        }
    }

    /// Clear token from localStorage
    pub fn clear() -> Result<(), JsValue> {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
            storage.remove_item(CSRF_TOKEN_KEY)?;
            storage.remove_item(CSRF_TOKEN_EXPIRY_KEY)?;
            Ok(())
        } else {
            Err(JsValue::from_str("localStorage not available"))
        }
    }
}

/// Get or generate CSRF token
pub fn get_csrf_token() -> String {
    // Try to load existing token
    if let Some(token) = CsrfToken::load() {
        return token.value().to_string();
    }

    // Generate new token if none exists or expired
    let token = CsrfToken::generate();
    let _ = token.store();
    token.value().to_string()
}

/// Validate CSRF token
pub fn validate_csrf_token(token: &str) -> bool {
    if let Some(stored_token) = CsrfToken::load() {
        // Constant-time comparison to prevent timing attacks
        constant_time_compare(token, stored_token.value())
    } else {
        false
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// Refresh CSRF token (should be called periodically)
pub fn refresh_csrf_token() {
    let token = CsrfToken::generate();
    let _ = token.store();
}

/// CSRF-protected form component
#[component]
pub fn CsrfProtectedForm(
    /// Form submission handler
    on_submit: Callback<web_sys::SubmitEvent>,
    /// Form children
    children: Children,
    #[prop(optional)] class: String,
    #[prop(optional)] method: String,
    #[prop(optional)] action: String,
) -> impl IntoView {
    let csrf_token = get_csrf_token();
    let method = if method.is_empty() {
        "POST".to_string()
    } else {
        method
    };

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        // Verify CSRF token before submitting
        let form_data = web_sys::FormData::new_with_form(
            &ev.target()
                .unwrap()
                .dyn_into::<web_sys::HtmlFormElement>()
                .unwrap(),
        )
        .unwrap();

        if let Some(token) = form_data.get("csrf_token").as_string() {
            if validate_csrf_token(&token) {
                on_submit.run(ev);
            } else {
                // CSRF token validation failed
                // Show error to user
                if let Some(window) = window() {
                    let _ = window.alert_with_message(
                        "Security error: Invalid CSRF token. Please refresh the page.",
                    );
                }
            }
        } else {
            // CSRF token missing from form
        }
    };

    view! {
        <form
            class=class
            method=method
            action=action
            on:submit=handle_submit
        >
            // Hidden CSRF token field
            <input
                type="hidden"
                name="csrf_token"
                value=csrf_token
            />
            {children()}
        </form>
    }
}

/// Hook for CSRF token management
pub fn use_csrf_token() -> (ReadSignal<String>, impl Fn()) {
    let (token, set_token) = signal(get_csrf_token());

    // Refresh token periodically (every 30 minutes)
    Effect::new(move |_| {
        let timeout = Timeout::new(1800000, move || {
            refresh_csrf_token();
            set_token.set(get_csrf_token());
        });
        timeout.forget();
    });

    let refresh = move || {
        refresh_csrf_token();
        set_token.set(get_csrf_token());
    };

    (token, refresh)
}

/// Double-submit cookie pattern for CSRF protection
/// This is used in conjunction with backend validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoubleSubmitCookie {
    pub token: String,
}

impl DoubleSubmitCookie {
    /// Create a new double-submit cookie
    pub fn new() -> Self {
        Self {
            token: CsrfToken::generate().value().to_string(),
        }
    }

    /// Set the cookie in the browser
    pub fn set_cookie(&self) -> Result<(), JsValue> {
        if let Some(document) = window().and_then(|w| w.document()) {
            // Set cookie with SameSite=Strict and Secure flags
            let cookie = format!(
                "csrf_token={}; SameSite=Strict; Secure; Path=/; Max-Age=3600",
                self.token
            );
            // Cast to HtmlDocument to access cookie methods
            if let Some(html_doc) = document.dyn_ref::<web_sys::HtmlDocument>() {
                html_doc.set_cookie(&cookie)?;
            }
            Ok(())
        } else {
            Err(JsValue::from_str("Document not available"))
        }
    }

    /// Get the cookie value
    pub fn get_cookie() -> Option<String> {
        let document = window()?.document()?;
        let html_doc = document.dyn_ref::<web_sys::HtmlDocument>()?;
        let cookies = html_doc.cookie().ok()?;

        // Parse cookies to find csrf_token
        for cookie in cookies.split(';') {
            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "csrf_token" {
                return Some(parts[1].to_string());
            }
        }

        None
    }

    /// Validate double-submit cookie
    pub fn validate(header_token: &str) -> bool {
        if let Some(cookie_token) = Self::get_cookie() {
            constant_time_compare(header_token, &cookie_token)
        } else {
            false
        }
    }
}

impl Default for DoubleSubmitCookie {
    fn default() -> Self {
        Self::new()
    }
}

/// API request with CSRF protection
pub async fn csrf_protected_request(
    method: &str,
    url: &str,
    body: Option<serde_json::Value>,
) -> Result<String, String> {
    let csrf_token = get_csrf_token();

    // Create request with CSRF token in header
    let opts = web_sys::RequestInit::new();
    opts.set_method(method);
    opts.set_mode(web_sys::RequestMode::Cors);
    opts.set_credentials(web_sys::RequestCredentials::Include);

    // Set headers
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("{:?}", e))?;
    headers
        .set("X-CSRF-Token", &csrf_token)
        .map_err(|e| format!("{:?}", e))?;
    opts.set_headers(&headers);

    // Set body if provided
    if let Some(body) = body {
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        opts.set_body(&JsValue::from_str(&body_str));
    }

    // Make request
    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;

    let window = window().ok_or("No window object")?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{:?}", e))?;

    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;

    // Check response status
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    // Get response text
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;

    Ok(text.as_string().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let token = CsrfToken::generate();
        assert_eq!(token.value().len(), 64); // 32 bytes = 64 hex chars
        assert!(!token.is_expired());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc123", "abc12"));
    }

    #[test]
    fn test_double_submit_cookie() {
        let cookie = DoubleSubmitCookie::new();
        assert!(!cookie.token.is_empty());
    }
}
