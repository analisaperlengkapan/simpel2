---
mode: agent
context: SIMPelv2 Perlengkapan Module Development
version: 0.4.0
framework: Leptos 0.8.x CSR SPA WASM
shared-components: v0.2.0 (40+ components, production-ready)
---

# Perlengkapan Microfrontend Development Guide

## Overview
Kembangkan `antarmuka/pembinaan/perlengkapan` sebagai perlengkapan-microfrontend menggunakan **Leptos 0.8.x CSR SPA WASM** dengan memanfaatkan **shared-microfrontend v0.2.0** yang sudah production-ready (40+ components, zero compilation errors, thread-safe).

## Technical Requirements

### Frontend Stack
- **Framework**: Leptos 0.8.x (CSR SPA)
- **Target**: wasm32-unknown-unknown
- **Build Tool**: Trunk 0.21.4+
- **Shared Components**: `antarmuka/shared/` v0.2.0
- **Bundle Size**: Max 2MB (enforced in CI)
- **Styling**: Kejaksaan RI branding, responsive, accessible

### Backend Integration
- **Service**: `layanan/pembinaan/perlengkapan` (Rust Axum)
- **Database**: PostgreSQL (tokio-postgres + deadpool-postgres)
- **Authentication**: JWT via `infra/authenc/`
- **API Gateway**: All requests via `infra/gerbang/`

## UI Specification

### Login Page
Tampilan awalnya merupakan halaman login dengan desain bagus menggunakan shared-microfrontend:
- **Header**: Logo + "SIMPEL KEJAKSAAN RI"
- **Subheader**: "Sistem Informasi Manajemen Perlengkapan"
- **NO input fields** untuk username/password
- **Single button**: "Login" (redirect to portal-microfrontend)
- **Flow**: Portal authentication → redirect back to perlengkapan dashboard

### Main Layout (Post-Login)
Nanti tampilannya:
- **Top Bar**: Foto pengguna (pojok kanan atas) → dropdown menu (Profil, Keluar)
- **Footer**: Pojok bawah
- **Sidebar**: Sisi kiri dengan menu:

- Dashbor
- Bank Aset
  - Daftar Aset
  - Peta Sebaran Aset
  - Cetak QR Code BMN
- Analisis Kebutuhan
  - Kebutuhan Pakaian Dinas
    - Pengajuan
    - Laporan
  - Kebutuhan BMN
    - Pengajuan
    - Analisis
    - Laporan
    - Roadmap Kebutuhan BMN
  - Standardisasi BMN
    - Pengajuan
    - Laporan
    - Standar Kejaksaan
- Pengadaan
  - Administrasi Pengadaan
  - Penyimpanan dan Distribusi
- Pengelolaan BMN
  - Pemakaian BMN
    - Pengajuan
    - Penyerahan
    - Perpanjangan
    - Pengembalian
    - Pencabutan
    - Laporan
  - Penerimaan Hibah
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Pengalihan BMN
    - Pengajuan
    - Serah Terima
    - Laporan
    - BMN marketplace
  - Penggunaan Rampasan
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Pemeliharaan
    - Pengajuan
    - Tindak Lanjut
    - Laporan
  - Penilaian
    - Pengajuan
    - Laporan
- Pengguna
  - Profil
  - Aktivitas
- Bantuan
  - Helpdesk
  - Panduan
  - FAQ
    ## Development Principles

### Code Quality Standards
- **Leptos 0.8.x Patterns**: Use modern signal patterns, updated callback methods, proper type annotations
- **Shared Components**: Maximize reuse from `antarmuka/shared/` v0.2.0 (Button, Input, Modal, Table, Navigation, etc.)
- **Thread Safety**: All components must be Send + Sync compatible
- **Zero Errors**: No compilation errors, warnings must be justified
- **Type Safety**: Leverage Rust's type system for compile-time guarantees

### Security Requirements
- **Zero-Trust**: No client-side secrets, all auth via JWT
- **MFA Integration**: Support TOTP multi-factor authentication
- **CSP Compliance**: Content Security Policy headers
- **Input Validation**: All user inputs sanitized
- **RBAC**: Role-Based Access Control for features

### Testing Requirements
- **Unit Tests**: `cargo test` for business logic
- **Integration Tests**: API endpoint testing with mock backend
- **WASM Tests**: `cargo test --target wasm32-unknown-unknown`
- **E2E Tests**: User flow testing (login → dashboard → features)
- **CI/CD**: All workflows must pass (ci-frontend.yml, ci-security.yml)

### Backend Development (layanan-perlengkapan)

**Technology Stack:**
- **Framework**: Rust Axum
- **Database**: PostgreSQL with:
  - `tokio-postgres` 0.7 (async driver)
  - `deadpool-postgres` 0.14 (connection pooling)
  - `refinery` 0.8 (migrations)
  - `sea-query` 0.31 (query builder)
- **Authentication**: JWT + MFA via `infra/authenc/`
- **Tracing**: `tracing-opentelemetry` 0.32+
- **Cryptography**: blake3, sha2 (NO sha1), ed25519-dalek

**Architecture:**
- RESTful APIs following OpenAPI 3.0 spec
- Multi-schema PostgreSQL (isolated from other services)
- Connection pooling for performance
- Structured logging with correlation IDs
- Prometheus metrics export
- Health check endpoints

## Implementation Workflow

1. **Planning**: Konfirmasi requirements, beri alternatif solusi jika ada ambiguitas
2. **Frontend Setup**:
   - Scaffold structure di `antarmuka/pembinaan/perlengkapan/`
   - Configure `Trunk.toml` dengan port yang sesuai
   - Import shared components v0.2.0
3. **Backend Setup**:
   - Scaffold structure di `layanan/pembinaan/perlengkapan/`
   - Setup PostgreSQL schema + refinery migrations
   - Configure deadpool connection pool
4. **Development Cycle**:
   - Implement feature
   - `cargo check --target wasm32-unknown-unknown` (frontend)
   - `cargo check` (backend)
   - `cargo clippy --all -- -D warnings` (NO warnings tolerated)
   - `cargo fmt --all` (code formatting)
   - `cargo test --all` (unit tests)
   - Manual testing: `trunk serve` + `cargo run --bin layanan-perlengkapan`
5. **Integration Testing**:
   - Test frontend ↔ backend connectivity
   - Test portal authentication flow
   - Test all CRUD operations
   - Validate error handling
6. **CI/CD Validation**:
   - Push to GitHub triggers workflows
   - All CI checks must pass (backend, frontend, security, quality)
   - WASM bundle size < 2MB
   - Code coverage > 70%
7. **Deployment Ready**:
   - Update documentation
   - Create migration guide (if needed)
   - Tag release following semver

## Best Practices & Next Practices

- **DRY**: Reuse shared components, don't duplicate logic
- **Separation of Concerns**: UI layer separate from business logic
- **API Gateway**: All external calls via `infra/gerbang/`
- **Observability**: Distributed tracing, structured logging
- **Performance**: Lazy loading, code splitting, WASM optimization
- **Accessibility**: WCAG 2.1 Level AA compliance
- **i18n Ready**: Prepare for internationalization
- **Progressive Enhancement**: Core functionality without JavaScript

## Portal Integration

- **Authentication Flow**:
  1. User clicks "Login" on perlengkapan
  2. Redirect to `portal-microfrontend` with return URL
  3. Portal handles authentication (username/password/MFA)
  4. Portal generates JWT token
  5. Redirect back to perlengkapan with token
  6. Perlengkapan validates token via `layanan-keamanan`
  7. Store token in secure storage (NOT localStorage)
- **Session Management**: 30-minute timeout, refresh token flow
- **Logout**: Clear token, redirect to portal

## Success Criteria

✅ Zero compilation errors
✅ Zero clippy warnings
✅ All tests passing (`cargo test --all`)
✅ WASM bundle < 2MB
✅ Portal authentication working
✅ All CRUD operations functional
✅ Responsive UI (mobile + desktop)
✅ Accessible (keyboard navigation, screen readers)
✅ CI/CD workflows passing
✅ Documentation complete

---

**Apabila ada yang belum jelas, konfirmasi terlebih dahulu dan beri opsi alternatif solusi.**
