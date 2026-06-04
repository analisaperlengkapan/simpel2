//! # Request Handlers
//!
//! HTTP request handlers for the Perlengkapan service

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::shared::error::*;
use crate::shared::middleware::Claims;
use crate::{models::*, services::PerlengkapanService};

// Health check handler
pub async fn health_check() -> Result<Json<ApiResponse<String>>, AppError> {
    Ok(Json(ApiResponse::success(
        "Service is healthy".to_string(),
        "Health check passed".to_string(),
    )))
}

// ============================================================================
// Export Handlers
// ============================================================================

use axum::response::{IntoResponse, Response};

/// Export query parameters
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// Entity type to export (kebutuhan_bmn, pakaian_dinas, etc.)
    pub entity_type: String,
    /// JSON-encoded filters
    pub filters: Option<String>,
    /// Maximum number of rows to export
    pub limit: Option<u32>,
    /// Tahun anggaran filter
    pub tahun_anggaran: Option<i32>,
    /// Satker ID filter
    pub satker_id: Option<Uuid>,
    /// Status filter
    pub status: Option<String>,
}

/// Export job response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Export job status response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: Option<f32>,
    pub document_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Export to Excel handler
///
/// For small datasets (<1000 rows), returns Excel file synchronously.
/// For large datasets (>=1000 rows), queues an async export job.
pub async fn export_to_excel(
    State(service): State<PerlengkapanService>,
    Query(query): Query<ExportQuery>,
    _claims: Claims,
) -> Result<Response, AppError> {
    // Validate entity type
    validate_entity_type(&query.entity_type)?;

    // Limit export size to 50,000 rows maximum
    let limit = query.limit.unwrap_or(50000).min(50000);

    // Check if async export needed (>1000 rows)
    if limit > 1000 {
        // Queue async export job
        let job_id = service.queue_export_job(query).await?;

        return Ok((
            StatusCode::ACCEPTED,
            Json(ExportJobResponse {
                job_id,
                status: "queued".to_string(),
                message: "Export job queued. You will be notified when ready.".to_string(),
            }),
        )
            .into_response());
    }

    // Synchronous export for small datasets
    let excel_data = service.export_to_excel_sync(query).await?;

    Ok((
        StatusCode::OK,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                &format!(
                    "attachment; filename=\"export_{}.xlsx\"",
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                ),
            ),
        ],
        excel_data,
    )
        .into_response())
}

/// Get export job status
pub async fn get_export_job_status(
    State(service): State<PerlengkapanService>,
    Path(job_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<ExportJobStatusResponse>>, AppError> {
    let status = service.get_export_job_status(job_id).await?;

    Ok(Json(ApiResponse::success(
        status,
        "Export job status retrieved successfully".to_string(),
    )))
}

/// Download completed export job
pub async fn download_export_job(
    State(service): State<PerlengkapanService>,
    Path(job_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Response, AppError> {
    let (filename, data) = service.download_export_job(job_id).await?;

    Ok((
        StatusCode::OK,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        data,
    )
        .into_response())
}

/// Validate entity type
fn validate_entity_type(entity_type: &str) -> Result<(), AppError> {
    match entity_type {
        "kebutuhan_bmn" | "pakaian_dinas" | "roadmap_sarpras" | "riwayat_pemenuhan" => Ok(()),
        _ => Err(bad_request(&format!(
            "Invalid entity type: {}. Supported types: kebutuhan_bmn, pakaian_dinas, roadmap_sarpras, riwayat_pemenuhan",
            entity_type
        ))),
    }
}
