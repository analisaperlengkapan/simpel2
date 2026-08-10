//! HTTP request handlers for the Secreton API.
//!
//! This module provides comprehensive REST endpoints for all Secreton engine operations.
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
//! - [`seal`] - Engine seal/unseal operations (Shamir secret sharing)
//! - [`health`] - Health checks and readiness probes
//!
//! ## Optional Features
//! - `raft` - Raft consensus endpoints (high availability clustering). Compiled
//!   only under the `raft-consensus` feature, so this is deliberately not an
//!   intra-doc link: it would not resolve in a default build.
//!
//! # Request Flow
//!
//! What the router in [`crate::create_api_router`] actually layers over `/v1`,
//! outermost first:
//!
//! 1. **Rate limiting** — [`crate::middleware::request_rate_middleware`]
//! 2. **Metrics** — [`crate::middleware::metrics_middleware`]
//! 3. **Authentication** — [`crate::middleware::auth_middleware`]: validates the
//!    bearer token and puts a [`crate::middleware::RequestContext`] (roles,
//!    permissions, JWT claims) into the request extensions
//! 4. **Seal check** — [`crate::middleware::seal_check_middleware`]: 503 while sealed
//! 5. **Handler execution**, then a standardized [`ApiResponse`]
//!
//! ## What is NOT in that stack
//!
//! This list used to claim an `rbac_middleware` and an `audit_middleware`.
//! **Neither function exists anywhere in the repo** — they were cited as the
//! enforcement mechanism while nothing enforced anything.
//!
//! - **No authorization layer.** Authentication is global; authorization is
//!   per-handler and currently thin. [`namespace`] checks
//!   `access_control.check_access`; [`policy`] checks the caller's admin role.
//!   The secret read/write path checks **nothing** beyond having a valid token —
//!   `SecretService::get_secret` takes a `user_id` and uses it only as the audit
//!   actor. Do not assume a handler is guarded because it is behind `/v1`.
//! - **No audit layer.** Nothing wraps the router to log requests. Audit records
//!   are written explicitly by the services that choose to, so a new handler
//!   that forgets to call the audit logger simply produces no trail.
//!
//! Anything added here must be layered in [`crate::create_api_router`] to take
//! effect. A doc entry is not a control.
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
pub mod classification;
pub mod crypto;
pub mod dynamic;
pub mod health;
pub mod inject;
pub mod key_hierarchy;
pub mod lease;
pub mod metrics;
pub mod namespace;
pub mod pki;
pub mod policy;
pub mod revocation;
pub mod rotation;
pub mod seal;
pub mod secret;
pub mod ssh;
pub mod totp;
pub mod transform;
pub mod webhook;
pub mod wrapping;
pub mod zero_knowledge;

#[cfg(feature = "raft-consensus")]
pub mod raft;

use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};

use std::sync::Arc;

use crate::{ApiResponse, ApiResult, config::ApiConfig, services::ServiceContainer};

/// Application state shared across handlers
pub type AppState = Arc<ServiceContainer>;

// Re-export common types
pub use secret::ListQuery;

/// Create the complete API router
pub fn create_router(config: &ApiConfig, services: Arc<ServiceContainer>) -> Router {
    create_protected_router(config, services.clone())
        .merge(create_unprotected_router(config, services))
}

/// Create the main application router for protected routes
pub fn create_protected_router(_config: &ApiConfig, services: Arc<ServiceContainer>) -> Router {
    let app_state = services.clone();

    // These are all the routes that should be protected by auth and seal checks
    let protected_sys_routes = namespace::create_routes()
        .merge(lease::create_routes())
        .merge(policy::create_routes())
        .merge(wrapping::create_routes())
        .merge(key_hierarchy::create_routes())
        .nest("/totp", totp::create_routes())
        .nest("/crypto", crypto::create_routes())
        .nest("/transform", transform::create_routes())
        .nest("/pki", pki::create_routes())
        .nest("/ssh", ssh::create_routes())
        .nest("/aws", aws::create_routes())
        .nest("/rotation", rotation::create_routes())
        .nest("/zk", zero_knowledge::create_routes())
        .nest("/inject", inject::create_routes())
        .nest("/webhooks", webhook::create_routes());

    Router::new()
        .route("/", get(root_handler))
        .nest("/auth", auth::create_routes())
        .nest("/secret", secret::create_routes())
        .route("/secrets", get(secret::list_secrets))
        .nest("/admin", admin::create_routes())
        .nest("/sys", protected_sys_routes)
        .nest("/dynamic", dynamic::create_routes())
        .with_state(app_state)
}

/// Create a router for unprotected system routes
pub fn create_unprotected_router(_config: &ApiConfig, services: Arc<ServiceContainer>) -> Router {
    let app_state = services.clone();

    // Routes that must be available even when the engine is sealed
    Router::new()
        .nest("/sys", seal::create_routes())
        .route("/health", get(health::health_check))
        .route("/version", get(get_version))
        .route("/metrics", get(get_metrics))
        .with_state(app_state)
}

/// Root endpoint handler (Used for root path "/" if needed)
#[allow(dead_code)]
async fn root_handler() -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let data = serde_json::json!({
        "service": "Secreton API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Advanced Security Engine System",
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
    // Use the metrics handler from the metrics module
    let response = metrics::metrics_handler().await;

    // Extract the body from the response
    match response.status() {
        StatusCode::OK => {
            // Extract body as string
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            String::from_utf8(body.to_vec()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Version information
#[derive(serde::Serialize, serde::Deserialize)]
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

        let response = server.get("/version").await;
        response.assert_status_ok();

        let body: ApiResponse<VersionInfo> = response.json();
        assert!(body.success);
        assert!(body.data.is_some());
    }
}
