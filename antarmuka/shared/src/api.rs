//! # SIMPelv2 API Client
//!
//! Centralized HTTP client for all API communications across microfrontends.

use crate::types::ApiResponse;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

/// API Error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    NetworkError(String),
    SerializationError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    ValidationError(Vec<String>),
    ServerError(String),
    NotFound(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network Error: {msg}"),
            ApiError::SerializationError(msg) => write!(f, "Serialization Error: {msg}"),
            ApiError::AuthenticationError(msg) => write!(f, "Authentication Error: {msg}"),
            ApiError::AuthorizationError(msg) => write!(f, "Authorization Error: {msg}"),
            ApiError::ValidationError(errors) => {
                write!(f, "Validation Errors: {}", errors.join(", "))
            }
            ApiError::ServerError(msg) => write!(f, "Server Error: {msg}"),
            ApiError::NotFound(msg) => write!(f, "Not Found: {msg}"),
        }
    }
}

/// Main API Client
#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "/api/v1".to_string(),
        }
    }

    /// Get authentication token from localStorage
    fn get_auth_token(&self) -> Option<String> {
        crate::utils::storage::load_string("auth_token")
            .ok()
            .flatten()
    }

    /// Generic GET request
    pub async fn get<T>(&self, endpoint: &str) -> Result<ApiResponse<T>, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = Request::get(&url);

        if let Some(token) = self.get_auth_token() {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }

        match request.send().await {
            Ok(response) => match response.json::<ApiResponse<T>>().await {
                Ok(data) => Ok(data),
                Err(e) => Err(ApiError::SerializationError(format!("Parse error: {e}"))),
            },
            Err(e) => Err(ApiError::NetworkError(format!("Request failed: {e}"))),
        }
    }

    /// Generic POST request
    pub async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<ApiResponse<T>, ApiError>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = Request::post(&url);

        if let Some(token) = self.get_auth_token() {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }

        match serde_json::to_string(body) {
            Ok(json_body) => match request.body(json_body) {
                Ok(req) => match req.send().await {
                    Ok(response) => match response.json::<ApiResponse<T>>().await {
                        Ok(data) => Ok(data),
                        Err(e) => Err(ApiError::SerializationError(format!("Parse error: {e}"))),
                    },
                    Err(e) => Err(ApiError::NetworkError(format!("Request failed: {e}"))),
                },
                Err(e) => Err(ApiError::NetworkError(format!("Request build failed: {e}"))),
            },
            Err(e) => Err(ApiError::SerializationError(format!(
                "Serialize error: {e}"
            ))),
        }
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
