# Design Document: Secreton Comprehensive Enhancement

## Overview

This design document outlines the technical approach for enhancing Secreton, the enterprise secrets management system for Kejaksaan Agung RI. The enhancements build upon the existing architecture (~87,000 lines of Rust code) organized in 9 crates following hexagonal architecture principles.

### Existing Implementation Analysis

Based on codebase review, the following components are already implemented:

**Crypto Layer (`secreton-crypto`)**:
- Shamir Secret Sharing with Feldman VSS (`shamir.rs` - 1,214 lines)
- Key derivation: PBKDF2, Argon2id, HKDF-like (`key_derivation.rs`)
- HMAC-SHA256 (`hashing.rs`)
- FPE/FF3-1 for tokenization (`fpe.rs`)
- Transit engine with batch operations (`transit/`)
- Post-quantum: ML-KEM, ML-DSA, Falcon (`pqc/`)

**Core Services (`secreton-core`)**:
- Seal/Unseal with Shamir integration (`services/seal.rs` - 1,323 lines)
- Key rotation scheduler (`services/key_manager.rs`)
- LRU Cache with TTL and sensitivity levels (`utils/cache.rs`)
- Lease management (`services/lease.rs`)
- PKI engine (`services/secrets/pki.rs`)
- SSH engine (`services/secrets/ssh.rs`)
- Transform engine (`services/secrets/transform.rs`)
- Dynamic secrets for PostgreSQL (`services/secrets/database.rs`)
- MFA/TOTP (`services/mfa.rs`)
- Rate limiting (in MFA config)

**API Layer (`secreton-api`)**:
- REST handlers for all engines
- gRPC server (`secreton-grpc`)

### Enhancement Scope

The enhancements focus on:
1. **Hierarchical Key Management** - Explicit MK → KEK → DEK with HKDF derivation (enhance existing)
2. **Zero-Knowledge E2EE** - Client-side encryption support (NEW)
3. **Cryptographic Services API** - REST endpoints for HMAC, RNG, re-encrypt (enhance existing)
4. **Dynamic Secrets Expansion** - MySQL, MongoDB, Redis, K8s (NEW engines)
5. **Infrastructure Integration** - K8s operator, Terraform provider (NEW)
6. **PKI/SSH Enhancements** - OCSP, auto-renewal, templates (enhance existing)
7. **Secret Revocation & Cleanup** - Cascade revocation (NEW service)
8. **Data Classification** - Indonesian government levels (NEW service)
9. **Tokenization Enhancement** - Batch operations, audit (enhance existing)
10. **HA/DR Enhancement** - Cross-region replication (enhance existing)
11. **Audit & Compliance** - Tamper-proof logging, compliance reports (enhance existing)
12. **Performance & Scalability** - Redis cache, rate limiting API (enhance existing)

## Architecture

### Enhanced Component Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        API Gateway (Envoy)                        │
└──────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
        ▼                           ▼                           ▼
┌───────────────┐         ┌───────────────┐         ┌───────────────┐
│  REST API     │         │   gRPC API    │         │  K8s Operator │
│  (Axum)       │         │   (Tonic)     │         │  (NEW)        │
└───────────────┘         └───────────────┘         └───────────────┘
        │                           │                           │
        └───────────────────────────┼───────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────-─────────────┐
│                      Core Services Layer                          │
├──────────────┬──────────────┬──────────────┬───────-──────────────┤
│ Key Hierarchy│ Zero-Knowledge│ Crypto API   │ Classification      │
│ Service      │ Service (NEW) │ Service (NEW)│ Service (NEW)       │
├──────────────┼──────────────┼──────────────┼─────────-────────────┤
│ Dynamic      │ PKI          │ SSH          │ Revocation           │
│ Secrets      │ Service      │ Service      │ Service (NEW)        │
├──────────────┼──────────────┼──────────────┼──────────-───────────┤
│ Transform    │ Audit        │ Lease        │ Cache                │
│ Engine       │ Service      │ Service      │ Service (NEW)        │
└──────────────┴──────────────┴──────────────┴──────────────────-───┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Crypto Layer (secreton-crypto)              │
├──────────────┬──────────────┬──────────────┬─────────────────────┤
│ Shamir SSS   │ HKDF         │ HMAC API     │ FPE (FF3-1)         │
│ (existing)   │ (existing)   │ (enhance)    │ (existing)          │
├──────────────┼──────────────┼──────────────┼─────────────────────┤
│ AES-GCM      │ ChaCha20     │ Ed25519      │ Post-Quantum        │
│ (existing)   │ (existing)   │ (existing)   │ (existing)          │
└──────────────┴──────────────┴──────────────┴─────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Storage Layer                               │
├──────────────┬──────────────┬──────────────┬─────────────────────┤
│ PostgreSQL   │ Raft         │ File         │ Redis Cache         │
│ (existing)   │ (existing)   │ (existing)   │ (NEW)               │
└──────────────┴──────────────┴──────────────┴─────────────────────┘
```

## Components and Interfaces

### 1. Key Hierarchy Service

**Location**: `crates/core/src/services/key_hierarchy.rs` (NEW)

```rust
/// Key hierarchy levels
pub enum KeyLevel {
    MasterKey,
    KeyEncryptionKey,
    DataEncryptionKey,
}

/// Key metadata without exposing key material
pub struct KeyMetadata {
    pub id: Uuid,
    pub level: KeyLevel,
    pub parent_id: Option<Uuid>,
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub version: u32,
}

/// Key Hierarchy Service trait
#[async_trait]
pub trait KeyHierarchyService: Send + Sync {
    /// Initialize master key with Shamir shares
    async fn initialize(&self, config: ShamirConfig) -> Result<Vec<Share>, KeyError>;

    /// Derive KEK from master key
    async fn derive_kek(&self, context: &str) -> Result<KeyMetadata, KeyError>;

    /// Create DEK encrypted with KEK
    async fn create_dek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError>;

    /// Get key lineage
    async fn get_lineage(&self, key_id: Uuid) -> Result<Vec<KeyMetadata>, KeyError>;

    /// Rotate KEK and re-encrypt all DEKs
    async fn rotate_kek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError>;
}
```

### 2. Zero-Knowledge Service

**Location**: `crates/core/src/services/zero_knowledge.rs` (NEW)

```rust
/// Zero-knowledge secret metadata
pub struct ZeroKnowledgeMetadata {
    pub path: String,
    pub encryption_algorithm: String,
    pub key_derivation_params: KeyDerivationParams,
    pub created_at: DateTime<Utc>,
}

/// Key derivation parameters for client
pub struct KeyDerivationParams {
    pub algorithm: String,  // "hkdf-sha256"
    pub salt: Vec<u8>,
    pub info: Vec<u8>,
    pub key_length: usize,
}

#[async_trait]
pub trait ZeroKnowledgeService: Send + Sync {
    /// Store pre-encrypted secret
    async fn store(&self, path: &str, encrypted_data: Vec<u8>,
                   metadata: ZeroKnowledgeMetadata) -> Result<(), ZkError>;

    /// Retrieve encrypted secret
    async fn retrieve(&self, path: &str) -> Result<(Vec<u8>, ZeroKnowledgeMetadata), ZkError>;

    /// Derive key parameters for client
    async fn derive_params(&self, client_entropy: &[u8]) -> Result<KeyDerivationParams, ZkError>;

    /// Check if path is zero-knowledge enabled
    async fn is_zk_enabled(&self, path: &str) -> Result<bool, ZkError>;
}
```

### 3. Cryptographic Services API

**Location**: `crates/api/src/handlers/crypto.rs` (NEW)

```rust
/// HMAC request
#[derive(Debug, Deserialize, Serialize)]
pub struct HmacRequest {
    pub key_name: String,
    pub algorithm: HmacAlgorithm,
    pub input: String,  // base64 encoded
    pub output_format: OutputFormat,
}

/// HMAC algorithm options
#[derive(Debug, Deserialize, Serialize)]
pub enum HmacAlgorithm {
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
}

/// Random bytes request
#[derive(Debug, Deserialize, Serialize)]
pub struct RandomRequest {
    pub bytes: usize,  // 1-65536
    pub format: OutputFormat,
}

/// Re-encryption request
#[derive(Debug, Deserialize, Serialize)]
pub struct ReencryptRequest {
    pub source_key: String,
    pub destination_key: String,
    pub ciphertext: String,
}

/// Output format options
#[derive(Debug, Deserialize, Serialize)]
pub enum OutputFormat {
    Hex,
    Base64,
    Raw,
}
```

### 4. Data Classification Service

**Location**: `crates/core/src/services/classification.rs` (NEW)

```rust
/// Indonesian government classification levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum ClassificationLevel {
    Biasa,          // Public
    Terbatas,       // Internal
    Rahasia,        // Confidential
    SangatRahasia,  // Top Secret
}

/// Classification metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetadata {
    pub level: ClassificationLevel,
    pub requires_mfa: bool,
    pub clearance_required: ClassificationLevel,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait ClassificationService: Send + Sync {
    /// Set classification for secret
    async fn classify(&self, path: &str, level: ClassificationLevel) -> Result<(), ClassError>;

    /// Check access based on clearance
    async fn check_access(&self, path: &str, user_clearance: ClassificationLevel)
        -> Result<bool, ClassError>;

    /// Generate classification report
    async fn generate_report(&self) -> Result<ClassificationReport, ClassError>;

    /// Enforce MFA for high-classification access
    async fn require_mfa(&self, path: &str) -> Result<bool, ClassError>;
}
```

### 5. Revocation Service

**Location**: `crates/core/src/services/revocation.rs` (NEW)

```rust
/// Revocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRequest {
    pub path: String,
    pub reason: String,
    pub cascade: bool,
    pub emergency: bool,
}

/// Revocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    pub id: Uuid,
    pub path: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,
    pub reason: String,
    pub cascade_count: usize,
}

#[async_trait]
pub trait RevocationService: Send + Sync {
    /// Revoke secret and associated leases
    async fn revoke(&self, request: RevocationRequest) -> Result<RevocationRecord, RevError>;

    /// Emergency revocation by pattern
    async fn emergency_revoke(&self, pattern: &str) -> Result<Vec<RevocationRecord>, RevError>;

    /// Get revocation history
    async fn get_history(&self, path: &str) -> Result<Vec<RevocationRecord>, RevError>;

    /// Detect orphaned secrets
    async fn detect_orphans(&self, threshold_days: u32) -> Result<Vec<String>, RevError>;
}
```

### 6. Cache Service

**Location**: `crates/core/src/services/cache.rs` (NEW)

```rust
/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_entries: usize,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Ttl,
}

#[async_trait]
pub trait CacheService: Send + Sync {
    /// Get cached secret
    async fn get(&self, key: &str) -> Option<CachedSecret>;

    /// Set cached secret
    async fn set(&self, key: &str, value: CachedSecret, ttl: Duration) -> Result<(), CacheError>;

    /// Invalidate cache entry
    async fn invalidate(&self, key: &str) -> Result<(), CacheError>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;
}
```

## Data Models

### Key Hierarchy Models

```rust
/// Encrypted DEK storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDek {
    pub id: Uuid,
    pub kek_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
}

/// KEK metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KekMetadata {
    pub id: Uuid,
    pub context: String,
    pub version: u32,
    pub dek_count: usize,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}
```

### Classification Models

```rust
/// Secret with classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedSecret {
    pub path: String,
    pub classification: ClassificationMetadata,
    pub data: SecretData,
    pub access_log: Vec<AccessRecord>,
}

/// Access record for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRecord {
    pub user_id: String,
    pub clearance: ClassificationLevel,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub mfa_verified: bool,
}
```

### Audit Models

```rust
/// Tamper-proof audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: AuditAction,
    pub resource: String,
    pub client_ip: String,
    pub metadata: HashMap<String, String>,
    pub hmac: Vec<u8>,  // HMAC for integrity
    pub previous_hash: Vec<u8>,  // Chain link
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Revoke,
    Rotate,
    Login,
    Logout,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following correctness properties must be validated:

### Property 1: Shamir Secret Sharing Round-Trip
*For any* master key and valid Shamir configuration (threshold T, shares N where T ≤ N), generating N shares and reconstructing with any T shares SHALL produce the original master key.
**Validates: Requirements 1.1**

### Property 2: HKDF Key Derivation Determinism
*For any* master key and context string, deriving a KEK with the same inputs SHALL always produce the same KEK, and different contexts SHALL produce different KEKs.
**Validates: Requirements 1.2**

### Property 3: DEK Encryption Round-Trip
*For any* DEK and KEK, encrypting the DEK with the KEK and then decrypting SHALL produce the original DEK.
**Validates: Requirements 1.3**

### Property 4: Key Metadata Serialization Round-Trip
*For any* valid KeyMetadata, serializing to JSON and deserializing SHALL produce an equivalent KeyMetadata object.
**Validates: Requirements 1.6, 1.7**

### Property 5: KEK Rotation Preserves DEK Accessibility
*For any* KEK with associated DEKs, after rotation all DEKs SHALL remain decryptable with the new KEK and SHALL NOT be decryptable with the old KEK.
**Validates: Requirements 1.5**

### Property 6: Zero-Knowledge Storage Round-Trip
*For any* pre-encrypted data stored in zero-knowledge mode, retrieving the data SHALL return the exact encrypted blob without modification.
**Validates: Requirements 2.1, 2.2**

### Property 7: Zero-Knowledge Audit Privacy
*For any* zero-knowledge operation, the audit log SHALL contain operation metadata (path, timestamp, actor) but SHALL NOT contain any secret content or encryption keys.
**Validates: Requirements 2.5**

### Property 8: HMAC Consistency
*For any* key and input data, computing HMAC with the same algorithm SHALL always produce the same output, and different keys SHALL produce different outputs.
**Validates: Requirements 3.1**

### Property 9: Random Bytes Length and Format
*For any* requested byte length (1-65536) and output format, the generated random data SHALL have the correct length and format encoding.
**Validates: Requirements 3.2, 3.5**

### Property 10: Re-encryption Round-Trip
*For any* plaintext encrypted with source key, re-encrypting to destination key and decrypting with destination key SHALL produce the original plaintext.
**Validates: Requirements 3.3**

### Property 11: Batch HMAC Equivalence
*For any* batch of inputs, batch HMAC SHALL produce the same results as computing individual HMACs for each input.
**Validates: Requirements 3.4**

### Property 12: API Request/Response Serialization Round-Trip
*For any* valid HMAC API request, serializing to JSON and parsing SHALL preserve all fields. Similarly for responses.
**Validates: Requirements 3.6, 3.7**

### Property 13: Dynamic Credential Uniqueness
*For any* database role, each credential generation SHALL produce unique username/password pairs with correct TTL.
**Validates: Requirements 4.1, 4.2, 4.3**

### Property 14: Lease Expiration Revokes Credentials
*For any* dynamic credential with TTL, after the TTL expires the credential SHALL be automatically revoked and inaccessible.
**Validates: Requirements 4.6**

### Property 15: Webhook Retry Exponential Backoff
*For any* failed webhook delivery, retry intervals SHALL follow exponential backoff (1s, 2s, 4s, 8s, 16s) up to max 60s.
**Validates: Requirements 5.5**

### Property 16: OCSP Status Consistency
*For any* certificate, OCSP response status SHALL match the certificate's actual status (good if valid, revoked if revoked).
**Validates: Requirements 6.1**

### Property 17: Certificate Template Enforcement
*For any* certificate issued under a template, the certificate SHALL conform to all template constraints (key usage, validity period, etc.).
**Validates: Requirements 6.3**

### Property 18: CRL Contains All Revoked Certificates
*For any* CRL generation, the CRL SHALL contain serial numbers of all revoked certificates and none of the valid certificates.
**Validates: Requirements 6.5**

### Property 19: SSH Certificate Validity Bounds
*For any* SSH certificate request with TTL, the issued certificate validity period SHALL be within bounds (min: 1 minute, max: 24 hours).
**Validates: Requirements 7.1**

### Property 20: SSH Certificate Contains Requested Principals
*For any* SSH certificate request with principals, the issued certificate SHALL contain exactly the requested principals.
**Validates: Requirements 7.2**

### Property 21: Revocation Invalidates Secret
*For any* revoked secret, subsequent read attempts SHALL fail with "secret revoked" error.
**Validates: Requirements 8.1**

### Property 22: Cascade Revocation Completeness
*For any* secret with dependencies, cascade revocation SHALL revoke all secrets in the dependency chain.
**Validates: Requirements 8.2**

### Property 23: Classification Enforcement
*For any* secret with classification RAHASIA or SANGAT_RAHASIA, access without MFA verification SHALL be denied.
**Validates: Requirements 9.2**

### Property 24: Clearance Level Access Control
*For any* user with clearance level L, access to secrets with classification higher than L SHALL be denied.
**Validates: Requirements 9.5**

### Property 25: Tokenization Format Preservation
*For any* input string, the tokenized output SHALL have the same length and character class distribution as the input.
**Validates: Requirements 10.1**

### Property 26: Tokenization Round-Trip
*For any* tokenized value, detokenization by authorized user SHALL return the original value.
**Validates: Requirements 10.2**

### Property 27: Backup/Restore Round-Trip
*For any* system state, creating a backup and restoring SHALL produce an equivalent system state with all secrets intact.
**Validates: Requirements 11.3, 11.4**

### Property 28: Audit Log Integrity
*For any* audit log entry, HMAC verification SHALL detect any tampering with the entry content.
**Validates: Requirements 12.2**

### Property 29: Audit Query Filtering
*For any* audit query with filters, the results SHALL contain only entries matching all filter criteria.
**Validates: Requirements 12.3**

### Property 30: Cache LRU Eviction
*For any* cache at capacity, inserting a new entry SHALL evict the least recently used entry.
**Validates: Requirements 13.3**

### Property 31: Rate Limiting Enforcement
*For any* client exceeding rate limit, subsequent requests SHALL receive HTTP 429 with Retry-After header.
**Validates: Requirements 13.4**

## Error Handling

### Error Types

```rust
/// Unified error type for Secreton
#[derive(Debug, thiserror::Error)]
pub enum SecretonError {
    #[error("Key hierarchy error: {0}")]
    KeyHierarchy(#[from] KeyError),

    #[error("Zero-knowledge error: {0}")]
    ZeroKnowledge(#[from] ZkError),

    #[error("Classification error: {0}")]
    Classification(#[from] ClassError),

    #[error("Revocation error: {0}")]
    Revocation(#[from] RevError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Authentication required")]
    AuthRequired,

    #[error("MFA required for classification level")]
    MfaRequired,

    #[error("Insufficient clearance level")]
    InsufficientClearance,

    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after: u64 },
}
```

### Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "INSUFFICIENT_CLEARANCE",
    "message": "User clearance TERBATAS insufficient for RAHASIA secret",
    "details": {
      "required_clearance": "RAHASIA",
      "user_clearance": "TERBATAS"
    }
  }
}
```

## Testing Strategy

### Dual Testing Approach

This implementation uses both unit tests and property-based tests:

1. **Unit Tests**: Verify specific examples, edge cases, and error conditions
2. **Property-Based Tests**: Verify universal properties that should hold across all inputs

### Property-Based Testing Framework

**Framework**: `proptest` (already in Cargo.toml)

**Configuration**:
- Minimum 100 iterations per property test
- Shrinking enabled for counterexample minimization
- Seed logging for reproducibility

### Test Annotations

Each property-based test MUST be annotated with:
```rust
// **Feature: secreton-comprehensive-enhancement, Property {N}: {property_text}**
// **Validates: Requirements X.Y**
```

### Test Categories

1. **Crypto Properties** (Properties 1-5, 8-12)
   - Location: `crates/crypto/tests/property_tests.rs`
   - Focus: Round-trip, determinism, uniqueness

2. **Storage Properties** (Properties 6-7, 27-28)
   - Location: `crates/storage/tests/property_tests.rs`
   - Focus: Data integrity, audit privacy

3. **Service Properties** (Properties 13-26, 29-31)
   - Location: `crates/core/tests/property_tests.rs`
   - Focus: Business logic correctness

### Unit Test Coverage

Unit tests cover:
- API endpoint validation
- Error handling paths
- Edge cases (empty inputs, max values)
- Integration points between components

### Test Execution

```bash
# Run all tests
cargo test --workspace

# Run property tests only
cargo test --workspace -- property

# Run with verbose output
cargo test --workspace -- --nocapture

# Run specific property test
cargo test --package secreton-crypto -- shamir_roundtrip
```

