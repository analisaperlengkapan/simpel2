# gRPC Integration Analysis Report

**Date:** 2025-02-23
**Scope:** Authenc, Secreton, Integrasi, Perlengkapan, Portal

## Executive Summary
The core infrastructure services (`authenc`, `secreton`) exhibit highly optimized gRPC integration with connection pooling, circuit breakers, and security features. However, the business logic integration between `perlengkapan` and `integrasi` deviates from the microservice architecture, relying on a direct monolithic library dependency that bypasses the `integrasi` gRPC service layer.

## Detailed Findings

### 1. Authenc <-> Secreton (Optimal)
*   **Protocol:** gRPC (Tonic)
*   **Client Location:** `infra/authenc/src/secreton_client`
*   **Connection Management:**
    *   Uses `tonic::transport::Channel` with internal pooling.
    *   Channel is reused across requests (Singleton pattern in AppState).
*   **Resilience:**
    *   Custom `CircuitBreaker` implementation.
    *   Exponential backoff retries for connection failures.
*   **Security:** Supports TLS/mTLS configuration via `new_with_tls`.

### 2. Perlengkapan <-> Authenc (Good)
*   **Protocol:** gRPC (Tonic)
*   **Client Location:** `layanan/pembinaan/perlengkapan/src/grpc_clients.rs`
*   **Connection Management:**
    *   Client is initialized in `main.rs` and stored in `AppState`.
    *   Shared reference ensures connection reuse.
*   **Resilience:**
    *   Startup retry logic (5 attempts) in `main.rs`.

### 3. Perlengkapan <-> Integrasi (Suboptimal / Architectural Issue)
*   **Status:** **Disconnected / Library-Based**
*   **Defined Client:** `IntegrasiClient` is defined in `grpc_clients.rs` but **unused**.
*   **Actual Implementation:**
    *   `Perlengkapan` depends directly on `layanan-integrasi` as a crate (`path = "../../daskrimti/integrasi"`).
    *   The `SimanIntegration` module uses `MonsaktiClient` from this library to make direct HTTP calls to external APIs (MonSAKTI/SIMAN).
    *   This bypasses the `layanan-integrasi` gRPC service, effectively making `perlengkapan` a monolith with `integrasi`'s logic.
*   **Runtime Status:**
    *   In `layanan/pembinaan/perlengkapan/src/main.rs`, the service is initialized with `siman: None`, meaning SIMAN integration is **disabled** at runtime.

### 4. Portal <-> Backend Services (Standard REST)
*   **Protocol:** HTTP/JSON (REST)
*   **Library:** `gloo_net` (Fetch API)
*   **Optimality:**
    *   Appropriate for a WASM frontend.
    *   Avoids gRPC-Web complexity.
    *   Uses environment variables (`AUTHENC_API_URL`) for service discovery.

## Recommendations

1.  **Refactor Perlengkapan -> Integrasi:**
    *   Remove the `layanan-integrasi` path dependency from `perlengkapan/Cargo.toml`.
    *   Update `SimanIntegration` to use the `IntegrasiClient` (gRPC) defined in `grpc_clients.rs`.
    *   Update `main.rs` to initialize `IntegrasiClient` and pass it to `KebutuhanBmnService`.
2.  **Enable Integration:**
    *   Update `perlengkapan/src/main.rs` to properly initialize the SIMAN integration using the gRPC client.
