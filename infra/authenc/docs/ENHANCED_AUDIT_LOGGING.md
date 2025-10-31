# Enhanced Audit Logging

## Overview

The enhanced audit logging system provides comprehensive context capture for all audit events, including:

- **IP Address**: Extracted from X-Forwarded-For, X-Real-IP, CF-Connecting-IP, or True-Client-IP headers
- **User Agent**: Browser and client information
- **Geolocation**: Optional geographic location data based on IP address
- **Request/Response Payloads**: Sanitized payloads with PII removed
- **Session ID**: For correlating events across a user session
- **Request ID**: For distributed tracing and correlation

## Architecture

### Components

1. **Request Context Extraction** (`utils/request_context.rs`)
   - Extracts IP address from various proxy headers
   - Extracts user agent string
   - Extracts request/correlation IDs

2. **Geolocation Service** (`utils/geolocation.rs`)
   - Optional IP-to-location lookup
   - Extensible design for integration with MaxMind, IP2Location, etc.
   - Default implementation identifies private/local IPs

3. **Payload Sanitizer** (`utils/payload_sanitizer.rs`)
   - Removes sensitive fields (passwords, tokens, keys)
   - Masks PII fields (email, phone, address)
   - Configurable field lists
   - Size limits to prevent log bloat

4. **Enhanced Audit Service** (`services/enhanced_audit.rs`)
   - Orchestrates context capture
   - Enriches events with contextual data
   - Stores events with tamper-proof signatures

## Usage

### Basic Usage

```rust
use crate::services::enhanced_audit::{EnhancedAuditService, create_audit_context};
use crate::models::events::{Event, EventType};
use axum::http::HeaderMap;

// In your handler
async fn my_handler(
    State(audit_service): State<Arc<EnhancedAuditService>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    // Create audit context from headers
    let audit_context = audit_service.create_context(&headers).await;

    // Create your event
    let event = Event {
        id: Uuid::new_v4().to_string(),
        time: Utc::now(),
        event_type: EventType::Login,
        // ... other fields
    };

    // Log with enhanced context
    audit_service.log_user_event(event, &audit_context).await?;

    Ok(StatusCode::OK)
}
```

### With Request/Response Payloads

```rust
// Create context
let mut audit_context = audit_service.create_context(&headers).await;

// Add sanitized request payload
let request_json = json!({
    "username": request.username,
    "password": request.password,  // Will be redacted
    "email": request.email,         // Will be masked
});
audit_context = audit_context.with_request_payload(
    request_json,
    audit_service.sanitizer_config(),
);

// Add sanitized response payload
let response_json = serde_json::to_value(&response)?;
audit_context = audit_context.with_response_payload(
    response_json,
    audit_service.sanitizer_config(),
);

// Log event
audit_service.log_user_event(event, &audit_context).await?;
```

### With Session Correlation

```rust
// Add session ID for correlation
let audit_context = audit_context.with_session_id(session_id.to_string());

// All events with the same session ID can be correlated
audit_service.log_user_event(event, &audit_context).await?;
```

### Simplified Helper

For simple cases without payloads:

```rust
use crate::services::enhanced_audit::create_audit_context;

let audit_context = create_audit_context(&headers, Some(session_id)).await;

// Context now contains IP, user agent, request ID, and session ID
let ip = audit_context.request_context.ip_address;
let user_agent = audit_context.request_context.user_agent;
```

## Configuration

### Sanitizer Configuration

```rust
use crate::utils::payload_sanitizer::SanitizerConfig;
use std::collections::HashSet;

let mut config = SanitizerConfig::default();

// Add custom sensitive fields
config.additional_sensitive_fields.insert("custom_secret".to_string());

// Add custom maskable fields
config.additional_maskable_fields.insert("custom_pii".to_string());

// Set maximum payload size (bytes)
config.max_payload_size = 20_000;

// Enable/disable PII masking
config.mask_pii = true;
```

### Geolocation Service

The default implementation uses `SimpleGeolocationService` which identifies private/local IPs. For production, integrate with external services:

```rust
// Future: MaxMind GeoIP2 integration
use maxminddb::Reader;

let reader = Reader::open_readfile("GeoLite2-City.mmdb")?;
let geo_service = Arc::new(MaxMindGeolocationService::new(reader));

let audit_service = EnhancedAuditService::new(
    db,
    signature_service,
    Some(geo_service),
    None,
);
```

## Database Schema

The migration `029_enhanced_audit_details.sql` adds the following columns:

### event_log table
- `geolocation_data` (JSONB): Geographic location data
- `request_payload` (JSONB): Sanitized request payload
- `response_payload` (JSONB): Sanitized response payload
- `user_agent` (TEXT): User agent string
- `ip_address` (INET): Client IP address
- `session_id` (UUID): Session correlation ID
- `correlation_id` (UUID): Request correlation ID

### admin_audit_log table
- Same enhanced fields as event_log

### comprehensive_audit_trail view
Unified view combining user and admin events with all enhanced context.

## Data Sanitization

### Sensitive Fields (Completely Removed)

The following fields are replaced with `[REDACTED]`:
- password, password_hash
- secret, secret_key, api_key
- access_token, refresh_token, token
- authorization, auth_token, bearer
- private_key, client_secret
- mfa_secret, totp_secret
- backup_codes, recovery_codes
- credit_card, card_number, cvv
- ssn, social_security

### Maskable Fields (Partially Shown)

The following fields are masked to show partial data:
- email: `u***@example.com`
- phone: `+***`
- address: `1***`
- ip_address: `203.0.***`

### Custom Fields

Add custom sensitive or maskable fields via `SanitizerConfig`:

```rust
config.additional_sensitive_fields.insert("internal_token".to_string());
config.additional_maskable_fields.insert("employee_id".to_string());
```

## Geolocation Data Format

```json
{
  "country_code": "ID",
  "country_name": "Indonesia",
  "city": "Jakarta",
  "region": "Jakarta",
  "latitude": -6.2088,
  "longitude": 106.8456,
  "timezone": "Asia/Jakarta"
}
```

For private/local IPs:
```json
{
  "country_code": "XX",
  "country_name": "Private Network",
  "city": "Local",
  "region": null,
  "latitude": null,
  "longitude": null,
  "timezone": null
}
```

## Querying Enhanced Audit Logs

### SQL Examples

```sql
-- Find all events from a specific IP
SELECT * FROM event_log
WHERE ip_address = '203.0.113.1'::inet;

-- Find events with geolocation data
SELECT
    id,
    event_type,
    ip_address,
    geolocation_data->>'country_name' as country,
    geolocation_data->>'city' as city
FROM event_log
WHERE geolocation_data IS NOT NULL;

-- Find events by session
SELECT * FROM event_log
WHERE session_id = 'session-uuid'
ORDER BY created_at;

-- Find events by correlation ID
SELECT * FROM event_log
WHERE correlation_id = 'request-uuid'
ORDER BY created_at;

-- Search request payloads
SELECT * FROM event_log
WHERE request_payload @> '{"username": "testuser"}';

-- Comprehensive audit trail
SELECT * FROM comprehensive_audit_trail
WHERE user_id = 'user-uuid'
ORDER BY created_at DESC
LIMIT 100;
```

## Performance Considerations

1. **Payload Size Limits**: Default 10KB per payload to prevent log bloat
2. **Geolocation Caching**: Consider caching geolocation lookups for frequently seen IPs
3. **Async Logging**: Audit logging is async to avoid blocking request processing
4. **Index Usage**: GIN indexes on JSONB columns for efficient querying
5. **Partitioning**: Consider partitioning event_log by date for large volumes

## Security Considerations

1. **PII Protection**: All PII is masked or removed before storage
2. **Tamper-Proof**: Events are signed with HMAC-SHA256
3. **Compliance**: Supports GDPR, ISO 27001, and other regulatory requirements
4. **Access Control**: Audit logs should have restricted access
5. **Retention**: Configure appropriate retention policies

## Compliance

### GDPR
- PII is masked in audit logs
- Right to erasure: User data can be removed while preserving audit integrity
- Data minimization: Only necessary data is logged

### ISO 27001
- Comprehensive audit trail for all security events
- Tamper-proof logging with signatures
- Access logging for all administrative actions

### SOC 2
- Complete audit trail for compliance reporting
- Geolocation tracking for anomaly detection
- Session correlation for investigation

## Troubleshooting

### IP Address Not Captured

Check that your reverse proxy is setting the correct headers:
- X-Forwarded-For
- X-Real-IP
- CF-Connecting-IP (Cloudflare)
- True-Client-IP (Akamai)

### Geolocation Not Working

1. Verify IP address is public (not private/local)
2. Check geolocation service configuration
3. For production, integrate external service (MaxMind, IP2Location)

### Payloads Too Large

Adjust the sanitizer config:
```rust
config.max_payload_size = 50_000; // 50KB
```

### Performance Impact

1. Disable geolocation lookup if not needed
2. Reduce payload capture for high-volume endpoints
3. Use async logging (already default)
4. Consider sampling for very high-volume scenarios

## Migration Guide

### From Basic Audit Logging

```rust
// Before
db.log_event(
    realm_id,
    "LOGIN",
    "USER",
    None,
    None,
    None,
    Some(user_id),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    true,
    None,
    None,
    None,
).await?;

// After
let audit_context = audit_service.create_context(&headers).await;
let event = Event {
    id: Uuid::new_v4().to_string(),
    time: Utc::now(),
    event_type: EventType::Login,
    realm_id: realm_id.to_string(),
    user_id: Some(user_id.to_string()),
    // ... other fields
};
audit_service.log_user_event(event, &audit_context).await?;
```

## Best Practices

1. **Always capture context**: Use `create_context()` at the start of handlers
2. **Sanitize payloads**: Always use sanitizet/response data
3. **Add session IDs**: Enable correlationoss user sessions
4. **Use correlation IDs**: Enable distributed tracing
5. **Configure retention**: Set appropriate retention policies for your compliance needs
6. **Monitor performance**: Track audit logging latency and volume
7. **Test sanitization**: Verify sensitive data is properly redacted
8. **Review regularly**: Audit the audit logs for completeness

## Examples

See `handlers/audit_example.rs` for complete working examples.

