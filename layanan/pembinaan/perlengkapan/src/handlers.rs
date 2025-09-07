//! # Request Handlers
//!
//! HTTP request handlers for the Perlengkapan service

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::*,
    models::*,
    services::PerlengkapanService,
    middleware::Claims,
};

// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 { 1 }
fn default_per_page() -> i32 { 20 }

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
    claims: Claims,
) -> Result<Json<ApiResponse<DashboardStats>>, AppError> {
    let stats = service.get_dashboard_stats().await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Dashboard statistics retrieved successfully".to_string(),
    )))
}

// Aset handlers
pub async fn get_all_aset(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<Aset>>, AppError> {
    let (aset, total) = service.get_all_aset(pagination.page, pagination.per_page).await?;

    Ok(Json(PaginatedResponse::new(
        aset,
        total,
        pagination.page,
        pagination.per_page,
        "Aset retrieved successfully".to_string(),
    )))
}

pub async fn get_aset_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<Aset>>, AppError> {
    let aset = service.get_aset_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        aset,
        "Aset retrieved successfully".to_string(),
    )))
}

pub async fn create_aset(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreateAsetRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Aset>>), AppError> {
    let user_id = Some(claims.user_id);
    let aset = service.create_aset(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            aset,
            "Aset created successfully".to_string(),
        )),
    ))
}

pub async fn update_aset(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateAsetRequest>,
) -> Result<Json<ApiResponse<Aset>>, AppError> {
    let user_id = Some(claims.user_id);
    let aset = service.update_aset(id, request, user_id).await?;

    Ok(Json(ApiResponse::success(
        aset,
        "Aset updated successfully".to_string(),
    )))
}

pub async fn delete_aset(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    service.delete_aset(id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(ApiResponse::success(
            "Aset deleted".to_string(),
            "Aset deleted successfully".to_string(),
        )),
    ))
}

// Pengadaan handlers
pub async fn get_all_pengadaan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<Pengadaan>>, AppError> {
    let (pengadaan, total) = service.get_all_pengadaan(pagination.page, pagination.per_page).await?;

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
    claims: Claims,
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

// Analisis Kebutuhan handlers
pub async fn get_all_analisis(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<AnalisisKebutuhan>>, AppError> {
    let (analisis, total) = service.get_all_analisis(pagination.page, pagination.per_page).await?;

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
