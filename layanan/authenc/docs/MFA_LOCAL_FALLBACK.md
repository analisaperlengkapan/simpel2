# MFA Local Encrypted Storage Fallback

## Overview

The MFA local encrypted storage fallback provides a resilient mechanism for storing MFA secrets when the primary Secreton service is unavailable. This ensures that authentication can continue even during Secreton outages, with automatic synchronization when the service recovers.

## Features

- **AES-256-GCM Encryption**: All MFA secrets are encrypted at rest using AES-256-GCM
- **Automatic Fallback**: Seamlessly switches to local storage when Secreton is unavailable
- **Automatic Sync**: Background task automatically syncs secrets to Secreton when it recovers
- **Degraded Mode Detection**: Logs warnings when operating in degraded mode
- **Metrics Collection**: Tracks fallback usage for monitoring and alerting
- **Zero Data Loss**: Ensures no MFA secrets are lost during Secreton outages

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MFA Service Layer                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  MfaFallbackClient                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Try Secreton First                                  │   │
│  │  ├─ Success → Store in local cache                   │   │
│  │  └─ Failure → Use local storage                      │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                                    │
         ▼                                    ▼
┌──────────────────┐              ┌──────────────────────┐
│ SecretonClient   │              │ MfaLocalStorage      │
│ (Primary)        │              │ (Fallback)           │
│                  │              │                      │
│ - gRPC API       │              │ - AES-256-GCM        │
│ - Circuit Breaker│              │ - Encrypted File     │
│ - Retry Logic    │              │ - Sync Tracking      │
└──────────────────┘              └──────────────────────┘
         │                                    │
         ▼                                    ▼
┌──────────────────┐              ┌──────────────────────┐
│ Secreton Service │              │ Local Disk Storage   │
│ (Remote)         │              │ (Encrypted)          │
└──────────────────┘              └──────────────────────┘
```

## Configuration

### Environment Variables

```bash
# Enable local storage fallback (default: true)
MFA_FALLBACK_ENABLED=true

# Path to encrypted storage file (default: /var/lib/authenc/mfa_storage.enc)
MFA_FALLBACK_STORAGE_PATH=/var/lib/authenc/mfa_storage.enc

# Base64-encoded 32-byte encryption key (auto-generated if not provided)
MFA_FALLBACK_ENCRYPTION_KEY=<base64-encoded-key>

# Sync interval in seconds (default: 60)
MFA_FALLBACK_SYNC_INTERVAL=60

# Maximum sync attempts before giving up (default: 10)
MFA_FALLBACK_MAX_SYNC_ATTEMPTS=10

# Enable automatic sync when Secreton recovers (default: true)
MFA_FALLBACK_AUTO_SYNC=true
```

### Configuration File (TOML)

```toml
[mfa_fallback]
enabled = true
storage_path = "/var/lib/authenc/mfa_storage.enc"
encryption_key = "<base64-encoded-key>"
sync_interval_seconds = 60
max_sync_attempts = 10
auto_sync_enabled = true
log_degraded_warnings = true
```

## Usage

### Initialization

```rust
use authenc::services::mfa_local_storage::MfaLocalStorage;
use authenc::services::mfa_fallback_client::MfaFallbackClient;
use authenc::vault::secreton_client::SecretonClient;
use authenc::config::MfaFallbackConfig;
use std::sync::Arc;

// Load configuration
let config = MfaFallbackConfig::from_env();
let encryption_key = config.get_encryption_key()?;

// Create Secreton client
let secreton = Arc::new(SecretonClient::new(
    "https://secreton.example.com".to_string(),
    "your-auth-token".to_string(),
));

// Create local storage
let local_storage = Arc::new(MfaLocalStorage::new(
    config.storage_path.clone(),
    &encryption_key,
)?);

// Create fallback client
let fallback_client = Arc::new(MfaFallbackClient::new(
    secreton,
    local_storage,
));

// Initialize (loads existing secrets and starts sync task)
fallback_client.initialize().await?;
```

### Using with MFA Service

```rust
use authenc::services::mfa_service::MfaService;

// Create MFA service with fallback client
let mfa_service = MfaService::new(
    fallback_client, // Implements MfaClient trait
    db_pool,
);

// Use normally - fallback is automatic
let setup_data = mfa_service.setup_mfa(user_id).await?;
```

## Degraded Mode

When Secreton is unavailable, the system automatically enters degraded mode:

1. **Detection**: Circuit breaker detects Secreton failures
2. **Fallback**: Switches to local encrypted storage
3. **Warning Logs**: Logs degraded mode entry with timestamp
4. **Metrics**: Updates degraded mode metrics for monitoring
5. **Recovery**: Background task attempts to reconnect to Secreton

### Degraded Mode Logs

```
WARN Entered degraded mode - using local encrypted storage for MFA secrets
INFO Secreton is available again, attempting to sync secrets
INFO Successfully synced 5 secrets to Secreton
INFO Exited degraded mode - Secreton is now available
```

## Automatic Synchronization

The fallback client runs a background task that:

1. Checks Secreton availability every 60 seconds (configurable)
2. When Secreton recovers, enters "recovering" mode
3. Syncs all pending secrets to Secreton
4. Marks successfully synced secrets
5. Exits degraded mode when all secrets are synced

### Sync Process

```
┌─────────────────────────────────────────────────────────┐
│ Background Sync Task (runs every 60s)                   │
└─────────────────────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ Check Degraded Mode  │
         └──────────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ Health Check Secreton│
         └──────────────────────┘
                    │
         ┌──────────┴──────────┐
         │                     │
    Available            Unavailable
         │                     │
         ▼                     ▼
┌─────────────────┐    ┌──────────────┐
│ Enter Recovering│    │ Stay Degraded│
│ Mode            │    └──────────────┘
└─────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ Get Secrets Needing Sync            │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ For Each Secret:                    │
│  1. Retrieve from local storage     │
│  2. Store in Secreton               │
│  3. Mark as synced on success       │
│  4. Increment attempts on failure   │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ All Synced? → Exit Degraded Mode    │
│ Partial? → Stay in Recovering Mode  │
└─────────────────────────────────────┘
```

## Metrics

The fallback system exposes the following metrics:

```rust
pub struct LocalStorageMetrics {
    /// Number of secrets stored locally
    pub secrets_stored: u64,

    /// Number of secrets retrieved from local storage
    pub secrets_retrieved: u64,

    /// Number of successful syncs to Secreton
    pub successful_syncs: u64,

    /// Number of failed syncs to Secreton
    pub failed_syncs: u64,

    /// Time spent in degraded mode (seconds)
    pub degraded_mode_duration: u64,

    /// Last degraded mode entry time
    pub last_degraded_entry: Option<DateTime<Utc>>,
}
```

### Prometheus Metrics

```prometheus
# Number of MFA secrets stored in local fallback
authenc_mfa_local_storage_secrets_total

# Number of MFA secrets retrieved from local fallback
authenc_mfa_local_storage_retrievals_total

# Number of successful syncs to Secreton
authenc_mfa_local_storage_syncs_success_total

# Number of failed syncs to Secreton
authenc_mfa_local_storage_syncs_failed_total

# Time spent in degraded mode (seconds)
authenc_mfa_local_storage_degraded_duration_seconds

# Current degraded mode status (0=normal, 1=degraded, 2=recovering)
authenc_mfa_local_storage_degraded_mode
```

## Security Considerations

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Size**: 256 bits (32 bytes)
- **Nonce**: 96 bits (12 bytes), randomly generated per encryption
- **Authentication Tag**: 128 bits (16 bytes)

### Key Management

1. **Generation**: Keys are generated using cryptographically secure random number generator
2. **Storage**: Keys should be stored in environment variables or secure key management systems
3. **Rotation**: Keys can be rotated by re-encrypting all secrets with a new key
4. **Access**: Only the Authenc process should have access to the encryption key

### File Permissions

The encrypted storage file should have restricted permissions:

```bash
# Set ownership to authenc user
chown authenc:authenc /var/lib/authenc/mfa_storage.enc

# Set permissions to read/write for owner only
chmod 600 /var/lib/authenc/mfa_storage.enc
```

### Audit Logging

All operations are logged for audit purposes:

- Secret storage events
- Secret retrieval events
- Degraded mode transitions
- Sync attempts and results
- Encryption/decryption operations

## Monitoring and Alerting

### Recommended Alerts

1. **Degraded Mode Alert**

   ```yaml
   alert: MfaFallbackDegradedMode
   expr: authenc_mfa_local_storage_degraded_mode > 0
   for: 5m
   annotations:
     summary: "MFA fallback is in degraded mode"
     description: "Secreton is unavailable, using local storage"
   ```

2. **Sync Failure Alert**

   ```yaml
   alert: MfaFallbackSyncFailures
   expr: rate(authenc_mfa_local_storage_syncs_failed_total[5m]) > 0
   for: 10m
   annotations:
     summary: "MFA fallback sync failures detected"
     description: "Failed to sync secrets to Secreton"
   ```

3. **Extended Degraded Mode Alert**

   ```yaml
   alert: MfaFallbackExtendedDegradedMode
   expr: authenc_mfa_local_storage_degraded_duration_seconds > 3600
   annotations:
     summary: "MFA fallback in degraded mode for over 1 hour"
     description: "Secreton has been unavailable for extended period"
   ```

## Troubleshooting

### Issue: Secrets not syncing to Secreton

**Symptoms**: Degraded mode persists, sync failures in logs

**Solutions**:

1. Check Secreton connectivity: `curl https://secreton.example.com/v1/health`
2. Verify authentication token is valid
3. Check network connectivity and firewall rules
4. Review Secreton logs for errors
5. Increase sync interval if rate limited

### Issue: Encryption key errors

**Symptoms**: "Invalid encryption key" errors, decryption failures

**Solutions**:

1. Verify encryption key is 32 bytes (256 bits)
2. Check base64 encoding is correct
3. Ensure key hasn't changed since secrets were encrypted
4. If key is lost, secrets cannot be recovered (regenerate MFA for affected users)

### Issue: Storage file corruption

**Symptoms**: "Failed to load local MFA storage" errors

**Solutions**:

1. Check file permissions (should be 600)
2. Verify file is not corrupted: `file /var/lib/authenc/mfa_storage.enc`
3. Check disk space: `df -h /var/lib/authenc`
4. Restore from backup if available
5. If unrecoverable, delete file and regenerate MFA for affected users

## Best Practices

1. **Regular Backups**: Backup the encrypted storage file regularly
2. **Key Rotation**: Rotate encryption keys periodically (e.g., every 90 days)
3. **Monitoring**: Set up alerts for degraded mode and sync failures
4. **Testing**: Regularly test fallback by simulating Secreton outages
5. **Documentation**: Document key management procedures
6. **Capacity Planning**: Monitor storage file size and plan for growth
7. **Security Audits**: Regularly audit access to encryption keys and storage files

## Performance Impact

The fallback mechanism has minimal performance impact:

- **Storage Overhead**: ~1KB per MFA secret (encrypted)
- **Memory Overhead**: Secrets cached in memory (configurable)
- **CPU Overhead**: AES-GCM encryption/decryption is fast (~1ms per operation)
- **Disk I/O**: Writes are batched and asynchronous
- **Network**: No additional network calls when Secreton is available

## Compliance

The local encrypted storage fallback maintains compliance with:

- **GDPR**: Encryption at rest, audit logging, right to erasure
- **ISO 27001**: Access control, encryption, audit trails
- **PCI DSS**: Strong cryptography, key management, audit logging
- **NIST**: FIPS 140-2 compliant encryption algorithms

## Future Enhancements

Planned improvements for future releases:

1. **Key Rotation**: Automatic key rotation with re-encryption
2. **Compression**: Compress encrypted data to reduce storage
3. **Replication**: Replicate encrypted storage across nodes
4. **HSM Integration**: Store encryption keys in Hardware Security Module
5. **Backup Automation**: Automatic encrypted backups to S3/MinIO
6. **Recovery Codes**: Enhanced recovery code management
7. **WebAuthn Support**: Extend fallback to WebAuthn credentials

## References

- [AES-GCM Specification](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf)
- [TOTP RFC 6238](https://tools.ietf.org/html/rfc6238)
- [Secreton Documentation](../../../layanan/secreton/README.md)
- [MFA Service Documentation](./MFA_SERVICE.md)
