//! Axum middleware for authenc-api
//!
//! This module contains all middleware components for the Authenc API:
//! - Authentication and authorization
//! - Rate limiting (basic and adaptive)
//! - CSRF protection
//! - CORS configuration
//! - Input validation
//! - Request size limiting
//! - Response compression
//! - Security monitoring
//! - RBAC enforcement
//! - mTLS authentication
//! - MFA-specific middleware

// Core middleware modules
pub mod adaptive_rate_limit;
pub mod adaptive_rate_limit_integration;
pub mod auth;
pub mod compression;
pub mod cors;
pub mod csrf;
pub mod mfa_performance;
pub mod mfa_rate_limit;
pub mod mtls;
pub mod rate_limit;
pub mod rbac;
pub mod security;
pub mod size_limit;
pub mod validation;

// Test modules
#[cfg(test)]
pub mod rate_limit_test;

// Re-export core middleware
pub use auth::{
    AUTH_USER_KEY, AuthState, AuthUser, AuthUserExt, RequireAuth, apply_auth_layer, auth_middleware,
};
pub use cors::{CorsConfig, Environment, development_cors, production_cors};
pub use csrf::{
    CsrfConfig, CsrfState, apply_csrf_layer, csrf_protection_middleware,
    generate_csrf_token_response,
};

// Re-export rate limiting middleware
pub use adaptive_rate_limit::{
    AdaptiveRateLimitConfig, AdaptiveRateLimitLayer, AdaptiveRateLimitMiddleware,
    AdaptiveRateLimiter, ThreatLevel, adaptive_rate_limit_layer, adaptive_rate_limit_middleware,
};
pub use adaptive_rate_limit_integration::{
    AuthResultExt, RateLimitResponse, create_rate_limit_response, extract_ip,
};
pub use mfa_performance::{
    MfaCacheMiddleware, MfaDatabaseMiddleware, MfaServiceMonitor, mfa_performance_middleware,
};
pub use mfa_rate_limit::{MfaRateLimitConfig, MfaRateLimiterState, mfa_rate_limit_middleware};
pub use rate_limit::{
    RateLimitConfig, RateLimitLayer, RateLimitMiddleware, RateLimiterState, rate_limit_layer,
    rate_limit_middleware,
};

// Re-export other middleware
pub use compression::{ContentEncoding, compression_middleware};
pub use mtls::{ClientCertInfo, MtlsConfig, mtls_middleware};
pub use rbac::{RbacLayer, rbac_middleware};
pub use security::{
    SecurityMonitoringConfig, SecurityMonitoringState, security_monitoring_middleware,
};
pub use size_limit::layer::RequestSizeLimitLayer;
pub use size_limit::{MAX_REQUEST_BODY_SIZE, request_size_limit_middleware};
pub use validation::{InputValidationConfig, input_validation_middleware};
