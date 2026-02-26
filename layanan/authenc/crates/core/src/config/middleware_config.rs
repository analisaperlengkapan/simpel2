//! Middleware configuration types
//!
//! Pure configuration structs for security middleware. These types are
//! defined here (in the core config layer) so that the API/middleware layer
//! can reference a common configuration contract without circular dependencies.

use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per minute
    pub requests_per_minute: u64,
    /// Path patterns to exclude from rate limiting
    pub excluded_paths: Vec<String>,
    /// Whether to enable rate limiting
    pub enabled: bool,
    /// Whether to enable progressive delays for rate-limited requests
    pub progressive_delays: bool,
    /// Base delay in milliseconds for rate-limited requests
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds for rate-limited requests
    pub max_delay_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            excluded_paths: vec![
                "/health".to_string(),
                "/health/ready".to_string(),
                "/health/live".to_string(),
                "/metrics".to_string(),
            ],
            enabled: true,
            progressive_delays: true,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
        }
    }
}

/// Adaptive rate limiting configuration
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
            failed_attempts_window_secs: 300,
            threat_level_decay_secs: 600,
            max_threat_level: 10,
        }
    }
}

/// MFA-specific rate limiting configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MfaRateLimitConfig {
    /// Maximum MFA verification attempts per minute per IP
    pub max_attempts_per_minute_per_ip: u32,
    /// Maximum MFA verification attempts per minute per user
    pub max_attempts_per_minute_per_user: u32,
    /// Maximum MFA setup attempts per hour per IP
    pub max_setup_attempts_per_hour_per_ip: u32,
    /// Progressive delay base in milliseconds
    pub progressive_delay_base_ms: u64,
    /// Maximum progressive delay in milliseconds
    pub progressive_delay_max_ms: u64,
    /// Account lockout threshold (failed attempts)
    pub account_lockout_threshold: u32,
    /// Account lockout duration in minutes
    pub account_lockout_duration_minutes: u32,
    /// Whether to enable progressive delays
    pub enable_progressive_delays: bool,
    /// Whether to enable account lockout
    pub enable_account_lockout: bool,
}

impl Default for MfaRateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_minute_per_ip: 10,
            max_attempts_per_minute_per_user: 5,
            max_setup_attempts_per_hour_per_ip: 3,
            progressive_delay_base_ms: 2000,
            progressive_delay_max_ms: 30000,
            account_lockout_threshold: 5,
            account_lockout_duration_minutes: 15,
            enable_progressive_delays: true,
            enable_account_lockout: true,
        }
    }
}

/// CSRF protection configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsrfConfig {
    /// Whether CSRF protection is enabled
    pub enabled: bool,
    /// Name of the CSRF token header
    pub header_name: String,
    /// Name of the CSRF token cookie
    pub cookie_name: String,
    /// CSRF token length in bytes
    pub token_length: usize,
    /// Paths to exclude from CSRF protection
    pub excluded_paths: Vec<String>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header_name: "X-CSRF-Token".to_string(),
            cookie_name: "csrf_token".to_string(),
            token_length: 32,
            excluded_paths: vec![
                "/health".to_string(),
                "/health/ready".to_string(),
                "/health/live".to_string(),
                "/metrics".to_string(),
            ],
        }
    }
}

/// Input validation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputValidationConfig {
    /// Whether to enable input validation
    pub enabled: bool,
    /// Maximum length for query parameters
    pub max_query_param_length: usize,
    /// Maximum length for headers
    pub max_header_length: usize,
    /// Maximum request body size in bytes
    pub max_request_body_size: usize,
    /// Whether to block suspicious patterns
    pub block_suspicious_patterns: bool,
    /// Whether to validate content type headers
    pub validate_content_type: bool,
    /// Allowed content types for requests with bodies
    pub allowed_content_types: Vec<String>,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_query_param_length: 2048,
            max_header_length: 4096,
            max_request_body_size: 1024 * 1024,
            block_suspicious_patterns: true,
            validate_content_type: true,
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
                "multipart/form-data".to_string(),
                "text/plain".to_string(),
            ],
        }
    }
}

/// Security monitoring configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    /// Whether security monitoring is enabled
    pub enabled: bool,
    /// Threshold for suspicious activity alerts (requests per minute)
    pub suspicious_threshold_rpm: u64,
    /// Paths to monitor for security events
    pub monitored_paths: Vec<String>,
    /// Whether to log all authentication attempts
    pub log_auth_attempts: bool,
    /// Whether to log all authorization failures
    pub log_authz_failures: bool,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            suspicious_threshold_rpm: 100,
            monitored_paths: vec![
                "/auth/login".to_string(),
                "/auth/register".to_string(),
                "/account".to_string(),
                "/admin".to_string(),
            ],
            log_auth_attempts: true,
            log_authz_failures: true,
        }
    }
}
