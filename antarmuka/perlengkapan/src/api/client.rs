//! Centralized HTTP client for the Perlengkapan API.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;

/// Base path for the perlengkapan backend API (proxied via gateway).
pub const API_BASE: &str = "/api/pembinaan/perlengkapan";

/// Get the auth token from localStorage (set by auth flow).
pub fn get_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("auth_token").ok().flatten())
}

/// Perform an authenticated GET request and deserialize the JSON response.
pub async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Perform an authenticated POST request with a JSON body.
pub async fn api_post<B: serde::Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let token = get_auth_token().unwrap_or_default();

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(body)
        .map_err(|e| format!("Serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.ok() {
        return Err(format!("API error: HTTP {}", resp.status()));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}
