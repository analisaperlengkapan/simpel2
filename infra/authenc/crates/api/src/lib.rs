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

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod handlers;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod session_store;
pub mod state;

// Re-export commonly used types
pub use handlers::{
    authorize_handler, delete_credential_handler, discovery_handler,
    finish_authentication_handler, finish_registration_handler, get_current_user_handler,
    introspect_handler, list_credentials_handler, login_handler, logout_handler,
    refresh_token_handler, start_authentication_handler, start_registration_handler,
    token_handler, update_credential_handler, userinfo_handler, validate_token_handler,
    ErrorResponse,
};

pub use middleware::{
    auth_layer, auth_middleware, csrf_layer, csrf_protection_middleware, development_cors,
    generate_csrf_token_response, production_cors, rate_limit_middleware, AdaptiveRateLimiter,
    AuthState, AuthUser, AuthUserExt, CorsConfig, CsrfConfig, CsrfState, Environment,
    RateLimitConfig, RateLimitError, RateLimiter, RequireAuth, AUTH_USER_KEY,
};

pub use router::{
    create_phase1_router, create_phase2_router, create_phase3_router, create_unified_router,
};

pub use routes::{create_router, create_router_with_middleware};

pub use state::ApiState;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
