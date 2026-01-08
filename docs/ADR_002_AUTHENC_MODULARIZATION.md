# Architecture Decision Record: Authenc Modularization Strategy

**Status**: Proposed
**Date**: 2026-01-08
**Context**: Decision on whether to split `authenc` into multiple physical crates (like Secreton) or keep it as a modular monolith.

## 1. Executive Summary
**Recommendation**: **Keep as Modular Monolith** (Single Crate with multiple Modules).

Unlike `secreton` which has distinct "Agent" vs "Server" vs "CLI" runtime profiles that benefit from physical separation, `authenc` is a cohesive Identity Provider application. Splitting it into physical crates introduces complexity without significant architectural benefit, given that standard shared logic (`crypto`, `types`) has already been moved to `lib/`.

---

## 2. Option Analysis

### Option A: Physical Modularization (Multi-Crate)
Splitting into `crates/api`, `crates/core`, `crates/models`, etc.
*   **Pros**:
    *   **Strict Boundaries**: Impossible to create cyclic dependencies between layers.
    *   **Build Time**: Parallel compilation of crates (small gain for 1 app).
*   **Cons**:
    *   **Complexity**: Managing internal path dependencies and versions.
    *   **Refactoring Friction**: moving a struct from `core` to `api` requires editing `Cargo.toml`.
    *   **Overhead**: `authenc` is rarely imported by others; it is consumed as a service (HTTP/gRPC).

### Option B: Logical Modularization (Modular Monolith) - **Recommended**
Using Rust `mod` system (`src/services`, `src/handlers`, `src/models`).
*   **Pros**:
    *   **Simplicity**: Single `Cargo.toml`, unified versioning.
    *   **Refactoring Velocity**: Easy to move code between modules.
    *   **Compiler Optimization**: `LTO` (Link Time Optimization) works best on single compilation units.
*   **Cons**:
    *   Easier to violate layer boundaries (mitigated by strict `pub(crate)` vs `pub` usage).

---

## 3. Why Secreton was different?
`secreton` was split because it had:
1.  **Secreton Agent**: A standalone binary sidecar running on client nodes.
2.  **Secreton CLI**: A management tool distributed to admins.
3.  **Secreton Server**: The main vault.

`authenc` primarily outputs **one artifact**: The IAM Server. (The CLI is a small utility, distinct from the core business domain).

## 4. Architectural Guidelines
1.  **Shared Libraries**: Logic used by arguably *anyone* (e.g., standard crypto wrappers, shared types) goes to `lib/` (Root Workspace).
2.  **Business Logic**: Stays inside `infra/authenc/src/services/`.
3.  **Transport Adapters**: Stays inside `infra/authenc/src/handlers/` (Axum) or `grpc/` (Tonic).

## 5. Conclusion
Adopt the **Modular Monolith** pattern.
*   Enforce boundaries via `mod.rs` visibility rules.
*   Do NOT fracture into multiple crates unless a component needs to be deployed independently (e.g., if we build a standalone `authenc-sidecar`).
