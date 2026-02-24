# Security Implementation Guide - SIMPEL

**Date:** February 10, 2026
**Version:** 1.0
**Status:** Implementation Guide

## Overview

This document provides step-by-step instructions for implementing the security fixes identified in the Security Audit Report. All backend services must implement these security measures before production deployment.

---

## 1. Rate Limiting Implementation

### Backend Services (Axum)

Add rate limiting to all backend services:

```rust
use lib_common::middleware::{RateLimiter, rate_limit_middleware};
use std::sync::Arc;
use std::time::Duration;

// In main.rs or app initialization
#[tokio::main]
async fn main() {
    // Create rate limiter: 100 requests per minute per IP
    let rate_limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(60)));

    // Clone for cleanup task
    let limiter_clone = rate_limiter.clone();

    // Spawn cleanup task (runs every 5 minutes)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            limiter_clone.cleanup().await;
        }
    });

    // Create router with rate limiting
    let app = Router::new()
        .route("/api/v1/users", get(list_users))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .with_state(app_state);

    // Start server...
}
```

### Configuration

Add to service configuration:

```toml
[rate_limit]
enabled = true
max_requests_per_minute = 100
window_seconds = 60
```

### Testing

```bash
# Test rate limiting
for i in {1..110}; do
    curl -w "%{http_code}\n" http://localhost:3000/api/v1/users
done

# Expected: First 100 return 200, remaining return 429
```

---

## 2. Security Headers Implementation

### Backend Services (Axum)

Add security headers middleware to all services:

```rust
use lib_common::middleware::security_headers_middleware;

// In router setup
let app = Router::new()
    .route("/api/v1/users", get(list_users))
    .layer(axum::middleware::from_fn(security_headers_middleware))
    .with_state(app_state);
```

### Verify Headers

```bash
curl -I http://localhost:3000/api/v1/users

# Expected headers:
# Strict-Transport-Security: max-age=31536000; includeSubDomains
# X-Frame-Options: DENY
# X-Content-Type-Options: nosniff
# X-XSS-Protection: 1; mode=block
# Referrer-Policy: strict-origin-when-cross-origin
# Content-Security-Policy: default-src 'self'; ...
# Permissions-Policy: geolocation=(), microphone=(), camera=()
```

---

## 3. CSRF Protection Implementation

### Backend Services (Axum)

Add CSRF validation middleware:

```rust
use lib_common::middleware::csrf_validation_middleware;

// In router setup
let app = Router::new()
    .route("/api/v1/users", post(create_user).put(update_user))
    .layer(axum::middleware::from_fn(csrf_validation_middleware))
    .with_state(app_state);
```

### Frontend Integration

The frontend already implements CSRF tokens. Ensure backend validates them:

```rust
// CSRF token is sent in X-CSRF-Token header
// Cookie csrf_token must match header value
```

### Testing

```bash
# Test CSRF protection
curl -X POST http://localhost:3000/api/v1/users \
  -H "Content-Type: application/json" \
  -H "X-CSRF-Token: invalid_token" \
  -d '{"username":"test"}'

# Expected: 403 Forbidden
```

---

## 4. Input Validation Implementation

### Backend Services (Axum)

Add input validation middleware:

```rust
use lib_common::middleware::input_validation_middleware;

// In router setup
let app = Router::new()
    .route("/api/v1/users", post(create_user))
    .layer(axum::middleware::from_fn(input_validation_middleware))
    .with_state(app_state);
```

### Request Handler Validation

Add validation to all request handlers:

```rust
use lib_common::validation::{
    validate_email, validate_username, validate_length,
    ValidationErrors, ValidationError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Validate username
        if !validate_username(&self.username) {
            errors.add(ValidationError::new(
                "username",
                "Username must be 3-50 alphanumeric characters with _ or -",
            ));
        }

        // Validate email
        if !validate_email(&self.email) {
            errors.add(ValidationError::new(
                "email",
                "Invalid email format",
            ));
        }

        // Validate password length
        if let Err(e) = validate_length(&self.password, Some(8), Some(128), "password") {
            errors.add(ValidationError::new("password", e.to_string()));
        }

        errors.to_result()
    }
}

// In handler
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    // Validate request
    request.validate()
        .map_err(|e| AppError::ValidationError(e))?;

    // Process request...
}
```

---

## 5. XSS Prevention Implementation

### Frontend Components

All components using `inner_html` must sanitize content:

```rust
use lib_ui::utils::security::sanitize_html;

#[component]
pub fn MyComponent(html_content: String) -> impl IntoView {
    // Sanitize HTML before rendering
    let sanitized = sanitize_html(&html_content);

    view! {
        <div inner_html=sanitized></div>
    }
}
```

### Audit All inner_html Usage

Run this command to find all `inner_html` usage:

```bash
grep -r "inner_html" antarmuka/ --include="*.rs"
```

For each occurrence, verify:
1. Content is from trusted source (hardcoded icons, etc.), OR
2. Content is sanitized using `sanitize_html()`

### SafeHtml Component

Use the SafeHtml component for user-generated content:

```rust
use lib_ui::utils::security::SafeHtml;

view! {
    <SafeHtml
        html=user_generated_content
        class="user-content"
    />
}
```

---

## 6. SQL Injection Prevention

### Verification

The codebase already uses parameterized queries. Verify no string concatenation:

```bash
# Search for potential SQL injection patterns
grep -r "format!" layanan/ --include="*.rs" | grep -i "select\|insert\|update\|delete"
```

### Best Practices

Always use parameterized queries:

```rust
// ✅ GOOD - Parameterized query
let user = client
    .query_one(
        "SELECT * FROM users WHERE username = $1",
        &[&username],
    )
    .await?;

// ❌ BAD - String concatenation (DO NOT DO THIS)
let query = format!("SELECT * FROM users WHERE username = '{}'", username);
let user = client.query_one(&query, &[]).await?;
```

---

## 7. Dependency Security

### Regular Audits

Add to CI/CD pipeline:

```yaml
# .gitlab-ci.yml or .github/workflows/security.yml
security-audit:
  script:
    - cargo install cargo-audit
    - cargo audit
    - cargo install cargo-deny
    - cargo deny check
  only:
    - main
    - merge_requests
```

### Manual Audit

```bash
# Install tools
cargo install cargo-audit cargo-deny

# Run security audit
cargo audit

# Check licenses and advisories
cargo deny check
```

### Update Dependencies

```bash
# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated

# Update specific dependency
cargo update -p <package_name>
```

---

## 8. Token Storage Security

### Current Implementation

Tokens are stored in localStorage (vulnerable to XSS).

### Recommended Implementation

Migrate to httpOnly cookies:

```rust
// Backend: Set httpOnly cookie
use axum::http::{header, HeaderValue};

async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, AppError> {
    // Authenticate user...
    let access_token = generate_jwt(&user)?;

    // Set httpOnly cookie
    let cookie = format!(
        "access_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=900",
        access_token
    );

    let mut response = Response::new(Body::empty());
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)?,
    );

    Ok(response)
}
```

```rust
// Frontend: Remove localStorage usage
// Tokens are automatically sent in cookies
// No need to manually include in requests
```

---

## 9. SameSite Cookie Attribute

### Implementation

All authentication cookies must use SameSite=Strict:

```rust
// In cookie setting
let cookie = format!(
    "{}={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
    name, value, max_age
);
```

### Verification

```bash
# Check cookie attributes
curl -I http://localhost:3000/api/auth/login

# Expected Set-Cookie header:
# Set-Cookie: access_token=...; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=900
```

---

## 10. Audit Logging

### Implementation

Ensure all critical operations are logged:

```rust
use lib_common::audit::{AuditLogger, AuditEvent};

// In handler
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Delete user...

    // Log audit event
    state.audit_logger.log(
        AuditEvent::UserDeleted {
            user_id,
            deleted_by: current_user.id,
        },
        current_user.id,
        client_ip,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}
```

### Critical Operations to Log

- User authentication (success/failure)
- User creation/update/deletion
- Role/permission changes
- Workflow state transitions
- Batch operations
- Secret access (via Secreton)
- Configuration changes

---

## 11. Testing Security Fixes

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        // First 5 requests succeed
        for _ in 0..5 {
            assert!(limiter.check("test_ip").await);
        }

        // 6th request fails
        assert!(!limiter.check("test_ip").await);
    }

    #[test]
    fn test_xss_sanitization() {
        let input = "<script>alert('xss')</script><p>Hello</p>";
        let output = sanitize_html(input);
        assert!(!output.contains("<script"));
        assert!(output.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_csrf_validation() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
    }
}
```

### Integration Tests

```bash
# Test security headers
./tests/security/test_headers.sh

# Test rate limiting
./tests/security/test_rate_limit.sh

# Test CSRF protection
./tests/security/test_csrf.sh

# Test input validation
./tests/security/test_validation.sh
```

---

## 12. Deployment Checklist

Before deploying to production:

- [ ] Rate limiting implemented in all services
- [ ] Security headers added to all responses
- [ ] CSRF validation enabled for state-changing operations
- [ ] Input validation added to all request handlers
- [ ] All `inner_html` usage audited and sanitized
- [ ] SQL queries use parameterized statements
- [ ] Dependencies audited with `cargo audit`
- [ ] Security patches applied
- [ ] Token storage migrated to httpOnly cookies
- [ ] SameSite=Strict set for all auth cookies
- [ ] Audit logging implemented for critical operations
- [ ] Security tests passing
- [ ] Penetration testing completed
- [ ] Security documentation updated

---

## 13. Monitoring and Alerting

### Metrics to Monitor

```rust
// Add Prometheus metrics
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref RATE_LIMIT_EXCEEDED: Counter = Counter::new(
        "rate_limit_exceeded_total",
        "Total number of rate limit violations"
    ).unwrap();

    static ref CSRF_VALIDATION_FAILED: Counter = Counter::new(
        "csrf_validation_failed_total",
        "Total number of CSRF validation failures"
    ).unwrap();

    static ref XSS_DETECTED: Counter = Counter::new(
        "xss_detected_total",
        "Total number of XSS attempts detected"
    ).unwrap();
}

// In middleware
if !limiter.check(&client_ip).await {
    RATE_LIMIT_EXCEEDED.inc();
    return Err(StatusCode::TOO_MANY_REQUESTS);
}
```

### Alerts

Configure alerts for:
- High rate of rate limit violations
- CSRF validation failures
- XSS detection
- Failed authentication attempts
- Unusual API usage patterns

---

## 14. Security Training

### Developer Training Topics

1. OWASP Top 10 vulnerabilities
2. Secure coding practices in Rust
3. Input validation and sanitization
4. Authentication and authorization
5. Cryptography best practices
6. Secure API design
7. Security testing

### Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Secure Coding in Rust](https://doc.rust-lang.org/nomicon/)

---

## 15. Incident Response

### Security Incident Procedure

1. **Detect**: Monitor logs and alerts
2. **Contain**: Isolate affected systems
3. **Investigate**: Analyze logs and audit trail
4. **Remediate**: Apply fixes and patches
5. **Document**: Record incident details
6. **Review**: Post-mortem analysis

### Contact Information

- Security Team: security@kejaksaan.go.id
- On-Call Engineer: +62-XXX-XXXX-XXXX
- Incident Response: incident@kejaksaan.go.id

---

## Conclusion

Implementing these security measures will bring SIMPEL into compliance with all security requirements (NFR-S001 through NFR-S008). Regular security audits and continuous monitoring are essential to maintain a strong security posture.

**Next Steps:**
1. Implement high-priority fixes (rate limiting, security headers, CSRF validation)
2. Run security tests
3. Update dependencies
4. Conduct penetration testing
5. Deploy to production

**Estimated Implementation Time:** 2-3 weeks

---

**Document Version:** 1.0
**Last Updated:** February 10, 2026
**Next Review:** May 10, 2026
