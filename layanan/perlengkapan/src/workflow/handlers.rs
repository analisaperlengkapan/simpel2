//! Workflow Monitoring API Handlers
//!
//! REST API endpoints for workflow monitoring dashboard

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::workflow::monitoring::{WorkflowMonitor, WorkflowSummary};

// ═══════════════════════════════════════════════════════════════════════════
// Response Types
// ═══════════════════════════════════════════════════════════════════════════

// The envelope is the shared one, not a local copy. A second declaration here
// is how `penghapusan_bmn` came to emit a paginated envelope without
// `total_pages`: its local twin looked authoritative, drifted from the shared
// shape, and serde rejected every response while the API answered 200.
pub use lib_perlengkapan::response::ApiResponse;

// ═══════════════════════════════════════════════════════════════════════════
// Handler Functions
// ═══════════════════════════════════════════════════════════════════════════

/// GET /api/v1/workflow/monitoring/metrics
///
/// Get overall workflow metrics for monitoring dashboard
pub async fn get_workflow_metrics(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    require_monitoring(&claims)?;
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    let metrics = monitor
        .get_metrics()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get metrics: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(metrics, "Success")),
    ))
}

/// Operational monitoring is cross-module and cross-satker (queues, bottlenecks,
/// per-satker dwell times) and is not confined to any satker; it is the
/// administrators' console (`/admin/workflow-monitoring`). It answered any
/// authenticated caller before.
fn require_monitoring(claims: &Claims) -> Result<(), AppError> {
    claims.require_capability(lib_core::authz::Capability::ViewAudit)
}

/// GET /api/v1/workflow/monitoring/active
///
/// Get list of active workflows
pub async fn get_active_workflows(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    require_monitoring(&claims)?;
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

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(all_workflows, "Success")),
    ))
}

/// GET /api/v1/workflow/monitoring/sla-breaches
///
/// Get list of workflows that have breached SLA
pub async fn get_sla_breaches(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    require_monitoring(&claims)?;
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
        Json(ApiResponse::success(
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
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    require_monitoring(&claims)?;
    let monitor = WorkflowMonitor::new(state.db_pool.clone());

    let bottlenecks = monitor
        .detect_bottlenecks()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to detect bottlenecks: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(bottlenecks, "Success")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success(vec!["test"], "Success");
        let json = serde_json::to_value(&response).unwrap();

        // Assert the KEY SET, not the presence of a substring. The previous
        // version checked only that the output contained "success" and "true",
        // which passes just as well if `data` and `message` are dropped — so it
        // could not have caught the drift that motivated the duplicate-envelope
        // guard, where exactly those keys went missing from a local copy.
        let keys: std::collections::BTreeSet<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            ["data", "message", "success"].into_iter().collect(),
            "the wire envelope must carry exactly success/data/message"
        );
        assert_eq!(json["success"], serde_json::json!(true));
        assert_eq!(json["data"], serde_json::json!(["test"]));
        assert_eq!(json["message"], serde_json::json!("Success"));
    }
}
