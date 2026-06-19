use super::params::*;
use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

/// POST /kebutuhan-bmn/satker/:id/validator-wilayah
/// Validator Wilayah forwards to Pusat or returns to Operator
pub async fn validator_wilayah_action(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("validator_wilayah")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .validator_wilayah_action(
            satker_id,
            request,
            Some(claims.user_id),
            Some(user_info),
            claims.role.clone(),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Aksi Validator Wilayah berhasil".to_string(),
    )))
}
/// POST /kebutuhan-bmn/satker/:id/keputusan-pusat
/// Validator Pusat makes final decision: approve or reject
pub async fn validator_pusat_keputusan(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorPusatKeputusanRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("validator_pusat")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .validator_pusat_keputusan(
            satker_id,
            request,
            Some(claims.user_id),
            Some(user_info),
            claims.role.clone(),
        )
        .await?;

    let message = if satker.status == KebutuhanBmnStatus::Approved {
        "Kebutuhan BMN disetujui oleh Validator Pusat"
    } else {
        "Kebutuhan BMN ditolak oleh Validator Pusat"
    };

    Ok(Json(ApiResponse::success(satker, message.to_string())))
}
