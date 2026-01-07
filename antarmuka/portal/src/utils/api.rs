//! API Client for communicating with Secreton Backend
//!
//! This module provides reusable functions to fetch data from the backend APIs.
//! It handles serialization, deserialization, and error propagation.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// System statistics returned by /admin/metrics
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub disk_usage: DiskMetrics,
    pub network: NetworkMetrics,
    pub vault: VaultMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CpuMetrics {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VaultMetrics {
    pub total_secrets: u64,
    pub total_keys: u64,
    pub total_policies: u64,
    pub active_sessions: u64,
    pub operations_per_second: f64,
}

/// Generic API response wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorDetail>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

/// Error type for API operations
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Secret list item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretListItem {
    pub path: String,
    pub version: u32,
    pub created_at: String, // ISO date string
    pub updated_at: String,
}

/// Secret data response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretResponse {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSecretRequest {
    pub data: HashMap<String, String>,
    pub metadata: Option<SecretMetadata>,
    pub ttl: Option<u64>,
}

const API_BASE_URL: &str = "/api/v1";

/// Fetch system metrics from /admin/metrics
pub async fn get_system_metrics() -> Result<SystemMetrics, ApiError> {
    let url = format!("{}/admin/metrics", API_BASE_URL);
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Server(format!("Status: {}", resp.status())));
    }

    let json: ApiResponse<SystemMetrics> = resp
        .json()
        .await
        .map_err(|e| ApiError::Serialization(e.to_string()))?;

    if let Some(data) = json.data {
        Ok(data)
    } else {
        Err(ApiError::Server(
            json.error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }
}

/// Fetch list of secrets
pub async fn get_secrets(filter: Option<&str>) -> Result<Vec<SecretListItem>, ApiError> {
    let mut url = format!("{}/secrets", API_BASE_URL);
    if let Some(f) = filter {
        url.push_str(&format!("?filter={}", f));
    }

    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Server(format!("Status: {}", resp.status())));
    }

    let json: ApiResponse<Vec<SecretListItem>> = resp
        .json()
        .await
        .map_err(|e| ApiError::Serialization(e.to_string()))?;

    Ok(json.data.unwrap_or_default())
}

/// Create or update a secret
pub async fn create_secret(path: &str, request: &CreateSecretRequest) -> Result<SecretResponse, ApiError> {
    let url = format!("{}/data/{}", API_BASE_URL, path);

    let resp = Request::post(&url)
        .json(request)
        .map_err(|e| ApiError::Serialization(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Server(format!("Status: {}", resp.status())));
    }

    let json: ApiResponse<SecretResponse> = resp
        .json()
        .await
        .map_err(|e| ApiError::Serialization(e.to_string()))?;

     if let Some(data) = json.data {
        Ok(data)
    } else {
        Err(ApiError::Server(
            json.error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }
}
