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

use crate::{errors::*, middleware::Claims, models::*, services::PerlengkapanService};

// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub category: Option<String>,
}

impl PaginationQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(bad_request("Page must be greater than 0"));
        }
        if self.page > 100_000 {
            return Err(bad_request("Page must be less than or equal to 100,000"));
        }
        if self.per_page < 1 {
            return Err(bad_request("Per page must be greater than 0"));
        }
        if self.per_page > 1000 {
            return Err(bad_request("Per page must be less than or equal to 1000"));
        }
        Ok(())
    }
}

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    20
}

// Health check handler
pub async fn health_check() -> Result<Json<ApiResponse<String>>, AppError> {
    Ok(Json(ApiResponse::success(
        "Service is healthy".to_string(),
        "Health check passed".to_string(),
    )))
}

// Dashboard handlers
pub async fn get_dashboard_stats(
    State(service): State<PerlengkapanService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<DashboardStats>>, AppError> {
    let stats = service.get_dashboard_stats().await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Dashboard statistics retrieved successfully".to_string(),
    )))
}

// Asset handlers (Read-Only)
pub async fn get_all_assets(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Asset>>, AppError> {
    pagination.validate()?;
    let (assets, total) = service
        .get_all_assets(pagination.page, pagination.per_page, pagination.category)
        .await?;

    Ok(Json(PaginatedResponse::new(
        assets,
        total,
        pagination.page,
        pagination.per_page,
        "Assets retrieved successfully".to_string(),
    )))
}

pub async fn get_asset_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Asset>>, AppError> {
    let asset = service.get_asset_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        asset,
        "Asset retrieved successfully".to_string(),
    )))
}

// Pengadaan handlers
pub async fn get_all_pengadaan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pengadaan>>, AppError> {
    pagination.validate()?;
    let (pengadaan, total) = service
        .get_all_pengadaan(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pengadaan,
        total,
        pagination.page,
        pagination.per_page,
        "Pengadaan retrieved successfully".to_string(),
    )))
}

pub async fn get_pengadaan_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Pengadaan>>, AppError> {
    let pengadaan = service.get_pengadaan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        pengadaan,
        "Pengadaan retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePengadaanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pengadaan>>), AppError> {
    let user_id = Some(claims.user_id);
    let pengadaan = service.create_pengadaan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pengadaan,
            "Pengadaan created successfully".to_string(),
        )),
    ))
}

// Pemakaian handlers
pub async fn get_all_pemakaian(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pemakaian>>, AppError> {
    pagination.validate()?;
    let (pemakaian, total) = service
        .get_all_pemakaian(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pemakaian,
        total,
        pagination.page,
        pagination.per_page,
        "Pemakaian retrieved successfully".to_string(),
    )))
}

pub async fn create_pemakaian(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePemakaianRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pemakaian>>), AppError> {
    let user_id = Some(claims.user_id);
    let pemakaian = service.create_pemakaian(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pemakaian,
            "Pemakaian recorded successfully".to_string(),
        )),
    ))
}

// Analisis Kebutuhan handlers
pub async fn get_all_analisis(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<AnalisisKebutuhan>>, AppError> {
    pagination.validate()?;
    let (analisis, total) = service
        .get_all_analisis(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        analisis,
        total,
        pagination.page,
        pagination.per_page,
        "Analisis kebutuhan retrieved successfully".to_string(),
    )))
}

pub async fn create_analisis(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreateAnalisisRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AnalisisKebutuhan>>), AppError> {
    let user_id = Some(claims.user_id);
    let analisis = service.create_analisis(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            analisis,
            "Analisis kebutuhan created successfully".to_string(),
        )),
    ))
}
