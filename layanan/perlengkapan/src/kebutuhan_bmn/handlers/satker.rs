use super::params::*;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

pub async fn add_satker_to_pengajuan(
    State(service): State<KebutuhanBmnService>,
    Path(pengajuan_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<AddSatkerRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanKebutuhanBmnSatker>>), AppError> {
    info!(
        "Adding satker {} to pengajuan {}",
        request.satker_id, pengajuan_id
    );

    let user_id = Some(claims.user_id);
    let satker = service
        .add_satker_to_pengajuan(
            pengajuan_id,
            &request.satker_id,
            request.satker_name,
            user_id,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            satker,
            "Satker added successfully".to_string(),
        )),
    ))
}
/// GET /kebutuhan-bmn/satker/:id
/// Get satker detail with barang list
pub async fn get_satker_detail(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<SatkerWithBarangResponse>>, AppError> {
    pagination.validate()?;

    let response = service
        .get_satker_with_barang(satker_id, pagination.page, pagination.per_page)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Satker detail retrieved successfully".to_string(),
    )))
}
/// POST /kebutuhan-bmn/satker/:id/transition
/// Transition satker to a new workflow status
pub async fn transition_satker_status(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    // Coarse-grained gate; the workflow engine enforces the per-state required role.
    claims.require_any_role(&[
        "operator_satker",
        "validator_wilayah",
        "validator_pusat",
        "admin",
    ])?;
    info!(
        "Transitioning satker {} to status {}",
        satker_id, request.target_status
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let satker = service
        .transition_satker_status(satker_id, request, user_id, user_info, claims.role.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Satker status updated successfully".to_string(),
    )))
}
/// GET /kebutuhan-bmn/satker/:id/aktivitas
/// Get workflow history for a satker
pub async fn get_satker_aktivitas(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>>, AppError> {
    let aktivitas = service.get_satker_aktivitas(satker_id).await?;

    Ok(Json(ApiResponse::success(
        aktivitas,
        "Aktivitas retrieved successfully".to_string(),
    )))
}
// ============================================================================
// Satker Workflow Handlers (Validator Wilayah & Pusat)
// ============================================================================

/// POST /kebutuhan-bmn/satker/:id/submit-wilayah
/// Operator Satker submits pengajuan to Validator Wilayah
pub async fn submit_satker_to_wilayah(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<SubmitKebutuhanSatkerRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("operator_satker")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .submit_satker_to_wilayah(
            satker_id,
            request,
            Some(claims.user_id),
            Some(user_info),
            claims.role.clone(),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Pengajuan berhasil dikirim ke Validator Wilayah".to_string(),
    )))
}
