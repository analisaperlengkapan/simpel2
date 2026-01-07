pub use shared_microfrontend::utils::api::{
    ApiError, ApiErrorResponse, ApiResponse, ApiResponseData,
};

use crate::types::{AuditLogEntry, MaintenanceResult, SystemMetricsResponse};

/// Base API URL - injected at build time or defaults to localhost
const API_BASE_URL: &str = option_env!("API_BASE_URL").unwrap_or("http://localhost:8080/v1");

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
    let url = format!("{}/admin/metrics", API_BASE_URL);
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
    let url = format!("{}/admin/logs?limit={}", API_BASE_URL, limit);
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
    let url = format!("{}/admin/maintenance/gc", API_BASE_URL);
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
