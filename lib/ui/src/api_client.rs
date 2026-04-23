//! Shared authenticated HTTP client for SIMPEL microfrontends.
//!
//! Provides a centralized, zero-boilerplate API client with automatic JWT
//! injection — similar to Laravel's `Http::withToken()` or Axios interceptors.
//!
//! # Usage
//!
//! ```rust,ignore
//! use lib_ui::api_client::{ApiClient, ApiError, ApiResult};
//!
//! let client = ApiClient::new("/api/pembinaan/perlengkapan");
//! let items: Vec<Item> = client.get("/items").await?;
//! let created: Item = client.post("/items", &new_item).await?;
//! client.delete_empty("/items/123").await?;
//! ```

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Field-level validation error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

/// Structured API error with user-friendly messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiError {
    Network(String),
    Auth(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Validation(Vec<FieldError>),
    Server(String),
    Parse(String),
    Unknown(String),
}

impl ApiError {
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Map an HTTP status code + response body to a typed error.
    pub fn from_status(status: u16, body: &str) -> Self {
        match status {
            401 => Self::Auth("Sesi Anda telah berakhir. Silakan login kembali.".into()),
            403 => Self::Forbidden(
                "Anda tidak memiliki hak akses untuk melakukan tindakan ini.".into(),
            ),
            404 => Self::NotFound("Data yang diminta tidak ditemukan.".into()),
            409 => Self::Conflict(if body.is_empty() {
                "Terjadi konflik data.".into()
            } else {
                body.to_string()
            }),
            400 | 422 => Self::Validation(parse_validation_body(body)),
            500..=599 => Self::Server(format!("Kesalahan server ({status}): {body}")),
            _ => Self::Unknown(format!("HTTP {status}: {body}")),
        }
    }

    /// User-friendly error message suitable for display in UI.
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(_) => {
                "Gagal terhubung ke server. Periksa koneksi internet Anda.".into()
            }
            Self::Auth(msg) | Self::Forbidden(msg) | Self::NotFound(msg) | Self::Conflict(msg) => {
                msg.clone()
            }
            Self::Validation(errors) => {
                if errors.is_empty() {
                    "Data yang dikirim tidak valid.".into()
                } else {
                    errors
                        .iter()
                        .map(|e| format!("{}: {}", e.field, e.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
            Self::Server(_) => {
                "Terjadi kesalahan di server. Silakan coba lagi beberapa saat.".into()
            }
            Self::Parse(_) => "Format respons server tidak dapat diproses.".into(),
            Self::Unknown(msg) => msg.clone(),
        }
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(s) => write!(f, "Network error: {s}"),
            Self::Auth(s) => write!(f, "Auth error: {s}"),
            Self::Forbidden(s) => write!(f, "Forbidden: {s}"),
            Self::NotFound(s) => write!(f, "Not found: {s}"),
            Self::Conflict(s) => write!(f, "Conflict: {s}"),
            Self::Validation(errors) => {
                write!(f, "Validation error: ")?;
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", e.field, e.message)?;
                }
                Ok(())
            }
            Self::Server(s) => write!(f, "Server error: {s}"),
            Self::Parse(s) => write!(f, "Parse error: {s}"),
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        match err {
            gloo_net::Error::JsError(e) => Self::Network(e.message),
            gloo_net::Error::SerdeError(e) => Self::Parse(e.to_string()),
            gloo_net::Error::GlooError(msg) => Self::Network(msg),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        Self::Unknown(s)
    }
}

/// Convenience type alias.
pub type ApiResult<T> = Result<T, ApiError>;

// ============================================================================
// API CLIENT
// ============================================================================

const AUTH_TOKEN_KEY: &str = "auth_token";

/// Read the JWT from localStorage.
pub fn get_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(AUTH_TOKEN_KEY).ok().flatten())
        .filter(|t| !t.is_empty())
}

/// Read the JWT, returning `ApiError::Auth` if missing.
pub fn require_auth_token() -> ApiResult<String> {
    get_auth_token().ok_or_else(|| {
        ApiError::auth("Sesi Anda telah berakhir. Silakan login kembali melalui portal.")
    })
}

/// Shared authenticated HTTP client.
///
/// Automatically injects `Authorization: Bearer <token>` on every request
/// and maps HTTP status codes to typed `ApiError` variants.
#[derive(Clone, Debug)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    fn full_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::get(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn get_optional<T: DeserializeOwned>(&self, path: &str) -> ApiResult<Option<T>> {
        match self.get::<T>(path).await {
            Ok(v) => Ok(Some(v)),
            Err(ApiError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn get_binary(&self, path: &str) -> ApiResult<Vec<u8>> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::get(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(ApiError::from)?;
        let status = resp.status();
        if !(200..300).contains(&status) {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::from_status(status, &body));
        }
        resp.binary().await.map_err(ApiError::from)
    }

    pub async fn post<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::post(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .map_err(ApiError::from)?
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::post(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn put<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::put(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .map_err(ApiError::from)?
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn patch<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::patch(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .map_err(ApiError::from)?
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::delete(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::from)?;
        parse_response(resp).await
    }

    pub async fn delete_empty(&self, path: &str) -> ApiResult<()> {
        let token = require_auth_token()?;
        let url = self.full_url(path);
        let resp = Request::delete(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(ApiError::from)?;
        let status = resp.status();
        if !(200..300).contains(&status) {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::from_status(status, &body));
        }
        Ok(())
    }
}

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

async fn parse_response<T: DeserializeOwned>(
    resp: gloo_net::http::Response,
) -> ApiResult<T> {
    let status = resp.status();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::from_status(status, &body));
    }
    resp.json::<T>().await.map_err(ApiError::from)
}

fn parse_validation_body(body: &str) -> Vec<FieldError> {
    if body.is_empty() {
        return Vec::new();
    }
    if let Ok(errors) = serde_json::from_str::<Vec<FieldError>>(body) {
        return errors;
    }
    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(errors) = wrapper.get("errors").and_then(|v| v.as_array()) {
            return errors
                .iter()
                .filter_map(|e| {
                    let field = e.get("field")?.as_str()?.to_string();
                    let message = e.get("message")?.as_str()?.to_string();
                    Some(FieldError { field, message })
                })
                .collect();
        }
        if let Some(msg) = wrapper.get("message").and_then(|v| v.as_str()) {
            return vec![FieldError {
                field: "general".to_string(),
                message: msg.to_string(),
            }];
        }
    }
    vec![FieldError {
        field: "general".to_string(),
        message: body.to_string(),
    }]
}
