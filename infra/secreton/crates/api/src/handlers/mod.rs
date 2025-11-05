//! HTTP request handlers for the Secreton API.
//!
//! Provides comprehensive REST endpoints for secret management,
//! authentication, authorization, and administrative functions.

pub mod admin;
pub mod auth;
pub mod dynamic;
pub mod health;
pub mod lease;
pub mod namespace;
pub mod policy;
pub mod seal;
pub mod secret;
pub mod wrapping;

#[cfg(feature = "raft-consensus")]
pub mod raft;

use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};

use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

use crate::{ApiResponse, ApiResult, config::ApiConfig, services::ServiceContainer};

/// Application state shared across handlers
pub type AppState = Arc<ServiceContainer>;

// Re-export common types
pub use secret::ListQuery;

/// Create the main application router
pub fn create_router(config: &ApiConfig, services: Arc<ServiceContainer>) -> Router {
    let app_state = services.clone();

    // Build system routes
    let sys_routes = seal::create_routes()
        .merge(namespace::create_routes())
        .merge(lease::create_routes())
        .merge(policy::create_routes())
        .merge(wrapping::create_routes());

    // Add raft routes if feature is enabled
    #[cfg(feature = "raft-consensus")]
    {
        sys_routes = sys_routes.merge(raft::create_routes());
    }

    // Create API v1 routes
    let api_v1 = Router::new()
        .nest("/auth", auth::create_routes())
        .nest("/secrets", secret::create_routes())
        .nest("/admin", admin::create_routes())
        .nest("/sys", sys_routes)
        .nest("/dynamic", dynamic::create_routes())
        .route("/health", get(health::health_check))
        .route("/version", get(get_version))
        .route("/metrics", get(get_metrics));

    // Main router with middleware stack
    Router::new()
        .nest("/api/v1", api_v1)
        .route("/", get(root_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(config.http.timeout))
                .layer(CorsLayer::permissive()), // TODO: Configure properly
                                                 // Note: auth_middleware and rate_limit middleware should be applied
                                                 // using axum::middleware::from_fn_with_state if needed
        )
        .with_state(app_state)
}

/// Root endpoint handler
async fn root_handler() -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let data = serde_json::json!({
        "service": "Secreton API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Advanced Security Vault System",
        "documentation": "/api/v1/docs"
    });

    Ok(Json(ApiResponse::success(data)))
}

/// Get API version information
async fn get_version() -> ApiResult<Json<ApiResponse<VersionInfo>>> {
    let version_info = VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: env!("BUILD_DATE").to_string(),
        git_commit: env!("GIT_COMMIT").to_string(),
        rust_version: env!("RUST_VERSION").to_string(),
    };

    Ok(Json(ApiResponse::success(version_info)))
}

/// Get Prometheus metrics
async fn get_metrics(State(_state): State<AppState>) -> Result<String, StatusCode> {
    // TODO: Implement metrics collection
    Ok("# Secreton API Metrics\n".to_string())
}

/// Version information
#[derive(serde::Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub build_date: String,
    pub git_commit: String,
    pub rust_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_root_endpoint() {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_router(&config, services);
        let server = TestServer::new(app);

        let response = server.get("/").await;
        response.assert_status_ok();

        let body: ApiResponse<serde_json::Value> = response.json();
        assert!(body.success);
        assert!(body.data.is_some());
    }

    #[tokio::test]
    async fn test_version_endpoint() {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_router(&config, services);
        let server = TestServer::new(app);

        let response = server.get("/api/v1/version").await;
        response.assert_status_ok();

        let body: ApiResponse<VersionInfo> = response.json();
        assert!(body.success);
        assert!(body.data.is_some());
    }
}
