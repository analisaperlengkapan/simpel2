//! Middleware modules for Axum backend services

#[cfg(feature = "axum")]
pub mod cors;
#[cfg(feature = "axum")]
pub mod logging;
#[cfg(feature = "axum")]
pub mod security;
#[cfg(feature = "axum")]
pub mod timeout;

#[cfg(feature = "axum")]
pub use security::{
    RateLimiter, csrf_validation_middleware, input_validation_middleware, rate_limit_middleware,
    security_headers_middleware, security_middleware_stack,
};
