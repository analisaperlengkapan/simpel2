# Secreton vs HashiCorp Secret Vault: Architecture Comparison

## Overview

Secreton follows HashiCorp Secret Vault's proven architectural patterns while being implemented in Rust for enhanced memory safety and performance. This document outlines the design decisions and compatibility considerations.

## Storage Backend Philosophy

### HashiCorp Secret Vault's Approach

HashiCorp Secret Vault explicitly **does not recommend** using traditional relational databases (PostgreSQL, MySQL) as storage backends for production deployments. From their documentation:

> "The storage backend is untrusted and is responsible only for durable storage of encrypted data. The storage backend never receives cleartext data."

Key principles:

1. **Storage backends are untrusted** - All data is encrypted before storage
2. **No complex queries needed** - Simple key-value operations only
3. **HA through distributed systems** - Use Consul or Integrated Storage (Raft)
4. **Separation of concerns** - Storage ≠ Database

### Secreton's Implementation

Secreton adopts the same philosophy:

```
┌─────────────────────────────────────────────────────────────┐
│                    Secreton Architecture                     │
├─────────────────────────────────────────────────────────────┤
│  Application Layer (Rust)                                    │
│  ├── Crypto Engine (RustCrypto)                             │
│  ├── Secrets Engines (Transit, KV, PKI, etc.)               │
│  └── Policy Engine (RBAC, ACL)                              │
├─────────────────────────────────────────────────────────────┤
│  Storage Interface (Untrusted)                               │
│  ├── All data encrypted before storage                       │
│  ├── Simple KV operations only (get, put, list, delete)     │
│  └── No SQL queries, no complex transactions                │
├─────────────────────────────────────────────────────────────┤
│  Backend Selection (Choose One)                              │
│  ├── Consul     ✅ RECOMMENDED FOR HA                       │
│  ├── Raft      ✅ RECOMMENDED FOR HA (built-in)            │
│  ├── S3        ✅ RECOMMENDED FOR CLOUD                     │
│  ├── File      ✅ RECOMMENDED FOR DEV/SINGLE-NODE          │
│  └── PostgreSQL ⚠️  LEGACY ONLY (not for HA)               │
└─────────────────────────────────────────────────────────────┘
```

## Storage Backend Comparison

### Production HA Backends (Recommended)

| Feature                   | Consul         | Raft (Integrated) | HashiCorp Secret Vault Equivalent  |
| ------------------------- | -------------- | ----------------- | --------------------------- |
| **External Dependencies** | Consul cluster | None (built-in)   | Consul / Integrated Storage |
| **High Availability**     | ✅ Yes         | ✅ Yes            | ✅ Yes                      |
| **Automatic Failover**    | ✅ Yes         | ✅ Yes            | ✅ Yes                      |
| **Leader Election**       | ✅ Consul      | ✅ Raft           | ✅ Yes                      |
| **Minimum Nodes**         | 3 recommended  | 3 recommended     | 3 recommended               |
| **Service Discovery**     | ✅ Yes         | ❌ No             | Consul only                 |
| **Setup Complexity**      | Medium         | Low               | Medium / Low                |
| **Production Ready**      | ✅ Yes         | ✅ Yes            | ✅ Yes                      |
| **Database Required**     | ❌ No          | ❌ No             | ❌ No                       |

### Cloud Backends

| Feature                | S3            | Azure Blob       | GCS          | HashiCorp Secret Vault Equivalent |
| ---------------------- | ------------- | ---------------- | ------------ | -------------------------- |
| **Durability**         | 11 9's        | 16 9's           | 11 9's       | Same backends              |
| **Availability**       | 99.99%        | 99.9%            | 99.95%       | Same backends              |
| **Encryption at Rest** | ✅ SSE-S3/KMS | ✅ Azure Storage | ✅ CMEK      | ✅ Yes                     |
| **Cross-Region**       | ✅ Yes        | ✅ Yes           | ✅ Yes       | ✅ Yes                     |
| **Versioning**         | ✅ Yes        | ✅ Yes           | ✅ Yes       | ✅ Yes                     |
| **HA Support**         | ⚠️ See Note¹  | ⚠️ See Note¹     | ⚠️ See Note¹ | ⚠️ See Note¹               |
| **Database Required**  | ❌ No         | ❌ No            | ❌ No        | ❌ No                      |

¹ Cloud backends provide data durability but not active-active HA. Use with DynamoDB (AWS) or etcd for HA coordination.

### Development & Single-Node Backends

| Feature                   | File             | Memory       | HashiCorp Secret Vault Equivalent |
| ------------------------- | ---------------- | ------------ | -------------------------- |
| **External Dependencies** | None             | None         | Same                       |
| **Persistence**           | ✅ Yes           | ❌ No        | Same                       |
| **Performance**           | Fast             | Fastest      | Same                       |
| **Suitable For**          | Dev, IoT, Edge   | Testing only | Same                       |
| **HA Support**            | ❌ No            | ❌ No        | ❌ No                      |
| **Production**            | Single-node only | ❌ Never     | Single-node only           |

### Legacy Backend (Not Recommended)

| Feature               | PostgreSQL          | MySQL (Secret Vault)       | Why Not Recommended      |
| --------------------- | ------------------- | ------------------- | ------------------------ |
| **Database Required** | ✅ Yes              | ✅ Yes              | Extra operational burden |
| **HA Complexity**     | High                | High                | Requires DB clustering   |
| **Performance**       | Slower              | Slower              | Network + DB overhead    |
| **Maintenance**       | DB updates, backups | DB updates, backups | Additional maintenance   |
| **Secreton Status**   | Legacy support      | Not implemented     | Use Consul/Raft instead  |
| **Secret Vault Status**      | Community edition   | Community edition   | Deprecated approach      |

## Why Not Use PostgreSQL for HA?

### HashiCorp Secret Vault's Position

From Secret Vault documentation:

> "The PostgreSQL backend is a **community-supported** backend and is **not recommended for production use**."

Reasons:

1. **Additional operational complexity** - Database requires its own HA setup
2. **Performance overhead** - Network calls + SQL parsing for simple KV operations
3. **Not battle-tested** - Consul and Raft have years of production validation
4. **Unnecessary features** - ACID transactions, SQL queries not needed for encrypted blobs

### Secreton's Position

We maintain PostgreSQL support for:

- **Legacy migration** - Organizations migrating from database-centric systems
- **Development** - Quick setup for developers familiar with Postgres
- **Testing** - Integration tests requiring persistent storage

**But for production HA, use:**

- **Consul** - If you need service discovery + HA
- **Raft** - If you want zero external dependencies
- **S3** - If you're cloud-native

## Migration Path

### From PostgreSQL to Consul/Raft

1. **Current Setup (Not Recommended)**

```toml
[storage]
backend = "postgres"
connection_string = "postgresql://..."
```

2. **Recommended Setup (Consul)**

```toml
[storage]
backend = "consul"
address = "127.0.0.1:8500"
path = "secreton/"
```

3. **Recommended Setup (Raft - Zero Dependencies)**

```toml
[storage]
backend = "raft"
node_id = 1
peers = ["2:node2:7001", "3:node3:7001"]
```

### Data Migration Tool

```bash
# Export from PostgreSQL
secreton-cli storage export \
  --source postgres://... \
  --format encrypted-backup \
  --output /backup/secreton.enc

# Import to Consul
secreton-cli storage import \
  --target consul://127.0.0.1:8500/secreton/ \
  --input /backup/secreton.enc
```

## Deployment Patterns

### Pattern 1: Consul-Backed HA (Recommended)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Secreton 1  │────▶│  Consul 1   │◀────│ Secreton 2  │
│  (Active)   │     │  (Leader)   │     │ (Standby)   │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                    ┌──────┴──────┐
                    │   Consul 2  │
                    │  (Follower) │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │   Consul 3  │
                    │  (Follower) │
                    └─────────────┘
```

**Advantages:**

- Service discovery included
- Health checks automatic
- Used by HashiCorp Secret Vault in production
- Well-documented operational practices

### Pattern 2: Raft Integrated Storage (Recommended)

```
┌──────────────────────────────────────────────┐
│         Secreton Cluster (Raft)              │
│                                              │
│  ┌─────────────┐  ┌─────────────┐           │
│  │ Secreton 1  │  │ Secreton 2  │           │
│  │  + Raft     │──│  + Raft     │           │
│  │  (Leader)   │  │ (Follower)  │           │
│  └─────────────┘  └─────────────┘           │
│         │                │                   │
│         └────────┬───────┘                   │
│                  │                           │
│          ┌───────┴────────┐                  │
│          │   Secreton 3   │                  │
│          │    + Raft      │                  │
│          │   (Follower)   │                  │
│          └────────────────┘                  │
└──────────────────────────────────────────────┘
```

**Advantages:**

- Zero external dependencies
- Simpler deployment
- Perfect for Kubernetes
- Same as Secret Vault's Integrated Storage

### Pattern 3: Cloud-Native (S3)

```
┌─────────────┐     ┌──────────────────────┐
│ Secreton 1  │────▶│    AWS S3 Bucket     │
└─────────────┘     │  (Encrypted at Rest) │
                    └──────────────────────┘
┌─────────────┐              │
│ Secreton 2  │──────────────┘
└─────────────┘
```

**Advantages:**

- Unlimited scalability
- 11 9's durability
- No infrastructure management
- Multi-region replication

## Performance Comparison

### Latency (Typical)

| Backend            | Get (p50) | Put (p50) | Get (p99) | Put (p99) |
| ------------------ | --------- | --------- | --------- | --------- |
| Memory             | 10μs      | 15μs      | 50μs      | 100μs     |
| File               | 100μs     | 500μs     | 1ms       | 5ms       |
| Consul (local)     | 2ms       | 3ms       | 10ms      | 20ms      |
| Raft (local)       | 1ms       | 5ms       | 15ms      | 30ms      |
| S3 (same region)   | 20ms      | 50ms      | 100ms     | 200ms     |
| PostgreSQL (local) | 5ms       | 10ms      | 25ms      | 50ms      |

### Throughput (Ops/sec, single node)

| Backend    | Reads    | Writes  |
| ---------- | -------- | ------- |
| Memory     | 100,000+ | 50,000+ |
| File       | 10,000   | 5,000   |
| Consul     | 5,000    | 2,000   |
| Raft       | 8,000    | 3,000   |
| S3         | 1,000    | 500     |
| PostgreSQL | 3,000    | 1,500   |

**Note:** These are rough estimates. Actual performance depends on hardware, network, and configuration.

## Security Considerations

### Data at Rest

All backends store **encrypted data only**:

- ✅ Consul: Encrypted blobs in KV store
- ✅ Raft: Encrypted log entries
- ✅ S3: Encrypted objects (+ optional SSE-KMS)
- ✅ File: Encrypted files on disk
- ✅ PostgreSQL: Encrypted BLOBs in database

### Key Management

**Master Key (Seal Key):**

- Never stored in storage backend
- Provided via:
  - Environment variable (dev)
  - HSM (production)
  - Cloud KMS (AWS KMS, Azure Key Secret Vault)
  - Shamir Secret Sharing (manual unseal)

**Data Encryption Key (DEK):**

- Stored encrypted by Master Key
- Rotatable without data migration

## Operational Recommendations

### Small Deployments (< 10 instances)

- **Use:** Raft Integrated Storage
- **Why:** Simple, no external dependencies

### Medium Deployments (10-100 instances)

- **Use:** Consul
- **Why:** Service discovery + HA + proven at scale

### Large Deployments (> 100 instances)

- **Use:** Consul or S3 (depending on cloud strategy)
- **Why:** Battle-tested, unlimited scale

### Cloud-Native

- **Use:** S3, Azure Blob, or GCS
- **Why:** Managed service, no infrastructure

### Air-Gapped / Edge

- **Use:** File or Raft
- **Why:** No external network dependencies

### Legacy Migration

- **Use:** PostgreSQL temporarily, plan migration to Consul/Raft
- **Why:** Compatibility, but plan upgrade path

## Conclusion

**Secreton's storage architecture mirrors HashiCorp Secret Vault:**

- ✅ Storage backends are untrusted
- ✅ All data encrypted before storage
- ✅ Consul and Raft recommended for HA
- ✅ Cloud backends for scalability
- ✅ File/Memory for development
- ⚠️ PostgreSQL available but not recommended

**For new deployments, choose:**

1. **Consul** - If you have Consul infrastructure
2. **Raft** - If you want simplicity and zero dependencies
3. **S3** - If you're cloud-native

**Avoid PostgreSQL** unless you have a specific legacy requirement.
