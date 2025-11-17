# User Federation Implementation - Complete

## Overview

This document describes the complete end-to-end user federation implementation in Authenc, following enterprise IAM best practices (Keycloak/RedHat SSO compatible).

## Architecture

### Components

1. **Federation Manager** (`services/federation_manager.rs`)

   - Central orchestration for all federation providers
   - LDAP/Active Directory provider management
   - Social login provider management (Google, GitHub, Facebook)
   - Just-In-Time (JIT) user provisioning
   - Federated identity link management

2. **User Sync Service** (`services/user_sync_service.rs`)

   - Periodic batch synchronization from LDAP/AD
   - Scheduled sync jobs with configurable intervals
   - Manual sync triggers via API
   - Sync statistics and history tracking

3. **SPI Providers** (`spi/`)

   - `ldap_federation.rs` - Complete LDAP/AD integration with ldap3 crate
   - `social.rs` - OAuth2/OIDC social providers (Google, GitHub, Facebook)
   - Pluggable provider architecture

4. **Admin API** (`handlers/federation_admin.rs`)

   - Identity provider CRUD operations
   - Manual sync triggers
   - Federation statistics and monitoring
   - User identity link management

5. **Federated Login** (`handlers/federated_login.rs`)
   - LDAP/AD login flow
   - Social login OAuth2 flow (authorize + callback)
   - JWT token generation for federated users
   - Provider discovery endpoint

## Database Schema

### Tables

1. **`federated_identity_links`**

   - Links local users to external identity providers
   - Stores federated user ID and attributes
   - Tracks authentication statistics
   - Optional token storage (configurable per provider)

2. **`identity_broker_configs`**

   - Identity provider configurations
   - Provider type (LDAP, AD, Social)
   - Trust settings (email verification, token storage)
   - Link-only mode for existing users

3. **`identity_provider_mappers`**

   - Attribute mapping from external IdP to local user model
   - Mapper types: attribute-importer, role-mapper, etc.
   - Synchronization modes: IMPORT, FORCE, LEGACY

4. **`federated_auth_log`**
   - Audit log for federated authentication attempts
   - Security monitoring and compliance

## Features

### ✅ Implemented

#### LDAP/Active Directory Federation

- Full LDAP integration using `ldap3` crate
- User authentication against LDAP/AD
- User search and discovery
- Attribute mapping and synchronization
- Group membership extraction
- Batch user import/sync
- SSL/TLS support (LDAPS)
- Connection pooling
- Configurable search filters and base DN

#### Social Login Providers

- Google OAuth2/OIDC
- GitHub OAuth
- Facebook OAuth
- Generic OAuth2 provider support
- Authorization code flow with PKCE
- Token exchange and refresh
- User profile fetching
- Provider-specific profile parsing

#### Just-In-Time (JIT) Provisioning

- Automatic user creation on first login
- Attribute mapping from external provider
- Email verification trust settings
- Configurable link-only mode
- Federated user flag

#### User Synchronization

- Periodic batch sync from LDAP/AD
- Configurable sync interval
- Batch size configuration
- Sync statistics (added/updated/failed)
- Manual sync triggers
- Sync history tracking

#### Admin Management

- Identity provider CRUD via REST API
- Provider enable/disable
- Manual sync triggers
- Federation statistics
- User identity link management
- Provider configuration validation

#### Security Features

- Zero-trust architecture compatible
- Encrypted token storage (optional)
- Audit logging for all federated auth
- CSRF protection (state parameter)
- Token expiration tracking
- Authentication attempt counting

## API Endpoints

### Federated Login (Public)

```http
# LDAP/AD Login
POST /api/v1/auth/federated/login/ldap
Content-Type: application/json

{
  "provider_alias": "corporate-ldap",
  "username": "john.doe",
  "password": "secure_password",
  "realm_id": "realm-uuid" // optional
}

Response:
{
  "success": true,
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user_id": "user-uuid",
  "username": "john.doe"
}

# Social Login - Get Authorization URL
GET /api/v1/auth/federated/login/social/authorize?provider_alias=google&realm_id=realm-uuid

Response:
{
  "authorization_url": "https://accounts.google.com/o/oauth2/v2/auth?...",
  "state": "google:realm-uuid:session-uuid"
}

# Social Login - Callback
GET /api/v1/auth/federated/login/social/callback?code=auth_code&state=state_param

Response: (same as LDAP login response)

# List Available Providers
GET /api/v1/auth/federated/providers

Response:
[
  {
    "alias": "corporate-ldap",
    "display_name": "Corporate LDAP",
    "provider_type": "ldap",
    "enabled": true
  },
  {
    "alias": "google",
    "display_name": "Google",
    "provider_type": "social",
    "enabled": true
  }
]
```

### Federation Admin (Admin-only)

```http
# List Identity Providers
GET /api/v1/admin/federation/identity-providers?realm_id=realm-uuid

# Create Identity Provider
POST /api/v1/admin/federation/identity-providers
Content-Type: application/json

{
  "alias": "corporate-ad",
  "display_name": "Corporate Active Directory",
  "provider_type": "active_directory",
  "enabled": true,
  "trust_email": true,
  "store_token": false,
  "link_only": false,
  "config": {
    "server_url": "ldaps://ad.company.com:636",
    "base_dn": "dc=company,dc=com",
    "bind_dn": "cn=service,ou=users,dc=company,dc=com",
    "bind_password": "service_password",
    "user_search_filter": "(sAMAccountName={0})",
    "username_attribute": "sAMAccountName",
    "sync_enabled": true,
    "sync_interval": 3600,
    "batch_size": 100
  }
}

# Update Identity Provider
PUT /api/v1/admin/federation/identity-providers/{id}
Content-Type: application/json

{
  "display_name": "Updated Display Name",
  "enabled": true
}

# Delete Identity Provider
DELETE /api/v1/admin/federation/identity-providers/{id}

# Trigger Manual Sync
POST /api/v1/admin/federation/sync/trigger/{alias}?realm_id=realm-uuid

Response:
{
  "provider_alias": "corporate-ldap",
  "users_added": 50,
  "users_updated": 120,
  "users_removed": 0,
  "failed": 2,
  "duration_ms": 15000,
  "timestamp": "2025-11-11T10:00:00Z"
}

# Get Sync Status
GET /api/v1/admin/federation/sync/status

Response:
{
  "is_running": false,
  "timestamp": "2025-11-11T10:00:00Z"
}

# Get Sync History
GET /api/v1/admin/federation/sync/history/{alias}?realm_id=realm-uuid&limit=10

# Get User Identity Links
GET /api/v1/admin/federation/users/{user_id}/identity-links

# Delete User Identity Link
DELETE /api/v1/admin/federation/users/{user_id}/identity-links/{link_id}

# Get Federation Statistics
GET /api/v1/admin/federation/statistics/{alias}

Response:
{
  "provider_alias": "corporate-ldap",
  "total_links": 500,
  "total_users": 500,
  "total_authentications": 15000,
  "timestamp": "2025-11-11T10:00:00Z"
}
```

## Configuration

### Environment Variables / Config File

```toml
[federation]
# Enable user synchronization
sync_enabled = true

# Sync interval in minutes (default: 60)
sync_interval_minutes = 60

# Batch size for sync operations (default: 100)
batch_size = 100
```

### LDAP Provider Configuration

```json
{
  "server_url": "ldaps://ldap.example.com:636",
  "base_dn": "dc=example,dc=com",
  "bind_dn": "cn=admin,dc=example,dc=com",
  "bind_password": "admin_password",
  "user_search_filter": "(uid={0})",
  "username_attribute": "uid",
  "rdn_attribute": "uid",
  "uuid_attribute": "entryUUID",
  "user_object_classes": ["person", "organizationalPerson", "user"],
  "connection_timeout": 30,
  "read_timeout": 30,
  "connection_pool_size": 10,
  "use_ssl": true,
  "custom_user_attributes": {
    "employeeNumber": "employeeNumber",
    "department": "departmentNumber",
    "title": "title"
  },
  "import_enabled": true,
  "sync_enabled": true,
  "sync_interval": 3600,
  "batch_size": 100
}
```

### Active Directory Configuration

```json
{
  "server_url": "ldaps://ad.company.com:636",
  "base_dn": "dc=company,dc=com",
  "bind_dn": "cn=service,ou=users,dc=company,dc=com",
  "bind_password": "service_password",
  "user_search_filter": "(sAMAccountName={0})",
  "username_attribute": "sAMAccountName",
  "rdn_attribute": "cn",
  "uuid_attribute": "objectGUID",
  "user_object_classes": ["user", "person", "organizationalPerson"],
  "connection_timeout": 30,
  "read_timeout": 30,
  "use_ssl": true,
  "custom_user_attributes": {
    "employeeID": "employeeID",
    "department": "department",
    "manager": "manager"
  },
  "sync_enabled": true,
  "sync_interval": 3600,
  "batch_size": 100
}
```

### Social Provider Configuration (Google)

```json
{
  "provider_type": "Google",
  "client_id": "your-google-client-id.apps.googleusercontent.com",
  "client_secret": "your-google-client-secret",
  "authorization_url": "https://accounts.google.com/o/oauth2/v2/auth",
  "token_url": "https://oauth2.googleapis.com/token",
  "user_info_url": "https://www.googleapis.com/oauth2/v2/userinfo",
  "scope": "openid email profile",
  "enabled": true,
  "trust_email": true,
  "store_tokens": true,
  "link_only": false
}
```

## Authentication Flows

### LDAP/AD Login Flow

```
1. User submits credentials to /api/v1/auth/federated/login/ldap
2. FederationManager.authenticate_ldap() called
3. LDAP provider authenticates user via LDAP bind
4. Check for existing federated identity link
5. If exists: Load local user, update auth stats
6. If not exists: JIT provision new user, create identity link
7. Generate JWT access/refresh tokens
8. Return tokens to client
```

### Social Login Flow

```
1. Client requests authorization URL: GET /login/social/authorize?provider_alias=google
2. Server generates state parameter and authorization URL
3. Client redirects user to authorization URL
4. User authenticates with social provider
5. Provider redirects back to callback: GET /login/social/callback?code=...&state=...
6. Server validates state parameter
7. FederationManager.authenticate_social() called
8. Exchange authorization code for access token
9. Fetch user profile from social provider
10. Check for existing federated identity link
11. If exists: Load local user
12. If not exists: JIT provision new user, create identity link
13. Generate JWT access/refresh tokens
14. Return tokens to client
```

### User Sync Flow

```
1. Sync scheduler triggers at configured interval
2. UserSyncService.sync_all_providers() called
3. For each enabled LDAP/AD provider:
   a. Load provider configuration
   b. Search LDAP for all users (batch by batch_size)
   c. For each user:
      - Check if federated identity link exists
      - If exists: Update user attributes
      - If not exists: Create new user + identity link
   d. Track statistics (added/updated/failed)
4. Log sync results
5. Store sync history
```

## Security Considerations

### Authentication Security

- Passwords never stored for federated users
- LDAP credentials validated against directory server
- OAuth2 authorization code flow with PKCE
- State parameter for CSRF protection
- Token expiration tracking

### Data Privacy

- Optional token storage (configurable per provider)
- Encrypted storage for sensitive attributes
- Audit logging for all federated auth
- GDPR-compliant user data handling

### Network Security

- LDAPS (LDAP over SSL/TLS) recommended
- Certificate validation
- Connection timeout configuration
- Retry logic with exponential backoff

## Monitoring and Observability

### Metrics

- Federation authentication success/failure rate
- Sync job duration and statistics
- Provider availability
- JIT provisioning rate
- Identity link count per provider

### Logging

- All federated authentication attempts
- Sync job execution logs
- Provider configuration changes
- User provisioning events
- Error and warning logs

### Audit Trail

- `federated_auth_log` table records all federated auth attempts
- Provider configuration changes audited
- User linking/unlinking events
- Token refresh events

## Testing

### Unit Tests

- Provider configuration validation
- Attribute mapping logic
- Token generation
- Error handling

### Integration Tests

- LDAP authentication flow
- Social OAuth2 flow
- JIT provisioning
- User sync operations
- API endpoints

### Manual Testing

```bash
# 1. Create LDAP provider
curl -X POST http://localhost:3000/api/v1/admin/federation/identity-providers \
  -H "Content-Type: application/json" \
  -d '{
    "alias": "test-ldap",
    "display_name": "Test LDAP",
    "provider_type": "ldap",
    "enabled": true,
    "trust_email": true,
    "config": {
      "server_url": "ldap://localhost:389",
      "base_dn": "dc=example,dc=com",
      "user_search_filter": "(uid={0})",
      "sync_enabled": true
    }
  }'

# 2. Test LDAP login
curl -X POST http://localhost:3000/api/v1/auth/federated/login/ldap \
  -H "Content-Type: application/json" \
  -d '{
    "provider_alias": "test-ldap",
    "username": "testuser",
    "password": "testpass"
  }'

# 3. Trigger manual sync
curl -X POST http://localhost:3000/api/v1/admin/federation/sync/trigger/test-ldap?realm_id=realm-uuid

# 4. Get federation statistics
curl http://localhost:3000/api/v1/admin/federation/statistics/test-ldap
```

## Migration Guide

### From Existing IAM System

1. **Export users from existing system**
2. **Configure identity providers in Authenc**
3. **Run initial sync** to import all users
4. **Test authentication** with sample users
5. **Enable scheduled sync**
6. **Monitor sync jobs** and authentication rates
7. **Gradually migrate** applications to use Authenc

### From Manual User Management

1. **Set up LDAP/AD provider**
2. **Enable JIT provisioning**
3. **Configure trust settings** (email verification)
4. **Test with pilot group**
5. **Roll out to all users**
6. **Disable manual user creation**

## Troubleshooting

### Common Issues

#### LDAP Connection Failures

- Check LDAP server URL and port
- Verify SSL/TLS certificate
- Check bind DN and password
- Test network connectivity
- Review LDAP server logs

#### Sync Not Running

- Check federation config: `sync_enabled` must be true
- Verify `sync_interval_minutes` is set
- Check sync service initialization logs
- Look for sync job error logs

#### JIT Provisioning Failures

- Check `link_only` mode (should be false for JIT)
- Verify email uniqueness constraints
- Review user attribute mapping
- Check database constraints

#### Social Login Issues

- Verify OAuth2 client credentials
- Check redirect URI configuration
- Validate state parameter handling
- Review OAuth2 provider logs

## Best Practices

1. **Use LDAPS** (LDAP over SSL/TLS) for production
2. **Configure connection pooling** for performance
3. **Set appropriate sync intervals** (hourly is typical)
4. **Monitor sync job statistics** and failures
5. **Use link-only mode** for existing user bases
6. **Test in staging** before production rollout
7. **Keep provider credentials secure** (use Secreton)
8. **Enable audit logging** for compliance
9. **Configure appropriate batch sizes** for sync
10. **Use service accounts** with minimal LDAP permissions

## Performance Considerations

- **Connection Pooling**: Reuse LDAP connections
- **Batch Processing**: Sync users in configurable batches
- **Async Operations**: All federation ops are async
- **Caching**: Consider caching frequently accessed data
- **Database Indexes**: Ensure indexes on federated_user_id
- **Monitoring**: Track sync duration and optimize

## Compliance

- **GDPR**: User data minimization, right to erasure
- **FIPS**: Cryptographic operations comply with FIPS 140-2
- **Zero-Trust**: No implicit trust, continuous verification
- **Audit Trail**: Complete audit log for compliance
- **Data Retention**: Configurable retention policies

## Future Enhancements

### Planned Features

- [ ] SCIM 2.0 provisioning support
- [ ] Azure AD Graph API integration
- [ ] Okta federation
- [ ] SAML attribute mapping improvements
- [ ] Conditional access policies
- [ ] Multi-factor authentication integration
- [ ] Custom federation providers via plugins
- [ ] Advanced attribute transformation rules
- [ ] Bulk user import/export tools
- [ ] Federation analytics dashboard

## References

- [Keycloak User Federation Documentation](https://www.keycloak.org/docs/latest/server_admin/#_user-storage-federation)
- [LDAP RFC 4511](https://www.rfc-editor.org/rfc/rfc4511)
- [OAuth 2.0 RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [SAML 2.0 Specification](http://docs.oasis-open.org/security/saml/v2.0/)
