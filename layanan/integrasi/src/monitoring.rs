//! The HTTP port of layanan-integrasi (8080): one health endpoint, nothing else.
//!
//! # Why the other four were removed (2026-11-12)
//!
//! `/api/monitoring/sync-status`, `/sync-history`, `/alerts` and `/dashboard` answered without
//! authentication, with CORS open to any origin, to every pod NetworkPolicy lets reach this port
//! (backend, frontend, authenc, simpelv1). No page, script or service in the repo called them, and
//! the ingress does not route to them. What they served was `integrasi.api_log`, which only holds
//! calls that got an HTTP answer: a sync that died on DNS, on its token, or before its first
//! request did not appear in them at all, so "is the sync running, and is something wrong" (what
//! an operator actually asks) was the one question they could not answer. They had also built
//! their SQL from the query string until #1091.
//!
//! That question is now answered by `integrasi.sync_status`, which every run writes
//! (`crate::sync_run`), and shown to administrators by perlengkapan behind its own login and roles.
//!
//! # Health
//!
//! `GET /api/monitoring/health` stays, unauthenticated as health endpoints are: it answers whether
//! this server's database handle works (`200 healthy` / `503 unhealthy`) and nothing about the
//! data. Operators use it after a database restart (see `db_handle`).

use crate::db_handle::DbHandle;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use std::sync::Arc;

/// Shared state for the health endpoint
#[derive(Clone)]
pub struct MonitoringState {
    pub db_client: Arc<DbHandle>,
}

/// The router of the HTTP port.
pub fn create_monitoring_router(state: MonitoringState) -> Router {
    Router::new()
        .route("/api/monitoring/health", get(health_check))
        .with_state(state)
}

/// Health check endpoint
async fn health_check(State(state): State<MonitoringState>) -> impl IntoResponse {
    match state.db_client.simple_query("SELECT 1").await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "healthy",
                "database": "connected"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}
