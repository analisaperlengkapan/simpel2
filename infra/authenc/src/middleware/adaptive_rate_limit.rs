//! Adaptive rate limiting middleware with threat level detection
//!
//! This module provides adaptive rate limiting that adjusts limits based on
//! detected threat levels. Failed authentication attempts and suspicious patterns
//! increase the threat level, which reduces rate limits to protect the system.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::AuthencError;

/// Threat level for adaptive rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ThreatLevel {
    /// Normal operations (0-2)
    Normal = 0,
    /// Elevated threat (3-5)
    Elevated = 3,
    /// High threat (6-8)
    High = 6,
    /// Critical threat (9+)
    Critical = 9,
}

impl ThreatLevel {
    /// Get rate limit for this threat level (requests per minute)
    pub fn rate_limit(&self) -> u64 {
        match self {
            ThreatLevel::Normal => 100,
            ThreatLevel::Elevated => 50,
            ThreatLevel::High => 20,
            ThreatLevel::Critical => 5,
        }
    }

    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Self {
        match value {
            0..=2 => ThreatLevel::Normal,
            3..=5 => ThreatLevel::Elevated,
            6..=8 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        }
    }

    /// Get the name of the threat level
    pub fn name(&self) -> &'static str {
        match self {
            ThreatLevel::Normal => "normal",
            ThreatLevel::Elevated => "elevated",
            ThreatLevel::High => "high",
            ThreatLevel::Critical => "critical",
        }
    }

    /// Convert to the dynamic config ThreatLevel
    pub fn to_dynamic_threat_level(&self) -> crate::config::dynamic::ThreatLevel {
        match self {
            ThreatLevel::Normal => crate::config::dynamic::ThreatLevel::Low,
            ThreatLevel::Elevated => crate::config::dynamic::ThreatLevel::Medium,
            ThreatLevel::High => crate::config::dynamic::ThreatLevel::High,
            ThreatLevel::Critical => crate::config::dynamic::ThreatLevel::Critical,
        }
    }
}

/// Configuration for adaptive rate limiting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveRateLimitConfig {
    /// Whether adaptive rate limiting is enabled
    pub enabled: bool,
    /// Path patterns to exclude from rate limiting
    pub excluded_paths: Vec<String>,
    /// Number of failed attempts before increasing threat level
    pub failed_attempts_threshold: u64,
    /// Time window for tracking failed attempts (seconds)
    pub failed_attempts_window_secs: u64,
    /// How long to maintain elevated threat level (seconds)
    pub threat_level_decay_secs: u64,
    /// Maximum threat level (0-10)
    pub max_threat_level: u8,
}

impl Default for AdaptiveRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_paths: vec![
                "/health".to_string(),
                "/health/ready".to_string(),
                "/health/live".to_string(),
                "/metrics".to_string(),
            ],
            failed_attempts_threshold: 5,
            failed_attempts_window_secs: 300, // 5 minutes
            threat_level_decay_secs: 600,     // 10 minutes
            max_threat_level: 10,
        }
    }
}

/// Rate limit entry for tracking requests
#[derive(Debug)]
struct RateLimitEntry {
    /// Number of requests in current window
    count: AtomicU64,
    /// Start of current window
    window_start: Instant,
    /// Number of failed attempts
    failed_attempts: AtomicU64,
    /// Last failed attempt timestamp
    last_failed_attempt: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            window_start: Instant::now(),
            failed_attempts: AtomicU64::new(0),
            last_failed_attempt: Instant::now(),
        }
    }
}

/// Adaptive rate limiter state
#[derive(Clone)]
pub struct AdaptiveRateLimiter {
    config: AdaptiveRateLimitConfig,
    /// Per-IP rate limit entries
    limits: Arc<DashMap<String, RateLimitEntry>>,
    /// Global threat level (0-10)
    threat_level: Arc<AtomicU8>,
    /// Last threat level update
    last_threat_update: Arc<std::sync::Mutex<Instant>>,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter
    pub fn new(config: AdaptiveRateLimitConfig) -> Self {
        let limiter = Self {
            config: config.clone(),
            limits: Arc::new(DashMap::with_capacity(10_000)),
            threat_level: Arc::new(AtomicU8::new(0)),
            last_threat_update: Arc::new(std::sync::Mutex::new(Instant::now())),
        };

        // Spawn background task for cleanup and threat level decay
        if config.enabled {
            let limits = limiter.limits.clone();
            let threat_level = limiter.threat_level.clone();
            let last_threat_update = limiter.last_threat_update.clone();
            let decay_duration = Duration::from_secs(config.threat_level_decay_secs);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let now = Instant::now();

                    // Clean up old entries
                    limits.retain(|_, entry| {
                        now.duration_since(entry.window_start) < Duration::from_secs(120)
                    });

                    // Decay threat level if no recent updates
                    let last_update = *last_threat_update.lock().unwrap();
                    if now.duration_since(last_update) > decay_duration {
                        let current = threat_level.load(Ordering::Relaxed);
                        if current > 0 {
                            let new_level = current.saturating_sub(1);
                            threat_level.store(new_level, Ordering::Relaxed);
                            info!(
                                old_level = current,
                                new_level = new_level,
                                "Threat level decayed due to inactivity"
                            );
                            *last_threat_update.lock().unwrap() = now;
                        }
                    }
                }
            });
        }

        limiter
    }

    /// Get current threat level
    pub fn get_threat_level(&self) -> ThreatLevel {
        let level = self.threat_level.load(Ordering::Relaxed);
        ThreatLevel::from_u8(level)
    }

    /// Record a failed authentication attempt
    pub fn record_failed_attempt(&self, ip: &str) {
        if !self.config.enabled {
            return;
        }

        let now = Instant::now();
        let mut entry = self
            .limits
            .entry(ip.to_string())
            .or_insert_with(RateLimitEntry::new);

        // Reset failed attempts counter if window expired
        let window_duration = Duration::from_secs(self.config.failed_attempts_window_secs);
        if now.duration_since(entry.last_failed_attempt) > window_duration {
            entry.failed_attempts.store(0, Ordering::Relaxed);
        }

        // Increment failed attempts
        let failed_count = entry.failed_attempts.fetch_add(1, Ordering::Relaxed) + 1;
        entry.last_failed_attempt = now;

        debug!(
            %ip,
            failed_count,
            threshold = self.config.failed_attempts_threshold,
            "Recorded failed authentication attempt"
        );

        // Increase threat level if threshold exceeded
        if failed_count >= self.config.failed_attempts_threshold {
            self.increase_threat_level(ip, failed_count);
        }
    }

    /// Increase global threat level based on failed attempts
    fn increase_threat_level(&self, ip: &str, failed_count: u64) {
        let current = self.threat_level.load(Ordering::Relaxed);
        if current >= self.config.max_threat_level {
            return;
        }

        // Calculate threat increase (1 level per threshold exceeded)
        let threshold_exceeded = (failed_count / self.config.failed_attempts_threshold) as u8;
        let new_level = (current + threshold_exceeded).min(self.config.max_threat_level);

        if new_level > current {
            self.threat_level.store(new_level, Ordering::Relaxed);
            *self.last_threat_update.lock().unwrap() = Instant::now();

            let old_threat = ThreatLevel::from_u8(current);
            let new_threat = ThreatLevel::from_u8(new_level);

            warn!(
                %ip,
                failed_count,
                old_level = current,
                new_level = new_level,
                old_threat = old_threat.name(),
                new_threat = new_threat.name(),
                old_limit = old_threat.rate_limit(),
                new_limit = new_threat.rate_limit(),
                "Threat level increased due to failed attempts"
            );
        }
    }

    /// Check if request should be rate limited
    pub fn check_rate_limit(&self, path: &str, ip: &str) -> Result<(), AuthencError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Skip rate limiting for excluded paths
        if self
            .config
            .excluded_paths
            .iter()
            .any(|p| path.starts_with(p))
        {
            return Ok(());
        }

        let now = Instant::now();
        let threat_level = self.get_threat_level();
        let limit = threat_level.rate_limit();

        // Get or create entry for this IP
        let mut entry = self
            .limits
            .entry(ip.to_string())
            .or_insert_with(RateLimitEntry::new);

        // Reset counter if window expired (1 minute)
        if now.duration_since(entry.window_start) > Duration::from_secs(60) {
            entry.count.store(0, Ordering::Relaxed);
            entry.window_start = now;
        }

        // Check rate limit
        let count = entry.count.fetch_add(1, Ordering::Relaxed) + 1;

        if count > limit {
            warn!(
                %ip,
                %path,
                count,
                limit,
                threat_level = threat_level.name(),
                "Rate limit exceeded"
            );
            return Err(AuthencError::RateLimitExceeded);
        }

        debug!(
            %ip,
            %path,
            count,
            limit,
            threat_level = threat_level.name(),
            "Request within rate limit"
        );

        Ok(())
    }
}

/// Middleware function for adaptive rate limiting
pub async fn adaptive_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(state): State<Arc<AdaptiveRateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let path = request.uri().path();
    let ip = addr.ip().to_string();

    match state.check_rate_limit(path, &ip) {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(AuthencError::RateLimitExceeded) => {
            let _threat_level = state.get_threat_level();
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create an adaptive rate limit layer
pub fn adaptive_rate_limit_layer(config: AdaptiveRateLimitConfig) -> AdaptiveRateLimitLayer {
    AdaptiveRateLimitLayer::new(AdaptiveRateLimiter::new(config))
}

/// Layer for applying adaptive rate limiting
#[derive(Clone)]
pub struct AdaptiveRateLimitLayer {
    state: Arc<AdaptiveRateLimiter>,
}

impl AdaptiveRateLimitLayer {
    /// Create a new adaptive rate limit layer
    pub fn new(limiter: AdaptiveRateLimiter) -> Self {
        Self {
            state: Arc::new(limiter),
        }
    }

    /// Get the underlying rate limiter
    pub fn limiter(&self) -> &Arc<AdaptiveRateLimiter> {
        &self.state
    }
}

impl<S> tower::Layer<S> for AdaptiveRateLimitLayer {
    type Service = AdaptiveRateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AdaptiveRateLimitMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

/// Middleware service that enforces adaptive rate limiting
#[derive(Clone)]
pub struct AdaptiveRateLimitMiddleware<S> {
    inner: S,
    state: Arc<AdaptiveRateLimiter>,
}

impl<S, B> tower::Service<Request<B>> for AdaptiveRateLimitMiddleware<S>
where
    S: tower::Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let path = req.uri().path().to_string();
        let ip = req
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let state = self.state.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            match state.check_rate_limit(&path, &ip) {
                Ok(_) => future.await,
                Err(AuthencError::RateLimitExceeded) => {
                    let threat_level = state.get_threat_level();
                    let limit = threat_level.rate_limit();

                    Ok((
                        StatusCode::TOO_MANY_REQUESTS,
                        [
                            ("retry-after", "60".to_string()),
                            ("x-ratelimit-limit", limit.to_string()),
                            ("x-ratelimit-remaining", "0".to_string()),
                            ("x-threat-level", threat_level.name().to_string()),
                        ],
                        format!(
                            "Rate limit exceeded. Current threat level: {}. Limit: {} requests/min.",
                            threat_level.name(),
                            limit
                        ),
                    )
                        .into_response())
                }
                Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level_conversion() {
        assert_eq!(ThreatLevel::from_u8(0), ThreatLevel::Normal);
        assert_eq!(ThreatLevel::from_u8(2), ThreatLevel::Normal);
        assert_eq!(ThreatLevel::from_u8(3), ThreatLevel::Elevated);
        assert_eq!(ThreatLevel::from_u8(5), ThreatLevel::Elevated);
        assert_eq!(ThreatLevel::from_u8(6), ThreatLevel::High);
        assert_eq!(ThreatLevel::from_u8(8), ThreatLevel::High);
        assert_eq!(ThreatLevel::from_u8(9), ThreatLevel::Critical);
        assert_eq!(ThreatLevel::from_u8(10), ThreatLevel::Critical);
    }

    #[test]
    fn test_threat_level_rate_limits() {
        assert_eq!(ThreatLevel::Normal.rate_limit(), 100);
        assert_eq!(ThreatLevel::Elevated.rate_limit(), 50);
        assert_eq!(ThreatLevel::High.rate_limit(), 20);
        assert_eq!(ThreatLevel::Critical.rate_limit(), 5);
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiting() {
        let config = AdaptiveRateLimitConfig {
            enabled: true,
            excluded_paths: vec!["/health".to_string()],
            failed_attempts_threshold: 3,
            failed_attempts_window_secs: 300,
            threat_level_decay_secs: 600,
            max_threat_level: 10,
        };

        let limiter = AdaptiveRateLimiter::new(config);

        // Initially at normal threat level
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Normal);

        // Should allow 100 requests at normal level
        for i in 0..100 {
            assert!(
                limiter.check_rate_limit("/api/test", "192.168.1.1").is_ok(),
                "Request {} should succeed at normal threat level",
                i + 1
            );
        }

        // 101st request should be rate limited
        assert!(
            limiter
                .check_rate_limit("/api/test", "192.168.1.1")
                .is_err()
        );

        // Health check should not be rate limited
        assert!(limiter.check_rate_limit("/health", "192.168.1.1").is_ok());
    }

    #[tokio::test]
    async fn test_failed_attempt_tracking() {
        let config = AdaptiveRateLimitConfig {
            enabled: true,
            excluded_paths: vec![],
            failed_attempts_threshold: 3,
            failed_attempts_window_secs: 300,
            threat_level_decay_secs: 600,
            max_threat_level: 10,
        };

        let limiter = AdaptiveRateLimiter::new(config);

        // Record failed attempts - need threshold (3) to trigger increase
        limiter.record_failed_attempt("192.168.1.1");
        limiter.record_failed_attempt("192.168.1.1");
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Normal);

        // Third failed attempt reaches threshold, but threat level increases
        // only when failed_count >= threshold AND exceeds by threshold amount
        // Need 3 more attempts (total 6) to get threat level increase
        limiter.record_failed_attempt("192.168.1.1");
        limiter.record_failed_attempt("192.168.1.1");
        limiter.record_failed_attempt("192.168.1.1");
        limiter.record_failed_attempt("192.168.1.1");
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Elevated);

        // Rate limit should now be 50 instead of 100
        for i in 0..50 {
            assert!(
                limiter.check_rate_limit("/api/test", "192.168.1.2").is_ok(),
                "Request {} should succeed at elevated threat level",
                i + 1
            );
        }

        // 51st request should be rate limited
        assert!(
            limiter
                .check_rate_limit("/api/test", "192.168.1.2")
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_threat_level_escalation() {
        let config = AdaptiveRateLimitConfig {
            enabled: true,
            excluded_paths: vec![],
            failed_attempts_threshold: 5,
            failed_attempts_window_secs: 300,
            threat_level_decay_secs: 600,
            max_threat_level: 10,
        };

        let limiter = AdaptiveRateLimiter::new(config);

        // ThreatLevel::from_u8: 0-2=Normal, 3-5=Elevated, 6-8=High, 9+=Critical
        // Threat increases cumulatively on each failure >= threshold
        // Formula: level += failed_count / threshold (integer division)
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Normal);

        // 5 failures: fail 5 triggers, level=0+5/5=1 (Normal)
        for _ in 0..5 {
            limiter.record_failed_attempt("192.168.1.1");
        }
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Normal); // level=1

        // 5 more (total 10): levels added: 6/5=1, 7/5=1, 8/5=1, 9/5=1, 10/5=2 = +6 -> level=7 (High)
        for _ in 0..5 {
            limiter.record_failed_attempt("192.168.1.1");
        }
        assert_eq!(limiter.get_threat_level(), ThreatLevel::High); // level=7

        // 3 more (total 13): 11/5=2, 12/5=2, 13/5=2 = +6 -> level=13->10 (Critical)
        for _ in 0..3 {
            limiter.record_failed_attempt("192.168.1.1");
        }
        assert_eq!(limiter.get_threat_level(), ThreatLevel::Critical); // level=10 (capped)
    }
}
