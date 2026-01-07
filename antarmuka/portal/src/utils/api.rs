//! API Client utilities
//!
//! Provides helper functions for making authenticated API requests

use crate::features::auth::AuthService;
use gloo_net::http::{Request, Response};
use serde::Serialize;
use serde_json::Value;
use std::fmt;

/// Error type for API requests
#[derive(Debug)]
pub enum ApiError {
    Network(String),
    Response { status: u16, message: String },
    Serialization(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Response { status, message } => write!(f, "API error {}: {}", status, message),
            ApiError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// Helper to get the API base URL
fn get_api_url() -> String {
    // In production this would come from env
    "http://localhost:8200".to_string()
}

/// Make an authenticated API request
pub async fn fetch_api<T: Serialize>(
    method: &str,
    path: &str,
    body: Option<&T>,
) -> Result<Response, ApiError> {
    let url = format!("{}{}", get_api_url(), path);
    let token = AuthService::get_token().ok_or_else(|| {
        ApiError::Response {
            status: 401,
            message: "Not authenticated".to_string()
        }
    })?;

    let mut request = match method {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PUT" => Request::put(&url),
        "DELETE" => Request::delete(&url),
        _ => return Err(ApiError::Network(format!("Unsupported method: {}", method))),
    };

    // Add headers
    let mut builder = request
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json");

    // Add body if present
    // Note: To handle type compatibility issues with gloo-net where builder might be inferred incorrectly,
    // we shadow the variable or use a new one.
    // The key issue is that `request` (from match) is a `RequestBuilder`.
    // `builder` is a `RequestBuilder`.
    // `builder.json()` returns `RequestBuilder`.
    // But the compiler seems confused about lifetime or exact type instance.

    // Simplification: We only need body for POST/PUT usually.
    if let Some(data) = body {
        // We rely on the fact that json() consumes the builder and returns a new one (or same one)
        let body_builder = builder.json(data).map_err(|e| ApiError::Serialization(e.to_string()))?;
        // Now use this builder to send
        let response = body_builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        // Check status (duplicated logic, but solves the type inference issue by returning early)
        if !response.ok() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            let message = if let Ok(json) = serde_json::from_str::<Value>(&text) {
                json.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&text)
                    .to_string()
            } else {
                text
            };

            return Err(ApiError::Response { status, message });
        }

        return Ok(response);
    }

    // Send request
    let response = builder
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    // Check status
    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // Try to parse error message from JSON
        let message = if let Ok(json) = serde_json::from_str::<Value>(&text) {
            json.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or(&text)
                .to_string()
        } else {
            text
        };

        return Err(ApiError::Response { status, message });
    }

    Ok(response)
}
