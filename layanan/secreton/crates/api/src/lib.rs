//! Secreton API Library
//!
//! Simple HTTP API for the Secreton transit engine

use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

pub mod audit;
pub mod auth;
pub mod config;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod helpers;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod pki;
pub mod prelude;
pub mod response;
// TODO: Re-enable after OpenRaft migration is complete
// pub mod raft;
pub mod services;
pub mod tls_optimization;
pub mod transit;

// Re-export gRPC from separate crate
pub use secreton_grpc as grpc;

use axum::extract::State;
pub use error::{ApiError, ApiResult};
pub use handlers::{AppState, ListQuery};
pub use middleware::RequestContext;
pub use models::{PaginatedResponse, PaginationQuery};
pub use pki::{PkiApiState, create_pki_router};
pub use response::{
    ApiResponse, DependencyStatus, ErrorDetails, HealthCheckDependencies, HealthCheckResponse,
    ResponseMetadata,
};
// TODO: Re-enable after OpenRaft migration
// pub use raft::{create_raft_router, RaftApiState};
pub use transit::{TransitApiState, create_transit_router};
#[derive(Clone)]
pub struct ApiState {
    pub transit: TransitApiState,
    pub pki: PkiApiState,
    pub services: std::sync::Arc<crate::services::ServiceContainer>,
    pub prometheus_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
    pub metrics: std::sync::Arc<crate::metrics::GlobalMetrics>,
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
    // Legacy v1 routers (transit, pki).
    //
    // `/v1/kv` used to live here. It was a `HashMap` behind an `RwLock`: every
    // secret written to it was gone on the next restart, and none of it was
    // encrypted at rest. It was removed in #130 — the persistent, encrypted,
    // policy-scoped path is `/v1/secret/data/{path}` below, which goes through
    // `ServiceContainer::secret_storage`. `tests/no_inmemory_kv.rs` fails if
    // anything mounts an in-memory secret store here again.
    let v1_legacy = Router::new()
        .nest("/transit", create_transit_router(state.transit.clone()))
        .nest("/pki", create_pki_router(state.pki.clone()));

    // New v1 router built from handlers (includes /sys, /auth, /secrets, /dynamic, etc.)
    let config = config::ApiConfig::load().unwrap_or_default();
    let protected_routes = handlers::create_protected_router(&config, state.services.clone());
    let unprotected_routes = handlers::create_unprotected_router(&config, state.services.clone());

    let v1_handlers = protected_routes.merge(unprotected_routes);

    // Combine legacy and new handlers into a single Router.
    //
    // Layer order is load-bearing: tower wraps the current service on each
    // `.layer()`, so the LAST one added is the OUTERMOST. Reading the list
    // bottom-up gives the execution order:
    //
    //     metrics -> auth -> seal -> policy -> handler
    //
    // `policy_check_middleware` is added first precisely so it runs last, on a
    // request that is already authenticated (it reads the RequestContext that
    // `auth_middleware` inserts) and already known to be against an unsealed
    // engine. Adding it after `auth_middleware` in this list would run it
    // BEFORE auth, where the context does not exist yet and every request would
    // be refused. It was written long ago and never mounted at all (#129);
    // `tests/policy_enforcement.rs` fails if it is removed again.
    let v1_router = v1_legacy
        .merge(v1_handlers)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::policy_check_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::seal_check_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::metrics_middleware,
        ));

    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/live", get(health_check))
        .route("/version", get(get_version))
        .route("/metrics", get(get_metrics))
        .route("/metrics/prometheus", get(get_prometheus_metrics))
        .route("/metrics/tls", get(get_tls_metrics))
        .nest_service("/v1", v1_router)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::request_rate_middleware,
        ))
        // Common infrastructure layers
        .layer(axum::middleware::from_fn(
            lib_backend::middleware::security::security_headers_middleware,
        ))
        .layer(lib_backend::middleware::logging::RequestLogger)
        .layer(axum::middleware::from_fn(
            lib_backend::correlation::correlation_id_middleware,
        ))
        .layer(lib_backend::middleware::cors::standard_cors(vec![
            "*".to_string(),
        ]))
        .with_state(state)
}

pub async fn get_prometheus_metrics(State(state): State<ApiState>) -> String {
    if let Some(handle) = &state.prometheus_handle {
        handle.render()
    } else {
        "# Prometheus metrics not initialized".to_string()
    }
}

pub async fn get_metrics(State(state): State<ApiState>) -> Json<crate::metrics::MetricsResponse> {
    Json(state.metrics.snapshot())
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    })
}

/// Readiness check endpoint - returns ready when the service can accept traffic
pub async fn ready_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ready".to_string(),
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
    let metrics = crate::tls_optimization::get_tls_metrics();
    Json(TlsMetricsResponse {
        total_handshakes: metrics.total_handshakes,
        successful_handshakes: metrics.successful_handshakes,
        session_resumptions: metrics.session_resumptions,
        handshake_failures: metrics.handshake_failures,
        average_handshake_time_ms: metrics.average_handshake_time_ms,
        success_rate_percent: metrics.get_success_rate(),
        resumption_rate_percent: metrics.get_resumption_rate(),
    })
}
