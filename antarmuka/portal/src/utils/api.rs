//! API Client utilities for accessing Secreton API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gloo_net::http::Request;

/// System metrics structure (matching backend)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub disk_usage: DiskMetrics,
    pub network: NetworkMetrics,
    pub vault: VaultMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CpuMetrics {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct VaultMetrics {
    pub total_secrets: u64,
    pub total_keys: u64,
    pub total_policies: u64,
    pub active_sessions: u64,
    pub operations_per_second: f64,
}

/// Secret list item
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SecretListItem {
    pub path: String,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// Secret metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

/// Secret create request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSecretRequest {
    pub data: HashMap<String, String>,
    pub metadata: Option<SecretMetadata>,
    pub ttl: Option<u64>,
}

/// API Response wrapper
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
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    Network(String),
    Api(String),
    Serialization(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Api(msg) => write!(f, "API error: {}", msg),
            ApiError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// Base URL for API calls
fn get_base_url() -> String {
    "/api/v1".to_string()
}

/// Get system metrics
pub async fn get_system_metrics() -> Result<SystemMetrics, ApiError> {
    let url = format!("{}/admin/metrics", get_base_url());
    let token = get_token();

    let resp = Request::get(&url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Api(format!("Status {}: {}", resp.status(), resp.status_text())));
    }

    let api_resp: ApiResponse<SystemMetrics> = resp.json().await
        .map_err(|e| ApiError::Serialization(e.to_string()))?;

    if let Some(data) = api_resp.data {
        Ok(data)
    } else {
        Err(ApiError::Api(api_resp.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string())))
    }
}

/// Get secrets list
pub async fn get_secrets(filter: Option<String>) -> Result<Vec<SecretListItem>, ApiError> {
    let mut url = format!("{}/secrets", get_base_url());
    if let Some(f) = filter {
        url.push_str(&format!("?filter={}", f));
    }
    let token = get_token();

    let resp = Request::get(&url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Api(format!("Status {}: {}", resp.status(), resp.status_text())));
    }

    let api_resp: ApiResponse<Vec<SecretListItem>> = resp.json().await
        .map_err(|e| ApiError::Serialization(e.to_string()))?;

    if let Some(data) = api_resp.data {
        Ok(data)
    } else {
        Err(ApiError::Api(api_resp.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string())))
    }
}

/// Create secret
pub async fn create_secret(path: String, req: CreateSecretRequest) -> Result<(), ApiError> {
    // URL encode path segments
    let encoded_path: Vec<String> = path.split('/').map(|s| urlencoding::encode(s).to_string()).collect();
    let safe_path = encoded_path.join("%2F");

    let url = format!("{}/data/{}", get_base_url(), safe_path);
    let token = get_token();
    let body_str = serde_json::to_string(&req).map_err(|e| ApiError::Serialization(e.to_string()))?;

    let resp = Request::post(&url)
        .header("Authorization", &token)
        .header("Content-Type", "application/json")
        .body(body_str)
        .map_err(|e| ApiError::Network(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        let error_text = resp.text().await.unwrap_or_default();
        return Err(ApiError::Api(format!("Status {}: {}", resp.status(), error_text)));
    }

    Ok(())
}

/// Helper to get token from storage
fn get_token() -> String {
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        if let Ok(Some(session_json)) = storage.get_item("user_session") {
             if let Ok(session) = serde_json::from_str::<serde_json::Value>(&session_json) {
                 if let Some(token) = session.get("token").and_then(|t| t.as_str()) {
                     return format!("Bearer {}", token);
                 }
             }
        }
    }
    "".to_string()
}
