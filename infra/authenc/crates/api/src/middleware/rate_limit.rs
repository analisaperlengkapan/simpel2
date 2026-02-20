//! Rate limiting middleware

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per IP per minute
    pub per_ip_limit: u32,
    /// Maximum requests per user per minute
    pub per_user_limit: u32,
    /// Window duration
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            per_ip_limit: 100,
            per_user_limit: 1000,
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    ip_buckets: Arc<Mutex<HashMap<IpAddr, RateLimitBucket>>>,
    user_buckets: Arc<Mutex<HashMap<Uuid, RateLimitBucket>>>,
}

/// Rate limit bucket for tracking requests
#[derive(Debug, Clone)]
struct RateLimitBucket {
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            ip_buckets: Arc::new(Mutex::new(HashMap::new())),
            user_buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if IP is rate limited
    pub fn check_ip(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut buckets = self.ip_buckets.lock().unwrap();
        let now = Instant::now();

        let bucket = buckets.entry(ip).or_insert(RateLimitBucket {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(bucket.window_start) >= self.config.window_duration {
            bucket.count = 0;
            bucket.window_start = now;
        }

        // Check limit
        if bucket.count >= self.config.per_ip_limit {
            let retry_after = self.config.window_duration
                - now.duration_since(bucket.window_start);
            return Err(RateLimitError::IpLimitExceeded {
                retry_after: retry_after.as_secs(),
            });
        }

        bucket.count += 1;
        Ok(())
    }

    /// Check if user is rate limited
    pub fn check_user(&self, user_id: Uuid) -> Result<(), RateLimitError> {
        let mut buckets = self.user_buckets.lock().unwrap();
        let now = Instant::now();

        let bucket = buckets.entry(user_id).or_insert(RateLimitBucket {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(bucket.window_start) >= self.config.window_duration {
            bucket.count = 0;
            bucket.window_start = now;
        }

        // Check limit
        if bucket.count >= self.config.per_user_limit {
            let retry_after = self.config.window_duration
                - now.duration_since(bucket.window_start);
            return Err(RateLimitError::UserLimitExceeded {
                retry_after: retry_after.as_secs(),
            });
        }

        bucket.count += 1;
        Ok(())
    }

    /// Clean up expired buckets (should be called periodically)
    pub fn cleanup(&self) {
        let now = Instant::now();

        // Clean IP buckets
        {
            let mut buckets = self.ip_buckets.lock().unwrap();
            buckets.retain(|_, bucket| {
                now.duration_since(bucket.window_start) < self.config.window_duration
            });
        }

        // Clean user buckets
        {
            let mut buckets = self.user_buckets.lock().unwrap();
            buckets.retain(|_, bucket| {
                now.duration_since(bucket.window_start) < self.config.window_duration
            });
        }
    }
}

/// Rate limit error
#[derive(Debug)]
pub enum RateLimitError {
    IpLimitExceeded { retry_after: u64 },
    UserLimitExceeded { retry_after: u64 },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::IpLimitExceeded { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                "Rate limit exceeded for IP address",
            )
                .into_response(),
            RateLimitError::UserLimitExceeded { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                "Rate limit exceeded for user",
            )
                .into_response(),
        }
    }
}

/// Rate limiting middleware
///
/// Applies per-IP rate limiting to all requests.
/// Per-user rate limiting requires JWT authentication middleware to run first.
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    limiter: axum::extract::State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Check IP rate limit
    limiter.check_ip(addr.ip())?;

    // TODO: Extract user ID from JWT claims (if authenticated)
    // and check per-user rate limit

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Adaptive rate limiting based on risk score
///
/// This is a placeholder for future implementation.
/// Risk score can be calculated based on:
/// - Failed login attempts
/// - Suspicious patterns
/// - Geographic location
/// - Device fingerprint
pub struct AdaptiveRateLimiter {
    base_limiter: RateLimiter,
}

impl AdaptiveRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            base_limiter: RateLimiter::new(config),
        }
    }

    /// Calculate risk score for a request
    ///
    /// Returns a score from 0.0 (low risk) to 1.0 (high risk)
    pub fn calculate_risk_score(&self, _ip: IpAddr, _user_id: Option<Uuid>) -> f32 {
        // TODO: Implement risk scoring
        // - Check failed login attempts
        // - Check geographic location
        // - Check device fingerprint
        // - Check time of day
        // - Check request patterns

        0.0 // Placeholder
    }

    /// Adjust rate limit based on risk score
    pub fn adjusted_limit(&self, base_limit: u32, risk_score: f32) -> u32 {
        // Higher risk = lower limit
        let multiplier = 1.0 - (risk_score * 0.8); // Max 80% reduction
        (base_limit as f32 * multiplier).max(1.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_rate_limiter_ip() {
        let config = RateLimitConfig {
            per_ip_limit: 5,
            per_user_limit: 100,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check_ip(ip).is_ok());
        }

        // 6th request should fail
        assert!(limiter.check_ip(ip).is_err());
    }

    #[test]
    fn test_rate_limiter_user() {
        let config = RateLimitConfig {
            per_ip_limit: 100,
            per_user_limit: 10,
            window_duration: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let user_id = Uuid::new_v4();

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(limiter.check_user(user_id).is_ok());
        }

        // 11th request should fail
        assert!(limiter.check_user(user_id).is_err());
    }

    #[test]
    fn test_adaptive_rate_limiter() {
        let config = RateLimitConfig::default();
        let limiter = AdaptiveRateLimiter::new(config);

        // Low risk should not reduce limit much
        let low_risk_limit = limiter.adjusted_limit(100, 0.1);
        assert!(low_risk_limit >= 90);

        // High risk should significantly reduce limit
        let high_risk_limit = limiter.adjusted_limit(100, 0.9);
        assert!(high_risk_limit <= 30);
    }
}
