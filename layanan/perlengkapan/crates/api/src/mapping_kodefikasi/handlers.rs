//! # Mapping Kodefikasi Handlers
//!
//! HTTP request handlers for mapping kodefikasi endpoints

use crate::errors::AppError;
use crate::mapping_kodefikasi::models::*;
use crate::mapping_kodefikasi::services::MappingService;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for non-standard codes detection
#[derive(Debug, Deserialize)]
pub struct DetectQuery {
    pub with_suggestions: Option<bool>,
}

/// Detect non-standard codes from SIMAN data
///
/// GET /api/v1/mapping/detect
pub async fn detect_non_standard_codes(
    State(state): State<AppState>,
    Query(query): Query<DetectQuery>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let codes = if query.with_suggestions.unwrap_or(false) {
        service.detect_non_standard_codes_with_suggestions().await?
    } else {
        service.detect_non_standard_codes().await?
    };

    Ok((StatusCode::OK, Json(codes)))
}

/// Get mapping suggestions for a specific code
///
/// GET /api/v1/mapping/suggestions?nama=...
#[derive(Debug, Deserialize)]
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

/// Create a mapping proposal
///
/// POST /api/v1/mapping/proposals
pub async fn create_mapping_proposal(
    State(state): State<AppState>,
    Json(request): Json<MappingProposalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let proposal = service.create_proposal(request).await?;

    // TODO: Send notification to Admin Pusat for verification
    // This would use the notifikasi service

    Ok((StatusCode::CREATED, Json(proposal)))
}

/// Get all mapping proposals
///
/// GET /api/v1/mapping/proposals
pub async fn get_all_mapping_proposals(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let proposals = service.get_all_proposals().await?;

    Ok((StatusCode::OK, Json(proposals)))
}

/// Get proposal by ID
///
/// GET /api/v1/mapping/proposals/:id
pub async fn get_mapping_proposal_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let proposal = service.get_proposal_by_id(id).await?;

    Ok((StatusCode::OK, Json(proposal)))
}

/// Verify a mapping proposal (Admin Pusat only)
///
/// PUT /api/v1/mapping/proposals/:id/verify
pub async fn verify_mapping_proposal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<VerifyMappingRequest>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let proposal = service.verify_proposal(id, request).await?;

    // TODO: Send notification to proposer about verification result
    // This would use the notifikasi service

    Ok((StatusCode::OK, Json(proposal)))
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

/// Get mapping progress by wilayah
///
/// GET /api/v1/mapping/progress/wilayah
pub async fn get_mapping_progress_by_wilayah(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let service = MappingService::new(state.db_pool.clone());

    let progress = service.get_mapping_progress_by_wilayah().await?;

    Ok((StatusCode::OK, Json(progress)))
}
