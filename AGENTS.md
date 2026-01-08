# 🤖 AGENTS.md - AI Developer Guide for SIMPelv2

> **Notice to Agents**: This file serves as the **primary source of truth** for AI agents working on this repository. Read this before planning or executing tasks to understand the architecture, conventions, and workflows.

## 🌍 Project Context
**SIMPelv2** is a mission-critical **Rust Monorepo** for the Indonesian Attorney General's Office (Kejaksaan RI). It uses a microservices architecture for the backend (`layanan/`) and a microfrontend architecture for the frontend (`antarmuka/`).

### 🔑 Key Tech Stack
-   **Language**: Rust (Edition 2024, Version 1.90+)
-   **Backend Framework**: Axum (HTTP), Tonic (gRPC)
-   **Frontend Framework**: Leptos (WASM)
-   **Database**: PostgreSQL (via `tokio-postgres` / `deadpool`)
-   **Infrastructure**: Docker, Kubernetes

---

## 🏗️ Workspace Structure & Naming Conventions

### 1. Structure Overview
```bash
/var/www/simpelv2/
├── Cargo.toml          # ROOT WORKSPACE MANIFEST (Single source of truth)
├── lib/                # SHARED LIBRARIES (Avoid duplication here)
│   ├── types/          # Domain types (lib-types)
│   ├── crypto/         # Cryptography (lib-crypto)
│   ├── storage/        # Database & Raft (lib-storage)
│   ├── middleware/     # Axum middlewares (lib-middleware)
│   ├── utils/          # Common utilities (lib-utils)
│   └── ui/             # Shared UI components (lib-ui)
├── infra/              # CORE INFRASTRUCTURE
│   ├── authenc/        # Identity Provider (authenc)
│   └── secreton/       # Secret Management (secreton)
├── layanan/            # BACKEND MICROSERVICES
│   └── daskrimti/      # Main domain services
└── antarmuka/          # FRONTEND MICROFRONTENDS
    └── [domain]/       # Microfrontend implementations
```

### 2. Naming & Placement Rules
| Type | Directory Location | Package Name Schema | Example |
| :--- | :--- | :--- | :--- |
| **Microfrontend** | `antarmuka/[name]/` | `[name]-microfrontend` | `portal-microfrontend` |
| **Microservice** | `layanan/daskrimti/[name]/` | `layanan-[name]` | `layanan-portal` |
| **Shared Lib** | `lib/[name]/` | `lib-[name]` | `lib-utils` |
| **Infra** | `infra/[name]/` | `[name]` | `authenc` |

---

## 📏 Critical Conventions (DO NOT VIOLATE)

### 1. 📦 Dependency Management
-   **Root Cargo.toml is King**: ALL external dependencies MUST be defined in `[workspace.dependencies]` in the root `Cargo.toml`.
-   **Inheritance**: Member crates MUST use `dependeny_name = { workspace = true }`. **NEVER** specify versions in member `Cargo.toml` files.
-   **Versioning**: All internal crates (including `authenc` and `secreton`) inherit `version.workspace = true`. The current workspace version is **0.1.0**.

### 2. 🧹 Code Quality & Maintenance Tools
Use these standard commands instead of `make`:
-   **Verification**: `cargo check --workspace` (Run frequently!)
-   **Formatting**: `cargo fmt --all` (Enforce style guides)
-   **Linting/Fixing**: `cargo fix --workspace --allow-dirty` (Auto-fix warnings)
-   **Security**: `cargo audit` (Check for vulnerabilities in dependencies)

### 3. 🛡️ System Integration Strategy
The ecosystem is designed to be tightly integrated:
-   **Authenc (Identity)**: centralizes user identities. Services should NOT manage users locally.
-   **Secreton (Vault)**: centralizes secrets/keys. Services retrieve DB credentials/keys from Secreton at startup.
-   **Lib-Middleware**: The bridge. All `layanan-*` services MUST use `lib-middleware` to transparently integrate:
    -   JWT Validation (via Authenc public keys)
    -   Tracing/Logging
    -   Error Handling
-   **Portal**: The visual integrator. Consumes `lib-ui` for consistent design system and composes `*-microfrontend` WASM bundles.

---

## 🛠️ Common Workflows for Agents

### A. Add a New Microservice
1.  Create directory: `layanan/daskrimti/my-feature/`.
2.  Initialize `Cargo.toml`. Name it `layanan-my-feature`.
3.  Add to root `members`.
4.  Add `lib-middleware` and `lib-utils` dependencies.
5.  Implement Axum router using `lib-middleware` layers.

### B. Add a New Microfrontend
1.  Create directory: `antarmuka/my-feature/`.
2.  Initialize `Cargo.toml`. Name it `my-feature-microfrontend`.
3.  Add to root `members`.
4.  Add `lib-ui` dependency.
5.  Implement Leptos components.

### C. Debugging
-   **Do NOT look for a Makefile**. It does not exist. Use cargo commands directly.
-   If `cargo audit` fails, check if the vulnerability affects the specific deployment usage before upgrading.
