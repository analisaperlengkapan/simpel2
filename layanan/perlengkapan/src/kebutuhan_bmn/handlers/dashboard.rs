use axum::{Json, extract::State};
use tracing::debug;

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

// ============================================================================
// Dashboard Handlers
// ============================================================================

/// GET /kebutuhan-bmn/dashboard
/// Get dashboard statistics for kebutuhan BMN
pub async fn get_dashboard_stats(
    State(service): State<KebutuhanBmnService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<KebutuhanBmnDashboardStats>>, AppError> {
    debug!("Getting kebutuhan BMN dashboard stats");

    let stats = service.get_dashboard_stats().await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Dashboard statistics retrieved successfully".to_string(),
    )))
}
