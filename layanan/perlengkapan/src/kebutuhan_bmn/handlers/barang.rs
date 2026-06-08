use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

// ============================================================================
// Barang Handlers
// ============================================================================

/// POST /kebutuhan-bmn/satker/:id/barang
/// Add a barang to a satker
pub async fn create_barang(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<CreateBarangRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanKebutuhanBmnBarang>>), AppError> {
    info!("Creating barang for satker {}: {}", satker_id, request.nama);

    let user_id = Some(claims.user_id);
    let barang = service.create_barang(satker_id, request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            barang,
            "Barang created successfully".to_string(),
        )),
    ))
}
/// PUT /kebutuhan-bmn/barang/:id/approval
/// Update barang approval (jml_setuju)
pub async fn update_barang_approval(
    State(service): State<KebutuhanBmnService>,
    Path(barang_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateBarangApprovalRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnBarang>>, AppError> {
    info!("Updating barang approval: {}", barang_id);

    let user_id = Some(claims.user_id);
    let barang = service
        .update_barang_approval(barang_id, request, user_id)
        .await?;

    Ok(Json(ApiResponse::success(
        barang,
        "Barang approval updated successfully".to_string(),
    )))
}
/// DELETE /kebutuhan-bmn/barang/:id
/// Delete a barang
pub async fn delete_barang(
    State(service): State<KebutuhanBmnService>,
    Path(barang_id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!("Deleting barang: {}", barang_id);

    let user_id = Some(claims.user_id);
    service.delete_barang(barang_id, user_id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Barang deleted successfully".to_string(),
    )))
}
/// POST /kebutuhan-bmn/prioritas
/// Set priorities for multiple barang
pub async fn set_barang_prioritas(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    Json(request): Json<SetPrioritasRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!(
        "Setting prioritas for {} items by user {}",
        request.items.len(),
        claims.user_id
    );

    service.set_barang_prioritas(request).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Prioritas updated successfully".to_string(),
    )))
}
