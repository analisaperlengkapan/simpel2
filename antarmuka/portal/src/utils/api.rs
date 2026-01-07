//! API Client utilities
//!
//! Provides helper functions for making authenticated API requests.

use crate::features::auth::{AuthService, UserSession};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

/// System stats response
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SystemStats {
    pub uptime_seconds: u64,
    pub total_users: u64,
    pub active_sessions: u64,
    pub total_secrets: u64,
    pub total_keys: u64,
    pub storage_usage_bytes: u64,
    pub cache_hit_rate: f64,
    pub requests_per_minute: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretListItem {
    pub path: String,
    // Add other fields if needed
}

/// Helper function to perform authenticated fetch
pub async fn fetch_api<T>(url: &str, method: &str, body: Option<&impl Serialize>) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>
{
    // Get auth token
    let token = AuthService::get_token().ok_or("Not authenticated")?;

    // Get API base URL (can be configured)
    let base_url = std::env::var("SECRETON_API_URL").unwrap_or_else(|_| "http://localhost:8200/v1".to_string());
    let full_url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{}{}", base_url, url)
    };

    let mut request = match method {
        "GET" => Request::get(&full_url),
        "POST" => Request::post(&full_url),
        "PUT" => Request::put(&full_url),
        "DELETE" => Request::delete(&full_url),
        _ => return Err(format!("Unsupported method: {}", method)),
    };

    request = request.header("Authorization", &format!("Bearer {}", token));
    request = request.header("Content-Type", "application/json");

    let request_builder = if let Some(b) = body {
        request.json(b).map_err(|e| format!("Failed to serialize body: {}", e))?
    } else {
        request.build().map_err(|e| format!("Failed to build request: {}", e))?
    };

    let response = request_builder.send().await.map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("API Error: {}", response.status()));
    }

    let api_response: ApiResponse<T> = response.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;

    if api_response.success {
        api_response.data.ok_or_else(|| "No data in response".to_string())
    } else {
        Err(api_response.error.map(|e| e.message).unwrap_or_else(|| "Unknown API error".to_string()))
    }
}

/// Fetch system statistics
pub async fn get_system_metrics() -> Result<crate::pages::dashboard::SystemMetrics, String> {
    // Note: The backend endpoint is /metrics but returns a struct that wraps stats
    // We'll define a simpler fetch for now that matches the dashboard expectations
    // The dashboard expects `SystemMetrics` struct which is defined in dashboard.rs or locally
    // For now let's just use the `SystemStats` struct defined above and map it if needed,
    // or assume the caller handles it.

    // Actually, `dashboard.rs` probably uses its own types or hardcoded values.
    // We will update dashboard.rs to use the types we define here or imports.
    fetch_api("/metrics", "GET", None::<&String>).await
}

/// List secrets
pub async fn list_secrets(path: &str) -> Result<Vec<SecretListItem>, String> {
    fetch_api(&format!("/secrets?filter={}", path), "GET", None::<&String>).await
}

/// Create secret
pub async fn create_secret(path: &str, data: std::collections::HashMap<String, String>) -> Result<(), String> {
    let payload = serde_json::json!({
        "data": data,
        "metadata": {
            "description": "Created via Portal",
            "tags": ["portal"],
            "owner": "user" // Backend will override if needed
        }
    });

    // Create secret returns SecretResponse, but we might just ignore data
    let _: serde_json::Value = fetch_api(&format!("/data/{}", path), "POST", Some(&payload)).await?;
    Ok(())
}
