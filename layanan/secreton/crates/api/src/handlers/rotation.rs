//! Auto-Rotation Engine API handlers

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::rotation::{RotationHistory, RotationPolicy, RotationStatistics};

use super::AppState;

/// Create rotation routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/policies", post(create_policy))
        .route("/policies", get(list_policies))
        .route("/policies/:policy_id", get(get_policy))
        .route("/policies/:policy_id", post(update_policy))
        .route("/policies/:policy_id", delete(delete_policy))
        .route("/policies/:policy_id/execute", post(execute_rotation))
        .route("/history", get(get_history))
        .route("/history/:history_id/rollback", post(rollback_rotation))
        .route("/statistics", get(get_statistics))
        .route("/scheduler/start", post(start_scheduler))
        .route("/scheduler/stop", post(stop_scheduler))
}

/// Create rotation policy
#[tracing::instrument(skip(state, request))]
async fn create_policy(
    State(state): State<AppState>,
    Json(request): Json<RotationPolicy>,
) -> ApiResult<Json<ApiResponse<RotationPolicy>>> {
    info!("Creating rotation policy: {}", request.name);

    match state.rotation_engine.create_policy(request).await {
        Ok(policy) => {
            info!("Rotation policy created successfully");
            Ok(Json(ApiResponse::success(policy)))
        }
        Err(e) => {
            error!("Failed to create rotation policy: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create policy: {}",
                e
            )))
        }
    }
}

/// Get rotation policy
#[tracing::instrument(skip(state))]
async fn get_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<RotationPolicy>>> {
    info!("Getting rotation policy: {}", policy_id);

    match state.rotation_engine.get_policy(&policy_id).await {
        Ok(policy) => Ok(Json(ApiResponse::success(policy))),
        Err(e) => {
            error!("Failed to get rotation policy: {:?}", e);
            Err(ApiError::not_found(format!("Policy not found: {}", e)))
        }
    }
}

/// List rotation policies
#[tracing::instrument(skip(state))]
async fn list_policies(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<PoliciesListResponse>>> {
    info!("Listing rotation policies");

    let policies = state.rotation_engine.list_policies().await;
    Ok(Json(ApiResponse::success(PoliciesListResponse {
        policies,
    })))
}

/// Update rotation policy
#[tracing::instrument(skip(state, request))]
async fn update_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
    Json(mut request): Json<RotationPolicy>,
) -> ApiResult<Json<ApiResponse<RotationPolicy>>> {
    info!("Updating rotation policy: {}", policy_id);

    // Ensure policy ID matches
    request.id = policy_id;

    match state.rotation_engine.update_policy(request).await {
        Ok(policy) => {
            info!("Rotation policy updated successfully");
            Ok(Json(ApiResponse::success(policy)))
        }
        Err(e) => {
            error!("Failed to update rotation policy: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to update policy: {}",
                e
            )))
        }
    }
}

/// Delete rotation policy
#[tracing::instrument(skip(state))]
async fn delete_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting rotation policy: {}", policy_id);

    match state.rotation_engine.delete_policy(&policy_id).await {
        Ok(_) => {
            info!("Rotation policy deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete rotation policy: {:?}", e);
            Err(ApiError::not_found(format!(
                "Failed to delete policy: {}",
                e
            )))
        }
    }
}

/// Execute rotation for a policy
#[tracing::instrument(skip(state))]
async fn execute_rotation(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
    Json(request): Json<ExecuteRotationRequest>,
) -> ApiResult<Json<ApiResponse<RotationHistory>>> {
    info!("Executing rotation for policy: {}", policy_id);

    let triggered_by = request.triggered_by.unwrap_or_else(|| "manual".to_string());

    match state
        .rotation_engine
        .execute_rotation(&policy_id, triggered_by)
        .await
    {
        Ok(history) => {
            info!("Rotation executed successfully");
            Ok(Json(ApiResponse::success(history)))
        }
        Err(e) => {
            error!("Failed to execute rotation: {:?}", e);
            Err(ApiError::bad_request(format!("Rotation failed: {}", e)))
        }
    }
}

/// Get rotation history
#[tracing::instrument(skip(state))]
async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQueryParams>,
) -> ApiResult<Json<ApiResponse<HistoryResponse>>> {
    info!("Getting rotation history");

    let history = state.rotation_engine.get_history(params.limit).await;
    Ok(Json(ApiResponse::success(HistoryResponse { history })))
}

/// Rollback rotation
#[tracing::instrument(skip(state))]
async fn rollback_rotation(
    State(state): State<AppState>,
    Path(history_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Rolling back rotation: {}", history_id);

    match state.rotation_engine.rollback_rotation(&history_id).await {
        Ok(_) => {
            info!("Rotation rolled back successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to rollback rotation: {:?}", e);
            Err(ApiError::bad_request(format!("Rollback failed: {}", e)))
        }
    }
}

/// Get rotation statistics
#[tracing::instrument(skip(state))]
async fn get_statistics(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<RotationStatistics>>> {
    info!("Getting rotation statistics");

    let stats = state.rotation_engine.get_statistics().await;
    Ok(Json(ApiResponse::success(stats)))
}

/// Start rotation scheduler
#[tracing::instrument(skip(state))]
async fn start_scheduler(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SchedulerStatusResponse>>> {
    info!("Starting rotation scheduler");

    state.rotation_engine.start_scheduler().await;

    Ok(Json(ApiResponse::success(SchedulerStatusResponse {
        running: true,
        message: "Rotation scheduler started".to_string(),
    })))
}

/// Stop rotation scheduler
#[tracing::instrument(skip(state))]
async fn stop_scheduler(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SchedulerStatusResponse>>> {
    info!("Stopping rotation scheduler");

    state.rotation_engine.stop_scheduler().await;

    Ok(Json(ApiResponse::success(SchedulerStatusResponse {
        running: false,
        message: "Rotation scheduler stopped".to_string(),
    })))
}

/// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliciesListResponse {
    pub policies: Vec<RotationPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRotationRequest {
    pub triggered_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQueryParams {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub history: Vec<RotationHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatusResponse {
    pub running: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    #[ignore = "Requires database/infrastructure"]
    async fn test_list_policies() {
        let config = ApiConfig::default();
        let services = ServiceContainer::new(&config).await.unwrap();
        let app = create_routes().with_state(services.into());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/policies")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
