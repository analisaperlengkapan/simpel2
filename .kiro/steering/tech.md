# Technology Stack

## Core Technologies

**Language**: Rust 1.90+ (edition 2024)
**Frontend**: Leptos 0.8.x + WebAssembly
**Backend**: Axum 0.8.6 web framework
**Database**: PostgreSQL 15+ with tokio-postgres
**Build System**: Cargo workspace + Trunk 0.21.14 (for WASM)

## Key Dependencies

**Backend**:
- axum, axum-extra - Web framework
- tokio - Async runtime
- serde, serde_json - Serialization
- tower, tower-http - Middleware (CORS, tracing, compression)
- deadpool-postgres - Connection pooling
- refinery - Database migrations
- jsonwebtoken - JWT authentication (v10 with aws_lc_rs)
- argon2 - Password hashing
- vaultrs - HashiCorp Vault integration

**Frontend**:
- leptos, leptos_router, leptos_meta - UI framework
- wasm-bindgen - WASM bindings
- web-sys, js-sys - Browser APIs
- gloo - WASM utilities (net, storage, timers)

**Security**:
- ed25519-dalek - Digital signatures (preferred over RSA)
- x25519-dalek - Key exchange
- blake3, sha2 - Hashing
- aes-gcm - Encryption

**Monitoring**:
- tracing, tracing-subscriber - Structured logging
- prometheus, metrics - Metrics collection
- opentelemetry - Distributed tracing
- sentry - Error tracking

## Build Commands

### Development

```bash
# Setup development environment
make setup-dev

# Start all services
make up-dev

# Build all microfrontends
make build-frontends

# Serve portal with hot reload
make serve-portal

# Run backend tests
make test-backend

# Run frontend tests
make test-frontends
```

### Testing

```bash
# Run all tests
cargo test

# Run specific service tests
cargo test --package layanan-keamanan

# Run with coverage
make test-coverage

# Linting
cargo clippy --all-targets --all-features

# Formatting
cargo fmt --all
```

### Building

```bash
# Build all backend services
cargo build --release

# Build specific microfrontend
cd antarmuka/portal
trunk build --release

# Build all frontends in parallel
make build-all-fe

# Build with Cargo Chef (Docker optimization)
make build-chef
```

### Deployment

```bash
# Deploy to staging
make deploy-staging

# Deploy to production
make deploy-prod

# Deploy to Kubernetes
microk8s kubectl apply -f infra/k8s/
```

## Project Structure

```
simpelv2/
├── antarmuka/          # Frontend microfrontends (Leptos)
│   ├── shared/         # Shared component library
│   ├── portal/         # Main gateway
│   └── [modules]/      # Individual microfrontends
├── layanan/            # Backend microservices (Rust)
│   ├── shared/         # Shared backend services
│   └── [services]/     # Individual services
├── infra/              # Infrastructure
│   ├── authenc/        # IAM system (separate workspace)
│   ├── secreton/       # Secret management (separate workspace)
│   ├── nginx/          # Reverse proxy config
│   └── k8s/            # Kubernetes manifests
├── scripts/            # Build and automation scripts
│   ├── cli/            # CLI tool
│   └── makefiles/      # Modular Makefiles
└── docs/               # Documentation
```

## Workspace Configuration

The project uses Cargo workspace with resolver 2. Key points:

- **Centralized dependencies** in root Cargo.toml `[workspace.dependencies]`
- **No version pinning** in member crates (use `workspace = true`)
- **Separate workspaces** for authenc and secreton (architectural independence)
- **Default members** exclude problematic microfrontends for faster builds

## Development Tools

**Required**:
- Rust toolchain (rustup)
- cargo-watch - File watching
- trunk - WASM bundler
- sqlx-cli - Database migrations
- Docker & Docker Compose

**Recommended**:
- rust-analyzer - IDE support
- cargo-audit - Security auditing
- cargo-chef - Docker build optimization

## CI/CD

GitLab CI with 9 stages:
1. Preparation - Dependencies
2. Quality - Format, clippy, docs
3. Security - Audit, deny, SAST
4. Build - Parallel builds with caching
5. Test - Unit, integration, coverage
6. Security-Scan - Trivy, secrets, IaC
7. SBOM - Supply chain security
8. Deploy - Multi-environment
9. Cleanup - Cache management

## Performance Profiles

**Development** (`cargo build`):
- opt-level = 1 (light optimization)
- incremental = true
- Dependencies optimized at level 2

**Release** (`cargo build --release`):
- opt-level = 3 (maximum optimization)
- lto = "fat" (full link-time optimization)
- codegen-units = 1
- strip = true (remove debug symbols)

**WASM** (`trunk build --release`):
- opt-level = "z" (size optimization)
- lto = "fat"
- Custom profile: release-wasm

## Security Standards

- No unsafe code (workspace lint: `unsafe_code = "forbid"`)
- Ed25519 signatures (not RSA)
- Argon2 password hashing
- JWT with aws_lc_rs backend
- No deprecated dependencies (garde replaces validator)
- Regular security audits with cargo-audit
