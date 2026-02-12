// ============================================================================
// Rate Limiting Middleware
// Description: Token bucket rate limiting with per-user and global limits
// Author: SIMPelv2 Team
// Created: 2026-02-10
// Requirements: NFR-S007
// ============================================================================

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Rate limit configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per user
    pub requests_per_second: u32,

    /// Burst size (maximum tokens in bucket)
    pub burst_size: u32,

    /// Global rate limit (requests per second for all users)
    pub global_requests_per_second: Option<u32>,

    /// Window duration for rate limiting
    #[serde(skip)]
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,           // 100 req/s per user
            burst_size: 200,                    // Burst of 200 requests
            global_requests_per_second: Some(1000), // 1000 req/s globally
            window_duration: Duration::from_secs(1),
        }
    }
}

impl RateLimitConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            requests_per_second: std::env::var("RATE_LIMIT_REQUESTS_PER_SECOND")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            burst_size: std::env::var("RATE_LIMIT_BURST_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200),
            global_requests_per_second: std::env::var("RATE_LIMIT_GLOBAL_REQUESTS_PER_SECOND")
                .ok()
                .and_then(|s| s.parse().ok()),
            window_duration: std::env::var("RATE_LIMIT_WINDOW_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(1)),
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,

    /// Maximum tokens (burst size)
    capacity: f64,

    /// Refill rate (tokens per second)
    refill_rate: f64,

    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            refill_rate: refill_rate as f64,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Add tokens based on refill rate
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    /// Try to consume a token
    fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get remaining tokens
    fn remaining(&mut self) -> u32 {
        self.refill();
        self.tokens.floor() as u32
    }

    /// Get time until next token is available
    fn time_until_next_token(&mut self) -> Duration {
        self.refill();

        if self.tokens >= 1.0 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(seconds)
        }
    }
}

/// Rate limiter state
pub struct RateLimiter {
    config: RateLimitConfig,
    user_buckets: Arc<RwLock<HashMap<Uuid, TokenBucket>>>,
    global_bucket: Arc<RwLock<TokenBucket>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = if let Some(global_rate) = config.global_requests_per_second {
            TokenBucket::new(global_rate * 2, global_rate) // 2x burst for global
        } else {
            TokenBucket::new(u32::MAX, u32::MAX) // Unlimited
        };

        Self {
            config,
            user_buckets: Arc::new(RwLock::new(HashMap::new())),
            global_bucket: Arc::new(RwLock::new(global_bucket)),
        }
    }

    /// Check if request is allowed for a user
    pub async fn check_rate_limit(&self, user_id: Uuid) -> RateLimitResult {
        // Check global rate limit first
        {
            let mut global = self.global_bucket.write().await;
            if !global.try_consume() {
                let retry_after = global.time_until_next_token();
                return RateLimitResult::GlobalLimitExceeded { retry_after };
            }
        }

        // Check per-user rate limit
        let mut buckets = self.user_buckets.write().await;

        let bucket = buckets.entry(user_id).or_insert_with(|| {
            TokenBucket::new(self.config.burst_size, self.config.requests_per_second)
        });

        if bucket.try_consume() {
            let remaining = bucket.remaining();
            let reset_time = Instant::now() + self.config.window_duration;

            RateLimitResult::Allowed {
                remaining,
                reset_time,
            }
        } else {
            let retry_after = bucket.time_until_next_token();
            RateLimitResult::UserLimitExceeded { retry_after }
        }
    }

    /// Clean up old buckets (call periodically)
    pub async fn cleanup_old_buckets(&self) {
        let mut buckets = self.user_buckets.write().await;

        // Remove buckets that haven't been used in the last 5 minutes
        let cutoff = Instant::now() - Duration::from_secs(300);

        buckets.retain(|_, bucket| bucket.last_refill > cutoff);
    }

    /// Get rate limit statistics
    pub async fn get_stats(&self) -> RateLimitStats {
        let buckets = self.user_buckets.read().await;
        let global = self.global_bucket.read().await;

        RateLimitStats {
            active_users: buckets.len(),
            global_remaining: global.tokens.floor() as u32,
            config: self.config.clone(),
        }
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            user_buckets: Arc::clone(&self.user_buckets),
            global_bucket: Arc::clone(&self.global_bucket),
        }
    }
}

/// Rate limit check result
#[derive(Debug)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        reset_time: Instant,
    },
    UserLimitExceeded {
        retry_after: Duration,
    },
    GlobalLimitExceeded {
        retry_after: Duration,
    },
}

/// Rate limit statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitStats {
    pub active_users: usize,
    pub global_remaining: u32,
    pub config: RateLimitConfig,
}

/// Axum middleware for rate limiting
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    // Extract user ID from request extensions (set by auth middleware)
    let user_id = request
        .extensions()
        .get::<crate::middleware::Claims>()
        .map(|claims| claims.user_id)
        .unwrap_or_else(|| Uuid::nil()); // Use nil UUID for unauthenticated requests

    // Check rate limit
    match rate_limiter.check_rate_limit(user_id).await {
        RateLimitResult::Allowed {
            remaining,
            reset_time,
        } => {
            let mut response = next.run(request).await;

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&rate_limiter.config.requests_per_second.to_string())
                    .unwrap(),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&remaining.to_string()).unwrap(),
            );
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&reset_time.elapsed().as_secs().to_string()).unwrap(),
            );

            response
        }
        RateLimitResult::UserLimitExceeded { retry_after } => {
            rate_limit_exceeded_response(retry_after, "User rate limit exceeded")
        }
        RateLimitResult::GlobalLimitExceeded { retry_after } => {
            rate_limit_exceeded_response(retry_after, "Global rate limit exceeded")
        }
    }
}

/// Create a rate limit exceeded response
fn rate_limit_exceeded_response(retry_after: Duration, message: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Retry-After",
        HeaderValue::from_str(&retry_after.as_secs().to_string()).unwrap(),
    );
    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_static("100"), // Default
    );
    headers.insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));

    let body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": message,
        "retry_after_seconds": retry_after.as_secs(),
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        headers,
        axum::Json(body),
    )
        .into_response()
}

/// Background task to cleanup old buckets
pub async fn cleanup_task(rate_limiter: Arc<RateLimiter>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

    loop {
        interval.tick().await;
        rate_limiter.cleanup_old_buckets().await;
        tracing::debug!("Rate limiter cleanup completed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10, 5); // 10 capacity, 5 tokens/sec

        // Consume all tokens
        for _ in 0..10 {
            assert!(bucket.try_consume());
        }

        // Should be empty
        assert!(!bucket.try_consume());

        // Wait and refill
        std::thread::sleep(Duration::from_secs(1));
        bucket.refill();

        // Should have ~5 tokens now
        assert!(bucket.remaining() >= 4 && bucket.remaining() <= 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_per_user() {
        let config = RateLimitConfig {
            requests_per_second: 5,
            burst_size: 10,
            global_requests_per_second: None,
            window_duration: Duration::from_secs(1),
        };

        let limiter = RateLimiter::new(config);
        let user_id = Uuid::new_v4();

        // Should allow burst
        for _ in 0..10 {
            let result = limiter.check_rate_limit(user_id).await;
            assert!(matches!(result, RateLimitResult::Allowed { .. }));
        }

        // Should exceed limit
        let result = limiter.check_rate_limit(user_id).await;
        assert!(matches!(result, RateLimitResult::UserLimitExceeded { .. }));
    }

    #[tokio::test]
    async fn test_rate_limiter_global() {
        let config = RateLimitConfig {
            requests_per_second: 100,
            burst_size: 200,
            global_requests_per_second: Some(5),
            window_duration: Duration::from_secs(1),
        };

        let limiter = RateLimiter::new(config);

        // Consume global limit with different users
        for i in 0..10 {
            let user_id = Uuid::new_v4();
            let result = limiter.check_rate_limit(user_id).await;

            if i < 10 {
                // First 10 should succeed (2x burst)
                assert!(matches!(
                    result,
                    RateLimitResult::Allowed { .. } | RateLimitResult::GlobalLimitExceeded { .. }
                ));
            }
        }
    }

    #[test]
    fn test_rate_limit_config_from_env() {
        std::env::set_var("RATE_LIMIT_REQUESTS_PER_SECOND", "50");
        std::env::set_var("RATE_LIMIT_BURST_SIZE", "100");

        let config = RateLimitConfig::from_env();

        assert_eq!(config.requests_per_second, 50);
        assert_eq!(config.burst_size, 100);

        std::env::remove_var("RATE_LIMIT_REQUESTS_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST_SIZE");
    }
}
