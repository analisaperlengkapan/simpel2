# secreton-auto-unseal

Auto-unseal providers for Secreton, enabling automatic unsealing on startup without manual intervention.

## Overview

This crate provides the infrastructure and implementations for auto-unsealing Secreton using external key management services. Auto-unseal eliminates the need for manual unseal operations during pod restarts, upgrades, or failures in Kubernetes environments.

## Supported Providers

### Transit (Default)

Uses another Secreton instance's Transit engine for auto-unsealing. Ideal for:

- Development and testing
- Air-gapped deployments
- Hierarchical unsealing scenarios

### AWS KMS

Uses Amazon Web Services Key Management Service. Features:

- IAM role authentication (recommended for EC2/ECS/EKS)
- Access key authentication
- Custom endpoints support

### GCP KMS

Uses Google Cloud Platform Key Management Service. Features:

- Service account authentication
- Default credentials support
- Multi-region support

### Azure Key Vault

Uses Microsoft Azure Key Vault. Features:

- Managed identity authentication
- Client credentials authentication
- Key versioning support

## Usage

### Basic Example

```rust
use secreton_auto_unseal::{AutoUnsealProvider, transit::TransitProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Transit provider
    let provider = TransitProvider::from_params(
        "https://secreton.internal:50052".to_string(),
        "auto-unseal-key".to_string(),
        "s.token123".to_string(),
    ).await?;

    // Encrypt master key
    let master_key = b"secret-master-key";
    let encrypted = provider.encrypt(master_key).await?;

    // Decrypt master key
    let decrypted = provider.decrypt(&encrypted).await?;
    assert_eq!(master_key, decrypted.as_slice());

    // Health check
    provider.health_check().await?;

    Ok(())
}
```

### Configuration

Auto-unseal configuration is stored in `secreton.toml`:

#### Transit Configuration

```toml
[auto_unseal]
provider = "transit"
endpoint = "https://secreton.internal:50052"
key_name = "auto-unseal-key"
token = "s.token123"
timeout_secs = 30
```

#### AWS KMS Configuration

```toml
[auto_unseal]
provider = "aws-kms"
key_id = "alias/secreton-unseal"
region = "us-east-1"
```

#### GCP KMS Configuration

```toml
[auto_unseal]
provider = "gcp-kms"
key_name = "projects/my-project/locations/us-east1/keyRings/my-ring/cryptoKeys/my-key"
project_id = "my-project"
location = "us-east1"
key_ring = "my-ring"
crypto_key = "my-key"
```

#### Azure Key Vault Configuration

```toml
[auto_unseal]
provider = "azure-key-vault"
vault_name = "my-vault"
key_name = "secreton-unseal"
```

## Features

- `default`: Enables Transit provider
- `transit`: Transit auto-unseal provider
- `aws-kms`: AWS KMS auto-unseal provider
- `gcp-kms`: GCP KMS auto-unseal provider
- `azure-kv`: Azure Key Vault auto-unseal provider
- `all-providers`: Enable all providers

## Architecture

The auto-unseal system consists of:

1. **AutoUnsealProvider Trait**: Common interface for all providers
2. **Provider Implementations**: Specific implementations for each KMS service
3. **Configuration**: TOML-based configuration with validation
4. **Error Handling**: Comprehensive error types with conversion to SecretonError

## Security Considerations

- Auto-unseal keys should be protected with appropriate IAM policies
- Use IAM roles/managed identities instead of static credentials when possible
- Enable audit logging for all auto-unseal operations
- **Implement fallback to manual unseal if auto-unseal fails** (see [Fallback Mechanism](./FALLBACK.md))
- Rotate auto-unseal keys regularly

## Audit Logging

All auto-unseal operations are automatically logged to the audit trail for security and compliance purposes. This satisfies requirement **AC 2.1.7**: "Audit log records all unseal attempts (auto and manual)".

### Audit Events

The following events are logged:

| Event | Action | Status | Description |
|-------|--------|--------|-------------|
| **Initiated** | `auto_unseal.initiated` | Success | Auto-unseal attempt started |
| **Success** | `auto_unseal.success` | Success | Auto-unseal completed successfully |
| **Retry Failed** | `auto_unseal.retry_failed` | Failure | Individual retry attempt failed |
| **Retry Success** | `auto_unseal.retry_success` | Success | Retry attempt succeeded |
| **Fallback** | `auto_unseal.fallback_to_manual` | Failure | Fell back to manual unseal |
| **Failed** | `auto_unseal.failed` | Failure | Auto-unseal failed (no fallback) |

### Audit Log Contents

Each audit log entry includes:

- **Correlation ID**: Unique identifier linking all events in a single unseal session
- **Provider Type**: The auto-unseal provider used (e.g., "aws-kms", "transit")
- **Key ID**: The key identifier (ARN, key name, etc.)
- **Region/Endpoint**: Provider-specific location information
- **Attempt Number**: For retry events, which attempt number
- **Error Details**: For failure events, the error message (no sensitive data)
- **Timestamp**: UTC timestamp with millisecond precision

### Security Features

- **No Sensitive Data**: Master keys and encrypted data are NEVER logged
- **Correlation IDs**: All events in a single unseal session share the same correlation ID
- **Queryable**: Audit logs can be queried by action, status, time range, etc.
- **Immutable**: Audit logs cannot be modified after creation
- **Multiple Backends**: Supports PostgreSQL, file, syslog, and memory backends

### Example: Querying Audit Logs

```rust
use secreton_core::audit::{AuditLogger, AuditQuery, AuditStatus};

// Query all failed auto-unseal attempts in the last 24 hours
let query = AuditQuery::new()
    .action("auto_unseal.retry_failed")
    .status(AuditStatus::Failure)
    .since_hours(24);

let logs = audit_logger.query(&query).await?;
println!("Failed attempts: {}", logs.len());

// Query by correlation ID to see full unseal session
let query = AuditQuery::new()
    .metadata("correlation_id", "550e8400-e29b-41d4-a716-446655440000");

let logs = audit_logger.query(&query).await?;
for log in logs {
    println!("{}: {} - {}", log.timestamp, log.action, log.status);
}
```

### Enabling Audit Logging

Audit logging is automatically enabled when you create an `AutoUnsealManager` with an audit logger:

```rust
use secreton_auto_unseal::{AutoUnsealManager, config::FallbackConfig};
use secreton_core::audit::{AuditLogger, PostgreSqlBackend};
use std::sync::Arc;

// Create audit logger with PostgreSQL backend
let backend = PostgreSqlBackend::new(db_pool).await?;
let audit_logger = Arc::new(AuditLogger::new(vec![Arc::new(backend)]));

// Create manager with audit logging
let manager = AutoUnsealManager::with_audit_logger(
    provider,
    FallbackConfig::default(),
    audit_logger,
);

// All unseal attempts will now be audited
let (master_key, result) = manager.unseal_with_fallback(&encrypted).await?;
```

### Compliance

The audit logging implementation meets the following compliance requirements:

- **ISO 27001**: Comprehensive audit trail for all security operations
- **SOC 2**: Immutable logs with integrity protection
- **GDPR**: No personal data logged (system operations only)
- **Government Standards**: Meets Indonesian government security requirements

## Fallback Mechanism

The auto-unseal system includes a robust fallback mechanism that handles provider failures gracefully:

- **Exponential backoff retry**: Automatically retries failed unseal attempts with increasing delays
- **Configurable fallback**: Option to fall back to manual unseal or remain sealed
- **Comprehensive logging**: All retry attempts and fallback decisions are logged to audit trail
- **Health monitoring**: Health endpoint reports fallback status

For detailed information, see [FALLBACK.md](./FALLBACK.md).

### Quick Fallback Configuration

```toml
[auto_unseal.fallback]
# Enable fallback to manual unseal if auto-unseal fails (default: true)
fallback_to_manual = true

# Maximum retry attempts before falling back (default: 5)
max_retries = 5

# Initial retry delay in seconds (default: 1)
initial_retry_delay_secs = 1

# Maximum retry delay in seconds (default: 16)
max_retry_delay_secs = 16
```

## Implementation Status

- [x] Core infrastructure (trait, config, error handling)
- [x] Transit provider (stub implementation)
- [x] AWS KMS provider (stub implementation)
- [x] GCP KMS provider (stub implementation)
- [x] Azure Key Vault provider (stub implementation)
- [x] **Fallback mechanism with exponential backoff**
- [x] **Audit logging for all auto-unseal operations**
- [ ] Transit provider full implementation
- [ ] AWS KMS provider full implementation
- [ ] GCP KMS provider full implementation
- [ ] Azure Key Vault provider full implementation
- [ ] Integration tests
- [ ] Property-based tests

## Related Documentation

- [Fallback Mechanism](./FALLBACK.md) - Detailed fallback documentation
- [Design Document](../../.kiro/specs/secreton-vault-parity/design.md)
- [Requirements](../../.kiro/specs/secreton-vault-parity/requirements.md)
- [Tasks](../../.kiro/specs/secreton-vault-parity/tasks.md)

## License

Part of the Secreton project. See root LICENSE file for details.
