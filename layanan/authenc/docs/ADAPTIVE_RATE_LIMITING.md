# Adaptive Rate Limiting

## Overview

The adaptive rate limiting middleware provides dynamic rate limiting based on detected threat levels. It automatically adjusts rate limits in response to failed authentication attempts and suspicious patterns, providing defense-in-depth protection against brute force attacks and abuse.

## Features

- **Threat Level Detection**: Automatically detects and responds to suspicious activity
- **Dynamic Rate Limits**: Adjusts limits based on current threat level
- **Failed Attempt Tracking**: Records and tracks failed authentication attempts per IP
- **Automatic Decay**: Threat levels automatically decay over time when no suspicious activity is detected
- **Per-IP Tracking**: Maintains separate rate limits and threat tracking for each IP address

## Threat Levels

The system supports four threat levels, each with different rate limits:

| Threat Level | Numeric Range | Rate Limit (req/min) | Description |
|--------------|---------------|----------------------|-------------|
| Normal       | 0-2           | 100                  | Standard operations |
| Elevated     | 3-5           | 50                   | Increased monitoring |
| High         | 6-8           | 20                   | Strict rate limiting |
| Critical     | 9+            | 5                    | Maximum protection |

## Configuration

```rust
use authenc::middleware::AdaptiveRateLimitConfig;

let config = AdaptiveRateLimitConfig {
    enabled: true,
    excluded_paths: vec![
        "/health".to_string(),
        "/metrics".to_string(),
    ],
    failed_attempts_threshold: 5,        // Increase threat after 5 failed attempts
    failed_attempts_window_secs: 300,    // Track attempts over 5 minutes
    threat_level_decay_secs: 600,        // Decay threat level after 10 minutes
    max_threat_level: 10,                // Maximum threat level
};
```

### Configuration Options

- **enabled**: Enable or disable adaptive rate limiting
- **excluded_paths**: Paths that bypass rate limiting (e.g., health checks)
- **failed_attempts_threshold**: Number of failed attempts before increasing threat level
- **failed_attempts_window_secs**: Time window for tracking failed attempts
- **threat_level_decay_secs**: How long to wait before decaying threat level
- **max_threat_level**: Maximum threat level (0-10)

## Usage

### Basic Setup

Add the adaptive rate limiter to your Axum application:

```rust
use axum::Router;
use authenc::middleware::{adaptive_rate_limit_layer, AdaptiveRateLimitConfig};

let config = AdaptiveRateLimitConfig::default();
let rate_limit_layer = adaptive_rate_limit_layer(config);

let app = Router::new()
    .route("/api/auth/login", post(login_handler))
    .layer(rate_limit_layer);
```

### Integration with Authentication Handlers

Use the `AuthResultExt` trait to automatically record failed authentication attempts:

```rust
use axum::{extract::{State, ConnectInfo}, Json};
use authenc::middleware::{AdaptiveRateLimiter, AuthResultExt, extract_ip};
use std::sync::Arc;

async fn login_handler(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(limiter): State<Arc<AdaptiveRateLimiter>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthencError> {
    let ip = extract_ip(&ConnectInfo(addr));

    // Authenticate user
    let result = authenticate_user(&request.username, &request.password).await;

    // Record the result with the adaptive rate limiter
    result.record_auth_result(&limiter, &ip)?;

    Ok(Json(LoginResponse { token: result? }))
}
```

### Manual Failed Attempt Recording

You can also manually record failed attempts:

```rust
use authenc::middleware::AdaptiveRateLimiter;

async fn handle_failed_login(limiter: &AdaptiveRateLimiter, ip: &str) {
    limiter.record_failed_attempt(ip);
}
```

### Checking Current Threat Level

```rust
use authenc::middleware::{AdaptiveRateLimiter, ThreatLevel};

fn check_threat_level(limiter: &AdaptiveRateLimiter) {
    let threat_level = limiter.get_threat_level();

    match threat_level {
        ThreatLevel::Normal => println!("System operating normally"),
        ThreatLevel::Elevated => println!("Elevated threat detected"),
        ThreatLevel::High => println!("High threat - strict rate limiting active"),
        ThreatLevel::Critical => println!("Critical threat - maximum protection"),
    }

    let limit = threat_level.rate_limit();
    println!("Current rate limit: {} requests/min", limit);
}
```

## How It Works

### Threat Level Escalation

1. **Failed Attempt Tracking**: Each failed authentication attempt is recorded per IP address
2. **Threshold Detection**: When failed attempts exceed the configured threshold, the global threat level increases
3. **Rate Limit Adjustment**: Higher threat levels result in stricter rate limits for all requests
4. **Automatic Decay**: If no suspicious activity is detected for the configured decay period, the threat level automatically decreases

### Rate Limiting

1. **Per-IP Tracking**: Each IP address has its own request counter
2. **Rolling Window**: Counters reset every minute
3. **Dynamic Limits**: The rate limit applied depends on the current threat level
4. **Excluded Paths**: Health checks and metrics endpoints bypass rate limiting

### Background Tasks

The adaptive rate limiter spawns background tasks for:

- **Cleanup**: Removes old rate limit entries every 60 seconds
- **Threat Decay**: Automatically reduces threat level when no recent suspicious activity

## Response Headers

When a request is rate limited, the following headers are included in the response:

- `retry-after`: Seconds to wait before retrying (60)
- `x-ratelimit-limit`: Current rate limit (requests per minute)
- `x-ratelimit-remaining`: Remaining requests (0 when rate limited)
- `x-threat-level`: Current threat level name (normal, elevated, high, critical)

## Example Response

```http
HTTP/1.1 429 Too Many Requests
retry-after: 60
x-ratelimit-limit: 50
x-ratelimit-remaining: 0
x-threat-level: elevated
Content-Type: application/json

{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Current threat level: elevated. Limit: 50 requests/min.",
  "threat_level": "elevated",
  "limit": 50,
  "retry_after": 60
}
```

## Integration with AppConfig

Add adaptive rate limiting to your application configuration:

```rust
use authenc::config::AppConfig;
use authenc::middleware::AdaptiveRateLimitConfig;

let mut config = AppConfig::default();
config.adaptive_rate_limit = AdaptiveRateLimitConfig {
    enabled: true,
    failed_attempts_threshold: 5,
    failed_attempts_window_secs: 300,
    threat_level_decay_secs: 600,
    max_threat_level: 10,
    excluded_paths: vec![
        "/health".to_string(),
        "/health/ready".to_string(),
        "/health/live".to_string(),
        "/metrics".to_string(),
    ],
};
```

## Monitoring

### Metrics

The adaptive rate limiter can be monitored through:

- Current threat level
- Failed attempts per IP
- Rate limit violations
- Request counts per threat level

### Logging

The middleware logs important events:

- Failed authentication attempts
- Threat level changes
- Rate limit violations

Example log entries:

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
```

## Best Practices

1. **Configure Appropriate Thresholds**: Set `failed_attempts_threshold` based on your security requirements and user behavior
2. **Monitor Threat Levels**: Set up alerts for elevated threat levels
3. **Exclude Health Checks**: Always exclude health check and metrics endpoints from rate limiting
4. **Combine with Other Security Measures**: Use adaptive rate limiting alongside other security features like MFA, CAPTCHA, and account lockout
5. **Test Thoroughly**: Test the rate limiting behavior under various load conditions
6. **Adjust Decay Time**: Configure `threat_level_decay_secs` based on your threat model

## Troubleshooting

### High False Positive Rate

If legitimate users are being rate limited:

- Increase `failed_attempts_threshold`
- Increase rate limits for each threat level
- Reduce `threat_level_decay_secs` for faster recovery

### Threat Level Not Increasing

If the threat level isn't increasing despite attacks:

- Verify `failed_attempts_threshold` is not too high
- Check that failed attempts are being recorded correctly
- Ensure the middleware is properly integrated with authentication handlers

### Performance Issues

If the rate limiter is causing performance problems:

- Reduce the size of the rate limit cache
- Increase cleanup interval
- Consider using a distributed cache for multi-instance deployments

## Security Considerations

- **IP Spoofing**: The rate limiter uses client IP addresses, which can be spoofed. Use behind a trusted reverse proxy.
- **Distributed Attacks**: The current implementation tracks threat levels globally. Consider per-IP threat levels for distributed attacks.
- **Resource Exhaustion**: The rate limiter maintains state per IP. Monitor memory usage in high-traffic scenarios.

## Future Enhancements

Potential improvements for future versions:

- Per-IP threat levels (in addition to global)
- Integration with external threat intelligence
- Machine learning-based anomaly detection
- Distributed rate limiting across multiple instances
- Configurable rate limits per threat level
- Geographic-based rate limiting
