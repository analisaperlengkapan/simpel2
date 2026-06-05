//! # Export Handlers
//!
//! HTTP handlers for the generic Excel export feature.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::export::models::{ExportJobResponse, ExportJobStatusResponse, ExportQuery};
use crate::export::services::ExportService;
use crate::shared::error::*;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

/// Export to Excel handler
///
/// For small datasets (<1000 rows), returns Excel file synchronously.
/// For large datasets (>=1000 rows), queues an async export job.
pub async fn export_to_excel(
    State(service): State<ExportService>,
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
    State(service): State<ExportService>,
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
    State(service): State<ExportService>,
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
pub(crate) fn validate_entity_type(entity_type: &str) -> Result<(), AppError> {
    match entity_type {
        "kebutuhan_bmn" | "pakaian_dinas" | "roadmap_sarpras" | "riwayat_pemenuhan" => Ok(()),
        _ => Err(bad_request(&format!(
            "Invalid entity type: {}. Supported types: kebutuhan_bmn, pakaian_dinas, roadmap_sarpras, riwayat_pemenuhan",
            entity_type
        ))),
    }
}
