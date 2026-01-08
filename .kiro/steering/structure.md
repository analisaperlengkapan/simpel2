# Project Structure and Organization

## Directory Layout

```
simpelv2/
├── antarmuka/              # Frontend microfrontends (Leptos + WASM)
├── layanan/                # Backend microservices (Rust + Axum)
├── infra/                  # Infrastructure and deployment
├── scripts/                # Build automation and tools
├── docs/                   # Documentation
├── config/                 # Configuration files
└── target/                 # Build artifacts (gitignored)
```

## Frontend Structure (antarmuka/)

Each microfrontend follows this pattern:

```
antarmuka/[module]/
├── src/
│   ├── lib.rs              # Entry point
│   ├── app.rs              # Main app component
│   ├── pages.rs            # Page components
│   ├── components/         # UI components
│   ├── api/                # API client (optional)
│   ├── types.rs            # Type definitions
│   └── utils/              # Utilities
├── styles/
│   └── main.css            # Module-specific styles
├── index.html              # HTML template
├── Trunk.toml              # Trunk configuration
├── Cargo.toml              # Dependencies
└── README.md               # Module documentation
```

**Shared Library** (`antarmuka/shared/`):
- `src/core/` - Types, constants, theme
- `src/components/` - Reusable UI components (layout, forms, feedback, navigation, display)
- `src/hooks/` - Custom hooks (storage, media query, debounce, auth)
- `src/utils/` - Utilities (validation, formatters, helpers)

## Backend Structure (layanan/)

Each microservice follows this pattern:

```
layanan/[service]/
├── src/
│   ├── lib.rs              # Library entry
│   ├── main.rs             # Binary entry (optional)
│   ├── handlers/           # HTTP handlers
│   ├── models/             # Data models
│   ├── services/           # Business logic
│   ├── db/                 # Database layer
│   ├── middleware/         # Custom middleware
│   └── error.rs            # Error types
├── migrations/             # SQL migrations
├── tests/                  # Integration tests
├── Cargo.toml              # Dependencies
└── README.md               # Service documentation
```

**Shared Services** (`layanan/shared/`):
- Common services used across multiple backends
- Each shared service is its own crate
- Examples: ai, bantuan, dasbor, dokumen, integrasi, konfigurasi, laporan, notifikasi

## Infrastructure (infra/)

**Authenc** (`infra/authenc/`):
- Separate workspace for IAM system
- Complete authentication and authorization
- MFA, SAML, OAuth2, OIDC support
- Own Cargo.lock and dependencies

**Secreton** (`infra/secreton/`):
- Separate workspace for secret management
- Secreton alternative
- Own Cargo.lock and dependencies

**Nginx** (`infra/nginx/`):
- Reverse proxy configuration
- SSL/TLS termination
- Load balancing

**Gerbang** (`infra/gerbang/`):
- API gateway (Envoy)
- Service mesh configuration

**K8s** (`infra/k8s/`):
- Kubernetes manifests
- Deployment, service, ingress configs

## Scripts (scripts/)

```
scripts/
├── cli/                    # CLI tool (Rust)
├── scripts/              # Build Scripts
│   ├── config.mk           # Configuration
│   ├── dev.mk              # Development targets
│   ├── ops.mk              # Operations targets
│   ├── rust.mk             # Rust workflow
│   ├── builds.mk           # Build operations
│   ├── test-core.mk        # Testing
│   └── help.mk             # Help system
├── tools/                  # Development tools
│   └── generators/         # Code generators
└── test/                   # Test automation
    └── validation/         # Validation scripts
```

## Documentation (docs/)

```
docs/
├── architecture/           # Architecture documentation
├── security/               # Security guidelines
├── api/                    # API documentation
├── deployment/             # Deployment guides
├── monitoring/             # Observability setup
└── [service-docs].md       # Individual service docs
```

## Configuration (config/)

```
config/
├── alertmanager/           # Alertmanager configs
├── prometheus/             # Prometheus configs
├── captcha.*.toml          # Captcha service configs
├── authenc.*.toml          # Authenc configs
└── secreton.*.toml         # Secreton configs
```

## Naming Conventions

**Directories**:
- `snake_case` for all directories
- Indonesian names for domain modules (badiklat, datun, etc.)
- English names for technical components (shared, infra, etc.)

**Files**:
- `snake_case.rs` for Rust source files
- `kebab-case.md` for documentation
- `kebab-case.toml` for configuration
- `kebab-case.yml` for YAML configs

**Rust Code**:
- `snake_case` for functions, variables, modules
- `PascalCase` for types, structs, enums, traits
- `SCREAMING_SNAKE_CASE` for constants
- `kebab-case` for crate names in Cargo.toml

**Components** (Leptos):
- `PascalCase` for component names
- Component files in `components/` directory
- One component per file (preferred)

## Module Organization

**Microfrontends** are organized by division:
- Each division has its own microfrontend
- Shared UI components in `antarmuka/shared`
- Independent deployment and versioning

**Microservices** are organized by function:
- Each service has a specific responsibility
- Shared services in `layanan/shared`
- Communication via REST APIs

**Infrastructure** is organized by layer:
- Security layer (authenc)
- Secret management (secreton)
- Networking (nginx, gerbang)
- Orchestration (k8s)

## Workspace Members

**Included in workspace**:
- All frontend microfrontends
- All backend services
- Shared libraries
- CLI tools

**Excluded from workspace**:
- `infra/authenc` - Separate workspace
- `infra/secreton` - Separate workspace
- `target/` - Build artifacts
- `dist/` - Distribution files

**Default members** (for faster builds):
- `scripts/cli`
- `antarmuka/shared`
- `antarmuka/portal`
- `antarmuka/pembinaan/*`
- `layanan/pembinaan/*`
- `layanan/shared/*`

## File Patterns

**Gitignored**:
- `target/` - Rust build artifacts
- `dist/` - Frontend build output
- `node_modules/` - Node dependencies
- `*.log` - Log files
- `.env` - Environment variables (use .env.example)
- `Cargo.lock` in libraries (keep in binaries)

**Version Controlled**:
- All source code
- Configuration templates (.example files)
- Documentation
- Build scripts
- CI/CD configs
- Kubernetes manifests

## Import Conventions

**Workspace dependencies**:
```rust
// Use workspace = true in member Cargo.toml
[dependencies]
axum = { workspace = true }
tokio = { workspace = true }
```

**Shared library imports**:
```rust
// Frontend
use shared_microfrontend::prelude::*;
use shared_microfrontend::components::*;

// Backend (if shared backend library exists)
use shared_backend::prelude::*;
```

**Module imports**:
```rust
// Prefer explicit imports
use crate::handlers::user_handler;
use crate::models::User;

// Use prelude for common items
use crate::prelude::*;
```

## Testing Organization

**Unit tests**:
- In same file as code: `#[cfg(test)] mod tests { ... }`
- Or in `tests/` subdirectory within module

**Integration tests**:
- In `tests/` directory at crate root
- One file per integration test suite

**Test naming**:
- `test_[function]_[scenario]_[expected]`
- Example: `test_login_with_valid_credentials_succeeds`

## Documentation Standards

**Code documentation**:
- Public APIs must have doc comments (`///`)
- Module-level docs in `lib.rs` or `mod.rs`
- Examples in doc comments when helpful

**README files**:
- Every module/service has a README.md
- Include: purpose, usage, API, configuration
- Keep concise and up-to-date

**Architecture docs**:
- High-level docs in `docs/`
- Diagrams using Mermaid or PlantUML
- Decision records for major changes
