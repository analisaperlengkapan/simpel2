//! Frontend client for `/api/pembinaan/perlengkapan/integrasi/*`.
//!
//! Backs the global circuit-breaker banner in `app_chrome`: when SIMAN /
//! MySIMKARI / MonSAKTI trip their breaker (Fase 2.2), the banner warns the
//! operator that data may be cached/stale instead of letting requests fail
//! silently.

use serde::Deserialize;

use crate::api::client::api_get;
use crate::api::error::AppResult;

#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrap<T> {
    pub success: bool,
    pub data: T,
}

/// One integrasi data source's breaker snapshot. Mirrors the backend
/// `IntegrasiCircuitStatus` in `shared::health`.
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrasiCircuitStatus {
    pub source: String,
    pub state: String,
    pub healthy: bool,
    pub label: String,
}

impl IntegrasiCircuitStatus {
    /// Friendly source name for the banner ("SIMAN", "MySIMKARI", …).
    pub fn source_display(&self) -> String {
        match self.source.as_str() {
            "mysimkari" => "MySIMKARI".to_string(),
            "siman" => "SIMAN".to_string(),
            "monsakti" => "MonSAKTI".to_string(),
            other => other.to_string(),
        }
    }
}

/// `GET /integrasi/circuit-status` — returns every source's breaker state.
/// Returns an empty vec when integrasi is not configured.
pub async fn fetch_circuit_status() -> AppResult<Vec<IntegrasiCircuitStatus>> {
    let resp: ApiResponseWrap<Vec<IntegrasiCircuitStatus>> =
        api_get("/integrasi/circuit-status").await?;
    if !resp.success {
        return Ok(Vec::new());
    }
    Ok(resp.data)
}
