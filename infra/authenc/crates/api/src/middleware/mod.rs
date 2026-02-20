//! Axum middleware

pub mod auth;
pub mod cors;
pub mod csrf;
pub mod rate_limit;

// Re-export middleware
pub use auth::{
    auth_layer, auth_middleware, AuthState, AuthUser, AuthUserExt, RequireAuth, AUTH_USER_KEY,
};
pub use cors::{development_cors, production_cors, CorsConfig, Environment};
pub use csrf::{
    csrf_layer, csrf_protection_middleware, generate_csrf_token_response, CsrfConfig, CsrfState,
};
pub use rate_limit::{
    rate_limit_middleware, AdaptiveRateLimiter, RateLimitConfig, RateLimitError, RateLimiter,
};


// Middleware modules will be added here as implementation progresses
// - auth.rs (JWT validation) ✅ MIGRATED
// - csrf.rs (CSRF protection) ✅ MIGRATED
// - cors.rs (CORS configuration) ✅ EXISTS
// - rate_limit.rs (Rate limiting) ✅ EXISTS
