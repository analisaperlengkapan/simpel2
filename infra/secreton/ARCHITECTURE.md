# Secreton Architecture Documentation

## Overview

Secreton is a modular, security-focused secrets management system designed for Indonesian government use (Kejaksaan RI). The codebase is organized into 6 specialized crates following hexagonal architecture principles.

**Total codebase**: ~87,000 lines of Rust code
**Architecture**: Layered with clear separation of concerns
**Target**: Production-ready secrets vault with RBAC, audit logging, and post-quantum cryptography

## Crate Structure

### 1. secreton-core (40,135 lines)

**Purpose**: Core business logic, domain models, and services

**Key modules**:

- `models/`: Domain models (User, Secret, Policy, Audit, etc.)
- `services/`: Business logic services
  - `auth/`: Authentication providers (LDAP, OAuth, Certificate, etc.)
  - `secrets/`: Secrets engines (KVv2, Database, Transit, etc.)
  - `policy.rs`: RBAC policy engine (1,569 lines)
  - `seal.rs`: Seal/unseal operations (1,315 lines)
  - `lease.rs`: Lease management (1,408 lines)
- `security/`: Security components (audit, seal wrapping, HSM)
- `storage/`: Storage abstraction layer
- `config/`: Configuration management
- `namespace/`: Multi-tenancy support

**Dependencies**: secreton-crypto, secreton-storage

**Architecture notes**:

- Contains business logic, not HTTP/transport concerns
- Services implement trait-based interfaces for extensibility
- Large files (policy.rs, seal.rs, lease.rs) are candidates for extraction into separate crates if needed

### 2. secreton-api (27,659 lines)

**Purpose**: HTTP and gRPC API layer

**Key modules**:

- `handlers/`: HTTP request handlers
  - `auth.rs`: Authentication endpoints
  - `secret.rs`: Secret CRUD operations
  - `policy.rs`: Policy management
  - `dynamic.rs`: Dynamic secrets
  - `lease.rs`, `seal.rs`, `wrapping.rs`, etc.
- `grpc/`: gRPC server implementation
  - `server.rs`: gRPC service (1,866 lines)
  - `generated/`: Protocol buffer generated code (3,753 lines)
- `services/`: API service wrappers
  - `admin.rs`, `auth.rs`, `vault.rs`: Thin wrappers around core services
- `middleware.rs`: Authentication, rate limiting, audit (1,159 lines)
- `config.rs`: API server configuration (1,451 lines)

**Dependencies**: secreton-core, secreton-crypto, secreton-storage, tonic (gRPC)

**Architecture notes**:

- HTTP handlers are thin - delegate to core services
- gRPC and REST coexist in same crate (could be split if needed)
- API models (DTOs) are separate from domain models in core

### 3. secreton-crypto (11,598 lines)

**Purpose**: Cryptographic operations and key management

**Key modules**:

- `hybrid.rs`: Hybrid classical + post-quantum cryptography
- `pq_key_management.rs`: Post-quantum key management (1,150 lines)
- `shamir.rs`: Shamir secret sharing (1,214 lines)
- `transit/`: Transit secrets engine crypto operations
  - `keys.rs`, `algorithms.rs`: Key management and algorithms

**Dependencies**: RustCrypto libraries, ML-KEM, ring

**Architecture notes**:

- Self-contained cryptographic operations
- Supports both classical (AES-256, ChaCha20) and post-quantum (ML-KEM)
- No business logic - pure cryptographic functions

### 4. secreton-storage (4,332 lines)

**Purpose**: Storage backend abstraction

**Key modules**:

- `lib.rs`: StorageBackend trait definition
- `postgres.rs`: PostgreSQL implementation
- `memory.rs`: In-memory storage (for testing)
- `encryption.rs`: Storage-level encryption

**Dependencies**: tokio-postgres, deadpool-postgres

**Architecture notes**:

- Trait-based abstraction allows multiple backends
- VaultEntry is the primary storage model
- Handles connection pooling and query building

### 5. secreton-cli (2,689 lines)

**Purpose**: Command-line interface

**Key modules**:

- `backup.rs`: Backup/restore operations (1,102 lines)
- `commands/`: CLI command implementations

**Dependencies**: clap, secreton-core

**Architecture notes**:

- Separate from API server
- Used for administration and operations

### 6. secreton-agent (759 lines)

**Purpose**: Distributed agent for consensus and replication

**Key modules**:

- `sink.rs`: Log sink for Raft

**Dependencies**: Raft libraries

**Architecture notes**:

- Small, focused crate for distributed operations
- Optional feature for high availability

## Dependency Graph

```
secreton-cli ─────┐
                  ├──> secreton-core ─┬──> secreton-crypto
secreton-api ─────┘                   └──> secreton-storage

secreton-agent ──────────> secreton-core
```

**Key principles**:

- No circular dependencies
- Core is the central business logic layer
- API and CLI depend on core, not vice versa
- Crypto and storage are leaf dependencies

## Design Patterns

### 1. Trait-Based Polymorphism

Multiple implementations of common interfaces:

- `AuthProvider` trait: userpass, LDAP, OAuth, certificate, Kubernetes
- `StorageBackend` trait: PostgreSQL, memory, future: S3, etc.
- `SecretsEngine` trait: KVv2, database, transit, etc.

### 2. Layered Architecture

```
┌─────────────────────────────────────┐
│   API Layer (HTTP/gRPC)             │  <- secreton-api
├─────────────────────────────────────┤
│   Business Logic (Services)         │  <- secreton-core
├─────────────────────────────────────┤
│   Storage & Crypto (Infrastructure) │  <- secreton-storage, secreton-crypto
└─────────────────────────────────────┘
```

### 3. Separation of Concerns

- **Models vs DTOs**: Core has domain models, API has request/response DTOs
- **Business logic vs transport**: Core doesn't know about HTTP
- **Audit trail**: Comprehensive logging at all layers

## Module Guidelines

### When to extract a new crate:

1. Module is self-contained (minimal dependencies on core)
2. Module is large (>2,000 lines)
3. Module has clear interface boundary
4. Module could be reused independently

### Candidates for extraction (future):

- `secreton-policy`: RBAC policy engine (~2K lines)
- `secreton-seal`: Seal/unseal operations (~2K lines)
- `secreton-lease`: Lease management (~2K lines)
- `secreton-engines`: Dynamic secrets engines (~4K lines)
- `secreton-grpc`: gRPC server separate from HTTP API (~6K lines)

### NOT candidates:

- Small modules (<500 lines)
- Tightly coupled to core business logic
- Trait implementations (auth providers, etc.)

## Code Organization Principles

1. **One concern per module**: Each file should have a single responsibility
2. **Minimal public API**: Export only what's needed by other modules
3. **Documentation**: Every public module, struct, and function should have docs
4. **Testing**: Unit tests in same file, integration tests in tests/
5. **Error handling**: Use Result types, custom error enums with thiserror

## Build and Development

- **Build time**: ~15-20 seconds for full workspace (incremental: 2-5s)
- **Test suite**: Unit tests in each crate, integration tests in tests/
- **CI/CD**: GitHub Actions workflows validate all crates
- **MSRV**: Rust 1.90+ (edition 2024)

## Performance Characteristics

- **secreton-core**: Large but well-organized, incremental builds are fast
- **secreton-api**: gRPC generated code (3.7K lines) doesn't change often
- **secreton-crypto**: Compute-intensive but efficient (uses RustCrypto)
- **secreton-storage**: I/O bound, uses async/await throughout

## Security Architecture

1. **Zero-trust**: All operations authenticated and authorized
2. **Audit trail**: Comprehensive logging of all operations
3. **Encryption**: Data encrypted at rest and in transit
4. **Post-quantum ready**: Hybrid classical + PQ cryptography
5. **Secrets isolation**: Namespace-based multi-tenancy

## Future Improvements

### Potential optimizations:

1. Split gRPC into separate crate (reduce API crate compilation time)
2. Extract large services (policy, seal, lease) into focused crates
3. Create shared models crate to reduce duplication
4. Implement lazy compilation for optional features

### Not recommended:

- Over-splitting into too many small crates (increases complexity)
- Moving trait implementations out of core (breaks cohesion)
- Aggressive refactoring without clear benefits

## Contributing

When adding new features:

1. **Check existing modules first** - Don't duplicate functionality
2. **Follow trait patterns** - Implement existing traits when possible
3. **Keep layer separation** - Don't put HTTP logic in core
4. **Document thoroughly** - Especially public APIs
5. **Add tests** - Unit tests for logic, integration tests for workflows

## References

- **Standards**: ISO/IEC 25010, OWASP ASVS, 12-Factor App
- **Cryptography**: NIST Post-Quantum Cryptography standards
- **Architecture**: Hexagonal Architecture, Domain-Driven Design
- **Rust**: Edition 2024, async/await, tokio runtime

---

**Last updated**: November 5, 2025
**Maintained by**: Cipherce Security Team
**For**: Kejaksaan Republik Indonesia
