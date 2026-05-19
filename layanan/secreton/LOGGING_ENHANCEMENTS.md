# Logging Enhancements with Tracing

This document summarizes the logging enhancements made to the Secreton codebase as part of task 9.3.

## Overview

Enhanced logging with structured tracing across all core service modules using the `#[instrument]` macro. This provides:

- Automatic span creation for function execution
- Structured field logging (request_id, user_id, operation, namespace)
- Consistent logging patterns across all crates
- Appropriate log levels (ERROR, WARN, INFO, DEBUG, TRACE)

## Enhanced Services

### 1. Lease Service (`crates/core/src/services/lease.rs`)

**Functions Enhanced:**

- `create_lease` - Logs user, resource, resource_type, namespace, ttl_secs, operation
- `renew_lease` - Logs lease_id, increment, operation
- `revoke_lease` - Logs lease_id, operation
- `lookup_lease` - Logs lease_id, operation
- `list_leases` - Logs user_id, namespace, resource_type, status, operation
- `cleanup_expired` - Logs operation

**Structured Fields:**

```rust
#[instrument(skip(self, metadata), fields(
    user = %user,
    resource = %resource,
    resource_type = %resource_type,
    namespace = %namespace,
    ttl_secs = %ttl_secs,
    operation = "create_lease"
))]
```

### 2. Policy Service (`crates/core/src/services/policy.rs`)

**Functions Enhanced:**

- `evaluate_with_sentinel` - Logs user, path, action, policy_count, operation
- `check_policy_with_sentinel` - Logs user, path, action, sentinel_policy_count, rbac_role_count, operation

**Structured Fields:**

```rust
#[instrument(skip(sentinel_policies, context), fields(
    user = %user,
    path = %path,
    action = %action,
    policy_count = sentinel_policies.len(),
    operation = "evaluate_sentinel"
))]
```

### 3. MFA Service (`crates/core/src/services/mfa.rs`)

**Functions Enhanced:**

- `enable_totp` - Logs user_id, issuer, account_name, operation
- `verify_totp` - Logs user_id, operation (code is skipped for security)
- `verify_recovery_code` - Logs user_id, operation (code is skipped for security)
- `disable_totp` - Logs user_id, operation
- `regenerate_recovery_codes` - Logs user_id, operation

**Security Note:** Sensitive data like TOTP codes and recovery codes are explicitly skipped from logging.

### 4. Seal Service (`crates/core/src/services/seal.rs`)

**Functions Enhanced:**

- `initialize` - Logs operation
- `seal` - Logs operation
- `unseal_with_share` - Logs share_length, operation (share bytes skipped for security)
- `start_rekey` - Logs new_shares, new_threshold, operation
- `rotate_master_key` - Logs operation

**Security Note:** Share bytes and master keys are never logged.

### 5. Identity Service (`crates/core/src/services/identity.rs`)

**Functions Enhanced:**

- `create_entity` - Logs name, operation
- `delete_entity` - Logs entity_id, operation
- `create_alias` - Logs entity_id, mount_accessor, name, operation
- `merge_entities` - Logs from_entity_id, to_entity_id, operation

### 6. Token Service (`crates/core/src/services/token.rs`)

**Functions Enhanced:**

- `create_token` - Logs token_type, policy_count, ttl, display_name, operation
- `renew_token` - Logs increment, operation (token_value skipped for security)
- `revoke_token` - Logs operation (token_value skipped for security)
- `cleanup_expired` - Logs operation

**Security Note:** Token values are never logged.

### 7. Metrics Service (`crates/core/src/services/metrics.rs`)

**Functions Enhanced:**

- `register` - Logs metric_name, metric_type, operation
- `increment_counter` - Logs metric_name, delta, operation (level=trace)
- `set_gauge` - Logs metric_name, value, operation (level=trace)

**Note:** Metric operations use TRACE level to avoid excessive logging.

### 8. Health Service (`crates/core/src/services/health.rs`)

**Functions Enhanced:**

- `check_health` - Logs operation

### 9. Rate Limit Service (`crates/core/src/services/rate_limit.rs`)

**Functions Enhanced:**

- `check` - Logs key, operation (level=debug)

### 10. Key Manager Service (`crates/core/src/services/key_manager.rs`)

**Functions Enhanced:**

- `rotate_keys` - Logs operation

## Logging Patterns

### 1. Operation Field

Every instrumented function includes an `operation` field that identifies the operation being performed:

```rust
operation = "create_lease"
operation = "verify_totp"
operation = "seal_engine"
```

### 2. User/Entity Identification

Functions that operate on behalf of users include user identification:

```rust
user = %user
user_id = %user_id
entity_id = %entity_id
```

### 3. Resource Identification

Functions that operate on resources include resource identification:

```rust
resource = %resource
resource_type = %resource_type
namespace = %namespace
path = %path
```

### 4. Security-Sensitive Data

Sensitive data is explicitly skipped from logging:

```rust
#[instrument(skip(self, code), fields(...))]  // TOTP codes
#[instrument(skip(self, token_value), fields(...))]  // Token values
#[instrument(skip(self, share_bytes), fields(...))]  // Shamir shares
```

### 5. Log Levels

- **TRACE**: High-frequency operations (metrics updates)
- **DEBUG**: Detailed debugging information (rate limit checks)
- **INFO**: Normal operations (default for most functions)
- **WARN**: Warnings and degraded states
- **ERROR**: Error conditions

## Benefits

1. **Distributed Tracing**: Automatic span creation enables distributed tracing across service boundaries
2. **Structured Logging**: All logs include structured fields for easy parsing and analysis
3. **Performance Monitoring**: Automatic timing of function execution
4. **Security Auditing**: Comprehensive logging of security-sensitive operations
5. **Debugging**: Detailed context for troubleshooting issues
6. **Observability**: Integration with OpenTelemetry and other observability tools

## Usage Examples

### Viewing Logs

```bash
# Set log level
export RUST_LOG=secreton_core=debug

# Run with structured JSON output
RUST_LOG=secreton_core=info cargo run
```

### Log Output Example

```
2025-10-29T10:30:45.123Z INFO secreton_core::services::lease: create_lease{user="user123" resource="secret/data/foo" resource_type="kv" namespace="default" ttl_secs=3600 operation="create_lease"}
2025-10-29T10:30:45.456Z INFO secreton_core::services::mfa: verify_totp{user_id="user123" operation="verify_totp"}: verification successful
2025-10-29T10:30:45.789Z INFO secreton_core::services::seal: unseal_with_share{share_length=64 operation="unseal_with_share"}: threshold met, unsealing engine
```

## Integration with Monitoring

The enhanced logging integrates seamlessly with:

- **Prometheus**: Metrics collection via tracing-subscriber
- **Jaeger/Zipkin**: Distributed tracing via OpenTelemetry
- **Elasticsearch**: Log aggregation and analysis
- **Grafana**: Visualization and alerting

## Next Steps

1. Add request_id propagation across service boundaries
2. Implement correlation IDs for request tracking
3. Add custom metrics extraction from spans
4. Configure log sampling for high-volume operations
5. Set up alerting based on log patterns

## Compliance

This logging enhancement supports:

- **Requirement 1.8**: Structured logging with tracing
- **Requirement 7.1**: Audit logging for all operations
- **Requirement 9.1**: Monitoring and observability
- **Security**: No sensitive data in logs (tokens, keys, codes)
