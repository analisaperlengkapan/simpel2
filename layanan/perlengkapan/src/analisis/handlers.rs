//! HTTP handlers for Analisis Kebutuhan.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};
use uuid::Uuid;

use super::models::{AnalisisKebutuhan, CreateAnalisisRequest};
use super::services::AnalisisService;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::shared::pagination::PaginationQuery;

pub async fn get_all_analisis(
    State(service): State<AnalisisService>,
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

pub async fn get_analisis_by_id(
    State(service): State<AnalisisService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<AnalisisKebutuhan>>, AppError> {
    let analisis = service.get_analisis_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        analisis,
        "Analisis kebutuhan retrieved successfully".to_string(),
    )))
}

pub async fn create_analisis(
    State(service): State<AnalisisService>,
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
