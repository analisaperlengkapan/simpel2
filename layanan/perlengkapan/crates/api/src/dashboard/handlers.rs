// Dashboard HTTP handlers

use crate::dashboard::models::*;
use crate::errors::AppError;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
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
) -> Result<Json<PerlengkapanDashboardMetrics>, AppError> {
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params)
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
) -> Result<impl IntoResponse, AppError> {
    // Get dashboard metrics
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params)
        .await?;

    // Generate Excel file
    let excel_bytes = state
        .dashboard_service
        .export_dashboard_to_excel(&metrics, params.tahun_anggaran)
        .await?;

    let filename = format!(
        "dashboard_perlengkapan_{}.xlsx",
        params.tahun_anggaran
    );

    let mut response = axum::response::Response::new(axum::body::Body::from(excel_bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
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
) -> Result<impl IntoResponse, AppError> {
    // Get dashboard metrics
    let metrics = state
        .dashboard_service
        .get_perlengkapan_dashboard_metrics(&params)
        .await?;

    // Generate PDF file
    let pdf_bytes = state
        .dashboard_service
        .export_dashboard_to_pdf(&metrics, params.tahun_anggaran)
        .await?;

    let filename = format!(
        "dashboard_perlengkapan_{}.pdf",
        params.tahun_anggaran
    );

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
