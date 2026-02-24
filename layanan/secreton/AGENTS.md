# 🤖 AGENTS.md - Secreton

> **AI Agent Guide** for working with the Secrets Management Service

## 🌍 Service Context

**Secreton** adalah Advanced Security Vault System untuk SIMPEL, dikembangkan oleh Cipherce dengan fokus pada Quantum-Safe Cryptography. Menyediakan:

### Core Features
- **Secret Storage**: Encrypted KV store untuk credentials, API keys, certificates
- **Transit Engine**: Encryption/decryption as a service (like HashiCorp Vault Transit)
- **Key Management**: Encryption keys, signing keys, automatic rotation
- **PKI Engine**: Certificate authority, certificate issuance
- **Format-Preserving Encryption (FPE)**: Encrypt data while preserving format

### Security Architecture
- **Encryption at Rest**: ChaCha20-Poly1305, AES-256-GCM
- **Encryption in Transit**: mTLS (gRPC), TLS 1.3 (REST)
- **Zero-knowledge**: Secreton never logs plaintext secrets
- **Key Wrapping**: Master key wraps Data Encryption Keys (DEKs)
- **Shamir's Secret Sharing**: Master key split across multiple custodians
- **Hybrid Cryptography**: Classical + Post-Quantum algorithms (ML-KEM, ML-DSA)

### Enterprise Features
- **HSM Integration**: PKCS#11 (Thales Luna, AWS CloudHSM, SoftHSM2, YubiHSM2)
- **High Availability**: Raft consensus for distributed deployment
- **Kubernetes Operator**: SecretonSecret CRD for auto-injection
- **CLI Tool**: Full-featured command-line interface
- **Agent/Sidecar**: Auto-renew secrets in pods
- **Audit Trail**: Complete cryptographic audit logging

**Compliance:**
- FIPS 140-3 compatible (with HSM)
- Zero-trust architecture
- Quantum-safe ready (hybrid PQC)

## 🔑 Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | Edition 2024, MSRV 1.90+ |
| gRPC Framework | Tonic + Prost | 0.14.x |
| HTTP Framework | Axum | 0.8.x |
| Symmetric Encryption | ChaCha20-Poly1305, AES-256-GCM | |
| Asymmetric | X25519 (key exchange), Ed25519 (signing) | |
| Post-Quantum | ML-KEM (encryption), ML-DSA (signing) | pqcrypto |
| Hashing | Blake3, SHA3-256, SHA-256 | |
| Key Derivation | Argon2id, PBKDF2, HKDF | |
| Storage Backend | PostgreSQL | tokio-postgres + deadpool |
| Consensus | Raft | (under migration to OpenRaft) |
| HSM | PKCS#11 | Optional |
| K8s Operator | kube-rs | |

## 🏗️ Architecture

```
layanan/secreton/
├── Cargo.toml              # Part of main simpelv2 workspace
├── crates/                 # Multi-crate workspace
│   │
│   ├── types/              # Shared types (secreton-types)
│   │   └── lib.rs          # SecurityLevel, Metadata, ResourceId, Tags
│   │
│   ├── core/               # Core business logic (secreton-core)
│   │   ├── lib.rs          # Re-exports from secreton-types
│   │   ├── audit/          # Audit logging
│   │   ├── auth/           # Authentication (AuthProvider, PqSignature)
│   │   ├── config/         # Configuration management
│   │   ├── error.rs        # CoreError types
│   │   ├── models/         # Data models (auth, policy, user)
│   │   ├── namespace/      # Namespace isolation
│   │   ├── pki/            # PKI/certificate management
│   │   ├── resilience/     # Circuit breaker, retry patterns
│   │   ├── security/       # Security utilities
│   │   ├── services/       # Business services
│   │   │   └── secrets/    # Secret engine
│   │   ├── storage/        # Storage abstraction
│   │   └── utils/          # Common utilities
│   │
│   ├── crypto/             # Cryptographic library (secreton-crypto)
│   │   ├── lib.rs          # CryptoEngine, HybridCrypto, TransitEngine
│   │   ├── encryption.rs   # AES-256-GCM, ChaCha20-Poly1305
│   │   ├── fpe.rs          # Format-Preserving Encryption
│   │   ├── hashing.rs      # Blake3, SHA3-256, SHA-256
│   │   ├── hybrid.rs       # Classical + PQC hybrid crypto
│   │   ├── key_derivation.rs  # Argon2id, PBKDF2, HKDF
│   │   ├── kv_engine.rs    # KV encryption engine
│   │   ├── pq_key_management.rs  # Post-quantum key management
│   │   ├── pqc/            # Post-quantum cryptography
│   │   ├── shamir.rs       # Shamir's Secret Sharing
│   │   ├── storage_integration.rs  # CryptoStorageBridge
│   │   └── transit/        # Transit engine (encrypt/decrypt service)
│   │
│   ├── storage/            # Storage backends (secreton-storage)
│   │   ├── lib.rs
│   │   ├── backends/       # PostgreSQL, etc.
│   │   ├── cache.rs        # Caching layer
│   │   ├── encrypted_storage.rs  # Encrypted at-rest
│   │   ├── factory.rs      # Backend factory
│   │   ├── memory.rs       # In-memory (testing)
│   │   ├── raft/           # Raft consensus storage
│   │   └── raft_backend.rs
│   │
│   ├── api/                # HTTP REST API (secreton-api)
│   │   ├── lib.rs          # ApiState, routers
│   │   ├── auth/           # Authentication handlers
│   │   ├── config.rs       # API configuration
│   │   ├── handlers/       # HTTP handlers
│   │   ├── kv.rs           # KV engine routes
│   │   ├── metrics.rs      # Prometheus metrics
│   │   ├── middleware/     # Axum middleware
│   │   ├── pki.rs          # PKI routes
│   │   ├── transit.rs      # Transit engine routes
│   │   └── services/       # Service layer
│   │
│   ├── grpc/               # gRPC service (secreton-grpc)
│   │   ├── lib.rs          # Generated proto, exports
│   │   ├── server.rs       # SecretonGrpcService
│   │   ├── tls.rs          # mTLS configuration
│   │   ├── interceptor.rs  # Auth interceptor
│   │   └── generated/      # Proto-generated code (OUT_DIR)
│   │
│   ├── cli/                # CLI tool (secreton-cli)
│   │   ├── main.rs         # CLI entry point
│   │   ├── lib.rs          # CLI library
│   │   ├── seal.rs         # Seal/unseal commands
│   │   ├── token.rs        # Token management
│   │   ├── policy.rs       # Policy commands
│   │   ├── audit.rs        # Audit log viewer
│   │   ├── backup.rs       # Backup/restore
│   │   └── operator.rs     # K8s operator commands
│   │
│   ├── hsm/                # HSM integration (secreton-hsm)
│   │   ├── lib.rs          # HsmBackend
│   │   ├── backend.rs      # PKCS#11 implementation
│   │   ├── config.rs       # HSM configuration
│   │   ├── pkcs11.rs       # PKCS#11 bindings
│   │   └── error.rs        # HSM errors
│   │
│   ├── k8s-operator/       # Kubernetes operator
│   │   └── ...             # SecretonSecret CRD controller
│   │
│   └── agent/              # Secreton agent (sidecar)
│       └── ...             # Auto-renew agent
│
├── proto/                  # gRPC proto definitions
│   ├── secreton/v1/        # Secreton service protos
│   └── common/v1/          # Common types
│
├── migrations/             # Database migrations
├── deploy/                 # Deployment configs
├── examples/               # Usage examples
├── monitoring/             # Monitoring configs
├── scripts/                # Operational scripts
└── tests/                  # Integration tests
```

## 📏 Critical Conventions

### 1. Configuration

**Main Configuration:** `crates/api/src/config.rs`

```rust
/// Main API configuration
pub struct ApiConfig {
    pub http: HttpConfig,            // HTTP server (bind address, timeouts)
    pub grpc: GrpcConfig,            // gRPC server (bind address, enabled)
    pub auth: AuthConfig,            // Authentication settings
    pub rate_limit: RateLimitConfig, // Rate limiting
    pub tls: Option<TlsConfig>,      // TLS certificates
    pub monitoring: MonitoringConfig, // Prometheus, OpenTelemetry
    pub cors: CorsConfig,            // CORS settings
    pub logging: LoggingConfig,      // Log levels, format
    pub hsm: HsmConfig,              // HSM/PKCS#11 settings
    pub database: DatabaseConfig,    // PostgreSQL connection
    pub storage: StorageConfig,      // Storage backend (Raft, etc.)
}

impl ApiConfig {
    /// Load from BootstrapConfig + ApplicationConfig
    pub fn from_bootstrap_and_application(
        bootstrap: &BootstrapConfig,
        app: &ApplicationConfig,
    ) -> Result<Self, String>;
}
```

**Core Configuration:** `crates/core/src/config/mod.rs`

```rust
/// Bootstrap configuration (loaded first)
pub struct BootstrapConfig {
    pub http: HttpBootstrapConfig,
    pub grpc: GrpcBootstrapConfig,
    pub storage: StorageBootstrapConfig,
}

/// Application configuration (loaded after bootstrap)
pub struct ApplicationConfig {
    pub encryption: EncryptionConfig,
    pub authentication: AuthenticationConfig,
    pub policies: PoliciesConfig,
    pub audit: AuditConfig,
}
```

**Environment Variables:**
```bash
# HTTP Server
SECRETON_HTTP_ADDRESS=0.0.0.0:8200
SECRETON_HTTP_TIMEOUT=30

# gRPC Server
SECRETON_GRPC_ADDRESS=0.0.0.0:50052
SECRETON_GRPC_ENABLED=true

# Storage
SECRETON_STORAGE_BACKEND=postgres  # or: raft, memory
DATABASE_URL=postgres://secreton:password@localhost:5432/secreton

# Raft (for HA setup)
SECRETON_RAFT_NODE_ID=1
SECRETON_RAFT_PEERS=node2:50053,node3:50054

# HSM (Optional)
SECRETON_HSM_ENABLED=false
SECRETON_HSM_LIBRARY=/usr/lib/softhsm/libsofthsm2.so
SECRETON_HSM_SLOT=0
SECRETON_HSM_PIN=1234

# TLS
SECRETON_TLS_CERT=/certs/server.crt
SECRETON_TLS_KEY=/certs/server.key
SECRETON_TLS_CA=/certs/ca.crt  # For mTLS

# Monitoring
SECRETON_METRICS_ENABLED=true
SECRETON_METRICS_ADDRESS=0.0.0.0:9090
```

### 2. Transit Engine (Encryption-as-a-Service)

**File:** `crates/crypto/src/transit/mod.rs`

Transit Engine provides encryption/decryption operations without exposing keys:

```rust
use secreton_crypto::transit::{
    TransitEngine, TransitOperations,
    EncryptRequest, EncryptResponse,
    DecryptRequest, DecryptResponse,
    KeyType, CreateKeyRequest,
};

// Create transit engine
let transit = TransitEngine::new(storage_backend);

// Create a named encryption key
transit.create_key(CreateKeyRequest {
    name: "my-app-key".into(),
    key_type: KeyType::ChaCha20Poly1305,
    exportable: false,
    allow_plaintext_backup: false,
}).await?;

// Encrypt data (key never leaves transit engine)
let response = transit.encrypt(EncryptRequest {
    key_name: "my-app-key".into(),
    plaintext: base64::encode("sensitive-data"),
    context: None, // Optional AEAD context
}).await?;

// response.ciphertext is base64-encoded encrypted data

// Decrypt data
let response = transit.decrypt(DecryptRequest {
    key_name: "my-app-key".into(),
    ciphertext: response.ciphertext,
    context: None,
}).await?;

// Key rotation (automatic re-encryption)
transit.rotate_key(RotateKeyRequest {
    name: "my-app-key".into(),
}).await?;
```

### 3. Secret Storage Pattern (KV Engine)

**File:** `crates/core/src/encryption.rs`

```rust
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};

pub struct SecretEngine {
    master_key: Key,
    storage: Box<dyn Storage>,
}

impl SecretEngine {
    pub async fn store_secret(
        &self,
        path: &str,
        data: &[u8],
        metadata: SecretMetadata,
    ) -> Result<SecretVersion> {
        // 1. Generate data encryption key (DEK)
        let dek = ChaCha20Poly1305::generate_key(&mut OsRng);

        // 2. Encrypt secret data with DEK
        let cipher = ChaCha20Poly1305::new(&dek);
        let nonce = Nonce::from_slice(b"unique nonce"); // Generate properly
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| SecretonError::EncryptionFailed(e.to_string()))?;

        // 3. Encrypt DEK with master key (key wrapping)
        let master_cipher = ChaCha20Poly1305::new(&self.master_key);
        let wrapped_dek = master_cipher.encrypt(nonce, dek.as_slice())
            .map_err(|e| SecretonError::EncryptionFailed(e.to_string()))?;

        // 4. Store encrypted data + wrapped DEK
        let version = self.storage.store(StoredSecret {
            path: path.to_string(),
            ciphertext,
            wrapped_dek,
            metadata,
            created_at: Utc::now(),
        }).await?;

        // 5. Audit log (never log plaintext!)
        audit_log(AuditEvent::SecretStored {
            path: path.to_string(),
            version: version.version,
            user: metadata.created_by.clone(),
        }).await;

        Ok(version)
    }

    pub async fn retrieve_secret(
        &self,
        path: &str,
        version: Option<u32>,
    ) -> Result<Vec<u8>> {
        // 1. Fetch from storage
        let stored = self.storage.get(path, version).await?;

        // 2. Unwrap DEK with master key
        let master_cipher = ChaCha20Poly1305::new(&self.master_key);
        let dek_bytes = master_cipher.decrypt(nonce, stored.wrapped_dek.as_ref())
            .map_err(|e| SecretonError::DecryptionFailed(e.to_string()))?;
        let dek = Key::from_slice(&dek_bytes);

        // 3. Decrypt secret data with DEK
        let cipher = ChaCha20Poly1305::new(dek);
        let plaintext = cipher.decrypt(nonce, stored.ciphertext.as_ref())
            .map_err(|e| SecretonError::DecryptionFailed(e.to_string()))?;

        // 4. Audit log (never log plaintext!)
        audit_log(AuditEvent::SecretRetrieved {
            path: path.to_string(),
            version: stored.version,
        }).await;

        Ok(plaintext)
    }
}
```

### 3. Sealing/Unsealing

**File:** `crates/crypto/src/seal.rs`

```rust
use lib_crypto::shamir::ShamirSecretSharing;

pub struct SealManager {
    config: SecretonConfig,
}

impl SealManager {
    // Initialize on first run
    pub async fn initialize(&self) -> Result<Vec<String>> {
        // 1. Generate master key
        let master_key = ChaCha20Poly1305::generate_key(&mut OsRng);

        // 2. Split using Shamir's Secret Sharing
        let sss = ShamirSecretSharing::new(
            self.config.shamir_threshold,
            self.config.shamir_shares,
        );

        let shares = sss.split(master_key.as_slice())?;

        // 3. Encrypt and store sealed master key
        // (encrypted with key derived from shares)
        let sealed_key = seal_master_key(&master_key, &shares)?;
        std::fs::write(&self.config.master_key_path, sealed_key)?;

        // 4. Return shares to operators (display once!)
        Ok(shares.into_iter()
            .map(hex::encode)
            .collect())
    }

    // Unseal on startup (requires threshold shares)
    pub async fn unseal(&self, shares: Vec<String>) -> Result<Key> {
        if shares.len() < self.config.shamir_threshold {
            return Err(SecretonError::InsufficientShares);
        }

        // 1. Decode shares from hex
        let decoded_shares: Vec<Vec<u8>> = shares
            .into_iter()
            .map(|s| hex::decode(s))
            .collect::<Result<_, _>>()?;

        // 2. Reconstruct master key using Shamir
        let sss = ShamirSecretSharing::new(
            self.config.shamir_threshold,
            self.config.shamir_shares,
        );

        let master_key_bytes = sss.reconstruct(&decoded_shares)?;
        let master_key = Key::from_slice(&master_key_bytes);

        // 3. Verify by decrypting sealed master key
        let sealed_key = std::fs::read(&self.config.master_key_path)?;
        verify_master_key(master_key, &sealed_key)?;

        Ok(*master_key)
    }
}
```

### 4. gRPC Service

**File:** `crates/grpc/src/secret_service.rs`

```rust
use tonic::{Request, Response, Status};

pub mod secret_proto {
    tonic::include_proto!("secret");
}

use secret_proto::secret_service_server::{SecretService, SecretServiceServer};

#[derive(Debug)]
pub struct SecretServiceImpl {
    engine: Arc<SecretEngine>,
}

#[tonic::async_trait]
impl SecretService for SecretServiceImpl {
    async fn store_secret(
        &self,
        request: Request<StoreSecretRequest>,
    ) -> Result<Response<StoreSecretResponse>, Status> {
        // 1. Authenticate request
        let auth = authenticate_request(&request)?;

        // 2. Authorize access to path
        authorize_path(&auth, &request.get_ref().path, "write")?;

        let req = request.into_inner();

        // 3. Store secret
        let version = self.engine
            .store_secret(
                &req.path,
                &req.data,
                SecretMetadata {
                    created_by: auth.user_id,
                    tags: req.tags,
                },
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StoreSecretResponse {
            version: version.version,
            created_at: version.created_at.to_rfc3339(),
        }))
    }

    async fn get_secret(
        &self,
        request: Request<GetSecretRequest>,
    ) -> Result<Response<GetSecretResponse>, Status> {
        // 1. Authenticate
        let auth = authenticate_request(&request)?;

        // 2. Authorize
        authorize_path(&auth, &request.get_ref().path, "read")?;

        let req = request.into_inner();

        // 3. Retrieve secret
        let data = self.engine
            .retrieve_secret(&req.path, req.version)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetSecretResponse {
            data,
        }))
    }
}
```

### 5. Kubernetes Operator

**File:** `crates/k8s-operator/src/controller.rs`

```rust
use kube::{
    api::{Api, ResourceExt},
    runtime::controller::{Action, Controller},
    Client,
};

// Custom Resource Definition
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug)]
#[kube(group = "secreton.kejaksaan.go.id", version = "v1", kind = "SecretInjection")]
pub struct SecretInjectionSpec {
    pub secret_path: String,
    pub target_namespace: String,
    pub target_secret_name: String,
}

pub async fn reconcile(
    injection: Arc<SecretInjection>,
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.client;

    // 1. Fetch secret from Secreton via gRPC
    let secret_data = ctx.secreton_client
        .get_secret(GetSecretRequest {
            path: injection.spec.secret_path.clone(),
            version: None,
        })
        .await?
        .into_inner()
        .data;

    // 2. Create/update Kubernetes Secret
    let k8s_secret = Secret {
        metadata: ObjectMeta {
            name: Some(injection.spec.target_secret_name.clone()),
            namespace: Some(injection.spec.target_namespace.clone()),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("value".to_string(), ByteString(secret_data)),
        ])),
        ..Default::default()
    };

    let secrets: Api<Secret> = Api::namespaced(
        client.clone(),
        &injection.spec.target_namespace,
    );

    secrets.patch(
        &injection.spec.target_secret_name,
        &PatchParams::apply("secreton-operator"),
        &Patch::Apply(&k8s_secret),
    ).await?;

    // 3. Requeue after 5 minutes (to sync updates)
    Ok(Action::requeue(Duration::from_secs(300)))
}
```

### 6. CLI Tool

**File:** `crates/cli/src/main.rs`

```bash
# Usage examples:

# Initialize Secreton (first time)
secreton-cli init --shares 5 --threshold 3

# Unseal (provide 3 shares)
secreton-cli unseal
# Enter share 1: abc123...
# Enter share 2: def456...
# Enter share 3: ghi789...
# ✅ Secreton unsealed successfully

# Store secret
secreton-cli put database/password "my-secure-password"
secreton-cli put jwt/signing-key @/path/to/key.pem

# Retrieve secret
secreton-cli get database/password
# my-secure-password

# List secrets
secreton-cli list database/

# Delete secret
secreton-cli delete database/old-password

# Rotate secret
secreton-cli rotate database/password --generate

# View secret versions
secreton-cli versions database/password

# Rollback to previous version
secreton-cli rollback database/password --version 2
```

## 🚀 Common Tasks

### Initialize Secreton (First Time)

```bash
# 1. Run initialization
secreton-cli init --shares 5 --threshold 3

# Output:
# Master Key Shares (STORE SECURELY!):
# Share 1: a1b2c3d4e5f6...
# Share 2: f6e5d4c3b2a1...
# Share 3: 1a2b3c4d5e6f...
# Share 4: 6f5e4d3c2b1a...
# Share 5: abcdef123456...

# 2. Distribute shares to 5 operators
# 3. Store shares in separate secure locations
# 4. Require 3 shares to unseal
```

### Unseal Secreton on Startup

```bash
# Each operator provides their share
secreton-cli unseal --share "a1b2c3d4e5f6..."

# Or interactive
secreton-cli unseal
# Enter share 1: a1b2c3d4e5f6...
# Enter share 2: f6e5d4c3b2a1...
# Enter share 3: 1a2b3c4d5e6f...
# ✅ Secreton unsealed successfully
```

### Rotate Master Key

```rust
// crates/crypto/src/rotation.rs
pub async fn rotate_master_key(
    engine: &SecretEngine,
    new_shares: usize,
    new_threshold: usize,
) -> Result<Vec<String>> {
    // 1. Generate new master key
    let new_master_key = ChaCha20Poly1305::generate_key(&mut OsRng);

    // 2. Re-encrypt all secrets with new key
    let secrets = engine.storage.list_all().await?;
    for secret in secrets {
        let plaintext = engine.retrieve_secret(&secret.path, None).await?;
        // Store with new key...
    }

    // 3. Split new key with Shamir
    let sss = ShamirSecretSharing::new(new_threshold, new_shares);
    let shares = sss.split(new_master_key.as_slice())?;

    // 4. Return new shares
    Ok(shares.into_iter().map(hex::encode).collect())
}
```

### Integrate with Layanan Service

```rust
// In layanan-portal/src/main.rs
use tonic::transport::Channel;

// Create Secreton gRPC client
let secreton_channel = Channel::from_static("https://secreton.internal:50052")
    .tls_config(tonic::transport::ClientTlsConfig::new())?
    .connect()
    .await?;

let mut secreton_client = SecretServiceClient::new(secreton_channel);

// Fetch database password from Secreton
let response = secreton_client
    .get_secret(GetSecretRequest {
        path: "database/portal/password".to_string(),
        version: None,
    })
    .await?;

let db_password = String::from_utf8(response.into_inner().data)?;

// Use in database connection
let database_url = format!(
    "postgres://portal:{}@localhost:5432/simpelv2",
    db_password
);
```

## ⚠️ Common Pitfalls

### ❌ DON'T

1. **Log plaintext secrets**
   ```rust
   // ❌ BAD
   tracing::info!("Stored secret: {}", plaintext);

   // ✅ GOOD
   tracing::info!("Stored secret at path: {}", path);
   ```

2. **Store master key in environment variable**
   ```bash
   # ❌ BAD
   MASTER_KEY=abc123def456...

   # ✅ GOOD - Use Shamir unsealing
   secreton-cli unseal
   ```

3. **Use single share for unsealing**
   ```rust
   // ❌ BAD
   SHAMIR_SHARES=1
   SHAMIR_THRESHOLD=1

   // ✅ GOOD
   SHAMIR_SHARES=5
   SHAMIR_THRESHOLD=3
   ```

4. **Skip encryption for "non-sensitive" secrets**
   ```rust
   // ❌ BAD - All secrets must be encrypted
   if secret_type == "config" {
       store_plaintext(data);
   }

   // ✅ GOOD - Encrypt everything
   engine.store_secret(path, data, metadata).await?;
   ```

5. **Allow unauthenticated access**
   ```rust
   // ❌ BAD
   REQUIRE_AUTHENTICATION=false

   // ✅ GOOD
   REQUIRE_AUTHENTICATION=true
   ```

### ✅ DO

1. **Always encrypt secrets** at rest and in transit
2. **Use Shamir's Secret Sharing** for master key
3. **Audit all secret access** (read/write/delete)
4. **Rotate secrets** regularly (automatic rotation)
5. **Use versioning** for rollback capability
6. **Require mTLS** for gRPC communication
7. **Test disaster recovery** (unseal from shares)

## 🔍 Troubleshooting

### Unsealing Fails

**Problem:** "Invalid shares" or "Insufficient shares"

**Solution:**
```bash
# Verify share count
secreton-cli status
# Required: 3 shares
# Provided: 2 shares

# Verify shares are correct
# Each share should be 64 hex characters
echo $SHARE | wc -c  # Should be 64
```

### Secret Not Found

**Problem:** "Secret does not exist at path"

**Solution:**
```bash
# List secrets to verify path
secreton-cli list --recursive

# Check version
secreton-cli versions database/password

# Verify permissions
secreton-cli acl get database/password
```

### HSM Connection Failed

**Problem:** "Failed to connect to HSM"

**Solution:**
```bash
# Check HSM library
ls -l $HSM_LIBRARY_PATH

# Test PKCS#11
pkcs11-tool --module $HSM_LIBRARY_PATH --list-slots

# Verify slot ID
HSM_SLOT_ID=0  # Correct slot number
```

## 📚 Key Files Reference

| File | Purpose |
|------|---------|
| `crates/core/src/encryption.rs` | Secret encryption/decryption |
| `crates/crypto/src/seal.rs` | Sealing/unsealing logic |
| `crates/crypto/src/shamir.rs` | Shamir's Secret Sharing |
| `crates/grpc/src/secret_service.rs` | gRPC secret service |
| `crates/cli/src/main.rs` | CLI tool |
| `crates/k8s-operator/src/controller.rs` | K8s operator |

## 🎓 Best Practices

1. **Use Shamir's Secret Sharing** (5 shares, 3 threshold minimum)
2. **Encrypt all secrets** with ChaCha20-Poly1305
3. **Never log plaintext** secrets or keys
4. **Require mTLS** for all gRPC communication
5. **Audit all operations** to audit log
6. **Rotate secrets** automatically (30-90 days)
7. **Use versioning** for all secrets
8. **Test disaster recovery** regularly
9. **Distribute shares** to different operators
10. **Use HSM** for production master key protection

---

**Last Updated:** February 2, 2026
**Maintainer:** SIMPEL Team
**Related:** `/AGENTS.md`, `/layanan/authenc/AGENTS.md`
