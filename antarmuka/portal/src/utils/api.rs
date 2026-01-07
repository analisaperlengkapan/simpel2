//! API Client utilities
//!
//! Provides helper functions for making authenticated API requests.

use crate::features::auth::{AuthService, UserSession};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

/// Backend System stats response (flat structure)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BackendSystemStats {
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

/// Fetch system statistics and map to Frontend SystemMetrics
pub async fn get_system_metrics() -> Result<crate::pages::dashboard::SystemMetrics, String> {
    let backend_stats: BackendSystemStats = fetch_api("/metrics", "GET", None::<&String>).await?;

    Ok(crate::pages::dashboard::SystemMetrics {
        uptime: backend_stats.uptime_seconds,
        memory_usage: crate::pages::dashboard::MemoryMetrics {
            total: 16 * 1024 * 1024 * 1024, // 16GB Placeholder (backend doesn't provide total yet)
            used: backend_stats.storage_usage_bytes, // Using storage as memory proxy or 0
        },
        vault: crate::pages::dashboard::VaultMetrics {
            total_secrets: backend_stats.total_secrets,
            total_keys: backend_stats.total_keys,
        },
    })
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
