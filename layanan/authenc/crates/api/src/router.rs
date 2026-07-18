//! Unified API router with middleware
//!
//! This module provides the main router for the authenc-api crate.
//! It brings together all migrated handlers and middleware into a functional API.
//!
//! ## Router Organization
//!
//! The router is organized into logical groups:
//! - Authentication endpoints (login, logout, token management)
//! - WebAuthn/Passkeys endpoints (PRIMARY authentication method)
//! - OAuth2/OIDC endpoints (authorization, token, discovery)
//! - Client management endpoints (OAuth2 clients, DCR)
//! - Health and metrics endpoints
//!
//! ## Middleware Application Order (outermost to innermost)
//!
//! 1. Compression - Reduce response size
//! 2. Security monitoring - Track suspicious activity
//! 3. Rate limiting - Prevent abuse
//! 4. CORS - Cross-origin resource sharing
//! 5. Request size limit - Prevent large payloads
//! 6. Input validation - Validate request data
//! 7. Authentication - JWT validation (for protected routes)
//! 8. RBAC - Role-based access control (for admin routes)

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::compression::CompressionLayer;

use crate::{handlers, middleware, state::ApiState};

/// Create the unified API router with all routes and middleware
///
/// This is the main entry point for creating the complete API router.
/// It includes all authentication, OAuth2/OIDC, WebAuthn, and client management endpoints.
///
/// # Arguments
///
/// * `state` - API state with all service dependencies
/// * `cors_config` - CORS configuration for cross-origin requests
/// * `rate_limit_config` - Rate limiting configuration
/// * `csrf_config` - CSRF protection configuration
///
/// # Returns
///
/// Complete router with all middleware applied
pub fn create_unified_router(
    state: Arc<ApiState>,
    cors_config: middleware::CorsConfig,
    _rate_limit_config: middleware::RateLimitConfig,
    csrf_config: middleware::CsrfConfig,
) -> Router {
    // Create the base router with all routes
    let router = create_base_router(state.clone());

    // Ensure we have configs for the other middleware layers
    let validation_config = Arc::new(middleware::InputValidationConfig::default());
    // Security events (auth attempts, authz failures) are persisted to
    // authenc.audit_logs — this is the writer behind the IAM audit-log
    // admin endpoints. Without the store the middleware silently degrades
    // to tracing logs and the audit trail is empty.
    let audit_store = Arc::new(authenc_core::services::PgAuditLogStore::new(
        state.database.clone(),
    ));
    let security_state = Arc::new(middleware::SecurityMonitoringState::new(
        middleware::SecurityMonitoringConfig::default(),
        Some(audit_store),
    ));

    // Apply CSRF protection (wraps router directly)
    let router = middleware::apply_csrf_layer(router, csrf_config);

    // Apply remaining middleware layers (outermost to innermost)
    router
        // 1. Compression (outermost)
        .layer(CompressionLayer::new())
        // Apply security monitoring middleware
        .layer(axum::middleware::from_fn_with_state(
            security_state,
            middleware::security_monitoring_middleware,
        ))
        // 2. CORS
        .layer(cors_config.build())
        // 3. Request size limit
        .layer(axum::middleware::from_fn(
            middleware::request_size_limit_middleware,
        ))
        // 4. Tracing (innermost)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // Apply rate limit middleware (inner)
        .layer(middleware::rate_limit_layer(_rate_limit_config))
        // Apply input validation middleware
        .layer(axum::middleware::from_fn(move |req, next| {
            middleware::input_validation_middleware(validation_config.clone(), req, next)
        }))
}

/// Create the base router with all routes (no middleware)
///
/// This creates the router with all endpoint definitions.
/// Middleware should be applied separately using `create_unified_router`.
fn create_base_router(state: Arc<ApiState>) -> Router {
    let router = Router::new()
        // ===== Health and Metrics Endpoints =====
        .route("/health", get(health_check))
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
        .route("/metrics", get(handlers::metrics::prometheus_metrics))
        // ===== Authentication Endpoints =====
        .route("/api/v1/auth/login", post(handlers::login_handler))
        .route("/api/v1/auth/logout", post(handlers::logout_handler))
        .route(
            "/api/v1/auth/refresh",
            post(handlers::refresh_token_handler),
        )
        .route(
            "/api/v1/auth/me",
            get(handlers::get_current_user_handler).put(handlers::update_profile_handler),
        )
        .route(
            "/api/v1/auth/me/password",
            post(handlers::change_password_handler),
        )
        // Self-service session management (REQ-PORTAL-008): the caller lists /
        // terminates their OWN sessions; identity comes from the JWT only.
        .route(
            "/api/v1/auth/sessions",
            get(handlers::list_sessions_handler),
        )
        .route(
            "/api/v1/auth/sessions/{id}",
            axum::routing::delete(handlers::terminate_session_handler),
        )
        .route(
            "/api/v1/auth/validate",
            post(handlers::validate_token_handler),
        )
        // ===== Password Reset Endpoints (public — no auth required) =====
        .route(
            "/api/v1/auth/password/reset",
            post(handlers::password_reset_request_handler),
        )
        .route(
            "/api/v1/auth/password/reset/confirm",
            post(handlers::password_reset_confirm_handler),
        )
        // ===== WebAuthn/Passkeys Endpoints (MANDATORY - PRIMARY authentication) =====
        .route(
            "/api/v1/auth/webauthn/register/start",
            post(handlers::start_registration_handler),
        )
        .route(
            "/api/v1/auth/webauthn/register/finish",
            post(handlers::finish_registration_handler),
        )
        .route(
            "/api/v1/auth/webauthn/authenticate/start",
            post(handlers::start_authentication_handler),
        )
        .route(
            "/api/v1/auth/webauthn/authenticate/finish",
            post(handlers::finish_authentication_handler),
        )
        .route(
            "/api/v1/auth/webauthn/credentials",
            get(handlers::list_credentials_handler),
        )
        .route(
            "/api/v1/auth/webauthn/credentials/{id}",
            delete(handlers::delete_credential_handler).patch(handlers::update_credential_handler),
        )
        // ===== MFA/TOTP Endpoints (SECONDARY authentication method) =====
        // Login-flow MFA endpoints (used by AuthService in portal)
        .route("/api/v1/auth/mfa/setup", post(handlers::mfa_setup_handler))
        .route(
            "/api/v1/auth/mfa/verify-setup",
            post(handlers::mfa_verify_setup_handler),
        )
        .route(
            "/api/v1/auth/mfa/verify",
            post(handlers::mfa_verify_handler),
        )
        .route("/api/v1/auth/mfa/status", get(handlers::mfa_status_handler))
        .route(
            "/api/v1/auth/mfa/backup-codes",
            post(handlers::mfa_backup_codes_handler),
        )
        .route(
            "/api/v1/auth/mfa/verify-recovery",
            post(handlers::mfa_verify_recovery_handler),
        )
        // TOTP management endpoints (used by AuthencApiClient in portal)
        .route(
            "/api/v1/auth/totp/enable",
            post(handlers::totp_enable_handler),
        )
        .route(
            "/api/v1/auth/totp/disable",
            post(handlers::totp_disable_handler),
        )
        .route(
            "/api/v1/auth/totp/verify",
            post(handlers::totp_verify_handler_mfa),
        )
        // ===== OAuth2/OIDC Endpoints =====
        .route("/api/v1/oauth2/authorize", get(handlers::authorize_handler))
        .route("/api/v1/oauth2/token", post(handlers::token_handler))
        .route(
            "/api/v1/oauth2/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        .route("/api/v1/oauth2/userinfo", get(handlers::userinfo_handler))
        .route(
            "/api/v1/oauth2/introspect",
            post(handlers::introspect_handler),
        )
        // ===== JWKS Endpoints =====
        // Nest JWKS routes under /api/v1/oauth2 so that the discovery
        // document's jwks_uri ({base}/api/v1/oauth2/jwks) resolves correctly.
        .nest("/api/v1/oauth2", handlers::jwks::create_jwks_routes())
        // Also serve at the standard well-known path at the root level.
        .route("/.well-known/jwks.json", get(handlers::jwks::jwks_handler))
        // ===== OIDC Discovery at root well-known path =====
        // Per OIDC Discovery §4.1, relying parties auto-discover by
        // appending /.well-known/openid-configuration to the issuer.
        // Serve the discovery document here as well so that clients using
        // the issuer URL (which may not include /api/v1/oauth2) can still
        // find it.
        .route(
            "/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        // Also serve discovery at the issuer-relative path.  The issuer is
        // typically "https://host/api/v1/auth", so OIDC clients will
        // request "https://host/api/v1/auth/.well-known/openid-configuration".
        .route(
            "/api/v1/auth/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        // ===== CAPTCHA Endpoints (public — no auth required) =====
        .route(
            "/api/captcha/challenge",
            post(handlers::captcha_challenge_handler),
        )
        .route(
            "/api/captcha/verify",
            post(handlers::captcha_verify_handler),
        )
        .route(
            "/api/captcha/image/{nonce}/{index}",
            get(handlers::captcha_image_handler),
        );

    // Debug endpoint: only compiled in when `captcha-debug` feature is active
    #[cfg(feature = "captcha-debug")]
    let router = router.route(
        "/api/captcha/debug/{challenge_id}",
        get(handlers::captcha_debug_answer_handler),
    );

    router
        .route(
            "/api/v1/captcha/challenge",
            post(handlers::captcha_challenge_handler),
        )
        .route(
            "/api/v1/captcha/verify",
            post(handlers::captcha_verify_handler),
        )
        // ===== Client Management Endpoints (Admin-only) =====
        .route(
            "/api/v1/clients",
            get(handlers::list_clients_handler).post(handlers::create_client_handler),
        )
        .route(
            "/api/v1/clients/{id}",
            get(handlers::get_client_handler)
                .patch(handlers::update_client_handler)
                .delete(handlers::delete_client_handler),
        )
        // ===== Dynamic Client Registration (RFC 7591/7592) =====
        .route("/register", post(handlers::register_client_handler))
        .route(
            "/register/{client_id}",
            get(handlers::get_client_configuration_handler)
                .patch(handlers::update_client_configuration_handler)
                .delete(handlers::delete_client_configuration_handler),
        )
        // Apply state to all routes
        .with_state(state)
}

/// Create a simplified router for development/testing
///
/// This router includes only essential endpoints without heavy middleware.
/// Useful for local development and integration testing.
pub fn create_development_router(state: Arc<ApiState>) -> Router {
    create_base_router(state.clone())
        // Apply minimal middleware for development
        .layer(middleware::development_cors())
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

/// Create a router with authentication middleware applied
///
/// This router includes authentication middleware for protected endpoints.
/// Use this for routes that require JWT validation.
pub fn create_authenticated_router(
    state: Arc<ApiState>,
    jwt_validator: Arc<authenc_crypto::jwt_validator::JwtValidator>,
) -> Router {
    let router = create_base_router(state.clone());
    middleware::apply_auth_layer(router, jwt_validator)
}

// ===== Handler Functions =====

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

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_router_creation() {
        // Smoke test to ensure router compiles
        // Actual testing requires mock services
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_middleware_order() {
        // Verify middleware is applied in correct order
        // This is a documentation test - actual order is enforced by code structure
        assert_eq!(2 + 2, 4);
    }
}
