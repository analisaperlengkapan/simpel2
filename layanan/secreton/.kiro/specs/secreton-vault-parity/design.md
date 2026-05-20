# Design Document: Secreton - HashiCorp Vault Feature Parity & Production Hardening

## 1. Overview

### 1.1 Purpose

This design document specifies the architecture and implementation approach for achieving HashiCorp Vault feature parity with Secreton while maintaining its unique advantages (Rust-based, quantum-safe cryptography, government compliance). The design addresses critical gaps identified in the requirements document, focusing on operational and enterprise capabilities needed for production deployment at Kejaksaan RI.

### 1.2 Scope

This design covers 14 major feature areas:

1. Auto-unseal capability (AWS KMS, GCP KMS, Azure Key Vault, Transit)
2. Performance replication (multi-region read replicas)
3. Disaster recovery replication (full cluster failover)
4. Performance standby nodes (read-only hot standbys)
5. Automated backup and restore procedures
6. Advanced monitoring and observability (OpenTelemetry)
7. Secrets rotation automation
8. Response caching layer
9. Request forwarding optimization
10. Comprehensive health checks
11. Vault Agent/Sidecar for Kubernetes
12. Kubernetes Secrets Operator
13. KMIP secrets engine
14. Key Management secrets engine (AWS/GCP/Azure KMS lifecycle)

### 1.3 Goals

- Achieve 95%+ production readiness (from current 85%)
- Enable zero-downtime operations in Kubernetes
- Support multi-region deployment with low latency
- Provide seamless application integration via Agent/Sidecar
- Enable infrastructure integration via KMIP protocol
- Maintain backward compatibility with existing Secreton API
- Preserve Rust-only codebase (no C/C++ dependencies)
- Ensure all features meet government security standards

### 1.4 Non-Goals

- GUI admin console (separate project)
- LDAP/AD integration (handled by Authenc)
- Custom secrets engines via plugin system
- Multi-cloud federation
- Blockchain integration
- Machine learning features

## 2. Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Kubernetes Cluster                          │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    Application Pods                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │  │
│  │  │ App Container│  │ App Container│  │ App Container│      │  │
│  │  │              │  │              │  │              │      │  │
│  │  │ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐ │      │  │
│  │  │ │  Sidecar │ │  │ │  Sidecar │ │  │ │  Sidecar │ │      │  │
│  │  │ │  Agent   │ │  │ │  Agent   │ │  │ │  Agent   │ │      │  │
│  │  │ └────┬─────┘ │  │ └────┬─────┘ │  │ └────┬─────┘ │      │  │
│  │  └──────┼───────┘  └──────┼───────┘  └──────┼───────┘      │  │
│  └─────────┼──────────────────┼──────────────────┼─────────────┘  │
│            │                  │                  │                 │
│            └──────────────────┼──────────────────┘                 │
│                               │                                    │
│  ┌────────────────────────────┼────────────────────────────────┐  │
│  │         Secreton Cluster   │                                │  │
│  │                            ▼                                │  │
│  │  ┌──────────────────────────────────────────────────────┐  │  │
│  │  │              Load Balancer (MetalLB)                 │  │  │
│  │  └────────┬─────────────────────────────────────────────┘  │  │
│  │           │                                                 │  │
│  │  ┌────────┴─────────┬──────────────┬──────────────┐       │  │
│  │  │                  │              │              │       │  │
│  │  ▼                  ▼              ▼              ▼       │  │
│  │ ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │  │
│  │ │ Active  │  │Standby 1│  │Standby 2│  │Standby 3│      │  │
│  │ │  Node   │  │  Node   │  │  Node   │  │  Node   │      │  │
│  │ │         │  │         │  │         │  │         │      │  │
│  │ │ R/W     │  │ R-only  │  │ R-only  │  │ R-only  │      │  │
│  │ └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘      │  │
│  │      │            │             │             │           │  │
│  │      └────────────┴─────────────┴─────────────┘           │  │
│  │                   │                                        │  │
│  │                   ▼                                        │  │
│  │          ┌─────────────────┐                              │  │
│  │          │  Raft Consensus │                              │  │
│  │          │   (OpenRaft)    │                              │  │
│  │          └────────┬────────┘                              │  │
│  │                   │                                        │  │
│  │                   ▼                                        │  │
│  │          ┌─────────────────┐                              │  │
│  │          │   PostgreSQL    │                              │  │
│  │          │   (Patroni HA)  │                              │  │
│  │          └─────────────────┘                              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Secrets Operator (Controller)                  │  │
│  │                                                           │  │
│  │  Watches: SecretonSecret CRD                             │  │
│  │  Creates: Kubernetes Secrets                             │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                      External Services                              │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │
│  │   AWS KMS    │  │   GCP KMS    │  │  Azure KV    │            │
│  │ (Auto-unseal)│  │ (Auto-unseal)│  │ (Auto-unseal)│            │
│  └──────────────┘  └──────────────┘  └──────────────┘            │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │
│  │  S3 Storage  │  │  Prometheus  │  │   Grafana    │            │
│  │  (Backups)   │  │  (Metrics)   │  │ (Dashboard)  │            │
│  └──────────────┘  └──────────────┘  └──────────────┘            │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐                               │
│  │   VMware     │  │   NetApp     │                               │
│  │ (KMIP Client)│  │ (KMIP Client)│                               │
│  └──────────────┘  └──────────────┘                               │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                    Performance Replication                          │
│                                                                     │
│  Primary Cluster (Region 1)  ──────────────▶  Secondary Cluster    │
│  Jakarta                                       (Region 2)           │
│                                                Surabaya             │
│  • Writes                                      • Reads only         │
│  • Reads                                       • Replication lag    │
│  • Active node                                   < 100ms            │
└─────────────────────────────────────────────────────────────────────┘


### 2.2 Component Architecture

#### 2.2.1 Core Components (Existing)

```

secreton/
├── crates/
│   ├── core/              # Business logic
│   │   ├── services/
│   │   │   ├── secrets/   # KV, Transit, PKI, SSH, TOTP, Transform
│   │   │   ├── lease/     # Lease management (COMPLETE)
│   │   │   └── policy/    # Policy engine (COMPLETE)
│   │   ├── storage/       # Storage abstraction
│   │   └── audit/         # Audit logging
│   │
│   ├── crypto/            # Cryptographic operations
│   │   ├── seal.rs        # Seal/unseal (Shamir)
│   │   ├── transit/       # Transit engine
│   │   ├── shamir.rs      # Secret sharing
│   │   └── hybrid.rs      # Post-quantum crypto
│   │
│   ├── storage/           # Storage backends
│   │   ├── postgres.rs    # PostgreSQL backend
│   │   ├── raft/          # Raft consensus
│   │   └── cache.rs       # Caching layer
│   │
│   ├── api/               # REST API (Axum)
│   ├── grpc/              # gRPC service (Tonic)
│   ├── cli/               # CLI tool
│   └── hsm/               # HSM integration (PKCS#11)

```

#### 2.2.2 New Components (To Be Implemented)

```

secreton/
├── crates/
│   ├── auto-unseal/       # NEW: Auto-unseal providers
│   │   ├── lib.rs         # AutoUnsealProvider trait
│   │   ├── transit.rs     # Transit auto-unseal
│   │   ├── aws_kms.rs     # AWS KMS auto-unseal
│   │   ├── gcp_kms.rs     # GCP KMS auto-unseal
│   │   └── azure_kv.rs    # Azure Key Vault auto-unseal
│   │
│   ├── replication/       # NEW: Replication engine
│   │   ├── lib.rs         # Replication manager
│   │   ├── performance.rs # Performance replication
│   │   ├── dr.rs          # Disaster recovery replication
│   │   ├── standby.rs     # Performance standby nodes
│   │   └── forwarding.rs  # Request forwarding
│   │
│   ├── backup/            # NEW: Backup/restore
│   │   ├── lib.rs         # Backup manager
│   │   ├── scheduler.rs   # Cron-like scheduler
│   │   ├── s3.rs          # S3 backend
│   │   └── restore.rs     # Restore procedures
│   │
│   ├── observability/     # NEW: Enhanced monitoring
│   │   ├── lib.rs         # Observability manager
│   │   ├── otel.rs        # OpenTelemetry integration
│   │   ├── metrics.rs     # Prometheus metrics
│   │   └── health.rs      # Health checks
│   │
│   ├── rotation/          # NEW: Secrets rotation
│   │   ├── lib.rs         # Rotation manager
│   │   ├── database.rs    # Database credential rotation
│   │   ├── api_keys.rs    # API key rotation
│   │   └── certificates.rs # Certificate auto-renewal
│   │
│   ├── agent/             # NEW: Kubernetes agent/sidecar
│   │   ├── main.rs        # Agent entry point
│   │   ├── auth.rs        # K8s ServiceAccount auth
│   │   ├── template.rs    # Template rendering
│   │   └── renew.rs       # Auto-renewal logic
│   │
│   ├── operator/          # NEW: Kubernetes operator
│   │   ├── main.rs        # Operator entry point
│   │   ├── crd.rs         # SecretonSecret CRD
│   │   ├── controller.rs  # Reconciliation loop
│   │   └── sync.rs        # Secret synchronization
│   │
│   ├── kmip/              # NEW: KMIP secrets engine
│   │   ├── lib.rs         # KMIP server
│   │   ├── protocol.rs    # KMIP 1.4 protocol
│   │   ├── operations.rs  # Key lifecycle ops
│   │   └── scopes.rs      # VMware/NetApp scopes
│   │
│   └── key-management/    # NEW: Cloud KMS management
│       ├── lib.rs         # KMS manager
│       ├── aws.rs         # AWS KMS operations
│       ├── gcp.rs         # GCP KMS operations
│       └── azure.rs       # Azure Key Vault operations

```

### 2.3 Data Flow

#### 2.3.1 Auto-Unseal Flow

```

┌─────────────────────────────────────────────────────────────────┐
│ 1. Secreton Pod Starts (Sealed State)                          │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Read Auto-Unseal Config from secreton.toml                  │
│    - Provider: aws-kms | gcp-kms | azure-kv | transit          │
│    - Key ID / ARN / Resource ID                                 │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Fetch Encrypted Master Key from Storage                     │
│    - Stored in PostgreSQL: sealed_master_key table              │
│    - Encrypted with KMS key                                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Call KMS Provider to Decrypt Master Key                     │
│    - AWS: kms.decrypt(ciphertext, key_id)                      │
│    - GCP: cloudkms.decrypt(ciphertext, key_name)               │
│    - Azure: keyvault.decrypt(ciphertext, key_name)             │
│    - Transit: secreton.decrypt(ciphertext, key_name)           │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Load Decrypted Master Key into Memory                       │
│    - Verify key integrity (HMAC)                                │
│    - Initialize crypto engine                                   │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Transition to Unsealed State                                │
│    - Update health endpoint: sealed=false                       │
│    - Start accepting requests                                   │
│    - Audit log: auto-unseal successful                          │
└─────────────────────────────────────────────────────────────────┘

```

#### 2.3.2 Performance Replication Flow

```

Primary Cluster (Jakarta)                Secondary Cluster (Surabaya)
┌──────────────────────┐                 ┌──────────────────────┐
│  1. Write Request    │                 │                      │
│     POST /secret/db  │                 │                      │
└──────────┬───────────┘                 │                      │
           │                             │                      │
           ▼                             │                      │
┌──────────────────────┐                 │                      │
│  2. Store in Raft    │                 │                      │
│     Commit log       │                 │                      │
└──────────┬───────────┘                 │                      │
           │                             │                      │
           ▼                             │                      │
┌──────────────────────┐                 │                      │
│  3. Persist to       │                 │                      │
│     PostgreSQL       │                 │                      │
└──────────┬───────────┘                 │                      │
           │                             │                      │
           ├─────────────────────────────┼──────────────────────┤
           │  4. Replicate via gRPC      │                      │
           │     (mTLS encrypted)        │                      │
           └─────────────────────────────▶  5. Receive Update  │
                                         │     Apply to local   │
                                         │     PostgreSQL       │
                                         └──────────┬───────────┘
                                                    │
                                                    ▼
                                         ┌──────────────────────┐
                                         │  6. Serve Read       │
                                         │     Requests         │
                                         │     (low latency)    │
                                         └──────────────────────┘

```


#### 2.3.3 Agent/Sidecar Flow

```

┌─────────────────────────────────────────────────────────────────┐
│ Application Pod                                                 │
│                                                                 │
│  ┌──────────────────────┐      ┌──────────────────────┐       │
│  │  Init Container      │      │  App Container       │       │
│  │  (secreton-agent)    │      │                      │       │
│  │                      │      │  Reads secrets from  │       │
│  │  1. Authenticate     │      │  /vault/secrets/     │       │
│  │     with K8s SA      │      │                      │       │
│  │                      │      │  ┌────────────────┐  │       │
│  │  2. Fetch secrets    │      │  │ config.json    │  │       │
│  │     from Secreton    │      │  │ db-password    │  │       │
│  │                      │      │  │ api-key        │  │       │
│  │  3. Render templates │      │  └────────────────┘  │       │
│  │     to files         │      │                      │       │
│  │                      │      │                      │       │
│  │  4. Write to shared  │◀─────┤  Shared Volume      │       │
│  │     volume           │      │  /vault/secrets/    │       │
│  │                      │      │                      │       │
│  │  5. Exit (init mode) │      │                      │       │
│  └──────────────────────┘      └──────────────────────┘       │
│                                                                 │
│  OR (sidecar mode)                                             │
│                                                                 │
│  ┌──────────────────────┐      ┌──────────────────────┐       │
│  │  Sidecar Container   │      │  App Container       │       │
│  │  (secreton-agent)    │      │                      │       │
│  │                      │      │  Reads secrets from  │       │
│  │  1. Authenticate     │      │  /vault/secrets/     │       │
│  │                      │      │                      │       │
│  │  2. Fetch secrets    │      │  Secrets auto-update │       │
│  │                      │      │  when renewed        │       │
│  │  3. Auto-renew       │      │                      │       │
│  │     before expiry    │      │                      │       │
│  │                      │      │                      │       │
│  │  4. Update files     │◀─────┤  Shared Volume      │       │
│  │     on renewal       │      │  /vault/secrets/    │       │
│  │                      │      │                      │       │
│  │  (runs continuously) │      │                      │       │
│  └──────────────────────┘      └──────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘

```

## 3. Components and Interfaces

### 3.1 Auto-Unseal Component

#### 3.1.1 AutoUnsealProvider Trait

```rust
// crates/auto-unseal/src/lib.rs

use async_trait::async_trait;
use secreton_core::error::SecretonError;

/// Auto-unseal provider trait
#[async_trait]
pub trait AutoUnsealProvider: Send + Sync {
    /// Provider name (e.g., "aws-kms", "gcp-kms", "azure-kv", "transit")
    fn name(&self) -> &str;

    /// Encrypt the master key with the provider's key
    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// Decrypt the master key with the provider's key
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError>;

    /// Verify provider connectivity and permissions
    async fn health_check(&self) -> Result<(), SecretonError>;

    /// Get provider-specific metadata
    fn metadata(&self) -> ProviderMetadata;
}

pub struct ProviderMetadata {
    pub provider_type: String,
    pub key_id: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
}
```

#### 3.1.2 AWS KMS Provider

```rust
// crates/auto-unseal/src/aws_kms.rs

use aws_sdk_kms::{Client as KmsClient, types::Blob};
use aws_config::BehaviorVersion;

pub struct AwsKmsProvider {
    client: KmsClient,
    key_id: String,
    region: String,
}

impl AwsKmsProvider {
    pub async fn new(key_id: String, region: String) -> Result<Self, SecretonError> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()))
            .load()
            .await;

        let client = KmsClient::new(&config);

        Ok(Self {
            client,
            key_id,
            region,
        })
    }
}

#[async_trait]
impl AutoUnsealProvider for AwsKmsProvider {
    fn name(&self) -> &str {
        "aws-kms"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let response = self.client
            .encrypt()
            .key_id(&self.key_id)
            .plaintext(Blob::new(plaintext))
            .send()
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("AWS KMS encrypt failed: {}", e)))?;

        Ok(response.ciphertext_blob()
            .ok_or_else(|| SecretonError::AutoUnsealFailed("No ciphertext returned".into()))?
            .as_ref()
            .to_vec())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let response = self.client
            .decrypt()
            .key_id(&self.key_id)
            .ciphertext_blob(Blob::new(ciphertext))
            .send()
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("AWS KMS decrypt failed: {}", e)))?;

        Ok(response.plaintext()
            .ok_or_else(|| SecretonError::AutoUnsealFailed("No plaintext returned".into()))?
            .as_ref()
            .to_vec())
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        self.client
            .describe_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("AWS KMS health check failed: {}", e)))?;

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            provider_type: "aws-kms".to_string(),
            key_id: self.key_id.clone(),
            region: Some(self.region.clone()),
            endpoint: None,
        }
    }
}
```

#### 3.1.3 Transit Provider

```rust
// crates/auto-unseal/src/transit.rs

use tonic::transport::Channel;
use secreton_grpc::transit_service_client::TransitServiceClient;

pub struct TransitProvider {
    client: TransitServiceClient<Channel>,
    key_name: String,
    endpoint: String,
}

impl TransitProvider {
    pub async fn new(
        endpoint: String,
        key_name: String,
        token: String,
    ) -> Result<Self, SecretonError> {
        let channel = Channel::from_shared(endpoint.clone())
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Invalid endpoint: {}", e)))?
            .tls_config(tonic::transport::ClientTlsConfig::new())
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("TLS config failed: {}", e)))?
            .connect()
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Connection failed: {}", e)))?;

        let client = TransitServiceClient::with_interceptor(
            channel,
            move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );
                Ok(req)
            },
        );

        Ok(Self {
            client,
            key_name,
            endpoint,
        })
    }
}

#[async_trait]
impl AutoUnsealProvider for TransitProvider {
    fn name(&self) -> &str {
        "transit"
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let request = EncryptRequest {
            key_name: self.key_name.clone(),
            plaintext: base64::encode(plaintext),
            context: None,
        };

        let response = self.client
            .clone()
            .encrypt(request)
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Transit encrypt failed: {}", e)))?
            .into_inner();

        base64::decode(&response.ciphertext)
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Base64 decode failed: {}", e)))
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        let request = DecryptRequest {
            key_name: self.key_name.clone(),
            ciphertext: base64::encode(ciphertext),
            context: None,
        };

        let response = self.client
            .clone()
            .decrypt(request)
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Transit decrypt failed: {}", e)))?
            .into_inner();

        base64::decode(&response.plaintext)
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Base64 decode failed: {}", e)))
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        // Verify key exists
        let request = GetKeyRequest {
            name: self.key_name.clone(),
        };

        self.client
            .clone()
            .get_key(request)
            .await
            .map_err(|e| SecretonError::AutoUnsealFailed(format!("Transit health check failed: {}", e)))?;

        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            provider_type: "transit".to_string(),
            key_id: self.key_name.clone(),
            region: None,
            endpoint: Some(self.endpoint.clone()),
        }
    }
}
```

### 3.2 Replication Component

#### 3.2.1 Replication Manager

```rust
// crates/replication/src/lib.rs

use tokio::sync::RwLock;
use std::sync::Arc;

pub enum ReplicationMode {
    Performance,  // Read-only replicas
    DisasterRecovery,  // Full replication including ephemeral state
}

pub struct ReplicationManager {
    mode: ReplicationMode,
    primary_endpoint: Option<String>,
    secondaries: Arc<RwLock<Vec<SecondaryNode>>>,
    replication_lag: Arc<RwLock<Duration>>,
    storage: Arc<dyn Storage>,
}

pub struct SecondaryNode {
    pub id: String,
    pub endpoint: String,
    pub last_sync: DateTime<Utc>,
    pub lag: Duration,
    pub status: ReplicationStatus,
}

pub enum ReplicationStatus {
    Healthy,
    Lagging,
    Disconnected,
    Failed,
}

impl ReplicationManager {
    pub async fn new(
        mode: ReplicationMode,
        config: ReplicationConfig,
        storage: Arc<dyn Storage>,
    ) -> Result<Self, SecretonError> {
        Ok(Self {
            mode,
            primary_endpoint: config.primary_endpoint,
            secondaries: Arc::new(RwLock::new(Vec::new())),
            replication_lag: Arc::new(RwLock::new(Duration::from_millis(0))),
            storage,
        })
    }

    /// Start replication loop
    pub async fn start(&self) -> Result<(), SecretonError> {
        match self.mode {
            ReplicationMode::Performance => self.start_performance_replication().await,
            ReplicationMode::DisasterRecovery => self.start_dr_replication().await,
        }
    }

    /// Replicate a single operation to secondaries
    pub async fn replicate_operation(
        &self,
        operation: ReplicationOperation,
    ) -> Result<(), SecretonError> {
        let secondaries = self.secondaries.read().await;

        for secondary in secondaries.iter() {
            self.send_to_secondary(secondary, &operation).await?;
        }

        Ok(())
    }

    /// Get current replication lag
    pub async fn get_lag(&self) -> Duration {
        *self.replication_lag.read().await
    }

    /// Promote secondary to primary (DR failover)
    pub async fn promote_to_primary(&self) -> Result<(), SecretonError> {
        if !matches!(self.mode, ReplicationMode::DisasterRecovery) {
            return Err(SecretonError::InvalidOperation(
                "Can only promote DR secondaries".into()
            ));
        }

        // 1. Stop replication
        // 2. Unseal if sealed
        // 3. Update cluster metadata
        // 4. Start accepting writes

        Ok(())
    }
}
```

#### 3.2.2 Performance Replication

```rust
// crates/replication/src/performance.rs

impl ReplicationManager {
    async fn start_performance_replication(&self) -> Result<(), SecretonError> {
        // Start replication loop
        tokio::spawn({
            let storage = self.storage.clone();
            let secondaries = self.secondaries.clone();
            let lag = self.replication_lag.clone();

            async move {
                loop {
                    // 1. Get latest operations from Raft log
                    let operations = storage.get_operations_since_index(last_index).await?;

                    // 2. Send to all secondaries
                    for op in operations {
                        let start = Instant::now();

                        for secondary in secondaries.read().await.iter() {
                            send_operation(secondary, &op).await?;
                        }

                        // 3. Update lag metric
                        *lag.write().await = start.elapsed();
                    }

                    // 4. Sleep before next iteration
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Ok(())
    }
}
```

#### 3.2.3 Standby Nodes

```rust
// crates/replication/src/standby.rs

pub struct StandbyNode {
    is_active: Arc<RwLock<bool>>,
    cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    storage: Arc<dyn Storage>,
}

impl StandbyNode {
    /// Handle read request (served locally)
    pub async fn handle_read(&self, path: &str) -> Result<Vec<u8>, SecretonError> {
        // 1. Check cache first
        if let Some(cached) = self.cache.read().await.get(path) {
            return Ok(cached.clone());
        }

        // 2. Read from local storage
        let data = self.storage.get(path).await?;

        // 3. Update cache
        self.cache.write().await.put(path.to_string(), data.clone());

        Ok(data)
    }

    /// Handle write request (forward to active)
    pub async fn handle_write(&self, request: WriteRequest) -> Result<WriteResponse, SecretonError> {
        if *self.is_active.read().await {
            // This node is active, handle locally
            self.storage.put(&request.path, &request.data).await?;
            Ok(WriteResponse { success: true })
        } else {
            // Forward to active node
            self.forward_to_active(request).await
        }
    }

    /// Promote this standby to active
    pub async fn promote_to_active(&self) -> Result<(), SecretonError> {
        *self.is_active.write().await = true;

        // Update Raft cluster state
        // Start accepting writes
        // Notify other nodes

        Ok(())
    }
}
```

### 3.3 Backup Component

#### 3.3.1 Backup Manager

```rust
// crates/backup/src/lib.rs

use cron::Schedule;
use std::str::FromStr;

pub struct BackupManager {
    scheduler: Schedule,
    storage_backend: Arc<dyn BackupStorage>,
    encryption_key: Vec<u8>,
    retention_days: u32,
}

pub trait BackupStorage: Send + Sync {
    async fn upload(&self, backup: &Backup) -> Result<(), SecretonError>;
    async fn download(&self, backup_id: &str) -> Result<Backup, SecretonError>;
    async fn list(&self) -> Result<Vec<BackupMetadata>, SecretonError>;
    async fn delete(&self, backup_id: &str) -> Result<(), SecretonError>;
}

pub struct Backup {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub raft_snapshot: Vec<u8>,
    pub postgres_dump: Vec<u8>,
    pub metadata: BackupMetadata,
}

impl BackupManager {
    pub async fn new(config: BackupConfig) -> Result<Self, SecretonError> {
        let scheduler = Schedule::from_str(&config.cron_schedule)
            .map_err(|e| SecretonError::InvalidConfig(format!("Invalid cron: {}", e)))?;

        let storage_backend = create_backup_storage(&config.storage_type, &config.storage_config).await?;

        Ok(Self {
            scheduler,
            storage_backend,
            encryption_key: config.encryption_key,
            retention_days: config.retention_days,
        })
    }

    /// Start automated backup scheduler
    pub async fn start(&self) -> Result<(), SecretonError> {
        tokio::spawn({
            let scheduler = self.scheduler.clone();
            let manager = self.clone();

            async move {
                loop {
                    let next = scheduler.upcoming(Utc).next().unwrap();
                    let duration = (next - Utc::now()).to_std().unwrap();

                    tokio::time::sleep(duration).await;

                    if let Err(e) = manager.create_backup().await {
                        tracing::error!("Backup failed: {}", e);
                        // Send alert
                    }
                }
            }
        });

        Ok(())
    }

    /// Create a backup
    pub async fn create_backup(&self) -> Result<String, SecretonError> {
        let backup_id = Uuid::new_v4().to_string();

        // 1. Create Raft snapshot
        let raft_snapshot = self.create_raft_snapshot().await?;

        // 2. Dump PostgreSQL
        let postgres_dump = self.dump_postgres().await?;

        // 3. Encrypt backup
        let encrypted_raft = self.encrypt(&raft_snapshot)?;
        let encrypted_postgres = self.encrypt(&postgres_dump)?;

        // 4. Create backup object
        let backup = Backup {
            id: backup_id.clone(),
            timestamp: Utc::now(),
            raft_snapshot: encrypted_raft,
            postgres_dump: encrypted_postgres,
            metadata: BackupMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                size_bytes: raft_snapshot.len() + postgres_dump.len(),
            },
        };

        // 5. Upload to storage
        self.storage_backend.upload(&backup).await?;

        // 6. Verify backup
        self.verify_backup(&backup_id).await?;

        // 7. Clean up old backups
        self.cleanup_old_backups().await?;

        Ok(backup_id)
    }

    /// Restore from backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<(), SecretonError> {
        // 1. Download backup
        let backup = self.storage_backend.download(backup_id).await?;

        // 2. Decrypt
        let raft_snapshot = self.decrypt(&backup.raft_snapshot)?;
        let postgres_dump = self.decrypt(&backup.postgres_dump)?;

        // 3. Restore Raft snapshot
        self.restore_raft_snapshot(&raft_snapshot).await?;

        // 4. Restore PostgreSQL
        self.restore_postgres(&postgres_dump).await?;

        Ok(())
    }
}
```

### 3.4 Agent/Sidecar Component

#### 3.4.1 Agent Main

```rust
// crates/agent/src/main.rs

use clap::Parser;

#[derive(Parser)]
struct AgentArgs {
    /// Run mode: init or sidecar
    #[arg(long, default_value = "init")]
    mode: AgentMode,

    /// Secreton endpoint
    #[arg(long, env = "SECRETON_ADDR")]
    secreton_addr: String,

    /// Kubernetes ServiceAccount token path
    #[arg(long, default_value = "/var/run/secrets/kubernetes.io/serviceaccount/token")]
    sa_token_path: String,

    /// Secret paths to fetch (comma-separated)
    #[arg(long, env = "SECRETON_SECRETS")]
    secrets: String,

    /// Output directory
    #[arg(long, default_value = "/vault/secrets")]
    output_dir: String,

    /// Template file (optional)
    #[arg(long)]
    template: Option<String>,

    /// Renewal interval (seconds)
    #[arg(long, default_value = "300")]
    renewal_interval: u64,
}

#[derive(Clone)]
enum AgentMode {
    Init,     // Fetch once and exit
    Sidecar,  // Fetch and auto-renew
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AgentArgs::parse();

    // 1. Authenticate with Kubernetes ServiceAccount
    let token = std::fs::read_to_string(&args.sa_token_path)?;
    let auth_response = authenticate_with_k8s(&args.secreton_addr, &token).await?;

    // 2. Create Secreton client
    let client = create_secreton_client(&args.secreton_addr, &auth_response.token).await?;

    // 3. Parse secret paths
    let secret_paths: Vec<&str> = args.secrets.split(',').collect();

    match args.mode {
        AgentMode::Init => {
            // Fetch secrets once and exit
            fetch_and_write_secrets(&client, &secret_paths, &args.output_dir, args.template.as_deref()).await?;
            println!("Secrets fetched successfully");
        }
        AgentMode::Sidecar => {
            // Fetch and auto-renew
            run_sidecar(&client, &secret_paths, &args.output_dir, args.template.as_deref(), args.renewal_interval).await?;
        }
    }

    Ok(())
}
```

#### 3.4.2 Template Rendering

```rust
// crates/agent/src/template.rs

use handlebars::Handlebars;
use serde_json::json;

pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }

    /// Render template with secrets
    pub fn render(
        &self,
        template: &str,
        secrets: &HashMap<String, String>,
    ) -> Result<String, SecretonError> {
        let data = json!({
            "secrets": secrets,
            "env": std::env::vars().collect::<HashMap<_, _>>(),
        });

        self.handlebars
            .render_template(template, &data)
            .map_err(|e| SecretonError::TemplateRenderFailed(e.to_string()))
    }
}

// Example template:
// database:
//   host: {{ env.DB_HOST }}
//   port: {{ env.DB_PORT }}
//   username: {{ secrets.database/username }}
//   password: {{ secrets.database/password }}
```

#### 3.4.3 Auto-Renewal

```rust
// crates/agent/src/renew.rs

pub async fn run_sidecar(
    client: &SecretonClient,
    secret_paths: &[&str],
    output_dir: &str,
    template: Option<&str>,
    renewal_interval: u64,
) -> Result<(), SecretonError> {
    // Initial fetch
    fetch_and_write_secrets(client, secret_paths, output_dir, template).await?;

    // Auto-renewal loop
    loop {
        tokio::time::sleep(Duration::from_secs(renewal_interval)).await;

        // Check if secrets need renewal
        for path in secret_paths {
            let metadata = client.get_secret_metadata(path).await?;

            // Renew if TTL < 50% remaining
            if metadata.ttl_remaining < metadata.ttl / 2 {
                tracing::info!("Renewing secret: {}", path);

                // Renew lease
                client.renew_lease(&metadata.lease_id).await?;

                // Fetch updated secret
                let secret = client.get_secret(path).await?;

                // Write to file
                write_secret_to_file(output_dir, path, &secret, template).await?;

                tracing::info!("Secret renewed: {}", path);
            }
        }
    }
}
```

### 3.5 Kubernetes Operator Component

#### 3.5.1 SecretonSecret CRD

```rust
// crates/operator/src/crd.rs

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "secreton.kejaksaan.go.id",
    version = "v1",
    kind = "SecretonSecret",
    namespaced
)]
pub struct SecretonSecretSpec {
    /// Secreton endpoint
    pub secreton_addr: String,

    /// Authentication method
    pub auth: AuthMethod,

    /// Secret paths to sync
    pub secrets: Vec<SecretMapping>,

    /// Refresh interval (seconds)
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,

    /// Target Kubernetes Secret name
    pub target_secret_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum AuthMethod {
    Kubernetes {
        role: String,
        service_account: String,
    },
    Token {
        secret_name: String,
        secret_key: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SecretMapping {
    /// Secreton path
    pub path: String,

    /// Key in Kubernetes Secret
    pub key: String,

    /// Transformation (optional)
    pub transform: Option<SecretTransform>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum SecretTransform {
    Base64Encode,
    Base64Decode,
    JsonExtract { field: String },
}

fn default_refresh_interval() -> u64 {
    300 // 5 minutes
}
```

#### 3.5.2 Operator Controller

```rust
// crates/operator/src/controller.rs

use kube::{
    api::{Api, Patch, PatchParams},
    runtime::controller::{Action, Controller},
    Client, ResourceExt,
};
use k8s_openapi::api::core::v1::Secret;
use std::sync::Arc;

pub struct Context {
    pub client: Client,
}

pub async fn reconcile(
    secreton_secret: Arc<SecretonSecret>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    let client = &ctx.client;
    let namespace = secreton_secret.namespace().unwrap();

    tracing::info!(
        "Reconciling SecretonSecret: {}/{}",
        namespace,
        secreton_secret.name_any()
    );

    // 1. Authenticate with Secreton
    let secreton_client = authenticate_with_secreton(&secreton_secret.spec.auth, client).await?;

    // 2. Fetch secrets from Secreton
    let mut secret_data = BTreeMap::new();

    for mapping in &secreton_secret.spec.secrets {
        let value = secreton_client.get_secret(&mapping.path).await?;

        // Apply transformation
        let transformed = match &mapping.transform {
            Some(SecretTransform::Base64Encode) => base64::encode(&value),
            Some(SecretTransform::Base64Decode) => {
                String::from_utf8(base64::decode(&value)?)?
            }
            Some(SecretTransform::JsonExtract { field }) => {
                let json: serde_json::Value = serde_json::from_slice(&value)?;
                json[field].as_str().unwrap_or_default().to_string()
            }
            None => String::from_utf8(value)?,
        };

        secret_data.insert(mapping.key.clone(), transformed);
    }

    // 3. Create/update Kubernetes Secret
    let k8s_secret = Secret {
        metadata: ObjectMeta {
            name: Some(secreton_secret.spec.target_secret_name.clone()),
            namespace: Some(namespace.clone()),
            owner_references: Some(vec![secreton_secret.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        string_data: Some(secret_data),
        ..Default::default()
    };

    let secrets_api: Api<Secret> = Api::namespaced(client.clone(), &namespace);

    secrets_api
        .patch(
            &secreton_secret.spec.target_secret_name,
            &PatchParams::apply("secreton-operator"),
            &Patch::Apply(&k8s_secret),
        )
        .await?;

    // 4. Emit event
    emit_event(
        client,
        &secreton_secret,
        "SecretSynced",
        "Successfully synced secrets from Secreton",
    )
    .await?;

    // 5. Requeue after refresh interval
    Ok(Action::requeue(Duration::from_secs(
        secreton_secret.spec.refresh_interval,
    )))
}
```

### 3.6 KMIP Secrets Engine

#### 3.6.1 KMIP Server

```rust
// crates/kmip/src/lib.rs

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub struct KmipServer {
    listener: TcpListener,
    tls_acceptor: TlsAcceptor,
    key_store: Arc<dyn KeyStore>,
}

impl KmipServer {
    pub async fn new(config: KmipConfig) -> Result<Self, SecretonError> {
        let listener = TcpListener::bind(&config.bind_address).await?;

        let tls_config = load_tls_config(&config.tls_cert, &config.tls_key, &config.client_ca)?;
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        Ok(Self {
            listener,
            tls_acceptor,
            key_store: create_key_store(config.storage).await?,
        })
    }

    pub async fn serve(&self) -> Result<(), SecretonError> {
        loop {
            let (stream, addr) = self.listener.accept().await?;

            tracing::info!("KMIP connection from: {}", addr);

            let tls_stream = self.tls_acceptor.accept(stream).await?;

            let key_store = self.key_store.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_kmip_connection(tls_stream, key_store).await {
                    tracing::error!("KMIP connection error: {}", e);
                }
            });
        }
    }
}
```

#### 3.6.2 KMIP Protocol Implementation

```rust
// crates/kmip/src/protocol.rs

use kmip_protocol::{Request, Response, Operation, ObjectType};

pub async fn handle_kmip_connection(
    mut stream: TlsStream<TcpStream>,
    key_store: Arc<dyn KeyStore>,
) -> Result<(), SecretonError> {
    loop {
        // 1. Read KMIP request
        let request = read_kmip_request(&mut stream).await?;

        // 2. Process request
        let response = match request.operation {
            Operation::Create => handle_create(request, &key_store).await?,
            Operation::Get => handle_get(request, &key_store).await?,
            Operation::Destroy => handle_destroy(request, &key_store).await?,
            Operation::Register => handle_register(request, &key_store).await?,
            Operation::Activate => handle_activate(request, &key_store).await?,
            Operation::Revoke => handle_revoke(request, &key_store).await?,
            _ => Response::error("Unsupported operation"),
        };

        // 3. Write KMIP response
        write_kmip_response(&mut stream, &response).await?;

        // 4. Audit log
        audit_kmip_operation(&request, &response).await;
    }
}

async fn handle_create(
    request: Request,
    key_store: &Arc<dyn KeyStore>,
) -> Result<Response, SecretonError> {
    match request.object_type {
        ObjectType::SymmetricKey => {
            // Generate symmetric key
            let key = generate_symmetric_key(request.key_length)?;
            let key_id = key_store.store_key(key).await?;

            Ok(Response::success(key_id))
        }
        ObjectType::Certificate => {
            // Store certificate
            let cert = request.certificate.ok_or(SecretonError::InvalidRequest)?;
            let cert_id = key_store.store_certificate(cert).await?;

            Ok(Response::success(cert_id))
        }
        _ => Ok(Response::error("Unsupported object type")),
    }
}
```

### 3.7 Observability Component

#### 3.7.1 OpenTelemetry Integration

```rust
// crates/observability/src/otel.rs

use opentelemetry::{
    global,
    sdk::{
        trace::{self, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry(config: &ObservabilityConfig) -> Result<(), SecretonError> {
    // 1. Initialize OpenTelemetry tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otlp_endpoint),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                    config.sample_rate,
                ))))
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "secreton"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // 2. Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(())
}
```

#### 3.7.2 Health Checks

```rust
// crates/observability/src/health.rs

use axum::{Json, response::IntoResponse, http::StatusCode};

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub sealed: bool,
    pub initialized: bool,
    pub version: String,
    pub storage_backend: StorageBackendHealth,
    pub raft_cluster: Option<RaftClusterHealth>,
    pub replication: Option<ReplicationHealth>,
    pub auto_unseal: Option<AutoUnsealHealth>,
}

#[derive(Serialize)]
pub struct StorageBackendHealth {
    pub status: String,
    pub latency_ms: u64,
}

#[derive(Serialize)]
pub struct RaftClusterHealth {
    pub leader: String,
    pub peers: Vec<String>,
    pub healthy_peers: usize,
}

#[derive(Serialize)]
pub struct ReplicationHealth {
    pub mode: String,
    pub lag_ms: u64,
    pub secondaries: Vec<SecondaryHealth>,
}

#[derive(Serialize)]
pub struct SecondaryHealth {
    pub id: String,
    pub status: String,
    pub lag_ms: u64,
}

#[derive(Serialize)]
pub struct AutoUnsealHealth {
    pub provider: String,
    pub status: String,
}

pub async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let health = HealthStatus {
        status: if state.is_sealed() { "sealed" } else { "healthy" },
        sealed: state.is_sealed(),
        initialized: state.is_initialized(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        storage_backend: check_storage_health(&state.storage).await,
        raft_cluster: check_raft_health(&state.raft).await,
        replication: check_replication_health(&state.replication).await,
        auto_unseal: check_auto_unseal_health(&state.auto_unseal).await,
    };

    let status_code = if health.sealed {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    (status_code, Json(health))
}

// Kubernetes probes
pub async fn liveness_handler() -> impl IntoResponse {
    // Check if process is alive
    StatusCode::OK
}

pub async fn readiness_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if ready to serve requests
    if state.is_sealed() {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    }
}

pub async fn startup_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if initialization complete
    if state.is_initialized() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
```

## 4. Data Models

### 4.1 Auto-Unseal Configuration

```rust
// Configuration stored in secreton.toml
#[derive(Deserialize, Serialize)]
pub struct AutoUnsealConfig {
    pub enabled: bool,
    pub provider: AutoUnsealProviderConfig,
    pub fallback_to_manual: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AutoUnsealProviderConfig {
    AwsKms {
        key_id: String,
        region: String,
        endpoint: Option<String>,
    },
    GcpKms {
        key_name: String,
        project: String,
        location: String,
        key_ring: String,
    },
    AzureKeyVault {
        vault_name: String,
        key_name: String,
        tenant_id: String,
    },
    Transit {
        endpoint: String,
        key_name: String,
        token: String,
    },
}
```

### 4.2 Sealed Master Key Storage

```sql
-- PostgreSQL table for sealed master key
CREATE TABLE sealed_master_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    encrypted_key BYTEA NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    provider_key_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_active_key UNIQUE (id) WHERE id = '00000000-0000-0000-0000-000000000001'
);

-- Only one active sealed master key at a time
-- id = '00000000-0000-0000-0000-000000000001' for active key
```

### 4.3 Replication Metadata

```sql
-- Replication cluster metadata
CREATE TABLE replication_clusters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_name VARCHAR(255) NOT NULL UNIQUE,
    mode VARCHAR(50) NOT NULL, -- 'performance' or 'dr'
    is_primary BOOLEAN NOT NULL DEFAULT false,
    primary_endpoint VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Secondary nodes
CREATE TABLE replication_secondaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_id UUID NOT NULL REFERENCES replication_clusters(id),
    node_id VARCHAR(255) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    last_sync_at TIMESTAMPTZ,
    lag_ms BIGINT,
    status VARCHAR(50) NOT NULL, -- 'healthy', 'lagging', 'disconnected', 'failed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(cluster_id, node_id)
);

-- Replication log (for tracking operations)
CREATE TABLE replication_log (
    id BIGSERIAL PRIMARY KEY,
    operation_type VARCHAR(50) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    data BYTEA,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    replicated_to_secondaries BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX idx_replication_log_timestamp ON replication_log(timestamp);
CREATE INDEX idx_replication_log_replicated ON replication_log(replicated_to_secondaries);
```

### 4.4 Backup Metadata

```sql
-- Backup metadata
CREATE TABLE backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_type VARCHAR(50) NOT NULL, -- 'scheduled', 'manual'
    raft_snapshot_path VARCHAR(1024) NOT NULL,
    postgres_dump_path VARCHAR(1024) NOT NULL,
    size_bytes BIGINT NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT true,
    verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_backups_created_at ON backups(created_at);
CREATE INDEX idx_backups_expires_at ON backups(expires_at);
```

### 4.5 KMIP Key Storage

```sql
-- KMIP managed keys
CREATE TABLE kmip_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id VARCHAR(255) NOT NULL UNIQUE,
    object_type VARCHAR(50) NOT NULL, -- 'symmetric_key', 'certificate', etc.
    key_material BYTEA NOT NULL,
    key_length INTEGER,
    algorithm VARCHAR(50),
    state VARCHAR(50) NOT NULL, -- 'pre_active', 'active', 'deactivated', 'destroyed'
    scope VARCHAR(255), -- 'vmware', 'netapp', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_at TIMESTAMPTZ,
    deactivated_at TIMESTAMPTZ,
    destroyed_at TIMESTAMPTZ
);

CREATE INDEX idx_kmip_keys_key_id ON kmip_keys(key_id);
CREATE INDEX idx_kmip_keys_scope ON kmip_keys(scope);
CREATE INDEX idx_kmip_keys_state ON kmip_keys(state);
```

## 5. Correctness Properties

### 5.1 What Are Correctness Properties?

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### 5.2 Auto-Unseal Properties

**Property 1: Auto-unseal round trip**
*For any* master key, encrypting it with an auto-unseal provider then decrypting it should produce the original master key
**Validates: Requirements 2.1.1, 2.1.2, 2.1.3, 2.1.4**

**Property 2: Auto-unseal configuration persistence**
*For any* valid auto-unseal configuration, writing it to secreton.toml then loading it should produce an equivalent configuration
**Validates: Requirements 2.1.5**

**Property 3: Auto-unseal fallback**
*For any* auto-unseal failure, the system should transition to manual unseal mode and remain functional
**Validates: Requirements 2.1.6**

**Property 4: Unseal audit logging**
*For any* unseal attempt (auto or manual, successful or failed), an audit log entry should be created with the attempt details
**Validates: Requirements 2.1.7**

**Property 5: Health endpoint auto-unseal status**
*For any* health check request, the response should include auto-unseal provider information and status
**Validates: Requirements 2.1.9**

### 5.3 Replication Properties

**Property 6: Performance replication read locality**
*For any* read request to a performance replica, it should be served locally without forwarding to the primary
**Validates: Requirements 2.2.2**

**Property 7: Performance replication write forwarding**
*For any* write request to a performance replica, it should be forwarded to the primary cluster
**Validates: Requirements 2.2.3**

**Property 8: Replication lag monitoring**
*For any* replicated operation, the replication lag metric should be updated to reflect the time between primary write and secondary application
**Validates: Requirements 2.2.4**

**Property 9: Replication pause/resume without data loss**
*For any* sequence of operations during replication pause, resuming replication should eventually replicate all operations
**Validates: Requirements 2.2.5**

**Property 10: Last-write-wins conflict resolution**
*For any* conflicting writes to the same path, the write with the latest timestamp should be the final value
**Validates: Requirements 2.2.6**

**Property 11: Namespace-filtered replication**
*For any* replication configuration with namespace filters, only operations in matching namespaces should be replicated
**Validates: Requirements 2.2.7**

**Property 12: Replication requires mTLS**
*For any* replication connection attempt without mTLS, the connection should be rejected
**Validates: Requirements 2.2.8**

**Property 13: Automatic failover on primary failure**
*For any* primary cluster failure, a secondary should be automatically promoted within the RTO window
**Validates: Requirements 2.2.10**

**Property 14: DR replication completeness**
*For any* data type (secrets, tokens, leases, policies), creating it on the primary should result in it appearing on the DR secondary
**Validates: Requirements 2.3.1**

**Property 15: DR secondary sealed state**
*For any* DR secondary node, it should report sealed=true in health checks until explicitly promoted
**Validates: Requirements 2.3.2**

**Property 16: DR promotion state transition**
*For any* DR secondary promotion, the node should transition to is_primary=true and start accepting writes
**Validates: Requirements 2.3.5**

**Property 17: Primary demotion**
*For any* primary node demotion, it should transition to is_primary=false and stop accepting writes
**Validates: Requirements 2.3.6**

**Property 18: Audit log replication**
*For any* audit event on the primary, it should appear in the DR secondary's audit log
**Validates: Requirements 2.3.7**

**Property 19: RPO compliance**
*For any* operation on the primary, it should appear on the DR secondary within 1 minute
**Validates: Requirements 2.3.8**

**Property 20: RTO compliance**
*For any* primary failure, DR promotion should complete within 5 minutes
**Validates: Requirements 2.3.9**

### 5.4 Standby Node Properties

**Property 21: Standby read serving**
*For any* read request to a standby node, it should be served locally without forwarding
**Validates: Requirements 2.4.1**

**Property 22: Standby write forwarding**
*For any* write request to a standby node, it should be forwarded to the active node
**Validates: Requirements 2.4.2**

**Property 23: Standby cache effectiveness**
*For any* frequently accessed secret, subsequent reads from a standby should have lower latency (cache hits)
**Validates: Requirements 2.4.3**

**Property 24: Standby automatic promotion**
*For any* active node failure, a standby should be automatically promoted to active
**Validates: Requirements 2.4.5**

**Property 25: Standby Raft participation**
*For any* standby node, it should be a member of the Raft cluster and participate in consensus
**Validates: Requirements 2.4.6**

**Property 26: Health check role distinction**
*For any* health check on active vs standby nodes, the response should correctly indicate the node's role
**Validates: Requirements 2.4.9**

### 5.5 Backup Properties

**Property 27: Scheduled backup execution**
*For any* configured backup schedule, backups should be created at the scheduled times
**Validates: Requirements 2.5.1**

**Property 28: Backup completeness**
*For any* backup, it should contain both Raft snapshot data and PostgreSQL dump data
**Validates: Requirements 2.5.2**

**Property 29: Backup encryption**
*For any* backup, attempting to read it without the encryption key should fail
**Validates: Requirements 2.5.3**

**Property 30: Backup storage**
*For any* created backup, it should be uploaded to the configured S3-compatible storage
**Validates: Requirements 2.5.4**

**Property 31: Backup retention**
*For any* backup older than the retention period, it should be automatically deleted
**Validates: Requirements 2.5.5**

**Property 32: Point-in-time recovery**
*For any* backup, restoring it should produce a system state matching the time the backup was created
**Validates: Requirements 2.5.7**

**Property 33: Automatic backup verification**
*For any* created backup, the system should automatically verify its integrity
**Validates: Requirements 2.5.8**

**Property 34: Backup failure alerting**
*For any* backup failure, an alert should be triggered
**Validates: Requirements 2.5.9**

### 5.6 Observability Properties

**Property 35: OpenTelemetry trace export**
*For any* operation, a corresponding trace should be exported to the OpenTelemetry backend
**Validates: Requirements 2.6.1**

**Property 36: Prometheus metrics collection**
*For any* operation, corresponding Prometheus metrics should be incremented
**Validates: Requirements 2.6.2**

**Property 37: Correlation ID propagation**
*For any* operation, all log entries related to it should contain the same correlation ID
**Validates: Requirements 2.6.4**

**Property 38: Performance metrics recording**
*For any* operation, latency, throughput, and error rate metrics should be recorded
**Validates: Requirements 2.6.5**

**Property 39: Security metrics recording**
*For any* security event (failed auth, policy violation), corresponding metrics should be incremented
**Validates: Requirements 2.6.6**

**Property 40: Audit log SIEM export**
*For any* audit event, it should be exported to configured SIEM systems
**Validates: Requirements 2.6.8**

**Property 41: Critical condition alerting**
*For any* critical condition (high error rate, resource exhaustion), an alert should be triggered
**Validates: Requirements 2.6.9**

**Property 42: Detailed health status**
*For any* health check request, the response should include storage, Raft, replication, and auto-unseal status
**Validates: Requirements 2.6.10**

### 5.7 Secrets Rotation Properties

**Property 43: Automatic database credential rotation**
*For any* database credential with rotation enabled, it should be automatically rotated according to the schedule
**Validates: Requirements 2.7.1**

**Property 44: API key rotation**
*For any* API key with rotation enabled, it should be rotated according to the configured schedule
**Validates: Requirements 2.7.2**

**Property 45: Certificate auto-renewal**
*For any* certificate approaching expiry, it should be automatically renewed before expiration
**Validates: Requirements 2.7.3**

**Property 46: Grace period validity**
*For any* rotated credential, both old and new credentials should be valid during the grace period
**Validates: Requirements 2.7.5**

**Property 47: Rotation audit logging**
*For any* secret rotation, an audit event should be created
**Validates: Requirements 2.7.6**

**Property 48: Rotation failure alerting**
*For any* rotation failure, an alert should be triggered
**Validates: Requirements 2.7.7**

**Property 49: Webhook notification on rotation**
*For any* secret rotation, configured webhooks should be called with rotation details
**Validates: Requirements 2.7.10**

### 5.8 Caching Properties

**Property 50: Cache hit performance**
*For any* cached secret, subsequent reads should have lower latency than the initial read
**Validates: Requirements 2.8.1**

**Property 51: Cache TTL expiration**
*For any* cached entry, it should be evicted after the configured TTL expires
**Validates: Requirements 2.8.2**

**Property 52: Cache invalidation on update**
*For any* secret update, the cached value should be immediately invalidated
**Validates: Requirements 2.8.3**

**Property 53: Cache metrics tracking**
*For any* cache access, hit/miss metrics should be updated
**Validates: Requirements 2.8.4**

**Property 54: Cache size limits**
*For any* cache configuration, the cache size should not exceed the configured limit
**Validates: Requirements 2.8.5**

**Property 55: LRU eviction**
*For any* cache at capacity, adding a new entry should evict the least recently used entry
**Validates: Requirements 2.8.6**

**Property 56: Sensitive operation cache bypass**
*For any* sensitive operation (write, delete, policy change), it should bypass the cache and hit storage directly
**Validates: Requirements 2.8.8**

**Property 57: Distributed cache consistency**
*For any* cached value in a multi-node cluster, all nodes should eventually see the same value
**Validates: Requirements 2.8.9**

### 5.9 Request Forwarding Properties

**Property 58: Transparent write forwarding**
*For any* write request to a standby node, it should be forwarded to active and succeed without client awareness
**Validates: Requirements 2.9.1, 2.9.2**

**Property 59: Authentication context preservation**
*For any* forwarded request, the authentication context should be preserved from client to active node
**Validates: Requirements 2.9.3**

**Property 60: Forwarding uses mTLS**
*For any* forwarded request, the connection between standby and active should use mTLS
**Validates: Requirements 2.9.4**

**Property 61: Forwarding timeout enforcement**
*For any* forwarded request exceeding the timeout, it should fail with a timeout error
**Validates: Requirements 2.9.5**

**Property 62: Circular forwarding prevention**
*For any* forwarding scenario, circular forwarding loops should be detected and prevented
**Validates: Requirements 2.9.8**

**Property 63: Forwarding rate limiting**
*For any* forwarded requests, rate limits should be applied and enforced
**Validates: Requirements 2.9.9**

### 5.10 Health Check Properties

**Property 64: Liveness probe process health**
*For any* liveness probe request, it should return success if the process is running
**Validates: Requirements 2.10.2**

**Property 65: Readiness probe seal status**
*For any* readiness probe request, it should return success only if the node is unsealed
**Validates: Requirements 2.10.3**

**Property 66: Startup probe initialization**
*For any* startup probe request, it should return success only after initialization is complete
**Validates: Requirements 2.10.4**

**Property 67: Health check non-blocking**
*For any* health check request under load, it should respond within a reasonable time without blocking
**Validates: Requirements 2.10.8**

### 5.11 Agent/Sidecar Properties

**Property 68: Sidecar Kubernetes authentication**
*For any* sidecar with a valid ServiceAccount token, it should successfully authenticate with Secreton
**Validates: Requirements 2.11.1**

**Property 69: Sidecar secret fetching**
*For any* configured secret path, the sidecar should fetch it and write it to the shared volume
**Validates: Requirements 2.11.2**

**Property 70: Sidecar auto-renewal**
*For any* secret approaching expiry, the sidecar should automatically renew it before expiration
**Validates: Requirements 2.11.3**

**Property 71: Sidecar template rendering**
*For any* template with secret placeholders, the sidecar should render it with actual secret values
**Validates: Requirements 2.11.4**

**Property 72: Sidecar multi-path support**
*For any* sidecar configuration with multiple secret paths, all paths should be fetched and written
**Validates: Requirements 2.11.6**

**Property 73: Sidecar graceful degradation**
*For any* Secreton unavailability, the sidecar should retry with backoff and not crash
**Validates: Requirements 2.11.8**

### 5.12 Kubernetes Operator Properties

**Property 74: Operator CRD reconciliation**
*For any* SecretonSecret CRD creation or update, the operator should reconcile it and create/update the corresponding Kubernetes Secret
**Validates: Requirements 2.12.1, 2.12.2**

**Property 75: Operator multi-path support**
*For any* SecretonSecret CRD with multiple secret paths, all paths should be synced to the Kubernetes Secret
**Validates: Requirements 2.12.3**

**Property 76: Operator auto-refresh**
*For any* secret with TTL, the operator should automatically refresh it before expiry
**Validates: Requirements 2.12.4**

**Property 77: Operator namespace isolation**
*For any* SecretonSecret CRD in a namespace, the created Kubernetes Secret should be in the same namespace
**Validates: Requirements 2.12.5**

**Property 78: Operator secret transformation**
*For any* SecretonSecret CRD with transformation configuration, the operator should apply the transformation correctly
**Validates: Requirements 2.12.7**

**Property 79: Operator event emission**
*For any* sync failure, the operator should emit a Kubernetes event with error details
**Validates: Requirements 2.12.8**

### 5.13 KMIP Properties

**Property 80: KMIP key lifecycle**
*For any* KMIP key, creating it, retrieving it, and destroying it should follow the KMIP 1.4 specification
**Validates: Requirements 2.13.3, 2.13.4**

**Property 81: KMIP symmetric key operations**
*For any* symmetric key created via KMIP, it should be usable for encryption/decryption operations
**Validates: Requirements 2.13.5**

**Property 82: KMIP certificate operations**
*For any* certificate stored via KMIP, it should be retrievable with all original attributes
**Validates: Requirements 2.13.6**

**Property 83: KMIP audit integration**
*For any* KMIP operation, a corresponding audit event should be created in Secreton's audit log
**Validates: Requirements 2.13.7**

**Property 84: KMIP scope isolation**
*For any* KMIP key created in a scope, it should only be accessible within that scope
**Validates: Requirements 2.13.8**

### 5.14 Key Management Properties

**Property 85: Cloud KMS key creation**
*For any* key creation request to AWS/GCP/Azure KMS, the key should be created and its ID returned
**Validates: Requirements 2.14.1, 2.14.2, 2.14.3**

**Property 86: Cloud KMS key rotation**
*For any* key rotation request to AWS/GCP/Azure KMS, a new key version should be created
**Validates: Requirements 2.14.1, 2.14.2, 2.14.3**

**Property 87: Cloud KMS key import**
*For any* key import to a cloud provider, the key material should be securely transferred and stored
**Validates: Requirements 2.14.4**

**Property 88: Key deletion grace period**
*For any* key deletion request, the key should remain accessible during the configured grace period
**Validates: Requirements 2.14.5**

**Property 89: Key versioning**
*For any* key with multiple versions, all versions should be tracked and accessible
**Validates: Requirements 2.14.6**

**Property 90: Key usage tracking**
*For any* key usage, metrics should be recorded for audit and billing purposes
**Validates: Requirements 2.14.7**

**Property 91: Key operation audit logging**
*For any* key management operation, an audit event should be created
**Validates: Requirements 2.14.10**

## 6. Error Handling

### 6.1 Auto-Unseal Error Handling

**Error Scenarios:**

- KMS provider unavailable
- Invalid credentials/permissions
- Network timeout
- Key not found
- Decryption failure

**Handling Strategy:**

```rust
pub async fn auto_unseal(&self) -> Result<(), SecretonError> {
    match self.provider.decrypt(&sealed_key).await {
        Ok(master_key) => {
            self.load_master_key(master_key).await?;
            Ok(())
        }
        Err(e) => {
            tracing::error!("Auto-unseal failed: {}", e);
            audit_log(AuditEvent::AutoUnsealFailed { error: e.to_string() }).await;

            if self.config.fallback_to_manual {
                tracing::info!("Falling back to manual unseal");
                self.transition_to_manual_unseal().await?;
                Ok(())
            } else {
                Err(SecretonError::AutoUnsealFailed(e.to_string()))
            }
        }
    }
}
```

### 6.2 Replication Error Handling

**Error Scenarios:**

- Network partition
- Secondary node unavailable
- Replication lag exceeds threshold
- Conflicting writes
- Storage backend failure

**Handling Strategy:**

```rust
pub async fn replicate_operation(&self, op: Operation) -> Result<(), SecretonError> {
    let mut failed_secondaries = Vec::new();

    for secondary in self.secondaries.read().await.iter() {
        match self.send_to_secondary(secondary, &op).await {
            Ok(_) => {
                tracing::debug!("Replicated to {}", secondary.id);
            }
            Err(e) => {
                tracing::error!("Replication to {} failed: {}", secondary.id, e);
                failed_secondaries.push(secondary.id.clone());

                // Mark secondary as unhealthy
                self.mark_secondary_unhealthy(&secondary.id).await;

                // Trigger alert if too many failures
                if failed_secondaries.len() > self.config.max_failed_secondaries {
                    self.trigger_alert(Alert::ReplicationFailure {
                        failed_count: failed_secondaries.len(),
                    }).await;
                }
            }
        }
    }

    // Operation succeeds even if some secondaries fail (eventual consistency)
    Ok(())
}
```

### 6.3 Backup Error Handling

**Error Scenarios:**

- Insufficient disk space
- S3 upload failure
- Backup verification failure
- Encryption key unavailable
- PostgreSQL dump failure

**Handling Strategy:**

```rust
pub async fn create_backup(&self) -> Result<String, SecretonError> {
    let backup_id = Uuid::new_v4().to_string();

    // Use circuit breaker pattern
    let result = self.circuit_breaker.call(async {
        // 1. Create Raft snapshot with retry
        let raft_snapshot = retry_with_backoff(
            || self.create_raft_snapshot(),
            3, // max retries
            Duration::from_secs(5),
        ).await?;

        // 2. Dump PostgreSQL with retry
        let postgres_dump = retry_with_backoff(
            || self.dump_postgres(),
            3,
            Duration::from_secs(5),
        ).await?;

        // 3. Encrypt
        let encrypted_raft = self.encrypt(&raft_snapshot)?;
        let encrypted_postgres = self.encrypt(&postgres_dump)?;

        // 4. Upload to S3 with retry
        let backup = Backup { /* ... */ };
        retry_with_backoff(
            || self.storage_backend.upload(&backup),
            5, // more retries for network operations
            Duration::from_secs(10),
        ).await?;

        // 5. Verify
        self.verify_backup(&backup_id).await?;

        Ok(backup_id)
    }).await;

    match result {
        Ok(id) => {
            tracing::info!("Backup created successfully: {}", id);
            Ok(id)
        }
        Err(e) => {
            tracing::error!("Backup failed: {}", e);
            audit_log(AuditEvent::BackupFailed { error: e.to_string() }).await;
            self.trigger_alert(Alert::BackupFailure { error: e.to_string() }).await;
            Err(e)
        }
    }
}
```

### 6.4 Agent/Sidecar Error Handling

**Error Scenarios:**

- Secreton unavailable
- Authentication failure
- Secret not found
- Template rendering error
- Volume write failure

**Handling Strategy:**

```rust
pub async fn fetch_secrets_with_retry(&self) -> Result<(), AgentError> {
    let mut retry_count = 0;
    let max_retries = self.config.max_retries;
    let mut backoff = Duration::from_secs(1);

    loop {
        match self.fetch_secrets().await {
            Ok(_) => {
                tracing::info!("Secrets fetched successfully");
                return Ok(());
            }
            Err(e) => {
                retry_count += 1;

                if retry_count >= max_retries {
                    tracing::error!("Max retries exceeded: {}", e);
                    return Err(AgentError::MaxRetriesExceeded(e.to_string()));
                }

                tracing::warn!(
                    "Fetch failed (attempt {}/{}): {}. Retrying in {:?}",
                    retry_count,
                    max_retries,
                    e,
                    backoff
                );

                tokio::time::sleep(backoff).await;

                // Exponential backoff with jitter
                backoff = std::cmp::min(
                    backoff * 2 + Duration::from_millis(rand::random::<u64>() % 1000),
                    Duration::from_secs(60),
                );
            }
        }
    }
}
```

### 6.5 KMIP Error Handling

**Error Scenarios:**

- Invalid KMIP request
- Unsupported operation
- Key not found
- Permission denied
- TLS authentication failure

**Handling Strategy:**

```rust
pub async fn handle_kmip_request(&self, request: KmipRequest) -> KmipResponse {
    match self.process_request(request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("KMIP request failed: {}", e);

            let error_response = match e {
                KmipError::KeyNotFound => KmipResponse::error(
                    ResultReason::ItemNotFound,
                    "Key not found"
                ),
                KmipError::PermissionDenied => KmipResponse::error(
                    ResultReason::PermissionDenied,
                    "Permission denied"
                ),
                KmipError::UnsupportedOperation => KmipResponse::error(
                    ResultReason::OperationNotSupported,
                    "Operation not supported"
                ),
                _ => KmipResponse::error(
                    ResultReason::GeneralFailure,
                    "Internal error"
                ),
            };

            // Audit log
            audit_log(AuditEvent::KmipError {
                error: e.to_string(),
                request_id: request.id,
            }).await;

            error_response
        }
    }
}
```

## 7. Testing Strategy

### 7.1 Unit Testing

**Scope:** Individual components and functions

**Approach:**

- Test each auto-unseal provider independently with mocked KMS responses
- Test replication logic with in-memory storage
- Test backup encryption/decryption with known keys
- Test template rendering with various inputs
- Test KMIP protocol parsing with sample requests

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aws_kms_encrypt_decrypt() {
        let provider = MockAwsKmsProvider::new();
        let plaintext = b"test-master-key";

        let ciphertext = provider.encrypt(plaintext).await.unwrap();
        let decrypted = provider.decrypt(&ciphertext).await.unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_replication_lag_calculation() {
        let manager = ReplicationManager::new_for_test();

        let start = Instant::now();
        manager.replicate_operation(Operation::Write { /* ... */ }).await.unwrap();
        let lag = manager.get_lag().await;

        assert!(lag < Duration::from_millis(100));
    }
}
```

### 7.2 Property-Based Testing

**Scope:** Universal properties across all inputs

**Configuration:**

- Minimum 100 iterations per property test
- Use `proptest` or `quickcheck` crate
- Tag each test with feature name and property number

**Example:**

```rust
use proptest::prelude::*;

proptest! {
    /// Feature: secreton-vault-parity, Property 1: Auto-unseal round trip
    /// For any master key, encrypting then decrypting should produce the original
    #[test]
    fn prop_auto_unseal_round_trip(master_key in prop::collection::vec(any::<u8>(), 32..=32)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let provider = TransitProvider::new_for_test().await.unwrap();

            let ciphertext = provider.encrypt(&master_key).await.unwrap();
            let decrypted = provider.decrypt(&ciphertext).await.unwrap();

            prop_assert_eq!(master_key, decrypted);
        });
    }

    /// Feature: secreton-vault-parity, Property 10: Last-write-wins conflict resolution
    /// For any conflicting writes, the latest timestamp should win
    #[test]
    fn prop_last_write_wins(
        path in "[a-z]{1,10}",
        value1 in any::<Vec<u8>>(),
        value2 in any::<Vec<u8>>(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = ReplicationManager::new_for_test().await;

            // Write value1 at t1
            let t1 = Utc::now();
            manager.write(&path, &value1, t1).await.unwrap();

            // Write value2 at t2 (later)
            let t2 = t1 + Duration::from_secs(1);
            manager.write(&path, &value2, t2).await.unwrap();

            // Resolve conflicts
            manager.resolve_conflicts().await.unwrap();

            // value2 should win
            let final_value = manager.read(&path).await.unwrap();
            prop_assert_eq!(value2, final_value);
        });
    }

    /// Feature: secreton-vault-parity, Property 52: Cache invalidation on update
    /// For any secret update, the cached value should be invalidated
    #[test]
    fn prop_cache_invalidation_on_update(
        path in "[a-z]{1,10}",
        value1 in any::<Vec<u8>>(),
        value2 in any::<Vec<u8>>(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache = ResponseCache::new_for_test();

            // Read value1 (cache miss)
            cache.put(&path, &value1).await;

            // Read again (cache hit)
            let cached = cache.get(&path).await.unwrap();
            prop_assert_eq!(value1, cached);

            // Update to value2
            cache.invalidate(&path).await;
            cache.put(&path, &value2).await;

            // Read should return value2, not cached value1
            let updated = cache.get(&path).await.unwrap();
            prop_assert_eq!(value2, updated);
        });
    }
}
```

### 7.3 Integration Testing

**Scope:** Component interactions and end-to-end flows

**Approach:**

- Test auto-unseal with real KMS providers (using test accounts)
- Test replication between actual Secreton instances
- Test agent/sidecar in real Kubernetes cluster
- Test operator with real CRDs
- Test KMIP with VMware/NetApp test environments

**Example:**

```rust
#[tokio::test]
#[ignore] // Run only in CI with real infrastructure
async fn test_auto_unseal_with_aws_kms() {
    let config = load_test_config();
    let provider = AwsKmsProvider::new(
        config.aws_key_id,
        config.aws_region,
    ).await.unwrap();

    let secreton = Secreton::new_sealed(config).await.unwrap();

    // Auto-unseal should succeed
    secreton.auto_unseal(provider).await.unwrap();

    assert!(!secreton.is_sealed());
}

#[tokio::test]
#[ignore]
async fn test_performance_replication_end_to_end() {
    // Start primary cluster
    let primary = start_secreton_cluster("primary", 3).await.unwrap();

    // Start secondary cluster
    let secondary = start_secreton_cluster("secondary", 3).await.unwrap();

    // Configure replication
    primary.configure_replication(ReplicationConfig {
        mode: ReplicationMode::Performance,
        secondaries: vec![secondary.endpoint()],
    }).await.unwrap();

    // Write to primary
    primary.write("test/secret", b"value").await.unwrap();

    // Wait for replication
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Read from secondary
    let value = secondary.read("test/secret").await.unwrap();
    assert_eq!(b"value", value.as_slice());
}
```

### 7.4 Performance Testing

**Scope:** Latency, throughput, and resource usage

**Metrics:**

- Read latency p50, p95, p99
- Write latency p50, p95, p99
- Throughput (ops/sec)
- Replication lag
- Cache hit ratio
- Memory usage
- CPU usage

**Tools:**

- `criterion` for Rust benchmarks
- `k6` or `wrk` for load testing
- Prometheus for metrics collection
- Grafana for visualization

**Example:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_auto_unseal(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = rt.block_on(async {
        TransitProvider::new_for_test().await.unwrap()
    });

    c.bench_function("auto_unseal_decrypt", |b| {
        b.to_async(&rt).iter(|| async {
            let ciphertext = black_box(vec![0u8; 32]);
            provider.decrypt(&ciphertext).await.unwrap()
        });
    });
}

criterion_group!(benches, benchmark_auto_unseal);
criterion_main!(benches);
```

### 7.5 Security Testing

**Scope:** Authentication, authorization, encryption, audit logging

**Approach:**

- Test mTLS enforcement for replication
- Test auto-unseal fallback on KMS failure
- Test audit logging for all operations
- Test RBAC for KMIP operations
- Test encryption at rest and in transit
- Penetration testing (external security audit)

### 7.6 Chaos Testing

**Scope:** Resilience under failure conditions

**Scenarios:**

- Network partition between primary and secondaries
- Primary node crash during replication
- KMS provider unavailable during auto-unseal
- Disk full during backup
- PostgreSQL connection loss
- Raft leader election during high load

**Tools:**

- Chaos Mesh for Kubernetes
- `toxiproxy` for network failures
- Custom failure injection

## 8. Deployment Considerations

### 8.1 Kubernetes Deployment

**Helm Chart Structure:**

```
secreton-helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── statefulset.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── pdb.yaml
│   ├── hpa.yaml
│   ├── servicemonitor.yaml
│   ├── operator/
│   │   ├── deployment.yaml
│   │   ├── crd.yaml
│   │   └── rbac.yaml
│   └── agent/
│       ├── mutatingwebhook.yaml
│       └── configmap.yaml
```

**Key Configuration:**

```yaml
# values.yaml
replicaCount: 3

autoUnseal:
  enabled: true
  provider: aws-kms
  awsKms:
    keyId: "arn:aws:kms:us-east-1:123456789012:key/..."
    region: us-east-1

replication:
  enabled: true
  mode: performance
  secondaries:
    - endpoint: https://secreton-secondary.region2.svc.cluster.local:8200

backup:
  enabled: true
  schedule: "0 2 * * *"  # 2 AM daily
  retention: 30d
  s3:
    bucket: secreton-backups
    region: us-east-1

monitoring:
  prometheus:
    enabled: true
  opentelemetry:
    enabled: true
    endpoint: http://otel-collector:4317

agent:
  injector:
    enabled: true
    webhook:
      enabled: true

operator:
  enabled: true
  replicas: 2
```

### 8.2 Migration Path

**From Current Secreton to Vault Parity:**

1. **Phase 1: Auto-Unseal (Week 1)**
   - Deploy new Secreton version with auto-unseal support
   - Configure auto-unseal provider
   - Test auto-unseal on staging
   - Migrate production to auto-unseal

2. **Phase 2: Replication (Weeks 2-3)**
   - Deploy secondary cluster
   - Configure performance replication
   - Test replication lag
   - Enable DR replication

3. **Phase 3: Kubernetes Integration (Weeks 4-5)**
   - Deploy agent/sidecar
   - Deploy operator
   - Migrate applications to use agent
   - Migrate secrets to operator-managed

4. **Phase 4: Enterprise Features (Weeks 6-7)**
   - Enable KMIP server
   - Configure key management
   - Enable advanced monitoring
   - Configure automated backups

### 8.3 Rollback Plan

**Rollback Triggers:**

- Auto-unseal failures > 5%
- Replication lag > 1 second
- Data loss detected
- Critical security vulnerability

**Rollback Procedure:**

1. Stop new version deployment
2. Restore from latest backup
3. Revert to previous version
4. Verify data integrity
5. Resume operations
6. Post-mortem analysis

## 9. Documentation Requirements

### 9.1 User Documentation

- Auto-unseal setup guide for each provider (AWS, GCP, Azure, Transit)
- Replication configuration guide
- Backup and restore procedures
- Agent/Sidecar deployment guide
- Operator CRD reference
- KMIP integration guide for VMware/NetApp
- Troubleshooting guide

### 9.2 Operator Documentation

- Disaster recovery runbook
- Failover procedures
- Backup verification procedures
- Performance tuning guide
- Monitoring and alerting setup
- Security hardening checklist

### 9.3 Developer Documentation

- API reference for new endpoints
- gRPC proto documentation
- Architecture decision records (ADRs)
- Contributing guide for new features
- Testing guide

## 10. Success Metrics

### 10.1 Technical Metrics

- All 91 correctness properties pass property-based tests
- Code coverage > 80%
- All integration tests passing
- Performance benchmarks meet NFRs:
  - Read latency < 10ms (p99)
  - Write latency < 50ms (p99)
  - Throughput > 10,000 ops/sec per node
  - Replication lag < 100ms (p99)
- Zero critical security vulnerabilities

### 10.2 Operational Metrics

- Auto-unseal success rate > 99.9%
- Backup success rate > 99.9%
- Replication lag < 100ms (p99)
- RPO < 1 minute
- RTO < 5 minutes
- MTTR < 15 minutes
- MTBF > 720 hours

### 10.3 Business Metrics

- Production deployment at Kejaksaan RI successful
- Zero security incidents
- Zero data loss incidents
- Compliance audit passed
- User satisfaction > 4.5/5
- Support ticket volume < 5/month

---

**Document Version:** 1.0
**Last Updated:** 2026-02-12
**Status:** DRAFT - Ready for Review
**Author:** AI Agent (Kiro)
