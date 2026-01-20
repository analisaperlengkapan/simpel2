# SIMPelv2 - Technology Stack

## Core Technologies

- **Language**: Rust 1.90+ (Edition 2024)
- **Frontend**: Leptos 0.8.12 (WASM reactive framework)
- **Backend**: Axum 0.8.6 (async web framework)
- **Database**: PostgreSQL 15+ with tokio-postgres + deadpool-postgres
- **Cache**: Redis 7+
- **Build (WASM)**: Trunk
- **Container Orchestration**: MicroK8s / Kubernetes

## Security Stack

- Ed25519 cryptography (NOT RSA - removed due to RUSTSEC-2023-0071)
- ChaCha20-Poly1305 encryption
- jsonwebtoken 10.x with aws_lc_rs backend
- garde for validation (replaces validator to avoid idna vulnerability)
- Post-quantum ready (ML-DSA, ML-KEM, Falcon)

## Key Dependencies

- `shared-microfrontend`: Shared UI component library (40+ components)
- `authenc`: Custom IAM service (separate workspace)
- `secreton`: Custom secrets management (separate workspace)
- gRPC via tonic for inter-service communication

## Common Commands

```bash
# Development
cargo build                           # Build workspace
cargo test                            # Run tests
cargo clippy                          # Lint code
cargo fmt                             # Format code

# Frontend (from antarmuka/* directories)
trunk serve --open                    # Dev server with hot reload
trunk build --release                 # Production build

# Make targets
make build-all-fe                     # Build all microfrontends
make rust-test                        # Comprehensive test suite
make rust-fmt                         # Format all code
make rust-clippy                      # Lint all code
make up-dev                           # Start dev environment
make down                             # Stop all services

# Docker
docker compose up -d postgres redis   # Start infrastructure
docker compose -f docker-compose.yml -f docker-compose.dev.yml up
```

## Build Profiles

- `dev`: Fast compilation, light optimization, incremental builds
- `release`: Maximum optimization, LTO, single codegen unit
- `release-wasm`: Optimized for WASM bundle size
- `dev-wasm`: Fast WASM development builds

## Workspace Structure

- Main workspace: `Cargo.toml` (root)
- Independent workspaces: `infra/authenc`, `infra/secreton` (architectural isolation)
- Dependencies centralized in root `[workspace.dependencies]`
