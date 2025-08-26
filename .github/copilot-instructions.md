
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
- **Deploy**: `make deploy-prod` (CI/CD pipeline, GitLab)
- **Individual Microfrontend**: `trunk serve --config antarmuka/<modul>/Trunk.toml`
- **Individual Microservice**: `cargo run --bin layanan-<service>`
- **Testing**: `cargo test --all` (unit/integration), `/scripts/test/` for E2E
- **Linting**: `cargo clippy --all -- -D warnings` (fail on warnings)
- **CI/CD**: GitLab CI, see `/docs/ci-cd.md`, `/docs/gitlab-ci-generator.md`

## Project Conventions (International Standard)
- **Code Quality**: Rust 2021 edition, enforced Clippy, workspace-level dependencies in root `Cargo.toml`.
- **Security**: Zero-trust, MFA, RBAC, JWT, CSP, Vault for all secrets, audit logging (see `/docs/security/`).
- **Documentation**: All architecture, API, and service docs in `/docs/` (keep up-to-date, use diagrams).
- **Port Mapping**: Each service/microfrontend uses a fixed port (see `/README.md`).
- **Shared Code**: Use `shared/` for cross-module logic/components (DRY principle).
- **12-Factor App**: Config via environment, stateless services, logs to stdout, disposable dev/prod parity.
- **Compliance**: Follow ISO/IEC 25010 for software quality, OWASP for security, GDPR for data privacy.

## Integration & Patterns (Next Practice)
- **API Gateway**: All API calls (internal/external) go through `infra/gerbang/` (Envoy/Nginx, service mesh ready).
- **Database**: PostgreSQL, multi-schema per service, migrations in each service folder, no shared DB logic.
- **Observability**: Distributed tracing, metrics, and logs via Prometheus, Grafana, Loki.
- **AI/ML**: All ML/AI features centralized in `layanan/ai/` (document OCR, LLM, vector search, RAG, RLHF).
- **Testing**: Integration tests in `/scripts/test/`, Rust unit tests in each service, contract tests for APIs.
- **DevSecOps**: Automated security scans, dependency checks, and SAST in CI/CD pipeline.
- **Cloud-Native Ready**: All services containerized, k8s manifests in `infra/k8s/`, Helm charts supported.

## Key Files & Directories
- `/antarmuka/` - Leptos microfrontends (WASM)
- `/layanan/` - Rust backend microservices
- `/infra/` - Gateway, Nginx, k8s, Vault, monitoring
- `/scripts/` - Automation, build, test scripts
- `/docs/` - All documentation (architecture, API, CI/CD, security)
- `/Makefile` - Main entry for developer workflows

## Examples
- **Add Microfrontend**: Copy folder in `antarmuka/`, update `Trunk.toml`, register in gateway config, follow port convention.
- **Add Microservice**: Copy folder in `layanan/`, update `Cargo.toml`, add to Docker Compose, create DB migration.
- **Run AI Service**: `cargo run --bin layanan-ai` from `/layanan/ai/`.
- **Add Integration Test**: Place in `/scripts/test/`, use Docker Compose for E2E.

---
For more, see `/README.md` and `/docs/README.md`. If a pattern or workflow is unclear, check `/docs/` or ask for clarification. Always follow secure, maintainable, and auditable coding practices.
