# Implementation Plan: Authenc & Portal IAM Comprehensive Refactoring

## Overview

This implementation plan transforms the monolithic Authenc service into a modern, enterprise-grade IAM platform with:
- **Multi-crate architecture**: 9 modular crates for maintainability
- **WebAuthn/Passkeys**: PRIMARY authentication method (MANDATORY)
- **Portal IAM rebuild**: Direct REST API integration, eliminating layanan-portal
- **FAPI compliance**: Optional phased implementation for financial-grade security

**Timeline**: 14 weeks (6 phases)
**Workflow**: Design-First (design.md ✅ → requirements.md ✅ → tasks.md)

---

## Tasks

### Phase 1: Preparation (Week 1-2)

- [x] 1. Set up multi-crate workspace structure
  - [x] 1.1 Create authenc-types crate with core traits and types
    - Create `layanan/authenc/crates/types/` directory structure
    - Define `UserId`, `RealmId`, `ClientId`, `SessionId` types
    - Define service traits: `UserStore`, `SessionStore`, `AuthenticationService`
    - Define `AuthResult` enum and error types
    - _Requirements: REQ-ARCH-001, REQ-ARCH-003_

  - [x] 1.2 Update root Cargo.toml with workspace members
    - Add all 9 authenc crates to workspace members
    - Define shared dependencies in `[workspace.dependencies]`
    - Configure workspace-level features
    - _Requirements: REQ-ARCH-001_

  - [x] 1.3 Create directory structure for all crates
    - Create `crates/core/`, `crates/crypto/`, `crates/storage/` directories
    - Create `crates/api/`, `crates/iam-api/`, `crates/grpc/` directories
    - Create `crates/mfa/`, `crates/federation/`, `crates/webauthn/` directories
    - Set up basic Cargo.toml for each crate
    - _Requirements: REQ-ARCH-001_

  - [x] 1.4 Set up CI/CD pipeline for multi-crate workspace
    - Update GitHub Actions workflow for workspace build
    - Add crate-specific test jobs
    - Configure cargo-deny for dependency auditing
    - Add clippy and rustfmt checks
    - _Requirements: REQ-MAINT-002, REQ-MAINT-003_

  - [x] 1.5 Document migration plan and architecture
    - Create MIGRATION.md with step-by-step guide
    - Document crate responsibilities and dependencies
    - Create architecture diagrams (mermaid)
    - Document rollback procedures
    - _Requirements: REQ-DOC-002, REQ-DOC-003_

- [x] 2. Checkpoint - Verify workspace structure
  - Ensure all crates compile with `cargo check --workspace`
  - Verify no circular dependencies
  - Ensure all tests pass, ask the user if questions arise.

---

### Phase 2: Core Migration (Week 3-6)

- [ ] 2.1 Pre-Migration Analysis - **PRIORITY: CRITICAL**
  - [x] 2.1.0 Create MIGRATION_ANALYSIS.md template
    - Create `layanan/authenc/MIGRATION_ANALYSIS.md` file
    - Add sections for each src/ directory (database/, crypto/, services/, handlers/, etc.)
    - Add template for tracking file migration status (✅ Migrated, 🔄 In Progress, ⚠️ Needs Modification, ❌ Do Not Migrate)
    - Add section for integration status tracking
    - Add section for end-to-end flow verification
    - Add section for deletion checklist
    - _Requirements: REQ-MIG-001_

  - [x] 2.1.1 Analyze src/ vs crates/ current state
    - Compare `src/database/` with `crates/storage/src/`
    - Compare `src/crypto/` with `crates/crypto/src/`
    - Compare `src/services/` with `crates/core/src/services/`
    - Identify files already migrated (avoid duplicate work)
    - Identify files that should NOT be migrated (keep in src/)
    - Document findings in MIGRATION_ANALYSIS.md
    - _Requirements: REQ-MIG-001, REQ-ARCH-001_

  - [x] 2.1.2 Assess migration feasibility for each module
    - For each file in `src/`, determine:
      - ✅ Ready to migrate (no dependencies on unmigrated code)
      - ⚠️ Needs modification first (circular dependencies, tight coupling)
      - ❌ Should NOT migrate (entry points, feature-gated code)
    - Create migration priority matrix
    - Identify integration points between crates
    - _Requirements: REQ-MIG-001, REQ-ARCH-002_

  - [x] 2.1.3 Map end-to-end integration flows
    - Document flow: Frontend (Portal) → authenc-api → authenc-core → authenc-storage → PostgreSQL
    - Document flow: Frontend (Portal) → authenc-iam-api → authenc-core → authenc-storage → PostgreSQL
    - Document flow: Backend Services → authenc-grpc → authenc-core → authenc-storage → PostgreSQL
    - Document flow: authenc-core → authenc-crypto → Secreton (gRPC)
    - Document flow: authenc-core → authenc-mfa → authenc-storage
    - Document flow: authenc-core → authenc-federation → External IdP
    - Document flow: authenc-api → authenc-webauthn → authenc-storage
    - Identify all crate boundaries and interfaces
    - _Requirements: REQ-ARCH-001, REQ-API-001, REQ-API-002, REQ-API-003_

  - [x] 2.1.4 Verify existing crate implementations
    - Check `crates/types/src/` - verify traits and domain types are complete
    - Check `crates/storage/src/stores/` - verify which stores are already implemented
    - Check `crates/core/src/services/` - verify which services are already implemented
    - Check `crates/api/src/handlers/` - verify which handlers are already implemented
    - Document what's done vs what remains
    - _Requirements: REQ-ARCH-001, REQ-ARCH-003, REQ-ARCH-004, REQ-ARCH-005_

  - [x] 2.1.5 Create integration test plan
    - Define integration test scenarios for each crate boundary
    - Define end-to-end test scenarios (Frontend → Backend → Database)
    - Define contract tests between crates (trait implementations)
    - Define performance test scenarios (<100ms p99 auth latency)
    - _Requirements: REQ-TEST-002, REQ-PERF-001_

- [x] 3. Migrate authenc-storage (Database Layer) - **PRIORITY: HIGH**
  - [x] 3.1 Migrate database connection and pooling
    - Move `src/database/mod.rs` → `crates/storage/src/database.rs`
    - Move `src/database/pool_config.rs` → `crates/storage/src/pool_config.rs`
    - Move `src/database/pool_monitor.rs` → `crates/storage/src/pool_monitor.rs`
    - Move `src/database/prepared_cache.rs` → `crates/storage/src/prepared_cache.rs`
    - Move `src/database/transaction.rs` → `crates/storage/src/transaction.rs`
    - **Status**: Foundation for all data access
    - _Requirements: REQ-ARCH-005, REQ-PERF-004_

  - [x] 3.2 Migrate database operations
    - Move `src/database/operations/` → `crates/storage/src/operations/`
    - Move `src/database/operations_legacy.rs` → `crates/storage/src/operations/legacy.rs`
    - Move `src/database/queries.rs` → `crates/storage/src/queries.rs`
    - Move `src/database/batch_operations.rs` → `crates/storage/src/batch.rs`
    - **Status**: Core CRUD operations
    - _Requirements: REQ-USER-001, REQ-USER-002, REQ-USER-003_

  - [x] 3.3 Migrate specialized stores
    - Move `src/database/audit_operations.rs` → `crates/storage/src/audit_operations.rs`
    - Move `src/database/captcha_operations.rs` → `crates/storage/src/captcha_operations.rs`
    - Move `src/database/satker_operations.rs` → `crates/storage/src/satker_operations.rs`
    - **Status**: Domain-specific operations
    - _Requirements: REQ-AUDIT-001, REQ-AUTH-003_

  - [x] 3.4 Migrate database migrations
    - Move `src/database/migrations.rs` → `crates/storage/src/migrations.rs`
    - Keep migration SQL files in `layanan/authenc/migrations/`
    - **Status**: Schema management
    - _Requirements: REQ-COMPAT-003_

  - [x] 3.5 Update storage crate exports
    - Update `crates/storage/src/lib.rs` with all modules
    - Re-export key types and traits
    - Add comprehensive documentation
    - **Status**: Public API definition
    - _Requirements: REQ-ARCH-005_

  - [x] 3.6 Write unit tests for storage layer
    - Test database connection pooling
    - Test prepared statement caching
    - Test transaction rollback
    - Test all store implementations
    - Target: >80% coverage
    - _Requirements: REQ-TEST-001, REQ-MAINT-001_

  - [x] 3.7 Verify storage integration with other crates
    - Test authenc-core → authenc-storage integration (service uses stores)
    - Test authenc-storage trait implementations match authenc-types traits
    - Verify no circular dependencies between storage and core
    - Run integration tests: create user → store in DB → retrieve user
    - _Requirements: REQ-ARCH-002, REQ-TEST-002_

  - [x] 3.8 Document what remains in src/database/
    - List files NOT migrated from src/database/ (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with storage migration status
    - _Requirements: REQ-MIG-001_

- [-] 4. Migrate authenc-crypto (Cryptography) - **PRIORITY: HIGH**
  - [x] 4.1 Migrate core cryptographic operations
    - Move `src/crypto/mod.rs` → `crates/crypto/src/lib.rs`
    - Move `src/crypto/enhanced.rs` → `crates/crypto/src/enhanced.rs`
    - Move `src/crypto/aes_gcm.rs` → `crates/crypto/src/aes_gcm.rs`
    - Move `src/crypto/shamir.rs` → `crates/crypto/src/shamir.rs`
    - **Status**: Core encryption/decryption
    - _Requirements: REQ-SEC-005, REQ-SEC-001_

  - [x] 4.2 Migrate key management
    - Move `src/crypto/ed25519_keys.rs` → `crates/crypto/src/keys/ed25519.rs`
    - Move `src/crypto/ecdsa_keys.rs` → `crates/crypto/src/keys/ecdsa.rs`
    - Move `src/crypto/ecdsa_p384_keys.rs` → `crates/crypto/src/keys/ecdsa_p384.rs`
    - Move `src/crypto/ecdsa_p521_keys.rs` → `crates/crypto/src/keys/ecdsa_p521.rs`
    - Move `src/crypto/eddsa_ed448_keys.rs` → `crates/crypto/src/keys/eddsa_ed448.rs`
    - Create `crates/crypto/src/keys/mod.rs` to organize key types
    - **Status**: JWT signing keys
    - _Requirements: REQ-TOKEN-001, REQ-SEC-002_

  - [x] 4.3 Migrate advanced cryptography
    - Move `src/crypto/pqc.rs` → `crates/crypto/src/pqc.rs` (Post-Quantum Crypto)
    - Move `src/crypto/mtls.rs` → `crates/crypto/src/mtls.rs`
    - Move `src/crypto/xmldsig.rs` → `crates/crypto/src/xmldsig.rs` (SAML signatures)
    - Move `src/crypto/dpop/` → `crates/crypto/src/dpop/` (DPoP for FAPI-2)
    - Move `src/crypto/sdjwt/` → `crates/crypto/src/sdjwt/` (Selective Disclosure JWT)
    - **Status**: Advanced security features
    - _Requirements: REQ-FAPI-003, REQ-SEC-004_

  - [x] 4.4 Migrate JWT utilities
    - Move `src/utils/jwt.rs` → `crates/crypto/src/jwt.rs`
    - Move `src/utils/jwt_key_manager.rs` → `crates/crypto/src/jwt_key_manager.rs`
    - Move `src/services/jwt_validator.rs` → `crates/crypto/src/jwt_validator.rs`
    - **Status**: JWT generation and validation
    - _Requirements: REQ-TOKEN-001, REQ-TOKEN-003_

  - [x] 4.5 Update crypto crate exports
    - Update `crates/crypto/src/lib.rs` with all modules
    - Re-export key cryptographic functions
    - Add comprehensive documentation
    - **Status**: Public API definition
    - _Requirements: REQ-ARCH-004_

  - [x] 4.6 Write unit tests for crypto operations
    - Test JWT encode/decode roundtrip
    - Test password hash/verify roundtrip
    - Test encryption/decryption roundtrip
    - Property-based tests for cryptographic invariants
    - _Requirements: REQ-TEST-001, REQ-TEST-003_

  - [x] 4.7 Verify crypto integration with other crates
    - Test authenc-core → authenc-crypto integration (services use crypto)
    - Test authenc-crypto → Secreton integration (key storage)
    - Verify JWT service works with authenc-api handlers
    - Run integration tests: generate JWT → validate JWT → extract claims
    - _Requirements: REQ-ARCH-002, REQ-TEST-002_

  - [x] 4.8 Document what remains in src/crypto/ and src/utils/
    - List files NOT migrated from src/crypto/ (if any)
    - List files NOT migrated from src/utils/ (jwt.rs, jwt_key_manager.rs)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with crypto migration status
    - _Requirements: REQ-MIG-001_

- [-] 5. Migrate authenc-core (Business Logic) - **PRIORITY: HIGH**
  - [x] 5.0 Migrate domain models to authenc-types - **PRIORITY: CRITICAL (DEPENDENCY FOR ALL OTHER TASKS)**
    - Move `src/models/user.rs` → `crates/types/src/domain/user.rs`
    - Move `src/models/realm.rs` → `crates/types/src/domain/realm.rs`
    - Move `src/models/role.rs` → `crates/types/src/domain/role.rs`
    - Move `src/models/permission.rs` → `crates/types/src/domain/permission.rs`
    - Move `src/models/consent.rs` → `crates/types/src/domain/consent.rs`
    - Move `src/models/social_account.rs` → `crates/types/src/domain/social_account.rs`
    - Move `src/models/session.rs` → `crates/types/src/domain/session.rs`
    - Move `src/models/group.rs` → `crates/types/src/domain/group.rs`
    - Move `src/models/organization.rs` → `crates/types/src/domain/organization.rs`
    - Move `src/models/satker.rs` → `crates/types/src/domain/satker.rs`
    - Create `crates/types/src/domain/mod.rs` to organize domain models
    - Update `crates/types/src/lib.rs` to export domain models
    - Re-enable operations in authenc-storage that depend on these models
    - **Status**: Core domain models (REQUIRED BY STORES AND OPERATIONS)
    - _Requirements: REQ-ARCH-003, REQ-USER-001_

  - [x] 5.1 Migrate service stores (Phase 1 - Core Stores)
    - Move `src/services/stores/user_store.rs` → `crates/core/src/stores/user_store.rs`
    - Move `src/services/stores/realm_store.rs` → `crates/core/src/stores/realm_store.rs`
    - Move `src/services/stores/role_store.rs` → `crates/core/src/stores/role_store.rs`
    - Move `src/services/stores/permission_store.rs` → `crates/core/src/stores/permission_store.rs`
    - Move `src/services/stores/consent_store.rs` → `crates/core/src/stores/consent_store.rs`
    - Move `src/services/stores/auth_flow_store.rs` → `crates/core/src/stores/auth_flow_store.rs`
    - Move `src/services/stores/social_account_store.rs` → `crates/core/src/stores/social_account_store.rs`
    - Create `crates/core/src/stores/mod.rs` to organize stores
    - **Status**: Core data stores (DEPENDS ON TASK 5.0)
    - _Requirements: REQ-ARCH-004, REQ-USER-001_

  - [x] 5.2 Migrate authentication services
    - Move `src/services/brute_force_protector.rs` → `crates/core/src/services/brute_force_protector.rs`
    - Move `src/services/anomaly_detector.rs` → `crates/core/src/services/anomaly_detector.rs`
    - Move `src/services/risk_engine.rs` → `crates/core/src/services/risk_engine.rs`
    - Move `src/services/auth_flow.rs` → `crates/core/src/services/auth_flow.rs`
    - Move `src/services/password_policy.rs` → `crates/core/src/services/password_policy.rs`
    - Create `crates/core/src/services/authentication.rs` (new unified service)
    - **Status**: Authentication logic
    - _Requirements: REQ-AUTH-001, REQ-AUTH-003_

  - [x] 5.3 Migrate session management
    - Move `src/services/session_store.rs` → `crates/core/src/services/session_store.rs`
    - Move `src/utils/sso_cookie.rs` → `crates/core/src/services/sso_cookie.rs`
    - **Status**: Session handling
    - _Requirements: REQ-AUTH-004_

  - [x] 5.4 Migrate realm and organization services
    - Move `src/services/realm.rs` → `crates/core/src/services/realm.rs`
    - Move `src/services/organization.rs` → `crates/core/src/services/organization.rs`
    - Move `src/services/satker_authorization.rs` → `crates/core/src/services/satker_authorization.rs`
    - **Status**: Multi-tenancy support
    - _Requirements: REQ-REALM-001, REQ-REALM-002_

  - [x] 5.5 Migrate OAuth2/OIDC services
    - Move `src/services/oidc_client_store.rs` → `crates/core/src/services/oidc_client_store.rs`
    - Move `src/services/oidc_code_store.rs` → `crates/core/src/services/oidc_code_store.rs`
    - Move `src/services/client_registration.rs` → `crates/core/src/services/client_registration.rs`
    - Move `src/services/client_registration_v2.rs` → `crates/core/src/services/client_registration_v2.rs`
    - Move `src/services/client_scope_service.rs` → `crates/core/src/services/client_scope_service.rs`
    - Move `src/services/protocol_mapper_service.rs` → `crates/core/src/services/protocol_mapper_service.rs`
    - Move `src/services/scope_store.rs` → `crates/core/src/services/scope_store.rs`
    - Move `src/services/service_account_store.rs` → `crates/core/src/services/service_account_store.rs`
    - Move `src/services/token_exchange.rs` → `crates/core/src/services/token_exchange.rs`
    - Move `src/services/device.rs` → `crates/core/src/services/device.rs` (Device Authorization Grant)
    - **Status**: OAuth2/OIDC implementation
    - _Requirements: REQ-OAUTH-001, REQ-OAUTH-002, REQ-OIDC-001_

  - [x] 5.6 Migrate UMA 2.0 services
    - Move `src/services/uma_policy_store.rs` → `crates/core/src/services/uma_policy_store.rs`
    - Move `src/services/resource_store.rs` → `crates/core/src/services/resource_store.rs`
    - Move `src/services/resource_server_store.rs` → `crates/core/src/services/resource_server_store.rs`
    - Move `src/services/permission_ticket_store.rs` → `crates/core/src/services/permission_ticket_store.rs`
    - Move `src/services/uma/` → `crates/core/src/services/uma/`
    - **Status**: User-Managed Access
    - _Requirements: REQ-ROLE-003_

  - [x] 5.7 Migrate audit and event services
    - Move `src/services/audit_events.rs` → `crates/core/src/services/audit_events.rs`
    - Move `src/services/audit_integrity.rs` → `crates/core/src/services/audit_integrity.rs`
    - Move `src/services/audit_signature.rs` → `crates/core/src/services/audit_signature.rs`
    - Move `src/services/enhanced_audit.rs` → `crates/core/src/services/enhanced_audit.rs`
    - Move `src/services/pg_audit_log_store.rs` → `crates/core/src/services/pg_audit_log_store.rs`
    - Move `src/services/audit_log_sink.rs` → `crates/core/src/services/audit_log_sink.rs`
    - Move `src/services/elasticsearch_audit_log_sink.rs` → `crates/core/src/services/elasticsearch_audit_log_sink.rs`
    - Move `src/services/kafka_audit_log_sink.rs` → `crates/core/src/services/kafka_audit_log_sink.rs`
    - Move `src/services/event_publisher.rs` → `crates/core/src/services/event_publisher.rs`
    - Move `src/services/event_retention.rs` → `crates/core/src/services/event_retention.rs`
    - Move `src/services/event_listeners.rs` → `crates/core/src/services/event_listeners.rs`
    - Move `src/services/events.rs` → `crates/core/src/services/events.rs`
    - Move `src/services/pg_event_store.rs` → `crates/core/src/services/pg_event_store.rs`
    - Move `src/services/kafka_event_listener.rs` → `crates/core/src/services/kafka_event_listener.rs`
    - **Status**: Audit trail and event system
    - _Requirements: REQ-AUDIT-001, REQ-AUDIT-002_

  - [x] 5.8 Migrate cache services
    - Move `src/services/cache/` → `crates/core/src/services/cache/`
    - Move `src/services/cache_invalidation_listener.rs` → `crates/core/src/services/cache_invalidation_listener.rs`
    - Move `src/utils/cache.rs` → `crates/core/src/services/cache_utils.rs`
    - **Status**: Caching layer
    - _Requirements: REQ-PERF-002_

  - [x] 5.9 Migrate advanced services
    - Move `src/services/client_policy/` → `crates/core/src/services/client_policy/`
    - Move `src/services/authorization/` → `crates/core/src/services/authorization/`
    - Move `src/services/zero_trust/` → `crates/core/src/services/zero_trust/`
    - Move `src/services/compliance/` → `crates/core/src/services/compliance/`
    - Move `src/services/fips/` → `crates/core/src/services/fips/`
    - Move `src/services/clustering/` → `crates/core/src/services/clustering/`
    - Move `src/services/observability/` → `crates/core/src/services/observability/`
    - Move `src/services/key_rotation.rs` → `crates/core/src/services/key_rotation.rs`
    - Move `src/services/database_optimizer.rs` → `crates/core/src/services/database_optimizer.rs`
    - Move `src/services/config_manager.rs` → `crates/core/src/services/config_manager.rs`
    - **Status**: Enterprise features
    - _Requirements: REQ-COMP-003, REQ-COMP-004_

  - [x] 5.10 Migrate PAR (Pushed Authorization Requests) for FAPI-2
    - Move `src/services/par/` → `crates/core/src/services/par/`
    - **Status**: FAPI-2 compliance (OPTIONAL)
    - _Requirements: REQ-FAPI-003_

  - [x] 5.11 Migrate configuration
    - Move `src/config/` → `crates/core/src/config/`
    - Move `src/app_init/` → `crates/core/src/init/`
    - **Status**: Configuration management
    - _Requirements: REQ-ARCH-004_

  - [x] 5.12 Migrate remaining OAuth2/OIDC models
    - Move `src/models/oauth2.rs` → `crates/types/src/domain/oauth2.rs`
    - Move `src/models/oidc_client.rs` → `crates/types/src/domain/oidc_client.rs`
    - Move `src/models/client_scope.rs` → `crates/types/src/domain/client_scope.rs`
    - Move `src/models/scope.rs` → `crates/types/src/domain/scope.rs`
    - Move `src/models/token.rs` → `crates/types/src/domain/token.rs`
    - Move `src/models/device.rs` → `crates/types/src/domain/device.rs`
    - Move `src/models/service_account.rs` → `crates/types/src/domain/service_account.rs`
    - Move `src/models/client_registration.rs` → `crates/types/src/domain/client_registration.rs`
    - Move `src/models/client_policy.rs` → `crates/types/src/domain/client_policy.rs`
    - Move `src/models/protocol_mapper.rs` → `crates/types/src/domain/protocol_mapper.rs`
    - Update imports in all crates
    - **Status**: OAuth2/OIDC domain models
    - _Requirements: REQ-ARCH-003, REQ-OAUTH-001_

  - [x] 5.13 Migrate UMA and advanced models
    - Move `src/models/resource.rs` → `crates/types/src/domain/resource.rs`
    - Move `src/models/resource_server.rs` → `crates/types/src/domain/resource_server.rs`
    - Move `src/models/permission_ticket.rs` → `crates/types/src/domain/permission_ticket.rs`
    - Move `src/models/audit.rs` → `crates/types/src/domain/audit.rs`
    - Move `src/models/audit_log.rs` → `crates/types/src/domain/audit_log.rs`
    - Move `src/models/events.rs` → `crates/types/src/domain/events.rs`
    - Move `src/models/webauthn.rs` → `crates/types/src/domain/webauthn.rs`
    - Move `src/models/saml.rs` → `crates/types/src/domain/saml.rs`
    - Move `src/models/dynamic_role.rs` → `crates/types/src/domain/dynamic_role.rs`
    - Move `src/models/model/` → `crates/types/src/domain/` (additional model files)
    - Update imports in all crates
    - **Status**: Advanced domain models
    - _Requirements: REQ-ARCH-003_

  - [x] 5.14 Migrate SPI (Service Provider Interface)
    - Move `src/spi/` → `crates/core/src/spi/`
    - **Status**: Plugin system
    - _Requirements: REQ-ARCH-004_

  - [x] 5.15 Update core crate exports
    - Update `crates/core/src/lib.rs` with all modules
    - Re-export key services and types
    - Add comprehensive documentation
    - **Status**: Public API definition
    - _Requirements: REQ-ARCH-004_

  - [x] 5.16 Write unit tests for core services
    - Test authentication flow (success, failure, MFA required)
    - Test user management operations
    - Test OAuth2 flows
    - Test brute force protection
    - Target: >80% coverage
    - _Requirements: REQ-TEST-001, REQ-MAINT-001_

  - [x] 5.17 Verify core integration with all dependencies
    - Test authenc-core → authenc-storage integration (all stores)
    - Test authenc-core → authenc-crypto integration (JWT, password hashing)
    - Test authenc-core → authenc-mfa integration (TOTP, backup codes)
    - Test authenc-core → authenc-federation integration (SSO, IdP)
    - Test authenc-core → authenc-webauthn integration (passkey auth)
    - Run end-to-end integration tests: login → authenticate → create session → generate JWT
    - _Requirements: REQ-ARCH-002, REQ-TEST-002_

  - [x] 5.18 Document what remains in src/services/, src/config/, src/models/, src/spi/
    - List files NOT migrated from src/services/ (if any)
    - List files NOT migrated from src/config/ (if any)
    - List files NOT migrated from src/models/ (if any)
    - List files NOT migrated from src/spi/ (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with core migration status
    - _Requirements: REQ-MIG-001_

- [x] 6. Migrate authenc-webauthn (MANDATORY - PRIMARY Authentication) - **PRIORITY: CRITICAL**
  - [x] 6.1 Migrate WebAuthn service
    - Move `src/services/webauthn.rs` → `crates/webauthn/src/service.rs`
    - Move `src/models/webauthn.rs` → `crates/webauthn/src/models.rs`
    - Move `src/handlers/webauthn.rs` → `crates/api/src/handlers/webauthn.rs` (API handlers)
    - **Status**: Core WebAuthn implementation
    - _Requirements: REQ-AUTH-005, REQ-WEBAUTHN-001, REQ-WEBAUTHN-002_

  - [x] 6.2 Implement credential store
    - Already exists in `crates/webauthn/src/credential_store.rs`
    - Verify PostgreSQL implementation
    - Add indexes for performance
    - **Status**: Credential storage
    - _Requirements: REQ-WEBAUTHN-003_

  - [x] 6.3 Write unit tests for WebAuthn service
    - Test passkey registration flow
    - Test passkey authentication flow
    - Test credential management operations
    - Test replay attack prevention
    - Test origin binding enforcement
    - Target: >90% coverage (critical security component)
    - _Requirements: REQ-TEST-001, REQ-MAINT-001_

  - [x] 6.4 Write property-based tests for WebAuthn
    - **Property 1: Counter monotonicity** - Credential counter always increases
    - **Validates: Requirements REQ-WEBAUTHN-004, REQ-SEC-012**
    - **Property 2: Origin binding** - Credentials only work for registered origin
    - **Validates: Requirements REQ-WEBAUTHN-008, REQ-SEC-011**
    - _Requirements: REQ-TEST-003_

  - [x] 6.5 Verify WebAuthn integration with other crates
    - Test authenc-webauthn → authenc-storage integration (credential store)
    - Test authenc-api → authenc-webauthn integration (handlers call service)
    - Test authenc-core → authenc-webauthn integration (authentication flow)
    - Run end-to-end integration tests: register passkey → authenticate with passkey → create session
    - Test with browser WebAuthn API (Chrome, Firefox, Safari)
    - _Requirements: REQ-ARCH-002, REQ-TEST-002, REQ-WEBAUTHN-001, REQ-WEBAUTHN-002_

  - [x] 6.6 Document what remains in src/services/webauthn.rs and src/models/webauthn.rs
    - List files NOT migrated (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with WebAuthn migration status
    - _Requirements: REQ-MIG-001_

- [x] 7. Checkpoint - Verify core implementation
  - Ensure all core services compile and pass tests
  - Verify WebAuthn integration works end-to-end
  - Run integration tests across all core crates (storage, crypto, core, webauthn)
  - Verify no files in src/database/, src/crypto/, src/services/ are still needed
  - Update MIGRATION_ANALYSIS.md with Phase 2 completion status
  - Ensure all tests pass, ask the user if questions arise.

**DEPENDENCY VERIFICATION**: This checkpoint MUST verify that:
- Task 3 (authenc-storage) is complete → Required by Task 5 (authenc-core)
- Task 4 (authenc-crypto) is complete → Required by Task 5 (authenc-core)
- Task 5 (authenc-core) is complete → Required by Task 6 (authenc-webauthn)
- Task 6 (authenc-webauthn) is complete → Required by Phase 3 (API migration)

---

### Phase 3: API Migration (Week 7-8)

- [x] 7.1 Pre-API Migration Analysis - **PRIORITY: CRITICAL**
  - [x] 7.1.1 Analyze src/handlers/ vs crates/api/src/handlers/
    - Compare existing handlers in src/ with crates/
    - Identify handlers already migrated
    - Identify handlers that need migration
    - Document handler dependencies on core services
    - _Requirements: REQ-MIG-001, REQ-API-001_

  - [x] 7.1.2 Analyze src/middleware/ vs crates/api/src/middleware/
    - Compare existing middleware in src/ with crates/
    - Identify middleware already migrated
    - Identify middleware that need migration
    - Document middleware dependencies
    - _Requirements: REQ-MIG-001, REQ-SEC-006_

  - [x] 7.1.3 Map Frontend → Backend API integration flows
    - Document flow: Portal Login Page → POST /api/v1/auth/login → authenc-api → authenc-core
    - Document flow: Portal Passkey Auth → POST /api/v1/auth/webauthn/authenticate → authenc-api → authenc-webauthn
    - Document flow: Portal User Profile → GET /api/v1/users/me → authenc-api → authenc-core
    - Document flow: Portal IAM Admin → POST /api/v1/iam/users → authenc-iam-api → authenc-core
    - Document flow: Other Microfrontends → GET /api/v1/auth/validate → authenc-api → authenc-core
    - Identify all REST API endpoints needed by frontend
    - _Requirements: REQ-API-001, REQ-API-002, REQ-PORTAL-ARCH-002_

  - [x] 7.1.4 Map Backend Services → gRPC integration flows
    - Document flow: layanan-perlengkapan → gRPC Authenticate() → authenc-grpc → authenc-core
    - Document flow: layanan-intel → gRPC ValidateToken() → authenc-grpc → authenc-core
    - Document flow: Other services → gRPC GetUser() → authenc-grpc → authenc-core
    - Identify all gRPC methods needed by backend services
    - _Requirements: REQ-API-003_

- [-] 8. Migrate authenc-api (Public REST API) - **PRIORITY: HIGH**

  **OPTIMIZATION NOTE**: Tasks 8.3 and 8.4 consolidated into 8.3 for better efficiency
  - [x] 8.1 Migrate authentication handlers
    - Move `src/handlers/auth_helpers.rs` → `crates/api/src/handlers/auth_helpers.rs`
    - Move `src/handlers/session.rs` → `crates/api/src/handlers/session.rs`
    - Move `src/handlers/totp.rs` → `crates/api/src/handlers/totp.rs`
    - Move `src/handlers/totp_verify.rs` → `crates/api/src/handlers/totp_verify.rs`
    - Move `src/handlers/webauthn.rs` → `crates/api/src/handlers/webauthn.rs` (already migrated in 6.1)
    - Create `crates/api/src/handlers/login.rs` (new unified login handler)
    - **Status**: Authentication endpoints
    - _Requirements: REQ-API-001, REQ-AUTH-001, REQ-WEBAUTHN-001_

  - [-] 8.2 Migrate Client Management (OAuth2 Clients, DCR, Protocol Mappers) - **PRIORITY: HIGH**
    - [x] 8.2.1 Phase 1: Types Migration (COMPLETE) ✅
      - Verify OAuth2Client, ClientRegistrationToken, InitialAccessToken exist in authenc-types
      - Verify ClientRegistrationPolicy, SoftwareStatementIssuer exist
      - Verify ProtocolMapper types exist
      - **Status**: All types already exist in `authenc-types::domain`
      - **Duration**: ~5 minutes (verification only)
      - _Requirements: REQ-ARCH-003, REQ-CLIENT-001_

    - [x] 8.2.2 Phase 2: Storage Operations Migration (COMPLETE) ✅
      - Fix imports in `crates/storage/src/operations/client_registration_ops.rs`
      - Fix imports in `crates/storage/src/operations/protocol_mappers_ops.rs`
      - Uncomment storage operations in `operations/mod.rs`
      - Uncomment imports in core services
      - **Status**: Storage operations enabled and compiling
      - **Duration**: ~30 minutes
      - _Requirements: REQ-ARCH-005, REQ-CLIENT-001_

    - [x] 8.2.3 Phase 3: Database API Fixes (COMPLETE) ✅
      - Fix 11 functions in `client_registration_ops.rs` to use `.try_into()` pattern
      - Change from `db.query_one::<T>()` to `db.query_one().await?.try_into()`
      - Change from `db.query::<T>()` to `db.query().await?` + `.into_iter().map(|r| r.try_into()).collect()`
      - Fix `AuthencError` usage from struct-style to function-style
      - Fix duplicate export in `operations/mod.rs`
      - **Status**: authenc-storage compiles successfully (0 errors)
      - **Duration**: ~20 minutes
      - _Requirements: REQ-ARCH-005, REQ-PERF-004_

    - [x] 8.2.4 Phase 4: API Handlers (IN PROGRESS) 🔄
      - Create `crates/api/src/handlers/client.rs`:
        - GET /api/v1/clients - List clients
        - POST /api/v1/clients - Create client
        - GET /api/v1/clients/{id} - Get client
        - PUT /api/v1/clients/{id} - Update client
        - DELETE /api/v1/clients/{id} - Delete client
      - Create `crates/api/src/handlers/client_registration.rs`:
        - POST /register - Register new client (RFC 7591)
        - GET /register/{client_id} - Get client configuration (RFC 7592)
        - PUT /register/{client_id} - Update client configuration (RFC 7592)
        - DELETE /register/{client_id} - Delete client (RFC 7592)
      - Add routes to `crates/api/src/lib.rs`
      - **Status**: Ready to start
      - **Duration**: ~1 hour
      - _Requirements: REQ-API-001, REQ-CLIENT-001, REQ-CLIENT-002_

    - [x] 8.2.5 Phase 5: Integration Testing (PENDING) ⏳
      - Test client CRUD operations
      - Test Dynamic Client Registration (DCR) flow
      - Test protocol mapper operations
      - Test client authentication
      - Test error handling
      - **Status**: Pending Phase 4 completion
      - **Duration**: ~30 minutes
      - _Requirements: REQ-TEST-002, REQ-CLIENT-001_

    - **Overall Progress**: 60% (3/5 phases complete)
    - **See**: `CLIENT_MANAGEMENT_MIGRATION_PROGRESS.md` for detailed tracking

  - [x] 8.3 Migrate remaining handlers (OAuth2/OIDC, Federation, Utilities) - **CONSOLIDATED**

    **8.3.1 OAuth2/OIDC Handlers**:
    - Move `src/handlers/oauth2.rs` → `crates/api/src/handlers/oauth2.rs`
    - Move `src/handlers/oauth2_authz_code.rs` → `crates/api/src/handlers/oauth2_authz_code.rs`
    - Move `src/handlers/oidc_provider.rs` → `crates/api/src/handlers/oidc_provider.rs`
    - Move `src/handlers/oidc_keys.rs` → `crates/api/src/handlers/oidc_keys.rs`
    - Move `src/handlers/oidc_sso.rs` → `crates/api/src/handlers/oidc_sso.rs`
    - Move `src/handlers/oidc_jwt.rs` → `crates/api/src/handlers/oidc_jwt.rs`
    - Move `src/handlers/oidc_ed25519.rs` → `crates/api/src/handlers/oidc_ed25519.rs`
    - Move `src/handlers/jwks.rs` → `crates/api/src/handlers/jwks.rs`
    - Move `src/handlers/jwt_ed25519.rs` → `crates/api/src/handlers/jwt_ed25519.rs`
    - Move `src/handlers/token_exchange.rs` → `crates/api/src/handlers/token_exchange.rs`
    - Move `src/handlers/device.rs` → `crates/api/src/handlers/device.rs`

    **8.3.2 Federation Handlers**:
    - Move `src/handlers/federated_auth.rs` → `crates/api/src/handlers/federated_auth.rs`
    - Move `src/handlers/federated_login.rs` → `crates/api/src/handlers/federated_login.rs`
    - Move `src/handlers/broker.rs` → `crates/api/src/handlers/broker.rs`
    - Move `src/handlers/social.rs` → `crates/api/src/handlers/social.rs`
    - Move `src/handlers/saml.rs` → `crates/api/src/handlers/saml.rs`
    - Move `src/handlers/sso.rs` → `crates/api/src/handlers/sso.rs`

    **8.3.3 Utility Handlers**:
    - Move `src/handlers/health.rs` → `crates/api/src/handlers/health.rs`
    - Move `src/handlers/metrics.rs` → `crates/api/src/handlers/metrics.rs`
    - Move `src/handlers/validation_helper.rs` → `crates/api/src/handlers/validation_helper.rs`
    - Move `src/handlers/consent_ui.rs` → `crates/api/src/handlers/consent_ui.rs`
    - Move `src/handlers/authorization.rs` → `crates/api/src/handlers/authorization.rs`

    **Status**: All remaining public API handlers
    **Duration**: ~2-3 hours (consolidated for efficiency)
    _Requirements: REQ-OAUTH-001, REQ-OAUTH-002, REQ-OIDC-001, REQ-FED-001, REQ-FED-002, REQ-API-001_

  - [x] 8.5 Migrate middleware
    - Move `src/middleware/auth_middleware.rs` → `crates/api/src/middleware/auth.rs`
    - Move `src/middleware/rate_limit.rs` → `crates/api/src/middleware/rate_limit.rs`
    - Move `src/middleware/adaptive_rate_limit.rs` → `crates/api/src/middleware/adaptive_rate_limit.rs`
    - Move `src/middleware/mfa_rate_limit.rs` → `crates/api/src/middleware/mfa_rate_limit.rs`
    - Move `src/middleware/csrf_protection.rs` → `crates/api/src/middleware/csrf.rs`
    - Move `src/middleware/input_validation.rs` → `crates/api/src/middleware/validation.rs`
    - Move `src/middleware/request_size_limit.rs` → `crates/api/src/middleware/size_limit.rs`
    - Move `src/middleware/compression.rs` → `crates/api/src/middleware/compression.rs`
    - Move `src/middleware/security_monitoring.rs` → `crates/api/src/middleware/security.rs`
    - Move `src/middleware/rbac.rs` → `crates/api/src/middleware/rbac.rs`
    - Move `src/middleware/mtls.rs` → `crates/api/src/middleware/mtls.rs`
    - Create `crates/api/src/middleware/mod.rs` to organize middleware
    - **Status**: API middleware
    - _Requirements: REQ-SEC-006, REQ-SEC-007_

  - [x] 8.4 Migrate routing and create ApiState - **CONSOLIDATED**

    **8.4.1 Routing Migration**:
    - Move `src/routes/config.rs` → `crates/api/src/routes.rs`
    - Move `src/axum_app/mod.rs` → `crates/api/src/app.rs`
    - Create unified router in `crates/api/src/router.rs`

    **8.4.2 ApiState Creation**:
    - Define `ApiState` struct with service dependencies
    - Add JWT service, auth service, user service, OAuth2 service
    - Add WebAuthn service reference
    - Create `crates/api/src/state.rs`

    **Status**: Route configuration and state management
    **Duration**: ~1 hour (consolidated for efficiency)
    _Requirements: REQ-API-001_

  - [x] 8.5 Integration testing and verification - **ENHANCED**

    **8.5.1 Unit Tests**:
    - Test authentication flow end-to-end
    - Test WebAuthn registration and authentication flows
    - Test OAuth2 authorization code flow
    - Test token validation
    - Test rate limiting

    **8.5.2 Integration Tests**:
    - Test authenc-api → authenc-core integration (all handlers call services)
    - Test authenc-api → authenc-webauthn integration (WebAuthn handlers)
    - Test authenc-api → authenc-crypto integration (JWT validation middleware)
    - Run end-to-end tests: Frontend (mock) → authenc-api → authenc-core → authenc-storage → PostgreSQL
    - Test CORS configuration works with Portal microfrontend origin

    **8.5.3 Crate Compilation Verification**:
    - Run `cargo check --package authenc-api` (must pass)
    - Run `cargo test --package authenc-api` (must pass)
    - Run `cargo clippy --package authenc-api` (must pass)

    **Status**: Comprehensive testing before moving to next phase
    **Duration**: ~1-2 hours
    _Requirements: REQ-ARCH-002, REQ-TEST-001, REQ-TEST-002, REQ-API-001_

  - [x] 8.6 Document migration status
    - List files NOT migrated from src/handlers/ (if any)
    - List files NOT migrated from src/middleware/ (if any)
    - List files NOT migrated from src/routes/ (if any)
    - List files NOT migrated from src/axum_app/ (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with API migration status
    - Mark authenc-api as COMPLETE in MIGRATION_ANALYSIS.md
    _Requirements: REQ-MIG-001_

- [x] 9. Migrate authenc-iam-api (Admin REST API) - **PRIORITY: MEDIUM**

  **OPTIMIZATION NOTE**: Streamlined into 4 focused tasks for better efficiency

  - [x] 9.1 Migrate all admin handlers - **CONSOLIDATED**
    - Move `src/handlers/admin.rs` → `crates/iam-api/src/handlers/admin.rs`
    - Move `src/handlers/client_registration.rs` → `crates/iam-api/src/handlers/client_registration.rs`
    - Move `src/handlers/dcr_admin.rs` → `crates/iam-api/src/handlers/dcr_admin.rs`
    - Move `src/handlers/client_policy.rs` → `crates/iam-api/src/handlers/client_policy.rs`
    - Move `src/handlers/federation_admin.rs` → `crates/iam-api/src/handlers/federation_admin.rs`
    - Move `src/handlers/jit_admin_service.rs` → `crates/iam-api/src/handlers/jit_admin_service.rs`
    - Move `src/handlers/group.rs` → `crates/iam-api/src/handlers/group.rs`
    - Move `src/handlers/organization.rs` → `crates/iam-api/src/handlers/organization.rs`
    - Move `src/handlers/satker.rs` → `crates/iam-api/src/handlers/satker.rs`
    - Move `src/handlers/audit.rs` → `crates/iam-api/src/handlers/audit.rs`
    - Move `src/handlers/spi_management.rs` → `crates/iam-api/src/handlers/spi_management.rs`
    - Move `src/handlers/spi_federation.rs` → `crates/iam-api/src/handlers/spi_federation.rs`
    - Move `src/handlers/uma.rs` → `crates/iam-api/src/handlers/uma.rs`
    - Move `src/handlers/zero_trust.rs` → `crates/iam-api/src/handlers/zero_trust.rs`
    - Move `src/handlers/oid4vc.rs` → `crates/iam-api/src/handlers/oid4vc.rs`
    **Status**: All IAM admin endpoints
    **Duration**: ~2-3 hours
    _Requirements: REQ-API-002, REQ-PORTAL-010 through REQ-PORTAL-016_

  - [x] 9.2 Create IamApiState and router - **CONSOLIDATED**

    **9.2.1 IamApiState Creation**:
    - Define `IamApiState` struct
    - Add user service, realm service, client service, role service
    - Add federation service reference
    - Create `crates/iam-api/src/state.rs`

    **9.2.2 IAM Router Creation**:
    - Create unified router in `crates/iam-api/src/router.rs`
    - Add admin authentication middleware
    - Add permission-based authorization

    **Status**: IAM API state management and routing
    **Duration**: ~1 hour
    _Requirements: REQ-API-002_

  - [x] 9.3 Integration testing and verification - **ENHANCED**

    **9.3.1 Unit Tests**:
    - Test user management operations
    - Test realm management operations
    - Test client management operations
    - Test role management operations
    - Test admin authorization

    **9.3.2 Integration Tests**:
    - Test authenc-iam-api → authenc-core integration (all handlers call services)
    - Test authenc-iam-api → authenc-storage integration (admin queries)
    - Run end-to-end tests: Portal IAM Admin (mock) → authenc-iam-api → authenc-core → authenc-storage → PostgreSQL
    - Test admin authentication middleware
    - Test permission-based authorization

    **9.3.3 Crate Compilation Verification**:
    - Run `cargo check --package authenc-iam-api` (must pass)
    - Run `cargo test --package authenc-iam-api` (must pass)
    - Run `cargo clippy --package authenc-iam-api` (must pass)

    **Status**: Comprehensive testing
    **Duration**: ~1-2 hours
    _Requirements: REQ-ARCH-002, REQ-TEST-001, REQ-TEST-002, REQ-API-002_

  - [x] 9.4 Document migration status
    - List admin handler files NOT migrated (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with IAM API migration status
    - Mark authenc-iam-api as COMPLETE in MIGRATION_ANALYSIS.md
    _Requirements: REQ-MIG-001_

- [x] 10. Migrate authenc-grpc (Service-to-Service gRPC) - **PRIORITY: HIGH**

  **OPTIMIZATION NOTE**: Streamlined into 3 focused tasks for better efficiency

  - [x] 10.1 Migrate all gRPC components - **CONSOLIDATED**

    **10.1.1 gRPC Service Implementation**:
    - Move `src/grpc/authenc_service.rs` → `crates/grpc/src/authenc_service.rs`
    - Move `src/grpc/captcha_service.rs` → `crates/grpc/src/captcha_service.rs`
    - Move `src/grpc/batch_operations.rs` → `crates/grpc/src/batch_operations.rs`
    - Move `src/grpc/health.rs` → `crates/grpc/src/health.rs`
    - Move `src/grpc/interceptors.rs` → `crates/grpc/src/interceptors.rs`
    - Move `src/grpc/mod.rs` → `crates/grpc/src/lib.rs`

    **10.1.2 Proto Files Verification**:
    - Ensure `proto/` directory is accessible
    - Verify build.rs generates code correctly
    - Update proto paths if needed

    **Status**: Complete gRPC service implementation
    **Duration**: ~1-2 hours
    _Requirements: REQ-API-003_

  - [x] 10.2 Integration testing and verification - **ENHANCED**

    **10.2.1 Unit Tests**:
    - Test authentication RPC
    - Test token validation RPC
    - Test mTLS connection
    - Test error handling

    **10.2.2 Integration Tests**:
    - Test authenc-grpc → authenc-core integration (all RPCs call services)
    - Test authenc-grpc → authenc-crypto integration (JWT validation)
    - Run end-to-end tests: Backend Service (mock) → authenc-grpc → authenc-core → authenc-storage → PostgreSQL
    - Test mTLS client certificate validation
    - Test gRPC interceptors (auth, logging)

    **10.2.3 Crate Compilation Verification**:
    - Run `cargo check --package authenc-grpc` (must pass)
    - Run `cargo test --package authenc-grpc` (must pass)
    - Run `cargo clippy --package authenc-grpc` (must pass)

    **Status**: Comprehensive testing
    **Duration**: ~1 hour
    _Requirements: REQ-ARCH-002, REQ-TEST-001, REQ-TEST-002, REQ-API-003_

  - [x] 10.3 Document migration status
    - List gRPC files NOT migrated (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with gRPC migration status
    - Mark authenc-grpc as COMPLETE in MIGRATION_ANALYSIS.md
    _Requirements: REQ-MIG-001_

- [x] 11. Checkpoint - Verify API implementation - **ENHANCED VERIFICATION**

  **11.1 Compilation Verification**:
  - Run `cargo check --workspace` (must pass)
  - Run `cargo build --workspace` (must pass)
  - Verify all API crates compile without errors

  **11.2 Unit Test Verification**:
  - Run `cargo test --package authenc-api` (must pass)
  - Run `cargo test --package authenc-iam-api` (must pass)
  - Run `cargo test --package authenc-grpc` (must pass)
  - Target: >80% test coverage for all API crates

  **11.3 Integration Test Verification**:
  - Test all API endpoints are functional
  - Verify WebAuthn endpoints work with browser WebAuthn API
  - Test gRPC service with mTLS
  - Run end-to-end integration tests: Frontend → authenc-api → authenc-core → authenc-storage → PostgreSQL
  - Run end-to-end integration tests: Backend Services → authenc-grpc → authenc-core → authenc-storage → PostgreSQL

  **11.4 Crate Boundary Verification**:
  - Verify authenc-api → authenc-core integration (all handlers)
  - Verify authenc-api → authenc-webauthn integration (WebAuthn handlers)
  - Verify authenc-api → authenc-crypto integration (JWT middleware)
  - Verify authenc-iam-api → authenc-core integration (admin handlers)
  - Verify authenc-grpc → authenc-core integration (all RPCs)
  - Verify authenc-grpc → authenc-crypto integration (JWT validation)

  **11.5 Migration Status Verification**:
  - Verify no files in src/handlers/ are still needed
  - Verify no files in src/middleware/ are still needed
  - Verify no files in src/grpc/ are still needed
  - Update MIGRATION_ANALYSIS.md with Phase 3 completion status
  - Mark Phase 3 as COMPLETE in MIGRATION_ANALYSIS.md

  **11.6 Rollback Point**:
  - Document current state for potential rollback
  - Ensure all changes are committed to version control
  - Tag release as `phase-3-complete`

  **Status**: Critical checkpoint before Phase 4
  **Duration**: ~1-2 hours
  **Ask user if questions arise**: Ensure all tests pass before proceeding

**DEPENDENCY VERIFICATION**: This checkpoint MUST verify that:
- Task 8 (authenc-api) is complete → Required by Task 15 (Portal rebuild)
- Task 9 (authenc-iam-api) is complete → Required by Task 15 (Portal rebuild)
- Task 10 (authenc-grpc) is complete → Required for backend service integration
- All Phase 2 tasks (storage, crypto, core, webauthn) are complete → Required by all API tasks

---

### Phase 4: Feature Migration (Week 9-10)

**OPTIMIZATION NOTE**: Phase 4 streamlined with better task consolidation and parallel execution opportunities

- [x] 11.1 Pre-Feature Migration Analysis - **PRIORITY: CRITICAL**
  - [x] 11.1.1 Analyze src/services/ (MFA and Federation modules)
    - Identify MFA-related services in src/services/
    - Identify Federation-related services in src/services/
    - Compare with existing implementations in crates/mfa/ and crates/federation/
    - Document dependencies between MFA/Federation and core services
    - Create migration priority matrix
    - **Duration**: ~30 minutes
    - _Requirements: REQ-MIG-001_

  - [x] 11.1.2 Map MFA integration flows
    - Document flow: User enables TOTP → authenc-api → authenc-mfa → authenc-storage (store secret in Secreton)
    - Document flow: User verifies TOTP → authenc-api → authenc-mfa → authenc-crypto (verify code)
    - Document flow: User uses backup code → authenc-api → authenc-mfa → authenc-storage
    - Identify all MFA integration points with core authentication
    - **Duration**: ~20 minutes
    - _Requirements: REQ-MFA-001, REQ-MFA-002, REQ-MFA-003_

  - [x] 11.1.3 Map Federation integration flows
    - Document flow: User clicks SSO button → authenc-api → authenc-federation → External IdP
    - Document flow: IdP callback → authenc-api → authenc-federation → authenc-core (create/link user)
    - Document flow: SAML authentication → authenc-api → authenc-federation → authenc-core
    - Identify all Federation integration points with core authentication
    - **Duration**: ~20 minutes
    - _Requirements: REQ-FED-001, REQ-FED-002, REQ-FED-003_

- [x] 12. Migrate authenc-mfa (Multi-Factor Authentication) - **PRIORITY: HIGH**

  **PARALLEL EXECUTION**: Can run in parallel with Task 13 (Federation)

  - [x] 12.1 Migrate all MFA components - **CONSOLIDATED**

    **12.1.1 MFA Services**:
    - Move `src/services/mfa_service.rs` → `crates/mfa/src/service.rs`
    - Move `src/services/mfa_admin_service.rs` → `crates/mfa/src/admin_service.rs`
    - Move `src/services/totp_store.rs` → `crates/mfa/src/totp_store.rs`
    - Move `src/services/mfa_fallback_client.rs` → `crates/mfa/src/fallback_client.rs`
    - Move `src/services/mfa_local_storage.rs` → `crates/mfa/src/local_storage.rs`
    - Move `src/services/mfa_security_monitor.rs` → `crates/mfa/src/security_monitor.rs`
    - Move `src/services/mfa_performance_monitor.rs` → `crates/mfa/src/performance_monitor.rs`
    - Move `src/services/mfa_audit_logger.rs` → `crates/mfa/src/audit_logger.rs`

    **12.1.2 MFA Middleware**:
    - Move `src/middleware/mfa_rate_limit.rs` → `crates/mfa/src/middleware/rate_limit.rs`
    - Move `src/middleware/mfa_performance_middleware.rs` → `crates/mfa/src/middleware/performance.rs`

    **12.1.3 Update MFA Crate Exports**:
    - Update `crates/mfa/src/lib.rs` with all modules
    - Re-export key MFA services
    - Add comprehensive documentation

    **Status**: Complete MFA core services and middleware
    **Duration**: ~2-3 hours
    _Requirements: REQ-MFA-001, REQ-MFA-002, REQ-MFA-003, REQ-AUTH-002, REQ-ARCH-001_

  - [x] 12.2 Testing and verification - **ENHANCED**

    **12.2.1 Unit Tests**:
    - Test TOTP generation and verification
    - Test backup code generation and validation
    - Test MFA policy enforcement
    - Target: >80% test coverage

    **12.2.2 Integration Tests**:
    - Test authenc-mfa → authenc-storage integration (TOTP store)
    - Test authenc-mfa → authenc-crypto integration (TOTP generation)
    - Test authenc-mfa → Secreton integration (secret storage)
    - Test authenc-core → authenc-mfa integration (authentication flow with MFA)
    - Test authenc-api → authenc-mfa integration (MFA endpoints)
    - Run end-to-end tests: Enable TOTP → Store secret → Verify code → Authenticate with MFA

    **12.2.3 Crate Compilation Verification**:
    - Run `cargo check --package authenc-mfa` (must pass)
    - Run `cargo test --package authenc-mfa` (must pass)
    - Run `cargo clippy --package authenc-mfa` (must pass)

    **Status**: Comprehensive testing
    **Duration**: ~1-2 hours
    _Requirements: REQ-ARCH-002, REQ-TEST-001, REQ-TEST-002, REQ-MFA-001, REQ-MFA-002_

  - [x] 12.3 Document migration status
    - List MFA service files NOT migrated (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with MFA migration status
    - Mark authenc-mfa as COMPLETE in MIGRATION_ANALYSIS.md
    _Requirements: REQ-MIG-001_

- [x] 13. Migrate authenc-federation (SSO/Federation) - **PRIORITY: MEDIUM**

  **PARALLEL EXECUTION**: Can run in parallel with Task 12 (MFA)

  - [x] 13.1 Migrate all Federation components - **CONSOLIDATED**

    **13.1.1 Federation Core Services**:
    - Move `src/services/federation_manager.rs` → `crates/federation/src/manager.rs`
    - Move `src/services/federation_provider.rs` → `crates/federation/src/provider.rs`
    - Move `src/services/advanced_federation.rs` → `crates/federation/src/advanced.rs`
    - Move `src/services/federation/` → `crates/federation/src/providers/`

    **13.1.2 SSO Services**:
    - Move `src/services/sso/` → `crates/federation/src/sso/`

    **13.1.3 Identity Broker**:
    - Move `src/services/broker/` → `crates/federation/src/broker/`

    **13.1.4 SAML Services**:
    - Move `src/services/saml.rs` → `crates/federation/src/saml/service.rs`
    - Move `src/services/saml_signature.rs` → `crates/federation/src/saml/signature.rs`

    **13.1.5 Social Login**:
    - Move `src/services/social/` → `crates/federation/src/social/`

    **13.1.6 User Synchronization**:
    - Move `src/services/user_sync_service.rs` → `crates/federation/src/user_sync.rs`
    - Move `src/services/mysimkari_sync.rs` → `crates/federation/src/mysimkari_sync.rs`

    **13.1.7 Update Federation Crate Exports**:
    - Update `crates/federation/src/lib.rs` with all modules
    - Re-export key federation services
    - Add comprehensive documentation

    **Status**: Complete Federation orchestration, SSO, SAML, social login
    **Duration**: ~3-4 hours
    _Requirements: REQ-FED-001, REQ-FED-002, REQ-FED-003, REQ-ARCH-001_

  - [x] 13.2 Testing and verification - **ENHANCED**

    **13.2.1 Unit Tests**:
    - Test OIDC provider integration
    - Test SAML provider integration
    - Test user account linking
    - Target: >80% test coverage

    **13.2.2 Integration Tests**:
    - Test authenc-federation → authenc-core integration (user provisioning)
    - Test authenc-federation → authenc-storage integration (IdP configuration)
    - Test authenc-federation → External IdP integration (OIDC, SAML)
    - Test authenc-api → authenc-federation integration (SSO endpoints)
    - Run end-to-end tests: Click SSO button → Redirect to IdP → Callback → Create/link user → Authenticate

    **13.2.3 Crate Compilation Verification**:
    - Run `cargo check --package authenc-federation` (must pass)
    - Run `cargo test --package authenc-federation` (must pass)
    - Run `cargo clippy --package authenc-federation` (must pass)

    **Status**: Comprehensive testing
    **Duration**: ~1-2 hours
    _Requirements: REQ-ARCH-002, REQ-TEST-001, REQ-TEST-002, REQ-FED-001, REQ-FED-002_

  - [x] 13.3 Document migration status
    - List Federation service files NOT migrated (if any)
    - Document reason for keeping each file in src/
    - Update MIGRATION_ANALYSIS.md with Federation migration status
    - Mark authenc-federation as COMPLETE in MIGRATION_ANALYSIS.md
    _Requirements: REQ-MIG-001_

- [x] 14. Checkpoint - Verify feature implementation - **ENHANCED VERIFICATION**

  **14.1 Compilation Verification**:
  - Run `cargo check --workspace` (must pass)
  - Run `cargo build --workspace` (must pass)
  - Verify all feature crates compile without errors

  **14.2 Unit Test Verification**:
  - Run `cargo test --package authenc-mfa` (must pass)
  - Run `cargo test --package authenc-federation` (must pass)
  - Target: >80% test coverage for all feature crates

  **14.3 Integration Test Verification**:
  - Ensure MFA works end-to-end
  - Test federation with external IdP
  - Run integration tests across all feature crates (mfa, federation)

  **14.4 Crate Boundary Verification**:
  - Verify authenc-mfa → authenc-storage integration
  - Verify authenc-mfa → authenc-crypto integration
  - Verify authenc-mfa → Secreton integration
  - Verify authenc-federation → authenc-core integration
  - Verify authenc-federation → authenc-storage integration
  - Verify authenc-api → authenc-mfa integration
  - Verify authenc-api → authenc-federation integration

  **14.5 Migration Status Verification**:
  - Verify no MFA/Federation files in src/services/ are still needed
  - Update MIGRATION_ANALYSIS.md with Phase 4 completion status
  - Mark Phase 4 as COMPLETE in MIGRATION_ANALYSIS.md

  **14.6 Rollback Point**:
  - Document current state for potential rollback
  - Ensure all changes are committed to version control
  - Tag release as `phase-4-complete`

  **Status**: Critical checkpoint before Phase 5
  **Duration**: ~1-2 hours
  **Ask user if questions arise**: Ensure all tests pass before proceeding

**DEPENDENCY VERIFICATION**: This checkpoint MUST verify that:
- Task 12 (authenc-mfa) is complete → Required by Task 15 (Portal MFA pages)
- Task 13 (authenc-federation) is complete → Required by Task 15 (Portal SSO pages)
- All Phase 3 tasks (API) are complete → Required for MFA/Federation endpoints

---

### Phase 5: Portal Refactoring (Week 11-12)

- [ ] 15. Rebuild Portal IAM Microfrontend (Leptos 0.8.x)
  - [ ] 15.1 Set up Portal project structure
    - Create `antarmuka/portal/` directory structure
    - Set up Cargo.toml with Leptos 0.8.x dependencies
    - Configure Trunk for WASM build
    - Set up Tailwind CSS
    - _Requirements: REQ-PORTAL-ARCH-001, REQ-PORTAL-ARCH-005_

  - [ ] 15.2 Implement AppState and context providers
    - Create `AppState` with RwSignal for user, auth_token
    - Add derived signals for is_authenticated, is_admin
    - Implement login/logout methods
    - Add localStorage integration for token persistence
    - _Requirements: REQ-PORTAL-ARCH-003_

  - [ ] 15.3 Implement AuthencApiClient
    - Create API client with grouped methods by resource
    - Add authentication methods (login, logout, refresh)
    - Add WebAuthn methods (register, authenticate, list, delete)
    - Add user profile methods
    - Add IAM admin methods (users, realms, clients, roles)
    - Use type-safe request/response types
    - _Requirements: REQ-PORTAL-ARCH-002, REQ-PORTAL-ARCH-004_

  - [ ] 15.4 Implement authentication pages
    - Create LoginPage with passkey and password options
    - Passkey authentication as PRIMARY method (default tab)
    - Password authentication as SECONDARY method
    - Add MFA verification page
    - Add password reset page
    - Add SSO provider buttons
    - _Requirements: REQ-PORTAL-001, REQ-PORTAL-002, REQ-PORTAL-003, AC-PORTAL-001_

  - [ ] 15.5 Implement user dashboard
    - Create DashboardPage with profile overview
    - Add recent activity widget
    - Add active sessions widget
    - Add security alerts widget
    - Add quick actions (change password, setup MFA, manage passkeys)
    - _Requirements: REQ-PORTAL-004_

  - [ ] 15.6 Implement user self-service pages
    - Create ProfileManagementPage for profile editing
    - Create PasswordManagementPage for password changes
    - Create MfaManagementPage for TOTP setup and backup codes
    - Create SessionManagementPage for active sessions
    - _Requirements: REQ-PORTAL-005, REQ-PORTAL-006, REQ-PORTAL-007, REQ-PORTAL-008_

  - [ ] 15.7 Implement passkey management page (MANDATORY)
    - Create PasskeysManagementPage component
    - Add passkey list with metadata (nickname, created date, last used)
    - Add "Add Passkey" button with registration flow
    - Implement browser WebAuthn API wrapper (create_credential, get_credential)
    - Add passkey deletion with confirmation
    - Add passkey nickname editing
    - Show passkey icons (platform authenticator vs security key)
    - _Requirements: REQ-PORTAL-009, REQ-WEBAUTHN-003, AC-PASSKEY-004, AC-PASSKEY-005_

  - [ ] 15.8 Implement IAM administration pages (admin-only)
    - Create UsersManagementPage with user list, search, filters
    - Create CreateUserModal and EditUserModal
    - Create RealmsManagementPage with realm CRUD
    - Create ClientsManagementPage with OAuth2 client management
    - Create RolesManagementPage with role and permission management
    - Create FederationManagementPage with IdP configuration
    - Create AuditLogsPage with log viewer and export
    - Create SystemConfigPage for system settings
    - _Requirements: REQ-PORTAL-010 through REQ-PORTAL-016_

  - [ ] 15.9 Implement routing and navigation
    - Set up Leptos router with routes
    - Add ProtectedRoute component for authenticated pages
    - Add AdminRoute component for admin-only pages
    - Add navigation menu with role-based visibility
    - _Requirements: REQ-PORTAL-ARCH-001_

  - [ ] 15.10 Implement responsive design and accessibility
    - Add mobile-first responsive layouts
    - Implement keyboard navigation
    - Add ARIA labels and roles
    - Test with screen readers
    - _Requirements: REQ-USE-001, REQ-USE-002, AC-PORTAL-002, AC-PORTAL-003_

  - [ ] 15.11 Write integration tests for Portal
    - Test login flow (passkey and password)
    - Test passkey registration and management
    - Test user self-service features
    - Test IAM admin features
    - Test responsive design
    - _Requirements: REQ-TEST-002_

- [ ] 16. Eliminate layanan-portal service
  - [ ] 16.1 Verify all Portal functionality migrated to direct API calls
    - Audit all Portal API calls to ensure they use Authenc REST API
    - Verify no gRPC calls from Portal
    - Verify JWT token management in localStorage
    - _Requirements: REQ-ELIM-001, REQ-ELIM-002_

  - [ ] 16.2 Update other microfrontends to use Authenc API directly
    - Update Perlengkapan microfrontend
    - Update Intel microfrontend
    - Update other microfrontends
    - Remove layanan-portal dependencies
    - _Requirements: REQ-ELIM-001_

  - [ ] 16.3 Decommission layanan-portal service
    - Remove layanan-portal from Kubernetes manifests
    - Remove layanan-portal from CI/CD pipeline
    - Archive layanan-portal codebase
    - Update documentation
    - _Requirements: REQ-ELIM-001_

- [ ] 17. Checkpoint - Verify Portal refactoring
  - Test Portal end-to-end with all features
  - Verify passkey authentication works across browsers
  - Verify layanan-portal is fully eliminated
  - Run end-to-end integration tests: Portal (real) → authenc-api/iam-api → authenc-core → authenc-storage → PostgreSQL
  - Test all user flows: Login (passkey/password) → Dashboard → Profile → MFA → Passkeys → Admin (if admin)
  - Verify responsive design on mobile, tablet, desktop
  - Verify accessibility with screen readers
  - Update MIGRATION_ANALYSIS.md with Phase 5 completion status
  - Ensure all tests pass, ask the user if questions arise.

**DEPENDENCY VERIFICATION**: This checkpoint MUST verify that:
- Task 15 (Portal rebuild) is complete → All Portal features implemented
- Task 16 (Eliminate layanan-portal) is complete → No more portal service layer
- All Phase 4 tasks (MFA, Federation) are complete → Required for Portal MFA/SSO features
- All Phase 3 tasks (API) are complete → Required for Portal API integration

---

### Phase 6: Cleanup and Optimization (Week 13-14)

- [ ] 17.1 Final Migration Analysis - **PRIORITY: CRITICAL**
  - [ ] 17.1.1 Comprehensive src/ directory audit
    - List ALL remaining files in src/ directory
    - For each file, determine:
      - ✅ Successfully migrated to crates/ (can be deleted)
      - ⚠️ Partially migrated (needs completion)
      - ❌ Must keep (entry points, feature-gated code)
    - Create final deletion checklist
    - _Requirements: REQ-MIG-001_

  - [ ] 17.1.2 Verify all crate integrations are complete
    - Verify authenc-types exports all needed traits and types
    - Verify authenc-storage implements all store traits
    - Verify authenc-crypto provides all cryptographic operations
    - Verify authenc-core uses all dependencies correctly
    - Verify authenc-api/iam-api/grpc expose all needed endpoints
    - Verify authenc-mfa/federation/webauthn integrate correctly
    - Run full integration test suite across all crates
    - _Requirements: REQ-ARCH-001, REQ-ARCH-002_

  - [ ] 17.1.3 Verify end-to-end flows are complete
    - Test: Portal Login (passkey) → authenc-api → authenc-webauthn → authenc-core → authenc-storage → PostgreSQL
    - Test: Portal Login (password) → authenc-api → authenc-core → authenc-crypto → authenc-storage → PostgreSQL
    - Test: Portal MFA → authenc-api → authenc-mfa → authenc-crypto → Secreton
    - Test: Portal IAM Admin → authenc-iam-api → authenc-core → authenc-storage → PostgreSQL
    - Test: Backend Service → authenc-grpc → authenc-core → authenc-storage → PostgreSQL
    - Test: SSO Login → authenc-api → authenc-federation → External IdP → authenc-core → authenc-storage
    - All flows must work without any code from src/
    - _Requirements: REQ-TEST-002_

- [ ] 18. Clean up old monolithic code - **PRIORITY: HIGH**
  - [ ] 18.1 Delete migrated source files (ONLY after verification)
    - ⚠️ CRITICAL: Only delete files confirmed as fully migrated in MIGRATION_ANALYSIS.md
    - Delete `src/services/` (after verifying all services migrated to crates)
    - Delete `src/handlers/` (after verifying all handlers migrated to crates)
    - Delete `src/middleware/` (after verifying all middleware migrated to crates)
    - Delete `src/database/` (after verifying all database code migrated to crates)
    - Delete `src/crypto/` (after verifying all crypto code migrated to crates)
    - Delete `src/grpc/` (after verifying all gRPC code migrated to crates)
    - Delete `src/models/` (after verifying all models migrated to crates/types)
    - Delete `src/config/` (after verifying all config migrated to crates/core)
    - Delete `src/spi/` (after verifying all SPI code migrated to crates/core)
    - Delete `src/utils/` (after verifying all utils migrated to appropriate crates)
    - Delete `src/routes/` (after verifying all routes migrated to crates/api)
    - Delete `src/axum_app/` (after verifying all app code migrated to crates/api)
    - **Status**: Remove old code
    - _Requirements: REQ-MIG-001_

  - [ ] 18.2 Keep essential root files (DO NOT DELETE)
    - Keep `src/main.rs` (entry point - update to use new crates)
    - Keep `src/lib.rs` (library root - update to re-export crates)
    - Keep `src/app.rs` (simplified AppState - update to use new crates)
    - Keep `src/server.rs` (server initialization - update to use new crates)
    - Keep `src/error.rs` (or move to authenc-types if appropriate)
    - Keep `src/app_logging.rs` (or move to authenc-core if appropriate)
    - Keep `src/admin_console/` (optional Leptos admin UI - feature-gated)
    - Keep `src/bin/` (CLI tools - separate binaries)
    - Keep `migrations/` (SQL migrations - stay in root)
    - Keep `proto/` (gRPC proto files - stay in root)
    - **Status**: Maintain entry points
    - _Requirements: REQ-ARCH-001_

  - [ ] 18.3 Verify src/ directory is minimal
    - After deletion, src/ should only contain:
      - `src/main.rs` (entry point)
      - `src/lib.rs` (re-exports)
      - `src/app.rs` (simplified AppState)
      - `src/server.rs` (server init)
      - `src/error.rs` (or moved to authenc-types)
      - `src/app_logging.rs` (or moved to authenc-core)
      - `src/admin_console/` (optional, feature-gated)
      - `src/bin/` (CLI tools)
    - Verify no business logic remains in src/
    - Verify all business logic is in crates/
    - _Requirements: REQ-ARCH-001, REQ-MIG-001_

  - [ ] 18.3 Update main.rs to use new crates
    - Import from `authenc_api`, `authenc_iam_api`, `authenc_grpc`
    - Import from `authenc_core`, `authenc_storage`, `authenc_crypto`
    - Remove old imports from `src/`
    - **Status**: Update entry point
    - _Requirements: REQ-ARCH-001_

  - [ ] 18.4 Update lib.rs to re-export crates
    - Re-export `authenc_types`, `authenc_core`, `authenc_crypto`
    - Re-export `authenc_storage`, `authenc_api`, `authenc_iam_api`
    - Re-export `authenc_grpc`, `authenc_mfa`, `authenc_federation`, `authenc_webauthn`
    - **Status**: Public API
    - _Requirements: REQ-ARCH-001_

  - [ ] 18.5 Update Cargo.toml dependencies
    - Remove unused dependencies
    - Ensure all crate dependencies are correct
    - Update workspace configuration
    - **Status**: Dependency cleanup
    - _Requirements: REQ-ARCH-001_

  - [ ] 18.6 Update documentation
    - Update AGENTS.md with new architecture
    - Update README.md with new structure
    - Update API documentation (OpenAPI/Swagger)
    - Create migration guide for developers
    - **Status**: Documentation update
    - _Requirements: REQ-DOC-001, REQ-DOC-002, REQ-DOC-003_

  - [ ] 18.7 Final verification that src/ can be safely deleted
    - Run full test suite with only crates/ code (no src/ business logic)
    - Verify all integration tests pass
    - Verify all end-to-end tests pass
    - Verify application compiles and runs with minimal src/
    - Document final state of src/ directory in MIGRATION_ANALYSIS.md
    - Get approval from team before final deletion
    - _Requirements: REQ-MIG-001, REQ-TEST-001, REQ-TEST-002_

- [ ] 19. Performance testing and optimization
  - [ ] 19.1 Run load tests
    - Test authentication throughput: target 1000 req/s
    - Test token validation throughput: target 5000 req/s
    - Test database connection pool under load
    - Use wrk or k6 for load testing
    - _Requirements: REQ-PERF-001, REQ-PERF-002, REQ-PERF-003, REQ-TEST-004_

  - [ ] 19.2 Optimize database queries
    - Add missing indexes
    - Optimize prepared statement cache
    - Tune connection pool size
    - _Requirements: REQ-PERF-004_

  - [ ] 19.3 Optimize Portal WASM bundle
    - Minimize WASM bundle size
    - Add code splitting
    - Optimize asset loading
    - Target: <2 second page load time
    - _Requirements: REQ-PERF-005, AC-PORTAL-001_

- [ ] 20. Security audit and compliance
  - [ ] 20.1 Run security audit
    - Run cargo audit for vulnerability scanning
    - Run cargo deny for license and advisory checks
    - Perform penetration testing
    - Validate OWASP Top 10 compliance
    - _Requirements: REQ-TEST-005, REQ-SEC-008_

  - [ ] 20.2 Verify WebAuthn security properties
    - Test replay attack prevention
    - Test origin binding enforcement
    - Test credential counter validation
    - Test phishing resistance
    - _Requirements: REQ-SEC-010, REQ-SEC-011, REQ-SEC-012_

  - [ ] 20.3 Verify OAuth 2.1 compliance
    - Verify PKCE enforcement for all clients
    - Verify implicit flow is disabled
    - Verify HTTPS enforcement
    - Verify redirect URI exact matching
    - _Requirements: REQ-COMP-003, REQ-FAPI-001_

  - [ ] 20.4 Security code review
    - Review authentication logic
    - Review cryptographic operations
    - Review token handling
    - Review input validation
    - _Requirements: REQ-TEST-005_

- [ ] 21. Production deployment preparation
  - [ ] 21.1 Update Kubernetes manifests
    - Update authenc deployment with new multi-crate structure
    - Add authenc-api and authenc-iam-api services
    - Update authenc-grpc service configuration
    - Add portal deployment with new build
    - Remove layanan-portal deployment
    - _Requirements: REQ-DEPLOY-002_

  - [ ] 21.2 Configure monitoring and observability
    - Add Prometheus metrics for all services
    - Configure structured logging
    - Set up distributed tracing
    - Configure alerting rules
    - _Requirements: REQ-MON-001, REQ-MON-002, REQ-MON-003, REQ-MON-004_

  - [ ] 21.3 Create deployment runbook
    - Document deployment steps
    - Document rollback procedures
    - Document troubleshooting guide
    - Document health check endpoints
    - _Requirements: REQ-DOC-003_

  - [ ] 21.4 Perform staging deployment
    - Deploy to staging environment
    - Run smoke tests
    - Verify all features work
    - Monitor for issues
    - _Requirements: REQ-MIG-001_

  - [ ] 21.5 Perform production deployment
    - Deploy to production with blue-green strategy
    - Monitor health checks
    - Verify zero-downtime deployment
    - Monitor metrics and logs
    - _Requirements: REQ-DEPLOY-003, REQ-REL-001_

- [ ] 22. Final checkpoint - Production verification
  - Verify all services are running in production
  - Verify WebAuthn works across all supported browsers
  - Verify performance targets are met
  - Verify monitoring and alerting are functional
  - Verify src/ directory only contains essential entry points (no business logic)
  - Verify all business logic is in crates/ and working correctly
  - Verify end-to-end integration: Frontend → Backend → Database works flawlessly
  - Verify end-to-end integration: Backend Services → gRPC → Core → Database works flawlessly
  - Update MIGRATION_ANALYSIS.md with final completion status
  - Archive MIGRATION_ANALYSIS.md for future reference
  - Ensure all tests pass, ask the user if questions arise.

---

### Phase 7 (OPTIONAL): FAPI-1 Implementation (4-6 weeks)

**Note**: This phase is OPTIONAL and should only be implemented if required for specific integrations (e.g., financial institutions, government agencies requiring FAPI certification).

- [ ] 23. Implement FAPI-1 Advanced Profile (OPTIONAL)
  - [ ] 23.1 Implement mTLS for token endpoint
    - Add client certificate verification
    - Configure TLS certificate validation
    - Add certificate-bound tokens
    - _Requirements: REQ-FAPI-002, AC-FAPI-001_

  - [ ] 23.2 Implement Signed Request Objects (JAR)
    - Add request object parsing
    - Add JWT signature verification
    - Add request object validation
    - Support request parameter in authorization endpoint
    - _Requirements: REQ-FAPI-002, AC-FAPI-002_

  - [ ] 23.3 Implement ID Token enhancements
    - Add c_hash claim (code hash)
    - Add s_hash claim (state hash)
    - Add at_hash claim (access token hash)
    - _Requirements: REQ-FAPI-002, AC-FAPI-003_

  - [ ] 23.4 Implement JARM (JWT-secured Authorization Response Mode)
    - Add JWT response mode support
    - Sign authorization responses
    - Add response_mode=jwt support
    - _Requirements: REQ-FAPI-002, AC-FAPI-004_

  - [ ] 23.5 Write integration tests for FAPI-1
    - Test mTLS client authentication
    - Test signed request objects
    - Test ID token hash claims
    - Test JARM responses
    - _Requirements: REQ-TEST-002_

  - [ ] 23.6 Run FAPI-1 conformance tests
    - Use OpenID Foundation conformance suite
    - Fix any conformance issues
    - Document conformance results
    - _Requirements: REQ-NFR-012_

---

### Phase 8 (OPTIONAL): FAPI-2 Implementation (6-8 weeks)

**Note**: This phase is OPTIONAL and should only be implemented if FAPI-2 compliance is required. Requires Phase 7 (FAPI-1) to be completed first.

- [ ] 24. Implement FAPI-2 Security Profile (OPTIONAL)
  - [ ] 24.1 Implement Pushed Authorization Requests (PAR)
    - Add PAR endpoint: POST /api/v1/oauth2/par
    - Generate request_uri for authorization requests
    - Store authorization request parameters
    - Add request_uri validation in authorization endpoint
    - _Requirements: REQ-FAPI-003, AC-FAPI-005_

  - [ ] 24.2 Implement DPoP (Demonstrating Proof-of-Possession)
    - Add DPoP proof validation
    - Bind access tokens to DPoP keys
    - Add DPoP-bound token validation
    - Add DPoP header validation in resource endpoints
    - _Requirements: REQ-FAPI-003, AC-FAPI-006, AC-FAPI-007_

  - [ ] 24.3 Implement sender-constrained access tokens
    - Bind tokens to client certificates (mTLS)
    - Bind tokens to DPoP keys
    - Add token binding validation
    - _Requirements: REQ-FAPI-003, REQ-SEC-013_

  - [ ] 24.4 Implement Grant Management API
    - Add grant management endpoints
    - Add grant query endpoint
    - Add grant revocation endpoint
    - Add grant metadata
    - _Requirements: REQ-FAPI-003, AC-FAPI-008_

  - [ ] 24.5 Implement RAR (Rich Authorization Requests)
    - Add authorization_details parameter support
    - Add authorization details validation
    - Add authorization details in token response
    - _Requirements: REQ-FAPI-003_

  - [ ] 24.6 Write integration tests for FAPI-2
    - Test PAR endpoint
    - Test DPoP proof validation
    - Test sender-constrained tokens
    - Test Grant Management API
    - Test RAR support
    - _Requirements: REQ-TEST-002_

  - [ ] 24.7 Run FAPI-2 conformance tests
    - Use OpenID Foundation conformance suite
    - Fix any conformance issues
    - Document conformance results
    - Obtain FAPI-2 certification (if required)
    - _Requirements: REQ-NFR-012_

---
## Notes

### Task Dependency Order Verification

**CRITICAL**: Tasks MUST be executed in the order specified below. Each task depends on the completion of its prerequisites.

**Phase 1: Preparation (Week 1-2)** ✅ COMPLETED
- Task 1: Set up multi-crate workspace structure
- Task 2: Checkpoint - Verify workspace structure
- **Dependencies**: None (foundation phase)

**Phase 2: Core Migration (Week 3-6)**
- Task 2.1: Pre-Migration Analysis → **MUST complete FIRST** (blocks all Phase 2 tasks)
- Task 3: Migrate authenc-storage → **Depends on**: Task 2.1
- Task 4: Migrate authenc-crypto → **Depends on**: Task 2.1
- Task 5: Migrate authenc-core → **Depends on**: Task 3 (storage), Task 4 (crypto)
- Task 6: Migrate authenc-webauthn → **Depends on**: Task 5 (core), Task 3 (storage)
- Task 7: Checkpoint → **Depends on**: Task 3, 4, 5, 6

**Dependency Chain**: 2.1 → (3, 4) → 5 → 6 → 7

**Phase 3: API Migration (Week 7-8)**
- Task 7.1: Pre-API Migration Analysis → **MUST complete FIRST** (blocks all Phase 3 tasks)
- Task 8: Migrate authenc-api → **Depends on**: Task 7 (Phase 2 complete), Task 7.1
- Task 9: Migrate authenc-iam-api → **Depends on**: Task 7 (Phase 2 complete), Task 7.1
- Task 10: Migrate authenc-grpc → **Depends on**: Task 7 (Phase 2 complete), Task 7.1
- Task 11: Checkpoint → **Depends on**: Task 8, 9, 10

**Dependency Chain**: 7 → 7.1 → (8, 9, 10) → 11

**Phase 4: Feature Migration (Week 9-10)**
- Task 11.1: Pre-Feature Migration Analysis → **MUST complete FIRST** (blocks all Phase 4 tasks)
- Task 12: Migrate authenc-mfa → **Depends on**: Task 11 (Phase 3 complete), Task 11.1
- Task 13: Migrate authenc-federation → **Depends on**: Task 11 (Phase 3 complete), Task 11.1
- Task 14: Checkpoint → **Depends on**: Task 12, 13

**Dependency Chain**: 11 → 11.1 → (12, 13) → 14

**Phase 5: Portal Refactoring (Week 11-12)**
- Task 15: Rebuild Portal IAM Microfrontend → **Depends on**: Task 14 (Phase 4 complete)
- Task 16: Eliminate layanan-portal → **Depends on**: Task 15
- Task 17: Checkpoint → **Depends on**: Task 15, 16

**Dependency Chain**: 14 → 15 → 16 → 17

**Phase 6: Cleanup and Optimization (Week 13-14)**
- Task 17.1: Final Migration Analysis → **MUST complete FIRST** (blocks all Phase 6 tasks)
- Task 18: Clean up old monolithic code → **Depends on**: Task 17 (Phase 5 complete), Task 17.1
- Task 19: Performance testing → **Depends on**: Task 18
- Task 20: Security audit → **Depends on**: Task 18
- Task 21: Production deployment → **Depends on**: Task 19, 20
- Task 22: Final checkpoint → **Depends on**: Task 21

**Dependency Chain**: 17 → 17.1 → 18 → (19, 20) → 21 → 22

**Phase 7 (OPTIONAL): FAPI-1 Implementation (4-6 weeks)**
- Task 23: Implement FAPI-1 → **Depends on**: Task 22 (Phase 6 complete)

**Phase 8 (OPTIONAL): FAPI-2 Implementation (6-8 weeks)**
- Task 24: Implement FAPI-2 → **Depends on**: Task 23 (FAPI-1 complete)

**Key Dependency Rules**:
1. **Pre-Analysis Tasks**: All `.1` tasks (2.1, 7.1, 11.1, 17.1) MUST complete before their phase tasks
2. **Storage First**: Task 3 (storage) must complete before Task 5 (core) - core depends on storage
3. **Crypto First**: Task 4 (crypto) must complete before Task 5 (core) - core depends on crypto
4. **Core Before WebAuthn**: Task 5 (core) must complete before Task 6 (webauthn) - webauthn depends on core
5. **Core Before API**: Task 7 (Phase 2 checkpoint) must complete before Task 8, 9, 10 (API) - API depends on core
6. **API Before Features**: Task 11 (Phase 3 checkpoint) must complete before Task 12, 13 (MFA, Federation) - features depend on API
7. **Features Before Portal**: Task 14 (Phase 4 checkpoint) must complete before Task 15 (Portal) - Portal depends on features
8. **Portal Before Cleanup**: Task 17 (Phase 5 checkpoint) must complete before Task 18 (Cleanup) - cleanup requires all features working
9. **Cleanup Before Production**: Task 18 (Cleanup) must complete before Task 21 (Production) - production requires clean codebase

**Parallel Execution Opportunities**:
- Task 3 (storage) and Task 4 (crypto) can run in parallel (no dependency between them)
- Task 8 (api), Task 9 (iam-api), Task 10 (grpc) can run in parallel (all depend on Phase 2, not each other)
- Task 12 (mfa) and Task 13 (federation) can run in parallel (all depend on Phase 3, not each other)
- Task 19 (performance) and Task 20 (security) can run in parallel (both depend on Task 18, not each other)

### Migration Analysis Document (MIGRATION_ANALYSIS.md)

Throughout the migration, maintain a living document `layanan/authenc/MIGRATION_ANALYSIS.md` that tracks:

**For each file in src/**:
- ✅ **Migrated**: File successfully moved to crates/, tested, and verified
- 🔄 **In Progress**: File migration started but not complete
- ⚠️ **Needs Modification**: File has issues (circular deps, tight coupling) - needs refactoring before migration
- ❌ **Do Not Migrate**: File must stay in src/ (entry points, feature-gated code, CLI tools)
- 📝 **Notes**: Any special considerations or dependencies

**Integration Status**:
- Document all crate boundary integrations (which crates depend on which)
- Document all end-to-end flows and their status (working/broken/untested)
- Document any blocking issues or circular dependencies discovered

**Deletion Checklist**:
- Before deleting any directory in src/, verify:
  1. All files in that directory are marked ✅ Migrated in MIGRATION_ANALYSIS.md
  2. All integration tests pass without that directory
  3. All end-to-end tests pass without that directory
  4. Team approval obtained

### Task Marking Convention
- Tasks marked with `*` (e.g., `- [ ]* Task name`) are OPTIONAL and can be skipped for faster MVP
- Tasks WITHOUT `*` are REQUIRED and must be implemented
- All WebAuthn/Passkeys tasks are REQUIRED (no asterisk) - PRIMARY authentication method
- All FAPI tasks (Phase 7 and 8) are OPTIONAL (with asterisk) - only implement if needed

### Testing Strategy
- Unit tests: >80% line coverage, >90% branch coverage
- Integration tests: All API endpoints and critical flows
- Property-based tests: Cryptographic invariants and security properties
- Performance tests: Authentication <100ms p99, token validation <50ms p99
- Security tests: Penetration testing, vulnerability scanning, OWASP Top 10

### WebAuthn Browser Compatibility
- Chrome 67+ (WebAuthn Level 1)
- Firefox 60+ (WebAuthn Level 1)
- Safari 13+ (WebAuthn Level 1)
- Edge 18+ (WebAuthn Level 1)
- Chrome 93+ (WebAuthn Level 2 - conditional UI)
- Safari 16+ (WebAuthn Level 3 - passkey sync)

### Passkey Platform Support
- **iOS/iPadOS**: Touch ID, Face ID, iCloud Keychain sync
- **macOS**: Touch ID, iCloud Keychain sync
- **Android**: Biometric authentication, Google Password Manager sync
- **Windows**: Windows Hello (biometric or PIN)
- **Security Keys**: YubiKey, Titan Key, FIDO2-compliant keys

### FAPI Implementation Decision Tree
1. **Do you need FAPI compliance?**
   - No → Skip Phase 7 and 8 (OPTIONAL)
   - Yes → Continue to step 2

2. **Which FAPI profile do you need?**
   - FAPI-1 only → Implement Phase 7 (4-6 weeks)
   - FAPI-2 → Implement Phase 7 + Phase 8 (10-14 weeks total)

3. **Do you need FAPI certification?**
   - No → Implement features, skip conformance testing
   - Yes → Run conformance tests and obtain certification

### Migration Rollback Plan
If critical issues are discovered during migration:
1. **Immediate rollback**: Revert to old monolithic code (keep in parallel during Phase 1-5)
2. **Gradual rollback**: Disable new features via feature flags
3. **Data rollback**: Database migrations are backward compatible

### Critical Migration Principles

**1. Analysis Before Action**
- NEVER migrate a file without first analyzing its dependencies
- NEVER delete a file without verifying it's fully migrated and tested
- ALWAYS document migration decisions in MIGRATION_ANALYSIS.md

**2. Integration Verification**
- ALWAYS test crate boundary integrations after each migration
- ALWAYS test end-to-end flows after each phase
- NEVER assume integration works - always verify with tests

**3. End-to-End Integration Requirements**
Every migration task MUST verify:
- **Vertical Integration**: Frontend → API → Core → Storage → Database
- **Horizontal Integration**: Crate A → Crate B → Crate C (dependency chain)
- **Cross-Cutting Integration**: All crates → authenc-types (trait implementations)

**4. Deletion Safety**
Before deleting any src/ directory:
- ✅ All files marked as migrated in MIGRATION_ANALYSIS.md
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ All end-to-end tests pass
- ✅ Application runs without that directory
- ✅ Team approval obtained

**5. Continuous Verification**
After each task:
- Run `cargo check --workspace` (must pass)
- Run `cargo test --workspace` (must pass)
- Run integration tests (must pass)
- Update MIGRATION_ANALYSIS.md (document progress)

### End-to-End Integration Flows (Must All Work)

**Flow 1: Passkey Authentication (Frontend → Backend → Database)**
```
Portal Login Page (Leptos WASM)
  → POST /api/v1/auth/webauthn/authenticate
    → authenc-api handler
      → authenc-webauthn service
        → authenc-storage credential_store
          → PostgreSQL (verify credential)
            → authenc-core authentication_service
              → authenc-storage session_store
                → PostgreSQL (create session)
                  → authenc-crypto jwt_service
                    → Generate JWT
                      → Return to Portal
```

**Flow 2: Password Authentication (Frontend → Backend → Database)**
```
Portal Login Page (Leptos WASM)
  → POST /api/v1/auth/login
    → authenc-api handler
      → authenc-core authentication_service
        → authenc-storage user_store
          → PostgreSQL (get user)
            → authenc-crypto password_hasher
              → Verify password
                → authenc-storage session_store
                  → PostgreSQL (create session)
                    → authenc-crypto jwt_service
                      → Generate JWT
                        → Return to Portal
```

**Flow 3: MFA Authentication (Frontend → Backend → Secreton)**
```
Portal MFA Page (Leptos WASM)
  → POST /api/v1/auth/mfa/verify
    → authenc-api handler
      → authenc-mfa service
        → Secreton gRPC (get TOTP secret)
          → authenc-crypto totp_verifier
            → Verify code
              → authenc-core authentication_service
                → authenc-storage session_store
                  → PostgreSQL (create session)
                    → authenc-crypto jwt_service
                      → Generate JWT
                        → Return to Portal
```

**Flow 4: IAM Admin (Frontend → Backend → Database)**
```
Portal IAM Admin Page (Leptos WASM)
  → POST /api/v1/iam/users
    → authenc-iam-api handler
      → authenc-core user_management_service
        → authenc-crypto password_hasher
          → Hash password
            → authenc-storage user_store
              → PostgreSQL (create user)
                → authenc-core event_publisher
                  → Publish UserCreated event
                    → Return to Portal
```

**Flow 5: Backend Service Authentication (Service → gRPC → Database)**
```
layanan-perlengkapan (Axum)
  → gRPC Authenticate(username, password)
    → authenc-grpc service
      → authenc-core authentication_service
        → authenc-storage user_store
          → PostgreSQL (get user)
            → authenc-crypto password_hasher
              → Verify password
                → authenc-crypto jwt_service
                  → Generate JWT
                    → Return to layanan-perlengkapan
```

**Flow 6: SSO Authentication (Frontend → Backend → External IdP → Database)**
```
Portal SSO Button (Leptos WASM)
  → GET /api/v1/auth/sso/google
    → authenc-api handler
      → authenc-federation service
        → Redirect to Google
          → User authenticates
            → Callback to /api/v1/auth/sso/callback
              → authenc-federation service
                → authenc-core user_management_service
                  → authenc-storage user_store
                    → PostgreSQL (create/link user)
                      → authenc-core authentication_service
                        → authenc-storage session_store
                          → PostgreSQL (create session)
                            → authenc-crypto jwt_service
                              → Generate JWT
                                → Return to Portal
```

All these flows MUST work before src/ can be deleted!

### Success Metrics
- **Technical**: >80% test coverage, <100ms p99 authentication latency
- **Business**: 99.9% uptime, <0.1% authentication error rate
- **Migration**: Zero data loss, zero downtime deployment

### Dependencies
- **External**: PostgreSQL 16.x, Redis 7.x (optional), Secreton service
- **Crates**: webauthn-rs 0.5.x (MANDATORY), jsonwebtoken, argon2, tonic, axum, leptos
- **Infrastructure**: Kubernetes cluster, Istio service mesh, MetalLB

### Related Documentation
- [Design Document](./design.md) - Complete architectural design
- [Requirements Document](./requirements.md) - Functional and non-functional requirements
- [AGENTS.md - Authenc](../../AGENTS.md) - Authenc-specific conventions
- [AGENTS.md - Root](../../../../AGENTS.md) - Project-wide conventions

---

**Document Version**: 1.0
**Last Updated**: 2026-02-19
**Authors**: SIMPEL Architecture Team
**Status**: Ready for Implementation
**Workflow**: Design-First (design.md ✅ → requirements.md ✅ → tasks.md ✅)
**Estimated Duration**: 14 weeks (core) + 4-6 weeks (FAPI-1, optional) + 6-8 weeks (FAPI-2, optional)
