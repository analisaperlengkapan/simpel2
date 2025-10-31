# Task 7.4: Adaptive Rate Limiting Implementation Summary

## Overview

Implemented adaptive rate limiting middleware with threat level detection for the Authenc IAM system. The middleware automatically adjusts rate limits based on failed authentication attempts and suspicious patterns, providing defense-in-depth protection against brute force attacks.

## Implementation Details

### Files Created

1. **`src/middleware/adaptive_rate_limit.rs`** (520 lines)
   - Core adaptive rate limiting implementation
   - `ThreatLevel` enum with 4 levels (Normal, Elevated, High, Critical)
   - `AdaptiveRateLimiter` struct with threat detection logic
   - `AdaptiveRateLimitConfig` for configuration
   - Tower middleware integration for Axum
   - Background tasks for cleanup and threat decay

2. **`src/middleware/adaptive_rate_limit_integration.rs`** (140 lines)
   - Integration helpers for authentication handlers
   - `AuthResultExt` trait for automatic failed attempt recording
   - Helper functions for IP extraction and response creation
   - Rate limit response type with threat level information

3. **`docs/ADAPTIVE_RATE_LIMITING.md`** (350 lines)
   - Comprehensive documentation
   - Configuration guide
   - Usage examples
   - Integration patterns
   - Monitoring and troubleshooting

### Files Modified

1. **`src/middleware/mod.rs`**
   - Added adaptive_rate_limit module export
   - Added adaptive_rate_limit_integration module export
   - Re-exported public types and functions

2. **`src/config/mod.rs`**
   - Added `AdaptiveRateLimitConfig` import
   - Added `adaptive_rate_limit` field to `AppConfig`
   - Added default configuration in `Default` implementation

## Features Implemented

### 1. Threat Level System

Four threat levels with dynamic rate limits:
- **Normal (0-2)**: 100 requests/min - Standard operations
- **Elevated (3-5)**: 50 requests/min - Increased monitoring
- **High (6-8)**: 20 requests/min - Strict rate limiting
- **Critical (9+)**: 5 requests/min - Maximum protection

### 2. Failed Attempt Tracking

- Per-IP tracking of failed authentication attempts
- Configurable threshold for threat level escalation
- Time-windowed tracking (default: 5 minutes)
- Automatic counter reset after window expiration

### 3. Automatic Threat Level Management

- **Escalation**: Increases threat level when failed attempts exceed threshold
- **Decay**: Automatically reduces threat level after inactivity period (default: 10 minutes)
- **Global Threat Level**: Affects all requests system-wide
- **Background Task**: Runs every 60 seconds for cleanup and decay

### 4. Rate Limiting

- Per-IP request counting with 1-minute rolling window
- Dynamic rate limits based on current threat level
- Excluded paths for health checks and metrics
- Detailed response headers with threat information

### 5. Integration Helpers

- `AuthResultExt` trait for seamless integration with auth handlers
- Automatic recording of failed authentication attempts
- Support for multiple error types (AuthenticationFailed, InvalidCredentials, InvalidOtpCode)
- Helper functions for IP extraction and response creation

## Configuration

### Default Configuration

```rust
AdaptiveRateLimitConfig {
    enabled: true,
    excluded_paths: vec![
        "/health".to_string(),
        "/health/ready".to_string(),
        "/health/live".to_string(),
        "/metrics".to_string(),
    ],
    failed_attempts_threshold: 5,        // Increase threat after 5 failed attempts
    failed_attempts_window_secs: 300,    // Track attempts over 5 minutes
    threat_level_decay_secs: 600,        // Decay threat level after 10 minutes
    max_threat_level: 10,                // Maximum threat level
}
```

### Configuration Options

- `enabled`: Enable/disable adaptive rate limiting
- `excluded_paths`: Paths that bypass rate limiting
- `failed_attempts_threshold`: Failed attempts before threat escalation
- `failed_attempts_window_secs`: Time window for tracking attempts
- `threat_level_decay_secs`: Inactivity period before threat decay
- `max_threat_level`: Maximum threat level (0-10)

## Usage Examples

### Basic Middleware Setup

```rust
use authenc::middleware::{adaptive_rate_limit_layer, AdaptiveRateLimitConfig};

let config = AdaptiveRateLimitConfig::default();
let rate_limit_layer = adaptive_rate_limit_layer(config);

let app = Router::new()
    .route("/api/auth/login", post(login_handler))
    .layer(rate_limit_layer);
```

### Integration with Authentication Handler

```rust
use authenc::middleware::{AdaptiveRateLimiter, AuthResultExt, extract_ip};

async fn login_handler(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(limiter): State<Arc<AdaptiveRateLimiter>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthencError> {
    let ip = extract_ip(&ConnectInfo(addr));

    let result = authenticate_user(&request.username, &request.password).await;

    // Automatically records failed attempts
    result.record_auth_result(&limiter, &ip)?;

    Ok(Json(LoginResponse { token: result? }))
}
```

## Response Format

When rate limited, the middleware returns:

```http
HTTP/1.1 429 Too Many Requests
retry-after: 60
x-ratelimit-limit: 50
x-ratelimit-remaining: 0
x-threat-level: elevated

{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Current threat level: elevated. Limit: 50 requests/min.",
  "threat_level": "elevated",
  "limit": 50,
  "retry_after": 60
}
```

## Testing

### Unit Tests Implemented

1. **test_threat_level_conversion**: Verifies threat level conversion from numeric values
2. **test_threat_level_rate_limits**: Validates rate limits for each threat level
3. **test_adaptive_rate_limiting**: Tests basic rate limiting at normal threat level
4. **test_failed_attempt_tracking**: Verifies failed attempt tracking and threat escalation
5. **test_threat_level_escalation**: Tests escalation through all threat levels
6. **test_auth_result_ext**: Tests the AuthResultExt trait integration
7. **test_create_rate_limit_response**: Validates rate limit response creation

### Test Coverage

- Threat level conversion and rate limits
- Per-IP rate limiting
- Failed attempt tracking
- Threat level escalation and decay
- Integration helpers
- Response formatting

## Logging

The middleware provides detailed logging:

```
WARN authenc::middleware::adaptive_rate_limit: Threat level increased due to failed attempts
  ip=192.168.1.100
  failed_count=5
  old_level=0
  new_level=3
  old_threat=normal
  new_threat=elevated
  old_limit=100
  new_limit=50

WARN authenc::middleware::adaptive_rate_limit: Rate limit exceeded
  ip=192.168.1.100
  path=/api/auth/login
  count=51
  limit=50
  threat_level=elevated

INFO authenc::middleware::adaptive_rate_limit: Threat level decayed due to inactivity
  old_level=3
  new_level=2
```

## Requirements Fulfilled

### Requirement 3.4: Security Hardening

✅ **"THE Authenc SHALL mengimplementasikan rate limiting per-endpoint dengan adaptive thresholds"**

The implementation provides:
- Per-endpoint rate limiting via middleware
- Adaptive thresholds based on threat levels (100 → 50 → 20 → 5 requests/min)
- Automatic adjustment based on failed authentication attempts
- Dynamic threat level management

### Task 7.4 Subtasks

✅ **Create AdaptiveRateLimiter struct with threat level tracking**
- Implemented with `ThreatLevel` enum and `AdaptiveRateLimiter` struct
- Tracks global threat level and per-IP failed attempts

✅ **Implement threat level detection based on failed attempts**
- Failed attempt tracking per IP with configurable threshold
- Automatic threat level escalation when threshold exceeded
- Time-windowed tracking with automatic reset

✅ **Add dynamic rate limit adjustment (normal: 100/min, elevated: 50/min, high: 20/min, critical: 5/min)**
- Four threat levels with specified rate limits
- Dynamic adjustment based on current threat level
- Per-IP request counting with rolling window

✅ **Integrate with existing rate_limit_axum middleware**
- Tower middleware implementation compatible with Axum
- Integration helpers for authentication handlers
- Configuration added to AppConfig
- Module exports in middleware/mod.rs

## Architecture

### Components

1. **AdaptiveRateLimiter**: Core rate limiting logic
   - Threat level management
   - Failed attempt tracking
   - Rate limit checking
   - Background cleanup and decay

2. **AdaptiveRateLimitLayer**: Tower layer for Axum integration
   - Middleware wrapping
   - Request/response handling

3. **AdaptiveRateLimitMiddleware**: Tower service implementation
   - Request interception
   - Rate limit enforcement
   - Response generation

4. **Integration Helpers**: Authentication handler integration
   - AuthResultExt trait
   - IP extraction utilities
   - Response formatting

### Data Structures

- `DashMap<String, RateLimitEntry>`: Concurrent per-IP tracking
- `AtomicU8`: Lock-free global threat level
- `Mutex<Instant>`: Last threat update timestamp
- Background tokio tasks for cleanup and decay

## Performance Considerations

### Memory Usage

- Pre-allocated DashMap with 10,000 entry capacity
- Automatic cleanup of old entries every 60 seconds
- Per-IP tracking with minimal overhead

### Concurrency

- Lock-free atomic operations for threat level
- Concurrent DashMap for per-IP tracking
- Background tasks don't block request processing

### Scalability

- O(1) rate limit checks
- Efficient per-IP tracking
- Automatic cleanup prevents memory growth

## Security Benefits

1. **Brute Force Protection**: Automatically increases protection during attacks
2. **Adaptive Response**: Adjusts to threat level without manual intervention
3. **Defense in Depth**: Complements other security measures (MFA, account lockout)
4. **Attack Detection**: Identifies and responds to suspicious patterns
5. **Automatic Recovery**: Threat levels decay when attacks subside

## Future Enhancements

Potential improvements:
- Per-IP threat levels (in addition to global)
- Integration with external threat intelligence
- Machine learning-based anomaly detection
- Distributed rate limiting across instances
- Configurable rate limits per threat level
- Geographic-based rate limiting
- Prometheus metrics integration

## Deployment Notes

### Prerequisites

- Rust 1.90+ with edition 2024
- Axum 0.8.6+
- Tower middleware support
- tokio async runtime

### Configuration

Add to `config/authenc.production.toml`:

```toml
[adaptive_rate_limit]
enabled = true
failed_attempts_threshold = 5
failed_attempts_window_secs = 300
threat_level_decay_secs = 600
max_threat_level = 10
excluded_paths = ["/health", "/health/ready", "/health/live", "/metrics"]
```

### Monitoring

Monitor these metrics:
- Current threat level
- Failed attempts per IP
- Rate limit violations
- Request counts per threat level

Set up alerts for:
- Threat level >= Elevated
- High rate of failed attempts
- Sustained high threat levels

## Conclusion

The adaptive rate limiting implementation provides robust, automatic protection against brute force attacks and abuse. It integrates seamlessly with the existing Authenc middleware stack and requires minimal configuration. The threat level system automatically adjusts protection based on detected suspicious activity, providing defense-in-depth without impacting legitimate users during normal operations.

## References

- Design Document: `.kiro/specs/authenc-comprehensive-optimization/design.md` (lines 1005-1030)
- Requirements: `.kiro/specs/authenc-comprehensive-optimization/requirements.md` (Requirement 3.4)
- Documentation: `infra/authenc/docs/ADAPTIVE_RATE_LIMITING.md`

