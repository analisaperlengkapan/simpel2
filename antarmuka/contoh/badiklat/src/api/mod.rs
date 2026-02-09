//! API client untuk komunikasi dengan backend services

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use leptos::*;

pub mod auth;
pub mod badiklat;
pub mod common;

pub use auth::*;
pub use badiklat::*;
pub use common::*;

/// Base URL untuk API backend
const API_BASE_URL: &str = "http://localhost:8080/api/v1";

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
}

/// Error type untuk API calls
#[derive(Debug, Clone)]
pub enum ApiError {
    NetworkError(String),
    SerializationError(String),
    ServerError(u16, String),
    Unauthorized,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ApiError::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            ApiError::Unauthorized => write!(f, "Unauthorized access"),
        }
    }
}

/// Helper function untuk membuat authenticated request
pub async fn authenticated_request(
    method: &str,
    endpoint: &str,
    body: Option<String>,
) -> Result<Request, ApiError> {
    let token = get_auth_token().ok_or(ApiError::Unauthorized)?;
    
    let mut request = Request::new(&format!("{}{}", API_BASE_URL, endpoint))
        .method(gloo_net::http::Method::from(method))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token));

    if let Some(body_data) = body {
        request = request.body(body_data);
    }

    Ok(request)
}

/// Get authentication token from local storage
fn get_auth_token() -> Option<String> {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::get("auth_token").ok()
}
