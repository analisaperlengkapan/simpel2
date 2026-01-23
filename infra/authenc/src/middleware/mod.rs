//! # Middleware Module
//!
//! This module contains various middleware components for the application, organized into categories:
//! - Core middleware (security, rate limiting)
//! - Authentication and authorization middleware
//! - Axum-specific middleware implementations
//!
//! ## Features
//! - `axum`: Enables Axum-specific middleware implementations
//! - `actix-web`: Enables Actix-Web-specific middleware implementations (not yet implemented)

// Axum middleware (primary implementation)

/// Authentication middleware for Axum web framework
/// Handles JWT token validation, user authentication, and session management.
/// Integrates with the authentication system to protect API endpoints.
/// Supports Bearer token authentication and session-based auth.
pub mod auth_middleware;

/// Rate limiting middleware for Axum
/// Implements rate limiting to prevent abuse and DoS attacks.
/// Supports various rate limiting strategies including sliding window,
/// fixed window, and token bucket algorithms.
/// Configurable limits per endpoint and user.
pub mod rate_limit;

/// Adaptive rate limiting middleware for Axum
/// Implements adaptive rate limiting with threat level detection.
/// Automatically adjusts rate limits based on failed authentication attempts
/// and suspicious patterns. Supports multiple threat levels:
/// - Normal: 100 requests/min
/// - Elevated: 50 requests/min
/// - High: 20 requests/min
/// - Critical: 5 requests/min
pub mod adaptive_rate_limit;

/// Integration helpers for adaptive rate limiting
/// Provides utilities to integrate adaptive rate limiting with authentication
/// handlers, automatically recording failed attempts and adjusting threat levels.
pub mod adaptive_rate_limit_integration;

/// Role-Based Access Control (RBAC) middleware for Axum
/// Enforces role-based permissions on API endpoints.
/// Checks user roles and permissions before allowing access to resources.
/// Integrates with the authorization system for fine-grained access control.
pub mod rbac;

/// Response compression middleware for Axum
/// Compresses HTTP responses to reduce bandwidth usage.
/// Supports gzip, deflate, and brotli compression algorithms.
/// Automatically negotiates compression based on client capabilities.
pub mod compression;

/// Input validation middleware for Axum
/// Validates and sanitizes incoming request data.
/// Prevents injection attacks and malformed data.
/// Supports custom validation rules and error handling.
pub mod input_validation;

/// Mutual TLS (mTLS) authentication middleware
/// Provides client certificate validation for API endpoints.
/// Can be configured to work with reverse proxies that handle
/// TLS termination and forward certificate information via headers.
pub mod mtls;

/// CSRF protection middleware for Axum
/// Prevents Cross-Site Request Forgery attacks.
/// Validates CSRF tokens on state-changing requests.
/// Configurable token generation and validation rules.
pub mod csrf_protection;

/// Security monitoring and alerting middleware for Axum
/// Monitors requests for suspicious activity and security events.
/// Logs authentication attempts, authorization failures, and attacks.
/// Integrates with audit logging for compliance and forensics.
pub mod security_monitoring;

/// MFA-specific rate limiting middleware for Axum
/// Implements specialized rate limiting for MFA operations including:
/// - Progressive delays for failed attempts
/// - Account lockout protection
/// - IP-based and user-based rate limiting
/// - Separate limits for setup vs verification operations
pub mod mfa_rate_limit;

/// MFA performance monitoring middleware for Axum
/// Automatically collects performance metrics for MFA operations including:
/// - Response times and success rates
/// - Cache hit ratios and database query performance
/// - Error categorization and alerting
/// - Real-time performance dashboards
pub mod mfa_performance_middleware;

/// Request size limit middleware for Axum
/// Enforces maximum request body size to prevent DoS attacks.
/// Rejects requests exceeding 1MB to protect against resource exhaustion.
/// Checks Content-Length header and limits body reading.
pub mod request_size_limit;

// Re-export middleware types for easier access
pub use rate_limit::{
    RateLimitConfig, RateLimitLayer, RateLimitMiddleware, RateLimiterState, rate_limit_layer,
    rate_limit_middleware,
};

pub use adaptive_rate_limit::{
    AdaptiveRateLimitConfig, AdaptiveRateLimitLayer, AdaptiveRateLimitMiddleware,
    AdaptiveRateLimiter, ThreatLevel, adaptive_rate_limit_layer, adaptive_rate_limit_middleware,
};

pub use adaptive_rate_limit_integration::{
    AuthResultExt, RateLimitResponse, create_rate_limit_response, extract_ip,
};

pub use compression::{ContentEncoding, compression_middleware};
pub use csrf_protection::{
    CsrfConfig, CsrfState, csrf_protection_middleware, generate_csrf_token_response,
};
pub use input_validation::{InputValidationConfig, input_validation_middleware};
pub use security_monitoring::{
    SecurityMonitoringConfig, SecurityMonitoringState, security_monitoring_middleware,
};

// Re-exports for convenience
pub use auth_middleware::{AuthState, auth_middleware};

pub use rbac::{RbacLayer, rbac_middleware};

pub use mfa_rate_limit::{MfaRateLimitConfig, MfaRateLimiterState, mfa_rate_limit_middleware};

pub use mfa_performance_middleware::{
    MfaCacheMiddleware, MfaDatabaseMiddleware, MfaServiceMonitor, mfa_performance_middleware,
};

pub use request_size_limit::{
    MAX_REQUEST_BODY_SIZE, layer::RequestSizeLimitLayer, request_size_limit_middleware,
};
