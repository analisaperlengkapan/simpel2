# Task 10.7: OIDC Logout Endpoint Implementation

## Overview

Implemented comprehensive OIDC logout endpoint with SSO session termination, federated logout propagation, and event publishing for audit trail and cache invalidation.

## Implementation Summary

### 1. Enhanced Logout Endpoint (`oidc_logout_with_sso`)

**Location**: `infra/authenc/src/handlers/oidc_sso.rs`

**Features Implemented**:
- ✅ SSO session termination
- ✅ Post-logout redirect URI validation (prevents open redirect attacks)
- ✅ Server-side session invalidation
- ✅ Federated provider logout propagation
- ✅ SSO cookie deletion
- ✅ Logout event publishing to event bus

**Security Enhancements**:
- Validates `post_logout_redirect_uri` against whitelist of allowed domains
- Defaults to Portal URL if validation fails
- Extracts client information (IP address, user agent) for audit logging
- Supports `id_token_hint` parameter for federated logout

### 2. Session Store Enhancement

**Location**: `infra/authenc/src/services/session_store.rs`

**Added Method**:
```rust
pub async fn invalidate_user_sessions(&self, user_id: Uuid) -> Result<(), AuthencError>
```

This method invalidates all active sessions for a user during logout, ensuring complete session termination.

### 3. Federation Provider Enhancement

**Location**: `infra/authenc/src/services/federation_provider.rs`

**Added Methods to Trait**:
```rust
fn name(&self) -> &str;
async fn logout(&self, id_token_hint: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
```

**Added Method to Registry**:
```rust
pub fn list_providers(&self) -> Vec<&dyn FederationProvider>
```

These additions enable logout propagation to federated identity providers (Google, GitHub, SAML, etc.).

### 4. Route Registration

**Location**: `infra/authenc/src/handlers/mod.rs`

**Added Route**:
```rust
.route("/oidc/logout", get(oidc_sso::oidc_logout_with_sso))
```

### 5. OIDC Discovery Enhancement

**Location**: `infra/authenc/src/handlers/oidc_ed25519.rs`

**Added Field**:
```json
{
  "end_session_endpoint": "http://localhost:8080/v1/oidc/logout"
}
```

This allows OIDC clients to discover the logout endpoint automatically.

## Logout Flow

```
1. Client sends GET /oidc/logout?post_logout_redirect_uri=...&id_token_hint=...
2. Extract SSO session from cookie
3. Validate post_logout_redirect_uri (prevent open redirect)
4. Invalidate all server-side sessions for user
5. Propagate logout to federated providers (if id_token_hint provided)
6. Publish logout event to event bus (Kafka)
   - Event includes: user_id, username, IP, user agent, timestamp
   - Used for audit trail and cache invalidation
7. Clear SSO cookie (Max-Age=0)
8. Redirect to validated post_logout_redirect_uri
```

## Event Publishing

The logout endpoint publishes a `UserLogout` event with the following data:

```rust
Event {
    event_type: EventType::UserLogout,
    event_category: EventCategory::Auth,
    user_id: Uuid,
    username: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    session_id: Option<Uuid>,
    event_data: {
        "logout_type": "oidc_sso",
        "post_logout_redirect_uri": String,
        "federated_logout": bool,
        "timestamp": String (RFC3339)
    }
}
```

This event is consumed by:
- Audit log sink (for compliance)
- Cache invalidation listeners (to clear user caches)
- Analytics services (for user behavior tracking)

## Security Considerations

### 1. Open Redirect Prevention

The `validate_post_logout_redirect_uri` function validates redirect URIs against a whitelist:

```rust
let allowed_patterns = vec![
    "http://localhost",
    "https://simpel.kejaksaan.go.id",
    "https://portal.simpel.kejaksaan.go.id",
    "https://authenc.simpel.kejaksaan.go.id",
];
```

If validation fails, it defaults to the Portal URL instead of allowing arbitrary redirects.

### 2. Federated Logout

The `propagate_federated_logout` function:
- Extracts provider information from `id_token_hint`
- Calls provider-specific logout endpoints
- Handles failures gracefully (logs warning but continues with local logout)
- Prevents logout failures from blocking user logout

### 3. Session Termination

- Invalidates ALL sessions for the user (not just current session)
- Clears both in-memory and database-backed sessions
- Removes SSO cookie with proper flags (Max-Age=0, HttpOnly, Secure, SameSite=Lax)

## Testing

### Unit Tests

**Location**: `infra/authenc/src/handlers/oidc_sso.rs`

**Tests Added**:
1. `test_oidc_logout_with_sso` - Basic logout functionality
2. `test_oidc_logout_with_post_logout_redirect` - Redirect URI handling
3. `test_validate_post_logout_redirect_uri` - URI validation

### Integration Testing

To test the complete logout flow:

```bash
# 1. Login to get SSO cookie
curl -X POST http://localhost:8080/v1/oidc/token \
  -d "grant_type=authorization_code&code=test_code"

# 2. Logout with redirect
curl -X GET "http://localhost:8080/v1/oidc/logout?post_logout_redirect_uri=https://portal.simpel.kejaksaan.go.id/logged-out" \
  -H "Cookie: AUTHENC_SSO=..."

# 3. Verify SSO cookie is cleared (Max-Age=0)
# 4. Verify redirect to post_logout_redirect_uri
# 5. Verify logout event published to Kafka
```

## Configuration

### Environment Variables

- `SECRETON_ENDPOINT` - Secreton service endpoint for MFA secret management
- `KAFKA_BROKERS` - Kafka brokers for event publishing
- `KAFKA_USER_EVENTS_TOPIC` - Topic for user events (including logout)

### SSO Cookie Configuration

```toml
[sso_cookie]
name = "AUTHENC_SSO"
domain = "simpel.kejaksaan.go.id"
path = "/"
max_age = 3600
secure = true
http_only = true
same_site = "Lax"
```

## Requirements Fulfilled

✅ **Requirement 19.5**: THE Authenc SHALL menyediakan logout endpoint yang menghapus SSO session dan redirect ke Portal

**Acceptance Criteria Met**:
1. ✅ SSO session termination - Invalidates all user sessions
2. ✅ Post-logout redirect URI parameter - Validated and supported
3. ✅ Logout propagation to federated providers - Implemented with graceful error handling
4. ✅ Clear AUTHENC_SSO cookie - Cookie deleted with Max-Age=0
5. ✅ Logout event publishing - Published to event bus for audit and cache invalidation

## Future Enhancements

1. **Database-backed redirect URI whitelist** - Currently hardcoded, should be configurable per client
2. **Logout token support** - Implement RP-initiated logout with logout tokens (OpenID Connect Back-Channel Logout)
3. **Front-channel logout** - Support for front-channel logout notifications to other clients
4. **Logout confirmation page** - Optional confirmation page before logout
5. **Session activity tracking** - Track session duration and activity for analytics

## Related Files

- `infra/authenc/src/handlers/oidc_sso.rs` - Main logout implementation
- `infra/authenc/src/handlers/oidc_ed25519.rs` - OIDC discovery enhancement
- `infra/authenc/src/handlers/mod.rs` - Route registration
- `infra/authenc/src/services/session_store.rs` - Session invalidation
- `infra/authenc/src/services/federation_provider.rs` - Federated logout
- `infra/authenc/src/events/mod.rs` - Event system
- `.kiro/specs/authenc-comprehensive-optimization/requirements.md` - Requirements document
- `.kiro/specs/authenc-comprehensive-optimization/design.md` - Design document

## Conclusion

Task 10.7 has been successfully implemented with comprehensive logout functionality that meets all security and compliance requirements. The implementation includes SSO session termination, federated logout propagation, event publishing for audit trail, and proper security measures to prevent open redirect attacks.
