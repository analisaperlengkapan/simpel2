// ============================================================================
// Penghapusan BMN Handlers
// Description: HTTP request handlers for BMN disposal
// Requirements: REQ-W001, REQ-W004
// ============================================================================

use super::models::*;
use super::services::PenghapusanBmnService;
use crate::errors::AppError;
use crate::middleware::Claims;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Pagination query parameters
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1))]
    pub page: Option<i32>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<i32>,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub message: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i32, per_page: i32, message: String) -> Self {
        Self {
            success: true,
            data,
            total,
            page,
            per_page,
            message,
        }
    }
}

/// Create penghapusan BMN
pub async fn create_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    claims: Claims,
    Json(request): Json<CreatePenghapusanBmnRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PenghapusanBmn>>), AppError> {
    request.validate()?;

    let penghapusan = service.create(request, claims.user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            penghapusan,
            "Penghapusan BMN created successfully".to_string(),
        )),
    ))
}

/// Get penghapusan BMN by ID
pub async fn get_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    let penghapusan = service.get_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Penghapusan BMN retrieved successfully".to_string(),
    )))
}

/// List penghapusan BMN with filters
#[derive(Debug, Deserialize)]
pub struct ListPenghapusanQuery {
    pub satker_id: Option<Uuid>,
    pub status: Option<String>,
    pub metode_penghapusan: Option<String>,
    pub tahun: Option<i32>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub async fn list_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Query(query): Query<ListPenghapusanQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<PenghapusanBmn>>, AppError> {
    let filters = PenghapusanBmnFilters {
        satker_id: query.satker_id,
        status: query.status,
        metode_penghapusan: query.metode_penghapusan,
        tahun: query.tahun,
    };

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let (penghapusan, total) = service.list(filters, page, per_page).await?;

    Ok(Json(PaginatedResponse::new(
        penghapusan,
        total,
        page,
        per_page,
        "Penghapusan BMN list retrieved successfully".to_string(),
    )))
}

/// Update penghapusan BMN
pub async fn update_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
    Json(request): Json<UpdatePenghapusanBmnRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    request.validate()?;

    let penghapusan = service.update(id, request).await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Penghapusan BMN updated successfully".to_string(),
    )))
}

/// Delete penghapusan BMN
pub async fn delete_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    service.delete(id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            (),
            "Penghapusan BMN deleted successfully".to_string(),
        )),
    ))
}

/// Workflow transition request
#[derive(Debug, Deserialize, Validate)]
pub struct TransitionRequest {
    #[validate(length(min = 1))]
    pub to_state: String,
    pub catatan: Option<String>,
}

/// Perform workflow transition
pub async fn transition_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<TransitionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    request.validate()?;

    // Get IP address from request (in production, extract from headers)
    let ip_address = "127.0.0.1".to_string();

    let penghapusan = service
        .transition(id, request.to_state, claims.user_id, request.catatan, ip_address)
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Workflow transition completed successfully".to_string(),
    )))
}

/// Get document for penghapusan BMN
pub async fn get_penghapusan_document(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let penghapusan = service.get_by_id(id).await?;

    let document_url = penghapusan
        .document_url
        .ok_or_else(|| AppError::NotFound("Document not found for this penghapusan".to_string()))?;

    Ok(Json(ApiResponse::success(
        document_url,
        "Document URL retrieved successfully".to_string(),
    )))
}
