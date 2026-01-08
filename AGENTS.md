# 🤖 AGENTS.md - AI Developer Guide for SIMPelv2

> **Notice to Agents**: This file serves as the **primary source of truth** for AI agents working on this repository. Read this before planning or executing tasks to understand the architecture, conventions, and workflows.

## 🌍 Project Context
**SIMPelv2** is a mission-critical **Rust Monorepo** for the Indonesian Attorney General's Office (Kejaksaan RI). It uses a microservices architecture for the backend (`layout/`) and a microfrontend architecture for the frontend (`antarmuka/`).

### 🔑 Key Tech Stack
-   **Language**: Rust (Edition 2024, Version 1.90+)
-   **Backend Framework**: Axum (HTTP), Tonic (gRPC)
-   **Frontend Framework**: Leptos (WASM)
-   **Database**: PostgreSQL (via `tokio-postgres` / `deadpool`)
-   **Infrastructure**: Docker, Kubernetes

---

## 🏗️ Workspace Structure

```bash
/var/www/simpelv2/
├── Cargo.toml          # ROOT WORKSPACE MANIFEST (Single source of truth)
├── lib/                # SHARED LIBRARIES (Avoid duplication here)
│   ├── types/          # Domain types (simpelv2-types -> lib-types)
│   ├── crypto/         # Cryptography (simpelv2-crypto -> lib-crypto)
│   ├── storage/        # Database & Raft (simpelv2-storage -> lib-storage)
│   ├── middleware/     # Axum middlewares (Auth, Tracing)
│   ├── utils/          # Common utilities (Error, Formatting)
│   └── ui/             # Shared UI components (formerly shared-microfrontend)
├── infra/              # CORE INFRASTRUCTURE
│   ├── authenc/        # Identity Provider (IAM)
│   └── secreton/       # Secret Management (Vault) - Unified Versioning
├── layanan/            # BACKEND MICROSERVICES
│   └── daskrimti/      # Main domain services (portal, bantuan, etc.)
└── antarmuka/          # FRONTEND MICROFRONTENDS
    ├── portal/         # Main Dashboard
    └── badiklat/       # Training module
```

---

## 📏 Critical Conventions (DO NOT VIOLATE)

### 1. 📦 Dependency Management
-   **Root Cargo.toml is King**: ALL external dependencies MUST be defined in `[workspace.dependencies]` in the root `Cargo.toml`.
-   **Inheritance**: Member crates MUST use `dependeny_name = { workspace = true }`. **NEVER** specify versions in member `Cargo.toml` files.
-   **Versioning**: All internal crates (including `authenc` and `secreton`) inherit `version.workspace = true`. The current workspace version is **0.1.0**.

### 2. 🧱 Code Organization
-   **DRY (Don't Repeat Yourself)**: Before writing a new utility, CHECK `lib/`.
    -   Need crypto? Use `lib-crypto`.
    -   Need database/raft? Use `lib-storage`.
    -   Need shared types? Use `lib-types`.
-   **Path Dependencies**: Use relative paths for internal dependencies (e.g., `path = "../../lib/types"`).
-   **Naming**: Internal libraries are named `lib-*` (e.g., `lib-types`, `lib-crypto`).

### 3. 🛡️ Build & Verification
The workspace is large. Always verify your changes widely.

-   **Check Everything**: `cargo check --workspace` (Run this frequently!)
-   **Build Everything**: `cargo build --workspace`
-   **Test Everything**: `cargo test --workspace`

### 4. 🔐 Security & Secrets
-   **Secreton**: This is the internal Vault. It uses `lib-crypto` for quantum-safe algorithms.
-   **Do NOT Hardcode Secrets**: Use configuration or `secreton-agent` for retrieving secrets.

---

## 🛠️ Common Workflows for Agents

### A. Add a New Dependency
1.  Add it to `[workspace.dependencies]` in root `Cargo.toml`.
2.  Run `cargo check --workspace` to ensure no conflicts.
3.  Add it to the member crate as `name.workspace = true`.

### B. Create a New Service (`layanan/new-service`)
1.  Create the directory and `Cargo.toml`.
2.  Add to `members` in root `Cargo.toml`.
3.  **Inherit versions**: `version.workspace = true`.
4.  Depend on `lib-middleware` for standard auth/logging.

### C. Debugging Build Errors
-   If `cargo check` fails on a "missing dependency" that exists:
    -   Check for **Circular Dependencies**.
    -   Check for **Feature Mismatches** (e.g., `tonic` needs `transport`).
    -   Check for **Legacy Naming** (e.g., `simpelv2-crypto` vs `lib-crypto`).

---

## 🚨 Known Quirks
-   **Missing Makefile**: `CONTRIBUTING.md` references a `Makefile`, but it may be missing in some environments. Use `cargo` commands directly.
-   **Secreton Integration**: Secreton was recently merged from a separate workspace. Ensure its internal paths always point to `lib-*` and not old `crates/*` if you refactor.
