//! API route definitions

use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{handlers, middleware, state::ApiState};

/// Create the public REST API router
///
/// This router includes all public endpoints for:
/// - Authentication (login, logout, token refresh)
/// - WebAuthn/Passkeys (registration, authentication, credential management)
/// - OAuth2/OIDC (authorization, token, discovery, userinfo)
/// - Token validation
pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        // Authentication endpoints
        .route("/api/v1/auth/login", post(handlers::login_handler))
        .route("/api/v1/auth/logout", post(handlers::logout_handler))
        .route(
            "/api/v1/auth/refresh",
            post(handlers::refresh_token_handler),
        )
        .route("/api/v1/auth/me", get(handlers::get_current_user_handler))
        .route(
            "/api/v1/auth/validate",
            post(handlers::validate_token_handler),
        )
        // WebAuthn/Passkeys endpoints (MANDATORY - PRIMARY authentication)
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
            "/api/v1/auth/webauthn/credentials/:id",
            delete(handlers::delete_credential_handler)
                .patch(handlers::update_credential_handler),
        )
        // OAuth2/OIDC endpoints
        .route("/api/v1/oauth2/authorize", get(handlers::authorize_handler))
        .route("/api/v1/oauth2/token", post(handlers::token_handler))
        .route(
            "/api/v1/oauth2/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        .route("/api/v1/oauth2/userinfo", get(handlers::userinfo_handler))
        // Optional: Token introspection (RFC 7662)
        .route(
            "/api/v1/auth/introspect",
            post(handlers::introspect_handler),
        )
        // Health check endpoint
        .route("/health", get(health_check))
        // Apply state
        .with_state(state)
}

/// Create the router with middleware
///
/// Applies CORS and rate limiting middleware to the router.
pub fn create_router_with_middleware(
    state: Arc<ApiState>,
    cors_config: middleware::CorsConfig,
    rate_limit_config: middleware::RateLimitConfig,
) -> Router {
    let rate_limiter = middleware::RateLimiter::new(rate_limit_config);

    create_router(state)
        // Apply CORS middleware
        .layer(cors_config.build())
        // Apply rate limiting middleware
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limit_middleware,
        ))
        // Apply tracing middleware
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        // This is a smoke test to ensure router compiles
        // Actual testing requires mock services
        // TODO: Add proper integration tests with mock services
    }
}
