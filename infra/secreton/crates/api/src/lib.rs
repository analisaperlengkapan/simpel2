//! Secreton API Library
//!
//! Simple HTTP API for the Secreton transit engine

use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

pub mod audit;
pub mod auth;
pub mod config;
pub mod error;
pub mod grpc;
pub mod handlers;
pub mod kv;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod prelude;
pub mod response;
// TODO: Re-enable after OpenRaft migration is complete
// pub mod raft;
pub mod services;
pub mod transit;

pub use error::{ApiError, ApiResult};
pub use kv::{create_kv_router, KVApiState, KVEngine};
pub use models::PaginatedResponse;
pub use response::{
    ApiResponse, DependencyStatus, ErrorDetails, HealthCheckDependencies, HealthCheckResponse,
    ResponseMetadata,
};
// TODO: Re-enable after OpenRaft migration
// pub use raft::{create_raft_router, RaftApiState};
pub use transit::{create_transit_router, TransitApiState};
#[derive(Clone)]
pub struct ApiState {
    pub transit: TransitApiState,
    pub kv: KVApiState,
    pub services: std::sync::Arc<crate::services::ServiceContainer>,
}

#[derive(Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8200,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
    pub build_date: String,
    pub git_commit: String,
}

#[derive(Serialize, Deserialize)]
pub struct TlsMetricsResponse {
    pub total_handshakes: u64,
    pub successful_handshakes: u64,
    pub session_resumptions: u64,
    pub handshake_failures: u64,
    pub average_handshake_time_ms: u64,
    pub success_rate_percent: f64,
    pub resumption_rate_percent: f64,
}

/// Create the main API router combining all endpoints
pub fn create_api_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/version", get(get_version))
        .route("/metrics", get(get_metrics))
        .route("/metrics/tls", get(get_tls_metrics))
        .nest("/v1/transit", create_transit_router(state.transit))
        .nest("/v1/secret", create_kv_router(state.kv))
}

#[derive(Serialize, Deserialize)]
pub struct MetricsResponse {
    pub rest_requests_total: u64,
    pub grpc_requests_total: u64,
    pub transit_operations_total: u64,
    pub kv_operations_total: u64,
    pub active_connections: u64,
    pub uptime_seconds: u64,
}

pub async fn get_metrics() -> Json<MetricsResponse> {
    // TODO: Implement actual metrics collection
    Json(MetricsResponse {
        rest_requests_total: 0,
        grpc_requests_total: 0,
        transit_operations_total: 0,
        kv_operations_total: 0,
        active_connections: 0,
        uptime_seconds: 0,
    })
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    })
}

pub async fn get_version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: "2024".to_string(),
        git_commit: "unknown".to_string(),
    })
}

pub async fn get_tls_metrics() -> Json<TlsMetricsResponse> {
    // Placeholder metrics - implement when TLS monitoring is needed
    Json(TlsMetricsResponse {
        total_handshakes: 0,
        successful_handshakes: 0,
        session_resumptions: 0,
        handshake_failures: 0,
        average_handshake_time_ms: 0,
        success_rate_percent: 0.0,
        resumption_rate_percent: 0.0,
    })
}
