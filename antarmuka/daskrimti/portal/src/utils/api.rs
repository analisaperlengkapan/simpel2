//! API Client for fetching data from the backend.

use crate::features::auth::AuthService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub disk_usage: DiskMetrics,
    pub network: NetworkMetrics,
    pub vault: VaultMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuMetrics {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultMetrics {
    pub total_secrets: u64,
    pub total_keys: u64,
    pub total_policies: u64,
    pub active_sessions: u64,
    pub operations_per_second: f64,
}

/// Secret list item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretListItem {
    pub path: String,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: String, // ISO string
    pub updated_at: String, // ISO string
}

/// Secret metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

/// Secret response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretResponse {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
}

/// API Response wrapper
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
    timestamp: String,
    request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    code: String,
    message: String,
    details: Option<serde_json::Value>,
}

const API_BASE_URL: &str = "http://localhost:8081/api/v1";

/// Fetch system metrics
pub async fn get_system_metrics() -> Result<SystemMetrics, String> {
    fetch_api("/admin/metrics").await
}

/// List secrets
pub async fn list_secrets() -> Result<Vec<SecretListItem>, String> {
    fetch_api("/secrets").await
}

/// Get secret details
pub async fn get_secret(path: &str) -> Result<SecretResponse, String> {
    let encoded_path = urlencoding::encode(path);
    fetch_api(&format!("/data/{}", encoded_path)).await
}

/// Create or update secret
pub async fn create_secret(
    path: &str,
    data: HashMap<String, String>,
    metadata: Option<SecretMetadata>,
) -> Result<SecretResponse, String> {
    let encoded_path = urlencoding::encode(path);
    let url = format!("{}/data/{}", API_BASE_URL, encoded_path);

    let body = serde_json::json!({
        "data": data,
        "metadata": metadata
    });

    let token = AuthService::get_token().ok_or("Not authenticated")?;

    let resp = gloo_net::http::Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let api_resp: ApiResponse<SecretResponse> = resp.json().await.map_err(|e| e.to_string())?;

    if api_resp.success {
        Ok(api_resp.data.ok_or("No data received")?)
    } else {
        Err(api_resp
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Delete secret
pub async fn delete_secret(path: &str) -> Result<(), String> {
    let encoded_path = urlencoding::encode(path);
    let url = format!("{}/data/{}", API_BASE_URL, encoded_path);

    let token = AuthService::get_token().ok_or("Not authenticated")?;

    let resp = gloo_net::http::Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let api_resp: ApiResponse<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;

    if api_resp.success {
        Ok(())
    } else {
        Err(api_resp
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Helper to fetch API data
async fn fetch_api<T: for<'de> Deserialize<'de>>(endpoint: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE_URL, endpoint);
    let token = AuthService::get_token().ok_or("Not authenticated")?;

    let resp = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let api_resp: ApiResponse<T> = resp.json().await.map_err(|e| e.to_string())?;

    if api_resp.success {
        Ok(api_resp.data.ok_or("No data received")?)
    } else {
        Err(api_resp
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}
