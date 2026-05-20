# Lease Integration with Secret Engines

## Overview

This document describes how leases are integrated with secret engines in Secreton, enabling automatic expiration and revocation of secrets and dynamic credentials.

## Architecture

Leases provide time-bound access to secrets and credentials with the following features:

- **Automatic Expiration**: Secrets and credentials are automatically revoked when their lease expires
- **Renewal**: Leases can be renewed to extend access time (up to max_ttl)
- **Revocation**: Manual revocation of leases and cascading revocation of child leases
- **Audit Logging**: All lease operations are logged for compliance

## Integration Points

### 1. KV Secrets Engine

The KV secrets engine supports optional TTL for secrets, creating leases when specified.

#### Writing Secrets with TTL

```rust
use secreton_core::services::secrets::Kvv2Engine;
use secreton_core::services::lease::LeaseManager;

let kv_engine = Kvv2Engine::new();
let lease_manager = LeaseManager::new(pool);

// Write secret with 1 hour TTL
let (version, lease) = kv_engine.write_with_lease(
    "secret/database/password",
    data,
    None,                    // cas
    Some(3600),              // TTL in seconds
    "user1",                 // user
    "default",               // namespace
    &lease_manager,
).await?;

// Lease information
println!("Lease ID: {}", lease.unwrap().id);
println!("Expires at: {}", lease.unwrap().expired_at);
```

#### Reading Secrets with Lease Information

```rust
// Read secret and get associated lease
let (version, lease) = kv_engine.read_with_lease(
    "secret/database/password",
    None,                    // version (None = latest)
    &lease_manager,
).await?;

if let Some(lease) = lease {
    println!("Secret has lease: {}", lease.id);
    println!("TTL remaining: {} seconds", (lease.expired_at - Utc::now()).num_seconds());
}
```

#### Automatic Revocation

When a lease expires, the secret version is automatically soft-deleted:

```rust
// Called automatically by lease expiration scheduler
kv_engine.revoke_on_lease_expiry("secret/database/password", 1).await?;
```

### 2. Dynamic Secrets Engine (Database)

The database secrets engine **always** creates leases for generated credentials.

#### Generating Database Credentials

```rust
use secreton_core::services::secrets::DatabaseSecretsEngine;

let db_engine = DatabaseSecretsEngine::new();

// Generate credentials with lease (recommended method)
let (credentials, lease) = db_engine.generate_credentials_ensure_lease(
    "readonly",              // role name
    Some(3600),              // TTL in seconds
    &lease_manager,
    "user1",                 // user
).await?;

// Credentials information
println!("Username: {}", credentials.username);
println!("Password: {}", credentials.password);
println!("Lease ID: {}", lease.id);
println!("Expires at: {}", credentials.expires_at);
```

#### Renewing Credentials

```rust
// Renew both credentials and lease
let (renewed_creds, renewed_lease) = db_engine.renew_lease_with_manager(
    &credentials.id,
    &lease.id,
    1800,                    // increment in seconds
    &lease_manager,
).await?;

println!("New expiration: {}", renewed_creds.expires_at);
println!("Renew count: {}", renewed_lease.renew_count);
```

#### Revoking Credentials

```rust
// Revoke credentials and lease together
db_engine.revoke_credentials_with_lease(
    &credentials.id,
    &lease_manager,
    &lease.id,
).await?;

// Database user is dropped and lease is revoked
```

## Lease Lifecycle

### 1. Creation

When a secret or credential is created with a TTL:

1. Secret/credential is stored
2. Lease is created in the database
3. Lease metadata includes resource path and version
4. Audit log entry is created

### 2. Active State

While the lease is active:

- Secret/credential can be accessed
- Lease can be renewed (if renewable)
- Lease information is included in responses
- Lease appears in lease listings

### 3. Renewal

When a lease is renewed:

1. TTL is extended (up to max_ttl)
2. Renew count is incremented
3. Last renewed timestamp is updated
4. Audit log entry is created

### 4. Expiration

When a lease expires:

1. Lease expiration scheduler detects expired lease
2. Secret/credential is automatically revoked
3. Lease status is changed to "revoked"
4. Audit log entry is created
5. Child leases are also revoked (cascading)

### 5. Manual Revocation

Leases can be manually revoked:

1. User calls revoke endpoint
2. Secret/credential is revoked
3. Lease status is changed to "revoked"
4. Child leases are revoked (cascading)
5. Audit log entry is created

## Lease Expiration Scheduler

The lease expiration scheduler runs in the background to automatically expire and revoke leases.

### Configuration

```rust
use secreton_core::services::lease::{LeaseManager, LeaseSchedulerConfig};

let config = LeaseSchedulerConfig {
    check_interval_secs: 60,           // Check every 60 seconds
    notification_threshold_secs: 300,  // Notify 5 minutes before expiry
    enable_notifications: true,
};

let lease_manager = Arc::new(LeaseManager::with_config(pool, config));

// Start the scheduler
let scheduler_handle = lease_manager.clone().start_expiration_scheduler();

// Scheduler runs until handle is dropped or aborted
```

### Scheduler Operations

Every check interval, the scheduler:

1. Queries for expired leases
2. Revokes each expired lease (and children)
3. Queries for leases expiring soon
4. Sends notifications (if enabled)
5. Updates metrics

## Audit Logging

All lease operations are logged with comprehensive metadata.

### Lease Creation

```json
{
  "operation": "lease_creation",
  "lease_id": "lease-abc123",
  "lease_ttl": "3600",
  "lease_renewable": "true",
  "lease_expires_at": "2025-10-27T11:00:00Z",
  "lease_resource": "/secret/data/test",
  "lease_resource_type": "kv"
}
```

### Lease Renewal

```json
{
  "operation": "lease_renewal",
  "lease_id": "lease-abc123",
  "renewal_increment": "1800",
  "renew_count": "2",
  "last_renewed_at": "2025-10-27T10:30:00Z"
}
```

### Lease Revocation

```json
{
  "operation": "lease_revocation",
  "lease_id": "lease-abc123",
  "revocation_reason": "manual_revocation"
}
```

### Lease Expiration

```json
{
  "operation": "lease_expiration",
  "lease_id": "lease-abc123",
  "expired_at": "2025-10-27T11:00:00Z"
}
```

## API Response Format

### Secret Response with Lease

```json
{
  "request_id": "req_abc123",
  "data": {
    "password": "secret123"
  },
  "lease": {
    "lease_id": "lease-abc123",
    "ttl": 3600,
    "renewable": true,
    "expires_at": "2025-10-27T11:00:00Z",
    "renew_count": 0
  }
}
```

### Credentials Response with Lease

```json
{
  "request_id": "req_def456",
  "data": {
    "username": "v-readonly-abc123",
    "password": "generated-password",
    "connection_url": "postgresql://v-readonly-abc123:generated-password@localhost:5432/mydb"
  },
  "lease": {
    "lease_id": "lease-def456",
    "ttl": 3600,
    "renewable": true,
    "expires_at": "2025-10-27T11:00:00Z",
    "renew_count": 0
  }
}
```

## Best Practices

### 1. Always Use Leases for Dynamic Credentials

Dynamic credentials (database, cloud API keys) should always have leases:

```rust
// ✅ Good: Always creates lease
let (creds, lease) = db_engine.generate_credentials_ensure_lease(
    role_name, ttl, &lease_manager, user
).await?;

// ❌ Bad: No lease management
let creds = db_engine.generate_credentials(role_name, ttl).await?;
```

### 2. Set Appropriate TTLs

- Short-lived credentials: 1-4 hours
- Medium-lived secrets: 4-24 hours
- Long-lived secrets: Consider not using TTL

### 3. Enable Lease Renewal

Make leases renewable so users can extend access without regenerating:

```rust
let lease = lease_manager.create_lease(
    user, resource, resource_type, namespace,
    ttl, max_ttl,
    true,  // ✅ renewable = true
    None, None, None, metadata
).await?;
```

### 4. Monitor Lease Metrics

Track lease operations for capacity planning:

- `secreton_leases_expired_total` - Total expired leases
- `secreton_leases_revoked_total` - Total revoked leases
- `secreton_leases_active_total` - Currently active leases
- `secreton_leases_expiration_errors_total` - Expiration errors

### 5. Handle Lease Expiration Gracefully

Applications should handle lease expiration:

```rust
match kv_engine.read("secret/data/test", None).await {
    Ok(secret) => {
        // Use secret
    }
    Err(Kvv2Error::VersionDeleted(_, _)) => {
        // Lease expired, request new secret
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_lease_integration() {
    let kv = Kvv2Engine::new();
    let lease_manager = LeaseManager::new(pool);

    // Write with TTL
    let (version, lease) = kv.write_with_lease(
        "test/secret", data, None, Some(3600),
        "user1", "default", &lease_manager
    ).await.unwrap();

    assert!(lease.is_some());

    // Read with lease info
    let (_, lease) = kv.read_with_lease(
        "test/secret", None, &lease_manager
    ).await.unwrap();

    assert!(lease.is_some());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_lease_expiration_flow() {
    // Create lease with short TTL
    let (version, lease) = kv.write_with_lease(
        "test/secret", data, None, Some(2),  // 2 seconds
        "user1", "default", &lease_manager
    ).await.unwrap();

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Trigger cleanup
    lease_manager.cleanup_expired().await.unwrap();

    // Verify secret is deleted
    let result = kv.read("test/secret", None).await;
    assert!(result.is_err());
}
```

## Troubleshooting

### Lease Not Created

**Problem**: Secret created but no lease returned

**Solution**: Ensure TTL is specified and positive:

```rust
// ❌ No TTL specified
let (version, lease) = kv.write_with_lease(..., None, ...).await?;
assert!(lease.is_none());  // No lease created

// ✅ TTL specified
let (version, lease) = kv.write_with_lease(..., Some(3600), ...).await?;
assert!(lease.is_some());  // Lease created
```

### Lease Not Expiring

**Problem**: Expired leases not being revoked

**Solution**: Ensure expiration scheduler is running:

```rust
let lease_manager = Arc::new(LeaseManager::new(pool));
let _scheduler = lease_manager.clone().start_expiration_scheduler();

// Keep scheduler handle alive
```

### Renewal Fails

**Problem**: Lease renewal returns error

**Possible Causes**:

1. Lease already expired
2. Max renewals reached
3. New TTL exceeds max_ttl
4. Lease not renewable

**Solution**: Check lease status and limits:

```rust
let lease = lease_manager.lookup_lease(lease_id).await?;

if lease.status != "active" {
    // Lease expired or revoked
}

if let Some(max_renewals) = lease.max_renewals {
    if lease.renew_count >= max_renewals {
        // Max renewals reached
    }
}
```

## Migration Guide

### Existing Secrets Without Leases

Existing secrets without leases will continue to work. To add leases:

1. Read existing secret
2. Write with TTL to create new version with lease
3. Old versions remain without leases

```rust
// Read existing secret
let old_version = kv.read("secret/data/test", None).await?;

// Write new version with lease
let (new_version, lease) = kv.write_with_lease(
    "secret/data/test",
    old_version.data,
    None,
    Some(3600),  // Add TTL
    "user1",
    "default",
    &lease_manager
).await?;
```

### Existing Dynamic Credentials

For existing dynamic credentials without leases:

1. Revoke old credentials
2. Generate new credentials with lease

```rust
// Revoke old credentials
db_engine.revoke_credentials(old_cred_id).await?;

// Generate new with lease
let (creds, lease) = db_engine.generate_credentials_ensure_lease(
    role_name, ttl, &lease_manager, user
).await?;
```

## Conclusion

Lease integration provides automatic lifecycle management for secrets and credentials, ensuring:

- **Security**: Automatic revocation of expired credentials
- **Compliance**: Comprehensive audit logging
- **Convenience**: Renewable leases for extended access
- **Reliability**: Background scheduler for automatic cleanup

All new secret and credential operations should use lease integration for production deployments.
