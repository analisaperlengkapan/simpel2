//! # authenc-api
//!
//! Public REST API for Authenc identity provider.
//!
//! This crate provides public HTTP endpoints for:
//! - Authentication (login, logout, token refresh)
//! - Token validation
//! - OAuth2/OIDC public flows
//! - User profile access
//! - Session management
//! - WebAuthn/Passkeys (PRIMARY authentication method)
//! - Client management (OAuth2 clients, DCR)
//!
//! ## Architecture
//!
//! The API is organized into several modules:
//! - `state` - API state with service dependencies
//! - `router` - Unified router with middleware
//! - `routes` - Route definitions
//! - `app` - Application setup and lifecycle
//! - `handlers` - Request handlers
//! - `middleware` - Middleware components
//!
//! ## Usage
//!
//! ```text
//! // Example API Server setup
//! use authenc_api::{ApiState, AxumApp, AppConfig};
//! use std::net::SocketAddr;
//! ```

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod app;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod session_store;
pub mod state;

// Re-export commonly used types
pub use app::{AppConfig, AxumApp};

pub use handlers::{
    ErrorResponse, authorize_handler, captcha_challenge_handler, captcha_image_handler,
    captcha_verify_handler, create_client_handler, delete_client_configuration_handler,
    delete_client_handler, delete_credential_handler, discovery_handler,
    finish_authentication_handler, finish_registration_handler, get_client_configuration_handler,
    get_client_handler, get_current_user_handler, introspect_handler, list_clients_handler,
    list_credentials_handler, login_handler, logout_handler, refresh_token_handler,
    register_client_handler, start_authentication_handler, start_registration_handler,
    token_handler, update_client_configuration_handler, update_client_handler,
    update_credential_handler, userinfo_handler, validate_token_handler,
};

pub use middleware::{
    AUTH_USER_KEY, AdaptiveRateLimiter, AuthState, AuthUser, AuthUserExt, CorsConfig, CsrfConfig,
    CsrfState, Environment, RateLimitConfig, RateLimiterState, RequireAuth, apply_auth_layer,
    apply_csrf_layer, auth_middleware, csrf_protection_middleware, development_cors,
    generate_csrf_token_response, production_cors, rate_limit_middleware,
};

pub use router::{create_authenticated_router, create_development_router, create_unified_router};

pub use routes::{create_router, create_router_with_middleware};

pub use state::ApiState;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
