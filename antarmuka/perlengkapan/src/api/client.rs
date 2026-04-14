//! Centralized HTTP client for the Perlengkapan API.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Base path for the perlengkapan backend API (proxied via gateway).
pub const API_BASE: &str = "/api/pembinaan/perlengkapan";

/// Get the auth token from localStorage (set by auth flow).
pub fn get_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("auth_token").ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn auth_token_or_err() -> Result<String, gloo_net::Error> {
    get_auth_token()
        .ok_or_else(|| gloo_net::Error::GlooError("No authentication token found".to_string()))
}

#[cfg(target_arch = "wasm32")]
pub async fn auth_get_json<T: DeserializeOwned>(url: &str) -> Result<T, gloo_net::Error> {
    let token = auth_token_or_err()?;

    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
}

#[cfg(target_arch = "wasm32")]
pub async fn auth_get_binary(url: &str) -> Result<Vec<u8>, gloo_net::Error> {
    let token = auth_token_or_err()?;

    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.binary().await
}

#[cfg(target_arch = "wasm32")]
pub async fn auth_post_json<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: &B,
) -> Result<T, gloo_net::Error> {
    let token = auth_token_or_err()?;

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(body)?
        .send()
        .await?;

    if !resp.ok() {
        let status = resp.status();
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            status, err_text
        )));
    }

    resp.json().await
}

#[cfg(target_arch = "wasm32")]
pub async fn auth_put_json<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: &B,
) -> Result<T, gloo_net::Error> {
    let token = auth_token_or_err()?;

    let resp = Request::put(url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(body)?
        .send()
        .await?;

    if !resp.ok() {
        let status = resp.status();
        let err_text = resp.text().await.unwrap_or_default();
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {} - {}",
            status, err_text
        )));
    }

    resp.json().await
}

#[cfg(target_arch = "wasm32")]
pub async fn auth_delete_json<T: DeserializeOwned>(url: &str) -> Result<T, gloo_net::Error> {
    let token = auth_token_or_err()?;

    let resp = Request::delete(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.ok() {
        return Err(gloo_net::Error::GlooError(format!(
            "API Error: {}",
            resp.status()
        )));
    }

    resp.json().await
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
