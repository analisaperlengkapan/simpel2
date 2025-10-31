# Response Wrapping Guide

## Overview

Response wrapping provides a secure mechanism for distributing secrets using one-time tokens. Instead of returning sensitive data directly, Secreton can wrap the response in a time-limited, single-use token. This prevents secrets from being exposed in logs, command history, or unauthorized access.

## Features

- **One-Time Use**: Wrapped tokens can only be unwrapped once, after which they are automatically deleted
- **TTL-Based Expiration**: Tokens expire after a configurable time period (1 second to 24 hours)
- **Encrypted Storage**: Wrapped data is encrypted at rest using AES-256-GCM
- **Namespace Isolation**: Tokens are scoped to namespaces for multi-tenancy
- **Audit Logging**: All wrapping operations are logged for compliance
- **Rate Limiting**: Protection against abuse with configurable rate limits

## Use Cases

### 1. Secure Secret Distribution
Wrap secrets before sending them to users or applications:
```bash
# Wrap a database password
curl -X POST http://localhost:8200/v1/sys/wrapping/wrap \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "username": "dbuser",
      "password": "super-secret-password",
      "host": "db.example.com"
    },
    "ttl": 300
  }'

# Response:
{
  "success": true,
  "data": {
    "token": "wrap_abc123-def456-ghi789",
    "created_at": "2025-10-27T10:00:00Z",
    "expires_at": "2025-10-27T10:05:00Z",
    "ttl": 300,
    "accessor": "wrap_abc123-def456-ghi789_accessor"
  }
}
```

### 2. Temporary Access Tokens
Create short-lived tokens for temporary access:
```bash
# Wrap an API key with 60-second TTL
curl -X POST http://localhost:8200/v1/sys/wrapping/wrap \
  -H "Content-Type: application/json" \
  -d '{
    "data": {"api_key": "sk_live_abc123"},
    "ttl": 60
  }'
```

### 3. Secure CI/CD Pipelines
Distribute secrets to CI/CD jobs without exposing them in logs:
```bash
# In CI/CD script:
# 1. Request wrapped secret
TOKEN=$(curl -X POST http://localhost:8200/v1/sys/wrapping/wrap \
  -H "Content-Type: application/json" \
  -d '{"data": {"deploy_key": "..."}, "ttl": 300}' \
  | jq -r '.data.token')

# 2. Pass token to deployment job (safe to log)
echo "Deployment token: $TOKEN"

# 3. Unwrap in deployment job
curl -X POST http://localhost:8200/v1/sys/wrapping/unwrap \
  -H "Content-Type: application/json" \
  -d "{\"token\": \"$TOKEN\"}"
```

## REST API Endpoints

### Wrap Data
**Endpoint**: `POST /v1/sys/wrapping/wrap`

**Request**:
```json
{
  "data": {
    "key1": "value1",
    "key2": "value2"
  },
  "ttl": 300
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "token": "wrap_abc123...",
    "created_at": "2025-10-27T10:00:00Z",
    "expires_at": "2025-10-27T10:05:00Z",
    "ttl": 300,
    "accessor": "wrap_abc123..._accessor"
  }
}
```

**Parameters**:
- `data` (required): Any JSON object to wrap
- `ttl` (optional): Time-to-live in seconds (default: 300, max: 86400)

### Unwrap Token
**Endpoint**: `POST /v1/sys/wrapping/unwrap`

**Request**:
```json
{
  "token": "wrap_abc123..."
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "data": {
      "key1": "value1",
      "key2": "value2"
    },
    "created_at": "2025-10-27T10:00:00Z",
    "expired_at": "2025-10-27T10:05:00Z"
  }
}
```

**Errors**:
- `404 Not Found`: Token doesn't exist
- `400 Bad Request`: Token already unwrapped or expired
- `403 Forbidden`: Namespace mismatch

### Lookup Token Metadata
**Endpoint**: `GET /v1/sys/wrapping/lookup/{token}`

**Response**:
```json
{
  "success": true,
  "data": {
    "token": "wrap_abc123...",
    "created_at": "2025-10-27T10:00:00Z",
    "expires_at": "2025-10-27T10:05:00Z",
    "ttl_remaining": 245,
    "namespace": "default",
    "status": "Active",
    "data_size": 128
  }
}
```

**Status Values**:
- `Active`: Token can be unwrapped
- `Unwrapped`: Token has been used (one-time use enforced)
- `Expired`: Token has expired

### Rewrap Token
**Endpoint**: `POST /v1/sys/wrapping/rewrap`

**Request**:
```json
{
  "token": "wrap_abc123...",
  "ttl": 600
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "token": "wrap_xyz789...",
    "created_at": "2025-10-27T10:05:00Z",
    "expires_at": "2025-10-27T10:15:00Z",
    "ttl": 600,
    "accessor": "wrap_xyz789..._accessor"
  }
}
```

**Note**: Rewrapping unwraps the original token and creates a new one with a new TTL.

## gRPC API

### WrapData
```protobuf
rpc WrapData(WrapDataRequest) returns (WrapDataResponse);

message WrapDataRequest {
  string data_json = 1;  // JSON-encoded data to wrap
  uint64 ttl = 2;        // Time-to-live in seconds
  string namespace = 3;  // Namespace for isolation
}

message WrapDataResponse {
  string token = 1;           // Wrapping token
  int64 created_at = 2;       // Creation timestamp
  int64 expires_at = 3;       // Expiration timestamp
  int64 ttl = 4;              // TTL in seconds
  string accessor = 5;        // Accessor for lookup
}
```

### UnwrapToken
```protobuf
rpc UnwrapToken(UnwrapTokenRequest) returns (UnwrapTokenResponse);

message UnwrapTokenRequest {
  string token = 1;      // Wrapping token
  string namespace = 2;  // Namespace
}

message UnwrapTokenResponse {
  string data_json = 1;  // Original data (JSON)
  int64 created_at = 2;  // Creation timestamp
  int64 expired_at = 3;  // Expiration timestamp
}
```

### LookupWrappingToken
```protobuf
rpc LookupWrappingToken(LookupWrappingTokenRequest) returns (LookupWrappingTokenResponse);
```

### RewrapToken
```protobuf
rpc RewrapToken(RewrapTokenRequest) returns (RewrapTokenRespon
```

## Automatic Response Wrapping (Middleware)

**Note**: Automatic response wrapping via `X-Vault-Wrap-TTL` header is partially implemented. For production use, use the explicit `/v1/sys/wrapping/wrap` endpoint.

### Planned Feature
```bash
# Request with automatic wrapping
curl -H "X-Vault-Wrap-TTL: 300" \
  http://localhost:8200/v1/secret/data/myapp

# Response will be automatically wrapped
{
  "success": true,
  "data": {
    "token": "wrap_abc123...",
    "ttl": 300
  }
}
```

## Security Considerations

### One-Time Use Enforcement
- Tokens are automatically deleted after unwrapping
- Attempting to unwrap twice returns `TokenAlreadyUnwrapped` error
- This prevents replay attacks and unauthorized access

### TTL Limits
- Minimum TTL: 1 second
- Maximum TTL: 86400 seconds (24 hours)
- Expired tokens cannot be unwrapped
- Automatic cleanup removes expired tokens

### Encryption
- All wrapped data is encrypted using AES-256-GCM
- Encryption keys are generated per wrap operation
- Keys are stored securely in metadata (in production, use Transit engine or HSM)

### Namespace Isolation
- Tokens are scoped to namespaces
- Users can only unwrap tokens in their namespace
- Cross-namespace access returns `InvalidNamespace` error

### Rate Limiting
- Wrap operations are rate-limited per client
- Prevents abuse and DoS attacks
- Configurable limits per namespace

## Audit Logging

All wrapping operations are logged:

```json
{
  "timestamp": "2025-10-27T10:00:00Z",
  "operation": "wrap",
  "token": "wrap_abc123...",
  "namespace": "default",
  "ttl": 300,
  "data_size": 128,
  "user": "admin",
  "client_ip": "192.168.1.100"
}
```

## Monitoring

### Metrics
- `secreton_wrapping_wraps_total`: Total wrap operations
- `secreton_wrapping_unwraps_total`: Total unwrap operations
- `secreton_wrapping_lookups_total`: Total lookup operations
- `secreton_wrapping_rewraps_total`: Total rewrap operations
- `secreton_wrapping_expired_total`: Total expired tokens
- `secreton_wrapping_active_tokens`: Current active tokens

### Health Checks
- Monitor token expiration rate
- Alert on high unwrap failure rate
- Track average token lifetime

## Best Practices

### 1. Use Short TTLs
```bash
# Good: 5-minute TTL for temporary access
{"ttl": 300}

# Avoid: 24-hour TTL (increases exposure window)
{"ttl": 86400}
```

### 2. Verify Token Before Use
```bash
# Check token status before unwrapping
curl http://localhost:8200/v1/sys/wrapping/lookup/wrap_abc123...

# Only unwrap if status is "Active"
```

### 3. Handle Errors Gracefully
```bash
# Implement retry logic for expired tokens
if [ "$STATUS" == "Expired" ]; then
  echo "Token expired, requesting new one..."
  # Request new wrapped secret
fi
```

### 4. Clean Up Tokens
```bash
# Tokens are automatically cleaned up on unwrap
# No manual cleanup needed
```

### 5. Use Namespace Isolation
```bash
# Always specify namespace for multi-tenant deployments
{
  "data": {...},
  "ttl": 300,
  "namespace": "satker-kja001"
}
```

## Troubleshooting

### Token Not Found
**Error**: `404 Not Found: Wrapping token not found`

**Causes**:
- Token already unwrapped (one-time use)
- Token expired and cleaned up
- Invalid token format

**Solution**:
- Request a new wrapped secret
- Check token expiration time
- Verify token format starts with `wrap_`

### Token Already Unwrapped
**Error**: `400 Bad Request: Token has already been unwrapped`

**Causes**:
- Attempting to unwrap the same token twice
- Token was unwrapped by another process

**Solution**:
- Request a new wrapped secret
- Implement proper token lifecycle management

### Token Expired
**Error**: `400 Bad Request: Token expired at 2025-10-27T10:05:00Z`

**Causes**:
- TTL elapsed before unwrapping
- Clock skew between systems

**Solution**:
- Use shorter TTLs for time-sensitive operations
- Sync system clocks (NTP)
- Request new wrapped secret

### Invalid Namespace
**Error**: `403 Forbidden: Token belongs to namespace 'satker-a', not 'satker-b'`

**Causes**:
- Attempting to unwrap token from different namespace
- Namespace mismatch in JWT claims

**Solution**:
- Verify namespace in JWT token
- Request wrapped secret in correct namespace

## Database Schema

```sql
CREATE TABLE wrapping_tokens (
    token VARCHAR(255) PRIMARY KEY,
    encrypted_data BYTEA NOT NULL,
    encryption_metadata JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    namespace VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    data_size INTEGER NOT NULL,
    INDEX idx_expires_at (expires_at),
    INDEX idx_namespace (namespace),
    INDEX idx_status (status)
);
```

## Migration

```sql
-- Migration: Create wrapping_tokens table
-- File: migrations/XXX_create_wrapping_tokens.sql

CREATE TABLE IF NOT EXISTS wrapping_tokens (
    token VARCHAR(255) PRIMARY KEY,
    encrypted_data BYTEA NOT NULL,
    encryption_metadata JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    data_size INTEGER NOT NULL,

    CONSTRAINT wrapping_tokens_status_check
        CHECK (status IN ('Active', 'Unwrapped', 'Expired'))
);

CREATE INDEX idx_wrapping_tokens_expires_at ON wrapping_tokens(expires_at);
CREATE INDEX idx_wrapping_tokens_namespace ON wrapping_tokens(namespace);
CREATE INDEX idx_wrapping_tokens_status ON wrapping_tokens(status);

COMMENT ON TABLE wrapping_tokens IS 'One-time use tokens for secure secret distribution';
COMMENT ON COLUMN wrapping_tokens.token IS 'Unique wrapping token (format: wrap_<uuid>)';
COMMENT ON COLUMN wrapping_tokens.encrypted_data IS 'AES-256-GCM encrypted wrapped data';
COMMENT ON COLUMN wrapping_tokens.encryption_metadata IS 'Encryption algorithm, nonce, and key metadata';
COMMENT ON COLUMN wrapping_tokens.status IS 'Token status: Active, Unwrapped, or Expired';
```

## Configuration

```toml
[wrapping]
# Enable response wrapping
enabled = true

# Default TTL in seconds (5 minutes)
default_ttl = 300

# Maximum TTL in seconds (24 hours)
max_ttl = 86400

# Maximum data size in bytes (1MB)
max_data_size = 1048576

# Cleanup interval in seconds (60 seconds)
cleanup_interval = 60

# Enable automatic wrapping via X-Vault-Wrap-TTL header
auto_wrap_enabled = false
```

## Examples

### Python Client
```python
import requests
import json

# Wrap secret
response = requests.post(
    'http://localhost:8200/v1/sys/wrapping/wrap',
    json={
        'data': {'password': 'secret123'},
        'ttl': 300
    }
)
token = response.json()['data']['token']

# Unwrap secret
response = requests.post(
    'http://localhost:8200/v1/sys/wrapping/unwrap',
    json={'token': token}
)
secret = response.json()['data']['data']
print(f"Password: {secret['password']}")
```

### Go Client
```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

func main() {
    // Wrap secret
    wrapReq := map[string]interface{}{
        "data": map[string]string{"password": "secret123"},
        "ttl":  300,
    }
    wrapResp, _ := http.Post(
        "http://localhost:8200/v1/sys/wrapping/wrap",
        "application/json",
        bytes.NewBuffer(marshal(wrapReq)),
    )

    var wrapData map[string]interface{}
    json.NewDecoder(wrapResp.Body).Decode(&wrapData)
    token := wrapData["data"].(map[string]interface{})["token"].(string)

    // Unwrap secret
    unwrapReq := map[string]string{"token": token}
    unwrapResp, _ := http.Post(
        "http://localhost:8200/v1/sys/wrapping/unwrap",
        "application/json",
        bytes.NewBuffer(marshal(unwrapReq)),
    )
}
```

### Rust Client
```rust
use serde_json::json;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Wrap secret
    let wrap_response = client
        .post("http://localhost:8200/v1/sys/wrapping/wrap")
        .json(&json!({
            "data":{"password": "secret123"},
            "ttl": 300
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let token = wrap_response["data"]["token"].as_str().unwrap();

    // Unwrap secret
    let unwrap_response = client
        .post("http://localhost:8200/v1/sys/wrapping/unwrap")
        .json(&json!({"token": token}))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let secret = &unwrap_response["data"]["data"];
    println!("Password: {}", secret["password"]);

    Ok(())
}
```

## References

- [HashiCorp Vault Response Wrapping](https://www.vaultproject.io/docs/concepts/response-wrapping)
- [Secreton API Documentation](./API_DOCUMENTATION.md)
- [Security Best Practices](./SECURITY_BEST_PRACTICES.md)

