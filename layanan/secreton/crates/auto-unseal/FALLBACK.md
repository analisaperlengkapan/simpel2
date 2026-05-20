# Auto-Unseal Fallback Mechanism

## Overview

The auto-unseal fallback mechanism provides resilience when the configured auto-unseal provider (AWS KMS, GCP KMS, Azure Key Vault, or Transit) becomes unavailable or fails to decrypt the master key. This feature ensures that Secreton can gracefully handle provider failures without requiring immediate operator intervention.

## Features

### 1. Exponential Backoff Retry

When auto-unseal fails, the system automatically retries with exponential backoff:

- **Initial delay**: 1 second (configurable)
- **Backoff multiplier**: 2x per retry
- **Maximum delay**: 16 seconds (configurable)
- **Maximum retries**: 5 attempts (configurable)

**Example retry sequence:**

```
Attempt 1: Immediate
Attempt 2: Wait 1 second
Attempt 3: Wait 2 seconds
Attempt 4: Wait 4 seconds
Attempt 5: Wait 8 seconds
Attempt 6: Wait 16 seconds (capped at max_retry_delay_secs)
```

### 2. Configurable Fallback

The fallback behavior is fully configurable:

```toml
[auto_unseal.fallback]
# Enable fallback to manual unseal if auto-unseal fails
fallback_to_manual = true

# Maximum retry attempts before falling back (default: 5)
max_retries = 5

# Initial retry delay in seconds (default: 1)
initial_retry_delay_secs = 1

# Maximum retry delay in seconds (default: 16)
max_retry_delay_secs = 16
```

### 3. Fallback Modes

#### Mode 1: Fallback Enabled (Default)

When `fallback_to_manual = true`:

1. Auto-unseal attempts with retries
2. If all retries fail, transition to manual unseal mode
3. Secreton remains sealed but accepts manual unseal shares
4. Audit log records the fallback event
5. Health endpoint reports fallback status

**Use case**: Production environments where availability is critical and operators can manually unseal if needed.

#### Mode 2: Fallback Disabled

When `fallback_to_manual = false`:

1. Auto-unseal attempts with retries
2. If all retries fail, Secreton remains sealed
3. No manual unseal is accepted
4. Requires fixing the provider issue and restarting

**Use case**: High-security environments where manual unseal is not permitted and auto-unseal is mandatory.

## Configuration

### Complete Example

```toml
# secreton.toml

[auto_unseal]
provider = "aws-kms"
key_id = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
region = "us-east-1"

[auto_unseal.fallback]
fallback_to_manual = true
max_retries = 5
initial_retry_delay_secs = 1
max_retry_delay_secs = 16
```

### Environment Variables

```bash
# Fallback configuration
SECRETON_AUTO_UNSEAL_FALLBACK_TO_MANUAL=true
SECRETON_AUTO_UNSEAL_MAX_RETRIES=5
SECRETON_AUTO_UNSEAL_INITIAL_RETRY_DELAY_SECS=1
SECRETON_AUTO_UNSEAL_MAX_RETRY_DELAY_SECS=16
```

## Usage

### Programmatic Usage

```rust
use secreton_auto_unseal::{
    AutoUnsealManager, AutoUnsealProvider, FallbackConfig, UnsealResult,
};

// Create provider (e.g., AWS KMS)
let provider: Box<dyn AutoUnsealProvider> = create_aws_kms_provider(config).await?;

// Configure fallback
let fallback_config = FallbackConfig {
    fallback_to_manual: true,
    max_retries: 5,
    initial_retry_delay_secs: 1,
    max_retry_delay_secs: 16,
};

// Create manager
let manager = AutoUnsealManager::new(provider, fallback_config);

// Attempt unseal with fallback
let encrypted_master_key = load_encrypted_master_key().await?;

match manager.unseal_with_fallback(&encrypted_master_key).await {
    Ok((master_key, UnsealResult::Success)) => {
        // Auto-unseal succeeded
        initialize_crypto_engine(master_key).await?;
    }
    Ok((_, UnsealResult::FallbackToManual)) => {
        // Auto-unseal failed, fell back to manual unseal
        // Wait for operator to provide unseal shares
        wait_for_manual_unseal().await?;
    }
    Err(e) => {
        // Auto-unseal failed and fallback is disabled
        // Secreton remains sealed
        error!("Auto-unseal failed: {}", e);
    }
}
```

## Monitoring

### Audit Logs

All auto-unseal attempts and fallback events are logged to the audit trail:

```json
{
  "timestamp": "2026-02-18T10:30:00Z",
  "event": "auto_unseal_attempt",
  "provider": "aws-kms",
  "attempt": 1,
  "status": "failed",
  "error": "Network timeout"
}

{
  "timestamp": "2026-02-18T10:30:15Z",
  "event": "auto_unseal_fallback",
  "provider": "aws-kms",
  "total_attempts": 6,
  "fallback_mode": "manual_unseal"
}
```

### Metrics

Prometheus metrics are exposed for monitoring:

```
# Auto-unseal attempts
secreton_auto_unseal_attempts_total{provider="aws-kms",status="success"} 100
secreton_auto_unseal_attempts_total{provider="aws-kms",status="failed"} 5

# Auto-unseal fallback events
secreton_auto_unseal_fallback_total{provider="aws-kms"} 2

# Auto-unseal retry count
secreton_auto_unseal_retries_total{provider="aws-kms"} 15

# Auto-unseal duration
secreton_auto_unseal_duration_seconds{provider="aws-kms",quantile="0.5"} 0.1
secreton_auto_unseal_duration_seconds{provider="aws-kms",quantile="0.99"} 5.2
```

### Health Endpoint

The health endpoint reports auto-unseal status:

```json
{
  "sealed": true,
  "auto_unseal": {
    "enabled": true,
    "provider": "aws-kms",
    "status": "fallback_to_manual",
    "last_attempt": "2026-02-18T10:30:15Z",
    "retry_count": 6,
    "fallback_reason": "Provider unavailable after 6 attempts"
  }
}
```

## Troubleshooting

### Common Failure Scenarios

#### 1. Network Connectivity Issues

**Symptoms:**

- Auto-unseal fails with "Network timeout" or "Connection refused"
- Retries exhaust after configured attempts

**Resolution:**

1. Check network connectivity to KMS provider
2. Verify firewall rules allow outbound HTTPS
3. Check DNS resolution for provider endpoint
4. If fallback is enabled, manually unseal with shares

#### 2. Invalid Credentials

**Symptoms:**

- Auto-unseal fails with "Invalid credentials" or "Permission denied"
- Fails immediately without retries

**Resolution:**

1. Verify IAM role/service account has correct permissions
2. Check credentials are not expired
3. Verify key ID/ARN is correct
4. Update credentials and restart Secreton

#### 3. KMS Key Not Found

**Symptoms:**

- Auto-unseal fails with "Key not found"
- Fails immediately without retries

**Resolution:**

1. Verify key ID/ARN is correct
2. Check key exists in the specified region
3. Verify key is not deleted or disabled
4. Update configuration and restart Secreton

#### 4. Rate Limiting

**Symptoms:**

- Auto-unseal fails with "Rate limit exceeded"
- Retries may succeed after backoff

**Resolution:**

1. Increase retry delays to avoid rate limits
2. Request rate limit increase from provider
3. Consider using Transit provider for higher throughput

### Debugging

Enable debug logging to see detailed retry information:

```bash
RUST_LOG=secreton_auto_unseal=debug secreton
```

Example debug output:

```
[DEBUG] Starting auto-unseal with fallback (provider=aws-kms, max_retries=5)
[INFO] Attempting auto-unseal (attempt=1/6)
[ERROR] Auto-unseal attempt failed (attempt=1, error=Network timeout)
[WARN] Retrying auto-unseal after delay (attempt=1, delay_secs=1, remaining=5)
[INFO] Attempting auto-unseal (attempt=2/6)
[ERROR] Auto-unseal attempt failed (attempt=2, error=Network timeout)
[WARN] Retrying auto-unseal after delay (attempt=2, delay_secs=2, remaining=4)
...
[ERROR] All auto-unseal retry attempts exhausted
[WARN] Falling back to manual unseal mode
```

## Best Practices

### 1. Production Deployments

- **Enable fallback**: Set `fallback_to_manual = true` for production
- **Configure retries**: Use at least 5 retries with exponential backoff
- **Monitor metrics**: Alert on fallback events
- **Test failover**: Regularly test manual unseal procedures

### 2. High-Security Environments

- **Disable fallback**: Set `fallback_to_manual = false` if manual unseal is not permitted
- **Use HSM**: Consider HSM-backed KMS keys for higher availability
- **Redundant providers**: Deploy multiple Secreton instances with different providers

### 3. Development/Testing

- **Shorter delays**: Use smaller retry delays for faster feedback
- **Enable fallback**: Allow manual unseal for easier testing
- **Mock providers**: Use Transit provider pointing to local Secreton

## Security Considerations

### 1. Fallback Audit Trail

All fallback events are logged to the audit trail with:

- Timestamp of fallback decision
- Provider type and configuration
- Number of retry attempts
- Failure reason

This ensures compliance and forensic analysis capabilities.

### 2. No Credential Logging

The fallback mechanism never logs:

- Master key (encrypted or decrypted)
- Provider credentials (tokens, access keys)
- Unseal shares

Only metadata about the failure is logged.

### 3. Timing Attack Mitigation

Retry delays use constant-time operations to prevent timing attacks that could reveal information about the master key or provider state.

## References

- [Auto-Unseal Configuration](./README.md#configuration)
- [Provider Setup Guides](./README.md#providers)
- [Secreton Architecture](../../ARCHITECTURE.md)
- [Audit Logging](../../docs/AUDIT_LOGGING.md)
