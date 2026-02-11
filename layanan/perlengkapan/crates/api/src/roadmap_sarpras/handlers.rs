//! Roadmap Sarpras HTTP handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lib_perlengkapan::models::{
    CreateRoadmapSarprasRequest, UpdateRoadmapRealizationRequest,
};
use serde_json::json;
use uuid::Uuid;

use crate::errors::AppError;

use super::models::{
    CreateRoadmapBatchRequest, ListRoadmapQuery, RoadmapComparisonQuery, SyncRealizationRequest,
};
use super::services::RoadmapService;

/// Create a single roadmap item
///
/// POST /api/v1/roadmap-sarpras
pub async fn create_roadmap(
    State(service): State<RoadmapService>,
    Json(request): Json<CreateRoadmapSarprasRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract user_id from JWT token
    let created_by = Uuid::nil(); // Placeholder

    let roadmap = service.create_roadmap(request, created_by).await?;

    Ok((StatusCode::CREATED, Json(roadmap)))
}

/// Create multiple roadmap items in a batch
///
/// POST /api/v1/roadmap-sarpras/batch
pub async fn create_roadmap_batch(
    State(service): State<RoadmapService>,
    Json(request): Json<CreateRoadmapBatchRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract user_id from JWT token
    let created_by = Uuid::nil(); // Placeholder

    let response = service.create_roadmap_batch(request, created_by).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get roadmap by ID
///
/// GET /api/v1/roadmap-sarpras/:id
pub async fn get_roadmap(
    State(service): State<RoadmapService>,
    Path(roadmap_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let roadmap = service.get_roadmap(roadmap_id).await?;

    Ok(Json(roadmap))
}

/// List roadmaps with filters and pagination
///
/// GET /api/v1/roadmap-sarpras
pub async fn list_roadmaps(
    State(service): State<RoadmapService>,
    Query(query): Query<ListRoadmapQuery>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.list_roadmaps(query).await?;

    Ok(Json(response))
}

/// Update roadmap realization
///
/// PUT /api/v1/roadmap-sarpras/:id/realization
pub async fn update_realization(
    State(service): State<RoadmapService>,
    Path(roadmap_id): Path<Uuid>,
    Json(request): Json<UpdateRoadmapRealizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: Extract user_id from JWT token
    let updated_by = Uuid::nil(); // Placeholder

    let roadmap = service
        .update_realization(roadmap_id, request, updated_by)
        .await?;

    Ok(Json(roadmap))
}

/// Sync realization increment (internal endpoint for MonSAKTI integration)
///
/// POST /api/v1/roadmap-sarpras/sync-realization
pub async fn sync_realization_increment(
    State(service): State<RoadmapService>,
    Json(request): Json<SyncRealizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let roadmap = service.sync_realization_increment(request).await?;

    Ok(Json(roadmap))
}

/// Get roadmap vs realization comparison
///
/// GET /api/v1/roadmap-sarpras/comparison
pub async fn get_comparison(
    State(service): State<RoadmapService>,
    Query(query): Query<RoadmapComparisonQuery>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.get_comparison(query).await?;

    Ok(Json(response))
}

/// Delete roadmap
///
/// DELETE /api/v1/roadmap-sarpras/:id
pub async fn delete_roadmap(
    State(service): State<RoadmapService>,
    Path(roadmap_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_roadmap(roadmap_id).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Roadmap deleted successfully"
        })),
    ))
}

/// Register roadmap routes
pub fn roadmap_routes() -> axum::Router<RoadmapService> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/", post(create_roadmap))
        .route("/", get(list_roadmaps))
        .route("/batch", post(create_roadmap_batch))
        .route("/comparison", get(get_comparison))
        .route("/sync-realization", post(sync_realization_increment))
        .route("/:id", get(get_roadmap))
        .route("/:id", delete(delete_roadmap))
        .route("/:id/realization", put(update_realization))
}
