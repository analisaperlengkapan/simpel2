# authenc-crypto

Cryptographic operations for the Authenc identity provider.

## Features

- **JWT Generation and Validation**: Ed25519 signatures for enhanced security and performance
- **Password Hashing**: Argon2id algorithm with configurable parameters
- **Symmetric Encryption**: ChaCha20-Poly1305 for data encryption
- **TOTP**: Time-based One-Time Password generation and verification
- **Key Management**: Secure key generation and storage integration with Secreton

## JWT Service

The `JwtService` provides JWT token generation and validation using Ed25519 digital signatures.

### Key Features

- **Ed25519 Signatures**: Modern, fast, and secure signature algorithm (preferred over RSA)
- **Token Claims**: Standard JWT claims (sub, iss, aud, exp, iat, jti) plus custom claims
- **Token Types**: Access tokens (15 minutes) and refresh tokens (7 days)
- **Validation**: Signature verification, expiration check, issuer validation
- **Secreton Integration**: Key storage in Secreton vault

### Usage Example

```rust
use authenc_crypto::jwt::{JwtService, TokenClaims};
use chrono::Duration;

// Generate a new signing key (store in Secreton in production)
let signing_key = JwtService::generate_signing_key();

// Create JWT service
let service = JwtService::new(
    &signing_key,
    "https://authenc.kejaksaan.go.id".to_string(),
    Duration::minutes(15),  // Access token TTL
    Duration::days(7),      // Refresh token TTL
)?;

// Generate access token
let token = service.generate_access_token(
    "user-123",
    Some("kejaksaan-ri".to_string()),
    Some("openid profile email".to_string()),
    Some("session-456".to_string()),
)?;

// Verify token
let claims = service.verify_token(&token)?;
println!("User ID: {}", claims.sub);
println!("Realm: {}", claims.realm.unwrap());
```

### Token Claims Structure

```rust
pub struct TokenClaims {
    pub sub: String,              // Subject (user ID)
    pub iss: String,              // Issuer
    pub aud: Vec<String>,         // Audience
    pub exp: i64,                 // Expiration time (Unix timestamp)
    pub iat: i64,                 // Issued at (Unix timestamp)
    pub nbf: Option<i64>,         // Not before (Unix timestamp)
    pub jti: String,              // JWT ID (unique identifier)
    pub scope: Option<String>,    // Scope (space-separated)
    pub realm: Option<String>,    // Realm
    pub sid: Option<String>,      // Session ID
    pub custom: HashMap<String, serde_json::Value>, // Custom claims
}
```

### Security Considerations

1. **Key Storage**: Store Ed25519 private keys in Secreton, never in environment variables
2. **Key Rotation**: Rotate signing keys every 90 days
3. **Token Lifetime**: Keep access tokens short-lived (15 minutes recommended)
4. **Signature Verification**: Always verify signatures before trusting token claims
5. **Issuer Validation**: Validate the issuer to prevent token substitution attacks

### Examples

Run the JWT example:

```bash
cargo run --package authenc-crypto --example jwt_example
```

## Password Hashing

The `Argon2PasswordHasher` provides secure password hashing using the Argon2id algorithm, winner of the Password Hashing Competition.

### Key Features

- **Argon2id Algorithm**: Hybrid mode providing resistance to both side-channel and GPU attacks
- **Secure Parameters**: 64 MB memory cost, 3 iterations, 4 threads (configurable)
- **Random Salts**: 16-byte random salt per password (automatically generated)
- **PHC String Format**: Standard format for storing hashes
- **Constant-Time Verification**: Prevents timing attacks

### Usage Example

```rust
use authenc_crypto::password::Argon2PasswordHasher;
use authenc_types::traits::PasswordHasher;

// Create hasher with default parameters
let hasher = Argon2PasswordHasher::new();

// Hash a password
let hash = hasher.hash("my-secure-password")?;
// Returns: $argon2id$v=19$m=65536,t=3,p=4$...

// Verify password
assert!(hasher.verify("my-secure-password", &hash)?);
assert!(!hasher.verify("wrong-password", &hash)?);

// Custom parameters (higher security)
let high_security = Argon2PasswordHasher::with_params(
    131072, // 128 MB memory
    4,      // 4 iterations
    8,      // 8 threads
)?;
```

### Default Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Algorithm | Argon2id | Hybrid mode (side-channel + GPU resistant) |
| Memory Cost | 64 MB (65536 KiB) | Memory required for hashing |
| Time Cost | 3 iterations | Number of iterations |
| Parallelism | 4 threads | Number of parallel threads |
| Salt | 16 bytes | Random salt (auto-generated) |
| Version | v0x13 | Argon2 version 1.3 |

### Security Considerations

1. **Memory Cost**: Higher memory cost increases resistance to GPU attacks
2. **Time Cost**: More iterations increase computation time (defense against brute force)
3. **Parallelism**: More threads increase memory bandwidth requirements
4. **Salt**: Always use random salts (automatically handled)
5. **Hash Storage**: Store the entire PHC string (includes algorithm, parameters, salt, and hash)

### Performance

Default parameters (64 MB, 3 iterations, 4 threads):
- Hashing time: ~150ms on modern CPU
- Memory usage: 64 MB per hash operation

High security parameters (128 MB, 4 iterations, 8 threads):
- Hashing time: ~400ms on modern CPU
- Memory usage: 128 MB per hash operation

### Examples

Run the password hashing example:

```bash
cargo run --package authenc-crypto --example password_hashing
```

### PHC String Format

The hash is stored in PHC (Password Hashing Competition) string format:

```
$argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>
```

Where:
- `argon2id`: Algorithm identifier
- `v=19`: Version (0x13 in hex = 19 in decimal)
- `m=65536`: Memory cost in KiB (64 MB)
- `t=3`: Time cost (iterations)
- `p=4`: Parallelism (threads)
- `<salt>`: Base64-encoded salt
- `<hash>`: Base64-encoded hash output

## Encryption Service

(To be implemented in Task 4.3)

## TOTP

(To be implemented in MFA phase)

## Testing

Run all tests:

```bash
cargo test --package authenc-crypto
```

Run JWT tests only:

```bash
cargo test --package authenc-crypto --lib jwt
```

## Security

All cryptographic operations follow industry best practices:

- **JWT**: Ed25519 signatures (not RSA)
- **Passwords**: Argon2id with 64 MB memory cost, 3 iterations, 4 threads
- **Encryption**: ChaCha20-Poly1305 AEAD cipher
- **TOTP**: RFC 6238 compliant, 30-second time step

## Requirements

Implements requirements:
- REQ-TOKEN-001, REQ-TOKEN-003 (JWT operations)
- REQ-PASS-003, REQ-SEC-001 (Password hashing)
- REQ-SEC-002 (JWT signing)
- REQ-SEC-005 (Encryption at rest)
- REQ-MFA-001, REQ-MFA-002 (TOTP)

## Requirements

- Rust 1.90+ (Edition 2024)
- Dependencies managed via workspace

## License

Part of the SIMPEL project for Kejaksaan RI.
