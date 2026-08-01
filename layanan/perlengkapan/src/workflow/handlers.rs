//! Workflow Monitoring API Handlers
//!
//! REST API endpoints for workflow monitoring dashboard

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::workflow::monitoring::{WorkflowMonitor, WorkflowSummary};

// ═══════════════════════════════════════════════════════════════════════════
// Response Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: "Success".to_string(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Handler Functions
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/v1/workflow/monitoring/metrics
///
/// Get overall workflow metrics for monitoring dashboard
pub async fn get_workflow_metrics(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    let metrics = monitor
        .get_metrics()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get metrics: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(metrics))))
}

/// GET /api/v1/workflow/monitoring/active
///
/// Get list of active workflows
pub async fn get_active_workflows(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    // One query over every non-terminal state, instead of six queries over a
    // hard-coded name list. That list ("DRAFT", "SUBMIT_SATKER",
    // "PENYUSUNAN_PRIORITAS", …) matched NOTHING: the real state names live in
    // `ms_workflow_status` and are Indonesian ("Draft", "Diajukan ke Validator
    // Wilayah", …), and one of the six describes a step the workflow does not
    // have. Asking the database which states are non-terminal removes the
    // hand-maintained list that could drift from the seed.
    let all_workflows = monitor
        .list_active_workflows(100, 0)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get workflows: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(all_workflows))))
}

/// GET /api/v1/workflow/monitoring/sla-breaches
///
/// Get list of workflows that have breached SLA
pub async fn get_sla_breaches(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    let metrics = monitor
        .get_metrics()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get metrics: {}", e)))?;

    // For now, return empty list as we need more complex query to get actual breached workflows
    // This would require joining with workflow definitions to get SLA limits per state
    let breaches: Vec<WorkflowSummary> = Vec::new();

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            breaches,
            format!("Total SLA breaches: {}", metrics.sla_breaches),
        )),
    ))
}

/// GET /api/v1/workflow/monitoring/bottlenecks
///
/// Get list of workflow bottlenecks
pub async fn get_bottlenecks(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    let bottlenecks = monitor
        .detect_bottlenecks()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to detect bottlenecks: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(bottlenecks))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success(vec!["test"]);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("true"));
    }
}
