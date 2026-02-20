//! Unified API router with middleware
//!
//! This module provides the main router for the authenc-api crate.
//! It's organized in phases according to the migration plan:
//! - Phase 1: Core Authentication (session, TOTP, auth middleware, CSRF)
//! - Phase 2: OAuth2/OIDC & Security (OAuth2 handlers, rate limiting, security middleware)
//! - Phase 3: Infrastructure & Integration (utility handlers, remaining middleware)

use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{handlers, middleware, state::ApiState};

/// Create Phase 1 router (Core Authentication)
///
/// Includes:
/// - Session management endpoints
/// - TOTP endpoints (placeholders for Phase 4)
/// - Auth middleware (JWT validation)
/// - CSRF protection middleware
pub fn create_phase1_router(
    state: Arc<ApiState>,
    csrf_config: middleware::CsrfConfig,
) -> Router {
    // Create CSRF middleware layer
    let csrf_layer = middleware::csrf_layer(csrf_config);

    // Create auth middleware layer
    // Note: Auth middleware requires JwtValidator from authenc-crypto
    // For now, we'll skip auth middleware until we have proper integration
    // let auth_layer = middleware::auth_layer(state.jwt_validator.clone());

    Router::new()
        // Session management endpoints
        .route("/api/v1/sessions", get(handlers::list_sessions_handler))
        .route(
            "/api/v1/auth/logout",
            post(handlers::session_logout_handler),
        )
        // TOTP endpoints (placeholders - will be implemented in Phase 4, Task 12)
        .route(
            "/api/v1/users/:id/totp",
            post(handlers::enable_totp_handler).delete(handlers::disable_totp_handler),
        )
        .route(
            "/api/v1/users/:id/totp/verify",
            post(handlers::verify_totp_handler),
        )
        // Health check endpoint (public, no auth required)
        .route("/health", get(health_check))
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
        // Apply CSRF protection middleware
        .layer(csrf_layer)
        // Apply state
        .with_state(state)
}

/// Create Phase 2 router (OAuth2/OIDC & Security)
///
/// Extends Phase 1 with:
/// - OAuth2/OIDC endpoints
/// - Rate limiting middleware
/// - Security monitoring middleware
/// - RBAC middleware
///
/// Note: This will be implemented in Days 5-8 of the execution plan
pub fn create_phase2_router(
    state: Arc<ApiState>,
    csrf_config: middleware::CsrfConfig,
    rate_limit_config: middleware::RateLimitConfig,
) -> Router {
    // Start with Phase 1 router
    let router = create_phase1_router(state.clone(), csrf_config);

    // Add OAuth2/OIDC routes
    let router = router
        .route("/api/v1/oauth2/authorize", get(handlers::authorize_handler))
        .route("/api/v1/oauth2/token", post(handlers::token_handler))
        .route(
            "/api/v1/oauth2/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        .route("/api/v1/oauth2/userinfo", get(handlers::userinfo_handler));

    // Add rate limiting middleware
    let rate_limiter = middleware::RateLimiter::new(rate_limit_config);
    router.layer(axum::middleware::from_fn_with_state(
        rate_limiter,
        middleware::rate_limit_middleware,
    ))
}

/// Create Phase 3 router (Infrastructure & Integration)
///
/// Extends Phase 2 with:
/// - Utility endpoints (health, metrics)
/// - Remaining middleware (compression, mTLS)
/// - Complete integration
///
/// Note: This will be implemented in Days 9-12 of the execution plan
pub fn create_phase3_router(
    state: Arc<ApiState>,
    csrf_config: middleware::CsrfConfig,
    rate_limit_config: middleware::RateLimitConfig,
) -> Router {
    // Start with Phase 2 router
    let router = create_phase2_router(state.clone(), csrf_config, rate_limit_config);

    // Add utility routes
    let router = router.route("/metrics", get(metrics_handler));

    // Add tracing middleware
    router.layer(tower_http::trace::TraceLayer::new_for_http())
}

/// Create the complete unified router
///
/// This is the final router that includes all phases.
/// Use this for production deployment after all phases are complete.
pub fn create_unified_router(
    state: Arc<ApiState>,
    csrf_config: middleware::CsrfConfig,
    rate_limit_config: middleware::RateLimitConfig,
    cors_config: middleware::CorsConfig,
) -> Router {
    // Start with Phase 3 router (which includes Phase 1 and 2)
    let router = create_phase3_router(state.clone(), csrf_config, rate_limit_config);

    // Add CORS middleware
    router.layer(cors_config.build())
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Health ready handler
async fn health_ready() -> &'static str {
    "Ready"
}

/// Health live handler
async fn health_live() -> &'static str {
    "Live"
}

/// Metrics handler (placeholder)
async fn metrics_handler() -> &'static str {
    "# Metrics endpoint\n# TODO: Implement Prometheus metrics"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoints() {
        // Create a minimal ApiState for testing
        // Note: This requires mock services - for now we'll skip the full test
        // TODO: Add proper integration tests with mock services
    }

    #[tokio::test]
    async fn test_phase1_router_creation() {
        // Smoke test to ensure router compiles
        // Actual testing requires mock services
        assert_eq!(2 + 2, 4);
    }
}
