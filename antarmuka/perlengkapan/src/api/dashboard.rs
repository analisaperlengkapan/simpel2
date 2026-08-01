//! Dashboard API — fetch stats from backend.

use super::client::api_get;
use super::types::{ApiResponse, DashboardStats};

/// Fetch dashboard statistics from `GET /dashboard/stats`.
pub async fn fetch_dashboard_stats() -> Result<DashboardStats, crate::api::AppError> {
    let resp: ApiResponse<DashboardStats> = api_get("/dashboard/stats").await?;
    Ok(resp.data)
}

/// Download the perlengkapan dashboard recap as `format` ("excel" | "pdf").
///
/// `tahun_anggaran` is REQUIRED by the endpoint (`DashboardParams.tahun_anggaran`
/// is a bare `i32`), so omitting it is a legitimate 400 — the caller always
/// passes one.
///
/// These two endpoints had no FE caller at all (#97), which is exactly why the
/// 500 they returned went unnoticed until an e2e probed them directly (#116).
#[cfg(target_arch = "wasm32")]
pub async fn export_dashboard(
    format: &str,
    tahun_anggaran: i32,
) -> Result<Vec<u8>, crate::api::AppError> {
    super::client::auth_get_binary(&format!(
        "{}/dashboard/perlengkapan/export/{format}?tahun_anggaran={tahun_anggaran}",
        super::client::API_BASE
    ))
    .await
}
