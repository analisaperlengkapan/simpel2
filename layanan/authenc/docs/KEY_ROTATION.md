# Automatic Key Rotation

## Overview

The automatic key rotation feature enhances security by periodically rotating cryptographic keys used in the Authenc system. This minimizes the impact of potential key compromise and follows security best practices.

## Features

- **Automatic Scheduling**: Keys are rotated automatically based on a configurable schedule (default: every 30 days)
- **Secreton Integration**: Integrates with Secreton for secure key management and rotation
- **Audit Logging**: All rotation events are logged to the database for compliance and monitoring
- **Notifications**: Optional notifications when key rotation completes
- **Grace Period**: Old keys remain valid for a configurable grace period to allow gradual transition
- **Multiple Key Types**: Supports rotation of different key types:
  - JWT signing keys (Ed25519)
  - Session encryption keys (AES-GCM)
  - MFA secret encryption keys
  - Database encryption keys

## Configuration

Add the following to your `config/authenc.production.toml`:

```toml
[key_rotation]
enabled = true
rotation_interval_days = 30
grace_period_days = 7
enable_notifications = true
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable/disable automatic key rotation |
| `rotation_interval_days` | integer | `30` | Number of days between rotations |
| `grace_period_days` | integer | `7` | Days old keys remain valid after rotation |
| `enable_notifications` | boolean | `true` | Send notifications on rotation completion |

## Environment Variables

You can also configure key rotation via environment variables:

```bash
KEY_ROTATION_ENABLED=true
KEY_ROTATION_INTERVAL_DAYS=30
KEY_ROTATION_GRACE_PERIOD_DAYS=7
KEY_ROTATION_NOTIFICATIONS=true
```

## Database Schema

The key rotation system uses the `key_rotation_audit` table to track all rotation events:

```sql
CREATE TABLE key_rotation_audit (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    key_id VARCHAR(255) NOT NULL,
    key_type VARCHAR(50) NOT NULL,
    old_version INTEGER NOT NULL,
    new_version INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    initiated_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Usage

### Registering Keys for Rotation

Keys must be registered with the rotation service to be automatically rotated:

```rust
use authenc::services::key_rotation::{KeyRotationService, KeyType};

// Register a JWT signing key
key_rotation_service.register_key(
    "jwt_signing_key_v1".to_string(),
    KeyType::JwtSigning,
    1, // current version
).await?;

// Register an MFA encryption key
key_rotation_service.register_key(
    "mfa_encryption_key_v1".to_string(),
    KeyType::MfaEncryption,
    1,
).await?;
```

### Manual Key Rotation

You can manually trigger key rotation for a specific key:

```rust
// Manually rotate a key
key_rotation_service.rotate_key(
    "jwt_signing_key_v1",
    &KeyType::JwtSigning
).await?;
```

### Viewing Rotation History

Query the rotation history for a specific key:

```rust
// Get last 10 rotation events for a key
let history = key_rotation_service.get_rotation_history(
    "jwt_signing_key_v1",
    10
).await?;

for event in history {
    println!("Rotation at {}: v{} -> v{} ({})",
        event.timestamp,
        event.old_version,
        event.new_version,
        event.status
    );
}
```

## Secreton Integration

The key rotation service integrates with Secreton's key management APIs:

### Signing Key Rotation

```
POST /v1/crypto/rotate-signing-key/{key_id}
```

Rotates a signing key and returns the new key material.

### Encryption Key Rotation

```
POST /v1/crypto/rotate-encryption-key/{key_id}
```

Rotates an encryption key and returns the new key material.

## Monitoring

### Metrics

The key rotation service exposes the following metrics:

- `authenc_key_rotation_total` - Total number of key rotations
- `authenc_key_rotation_failures` - Number of failed rotations
- `authenc_key_rotation_duration_seconds` - Duration of rotation operations

### Logs

Key rotation events are logged at INFO level:

```
INFO Starting rotation for key: jwt_signing_key_v1 (type: jwt_signing)
INFO Successfully rotated key jwt_signing_key_v1 from version 1 to 2
```

Failed rotations are logged at ERROR level:

```
ERROR Key rotation failed for jwt_signing_key_v1: Secreton unavailable
```

## Audit Trail

All rotation events are stored in the `key_rotation_audit` table with the following information:

- Event ID (UUID)
- Timestamp
- Key identifier
- Key type
- Old and new versions
- Status (Success, Failed, InProgress, Scheduled)
- Error message (if failed)
- Initiator (user or system)

Query rotation history:

```sql
SELECT * FROM key_rotation_audit
WHERE key_id = 'jwt_signing_key_v1'
ORDER BY timestamp DESC
LIMIT 10;
```

## Notifications

When `enable_notifications` is true, the system sends notifications on rotation completion. Notifications include:

- Key identifier
- Old and new versions
- Rotation status
- Timestamp

Notifications can be integrated with:

- Email
- Slack
- Microsoft Teams
- Custom webhooks

## Security Considerations

1. **Key Versioning**: Old keys are kept for the grace period to allow gradual transition
2. **Audit Logging**: All rotation events are logged for compliance
3. **Access Control**: Only system administrators can manually trigger rotations
4. **Secreton Integration**: Keys are managed securely by Secreton
5. **Circuit Breaker**: Rotation operations use circuit breaker pattern for reliability

## Troubleshooting

### Rotation Failures

If key rotation fails, check:

1. **Secreton Connectivity**: Ensure Secreton service is accessible
2. **Authentication**: Verify Secreton token is valid
3. **Permissions**: Check that the service account has rotation permissions
4. **Database**: Ensure audit table exists and is writable

### Viewing Failed Rotations

```sql
SELECT * FROM key_rotation_audit
WHERE status = 'Failed'
ORDER BY timestamp DESC;
```

### Manual Recovery

If automatic rotation fails, you can manually rotate keys:

```bash
# Using the admin API
curl -X POST https://authenc.example.com/admin/keys/rotate \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"key_id": "jwt_signing_key_v1", "key_type": "jwt_signing"}'
```

## Best Practices

1. **Regular Rotation**: Rotate keys every 30-90 days
2. **Grace Period**: Allow 7-14 days grace period for transition
3. **Monitoring**: Set up alerts for rotation failures
4. **Testing**: Test rotation in staging before production
5. **Backup**: Ensure Secreton has proper backup and recovery
6. **Documentation**: Document which keys are registered for rotation

## Compliance

The key rotation system helps meet compliance requirements:

- **ISO 27001**: Regular key rotation (A.10.1.2)
- **PCI DSS**: Cryptographic key management (Requirement 3.6)
- **NIST**: Key management best practices (SP 800-57)
- **GDPR**: Data protection by design (Article 25)

## References

- [NIST SP 800-57: Key Management](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [Secreton Documentation](../../../layanan/secreton/README.md)
- [Authenc Security Architecture](./SECURITY_ARCHITECTURE.md)
