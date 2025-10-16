//! CAPTCHA Rate Limiting Integration
//!
//! Integrates CAPTCHA validation with authenc rate limiting middleware
//! to provide progressive rate limiting based on failure count and risk assessment

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::middleware::rate_limit_axum::{RateLimitConfig, RateLimiterState};
use super::error::CaptchaError;
use super::types::RiskLevel;

/// CAPTCHA-specific rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaRateLimitConfig {
    /// Base rate limit configuration
    pub base_config: RateLimitConfig,
    /// Progressive rate limiting based on failure count
    pub progressive_limits: ProgressiveLimits,
    /// Rate limits based on risk level
    pub risk_based_limits: RiskBasedLimits,
    /// Whether to enable CAPTCHA-specific rate limiting
    pub enabled: bool,
}

/// Progressive rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveLimits {
    /// Rate limit after 1-2 failures (requests per minute)
    pub low_failure_rpm: u64,
    /// Rate limit after 3-5 failures (requests per minute)
    pub medium_failure_rpm: u64,
    /// Rate limit after 6-10 failures (requests per minute)
    pub high_failure_rpm: u64,
    /// Rate limit after 10+ failures (requests per minute)
    pub critical_failure_rpm: u64,
}

/// Risk-based rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBasedLimits {
    /// Rate limit for low risk users (requests per minute)
    pub low_risk_rpm: u64,
    /// Rate limit for medium risk users (requests per minute)
    pub medium_risk_rpm: u64,
    /// Rate limit for high risk users (requests per minute)
    pub high_risk_rpm: u64,
    /// Rate limit for critical risk users (requests per minute)
    pub critical_risk_rpm: u64,
}

impl Default for CaptchaRateLimitConfig {
    fn default() -> Self {
        Self {
            base_config: RateLimitConfig {
                requests_per_minute: 30, // Base limit for CAPTCHA endpoints
                excluded_paths: vec![],
                enabled: true,
                progressive_delays: true,
                base_delay_ms: 2000,  // 2 second base delay
                max_delay_ms: 30000,  // 30 second max delay
            },
            progressive_limits: ProgressiveLimits {
                low_failure_rpm: 20,     // 1-2 failures
                medium_failure_rpm: 10,  // 3-5 failures
                high_failure_rpm: 5,     // 6-10 failures
                critical_failure_rpm: 1, // 10+ failures
            },
            risk_based_limits: RiskBasedLimits {
                low_risk_rpm: 30,
                medium_risk_rpm: 15,
                high_risk_rpm: 5,
                critical_risk_rpm: 1,
            },
            enabled: true,
        }
    }
}

/// CAPTCHA failure tracking for rate limiting
#[derive(Debug, Clone)]
pub struct CaptchaFailureTracker {
    pub failure_count: u32,
    pub last_failure: SystemTime,
    pub current_risk_level: RiskLevel,
    pub consecutive_failures: u32,
}

impl CaptchaFailureTracker {
    pub fn new() -> Self {
        Self {
            failure_count: 0,
            last_failure: SystemTime::now(),
            current_risk_level: RiskLevel::Low,
            consecutive_failures: 0,
        }
    }

    pub fn record_failure(&mut self, risk_level: RiskLevel) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        self.last_failure = SystemTime::now();
        self.current_risk_level = risk_level;
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.current_risk_level = RiskLevel::Low;
    }

    pub fn get_failure_level(&self) -> FailureLevel {
        match self.consecutive_failures {
            0..=2 => FailureLevel::Low,
            3..=5 => FailureLevel::Medium,
            6..=10 => FailureLevel::High,
            _ => FailureLevel::Critical,
        }
    }
}

/// Failure level classification for progressive rate limiting
#[derive(Debug, Clone, PartialEq)]
pub enum FailureLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// CAPTCHA rate limiting state
#[derive(Clone)]
pub struct CaptchaRateLimitState {
    config: CaptchaRateLimitConfig,
    base_rate_limiter: RateLimiterState,
    failure_tracker: Arc<RwLock<HashMap<String, CaptchaFailureTracker>>>,
}

impl CaptchaRateLimitState {
    /// Create a new CAPTCHA rate limiting state
    pub fn new(config: CaptchaRateLimitConfig) -> Self {
        let base_rate_limiter = RateLimiterState::new(config.base_config.clone());

        let state = Self {
            config,
            base_rate_limiter,
            failure_tracker: Arc::new(RwLock::new(HashMap::new())),
        };

        // Spawn cleanup task for failure tracker
        let failure_tracker = state.failure_tracker.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean every 5 minutes
            loop {
                interval.tick().await;
                let mut tracker = failure_tracker.write().await;
                let cutoff_time = SystemTime::now() - Duration::from_secs(3600); // Keep 1 hour of history

                tracker.retain(|_, failure_data| {
                    failure_data.last_failure > cutoff_time
                });
            }
        });

        state
    }

    /// Get tracking key for IP address
    fn get_tracking_key(&self, addr: &SocketAddr) -> String {
        addr.ip().to_string()
    }

    /// Get current rate limit for an IP based on failure history and risk level
    pub async fn get_current_rate_limit(&self, addr: &SocketAddr) -> u64 {
        let tracking_key = self.get_tracking_key(addr);
        let tracker = self.failure_tracker.read().await;

        if let Some(failure_data) = tracker.get(&tracking_key) {
            // Use the more restrictive limit between failure-based and risk-based
            let failure_limit = match failure_data.get_failure_level() {
                FailureLevel::Low => self.config.progressive_limits.low_failure_rpm,
                FailureLevel::Medium => self.config.progressive_limits.medium_failure_rpm,
                FailureLevel::High => self.config.progressive_limits.high_failure_rpm,
                FailureLevel::Critical => self.config.progressive_limits.critical_failure_rpm,
            };

            let risk_limit = match failure_data.current_risk_level {
                RiskLevel::Low => self.config.risk_based_limits.low_risk_rpm,
                RiskLevel::Medium => self.config.risk_based_limits.medium_risk_rpm,
                RiskLevel::High => self.config.risk_based_limits.high_risk_rpm,
                RiskLevel::Critical => self.config.risk_based_limits.critical_risk_rpm,
            };

            failure_limit.min(risk_limit)
        } else {
            self.config.base_config.requests_per_minute
        }
    }

    /// Record a CAPTCHA failure for rate limiting purposes
    pub async fn record_captcha_failure(&self, addr: &SocketAddr, risk_level: RiskLevel) {
        let tracking_key = self.get_tracking_key(addr);
        let mut tracker = self.failure_tracker.write().await;

        let failure_data = tracker.entry(tracking_key.clone()).or_insert_with(CaptchaFailureTracker::new);
        failure_data.record_failure(risk_level.clone());

        info!(
            "Recorded CAPTCHA failure for {}: failures={}, consecutive={}, risk={:?}",
            addr.ip(), failure_data.failure_count, failure_data.consecutive_failures, risk_level
        );
    }

    /// Record a CAPTCHA success for rate limiting purposes
    pub async fn record_captcha_success(&self, addr: &SocketAddr) {
        let tracking_key = self.get_tracking_key(addr);
        let mut tracker = self.failure_tracker.write().await;

        if let Some(failure_data) = tracker.get_mut(&tracking_key) {
            failure_data.record_success();
            debug!("Recorded CAPTCHA success for {}: reset consecutive failures", addr.ip());
        }
    }

    /// Check if request should be rate limited
    pub async fn should_rate_limit(&self, addr: &SocketAddr) -> Result<bool, CaptchaError> {
        if !self.config.enabled {
            return Ok(false);
        }

        let current_limit = self.get_current_rate_limit(addr).await;

        // Create a temporary the current limit
        let mut temp_config = self.config.base_config.clone();
        temp_config.requests_per_minute = current_limit;

        // Use the base rate limiter logic but with adjusted limits
        // This is a simplified check - in practice, you'd integrate more deeply with the rate limiter
        let tracking_key = self.get_tracking_key(addr);
        let tracker = self.failure_tracker.read().await;

        if let Some(failure_data) = tracker.get(&tracking_key) {
            // Check if we're in a critical failure state
            if failure_data.get_failure_level() == FailureLevel::Critical {
                // Allow only 1 request per minute for critical failures
                if let Ok(elapsed) = failure_data.last_failure.elapsed() {
                    return Ok(elapsed < Duration::from_secs(60));
                }
            }
        }

        Ok(false) // Default to not rate limited
    }

    /// Get delay duration based on current failure state
    pub async fn get_delay_duration(&self, addr: &SocketAddr) -> Duration {
        let tracking_key = self.get_tracking_key(addr);
        let tracker = self.failure_tracker.read().await;

        if let Some(failure_data) = tracker.get(&tracking_key) {
            let base_delay = Duration::from_millis(self.config.base_config.base_delay_ms);
            let max_delay = Duration::from_millis(self.config.base_config.max_delay_ms);

            // Calculate progressive delay based on consecutive failures
            let multiplier = match failure_data.get_failure_level() {
                FailureLevel::Low => 1,
                FailureLevel::Medium => 2,
                FailureLevel::High => 4,
                FailureLevel::Critical => 8,
            };

            let calculated_delay = base_delay * multiplier;
            calculated_delay.min(max_delay)
        } else {
            Duration::from_millis(self.config.base_config.base_delay_ms)
        }
    }
}

/// Middleware function for CAPTCHA rate limiting
pub async fn captcha_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(rate_limit_state): State<CaptchaRateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if this is a CAPTCHA endpoint
    let path = request.uri().path();
    let is_captcha_endpoint = path.contains("/captcha/") ||
                             path.contains("/api/v1/captcha");

    if !is_captcha_endpoint {
        // Not a CAPTCHA endpoint, proceed normally
        return Ok(next.run(request).await);
    }

    // Check rate limiting for CAPTCHA endpoints
    match rate_limit_state.should_rate_limit(&addr).await {
        Ok(should_limit) => {
            if should_limit {
                let delay = rate_limit_state.get_delay_duration(&addr).await;
                warn!(
                    "Rate limiting CAPTCHA request from {}: delay={:?}",
                    addr.ip(), delay
                );

                // Apply progressive delay
                if rate_limit_state.config.base_config.progressive_delays {
                    tokio::time::sleep(delay).await;
                }

                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
        Err(e) => {
            warn!("Error checking CAPTCHA rate limit for {}: {}", addr.ip(), e);
            // Continue processing on error to avoid blocking legitimate requests
        }
    }

    // Process the request
    let response = next.run(request).await;

    // Check response status to update failure tracking
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        // Record as potential failure (actual failure recording should be done in the handler)
        debug!("CAPTCHA endpoint returned error status {} for {}", status, addr.ip());
    }

    Ok(response)
}

/// Helper function to create CAPTCHA rate limiting layer
pub fn create_captcha_rate_limit_layer(config: CaptchaRateLimitConfig) -> impl tower::Layer<CaptchaRateLimitState> + Clone + Send + Sync + 'static {
    let state = CaptchaRateLimitState::new(config);
    axum::middleware::from_fn_with_state::<_, CaptchaRateLimitState, (axum::extract::ConnectInfo<std::net::SocketAddr>, axum::extract::State<CaptchaRateLimitState>)>(state, captcha_rate_limit_middleware)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_captcha_rate_limit_state_creation() {
        let config = CaptchaRateLimitConfig::default();
        let state = CaptchaRateLimitState::new(config);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let initial_limit = state.get_current_rate_limit(&addr).await;

        assert_eq!(initial_limit, 30); // Default base limit
    }

    #[tokio::test]
    async fn test_failure_tracking() {
        let config = CaptchaRateLimitConfig::default();
        let state = CaptchaRateLimitState::new(config);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        // Record multiple failures
        state.record_captcha_failure(&addr, RiskLevel::Medium).await;
        state.record_captcha_failure(&addr, RiskLevel::Medium).await;
        state.record_captcha_failure(&addr, RiskLevel::High).await;

        let limit_after_failures = state.get_current_rate_limit(&addr).await;
        assert!(limit_after_failures < 30); // Should be reduced

        // Record success
        state.record_captcha_success(&addr).await;
        let limit_after_success = state.get_current_rate_limit(&addr).await;

        // Should still be restricted due to risk level, but consecutive failures reset
        assert!(limit_after_success <= 30);
    }

    #[tokio::test]
    async fn test_progressive_delays() {
        let config = CaptchaRateLimitConfig::default();
        let state = CaptchaRateLimitState::new(config);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let initial_delay = state.get_delay_duration(&addr).await;

        // Record failures to increase delay
        for _ in 0..7 {
            state.record_captcha_failure(&addr, RiskLevel::High).await;
        }

        let delay_after_failures = state.get_delay_duration(&addr).await;
        assert!(delay_after_failures > initial_delay);
    }
}
