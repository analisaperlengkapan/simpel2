# Copilot Instructions for SIMPelv2

## Project Overview

SIMPelv2 is a modular, microservices-based platform for Indonesian government asset management (BMN), using Rust (Axum) for backend and Leptos (WASM) for microfrontends. The architecture is designed for scalability, security, and maintainability, following international standards (ISO/IEC 25010, OWASP, 12-Factor App), best practices, and "next practice" (AI-driven, zero-trust, cloud-native).

## Architecture & Data Flow

- **Microfrontends**: Each `antarmuka/<modul>` is a Leptos WASM app, communicating with backend via REST/gRPC through the API gateway (`infra/gerbang/`).
- **Backend Microservices**: Each `layanan/<service>` is a Rust service, exposing REST APIs, using PostgreSQL (multi-schema), and Vault for secrets.
- **API Gateway**: Nginx/Envoy routes all traffic, enforces security headers, and provides service discovery.
- **Security**: Centralized in `layanan/keamanan/` (auth, RBAC, MFA, JWT, Vault integration, CSP, audit trail).
- **AI/ML**: Provided by `layanan/ai/` (LLM, OCR, RAG, RLHF, Qdrant vector search).
- **Observability**: Prometheus, Grafana, Loki, and structured logging in `infra/monitoring/`.

## Developer Workflows (Best Practice)

- **Setup**: Copy `.env.example` to `.env` and configure secrets via Vault (never commit credentials).
- **Build/Run All (Dev)**: `make up-dev` (Docker Compose: all services, frontends, gateway, monitoring).
- **Stop All**: `make down`
- **Logs**: `make logs` (aggregated logs)
- **Build for Production**: `make build` (optimized, WASM minified)
- **Deploy**: `make deploy-prod` (CI/CD pipeline, GitHub Actions)
- **Individual Microfrontend**: `trunk serve --config antarmuka/<modul>/Trunk.toml`
- **Individual Microservice**: `cargo run --bin layanan-<service>`
- **Testing**: `cargo test --all` (unit/integration), `/scripts/test/` for E2E
- **Linting**: `cargo clippy --all -- -D warnings` (fail on warnings)
- **CI/CD**: GitHub Actions workflows in `.github/workflows/` (ci-backend.yml, ci-frontend.yml, ci-security.yml, ci-quality.yml, release.yml)

## Project Conventions (International Standard)

- **Code Quality**: Rust 2024 edition (1.90+), enforced Clippy, workspace-level dependencies in root `Cargo.toml`.
- **Security**: Zero-trust, MFA, RBAC, JWT, CSP, Vault for all secrets, audit logging (see `/docs/security/`, `.github/SECURITY.md`).
- **Documentation**: All architecture, API, and service docs in `/docs/` (keep up-to-date, use diagrams).
- **Port Mapping**: Each service/microfrontend uses a fixed port (see `/README.md`).
- **Shared Code**: Use `shared/` for cross-module logic/components (DRY principle).
- **12-Factor App**: Config via environment, stateless services, logs to stdout, disposable dev/prod parity.
- **Compliance**: Follow ISO/IEC 25010 for software quality, OWASP for security, GDPR for data privacy.
- **Dependency Management**: Dependabot weekly updates (grouped by ecosystem), immediate security patches.

## Integration & Patterns (Next Practice)

- **API Gateway**: All API calls (internal/external) go through `infra/gerbang/` (Envoy/Nginx, service mesh ready).
- **Database**: PostgreSQL with tokio-postgres + deadpool-postgres (connection pooling), refinery (migrations), sea-query (query builder). Multi-schema per service, no shared DB logic. **NO sqlx** (migrated for security).
- **Observability**: Distributed tracing with tracing-opentelemetry 0.32+, metrics via Prometheus, logs via Grafana Loki.
- **AI/ML**: All ML/AI features centralized in `layanan/ai/` (document OCR, LLM, vector search, RAG, RLHF).
- **Testing**: Integration tests in `/scripts/test/`, Rust unit tests in each service, contract tests for APIs. CI coverage via cargo-llvm-cov.
- **DevSecOps**: Automated security scans (cargo-audit, cargo-deny, Semgrep, Trivy, TruffleHog) in GitHub Actions workflows.
- **Cloud-Native Ready**: All services containerized, k8s manifests in `infra/k8s/`, Helm charts supported.
- **Secrets Management**: Secreton (custom Rust vault) for all secrets, zero credentials in code/config.

## Key Files & Directories

- `/antarmuka/` - Leptos 0.8.x microfrontends (WASM, CSR SPA)
- `/layanan/` - Rust backend microservices (Axum)
- `/layanan/shared/authenc/` - Security service (97K LOC, Ed25519, SHA256 TOTP, MFA, RBAC)
- `/infra/` - Gateway, Nginx, k8s, Vault, monitoring
- `/infra/secreton/` - Custom Rust vault (ChaCha20-Poly1305, X25519, Argon2)
- `/scripts/` - Automation, build, test scripts
- `/docs/` - All documentation (architecture, API, CI/CD, security)
- `/.github/` - GitHub Actions workflows, Dependabot, issue templates, SECURITY.md
- `/Makefile` - Main entry for developer workflows
- `/Cargo.toml` - Workspace dependencies (centralized, edition 2024)

## Technology Stack Details

### Backend Dependencies (Modern Stack - 2024)

- **Database**: tokio-postgres 0.7, deadpool-postgres 0.14, refinery 0.8, sea-query 0.31 (**NO sqlx**)
- **Cryptography**: blake3 1.8.2, sha2 0.10.9, ed25519-dalek (**NO sha1**)
- **XML Parsing**: quick-xml 0.38.3 (**NO xml-rs/xmltree**)
- **Tracing**: tracing-opentelemetry 0.32+ for unified observability
- **Web Framework**: axum (Tokio ecosystem)
- **Authentication**: JWT, TOTP (SHA256), MFA, session management

### Frontend Stack

- **Framework**: Leptos 0.8.x with CSR (Client-Side Rendering) SPA
- **Build Tool**: Trunk 0.21.4+ for WASM optimization
- **Shared Components**: `antarmuka/shared/` (v0.2.0, 40+ components, Kejaksaan RI branding)
- **Target**: wasm32-unknown-unknown
- **Bundle Size Limit**: 2MB per microfrontend (enforced in CI)

### CI/CD Workflows

- **ci-backend.yml**: Rust check, test (PostgreSQL), clippy, fmt, release build
- **ci-frontend.yml**: WASM check/build (matrix 11 modules), bundle size validation
- **ci-security.yml**: cargo-audit, cargo-deny, Semgrep SAST, Trivy container scan, TruffleHog secrets scan
- **ci-quality.yml**: Documentation, code coverage (cargo-llvm-cov), benchmarks, dependency health
- **release.yml**: Multi-platform builds, Docker images, GitHub releases, K8s deployment

## Examples

- **Add Microfrontend**: Copy folder in `antarmuka/`, update `Trunk.toml`, register in gateway config, follow port convention, ensure CI workflows pass.
- **Add Microservice**: Copy folder in `layanan/`, update `Cargo.toml`, add to Docker Compose, create refinery migrations, ensure PostgreSQL connection pooling.
- **Run AI Service**: `cargo run --bin layanan-ai` from `/layanan/ai/`.
- **Add Integration Test**: Place in `/scripts/test/`, use Docker Compose for E2E, add to CI workflow.
- **Security Fix**: Report via `.github/SECURITY.md` process, create PR with security label, ensure cargo-audit passes.

---

For more, see `/README.md` and `/docs/README.md`. If a pattern or workflow is unclear, check `/docs/` or ask for clarification. Always follow secure, maintainable, and auditable coding practices. All PRs must pass GitHub Actions workflows before merge.
