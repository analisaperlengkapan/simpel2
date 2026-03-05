//! Integration helpers for adaptive rate limiting with authentication handlers
//!
//! This module provides utilities to integrate the adaptive rate limiter with
//! authentication handlers, automatically recording failed attempts and adjusting
//! threat levels.

use std::sync::Arc;

use axum::{
    Json,
    extract::ConnectInfo,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::warn;

use crate::middleware::adaptive_rate_limit::AdaptiveRateLimiter;
use authenc_types::AuthencError;

/// Extension trait for authentication results to integrate with adaptive rate limiting
pub trait AuthResultExt<T> {
    /// Record the authentication result with the adaptive rate limiter
    fn record_auth_result(
        self,
        limiter: &Arc<AdaptiveRateLimiter>,
        ip: &str,
    ) -> Result<T, AuthencError>;
}

impl<T> AuthResultExt<T> for Result<T, AuthencError> {
    fn record_auth_result(
        self,
        limiter: &Arc<AdaptiveRateLimiter>,
        ip: &str,
    ) -> Result<T, AuthencError> {
        match &self {
            Err(AuthencError::AuthenticationFailed(_))
            | Err(AuthencError::InvalidCredentials)
            | Err(AuthencError::InvalidMfaCode) => {
                limiter.record_failed_attempt(ip);
                warn!(%ip, "Failed authentication attempt recorded");
            }
            Ok(_) => {
                // Successful authentication - could implement success tracking here
            }
            _ => {
                // Other errors don't affect threat level
            }
        }
        self
    }
}

/// Helper to extract IP address from connection info
pub fn extract_ip(conn_info: &ConnectInfo<std::net::SocketAddr>) -> String {
    conn_info.0.ip().to_string()
}

/// Response type for rate-limited requests with threat level information
#[derive(Serialize)]
pub struct RateLimitResponse {
    pub error: String,
    pub message: String,
    pub threat_level: String,
    pub limit: u64,
    pub retry_after: u64,
}

impl IntoResponse for RateLimitResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("retry-after", self.retry_after.to_string()),
                ("x-ratelimit-limit", self.limit.to_string()),
                ("x-ratelimit-remaining", "0".to_string()),
                ("x-threat-level", self.threat_level.clone()),
            ],
            Json(self),
        )
            .into_response()
    }
}

/// Create a rate limit response with current threat level information
pub fn create_rate_limit_response(limiter: &AdaptiveRateLimiter) -> RateLimitResponse {
    let threat_level = limiter.get_threat_level();
    let limit = threat_level.rate_limit();

    RateLimitResponse {
        error: "rate_limit_exceeded".to_string(),
        message: format!(
            "Rate limit exceeded. Current threat level: {}. Limit: {} requests/min.",
            threat_level.name(),
            limit
        ),
        threat_level: threat_level.name().to_string(),
        limit,
        retry_after: 60,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::adaptive_rate_limit::AdaptiveRateLimitConfig;

    #[tokio::test]
    async fn test_auth_result_ext() {
        let config = AdaptiveRateLimitConfig::default();
        let limiter = Arc::new(AdaptiveRateLimiter::new(config));

        // Test failed authentication
        let result: Result<(), AuthencError> = Err(AuthencError::AuthenticationFailed("test".to_string()));
        let _ = result.record_auth_result(&limiter, "192.168.1.1");

        // Test successful authentication
        let result: Result<(), AuthencError> = Ok(());
        let _ = result.record_auth_result(&limiter, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_create_rate_limit_response() {
        let config = AdaptiveRateLimitConfig::default();
        let limiter = AdaptiveRateLimiter::new(config);

        let response = create_rate_limit_response(&limiter);
        assert_eq!(response.error, "rate_limit_exceeded");
        assert_eq!(response.threat_level, "normal");
        assert_eq!(response.limit, 100);
    }
}
