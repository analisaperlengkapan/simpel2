//! Unified HTTP client for the Perlengkapan API.
//!
//! All API functions in `crate::api::*` go through this module. There is one
//! canonical error type (`AppError`), one auth token source, and one request
//! pipeline that translates status codes into typed errors.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::api::error::{AppError, AppResult};

pub const API_BASE: &str = "/api/pembinaan/perlengkapan";

const AUTH_TOKEN_KEY: &str = "auth_token";

pub fn get_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(AUTH_TOKEN_KEY).ok().flatten())
        .filter(|t| !t.is_empty())
}

pub fn require_auth_token() -> AppResult<String> {
    get_auth_token().ok_or_else(|| {
        AppError::auth("Sesi Anda telah berakhir. Silakan login kembali melalui portal.")
    })
}

fn full_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else if path.starts_with('/') && path.starts_with(API_BASE) {
        path.to_string()
    } else if path.starts_with('/') {
        format!("{API_BASE}{path}")
    } else {
        format!("{API_BASE}/{path}")
    }
}

async fn parse_response<T: DeserializeOwned>(resp: gloo_net::http::Response) -> AppResult<T> {
    let status = resp.status();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }
    resp.json::<T>().await.map_err(AppError::from)
}

async fn parse_response_raw(resp: gloo_net::http::Response) -> AppResult<String> {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(AppError::from_status(status, &body));
    }
    Ok(body)
}

pub async fn api_get<T: DeserializeOwned>(path: &str) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_get_optional<T: DeserializeOwned>(path: &str) -> AppResult<Option<T>> {
    match api_get::<T>(path).await {
        Ok(v) => Ok(Some(v)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn api_get_binary(path: &str) -> AppResult<Vec<u8>> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(AppError::from)?;

    let status = resp.status();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }

    resp.binary().await.map_err(AppError::from)
}

pub async fn api_post<B: Serialize + ?Sized, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(AppError::from)?
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_post_empty<T: DeserializeOwned>(path: &str) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_put<B: Serialize + ?Sized, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(AppError::from)?
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_patch<B: Serialize + ?Sized, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(AppError::from)?
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_delete<T: DeserializeOwned>(path: &str) -> AppResult<T> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(AppError::from)?;

    parse_response(resp).await
}

pub async fn api_delete_empty(path: &str) -> AppResult<()> {
    let token = require_auth_token()?;
    let url = full_url(path);

    let resp = Request::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(AppError::from)?;

    let _ = parse_response_raw(resp).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Legacy aliases — to be removed as call sites are migrated to the functions
// above. These exist so the tree keeps compiling during the incremental port.
// ---------------------------------------------------------------------------

pub async fn auth_get_json<T: DeserializeOwned>(url: &str) -> AppResult<T> {
    let token = require_auth_token()?;
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(AppError::from)?;
    parse_response(resp).await
}

pub async fn auth_get_binary(url: &str) -> AppResult<Vec<u8>> {
    let token = require_auth_token()?;
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(AppError::from)?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::from_status(status, &body));
    }
    resp.binary().await.map_err(AppError::from)
}

pub async fn auth_post_json<B: Serialize + ?Sized, T: DeserializeOwned>(
    url: &str,
    body: &B,
) -> AppResult<T> {
    let token = require_auth_token()?;
    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(AppError::from)?
        .send()
        .await
        .map_err(AppError::from)?;
    parse_response(resp).await
}

pub async fn auth_put_json<B: Serialize + ?Sized, T: DeserializeOwned>(
    url: &str,
    body: &B,
) -> AppResult<T> {
    let token = require_auth_token()?;
    let resp = Request::put(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(AppError::from)?
        .send()
        .await
        .map_err(AppError::from)?;
    parse_response(resp).await
}

pub async fn auth_delete_json<T: DeserializeOwned>(url: &str) -> AppResult<T> {
    let token = require_auth_token()?;
    let resp = Request::delete(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(AppError::from)?;
    parse_response(resp).await
}
