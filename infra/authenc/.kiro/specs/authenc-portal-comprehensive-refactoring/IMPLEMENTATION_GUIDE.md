# Panduan Implementasi: Migrasi Multi-Crate Authenc

## 📋 Overview

Dokumen ini memberikan panduan step-by-step untuk melakukan migrasi dari struktur monolitik ke multi-crate architecture.

## 🚀 Quick Start

### Prerequisites

1. Backup kode saat ini:
```bash
cd infra/authenc
git checkout -b migration/multi-crate
git add .
git commit -m "Backup before multi-crate migration"
```

2. Pastikan semua tests pass:
```bash
cargo test --workspace
```

3. Pastikan tidak ada uncommitted changes:
```bash
git status
```

### Step 1: Dry Run Migration

Jalankan migration script dalam mode dry-run untuk melihat apa yang akan dilakukan:

```bash
chmod +x scripts/migrate_to_crates.sh
./scripts/migrate_to_crates.sh --dry-run
```

Review output dan pastikan semua file yang akan dimigrasi sudah benar.

### Step 2: Execute Migration

Jalankan migration script:

```bash
./scripts/migrate_to_crates.sh
```

Script ini akan:
- Migrate services ke crates/core/
- Migrate handlers ke crates/api/ dan crates/iam-api/
- Migrate middleware ke crates/api/
- Migrate MFA services ke crates/mfa/
- Migrate federation services ke crates/federation/

### Step 3: Update Imports

Jalankan import update script:

```bash
chmod +x scripts/update_imports.sh
./scripts/update_imports.sh
```

### Step 4: Update mod.rs Files

Update mod.rs di setiap crate untuk export modules yang baru:

#### crates/core/src/lib.rs
```rust
// Re-export services
pub mod services {
    pub mod captcha;
    pub mod audit_events;
    pub mod audit_integrity;
    pub mod audit_signature;
    pub mod enhanced_audit;
    pub mod event_publisher;
    pub mod event_retention;
    pub mod event_listeners;
    pub mod cache;
    pub mod cache_invalidation_listener;
    // ... other services
}

// Re-export config
pub mod config;

// Re-export init
pub mod init;

// Re-export SPI
pub mod spi;

// Re-export utils
pub mod utils;

// Re-export secreton client
pub mod secreton;
```

#### crates/api/src/lib.rs
```rust
// Re-export handlers
pub mod handlers {
    pub mod oauth2;
    pub mod oauth2_authz_code;
    pub mod oidc_provider;
    pub mod oidc_keys;
    pub mod oidc_sso;
    pub mod auth_helpers;
    pub mod token_exchange;
    pub mod webauthn;
    pub mod totp;
    pub mod totp_verify;
    pub mod session;
    pub mod health;
    pub mod metrics;
    pub mod jwks;
    // ... other handlers
}

// Re-export middleware
pub mod middleware;

// Re-export router
pub mod router;
```

#### crates/iam-api/src/lib.rs
```rust
// Re-export admin handlers
pub mod handlers {
    pub mod admin;
    pub mod client_registration;
    pub mod dcr_admin;
    pub mod client_policy;
    pub mod federation_admin;
    pub mod jit_admin_service;
    pub mod group;
    pub mod organization;
    pub mod satker;
    pub mod audit;
    pub mod spi_management;
    pub mod spi_federation;
    // ... other admin handlers
}

// Re-export admin router
pub mod router;
```

#### crates/mfa/src/lib.rs
```rust
// Re-export MFA services
pub mod fallback_client;
pub mod local_storage;
pub mod security_monitor;
pub mod performance_monitor;
pub mod audit_logger;
pub mod totp_store;

// Re-export main service
pub use authenc_core::services::mfa_service::MfaService;
pub use authenc_core::services::mfa_admin_service::MfaAdminService;
```

#### crates/federation/src/lib.rs
```rust
// Re-export federation services
pub mod manager;
pub mod provider;
pub mod advanced;

// Re-export SSO
pub mod sso;

// Re-export broker
pub mod broker;

// Re-export SAML
pub mod saml;

// Re-export social
pub mod social;
```

### Step 5: Update Cargo.toml Dependencies

Update dependencies di setiap crate untuk menggunakan workspace crates:

#### crates/api/Cargo.toml
```toml
[dependencies]
authenc-types = { path = "../types" }
authenc-core = { path = "../core" }
authenc-crypto = { path = "../crypto" }
authenc-storage = { path = "../storage" }

# Axum dependencies
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }

# ... other dependencies
```

#### crates/iam-api/Cargo.toml
```toml
[dependencies]
authenc-types = { path = "../types" }
authenc-core = { path = "../core" }
authenc-storage = { path = "../storage" }

# Axum dependencies
axum = { workspace = true }
tower = { workspace = true }

# ... other dependencies
```

#### crates/mfa/Cargo.toml
```toml
[dependencies]
authenc-types = { path = "../types" }
authenc-core = { path = "../core" }
authenc-crypto = { path = "../crypto" }
authenc-storage = { path = "../storage" }

# ... other dependencies
```

#### crates/federation/Cargo.toml
```toml
[dependencies]
authenc-types = { path = "../types" }
authenc-core = { path = "../core" }
authenc-storage = { path = "../storage" }

# ... other dependencies
```

### Step 6: Update Main Application

Update src/main.rs dan src/lib.rs untuk menggunakan crates baru:

#### src/lib.rs
```rust
// Re-export all crates
pub use authenc_types as types;
pub use authenc_core as core;
pub use authenc_crypto as crypto;
pub use authenc_storage as storage;
pub use authenc_api as api;
pub use authenc_iam_api as iam_api;
pub use authenc_grpc as grpc;
pub use authenc_mfa as mfa;
pub use authenc_federation as federation;
pub use authenc_webauthn as webauthn;

// Keep app.rs for backward compatibility
pub mod app;
pub mod error;
pub mod server;
```

#### src/app.rs
Update imports:
```rust
use authenc_core::services::*;
use authenc_storage::Database;
use authenc_crypto::*;
use authenc_types::*;
// ... etc
```

### Step 7: Build and Test

1. Build workspace:
```bash
cargo build --workspace
```

2. Fix compilation errors:
   - Update imports yang masih salah
   - Fix visibility issues (pub/pub(crate))
   - Fix module paths

3. Run tests:
```bash
cargo test --workspace
```

4. Fix test failures:
   - Update test imports
   - Fix test setup
   - Update mock data

### Step 8: Update Tests

Update test files untuk menggunakan crates baru:

```rust
// Before
use crate::services::captcha::*;

// After
use authenc_core::services::captcha::*;
```

### Step 9: Cleanup Old Code

Setelah semua tests pass, cleanup old code:

```bash
# Remove old src directories (keep only main.rs, lib.rs, app.rs, server.rs, error.rs)
rm -rf src/services/captcha
rm -rf src/services/audit_*
rm -rf src/services/event_*
rm -rf src/services/cache
rm -rf src/handlers
rm -rf src/middleware
# ... etc

# Keep only essential files in src/
# - main.rs
# - lib.rs
# - app.rs
# - server.rs
# - error.rs
# - app_logging.rs
```

### Step 10: Update Documentation

1. Update AGENTS.md:
```markdown
## Architecture

Authenc uses a multi-crate architecture:

- `authenc-types`: Shared types and traits
- `authenc-crypto`: Cryptographic operations
- `authenc-storage`: Database layer
- `authenc-core`: Business logic
- `authenc-api`: Public REST API
- `authenc-iam-api`: Admin REST API
- `authenc-grpc`: gRPC service
- `authenc-mfa`: MFA logic
- `authenc-federation`: SSO/Federation
- `authenc-webauthn`: WebAuthn/Passkeys
```

2. Update README.md dengan struktur baru

3. Update architecture diagrams

### Step 11: CI/CD Updates

Update CI/CD pipelines untuk multi-crate builds:

#### .github/workflows/ci.yml
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build workspace
        run: cargo build --workspace --all-features

      - name: Test workspace
        run: cargo test --workspace --all-features

      - name: Clippy
        run: cargo clippy --workspace --all-features -- -D warnings

      - name: Format check
        run: cargo fmt --all -- --check
```

### Step 12: Performance Testing

Run performance benchmarks untuk memastikan tidak ada regression:

```bash
cargo bench --workspace
```

Compare dengan baseline sebelum migrasi.

### Step 13: Final Verification

1. ✅ All tests pass
2. ✅ No clippy warnings
3. ✅ Code formatted
4. ✅ Documentation updated
5. ✅ CI/CD green
6. ✅ Performance benchmarks OK
7. ✅ Integration tests pass
8. ✅ Security tests pass

## 🔧 Troubleshooting

### Issue: Compilation Errors

**Problem**: `error[E0433]: failed to resolve: use of undeclared crate or module`

**Solution**:
1. Check Cargo.toml dependencies
2. Update imports to use correct crate names
3. Ensure mod.rs exports the module

### Issue: Circular Dependencies

**Problem**: `error: cyclic package dependency`

**Solution**:
1. Review dependency graph
2. Move shared code to authenc-types
3. Use trait objects for abstraction

### Issue: Test Failures

**Problem**: Tests fail after migration

**Solution**:
1. Update test imports
2. Update test setup code
3. Check for hardcoded paths
4. Update mock data

### Issue: Performance Regression

**Problem**: Slower build or runtime performance

**Solution**:
1. Enable LTO in release profile
2. Check for unnecessary dependencies
3. Profile critical paths
4. Optimize hot code

## 📚 Best Practices

### 1. Module Organization

```
crates/
├── types/          # Foundation - no dependencies
├── crypto/         # Depends on: types
├── storage/        # Depends on: types
├── core/           # Depends on: types, crypto, storage
├── api/            # Depends on: types, core
├── iam-api/        # Depends on: types, core
├── grpc/           # Depends on: types, core
├── mfa/            # Depends on: types, core, crypto
├── federation/     # Depends on: types, core
└── webauthn/       # Depends on: types, core, storage
```

### 2. Visibility Rules

- Use `pub` for public API
- Use `pub(crate)` for internal API
- Use `pub(super)` for parent module access
- Keep implementation details private

### 3. Error Handling

- Define errors in authenc-types
- Use thiserror for error types
- Implement From traits for conversions
- Provide context in error messages

### 4. Testing Strategy

- Unit tests in each crate
- Integration tests in workspace root
- Use test fixtures in authenc-types
- Mock external dependencies

### 5. Documentation

- Document all public APIs
- Provide examples in doc comments
- Keep README.md updated
- Maintain CHANGELOG.md

## 🎯 Success Metrics

- ✅ Build time: < 5 minutes (full workspace)
- ✅ Test time: < 10 minutes (all tests)
- ✅ Code coverage: > 80%
- ✅ Clippy warnings: 0
- ✅ Documentation coverage: > 90%
- ✅ Performance: No regression

## 📞 Support

Jika mengalami masalah:
1. Check troubleshooting section
2. Review migration plan
3. Check CI/CD logs
4. Ask team for help

---

**Document Version**: 1.0
**Last Updated**: 2026-02-19
**Status**: Ready for Use


---

## 📊 Phase 2 Completion: Storage Layer Integration Status

**Date**: 2026-02-03
**Task**: 3.7 - Storage Integration Verification
**Status**: ✅ **VERIFIED**

### Integration Verification Results

#### ✅ Dependency Structure
- **No circular dependencies** detected
- Clean dependency hierarchy: `types` → `storage` → `core`
- All Cargo.toml dependencies correctly configured

#### ✅ Trait Implementations
All storage traits properly implemented:

| Store Implementation | Trait | Status | Tests |
|---------------------|-------|--------|-------|
| `PostgresUserStore` | `UserStore` | ✅ Complete | 20 tests |
| `PostgresSessionStore` | `SessionStore` | ✅ Complete | 18 tests |
| `PostgresRealmStore` | `RealmStore` | ✅ Complete | 18 tests |
| `PostgresClientStore` | `ClientStore` | ✅ Complete | 18 tests |
| `PostgresCredentialStore` | `CredentialStore` | ✅ Complete | 20 tests |

**Total**: 94 unit tests passing

#### ✅ Compilation Status
```bash
# Individual crate checks
cargo check --package authenc-storage  # ✅ PASS
cargo check --package authenc-types    # ✅ PASS
cargo check --package authenc-core     # ✅ PASS
```

#### ⚠️ Pending Items for Phase 3

**Missing Store Implementations** (to be added in Task 5.x):
- `PostgresAuthorizationCodeStore` (for OAuth2 auth code flow)
- `PostgresRefreshTokenStore` (for OAuth2 refresh tokens)

**Integration Tests**:
- Current tests use mock implementations (appropriate for unit testing)
- Real database integration tests to be added in Phase 3
- Testcontainers setup needed for end-to-end testing

### Detailed Verification Report

See: [`STORAGE_INTEGRATION_VERIFICATION.md`](./STORAGE_INTEGRATION_VERIFICATION.md)

### Next Steps (Phase 3)

1. **Task 5.1-5.x**: Migrate authenc-core services
   - Update services to use `PostgresUserStore` instead of mocks
   - Add `PostgresAuthorizationCodeStore` implementation
   - Add `PostgresRefreshTokenStore` implementation

2. **Integration Testing**:
   - Create `tests/integration/` directory
   - Set up testcontainers for PostgreSQL
   - Add end-to-end tests: service → store → database

3. **Service Layer Updates**:
   - Replace mock stores in `UserManagementServiceImpl`
   - Replace mock stores in `AuthenticationServiceImpl`
   - Update dependency injection in `AppState`

### Phase 2 Progress: 50%

**Completed**:
- ✅ 3.1 - Create authenc-storage crate structure
- ✅ 3.2 - Migrate PostgresUserStore
- ✅ 3.3 - Migrate PostgresSessionStore
- ✅ 3.4 - Migrate PostgresRealmStore
- ✅ 3.5 - Migrate PostgresClientStore
- ✅ 3.6 - Migrate PostgresCredentialStore
- ✅ 3.7 - Verify storage integration

**Remaining**:
- ⏳ 3.8 - Update documentation
- ⏳ 3.9 - Final Phase 2 verification

---
