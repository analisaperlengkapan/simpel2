# Comprehensive Audit Events

This document describes the comprehensive audit event system implemented in Authenc for compliance and security monitoring.

## Overview

The audit event system provides detailed tracking of all security-relevant actions with complete context including:
- IP addresses
- User agents
- Timestamps
- Session information
- Actor information (who performed the action)
- Resource details

## Event Types

### 1. UserLogin Event

Tracks user authentication attempts with full context.

**Event Type**: `EventType::Login` (success) or `EventType::LoginError` (failure)

**Context Captured**:
- User ID and username
- Session ID
- Client ID
- IP address
- User agent
- Authentication method (password, MFA, OAuth2, etc.)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_user_login_event;

let event = create_user_login_event(
    realm_id,
    Some(user_id),
    Some(username),
    Some(session_id),
    Some(client_id),
    ip_address,
    user_agent,
    "password".to_string(),
    true, // success
);

// Publish to event bus
event_bus.dispatch_async(event);
```

### 2. UserLogout Event

Tracks user logout with session duration calculation.

**Event Type**: `EventType::Logout`

**Context Captured**:
- User ID and username
- Session ID
- Client ID
- IP address
- User agent
- Session start time
- Session end time
- Session duration (in seconds)
- Logout type (user_initiated, timeout, admin_forced)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_user_logout_event;

let event = create_user_logout_event(
    realm_id,
    user_id,
    Some(username),
    session_id,
    Some(client_id),
    ip_address,
    user_agent,
    session_start_time,
    "user_initiated".to_string(),
);

event_bus.dispatch_async(event);
```

### 3. MFAEnabled Event (User-Initiated)

Tracks when a user enables MFA for their own account.

**Event Type**: `EventType::MfaEnabled`

**Context Captured**:
- User ID and username
- IP address
- User agent
- MFA method (totp, webauthn, etc.)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_mfa_enabled_event;

let event = create_mfa_enabled_event(
    realm_id,
    user_id,
    Some(username),
    ip_address,
    user_agent,
    "totp".to_string(),
);

event_bus.dispatch_async(event);
```

### 4. MFAEnabled Admin Event (Admin-Initiated)

Tracks when an administrator enables MFA for another user.

**Event Type**: `AdminEvent` with `ResourceType::User` and `OperationType::Action`

**Context Captured**:
- Target user ID and username
- Admin user ID and username
- IP address (admin)
- User agent (admin)
- MFA method
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_mfa_enabled_admin_event;

let event = create_mfa_enabled_admin_event(
    realm_id,
    target_user_id,
    Some(target_username),
    admin_user_id,
    Some(admin_username),
    ip_address,
    user_agent,
    "totp".to_string(),
);

// Publish to admin event bus
admin_event_bus.dispatch_async(event);
```

### 5. MFADisabled Event (User-Initiated)

Tracks when a user disables MFA for their own account.

**Event Type**: `EventType::MfaDisabled`

**Context Captured**:
- User ID and username
- IP address
- User agent
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_mfa_disabled_event;

let event = create_mfa_disabled_event(
    realm_id,
    user_id,
    Some(username),
    ip_address,
    user_agent,
);

event_bus.dispatch_async(event);
```

### 6. MFADisabled Admin Event (Admin-Initiated)

Tracks when an administrator disables MFA for another user.

**Event Type**: `AdminEvent` with `ResourceType::User` and `OperationType::Action`

**Context Captured**:
- Target user ID and username
- Admin user ID and username
- IP address (admin)
- User agent (admin)
- Reason for disabling (optional)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_mfa_disabled_admin_event;

let event = create_mfa_disabled_admin_event(
    realm_id,
    target_user_id,
    Some(target_username),
    admin_user_id,
    Some(admin_username),
    ip_address,
    user_agent,
    Some("Account recovery".to_string()),
);

admin_event_bus.dispatch_async(event);
```

### 7. PermissionGranted Admin Event

Tracks when an administrator grants a permission to a user.

**Event Type**: `AdminEvent` with `ResourceType::Permission` and `OperationType::Create`

**Context Captured**:
- Target user ID and username
- Admin user ID and username
- IP address (admin)
- User agent (admin)
- Resource name
- Action (read, write, delete, etc.)
- Scope (optional, e.g., satker:12345)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_permission_granted_admin_event;

let event = create_permission_granted_admin_event(
    realm_id,
    target_user_id,
    Some(target_username),
    admin_user_id,
    Some(admin_username),
    ip_address,
    user_agent,
    "documents".to_string(),
    "write".to_string(),
    Some("satker:12345".to_string()),
);

admin_event_bus.dispatch_async(event);
```

### 8. PermissionRevoked Admin Event

Tracks when an administrator revokes a permission from a user.

**Event Type**: `AdminEvent` with `ResourceType::Permission` and `OperationType::Delete`

**Context Captured**:
- Target user ID and username
- Admin user ID and username
- IP address (admin)
- User agent (admin)
- Resource name
- Action (read, write, delete, etc.)
- Reason for revocation (optional)
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_permission_revoked_admin_event;

let event = create_permission_revoked_admin_event(
    realm_id,
    target_user_id,
    Some(target_username),
    admin_user_id,
    Some(admin_username),
    ip_address,
    user_agent,
    "documents".to_string(),
    "write".to_string(),
    Some("User role changed".to_string()),
);

admin_event_bus.dispatch_async(event);
```

### 9. PasswordChanged Event

Tracks password changes with context about how the change was initiated.

**Event Type**: `EventType::UpdatePassword`

**Context Captured**:
- User ID and username
- IP address
- User agent
- Whether changed by admin (vs self-service)
- Admin user ID (if applicable)
- Whether a reset token was used
- Timestamp

**Usage Example**:
```rust
use crate::services::audit_events::create_password_changed_event;

// Self-service password change
let event = create_password_changed_event(
    realm_id,
    user_id,
    Some(username),
    ip_address,
    user_agent,
    false, // not changed by admin
    None,  // no admin user ID
    false, // no reset token used
);

event_bus.dispatch_async(event);

// Admin-initiated password reset
let event = create_password_changed_event(
    realm_id,
    user_id,
    Some(username),
    ip_address,
    user_agent,
    true,  // changed by admin
    Some(admin_user_id),
    false, // no reset token used
);

event_bus.dispatch_async(event);

// Password reset via token
let event = create_password_changed_event(
    realm_id,
    user_id,
    Some(username),
    ip_address,
    user_agent,
    false, // not changed by admin
    None,
    true,  // reset token was used
);

event_bus.dispatch_async(event);
```

## Event Storage and Retrieval

All events are:
1. Published to Kafka for real-time processing
2. Stored in PostgreSQL for audit trail
3. Optionally streamed to Elasticsearch for SIEM integration
4. Signed with HMAC-SHA256 for tamper-proof logging

## Compliance Requirements

These comprehensive audit events satisfy the following compliance requirements:

### Requirement 10.1
> THE Authenc SHALL mencatat semua authentication attempts (success dan failure) dengan timestamp, IP address, dan user agent

✅ Implemented via `create_user_login_event` with full context

### Requirement 10.2
> WHEN administrative actions dilakukan, THE Authenc SHALL mencatat admin events dengan actor, action, target, dan timestamp

✅ Implemented via:
- `create_mfa_enabled_admin_event`
- `create_mfa_disabled_admin_event`
- `create_permission_granted_admin_event`
- `create_permission_revoked_admin_event`

## Integration with Event Bus

All audit events should be published through the event bus for:
- Asynchronous processing
- Kafka streaming
- Cache invalidation
- Real-time monitoring
- SIEM integration

Example integration:
```rust
// In your handler
if let Some(event_bus) = &state.event_bus {
    let event = create_user_login_event(
        realm_id,
        Some(user_id),
        Some(username),
        Some(session_id),
        Some(client_id),
        ip_address,
        user_agent,
        "password".to_string(),
        true,
    );

    event_bus.dispatch_async(event);
}
```

## Testing

Comprehensive unit tests are included in `src/services/audit_events.rs` to verify:
- Event creation with all required fields
- Proper event type assignment
- Detail field population
- Session duration calculation
- JSON serialization of admin event representations

Run tests with:
```bash
cargo test --package authenc audit_events
```

## Future Enhancements

Potential future additions:
- WebAuthn registration/authentication events
- OAuth2 consent grant/revoke events
- API key creation/revocation events
- Security policy violation events
- Anomaly detection alert events

## References

- Requirements: `.kiro/specs/authenc-comprehensive-optimization/requirements.md`
- Design: `.kiro/specs/authenc-comprehensive-optimization/design.md`
- Event Models: `src/models/events.rs`
- Event Publisher: `src/services/event_publisher.rs`
