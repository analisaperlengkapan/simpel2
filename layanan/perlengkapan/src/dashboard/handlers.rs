// Dashboard HTTP handlers

use crate::AppState;
use crate::bank_aset::AsetScope;
use crate::dashboard::models::*;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::shared::satker_scope::SatkerScope;
use axum::{
    Json,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};

/// GET /api/v1/dashboard/perlengkapan
///
/// Fetch complete perlengkapan dashboard metrics
///
/// Query parameters:
/// - tahun_anggaran: Fiscal year (required)
///
/// Returns:
/// - PerlengkapanDashboardMetrics with all aggregated metrics
pub async fn get_perlengkapan_dashboard_metrics(
    State(state): State<AppState>,
    Query(params): Query<DashboardParams>,
    // There is no auth middleware layer on this router — `Claims` IS the gate
    // (see shared/middleware) — so omitting it published this data to anyone
    // who could reach the service. Taking `Claims` closed that; it did NOT
    // scope the rows, which is a separate question and stayed open: measured on
    // staging, an operator at 0200010 read `total_by_satker` naming Jakarta
    // Selatan and Bandung.
    claims: Claims,
) -> Result<Json<PerlengkapanDashboardMetrics>, AppError> {
    let scope = SatkerScope::from_claims(&claims);
    let aset_scope = AsetScope::from_claims(&claims);
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params, &scope, &aset_scope)
        .await?;

    Ok(Json(metrics))
}

/// GET /api/v1/dashboard/perlengkapan/export/excel
///
/// Export dashboard metrics to Excel
///
/// Query parameters:
/// - tahun_anggaran: Fiscal year (required)
/// - format: Export format (excel or pdf, default: excel)
///
/// Returns:
/// - Excel file download
pub async fn export_dashboard_excel(
    State(state): State<AppState>,
    Query(params): Query<DashboardParams>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    // Same scope as the on-screen dashboard: an export that widened what the
    // page shows would be the leak all over again in a downloadable form.
    let scope = SatkerScope::from_claims(&claims);
    let aset_scope = AsetScope::from_claims(&claims);
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params, &scope, &aset_scope)
        .await?;

    // Generate Excel file
    let excel_bytes = state
        .dashboard_service
        .export_dashboard_to_excel(&metrics, params.tahun_anggaran)
        .await?;

    let filename = format!("dashboard_perlengkapan_{}.xlsx", params.tahun_anggaran);

    let mut response = axum::response::Response::new(axum::body::Body::from(excel_bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| AppError::Internal("Invalid header value".to_string()))?,
    );

    Ok(response)
}

/// GET /api/v1/dashboard/perlengkapan/export/pdf
///
/// Export dashboard metrics to PDF
///
/// Query parameters:
/// - tahun_anggaran: Fiscal year (required)
///
/// Returns:
/// - PDF file download
pub async fn export_dashboard_pdf(
    State(state): State<AppState>,
    Query(params): Query<DashboardParams>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    // Same scope as the on-screen dashboard: an export that widened what the
    // page shows would be the leak all over again in a downloadable form.
    let scope = SatkerScope::from_claims(&claims);
    let aset_scope = AsetScope::from_claims(&claims);
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params, &scope, &aset_scope)
        .await?;

    // Generate PDF file
    let pdf_bytes = state
        .dashboard_service
        .export_dashboard_to_pdf(&metrics, params.tahun_anggaran)
        .await?;

    let filename = format!("dashboard_perlengkapan_{}.pdf", params.tahun_anggaran);

    let mut response = axum::response::Response::new(axum::body::Body::from(pdf_bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pdf"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| AppError::Internal("Invalid header value".to_string()))?,
    );

    Ok(response)
}
