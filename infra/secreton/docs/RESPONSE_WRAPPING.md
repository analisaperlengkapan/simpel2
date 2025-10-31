# Response Wrapping Service

## Overview

The Response Wrapping Service provides a one-time token mechanism for secure secret distribution. Wrapped responses can only be unwrapped once, preventing secret exposure in logs, command history, or unauthorized access.

## Features

- **One-Time Use**: Tokens are automatically deleted after unwrapping
- **TTL-Based Expiration**: Configurable time-to-live (1 second to 24 hours)
- **Encrypted Storage**: Data encrypted using AES-256-GCM before storage
- **Namespace Isolation**: Multi-tenancy support with namespace-based access control
- **Size Limits**: Maximum 1MB per wrapped response to prevent abuse
- **Automatic Cleanup**: Background task removes expired tokens
- **Comprehensive Audit**: All operations logged for compliance
- **Metrics**: Prometheus metrics for monitoring

## Use Cases

### 1. Secure Secret Distribution
```rust
// Wrap a database password for one-time retrieval
let request = WrapRequest {
    data: json!({"password": "secret123"}),
    ttl: Duration::from_secs(300), // 5 minutes
    namespace: "default".to_string(),
};
let response = service.wrap(request).await?;

// Share the token (not the password)
println!("Token: {}", response.token);

// Recipient unwraps once
let secret = service.unwrap(&response.token, "default").await?;
```

### 2. API Response Wrapping
Automatically wrap sensitive API responses using the `X-Vault-Wrap-TTL` header.

### 3. Temporary Credentials
Wrap dynamic database credentials for secure distribution to applications.

## API

### Wrap Data
```rust
pub async fn wrap(&self, request: WrapRequest) -> Result<WrapResponse, WrappingError>
```

### Unwrap Token (One-Time Use)
```rust
pub async fn unwrap(&self, token: &str, namespace: &str) -> Result<JsonValue, WrappingError>
```

### Lookup Metadata
```rust
pub async fn lookup(&self, token: &str, namespace: &str) -> Result<WrappedTokenInfo, WrappingError>
```

### Cleanup Expired Tokens
```rust
pub async fn cleanup_expired(&self) -> Result<usize, WrappingError>
```

## Database Schema

```sql
CREATE TABLE wrapping_tokens (
    token VARCHAR(255) PRIMARY KEY,
    encrypted_data BYTEA NOT NULL,
    encryption_metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    data_size INTEGER NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Configuration

- **MAX_WRAPPED_DATA_SIZE**: 1MB (1024 * 1024 bytes)
- **DEFAULT_TTL_SECONDS**: 300 seconds (5 minutes)
- **MAX_TTL_SECONDS**: 86400 seconds (24 hours)

## Security Considerations

1. **One-Time Use**: Tokens are deleted immediately after unwrapping
2. **Encryption**: All data encrypted at rest using AES-256-GCM
3. **TTL Enforcement**: Expired tokens cannot be unwrapped
4. **Namespace Isolation**: Cross-namespace access prevented
5. **Size Limits**: Prevents resource exhaustion attacks
6. **Audit Logging**: All operations logged for forensics

## Testing

Integration tests require PostgreSQL:

```bash
# Start test database
docker run -d --name secreton-test-db \
  -e POSTGRES_DB=secreton_test \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:15

# Run tests
cargo test --package secreton-core --test wrapping_tests -- --ignored --test-threads=1
```

## Metrics

- `wraps_created`: Total number of wrap operations
- `unwraps_successful`: Successful unwrap operations
- `unwraps_failed`: Failed unwrap attempts
- `tokens_expired`: Tokens expired via cleanup
- `active_tokens`: Current active token count

## Future Enhancements

- Integration with Transit engine for key management
- HSM support for encryption keys
- Response wrapping middleware for automatic API wrapping
- Rewrap operation for extending TTL
- Batch wrap/unwrap operations
