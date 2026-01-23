---
inclusion: always
---

# Technology Stack

## Core Technologies

- **Language**: Rust (Edition 2024, MSRV 1.90+)
- **Backend Framework**: Axum 0.8.x (REST APIs)
- **RPC Framework**: Tonic 0.14.x + Prost 0.14.x (gRPC)
- **Frontend Framework**: Leptos 0.8.x (WASM, CSR mode)
- **Database**: PostgreSQL (tokio-postgres, deadpool)
- **Cache**: Redis
- **Build Tool**: Cargo workspace
- **Frontend Build**: Trunk (for WASM)

## Dependency Management

- ALL external dependencies defined in root `Cargo.toml` under `[workspace.dependencies]`
- Member crates use `dependency = { workspace = true }` (NEVER specify versions)
- `infra/authenc` and `infra/secreton` are PART OF main workspace (not separate)
- Secreton has sub-crates in `infra/secreton/crates/` (core, api, storage, crypto, types, agent, cli, grpc, hsm, k8s-operator)

## Common Commands

### Verification & Building
```bash
cargo check --workspace              # Quick compilation check
cargo build --workspace              # Full workspace build
cargo build --bin layanan-NAME       # Build specific service
```

### Code Quality
```bash
cargo fmt --all                      # Format all code
cargo clippy --workspace             # Lint (warnings as errors)
cargo test --workspace               # Run all tests
cargo audit                          # Security vulnerabilities
cargo deny check                     # License and advisory checks
```

### Frontend Development
```bash
cd antarmuka/daskrimti/portal
trunk serve --open                   # Dev server with hot reload
trunk build --release                # Production WASM build
```

### Infrastructure (Part of Main Workspace)
```bash
cargo build --bin authenc            # Build identity provider
cargo build --bin secreton           # Build secrets vault
```

## Key Libraries

- **Shared UI**: `shared-microfrontend` (alias for `lib-ui`)
- **Middleware**: `lib-middleware` (JWT validation, tracing)
- **Crypto**: `lib-crypto` (Ed25519 preferred over RSA)
- **Types**: `lib-types` (domain models)
- **Storage**: `lib-storage` (database, Raft)
- **Utils**: `lib-utils` (common utilities)
