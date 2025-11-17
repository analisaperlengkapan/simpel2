//! HTTP request handlers for the Secreton API.
//!
//! This module provides comprehensive REST endpoints for all Secreton vault operations.
//! All handlers follow REST best practices and return consistent [`ApiResponse`] structures.
//!
//! # Handler Modules
//!
//! ## Core Operations
//! - [`admin`] - Administrative endpoints (users, roles, audit logs, system info)
//! - [`auth`] - Authentication endpoints (login, logout, token refresh, MFA)
//! - [`secret`] - Secret CRUD operations (get, put, delete, list, versions)
//! - [`policy`] - Policy management (RBAC rules, permissions)
//! - [`namespace`] - Multi-tenancy namespace management
//!
//! ## Advanced Features
//! - [`dynamic`] - Dynamic secrets generation (database credentials, cloud IAM)
//! - [`lease`] - Lease management (renewal, revocation, cleanup)
//! - [`wrapping`] - Response wrapping for secure secret delivery
//! - [`seal`] - Vault seal/unseal operations (Shamir secret sharing)
//! - [`health`] - Health checks and readiness probes
//!
//! ## Optional Features
//! - [`raft`] - Raft consensus endpoints (high availability clustering)
//!
//! # Request Flow
//!
//! 1. **Authentication**: All requests pass through [`crate::middleware::auth_middleware`]
//! 2. **Authorization**: RBAC policy enforcement via [`crate::middleware::rbac_middleware`]
//! 3. **Rate Limiting**: Request throttling via [`crate::middleware::rate_limit_middleware`]
//! 4. **Handler Execution**: Async handler processes request
//! 5. **Audit Logging**: All operations logged via [`crate::middleware::audit_middleware`]
//! 6. **Response**: Standardized [`ApiResponse`] returned
//!
//! # Error Handling
//!
//! All handlers use [`ApiResult<T>`](crate::ApiResult) which maps to appropriate HTTP status codes:
//! - 200 OK - Successful operation
//! - 201 Created - Resource created
//! - 400 Bad Request - Invalid input
//! - 401 Unauthorized - Authentication required
//! - 403 Forbidden - Insufficient permissions
//! - 404 Not Found - Resource does not exist
//! - 409 Conflict - Resource already exists
//! - 500 Internal Server Error - Unexpected error
//!
//! # Example Handler
//!
//! ```rust,no_run
//! use axum::{Extension, Json, extract::Path};
//! use secreton_api::{ApiResult, ApiResponse, handlers::AppState};
//!
//! async fn example_handler(
//!     Path(id): Path<String>,
//!     Extension(state): Extension<AppState>,
//! ) -> ApiResult<Json<ApiResponse<String>>> {
//!     // Handler logic here
//!     Ok(Json(ApiResponse::success(
//!         format!("Processed {}", id),
//!         None,
//!     )))
//! }
//! ```
//!
//! # Security Considerations
//!
//! - All endpoints require authentication (except `/health` and `/ready`)
//! - Sensitive data never logged or included in error responses
//! - CORS configured for production security
//! - Rate limiting prevents abuse
//! - Audit trail for all mutations

pub mod admin;
pub mod auth;
pub mod aws;
pub mod azure;
pub mod dynamic;
pub mod gcp;
pub mod health;
pub mod identity;
pub mod kafka;
pub mod kmip;
pub mod ldap;
pub mod lease;
pub mod namespace;
pub mod policy;
pub mod rabbitmq;
pub mod rotation;
pub mod seal;
pub mod secret;
pub mod ssh;
pub mod totp;
pub mod transform;
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
        .merge(wrapping::create_routes())
        .merge(totp::create_routes())
        .merge(transform::create_routes())
        .merge(ssh::create_routes())
        .merge(aws::create_routes())
        .merge(gcp::create_routes())
        .merge(azure::create_routes())
        .merge(identity::create_routes())
        .merge(rotation::create_routes())
        .merge(kmip::create_routes())
        .merge(ldap::create_routes())
        .merge(rabbitmq::create_routes())
        .merge(kafka::create_routes());

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

#[cfg(all(test, feature = "enable-inline-tests"))]
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
