# Secreton - Enterprise Security Vault System

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-quantum--safe-green.svg)](docs/security.md)

**Secure secrets management and encryption service** built in Rust with modern cryptographic algorithms, post-quantum cryptography support, and comprehensive API design.

> **Note**: Secreton is part of the SIMPelv2 project for the Indonesian Attorney General's Office (Kejaksaan Agung RI).

## 🔐 Core Features

- **🔒 Modern Cryptography**: AES-GCM, ChaCha20-Poly1305, Ed25519, ECDSA-P256, X25519 support
- **🔑 Transit Engine**: Encryption/decryption-as-a-service with multiple algorithms
- **📦 KV Secrets Engine**: Versioned key-value secret storage with metadata tracking
- **🌐 RESTful API**: Clean HTTP API with comprehensive endpoint coverage
- **📊 Health Monitoring**: Built-in health checks and system observability
- **🛡️ Memory Safety**: Zero unsafe code with comprehensive async support

## 🏗️ Architecture

Secreton implements a modular, memory-safe architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   HTTP Clients  │───▶│   Secreton API   │───▶│   In-Memory     │
│   (REST/JSON)   │    │   (Rust/Axum)    │    │   Storage       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │   Crypto Engine  │
                       │   (RustCrypto)   │
                       └──────────────────┘
```

### Core Components

- **Transit Engine**: Encryption/decryption service with modern cryptographic algorithms
- **KV Engine**: Versioned secret storage with audit trails and metadata tracking
- **API Layer**: RESTful HTTP interface with comprehensive endpoints
- **Crypto Core**: Modern cryptographic implementations using RustCrypto ecosystem

## 🚀 Implemented Features

### Transit Engine (Encryption Service)
- **Multiple Algorithms**: AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305
- **Asymmetric Crypto**: Ed25519, ECDSA-P256, ECDSA-Secp256k1, X25519
- **Key Management**: Secure key generation and lifecycle management
- **Base64 Encoding**: Automatic encoding/decoding for API compatibility

### KV Secrets Engine (Secret Storage)
- **Versioned Storage**: Automatic versioning with rollback capabilities
- **Metadata Tracking**: Creation/update timestamps and version history
- **Path-based Access**: Hierarchical secret organization
- **Audit Logging**: Comprehensive operation logging

### HTTP API
- **Health Monitoring**: `/health`, `/version` endpoints
- **Transit Operations**: `/v1/transit/*` - encryption/decryption operations
- **Secret Management**: `/v1/secret/*` - key-value secret operations
- **TLS Metrics**: `/metrics/tls` - connection security monitoring

## 📚 API Documentation

### System Endpoints

#### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-21T10:00:00Z",
  "version": "1.0.0"
}
```

#### Version Information
```http
GET /version
```

**Response:**
```json
{
  "version": "1.0.0",
  "build_date": "2024",
  "git_commit": "unknown"
}
```

### Transit Engine API

#### List Encryption Keys
```http
GET /v1/transit/keys
```

**Response:**
```json
{
  "keys": ["my-app-key", "database-key"]
}
```

#### Create Encryption Key
```http
POST /v1/transit/keys/{key-name}
Content-Type: application/json

{
  "key_type": "aes256-gcm"
}
```

**Supported Key Types:**
- `aes256-gcm` (default)
- `chacha20-poly1305`
- `xchacha20-poly1305`
- `ed25519`
- `ecdsa-p256`
- `ecdsa-secp256k1`
- `x25519`

#### Encrypt Data
```http
POST /v1/transit/encrypt/{key-name}
Content-Type: application/json

{
  "plaintext": "SGVsbG8gV29ybGQ="
}
```

**Response:**
```json
{
  "ciphertext": "vault:v1:randomnonce:encrypteddata"
}
```

#### Decrypt Data
```http
POST /v1/transit/decrypt/{key-name}
Content-Type: application/json

{
  "ciphertext": "vault:v1:randomnonce:encrypteddata"
}
```

**Response:**
```json
{
  "plaintext": "SGVsbG8gV29ybGQ="
}
```

### KV Secrets Engine API

#### Store Secret
```http
POST /v1/secret/data/{path}
Content-Type: application/json

{
  "data": {
    "password": "my-secret-password",
    "api_key": "abc123"
  }
}
```

**Response:**
```json
{
  "version": 1,
  "created_time": "2025-08-21T10:00:00Z"
}
```

#### Retrieve Secret
```http
GET /v1/secret/data/{path}
```

**Response:**
```json
{
  "data": {
    "password": "my-secret-password",
    "api_key": "abc123"
  },
  "metadata": {
    "created_time": "2025-08-21T10:00:00Z",
    "version": 1
  }
}
```

## 🔧 Installation & Setup

### Prerequisites

- **Rust 1.90+** - For building from source
- **OpenSSL** - For TLS/crypto functionality

### Quick Start

```bash
# 1. Clone repository
git clone https://gitlab.com/analisiskebutuhan/secreton-vault-adhyaksa.git
cd secreton-vault-adhyaksa/secreton

# 2. Build the project
cargo build --release

# 3. Start the API server
cargo run --bin api_server

# Server starts on http://127.0.0.1:8200
```

### Usage Examples

#### Using curl with the API

```bash
# Health check
curl http://127.0.0.1:8200/health

# Create an encryption key
curl -X POST http://127.0.0.1:8200/v1/transit/keys/my-app-key \
  -H "Content-Type: application/json" \
  -d '{"key_type": "aes256-gcm"}'

# Encrypt data
curl -X POST http://127.0.0.1:8200/v1/transit/encrypt/my-app-key \
  -H "Content-Type: application/json" \
  -d '{"plaintext": "SGVsbG8gV29ybGQ="}'

# Store a secret
curl -X POST http://127.0.0.1:8200/v1/secret/data/myapp \
  -H "Content-Type: application/json" \
  -d '{"data": {"password": "secret123"}}'

# Retrieve a secret
curl http://127.0.0.1:8200/v1/secret/data/myapp
```

## ⚙️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SECRETON_HOST` | `127.0.0.1` | Server bind address |
| `SECRETON_PORT` | `8200` | Server port |
| `RUST_LOG` | `info` | Log level |

### Storage Options

Secreton supports multiple storage backends with automatic encryption and caching:

**Core Backends**:
- **Memory Backend** (default): Fast in-memory storage for development and testing
- **File Backend**: Simple file-based storage for single-node deployments
- **PostgreSQL Backend** (optional): Production-ready persistent storage with `postgres` feature
- **Redis Backend** (optional): High-performance caching with `redis-backend` feature
- **Raft Consensus** (optional): Distributed consensus for high availability with `raft-consensus` feature

**Storage Wrappers**:
- **EncryptedStorage**: Automatic encryption/decryption wrapper
- **CachedStorage**: In-memory caching with TTL support

Enable features in `Cargo.toml`:
```toml
# Single backend
secreton-storage = { path = "crates/storage", features = ["postgres"] }

# Distributed cluster with Raft
secreton-storage = { path = "crates/storage", features = ["raft-consensus"] }
```

**Usage Example**:
```rust
use secreton_storage::{StorageFactory, EncryptedStorage, CachedStorage};

// Create base backend
let backend = StorageFactory::create_postgres(connection_string, None).await?;

// Wrap with encryption
let encrypted = Arc::new(EncryptedStorage::new(backend, "vault-key".to_string()));

// Wrap with caching (300 second TTL)
let cached = Arc::new(CachedStorage::new(encrypted, 300));

// Use cached + encrypted storage
cached.store(&entry).await?;
```

### Distributed Deployment with Raft

Secreton supports distributed deployment using Raft consensus algorithm for:
- **High Availability**: Automatic leader election and failover
- **Strong Consistency**: All writes go through leader with replication
- **Fault Tolerance**: Cluster continues operating with majority of nodes
- **Dynamic Membership**: Add/remove nodes without downtime

#### Raft Cluster Setup

```bash
# Node 1 (Leader)
RAFT_NODE_ID=1 RAFT_LISTEN=127.0.0.1:7001 cargo run

# Node 2
RAFT_NODE_ID=2 RAFT_LISTEN=127.0.0.1:7002 RAFT_PEERS=1:127.0.0.1:7001 cargo run

# Node 3
RAFT_NODE_ID=3 RAFT_LISTEN=127.0.0.1:7003 RAFT_PEERS=1:127.0.0.1:7001 cargo run
```

#### Cluster Management API

```bash
# Get cluster status
curl http://127.0.0.1:8200/v1/raft/status

# Add a new node
curl -X POST http://127.0.0.1:8200/v1/raft/peers \
  -H "Content-Type: application/json" \
  -d '{"node_id": 4, "address": "127.0.0.1:7004"}'

# Remove a node
curl -X DELETE http://127.0.0.1:8200/v1/raft/peers/4

# Get leader information
curl http://127.0.0.1:8200/v1/raft/leader
```

## 🔒 Security Features

### Cryptographic Algorithms
- **Symmetric Encryption**: AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305
- **Asymmetric Cryptography**: Ed25519, ECDSA-P256, ECDSA-Secp256k1, X25519
- **Post-Quantum**: ML-DSA (Dilithium), ML-KEM (Kyber), Falcon
- **Hash Functions**: SHA-256, SHA-384, SHA-512, SHA3-256, SHA3-384, SHA3-512, BLAKE3
- **Key Derivation**: PBKDF2, Argon2id, scrypt, HKDF
- **Secret Sharing**: Shamir's Secret Sharing for distributed key management

### Crypto-Storage Integration
Secreton provides seamless integration between cryptographic operations and storage:
- **Automatic Encryption**: Data encrypted before storage with versioned keys
- **Key Rotation**: Transparent key rotation without data migration
- **Re-encryption**: Automatic re-encryption with new key versions
- **Metadata Tracking**: Full audit trail of encryption operations

```rust
use secreton_crypto::CryptoStorageBridge;

// Create bridge with default key
let bridge = CryptoStorageBridge::new("vault-key".to_string()).await?;

// Encrypt data for storage
let encrypted = bridge.encrypt_for_storage(data, None).await?;

// Decrypt from storage
let decrypted = bridge.decrypt_from_storage(&encrypted).await?;

// Rotate key
bridge.rotate_key("vault-key").await?;
```
