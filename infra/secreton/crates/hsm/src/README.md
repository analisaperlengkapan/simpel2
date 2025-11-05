# HSM (Hardware Security Module) Integration

This module provides Hardware Security Module integration for Secreton, implementing the `HsmVault` trait from Authenc.

## Overview

The HSM module enables Secreton to leverage hardware-backed security for cryptographic operations, providing:

- **PKCS#11 Support**: Standard interface for HSM communication
- **Key Generation**: Generate keys directly in HSM
- **Cryptographic Operations**: Encryption, decryption, and signing using HSM keys
- **Health Monitoring**: HSM availability and health checks
- **Multiple Provider Support**: Extensible architecture for different HSM types

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    HsmBackend                           │
│  - Manages HSM lifecycle                                │
│  - Coordinates operations                               │
│  - Maintains key metadata                               │
└────────────────┬────────────────────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
┌───▼──────────────┐  ┌──────▼──────────────┐
│ Pkcs11Provider   │  │  Future Providers   │
│ - PKCS#11 HSM    │  │  - AWS KMS          │
│ - Smart Cards    │  │  - Azure Key Vault  │
│                  │  │  - GCP KMS          │
└──────────────────┘  └─────────────────────┘
```

## Components

### HsmBackend

Main interface for HSM operations. Implements the `HsmVault` trait from Authenc.

**Key Methods:**
- `initialize()` - Initialize HSM connection
- `generate_hsm_key()` - Generate key in HSM
- `hsm_sign()` - Sign data using HSM key
- `hsm_encrypt()` - Encrypt data using HSM key
- `hsm_decrypt()` - Decrypt data using HSM key
- `list_hsm_keys()` - List all HSM keys
- `delete_hsm_key()` - Delete HSM key
- `health_check()` - Check HSM health

### Pkcs11Provider

PKCS#11 implementation for HSM communication.

**Features:**
- Session management
- Authentication (PIN/password)
- Key generation and management
- Cryptographic operations
- Error handling and retry logic

### HsmConfig

Configuration for HSM integration.

**Settings:**
- `enabled` - Enable/disable HSM
- `provider` - HSM provider type (pkcs11, aws-kms, etc.)
- `pkcs11_library_path` - Path to PKCS#11 library
- `slot_id` - HSM slot ID
- `token_label` - HSM token label
- `pin` - HSM PIN/password
- `connection_timeout` - Connection timeout
- `operation_timeout` - Operation timeout
- `health_check_enabled` - Enable health checks
- `max_retries` - Maximum retry attempts

## Usage

### Configuration

```toml
[hsm]
enabled = true
provider = "pkcs11"
pkcs11_library_path = "/usr/lib/softhsm/libsofthsm2.so"
slot_id = 0
token_label = "secreton-token"
pin = "1234"
key_label_prefix = "secreton-"
connection_timeout = 30
operation_timeout = 60
health_check_enabled = true
health_check_interval = 60
max_retries = 3
retry_delay_ms = 1000
```

### Initialization

```rust
use secreton_core::hsm::{HsmBackend, HsmConfig};

// Load configuration
let config = HsmConfig {
    enabled: true,
    provider: HsmProvider::Pkcs11,
    pkcs11_library_path: Some(PathBuf::from("/usr/lib/softhsm/libsofthsm2.so")),
    slot_id: Some(0),
    pin: Some("1234".to_string()),
    ..Default::default()
};

// Create and initialize HSM backend
let hsm = HsmBackend::new(config)?;
hsm.initialize().await?;
```

### Key Generation

```rust
// Generate RSA key
let metadata = hsm.generate_hsm_key(
    "my-signing-key",
    "RSA",
    2048,
    vec!["sign".to_string(), "verify".to_string()]
).await?;

println!("Generated key: {}", metadata.key_id);
```

### Signing

```rust
// Sign data
let data = b"Hello, World!";
let signature = hsm.hsm_sign(
    "my-signing-key",
    data,
    "RSA-SHA256"
).await?;

println!("Signature: {:?}", signature);
```

### Encryption

```rust
// Encrypt data
let plaintext = b"Secret message";
let ciphertext = hsm.hsm_encrypt(
    "my-encryption-key",
    plaintext
).await?;

// Decrypt data
let decrypted = hsm.hsm_decrypt(
    "my-encryption-key",
    &ciphertext
).await?;

assert_eq!(plaintext, &decrypted[..]);
```

### Health Monitoring

```rust
// Check HSM health
let is_healthy = hsm.health_check().await?;
if !is_healthy {
    eprintln!("HSM is not healthy!");
}
```

## Supported Algorithms

### Symmetric Encryption
- AES-128-GCM
- AES-192-GCM
- AES-256-GCM

### Asymmetric Cryptography
- RSA-2048
- RSA-3072
- RSA-4096
- ECDSA-P256
- ECDSA-P384
- ECDSA-P521
- Ed25519

## Security Considerations

1. **Key Protection**: Keys never leave the HSM in plaintext
2. **PIN Security**: Store HSM PIN securely (environment variables, secrets manager)
3. **Access Control**: Restrict HSM access to authorized processes only
4. **Audit Logging**: All HSM operations should be audited
5. **FIPS Compliance**: Use FIPS 140-2 Level 3+ certified HSMs for production

## Error Handling

The module provides comprehensive error handling:

```rust
use secreton_core::hsm::HsmError;

match hsm.hsm_sign("key-id", data, "RSA-SHA256").await {
    Ok(signature) => println!("Signed successfully"),
    Err(HsmError::KeyNotFound(id)) => eprintln!("Key not found: {}", id),
    Err(HsmError::NotInitialized) => eprintln!("HSM not initialized"),
    Err(HsmError::SigningFailed(msg)) => eprintln!("Signing failed: {}", msg),
    Err(e) => eprintln!("HSM error: {}", e),
}
```

## Testing

### Unit Tests

```bash
cargo test --package secreton-core --lib hsm
```

### Integration Tests

```bash
# Requires SoftHSM or real HSM
cargo test --package secreton-core --test hsm_integration
```

## Future Enhancements

- [ ] AWS KMS provider implementation
- [ ] Azure Key Vault provider implementation
- [ ] GCP KMS provider implementation
- [ ] Key rotation support
- [ ] Backup and restore
- [ ] Multi-HSM support for redundancy
- [ ] Performance optimizations (connection pooling)

## References

- [PKCS#11 Specification](http://docs.oasis-open.org/pkcs11/pkcs11-base/v2.40/os/pkcs11-base-v2.40-os.html)
- [FIPS 140-2 Standard](https://csrc.nist.gov/publications/detail/fips/140/2/final)
- [Authenc HsmVault Trait](../../../authenc/src/vault/mod.rs)

## License

This module is part of Secreton and follows the same license as the parent project.
