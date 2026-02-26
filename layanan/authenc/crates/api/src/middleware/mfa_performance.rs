//! MFA Performance Monitoring Middleware
//!
//! Monitors performance of MFA operations including cache hit rates,
//! database query latency, and service availability.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// MFA service monitor for tracking performance metrics
#[derive(Debug, Clone)]
pub struct MfaServiceMonitor {
    /// Whether monitoring is enabled
    pub enabled: bool,
}

impl MfaServiceMonitor {
    /// Create new MFA service monitor
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl Default for MfaServiceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// MFA cache middleware for caching MFA state
#[derive(Debug, Clone)]
pub struct MfaCacheMiddleware {
    pub cache_ttl_secs: u64,
}

impl MfaCacheMiddleware {
    /// Create new MFA cache middleware
    pub fn new(cache_ttl_secs: u64) -> Self {
        Self { cache_ttl_secs }
    }
}

impl Default for MfaCacheMiddleware {
    fn default() -> Self {
        Self::new(300)
    }
}

/// MFA database middleware for tracking DB operations
#[derive(Debug, Clone)]
pub struct MfaDatabaseMiddleware {
    pub slow_query_threshold_ms: u64,
}

impl MfaDatabaseMiddleware {
    /// Create new MFA database middleware
    pub fn new(slow_query_threshold_ms: u64) -> Self {
        Self {
            slow_query_threshold_ms,
        }
    }
}

impl Default for MfaDatabaseMiddleware {
    fn default() -> Self {
        Self::new(100)
    }
}

/// MFA performance monitoring middleware function
///
/// Tracks timing of MFA-related requests and logs performance metrics.
pub async fn mfa_performance_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;

    let duration = start.elapsed();

    // Log performance for MFA-related paths
    if path.contains("/mfa") || path.contains("/totp") {
        tracing::debug!(
            path = %path,
            duration_ms = %duration.as_millis(),
            status = %response.status(),
            "MFA request completed"
        );
    }

    response
}
