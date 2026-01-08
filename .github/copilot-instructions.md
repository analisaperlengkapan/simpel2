# SIMPelv2 AI Coding Agent Instructions

## Project Overview

SIMPelv2 is an **enterprise-grade asset management system** (Sistem Informasi Manajemen Pengelolaan BMN) for Indonesia's Attorney General's Office, built with **microfrontend + microservices architecture** in Rust. Live production: https://simpel.kejaksaan.go.id/

**Key Characteristics:**

- 11 independent Leptos 0.8.x microfrontends + shared component library
- Backend microservices (Axum framework)
- Zero-trust security architecture with custom IAM (Authenc) and secrets management (Secreton)
- Enterprise CI/CD with 12+ security tools
- Government compliance focus (FIPS, GDPR, zero-trust)

## Critical Architecture Patterns

### 1. **Microfrontend Independence**

Each microfrontend in `antarmuka/` is a standalone Leptos 0.8.x SPA:

- **NO shared state** between microfrontends
- Authentication via OAuth2 redirect flow through Portal
- Session stored in `localStorage` (not `sessionStorage`) for cross-tab sync
- All use `shared-microfrontend` library (40+ production-ready components)

**Auth Integration Pattern:**

```rust
use shared_microfrontend::hooks::use_auth;
use shared_microfrontend::components::auth::ProtectedRoute;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=LoginRedirectPage />
                <Route path="/dashboard" view=|| {
                    view! { <ProtectedRoute><Dashboard /></ProtectedRoute> }
                }/>
            </Routes>
        </Router>
    }
}
```

**Critical:** Microfrontends NEVER handle credentials - Portal does all authentication.

### 2. **Independent Infrastructure Workspaces**

`infra/authenc/` and `infra/secreton/` are **separate Cargo workspaces** (not in root workspace):

- Separate `Cargo.toml` and `Cargo.lock` for architectural independence
- Zero-trust: they communicate via mTLS, not shared code
- Build separately: `cd infra/authenc && cargo build`
- Never reference these from main workspace members

### 3. **Shared Component Library (`antarmuka/shared/`)**

Thread-safe, Leptos 0.8.x-compatible components:

- Use `RwSignal<T>` for reactive state (NOT `create_signal`)
- All components must be `Send + Sync`
- Export via `prelude::*` for easy imports
- **DO NOT duplicate** - microfrontends import from `shared-microfrontend`

## Development Workflows

### Building

```bash
# Frontend microfrontends (Leptos + WASM)
cd antarmuka/portal && trunk build --release

# Backend services (Rust)
cargo build --bin layanan-keamanan

# Common tasks
cargo run -p portal-microfrontend  # Run specific frontend
cargo test --workspace             # Run all tests
cargo clippy --workspace           # Lint all code
```

### Testing

```bash
# Rust unit + integration tests
cargo test --workspace               # All workspace tests
cargo test --package layanan-ai      # Specific service

# Frontend tests
cd antarmuka/portal && trunk test

# Security validation
cargo audit
```

### Key Scripts

- `scripts/security_validation.sh` - Comprehensive security checks
- `scripts/analyze-bundle-sizes.sh` - WASM bundle analysis

## Code Conventions

### Rust Backend (Axum Services)

```rust
// Standard service structure in layanan/
├── src/
│   ├── main.rs              // Entry point
│   ├── config.rs            // Configuration
│   ├── handlers/            // HTTP handlers
│   ├── services/            // Business logic
│   ├── models/              // Data models
│   └── error.rs             // Error types

// Use workspace dependencies (from root Cargo.toml)
axum = { workspace = true }
tokio = { workspace = true, features = ["full"] }
```

**Critical:** Use Ed25519 for signing (NOT RSA) - faster and more secure. See `workspace.dependencies` in root `Cargo.toml`.

### Frontend Microfrontends (Leptos 0.8.x)

```rust
// Use modern Leptos patterns
use leptos::prelude::*;  // NOT leptos::*

// Signals in 0.8.x
let (count, set_count) = signal(0);  // NOT create_signal

// Event handlers
on:click=move |_| set_count.set(count.get() + 1)  // NOT on_click

// Components must be Send + Sync
#[component]
pub fn MyComponent() -> impl IntoView { }  // Auto-derives Send + Sync
```

**Avoid:** Dynamic imports/lazy loading - Leptos 0.8 WASM doesn't support it. Use build-time code splitting via microfrontend architecture.

### Authentication Flow

**Portal** (only):

1. Login form + CAPTCHA → Authenc API
2. MFA setup (first time) / verification (returning)
3. OAuth2 token issuance + SSO cookie
4. Redirect back to microfrontend

**Microfrontends:**

1. Check `localStorage` for session
2. If missing → redirect to Portal with `return_url`
3. Portal redirects back after auth
4. Use `use_auth()` hook for session access

**Authenc + Secreton:**

- Authenc: Authentication, authorization, JWT, MFA
- Secreton: MFA secret storage, encryption keys
- Communicate via mTLS gRPC

## Database & Storage

```rust
// PostgreSQL with deadpool connection pooling
use deadpool_postgres::Pool;

// Migrations with refinery
// Located in layanan/*/migrations/

// Redis for caching
use redis::aio::ConnectionManager;
```

**Schema strategy:** Multi-schema PostgreSQL (one per service) - see `docker-compose.yml`.

## Docker & Deployment

```bash
# Development
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# Production
docker compose -f docker-compose.yml -f docker-compose.prod.yml up
```

**Dockerfile pattern:** Multi-stage with Cargo Chef for caching (see any `Dockerfile` in `antarmuka/` or `layanan/`).

## Security Patterns

### Zero-Trust Implementation

- **No implicit trust** between services
- All service-to-service via mTLS
- JWT validation on every request
- Immutable audit logs (PostgreSQL)

### Secrets Management

```rust
// NEVER use environment variables for secrets
// Use Secreton client to fetch at runtime
let secret = secreton_client.get_secret("KJA001/database_password").await?;
```

### Cryptography

- **Prefer:** Ed25519 (signing), X25519 (key exchange), ChaCha20-Poly1305 (encryption)
- **Avoid:** RSA (unless required for compatibility)
- Post-quantum readiness: Design with algorithm agility

## CI/CD Pipeline

9-stage GitLab pipeline (`gitlab-ci.yml`):

1. **Preparation** - Toolchain, dependencies
2. **Quality** - Format, clippy, spellcheck, docs
3. **Security** - Audit, deny, geiger, miri, vet, SAST
4. **Build** - Parallel cargo + trunk builds with sccache
5. **Test** - Unit, integration, coverage, performance, fuzz
6. **Security-Scan** - Trivy, license, secrets, IaC
7. **SBOM** - CycloneDX/SPDX generation
8. **Deploy** - Dev/staging/prod environments
9. **Cleanup** - Cache management

**Key tools:** cargo-audit, cargo-deny, semgrep, trivy, checkov, miri

## Common Pitfalls

❌ **DON'T:**

- Reference `infra/authenc` or `infra/secreton` from main workspace
- Use `create_signal` in Leptos 0.8.x (use `signal()`)
- Store JWT tokens in cookies (use `localStorage`)
- Duplicate auth logic in microfrontends
- Use RSA for new implementations
- Commit secrets to git

✅ **DO:**

- Import from `shared-microfrontend` for UI components
- Use `trunk build --release` for production WASM
- Run `make rust-fmt` and `make rust-clippy` before commits
- Test authentication flows across multiple browser tabs
- Use `ProtectedRoute` for all authenticated pages
- Check `make help` for available commands

## File Locations Reference

- **Frontend shared components:** `antarmuka/shared/src/components/`
- **Auth hooks:** `antarmuka/shared/src/hooks/use_auth.rs`
- **Backend services:** `layanan/{service_name}/`
- **CI/CD:** `.gitlab-ci.yml`
- **Docker configs:** `docker-compose*.yml`
- **K8s manifests:** `infra/k8s/`
- **Documentation:** `docs/` (40+ files)
- **Architecture docs:** `antarmuka/COMPLETE_AUTH_FLOW_ARCHITECTURE.md`, `docs/MFA_ARCHITECTURE_DOCUMENTATION.md`

## Quick Reference

```bash
# Start dev environment
docker compose up -d

# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Format + lint
cargo fmt --all && cargo clippy --workspace

# Serve portal with hot reload
cd antarmuka/portal && trunk serve --open

# Build specific service
cargo build --bin layanan-dasbor

# Security scan
cargo audit

# Deploy to dev
# Deployment is handled via CI/CD pipelines
```

## Questions to Ask Before Implementation

1. **For microfrontends:** Is this auth-related? → Use Portal + `use_auth()` hook
2. **For services:** Does this need secrets? → Use Secreton client (NOT env vars)
3. **For shared code:** Should this be in `antarmuka/shared/`? → Yes if reusable UI
4. **For dependencies:** Is this in workspace deps? → Check root `Cargo.toml` first
5. **For tests:** Integration or unit? → Integration use testcontainers for DB

## Additional Resources

- Contributing guide: `CONTRIBUTING.md`
- Main README: `README.md` (comprehensive overview)
- Auth architecture: `antarmuka/COMPLETE_AUTH_FLOW_ARCHITECTURE.md`
- MFA flow: `docs/MFA_ARCHITECTURE_DOCUMENTATION.md`
- Microfrontend integration: `antarmuka/shared/MICROFRONTEND_INTEGRATION.md`
