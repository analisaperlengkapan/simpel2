//! Metrics handlers
//!
//! Provides Prometheus-compatible metrics endpoints.

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Metrics summary response
#[derive(Debug, Serialize)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub active_sessions: u64,
    pub auth_success_count: u64,
    pub auth_failure_count: u64,
    pub mfa_verifications: u64,
    pub uptime_seconds: u64,
}

// ===== Routes =====

/// Create metrics routes
pub fn create_metrics_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/metrics/json", get(json_metrics))
}

// ===== Handlers =====

/// Prometheus-format metrics endpoint
async fn prometheus_metrics(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Collect and format Prometheus metrics
    "# HELP authenc_requests_total Total requests\n\
     # TYPE authenc_requests_total counter\n\
     authenc_requests_total 0\n\
     # HELP authenc_active_sessions Active sessions\n\
     # TYPE authenc_active_sessions gauge\n\
     authenc_active_sessions 0\n"
}

/// JSON-format metrics endpoint
async fn json_metrics(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Collect real metrics
    Json(MetricsSummary {
        total_requests: 0,
        active_sessions: 0,
        auth_success_count: 0,
        auth_failure_count: 0,
        mfa_verifications: 0,
        uptime_seconds: 0,
    })
}
