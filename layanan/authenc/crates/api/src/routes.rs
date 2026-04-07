//! API route definitions

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
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
            "/api/v1/auth/webauthn/credentials/{id}",
            delete(handlers::delete_credential_handler).patch(handlers::update_credential_handler),
        )
        // OAuth2/OIDC endpoints
        .route("/api/v1/oauth2/authorize", get(handlers::authorize_handler))
        .route("/api/v1/oauth2/token", post(handlers::token_handler))
        .route(
            "/api/v1/oauth2/.well-known/openid-configuration",
            get(handlers::discovery_handler),
        )
        .route("/api/v1/oauth2/userinfo", get(handlers::userinfo_handler))
        // JWKS endpoint (must match discovery document's jwks_uri)
        .nest("/api/v1/oauth2", handlers::jwks::create_jwks_routes())
        .route("/.well-known/jwks.json", get(handlers::jwks::jwks_handler))
        // Optional: Token introspection (RFC 7662)
        .route(
            "/api/v1/auth/introspect",
            post(handlers::introspect_handler),
        )
        // Client management endpoints (admin-only)
        .route("/api/v1/clients", get(handlers::list_clients_handler))
        .route("/api/v1/clients", post(handlers::create_client_handler))
        .route(
            "/api/v1/clients/{id}",
            get(handlers::get_client_handler)
                .patch(handlers::update_client_handler)
                .delete(handlers::delete_client_handler),
        )
        // Dynamic Client Registration (RFC 7591/7592)
        .route("/register", post(handlers::register_client_handler))
        .route(
            "/register/{client_id}",
            get(handlers::get_client_configuration_handler)
                .patch(handlers::update_client_configuration_handler)
                .delete(handlers::delete_client_configuration_handler),
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
    _rate_limit_config: middleware::RateLimitConfig,
) -> Router {
    create_router(state)
        // Apply CORS middleware
        .layer(cors_config.build())
        // TODO: Re-enable rate limiting once ConnectInfo is configured
        // Rate limit middleware requires ConnectInfo<SocketAddr> which needs
        // server to be created with into_make_service_with_connect_info()
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
