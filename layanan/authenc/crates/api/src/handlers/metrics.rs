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
pub async fn prometheus_metrics(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    use metrics_exporter_prometheus::PrometheusBuilder;
    use std::sync::OnceLock;

    static PROMETHEUS_HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> = OnceLock::new();

    let handle = PROMETHEUS_HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder")
    });

    handle.render()
}

/// JSON-format metrics endpoint
pub async fn json_metrics(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
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
