//! # Pemakaian BMN HTTP Handlers
//!
//! Request handlers for BMN usage permit REST API.
//! All endpoints require authentication via JWT middleware.

use axum::{extract::{Path, State}, Json, http::StatusCode};
use tracing::info;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::middleware::Claims;
use crate::models::ApiResponse;

use super::models::*;
use super::services::PemakaianBmnService;

/// GET /pemakaian-bmn/:id
/// Get a single permit with full details
pub async fn get_permit_by_id(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianDetailResponse>>, AppError> {
    let response = service.get_permit_detail(id).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Permit retrieved successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn
/// Create a new permit request
pub async fn create_permit(
    State(service): State<PemakaianBmnService>,
    claims: Claims,
    Json(request): Json<CreateIzinPemakaianRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    info!("Creating new permit for BMN {}", request.bmn_nup);

    let permit = service
        .create_permit(request, claims.user_id, claims.username.clone())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit created successfully".to_string(),
        )),
    ))
}

/// PUT /pemakaian-bmn/:id
/// Update a permit (only in DRAFT status)
pub async fn update_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateIzinPemakaianRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!("Updating permit {}", id);

    let permit = service
        .update_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit updated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn
/// List permits with pagination and filters
pub async fn list_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<ListPermitsQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PaginatedPermitsResponse>>, AppError> {
    let response = service.list_permits(query).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Permits retrieved successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/transition
/// Transition permit to a new workflow status
pub async fn transition_permit_status(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!(
        "Transitioning permit {} to status {}",
        id, request.target_status
    );

    let permit = service
        .transition_permit_status(id, request, claims.user_id)
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Status updated successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/activate
/// Activate a permit (generate permit number and set to ACTIVE)
pub async fn activate_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!("Activating permit {}", id);

    let permit = service.activate_permit(id, claims.user_id).await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit activated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/:id/document
/// Download permit document
///
/// Requirements: REQ-P006, REQ-D005
pub async fn get_permit_document(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<axum::response::Redirect, AppError> {
    info!("Fetching document for permit {}", id);

    let permit = service.get_permit_detail(id).await?;

    // Verify user has access to this permit
    // User can access if they created it or are in the same satker
    if permit.izin.created_by != claims.user_id {
        // TODO: Add satker-based authorization check
        // For now, allow all authenticated users
    }

    // Check if document exists - try new konsep_surat_url, then legacy document_url
    let document_url = permit.izin.konsep_surat_url
        .or(permit.izin.document_url)
        .ok_or_else(|| AppError::NotFound("Document not found for this permit".to_string()))?;

    // Redirect to document URL
    Ok(axum::response::Redirect::temporary(&document_url))
}

/// POST /pemakaian-bmn/:id/generate-konsep-surat
/// Generate konsep surat izin pemakaian BMN (DOCX)
pub async fn generate_konsep_surat(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<crate::models::ApiResponse<IzinPemakaianBmn>>, AppError> {
    let permit = service.generate_konsep_surat(id, claims.user_id).await?;

    Ok(Json(crate::models::ApiResponse::success(
        permit,
        "Konsep surat izin pemakaian BMN berhasil digenerate".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/upload-signed-pdf
/// Upload signed PDF izin pemakaian and mark as completed
pub async fn upload_signed_pdf(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<UploadSignedPdfRequest>,
) -> Result<Json<crate::models::ApiResponse<IzinPemakaianBmn>>, AppError> {
    let permit = service
        .upload_signed_pdf(id, body.signed_pdf_url, claims.user_id)
        .await?;

    Ok(Json(crate::models::ApiResponse::success(
        permit,
        "PDF izin pemakaian BMN yang ditandatangani berhasil diupload. Proses selesai.".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/revoke
/// Revoke a permit
pub async fn revoke_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RevokePermitRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!("Revoking permit {}", id);

    let permit = service
        .revoke_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit revoked successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/renew
/// Renew a permit (create new permit based on existing one)
pub async fn renew_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RenewPermitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    info!("Renewing permit {}", id);

    let permit = service
        .renew_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit renewed successfully".to_string(),
        )),
    ))
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/availability
/// Check if a BMN is available for new permit
pub async fn check_bmn_availability(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BmnAvailabilityResponse>>, AppError> {
    let response = service.check_bmn_availability(&bmn_nup).await?;

    let message = if response.is_available {
        "BMN is available".to_string()
    } else {
        "BMN is currently in use".to_string()
    };

    Ok(Json(ApiResponse::success(response, message)))
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/history
/// Get BMN usage history
pub async fn get_bmn_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BmnUsageStats>>, AppError> {
    let stats = service.get_bmn_usage_history(&bmn_nup).await?;

    Ok(Json(ApiResponse::success(
        stats,
        "BMN usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/pegawai/:pegawai_nip/history
/// Get pegawai usage history
pub async fn get_pegawai_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(pegawai_nip): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PegawaiUsageStats>>, AppError> {
    let stats = service.get_pegawai_usage_history(&pegawai_nip).await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Pegawai usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/expiring
/// Get permits expiring soon (for notifications)
pub async fn get_expiring_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<IzinPemakaianBmn>>>, AppError> {
    let days_threshold = params
        .get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(30);

    let permits = service.get_expiring_permits(days_threshold).await?;
    let count = permits.len();

    Ok(Json(ApiResponse::success(
        permits,
        format!("Found {} permits expiring within {} days", count, days_threshold),
    )))
}

/// POST /pemakaian-bmn/auto-expire
/// Auto-expire permits (admin/scheduler endpoint)
pub async fn auto_expire_permits(
    State(service): State<PemakaianBmnService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<usize>>, AppError> {
    let count = service.auto_expire_permits().await?;

    Ok(Json(ApiResponse::success(
        count,
        format!("Auto-expired {} permits", count),
    )))
}

// ============================================================================
// Monitoring Dashboard Handlers
// ============================================================================

/// GET /pemakaian-bmn/monitoring/active-usage
/// Get active usage monitoring dashboard
/// Requirements: REQ-P011
pub async fn get_active_usage_dashboard(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<ActiveUsageMonitoringDashboard>>, AppError> {
    let dashboard = service.get_active_usage_dashboard(query).await?;

    Ok(Json(ApiResponse::success(
        dashboard,
        "Active usage dashboard retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/monitoring/utilization-report
/// Get BMN utilization report
/// Requirements: REQ-P013
pub async fn get_bmn_utilization_report(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BmnUtilizationReport>>, AppError> {
    let report = service.get_bmn_utilization_report(query).await?;

    Ok(Json(ApiResponse::success(
        report,
        "BMN utilization report generated successfully".to_string(),
    )))
}
