//! Dashboard API — fetch stats from backend.

use super::client::api_get;
use super::types::{ApiResponse, DashboardStats};

/// Fetch dashboard statistics from `GET /dashboard/stats`.
pub async fn fetch_dashboard_stats() -> Result<DashboardStats, crate::api::AppError> {
    let resp: ApiResponse<DashboardStats> = api_get("/dashboard/stats").await?;
    Ok(resp.data)
}
