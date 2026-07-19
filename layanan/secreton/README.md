# Secreton

Secrets management and encryption service for [SIMPEL](https://simpel.kejaksaan.go.id/) — the Indonesian Attorney General's Office (Kejaksaan RI) asset management system. Built in Rust as part of the simpelv2 monorepo.

## Overview

Secreton provides encrypted secret storage, transit encryption, PKI certificate management, dynamic credential generation, and key rotation. It exposes a REST API (Axum), gRPC API (Tonic), and a CLI tool.

**Current version:** 0.1.0
**Rust edition:** 2024 (MSRV 1.90+)
**License:** Apache-2.0

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                        API Layer                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────┐  │
│  │  REST (Axum) │   │ gRPC (Tonic) │   │   CLI (clap)     │  │
│  │  :8200       │   │ :8201        │   │  secreton-cli    │  │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────────┘  │
│         └──────────────────┼───────────────────┘              │
│                            ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Service Container (AppState)               │  │
│  │                                                         │  │
│  │  SecretService · TransitEngine · PkiEngine · SshEngine  │  │
│  │  AuthService · SealService · NamespaceService           │  │
│  │  DatabaseSecretsEngine · TotpEngine · TransformEngine   │  │
│  │  AwsEngine                                              │  │
│  │  RotationEngine · LeaseManager · PolicyService          │  │
│  │  WrappingService · MfaService                           │  │
│  │  BackupManager · AuditLogger                            │  │
│  └─────────────────────────┬───────────────────────────────┘  │
│                            ▼                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────┐  │
│  │ CryptoEngine │   │  StorageBack │   │    HsmBackend    │  │
│  │ (RustCrypto) │   │  (Raft/PG/   │   │   (PKCS#11)     │  │
│  │              │   │   Mem/File)  │   │   (optional)     │  │
│  └──────────────┘   └──────────────┘   └──────────────────┘  │
└───────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
layanan/secreton/
├── Cargo.toml              # Root package (part of simpelv2 workspace)
├── crates/
│   ├── types/              # Shared types: SecurityLevel, ResourceId, Metadata, Tags
│   ├── core/               # Business logic: audit, auth, config, namespace, PKI, rotation, resilience
│   ├── crypto/             # RustCrypto: AES-GCM, ChaCha20-Poly1305, Ed25519, ECDSA, Shamir, FPE, hybrid PQ
│   ├── storage/            # StorageBackend trait + impls: Raft, PostgreSQL, memory, encrypted, cached
│   ├── api/                # REST + handler layer (Axum 0.8), all route definitions, middleware
│   ├── grpc/               # gRPC server (Tonic 0.14), proto-generated code, TLS, interceptors
│   ├── cli/                # CLI binary (clap): seal, secret, transit, policy, audit, backup, replication
│   ├── agent/              # Sidecar agent: auto-auth, token renewal, template rendering
│   ├── hsm/                # HSM via PKCS#11: Thales Luna, AWS CloudHSM, SoftHSM2, YubiHSM2
│   ├── k8s-operator/       # Kubernetes operator: SecretSync CRD, reconciler
│   ├── auto-unseal/        # Auto-unseal providers: Transit, AWS KMS, GCP KMS, Azure Key Vault
│   ├── backup/             # Backup/restore: scheduler, S3/local storage, alerting
│   ├── health/             # Health check registry: Healthy/Degraded/Unhealthy status
│   └── replication/        # Performance + DR replication: failover, conflict resolution
├── proto/                  # Protobuf definitions (secreton.proto, common.proto)
├── migrations/             # PostgreSQL schema migrations
├── deploy/                 # Kubernetes manifests (staging + production)
├── scripts/                # Operational scripts (init-db, compliance, disaster-recovery)
├── monitoring/             # Prometheus config
├── tests/                  # Integration, security, performance, compliance test suites
└── benches/                # Criterion benchmarks (transit, storage)
```

## Features

### Secret Storage (`/v1/secret`)

Versioned key-value secret store with metadata, audit trail, and encryption at rest.

```bash
# Write a secret
curl -X POST http://localhost:8200/v1/secret/data/myapp/db \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"data":{"username":"admin","password":"s3cret"}}'

# Read a secret
curl http://localhost:8200/v1/secret/data/myapp/db \
  -H "Authorization: Bearer $TOKEN"
```

**Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/secret/data/{path}` | Create secret |
| GET | `/v1/secret/data/{path}` | Read secret |
| PUT | `/v1/secret/data/{path}` | Update secret |
| DELETE | `/v1/secret/data/{path}` | Delete secret |
| GET | `/v1/secret/secrets` | List all secrets |

### Key Management (`/v1/secret/keys`)

Encryption key lifecycle: create, rotate, list versions.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/secret/keys` | Create key (AES-256-GCM, ChaCha20-Poly1305, Ed25519, etc.) |
| GET | `/v1/secret/keys` | List keys |
| GET | `/v1/secret/keys/{key_id}` | Get key info |
| PUT | `/v1/secret/keys/{key_id}` | Update key metadata |
| DELETE | `/v1/secret/keys/{key_id}` | Delete key |
| POST | `/v1/secret/keys/{key_id}/rotate` | Rotate key |
| GET | `/v1/secret/keys/{key_id}/versions` | List versions |

### Cryptographic Operations (`/v1/secret`)

Encrypt, decrypt, sign, verify, and hash data using managed keys.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/secret/encrypt` | Encrypt data with key |
| POST | `/v1/secret/decrypt` | Decrypt data with key |
| POST | `/v1/secret/sign` | Sign data |
| POST | `/v1/secret/verify` | Verify signature |
| POST | `/v1/secret/hash` | Hash data |

### Transit Engine (`/v1/transit`)

Encryption-as-a-service with named keys.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/transit/keys` | List transit keys |
| POST | `/v1/transit/keys/{key_name}` | Create transit key |
| POST | `/v1/transit/encrypt/{key_name}` | Encrypt via transit key |
| POST | `/v1/transit/decrypt/{key_name}` | Decrypt via transit key |

### PKI Certificate Management (`/v1/pki`, `/v1/sys/pki`)

Certificate authority and certificate lifecycle management.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/pki/ca/root` | Generate root CA |
| GET | `/v1/pki/ca/list` | List CAs |
| POST | `/v1/pki/roles/{role_name}` | Create PKI role |
| GET | `/v1/pki/roles` | List PKI roles |
| POST | `/v1/pki/issue/{role_name}` | Issue certificate |
| POST | `/v1/pki/revoke` | Revoke certificate |
| GET | `/v1/pki/cert/{serial}` | Get certificate |
| GET | `/v1/pki/crl` | Get CRL |
| POST | `/v1/sys/pki/ca/intermediate` | Generate intermediate CA |
| GET | `/v1/sys/pki/ocsp/{serial}` | OCSP lookup |
| POST | `/v1/sys/pki/templates` | Create certificate template |
| GET | `/v1/sys/pki/templates` | List templates |
| GET/POST | `/v1/sys/pki/renewal/config` | Renewal configuration |
| GET | `/v1/sys/pki/renewal/check` | Check renewal status |

### SSH Certificate Signing (`/v1/sys/ssh`)

SSH CA management and certificate signing.

| Method | Path | Description |
|--------|------|-------------|
| POST/GET | `/v1/sys/ssh/ca` | Create/list SSH CAs |
| POST/GET | `/v1/sys/ssh/roles` | Create/list SSH roles |
| POST | `/v1/sys/ssh/creds/{role}` | Generate SSH credentials |
| POST | `/v1/sys/ssh/sign/{ca}/{role}` | Sign SSH user key |
| POST | `/v1/sys/ssh/sign-host/{ca}/{role}` | Sign SSH host key |
| POST | `/v1/sys/ssh/otp/generate` | Generate OTP |
| POST | `/v1/sys/ssh/otp/verify` | Verify OTP |

### Dynamic Secrets (`/v1/dynamic`)

On-demand credential generation with automatic lease-based expiration.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/dynamic/database/config/{name}` | Configure database connection |
| POST | `/v1/dynamic/database/roles/{role}` | Create credential role |
| GET | `/v1/dynamic/database/creds/{role}` | Generate database credentials |

**PostgreSQL only.** `DatabaseType` also accepts MySQL, MongoDB, Redis, Cassandra
and MSSQL, but only PostgreSQL has a connection pool behind it; every other value
fails closed with `UnsupportedDatabase` when the connection is configured.

### Authentication (`/v1/auth`)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/auth/login` | Login |
| POST | `/v1/auth/logout` | Logout |
| POST | `/v1/auth/token/refresh` | Refresh token |
| POST | `/v1/auth/token/verify` | Verify token |
| POST | `/v1/auth/mfa/setup` | Setup MFA |
| POST | `/v1/auth/mfa/verify` | Verify MFA |
| POST | `/v1/auth/mfa/disable` | Disable MFA |
| GET | `/v1/auth/oauth/{provider}` | OAuth login |
| GET | `/v1/auth/oauth/{provider}/callback` | OAuth callback |
| GET | `/v1/auth/sessions` | List sessions |
| DELETE | `/v1/auth/sessions/{session_id}` | Revoke session |

### Seal/Unseal (`/v1/sys`)

Shamir secret sharing to protect the master encryption key. These endpoints are unauthenticated (pre-auth).

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/sys/init` | Initialize vault (generates root token + unseal keys) |
| GET | `/v1/sys/seal-status` | Get seal status |
| POST | `/v1/sys/unseal` | Submit unseal key share |
| POST | `/v1/sys/seal` | Seal the vault |
| POST | `/v1/sys/rekey/init` | Start rekey operation |
| POST | `/v1/sys/rekey/update` | Submit rekey share |

### Namespace Isolation (`/v1/sys/namespaces`)

Multi-tenant namespace support with resource isolation.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/sys/namespaces` | Create namespace |
| GET | `/v1/sys/namespaces` | List namespaces |
| GET | `/v1/sys/namespaces/{id}` | Get namespace |
| PUT | `/v1/sys/namespaces/{id}` | Update namespace |
| DELETE | `/v1/sys/namespaces/{id}` | Delete namespace |
| GET | `/v1/sys/namespaces/{id}/stats` | Namespace stats |

### Policy Management (`/v1/sys/policies`)

HCL-style access control policies.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/sys/policies` | List policies |
| POST | `/v1/sys/policies/{name}` | Create policy |
| GET | `/v1/sys/policies/{name}` | Read policy |
| PUT | `/v1/sys/policies/{name}` | Update policy |
| DELETE | `/v1/sys/policies/{name}` | Delete policy |
| POST | `/v1/sys/policies/{name}/test` | Test policy |

### Lease Management (`/v1/sys/leases`)

TTL-based lease management for dynamic secrets.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/sys/leases/renew` | Renew lease |
| POST | `/v1/sys/leases/revoke` | Revoke lease |
| POST | `/v1/sys/leases/revoke-prefix` | Revoke by prefix |
| GET | `/v1/sys/leases/lookup/{lease_id}` | Lookup lease |
| GET | `/v1/sys/leases` | List leases |
| GET | `/v1/sys/leases/stats` | Lease stats |

### Response Wrapping (`/v1/sys/wrapping`)

Single-use token wrapping for secure secret delivery.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/sys/wrapping/wrap` | Wrap data |
| POST | `/v1/sys/wrapping/unwrap` | Unwrap token |
| GET | `/v1/sys/wrapping/lookup/{token}` | Lookup wrapping token |
| POST | `/v1/sys/wrapping/rewrap` | Rewrap token |

### Secret Rotation (`/v1/sys/rotation`)

Automatic secret rotation with webhook notifications.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/sys/rotation/policies` | Create rotation policy |
| GET | `/v1/sys/rotation/policies` | List rotation policies |
| GET | `/v1/sys/rotation/policies/{id}` | Get rotation policy |
| POST | `/v1/sys/rotation/policies/{id}/execute` | Execute rotation |
| DELETE | `/v1/sys/rotation/policies/{id}` | Delete rotation policy |
| GET | `/v1/sys/rotation/history` | Rotation history |
| GET | `/v1/sys/rotation/statistics` | Rotation statistics |
| POST | `/v1/sys/rotation/scheduler/start` | Start auto-rotation scheduler |
| POST | `/v1/sys/rotation/scheduler/stop` | Stop scheduler |

### Additional Engines

| Engine | Base Path | Description |
|--------|-----------|-------------|
| TOTP | `/v1/sys/totp` | Time-based OTP key management and code generation/validation |
| Crypto (HMAC, Random) | `/v1/sys/crypto` | HMAC computation, random byte generation, re-encryption |
| Transform (FPE/Tokenization) | `/v1/sys/transform` | Format-preserving encryption, tokenization roles |
| AWS | `/v1/sys/aws` | Dynamic AWS IAM/STS credentials |
| Zero-Knowledge | `/v1/sys/zk` | Zero-knowledge secret storage |
| Inject | `/v1/sys/inject` | Environment variable injection |
| Webhooks | `/v1/sys/webhooks` | Webhook subscriptions and delivery management |

> **Removed 2026-07-18.** The GCP, Azure, Identity, KMIP, LDAP, RabbitMQ and Kafka
> engines were deleted. None of them had a client library for the system they
> named, so they generated a credential locally, stored a lease and reported
> success **without ever contacting that system** — a caller would have been
> handed a fabricated credential as though it were valid. Identity was a second
> OIDC provider, which conflicts with authenc being the identity authority.
> None had a consumer. Re-adding any of them requires a real client library plus
> an integration test proving the remote object was actually created.

> **Removed 2026-07-19.** The `core/src/services/auth/` tree (9 auth methods) and
> a duplicate, unreachable copy of the service layer (`admin_service`,
> `auth_service`, `secret_service`, `policy_service`, `dynamic_role_service`,
> `rbac`, `key_manager`) were deleted — 7.824 lines with zero consumers. The live
> authentication path is `core/src/auth/authenc_provider.rs` (delegating to
> authenc) plus the API crate's own services, which are what the server actually
> constructs. Five of the auth methods were simulated; LDAP's `bind()` accepted
> **any non-empty password** and its user lookup fabricated **any username**.
>
> **Kubernetes SA auth is not implemented server-side** despite the client and
> Helm policy config existing — there is no `/v1/auth/kubernetes/login` route,
> and `infra/helm/bootstrap-secreton.sh` targets HashiCorp Vault's API. Leave
> `secretonAuth.enabled` at `false`.

### Administration (`/v1/admin`)

User/role management, system config, maintenance, and security scanning.

| Category | Endpoints |
|----------|-----------|
| Users | `GET/POST /v1/admin/users`, `GET/PUT/DELETE /v1/admin/users/{id}`, roles, permissions |
| Roles | `GET/POST /v1/admin/roles`, `GET/PUT/DELETE /v1/admin/roles/{name}` |
| Config | `GET/PUT /v1/admin/config`, `POST /v1/admin/config/reload` |
| Maintenance | `POST /v1/admin/maintenance/gc\|compact\|vacuum` |
| Security | `POST /v1/admin/security/scan`, `GET /v1/admin/security/reports\|incidents` |
| System | `GET /v1/admin/metrics\|status\|logs` |

### Backup & Audit (`/v1/secret`)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/secret/backup` | Create backup |
| GET | `/v1/secret/backup` | List backups |
| GET | `/v1/secret/backup/{id}` | Get backup |
| POST | `/v1/secret/backup/{id}/restore` | Restore backup |
| DELETE | `/v1/secret/backup/{id}` | Delete backup |
| GET | `/v1/secret/audit` | View audit logs |
| GET | `/v1/secret/audit/export` | Export audit logs |

### Raft Cluster (feature-gated: `raft-consensus`)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/raft/join` | Join cluster |
| GET | `/v1/raft/peers` | List peers |
| DELETE | `/v1/raft/peers/{node_id}` | Remove peer |
| GET | `/v1/raft/status` | Cluster status |
| GET | `/v1/raft/election-stats` | Election statistics |
| POST | `/v1/raft/snapshot` | Create snapshot |
| GET | `/v1/raft/snapshots` | List snapshots |

### Health & Metrics

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/ready` | Readiness check |
| GET | `/live` | Liveness check |
| GET | `/version` | Version info |
| GET | `/metrics` | JSON metrics |
| GET | `/metrics/prometheus` | Prometheus metrics |

### gRPC API

36 RPCs defined in `proto/secreton.proto` (package `secreton.v1`):

- **Secrets**: StoreSecret, GetSecret, DeleteSecret, ListSecrets
- **Transit**: CreateKey, Encrypt, Decrypt, Sign, Verify, RotateKey
- **Cluster**: GetClusterStatus, ListPeers, AddNode, RemoveNode
- **Snapshots**: CreateSnapshot, ListSnapshots, RestoreSnapshot
- **Namespaces**: List, Create, Get, Update, Delete, GetStats
- **Dynamic Secrets**: GenerateDatabaseCredentials, CRUD for DatabaseRole/Connection
- **Leases**: Renew, Revoke, RevokePrefix, Lookup, List, GetStats
- **Policies**: List, Create, Get, Update, Delete, Test
- **Wrapping**: Wrap, Unwrap, Lookup, Rewrap
- **Health**: HealthCheck, GetMetrics

## CLI

Binary: `secreton-cli`

```
USAGE:
    secreton-cli [OPTIONS] <COMMAND>

GLOBAL FLAGS:
    --server <URL>       Server address (default: http://127.0.0.1:8200)
    --namespace <NS>     Target namespace
    --config <PATH>      Config file path
    --verbose            Verbose output

COMMANDS:
    status               System health and status
    login                Authenticate (--method, --username, --token)
    logout               End session

    secret put           Store a secret
    secret get           Read a secret
    secret list          List secrets
    secret delete        Delete a secret

    transit create-key   Create transit encryption key
    transit list-keys    List transit keys
    transit encrypt      Encrypt data
    transit decrypt      Decrypt data

    seal init            Initialize vault
    seal seal            Seal the vault
    seal unseal          Submit unseal key share
    seal status          Check seal status
    seal rekey init      Start rekey
    seal rekey update    Submit rekey share
    seal rekey cancel    Cancel rekey
    seal rekey status    Rekey status

    auto-unseal configure   Configure auto-unseal
    auto-unseal status      Auto-unseal status
    auto-unseal test        Test auto-unseal
    auto-unseal disable     Disable auto-unseal

    policy list          List policies
    policy read          Read a policy
    policy write         Create/update policy
    policy delete        Delete policy
    policy fmt           Format policy file
    policy validate      Validate policy syntax
    policy test          Test policy against request

    token create         Create a token
    token lookup         Lookup token details
    token renew          Renew token
    token revoke         Revoke token
    token capabilities   Check token capabilities

    audit list           Query audit logs (--user, --operation, --path, --start_time, --end_time, --limit, --format)

    backup create        Create backup
    backup restore       Restore from backup
    backup verify        Verify backup integrity
    backup list          List backups

    replication enable          Enable replication
    replication disable         Disable replication
    replication status          Replication status
    replication promote         Promote secondary to primary
    replication add-secondary   Add secondary node
    replication remove-secondary Remove secondary node
    replication lag             Check replication lag

    operator diagnose    Run diagnostics

    config set           Set config value
    config get           Get config value
    config show          Show all config
```

## Cryptography

All crypto implemented via the RustCrypto ecosystem (`secreton-crypto` crate).

| Category | Algorithms |
|----------|------------|
| Symmetric Encryption | AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305 |
| Digital Signatures | Ed25519, ECDSA-P256, ECDSA-secp256k1 |
| Key Exchange | X25519 |
| Hashing | SHA-256, SHA-384, SHA-512, SHA3-256, SHA3-384, SHA3-512, BLAKE3 |
| Key Derivation | Argon2id, PBKDF2-SHA256, PBKDF2-SHA512, HKDF-SHA256, HKDF-SHA512, scrypt |
| HMAC | HMAC-SHA256, HMAC-SHA384, HMAC-SHA512, HMAC-SHA3-256 |
| Secret Sharing | Shamir Secret Sharing (configurable shares/threshold) |
| Format-Preserving | FPE (FF1/FF3-1 style) |
| Post-Quantum (experimental) | ML-KEM (Kyber), ML-DSA (Dilithium) via hybrid classical+PQ |

## Configuration

Bootstrap config file: `secreton.toml`

```toml
log_level = "info"       # trace, debug, info, warn, error
log_format = "json"      # json, pretty

[storage]
backend = "raft"         # raft, postgres, file, memory

[storage.raft]
path = "/var/lib/secreton/raft"
node_id = "node1"

[storage.raft.performance]
election_timeout_ms = 1000
heartbeat_interval_ms = 300
snapshot_interval_secs = 120
max_appending_entries = 64

[listener.http]
address = "0.0.0.0:8200"
tls_enabled = false

[listener.grpc]
enabled = true
address = "0.0.0.0:8201"

[seal]
type = "shamir"          # shamir, aws-kms, gcp-kms, azure-kv

[seal.shamir]
shares = 5
threshold = 3

[telemetry]
prometheus_enabled = true
metrics_path = "/metrics"
```

**Storage backends:**

- `raft` — Integrated Raft consensus (recommended for HA). Supports multi-node cluster via `retry_join`.
- `postgres` — PostgreSQL. Connection URL via `SECRETON_STORAGE_URL` environment variable.
- `file` — Local filesystem (single-node only).
- `memory` — In-memory (development/testing only).

**Auto-unseal providers:**

- `shamir` — Manual unseal with key shares (default).
- `aws-kms` — AWS KMS.
- `gcp-kms` — Google Cloud KMS.
- `azure-kv` — Azure Key Vault.

## Deployment

### Docker

```bash
# Build
docker build -f layanan/secreton/Dockerfile -t secreton:latest .

# Run
docker run -d \
  -p 8200:8200 -p 8201:8201 \
  -v /data/secreton:/app/data \
  -e RUST_LOG=info \
  secreton:latest
```

The Dockerfile uses a 4-stage build (cargo-chef for layer caching) producing a minimal `debian:bookworm-slim` image. Runs as non-root user `secreton:1000`. Ports: 8200 (REST), 8201 (gRPC), 8300 (cluster).

### Kubernetes

Manifests in `deploy/staging/`:

| File | Resource |
|------|----------|
| `00-namespace.yaml` | Namespace |
| `01-secrets.yaml` | K8s Secret (TLS certs, DB credentials) |
| `02-configmaps.yaml` | ConfigMap (`secreton.toml`) |
| `03-postgres.yaml` | PostgreSQL Deployment + Service |
| `04-secreton-statefulset.yaml` | Secreton StatefulSet (with PVC) |
| `05-ingress-rbac.yaml` | Ingress + RBAC |
| `06-autoscaling-policies.yaml` | HPA |

### Quick Start (local development)

```bash
# 1. Start Secreton (in-memory storage, no TLS)
cargo run -p secreton-api --bin api_server

# 2. Initialize
curl -X POST http://localhost:8200/v1/sys/init \
  -H "Content-Type: application/json" \
  -d '{"secret_shares":5,"secret_threshold":3}'
# → Returns root_token and unseal keys

# 3. Unseal (submit 3 of 5 keys)
curl -X POST http://localhost:8200/v1/sys/unseal \
  -H "Content-Type: application/json" \
  -d '{"key":"<unseal_key_1>"}'
# Repeat for keys 2 and 3

# 4. Use the API
export TOKEN="<root_token>"
curl http://localhost:8200/v1/secret/secrets \
  -H "Authorization: Bearer $TOKEN"
```

## Database Migrations

PostgreSQL migrations in `migrations/`:

| Migration | Description |
|-----------|-------------|
| `create_audit_logs` | Audit logging tables |
| `create_secret_manager_state` | Secret manager state |
| `create_namespaces` | Namespace isolation tables |
| `create_dynamic_roles` | Dynamic secret roles |
| `create_leases` | Lease management |
| `create_policies` | Policy storage |
| `create_wrapping_tokens` | Response wrapping tokens |
| `create_raft_snapshots` | Raft snapshot metadata |
| `use_secreton_schema` | Schema namespacing |
| `dynamic_role_system` | Dynamic role system |
| `create_sealed_master_keys` | Sealed master key storage |

Initialize with: `psql -f scripts/init-db.sql`

## Testing

```bash
# All tests
cargo test -p secreton --workspace

# Specific crate
cargo test -p secreton-crypto
cargo test -p secreton-api

# E2E tests (requires running instance)
bash test_e2e_v3.sh

# Benchmarks
cargo bench -p secreton --bench transit_bench
cargo bench -p secreton --bench storage_bench
```

Test suites cover: integration, security validation, crypto roundtrips, PKI, Raft cluster, seal behavior, leases, performance, hybrid post-quantum crypto, and government compliance.

## Middleware Stack

Requests pass through these layers (in order):

1. **Rate limiting** — Per-IP request throttling
2. **Security headers** — HSTS, CSP, X-Frame-Options (via `lib-common`)
3. **Request logging** — Structured request/response logging (via `lib-common`)
4. **Correlation ID** — Request tracing (via `lib-common`)
5. **CORS** — Cross-origin resource sharing (via `lib-common`)
6. **Seal check** — Rejects requests if vault is sealed (returns 503)
7. **Authentication** — JWT token validation
8. **Metrics** — Per-endpoint latency and counter metrics

## Operational Scripts

| Script | Purpose |
|--------|---------|
| `scripts/init-db.sql` | Initialize PostgreSQL database and schema |
| `scripts/compliance-report.sh` | Generate compliance report |
| `scripts/disaster-recovery.sh` | Disaster recovery procedures |
| `scripts/ha-cluster-validation.sh` | Validate HA cluster health |
| `scripts/performance-bench.sh` | Run performance benchmarks |
| `scripts/integration-test.sh` | Run integration test suite |
| `scripts/e2e-validation.sh` | End-to-end validation |

## Build Profiles

| Profile | LTO | Codegen Units | Strip | Overflow Checks | Use Case |
|---------|-----|---------------|-------|-----------------|----------|
| `dev` | off | incremental | no | yes | Development |
| `release` | thin | 1 | yes | no | Production |
| `security` | fat | 1 | yes | yes | Security-critical deployment |
| `bench` | thin (inherits release) | 1 | yes | no | Benchmarking |

## Related Documentation

| Document | Location |
|----------|----------|
| Root workspace guide | `AGENTS.md` (repo root) |
| Secreton agent guide | `AGENTS.md` (this directory) |
| Configuration example | `secreton.toml.example` |
| Deployment manifests | `deploy/staging/`, `deploy/kubernetes/` |
| Proto definitions | `proto/secreton.proto`, `proto/common.proto` |
| Monitoring config | `monitoring/prometheus.yml` |
