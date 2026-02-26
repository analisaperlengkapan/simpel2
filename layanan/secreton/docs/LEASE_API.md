# Lease Management API

## Overview

The Lease Management API provides comprehensive lease lifecycle management for dynamic secrets and time-bound credentials. It supports automatic renewal, revocation cascading, and background cleanup with full integration for namespace isolation, authorization, audit logging, and monitoring.

## REST API Endpoints

### Renew Lease

Renew an existing lease to extend its expiration time.

**Endpoint:** `POST /v1/sys/leases/renew`

**Request Body:**
```json
{
  "lease_id": "lease-abc123",
  "increment": 3600
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "lease_id": "lease-abc123",
    "expired_at": "2025-10-22T10:00:00Z",
    "lease_duration": 7200,
    "renewable": true,
    "renew_count": 1,
    "max_renewals": null
  }
}
```

**Authorization:**
- User can renew their own leases
- Admin can renew any lease

**Audit:** Logs all renewal attempts (success and failure)

**Metrics:** Increments `secreton_lease_renewals_total` counter

---

### Revoke Lease

Revoke a lease and all its child leases (cascade).

**Endpoint:** `POST /v1/sys/leases/revoke`

**Request Body:**
```json
{
  "lease_id": "lease-abc123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "lease_id": "lease-abc123",
    "revoked_ids": ["lease-abc123", "lease-child1", "lease-child2"],
    "revoked_count": 3
  }
}
```

**Authorization:**
- User can revoke their own leases
- Admin can revoke any lease

**Audit:** Logs all revocation attempts and each child lease revocation

**Metrics:** Increments `secreton_lease_revocations_total` counter

---

### Revoke Leases by Prefix

Revoke all leases under a specific resource path prefix.

**Endpoint:** `POST /v1/sys/leases/revoke-prefix`

**Request Body:**
```json
{
  "prefix": "/secret/database/"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "prefix": "/secret/database/",
    "revoked_ids": ["lease-1", "lease-2", "lease-3"],
    "revoked_count": 3
  }
}
```

**Authorization:** Only admin users can revoke by prefix

**Audit:** Logs prefix revocation with all affected leases

**Metrics:** Increments `secreton_lease_prefix_revocations_total` counter

---

### Lookup Lease

Get detailed information about a specific lease.

**Endpoint:** `GET /v1/sys/leases/lookup/{lease_id}`

**Response:**
```json
{
  "success": true,
  "data": {
    "lease_id": "lease-abc123",
    "user": "user1",
    "resource": "/secret/database/prod",
    "resource_type": "database",
    "namespace": "satker-kja001",
    "issued_at": "2025-10-21T10:00:00Z",
    "expired_at": "2025-10-22T10:00:00Z",
    "ttl": 86400,
    "status": "active",
    "renewable": true,
    "max_ttl": 604800,
    "renew_count": 0,
    "max_renewals": null,
    "last_renewed_at": null,
    "parent_id": null,
    "child_ids": [],
    "metadata": {
      "created_by": "admin",
      "purpose": "production database access"
    }
  }
}
```

**Authorization:**
- User can lookup their own leases
- Admin can lookup any lease

**Audit:** Logs lease lookups

**Metrics:** Increments `secreton_lease_lookups_total` counter

---

### List Leases

List leases with filtering and pagination.

**Endpoint:** `GET /v1/sys/leases`

**Query Parameters:**
- `user_id` (optional): Filter by user ID
- `namespace` (optional): Filter by namespace
- `resource_type` (optional): Filter by resource type (e.g., "database", "kv")
- `status` (optional): Filter by status ("active", "revoked", "expired")
- `limit` (optional, default: 50): Maximum number of results
- `offset` (optional, default: 0): Pagination offset

**Response:**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "lease_id": "lease-abc123",
        "user": "user1",
        "resource": "/secret/database/prod",
        "resource_type": "database",
        "namespace": "satker-kja001",
        "issued_at": "2025-10-21T10:00:00Z",
        "expired_at": "2025-10-22T10:00:00Z",
        "ttl": 86400,
        "status": "active",
        "renewable": true,
        "max_ttl": 604800,
        "renew_count": 0,
        "max_renewals": null,
        "last_renewed_at": null,
        "parent_id": null,
        "child_ids": [],
        "metadata": {}
      }
    ],
    "pagination": {
      "total": 100,
      "limit": 50,
      "offset": 0,
      "has_next": true,
      "has_previous": false
    }
  }
}
```

**Authorization:**
- User can list their own leases in their namespace
- Admin can list all leases

**Audit:** Logs lease list operations

**Metrics:** Increments `secreton_lease_list_operations_total` counter

---

### Get Lease Statistics

Get aggregate statistics about leases.

**Endpoint:** `GET /v1/sys/leases/stats`

**Response:**
```json
{
  "success": true,
  "data": {
    "active_count": 150,
    "revoked_count": 25,
    "expired_count": 10,
    "expiring_soon_count": 5,
    "unique_users": 20,
    "unique_namespaces": 8,
    "by_resource_type": {
      "database": 100,
      "kv": 50
    },
    "by_namespace": {
      "satker-kja001": 75,
      "satker-kja002": 75
    }
  }
}
```

**Authorization:**
- User can see stats for their namespace
- Admin can see global stats

**Audit:** Logs stats requests

**Metrics:** Updates `secreton_active_leases_total` gauge

---

## gRPC API

All REST endpoints have corresponding gRPC methods defined in `secreton.proto`:

- `RenewLease(RenewLeaseRequest) returns (RenewLeaseResponse)`
- `RevokeLease(RevokeLeaseRequest) returns (RevokeLeaseResponse)`
- `RevokeLeasePrefix(RevokeLeasePrefixRequest) returns (RevokeLeasePrefixResponse)`
- `LookupLease(LookupLeaseRequest) returns (LookupLeaseResponse)`
- `ListLeases(ListLeasesRequest) returns (ListLeasesResponse)`
- `GetLeaseStats(GetLeaseStatsRequest) returns (GetLeaseStatsResponse)`

## Lease Information in Secret Responses

All secret responses that create leases include lease information:

```json
{
  "success": true,
  "data": {
    "username": "v-role-abc123",
    "password": "super-secret-password",
    "connection_url": "postgresql://...",
    "database": "mydb",
    "role": "readonly"
  },
  "lease_info": {
    "lease_id": "lease-abc123",
    "lease_duration": 3600,
    "renewable": true,
    "ttl": 3600,
    "expired_at": "2025-10-21T11:00:00Z"
  }
}
```

## Namespace Isolation

Leases are scoped to namespaces following the SIMKARI hierarchy:

- **Pusat (Central)**: Can access all leases across all namespaces
- **Wilayah (Regional)**: Can access leases in their region and child satkers
- **Satker (Unit)**: Can only access leases in their own namespace

Namespace is extracted from JWT claims (`satker_code`, `wilayah_code`, `admin_level`).

## Authorization

Authorization is enforced at multiple levels:

1. **Ownership**: Users can only manage their own leases
2. **Namespace**: Users can only see leases in their namespace (unless admin)
3. **Admin Privileges**: Admins can manage all leases
4. **Prefix Revocation**: Only admins can revoke by prefix

## Audit Logging

All lease operations are comprehensively logged:

- **Renewal**: Logs lease_id, increment, renew_count, new_expiration
- **Revocation**: Logs lease_id, revoked_count, revoked_ids (including children)
- **Prefix Revocation**: Logs prefix, revoked_count, all affected lease_ids
- **Lookup**: Logs lease_id
- **List**: Logs filters, pagination, result_count
- **Stats**: Logs active_count, revoked_count, expired_count

## Metrics

The following Prometheus metrics are exposed:

- `secreton_lease_renewals_total`: Counter of lease renewals
- `secreton_lease_revocations_total`: Counter of lease revocations
- `secreton_lease_prefix_revocations_total`: Counter of prefix revocations
- `secreton_lease_lookups_total`: Counter of lease lookups
- `secreton_lease_list_operations_total`: Counter of list operations
- `secreton_active_leases_total`: Gauge of currently active leases
- `secreton_leases_expired_total`: Counter of expired leases
- `secreton_leases_revoked_total`: Counter of revoked leases (by scheduler)
- `secreton_leases_expiration_notifications_total`: Counter of expiration notifications sent
- `secreton_leases_expiration_errors_total`: Counter of expiration check errors

## Automatic Expiration

The lease manager runs a background scheduler that:

1. Checks for expired leases every 60 seconds (configurable)
2. Automatically revokes expired leases
3. Sends notifications for leases expiring soon (within 5 minutes)
4. Cascades revocation to child leases
5. Executes revoke callbacks if configured
6. Updates metrics

## Error Handling

Comprehensive error handling with specific error codes:

- `LeaseNotFound`: Lease ID does not exist
- `LeaseExpired`: Lease has already expired
- `LeaseRevoked`: Lease has been revoked
- `RenewalNotAllowed`: Lease is not renewable or max renewals reached
- `InvalidTtl`: Invalid TTL or increment value
- `Forbidden`: User does not have permission
- `BadRequest`: Invalid request parameters

## Integration with Dynamic Secrets

Leases are automatically created when generating dynamic secrets:

```bash
# Generate database credentials (creates lease automatically)
curl -X GET http://localhost:8200/v1/dynamic/database/creds/readonly \
  -H "Authorization: Bearer $TOKEN"

# Response includes lease information
{
  "lease_id": "lease-abc123",
  "lease_duration": 3600,
  "renewable": true,
  "data": {
    "username": "v-readonly-abc123",
    "password": "...",
    ...
  }
}

# Renew the lease
curl -X POST http://localhost:8200/v1/sys/leases/renew \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"lease_id": "lease-abc123", "increment": 3600}'

# Revoke the lease (credentials are deleted)
curl -X POST http://localhost:8200/v1/sys/leases/revoke \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"lease_id": "lease-abc123"}'
```

## Database Schema

Leases are stored in PostgreSQL with the following schema:

```sql
CREATE TABLE leases (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    resource TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    namespace TEXT NOT NULL,
    status TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    expired_at TIMESTAMPTZ NOT NULL,
    last_renewed_at TIMESTAMPTZ,
    renewable BOOLEAN NOT NULL DEFAULT true,
    max_ttl BIGINT NOT NULL,
    renew_count INTEGER NOT NULL DEFAULT 0,
    max_renewals INTEGER,
    parent_id TEXT REFERENCES leases(id),
    revoke_callback TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_leases_user_id ON leases(user_id);
CREATE INDEX idx_leases_namespace ON leases(namespace);
CREATE INDEX idx_leases_status ON leases(status);
CREATE INDEX idx_leases_expired_at ON leases(expired_at);
CREATE INDEX idx_leases_resource ON leases(resource);
```

## Configuration

Lease scheduler configuration:

```toml
[lease_scheduler]
# Check interval in seconds
check_interval_secs = 60

# Notification threshold in seconds (5 minutes)
notification_threshold_secs = 300

# Enable expiration notifications
enable_notifications = true
```

## Testing

Integration tests are provided in `crates/api/tests/lease_integration_tests.rs`:

```bash
# Run lease integration tests (requires PostgreSQL)
cargo test --test lease_integration_tests -- --ignored

# Run all tests
cargo test
```

## Production Deployment

### Prerequisites

1. PostgreSQL database with leases table created
2. Proper JWT authentication configured
3. Namespace hierarchy configured
4. Audit logging enabled

### Startup

The lease manager automatically:
1. Loads existing leases from database
2. Starts the expiration scheduler
3. Begins monitoring for expired leases

### Monitoring

Monitor these metrics in Grafana:

- Active leases count
- Lease renewal rate
- Lease revocation rate
- Expiration check errors
- Average lease TTL

### Troubleshooting

**Leases not expiring:**
- Check scheduler is running: Look for "Starting lease expiration scheduler" in logs
- Check database connectivity
- Verify expired_at timestamps are correct

**High revocation rate:**
- Check if leases are being created with appropriate TTLs
- Verify renewal logic is working
- Check for application errors preventing renewal

**Permission errors:**
- Verify JWT claims include correct namespace information
- Check admin_level is set correctly
- Verify namespace hierarchy is configured

## Security Considerations

1. **Namespace Isolation**: Strictly enforced at API layer
2. **Authorization**: Multi-level checks (ownership, namespace, admin)
3. **Audit Logging**: All operations logged with full context
4. **Automatic Cleanup**: Expired leases automatically revoked
5. **Cascade Revocation**: Child leases revoked with parent
6. **Rate Limiting**: Prevents abuse of lease operations
7. **Input Validation**: All parameters validated before processing

## Future Enhancements

- Webhook support for lease expiration notifications
- Lease templates for common patterns
- Lease usage analytics and reporting
- Automatic lease renewal based on usage patterns
- Lease quotas per namespace
- Lease priority levels
- Batch lease operations
