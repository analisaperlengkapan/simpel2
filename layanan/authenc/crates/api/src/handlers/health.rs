//! Health check handlers
//!
//! Provides health check endpoints for the Authenc API service.

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::Serialize;
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Detailed health check response
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub database: ComponentHealth,
    pub services: ServiceHealth,
}

/// Component health status
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub latency_ms: Option<u64>,
}

/// Service health summary
#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub auth: String,
    pub jwt: String,
    pub oauth2: String,
}

// ===== Routes =====

/// Create health check routes
pub fn create_health_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        .route("/health/detailed", get(detailed_health))
}

// ===== Handlers =====

/// Basic health check
async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
    })
}

/// Readiness probe
async fn readiness_check(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Check database and service connectivity
    Json(serde_json::json!({
        "status": "ready"
    }))
}

/// Liveness probe
async fn liveness_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive"
    }))
}

/// Detailed health check with component status
async fn detailed_health(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Check actual component health
    Json(DetailedHealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: ComponentHealth {
            status: "unknown".to_string(),
            latency_ms: None,
        },
        services: ServiceHealth {
            auth: "unknown".to_string(),
            jwt: "unknown".to_string(),
            oauth2: "unknown".to_string(),
        },
    })
}
