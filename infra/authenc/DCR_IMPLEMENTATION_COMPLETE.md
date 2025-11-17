# Dynamic Client Registration (DCR) - Implementation Complete

## Overview

Comprehensive implementation of OAuth 2.0 Dynamic Client Registration (RFC 7591/7592) for Authenc with production-ready security and best practices.

## Features Implemented

### ✅ Core RFC 7591/7592 Compliance

- **POST /oauth2/register** - Register new clients dynamically
- **GET /oauth2/register/{client_id}** - Get client configuration
- **PUT /oauth2/register/{client_id}** - Update client configuration
- **DELETE /oauth2/register/{client_id}** - Delete client registration

### ✅ Security Enhancements

- **Bcrypt-hashed client secrets** (DEFAULT_COST = 12)
- **Registration access tokens** stored with bcrypt hashing in database
- **Initial access tokens** for protected registration endpoint
- **HTTPS enforcement** for redirect URIs (configurable)
- **Open redirect protection** via URI pattern validation
- **JWKS validation** for client-provided JSON Web Key Sets
- **Software statement JWT validation** (framework ready)

### ✅ Database Persistence

- **client_registration_tokens** - Persistent registration access tokens
- **initial_access_tokens** - Tokens for protecting registration endpoint
- **client_registration_policies** - Per-realm registration policies
- **client_registration_audit_log** - Comprehensive audit trail
- **software_statement_issuers** - Trusted JWT statement issuers

### ✅ Enhanced OAuth2Client Model

Full RFC 7591 metadata support:

- `logo_uri`, `client_uri`, `policy_uri`, `tos_uri`
- `jwks_uri`, `jwks`, `sector_identifier_uri`
- `subject_type`, algorithm preferences (signing/encryption)
- `application_type`, `contacts`, `software_id`, `software_version`
- `client_id_issued_at`, `client_secret_expires_at`

### ✅ Policy-Based Registration

Configurable per realm:

- Enable/disable dynamic registration
- Require initial access token
- Require software statement
- Allowed/blocked redirect URI patterns (regex)
- Maximum redirect URIs
- Allowed scopes, grant types, response types
- HTTPS requirement (with localhost exception)
- Client secret expiration
- Registration token expiration

### ✅ Admin API

**Endpoints:**

- `POST /api/v1/admin/dcr/initial-access-tokens` - Create IAT
- `GET /api/v1/admin/dcr/initial-access-tokens` - List IATs
- `DELETE /api/v1/admin/dcr/initial-access-tokens/:id` - Revoke IAT
- `GET /api/v1/admin/dcr/policies/:realm_id` - Get registration policy

## Database Schema

### Migration: V009\_\_client_registration_dcr.sql

```sql
-- Enhanced oauth2_clients table with RFC 7591 metadata
ALTER TABLE oauth2_clients ADD COLUMN ...

-- Registration access tokens
CREATE TABLE client_registration_tokens (
    id UUID PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    client_id UUID NOT NULL REFERENCES oauth2_clients(id),
    realm_id UUID REFERENCES realms(id),
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

-- Initial access tokens
CREATE TABLE initial_access_tokens (
    id UUID PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    realm_id UUID REFERENCES realms(id),
    count INTEGER NOT NULL DEFAULT 1,
    remaining_count INTEGER NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Registration policies
CREATE TABLE client_registration_policies (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id),
    name TEXT NOT NULL,
    allow_dynamic_registration BOOLEAN DEFAULT true,
    require_initial_access_token BOOLEAN DEFAULT false,
    require_software_statement BOOLEAN DEFAULT false,
    allowed_redirect_uri_patterns TEXT[],
    blocked_redirect_uri_patterns TEXT[],
    max_redirect_uris INTEGER DEFAULT 10,
    allowed_scopes TEXT[],
    default_scopes TEXT[],
    require_https_redirect_uris BOOLEAN DEFAULT true,
    allow_localhost_redirect BOOLEAN DEFAULT false
);

-- Audit log
CREATE TABLE client_registration_audit_log (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    client_id UUID REFERENCES oauth2_clients(id),
    success BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Usage Examples

### 1. Protected Registration (with Initial Access Token)

**Step 1: Admin creates Initial Access Token**

```bash
curl -X POST https://authenc.example.com/api/v1/admin/dcr/initial-access-tokens \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "count": 5,
    "expires_in": 86400
  }'
```

Response:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "token": "vF7dBQz...8HdKq3",
  "count": 5,
  "remaining_count": 5,
  "expires_at": "2025-11-12T10:00:00Z"
}
```

**Step 2: Client registers using IAT**

```bash
curl -X POST https://authenc.example.com/oauth2/register \
  -H "Authorization: Bearer vF7dBQz...8HdKq3" \
  -H "Content-Type: application/json" \
  -d '{
    "redirect_uris": ["https://app.example.com/callback"],
    "grant_types": ["authorization_code"],
    "response_types": ["code"],
    "client_name": "My Application",
    "application_type": "web",
    "contacts": ["admin@example.com"]
  }'
```

Response:

```json
{
  "client_id": "7a8f2c4e-3b1d-4a6c-9f8e-2d5b7c4a3e1f",
  "client_secret": "kJ8mN...pQr2",
  "client_id_issued_at": 1699790400,
  "client_secret_expires_at": null,
  "redirect_uris": ["https://app.example.com/callback"],
  "grant_types": ["authorization_code"],
  "response_types": ["code"],
  "client_name": "My Application",
  "registration_access_token": "xY9zA...bC4dE",
  "registration_client_uri": "/oauth2/register/7a8f2c4e-3b1d-4a6c-9f8e-2d5b7c4a3e1f"
}
```

### 2. Open Registration (without Initial Access Token)

**Prerequisites:** Set policy `require_initial_access_token = false`

```bash
curl -X POST https://authenc.example.com/oauth2/register \
  -H "Content-Type: application/json" \
  -d '{
    "redirect_uris": ["https://app.example.com/callback"],
    "client_name": "My App"
  }'
```

### 3. Update Client Configuration

```bash
curl -X PUT https://authenc.example.com/oauth2/register/CLIENT_ID \
  -H "Authorization: Bearer xY9zA...bC4dE" \
  -H "Content-Type: application/json" \
  -d '{
    "client_name": "My Updated App",
    "redirect_uris": ["https://app.example.com/callback", "https://app.example.com/callback2"],
    "logo_uri": "https://app.example.com/logo.png"
  }'
```

### 4. Get Client Configuration

```bash
curl -X GET https://authenc.example.com/oauth2/register/CLIENT_ID \
  -H "Authorization: Bearer xY9zA...bC4dE"
```

### 5. Delete Client

```bash
curl -X DELETE https://authenc.example.com/oauth2/register/CLIENT_ID \
  -H "Authorization: Bearer xY9zA...bC4dE"
```

## Configuration

### Default Policy Settings

When a realm first uses DCR, a default policy is auto-created:

```json
{
  "allow_dynamic_registration": true,
  "require_initial_access_token": false,
  "require_software_statement": false,
  "blocked_redirect_uri_patterns": ["http://localhost*"],
  "max_redirect_uris": 10,
  "allowed_scopes": ["openid", "profile", "email"],
  "default_scopes": ["openid", "profile"],
  "allowed_grant_types": ["authorization_code", "refresh_token"],
  "allowed_response_types": ["code"],
  "require_https_redirect_uris": true,
  "allow_localhost_redirect": true,
  "registration_token_expires_in": 31536000
}
```

### Customizing Policy

Policies are stored in `client_registration_policies` table. Admins can:

- Create custom policies per realm
- Set strict URI patterns: `allowed_redirect_uri_patterns: ["^https://.*\\.example\\.com/.*"]`
- Limit scopes: `allowed_scopes: ["openid", "profile"]`
- Enforce software statements: `require_software_statement: true`

## Security Considerations

### 1. Token Hashing

- Client secrets: **bcrypt with cost 12**
- Registration tokens: **bcrypt with cost 12**
- Initial access tokens: **bcrypt with cost 12**

### 2. URI Validation

- HTTPS enforcement (configurable)
- Regex pattern matching for allow/block lists
- Open redirect protection via strict validation
- Localhost allowed only if explicitly enabled

### 3. JWKS Validation

- Validates JSON structure
- Requires `keys` array
- Checks key format (TODO: Add cryptographic validation)

### 4. Audit Logging

All operations logged to `client_registration_audit_log`:

- Registration attempts (success/failure)
- Configuration updates
- Client deletions
- Token usage

### 5. Rate Limiting

TODO: Add rate limiting middleware for `/oauth2/register` endpoint

## Code Structure

### New Files Created

```
infra/authenc/
├── migrations/V009__client_registration_dcr.sql
├── src/
│   ├── models/oauth2.rs (enhanced with DCR fields)
│   ├── database/operations/
│   │   ├── mod.rs (added client_registration module)
│   │   └── client_registration_ops.rs (NEW - comprehensive DB ops)
│   ├── services/
│   │   ├── client_registration.rs (legacy - kept for compatibility)
│   │   └── client_registration_v2.rs (NEW - production implementation)
│   └── handlers/
│       ├── client_registration.rs (updated to use v2 service)
│       └── dcr_admin.rs (NEW - admin API)
```

### Key Components

**Service:** `ProductionClientRegistrationService`

- Database-backed (not in-memory)
- Bcrypt password hashing
- Policy enforcement
- URI validation
- JWKS validation

**Database Operations:** `client_registration_ops.rs`

- `create_registration_token()`
- `validate_registration_token()`
- `create_initial_access_token()`
- `consume_initial_access_token()`
- `get_or_create_default_policy()`
- `log_registration_audit()`
- `create_client_with_metadata()`
- `update_client_with_metadata()`

## Testing

### Manual Testing

1. **Start Authenc:**

   ```bash
   cd infra/authenc
   cargo build --release
   cargo run
   ```

2. **Run migration:**

   ```bash
   psql -U postgres -d authenc < migrations/V009__client_registration_dcr.sql
   ```

3. **Create initial access token:**

   ```bash
   curl -X POST http://localhost:8080/api/v1/admin/dcr/initial-access-tokens \
     -H "Content-Type: application/json" \
     -d '{"count": 1}'
   ```

4. **Register client:**
   ```bash
   curl -X POST http://localhost:8080/oauth2/register \
     -H "Authorization: Bearer YOUR_IAT" \
     -H "Content-Type: application/json" \
     -d '{
       "redirect_uris": ["https://example.com/callback"],
       "client_name": "Test Client"
     }'
   ```

### Integration Tests

TODO: Add comprehensive integration tests using testcontainers

## Migration from Old Implementation

**Before (in-memory):**

```rust
let service = DefaultClientRegistrationService::new(
    client_store.clone(),
    true,  // enable_dynamic_registration
    false, // require_software_statement
);
```

**After (database-backed):**

```rust
use crate::services::client_registration_v2::ProductionClientRegistrationService;

let service = ProductionClientRegistrationService::new(
    database.clone(),
    Some(realm_id),
    "/api/v1/oauth2/register".to_string(),
);
```

**Backward compatibility:** Type alias `DefaultClientRegistrationService = ProductionClientRegistrationService` maintains compatibility.

## Future Enhancements

### Phase 1 (Completed ✅)

- ✅ Database persistence
- ✅ Initial access tokens
- ✅ Registration access tokens
- ✅ Policy-based registration
- ✅ URI validation
- ✅ Audit logging

### Phase 2 (Next Sprint)

- [ ] Software statement JWT validation
- [ ] JWKS cryptographic validation
- [ ] Rate limiting middleware
- [ ] Integration tests
- [ ] Admin UI for policy management

### Phase 3 (Future)

- [ ] Client logo storage (S3/MinIO)
- [ ] Client reputation scoring
- [ ] Automatic client approval/rejection
- [ ] FAPI compliance
- [ ] Client attestation (Android/iOS)

## References

- [RFC 7591 - OAuth 2.0 Dynamic Client Registration Protocol](https://www.rfc-editor.org/rfc/rfc7591.html)
- [RFC 7592 - OAuth 2.0 Dynamic Client Registration Management Protocol](https://www.rfc-editor.org/rfc/rfc7592.html)
- [OpenID Connect Dynamic Client Registration 1.0](https://openid.net/specs/openid-connect-registration-1_0.html)

## Support

For questions or issues:

- Internal: Slack #simpelv2-security
- Email: security@kejaksaan.go.id
- Documentation: https://docs.simpel.kejaksaan.go.id/authenc/dcr
