use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::{AuditLogEntry, MaintenanceResult, SystemMetricsResponse};

/// API Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    Network(String),
    Parse(String),
    Request(String),
    Unauthorized,
    Forbidden,
    NotFound,
    Internal(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiError::Request(msg) => write!(f, "Request error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Forbidden => write!(f, "Forbidden"),
            ApiError::NotFound => write!(f, "Not found"),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// API Response Wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponseData<T> {
    pub data: T,
}

/// Base API URL - defaults to localhost if not set
fn get_api_base_url() -> &'static str {
    option_env!("API_BASE_URL").unwrap_or("http://localhost:8080/v1")
}

/// Helper to get authorization header with token
#[allow(dead_code)]
fn auth_header(token: Option<&str>) -> Vec<(&str, &str)> {
    match token {
        Some(t) => vec![("Authorization", t)],
        None => vec![],
    }
}

/// Fetch system metrics
pub async fn fetch_system_metrics(token: &str) -> Result<SystemMetricsResponse, ApiError> {
    let url = format!("{}/admin/metrics", get_api_base_url());
    let resp = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Request(format!(
            "Failed to fetch metrics: {}",
            resp.status()
        )));
    }

    let api_response: ApiResponse<SystemMetricsResponse> = resp
        .json()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::Request("No data returned".to_string()))
}

/// Fetch system audit logs
pub async fn fetch_audit_logs(
    token: &str,
    limit: Option<u32>,
) -> Result<Vec<AuditLogEntry>, ApiError> {
    let limit = limit.unwrap_or(50);
    let url = format!("{}/admin/logs?limit={}", get_api_base_url(), limit);
    let resp = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Request(format!(
            "Failed to fetch logs: {}",
            resp.status()
        )));
    }

    let api_response: ApiResponse<Vec<AuditLogEntry>> = resp
        .json()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::Request("No data returned".to_string()))
}

/// Trigger garbage collection
pub async fn trigger_garbage_collection(token: &str) -> Result<MaintenanceResult, ApiError> {
    let url = format!("{}/admin/maintenance/gc", get_api_base_url());
    let resp = gloo_net::http::Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.ok() {
        return Err(ApiError::Request(format!(
            "GC failed: {}",
            resp.status()
        )));
    }

    let api_response: ApiResponse<MaintenanceResult> = resp
        .json()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))?;

    api_response
        .data
        .ok_or_else(|| ApiError::Request("No data returned".to_string()))
}
