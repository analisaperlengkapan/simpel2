use super::params::*;
use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::services::KebutuhanBmnService;

/// GET /kebutuhan-bmn/siman/search
/// Search for existing assets from SIMAN inventory
pub async fn search_siman_assets(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SimanSearchQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<crate::kebutuhan_bmn::siman_integration::SimanAsset>>>, AppError> {
    if params.search.len() < 2 {
        return Err(bad_request("Search term must be at least 2 characters"));
    }

    let kategori_ref = params.kategori.as_deref();
    let assets = service
        .search_siman_assets(&params.search, kategori_ref, params.limit)
        .await?;

    let count = assets.len();
    Ok(Json(ApiResponse::success(
        assets,
        format!("Found {} matching assets", count),
    )))
}
/// GET /kebutuhan-bmn/siman/summary/:satker_id
/// Get asset summary from SIMAN for a specific satker
pub async fn get_siman_satker_summary(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<crate::kebutuhan_bmn::siman_integration::SatkerAssetSummary>>, AppError>
{
    let summary = service.get_satker_siman_summary(&satker_id).await?;

    Ok(Json(ApiResponse::success(
        summary,
        "SIMAN asset summary retrieved successfully".to_string(),
    )))
}
