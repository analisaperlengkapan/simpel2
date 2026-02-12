//! # Mapping Kodefikasi Handlers
//!
//! Simplified read-only HTTP handlers for mapping kodefikasi.
//! Provides standard/non-standard BMN code listing, progress stats, and CSV export.
//! No proposal/verification workflow.

use crate::errors::AppError;
use crate::mapping_kodefikasi::models::*;
use crate::mapping_kodefikasi::services::MappingService;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    Json,
};

/// Detect non-standard codes from SIMAN data
///
/// GET /api/v1/mapping/detect
pub async fn detect_non_standard_codes(
    State(state): State<AppState>,
    Query(query): Query<MappingListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    let (codes, total) = service
        .list_non_standard_codes(query.search.as_deref(), query.satker_id, page, per_page)
        .await?;

    Ok((
        StatusCode::OK,
        Json(MappingListResponse {
            success: true,
            data: codes,
            total,
            page,
            per_page,
        }),
    ))
}

/// List standard BMN codes from master table
///
/// GET /api/v1/mapping/standard
pub async fn list_standard_codes(
    State(state): State<AppState>,
    Query(query): Query<MappingListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    let (codes, total) = service
        .list_standard_codes(query.search.as_deref(), page, per_page)
        .await?;

    Ok((
        StatusCode::OK,
        Json(MappingListResponse {
            success: true,
            data: codes,
            total,
            page,
            per_page,
        }),
    ))
}

/// Get mapping suggestions for a specific code name
///
/// GET /api/v1/mapping/suggestions?nama=...
#[derive(Debug, serde::Deserialize)]
pub struct SuggestionsQuery {
    pub nama: String,
}

pub async fn get_mapping_suggestions(
    State(state): State<AppState>,
    Query(query): Query<SuggestionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());
    let suggestions = service.get_mapping_suggestions(&query.nama).await?;
    Ok((StatusCode::OK, Json(suggestions)))
}

/// Get mapping progress statistics
///
/// GET /api/v1/mapping/progress
pub async fn get_mapping_progress(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());
    let progress = service.get_mapping_progress().await?;
    Ok((StatusCode::OK, Json(progress)))
}

/// Get mapping progress by satker
///
/// GET /api/v1/mapping/progress/satker
pub async fn get_mapping_progress_by_satker(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());
    let progress = service.get_mapping_progress_by_satker().await?;
    Ok((StatusCode::OK, Json(progress)))
}

/// Export mapping data as CSV
///
/// GET /api/v1/mapping/export?format=csv&satker_id=...
pub async fn export_mapping(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let csv_data = service.export_csv(query.satker_id).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"mapping_kodefikasi.csv\"",
            ),
        ],
        csv_data,
    ))
}
