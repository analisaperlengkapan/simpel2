# Task 8 Execution Plan: Migrate authenc-api (Public REST API)

**Status**: 📋 PLANNING
**Phase**: Phase 3 - API Migration
**Priority**: HIGH
**Estimated Duration**: Week 7-8 (10-12 days)

## Overview

Task 8 migrates the public REST API from the monolithic `src/handlers/` structure to the new `crates/api/` crate. This is the most critical task in Phase 3 as it provides the interface between frontend microfrontends and the backend services.

**Scope**:

- 22 handler files to migrate (Priority 1-3)
- 12 middleware files to migrate
- Router configuration
- API state management
- Integration tests

**Dependencies**:

- ✅ authenc-core (Task 5) - Complete
- ✅ authenc-storage (Task 3) - Complete
- ✅ authenc-crypto (Task 4) - Complete
- ✅ authenc-webauthn (Task 6) - Complete
- ✅ Pre-API Migration Analysis (Task 7.1) - Complete

## Execution Strategy

### Phase-Based Approach

We'll migrate in 3 phases to minimize risk and ensure each component works before moving to the next:

**Phase 1 (Days 1-4)**: Core Authentication

- Migrate authentication handlers (session, TOTP, auth_helpers)
- Migrate authentication middleware (auth_middleware, csrf_protection)
- Test authentication flows

**Phase 2 (Days 5-8)**: OAuth2/OIDC & Security

- Migrate OAuth2/OIDC handlers
- Migrate rate limiting middleware
- Migrate security middleware
- Test OAuth2 flows

**Phase 3 (Days 9-12)**: Infrastructure & Integration

- Migrate utility handlers (health, metrics)
- Migrate remaining middleware
- Create unified router
- Integration tests
- Documentation

### Migration Principles

1. **One handler at a time**: Migrate, test, verify before moving to next
2. **Preserve functionality**: Maintain exact same API contracts
3. **Update imports**: Use new crate paths (authenc-core, authenc-storage, etc.)
4. **Test immediately**: Run tests after each migration
5. **Document changes**: Update MIGRATION_ANALYSIS.md continuously

## Detailed Execution Plan

### Phase 1: Core Authentication (Days 1-4)

#### Day 1: Authentication Handlers (Priority 1)

**Task 8.1.1: Migrate session.rs**

- [ ] Read `src/handlers/session.rs` to understand implementation
- [ ] Create `crates/api/src/handlers/session.rs`
- [ ] Update imports:
  - `crate::services::session_store` → `authenc_core::services::session_store`
  - `crate::models::session` → `authenc_types::domain::session`
  - `crate::error` → `authenc_types::error`
- [ ] Update handler signatures to use `State<Arc<ApiState>>`
- [ ] Test compilation: `cargo check -p authenc-api`
- [ ] Add to `crates/api/src/handlers/mod.rs`
- [ ] Mark as migrated in MIGRATION_ANALYSIS.md

**Task 8.1.2: Migrate totp.rs**

- [ ] Read `src/handlers/totp.rs` to understand implementation
- [ ] Create `crates/api/src/handlers/totp.rs`
- [ ] Update imports:
  - `crate::services::totp_store` → `authenc_core::services::totp_store`
  - `crate::services::mfa_service` → `authenc_core::services::mfa_service`
  - `crate::models::user` → `authenc_types::domain::user`
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.1.3: Migrate totp_verify.rs**

- [ ] Read `src/handlers/totp_verify.rs`
- [ ] Create `crates/api/src/handlers/totp_verify.rs`
- [ ] Update imports (similar to totp.rs)
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.1.4: Migrate auth_helpers.rs**

- [ ] Read `src/handlers/auth_helpers.rs`
- [ ] Create `crates/api/src/handlers/auth_helpers.rs`
- [ ] Update imports
- [ ] Update helper functions
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Checkpoint 1.1**: Verify authentication handlers compile

```bash
cargo check -p authenc-api
cargo test -p authenc-api --lib
```

#### Day 2: Authentication Middleware (Priority 1)

**Task 8.5.1: Migrate auth_middleware.rs**

- [ ] Read `src/middleware/auth_middleware.rs`
- [ ] Create `crates/api/src/middleware/auth.rs`
- [ ] Update imports:
  - `crate::services::jwt_validator` → `authenc_crypto::jwt_validator`
  - `crate::services::session_store` → `authenc_core::services::session_store`
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to `crates/api/src/middleware/mod.rs`
- [ ] Mark as migrated

**Task 8.5.2: Migrate csrf_protection.rs**

- [ ] Read `src/middleware/csrf_protection.rs`
- [ ] Create `crates/api/src/middleware/csrf.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Checkpoint 1.2**: Verify middleware compiles

```bash
cargo check -p authenc-api
```

#### Day 3: Create ApiState and Router (Phase 1)

**Task 8.7.1: Create ApiState**

- [ ] Create `crates/api/src/state.rs`
- [ ] Define `ApiState` struct:

  ```rust
  pub struct ApiState {
      pub database: Arc<Database>,
      pub jwt_service: Arc<JwtService>,
      pub session_store: Arc<SessionStore>,
      pub user_store: Arc<UserStore>,
      pub totp_store: Arc<TotpStore>,
      pub mfa_service: Arc<MfaService>,
      pub webauthn_service: Arc<WebAuthnService>,
      pub config: Arc<AppConfig>,
  }
  ```

- [ ] Add constructor methods
- [ ] Test compilation

**Task 8.6.1: Create Router (Phase 1 - Authentication only)**

- [ ] Create `crates/api/src/router.rs`
- [ ] Add authentication routes:

  ```rust
  Router::new()
      .route("/api/v1/auth/session", get(session_handler))
      .route("/api/v1/auth/totp/setup", post(totp_setup_handler))
      .route("/api/v1/auth/totp/verify", post(totp_verify_handler))
      .layer(auth_middleware_layer())
      .layer(csrf_protection_layer())
      .with_state(state)
  ```

- [ ] Test compilation

**Checkpoint 1.3**: Verify ApiState and Router compile

```bash
cargo check -p authenc-api
```

#### Day 4: Integration Tests (Phase 1)

**Task 8.8.1: Write authentication integration tests**

- [ ] Create `crates/api/tests/authentication_tests.rs`
- [ ] Test session creation
- [ ] Test TOTP setup flow
- [ ] Test TOTP verification flow
- [ ] Test auth middleware (valid/invalid tokens)
- [ ] Test CSRF protection
- [ ] Run tests: `cargo test -p authenc-api`

**Checkpoint 1.4**: Phase 1 Complete

- [ ] All authentication handlers migrated
- [ ] All authentication middleware migrated
- [ ] ApiState created
- [ ] Router (Phase 1) created
- [ ] Integration tests passing
- [ ] Update MIGRATION_ANALYSIS.md

**Phase 1 Success Criteria**:

- ✅ 4 handlers migrated (session, totp, totp_verify, auth_helpers)
- ✅ 2 middleware migrated (auth_middleware, csrf_protection)
- ✅ ApiState created
- ✅ Router (Phase 1) created
- ✅ Integration tests passing (>5 tests)
- ✅ No compilation errors

---

### Phase 2: OAuth2/OIDC & Security (Days 5-8)

#### Day 5: OAuth2/OIDC Handlers (Priority 2)

**Task 8.2.1: Compare oauth2.rs implementations**

- [ ] Read `src/handlers/oauth2.rs` (monolithic version)
- [ ] Read `crates/api/src/handlers/oauth2.rs` (already migrated)
- [ ] Identify missing features in crates/ version
- [ ] Document differences
- [ ] Plan merge strategy

**Task 8.2.2: Merge oauth2.rs features**

- [ ] Add missing features from src/ to crates/
- [ ] Update imports
- [ ] Test compilation
- [ ] Mark as complete

**Task 8.2.3: Migrate oauth2_authz_code.rs**

- [ ] Read `src/handlers/oauth2_authz_code.rs`
- [ ] Create `crates/api/src/handlers/oauth2_authz_code.rs`
- [ ] Update imports:
  - `crate::services::oidc_client_store` → `authenc_core::services::oidc_client_store`
  - `crate::services::oidc_code_store` → `authenc_core::services::oidc_code_store`
  - `crate::services::consent_store` → `authenc_core::services::consent_store`
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.4: Migrate oidc_ed25519.rs**

- [ ] Read `src/handlers/oidc_ed25519.rs`
- [ ] Create `crates/api/src/handlers/oidc_ed25519.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.5: Migrate oidc_provider.rs**

- [ ] Read `src/handlers/oidc_provider.rs`
- [ ] Create `crates/api/src/handlers/oidc_provider.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Checkpoint 2.1**: Verify OAuth2 handlers compile

```bash
cargo check -p authenc-api
```

#### Day 6: More OAuth2/OIDC Handlers (Priority 2)

**Task 8.2.6: Migrate oidc_sso.rs**

- [ ] Read `src/handlers/oidc_sso.rs`
- [ ] Create `crates/api/src/handlers/oidc_sso.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.7: Migrate jwks.rs**

- [ ] Read `src/handlers/jwks.rs`
- [ ] Create `crates/api/src/handlers/jwks.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.8: Migrate jwt_ed25519.rs**

- [ ] Read `src/handlers/jwt_ed25519.rs`
- [ ] Create `crates/api/src/handlers/jwt_ed25519.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.9: Migrate oidc_keys.rs**

- [ ] Read `src/handlers/oidc_keys.rs`
- [ ] Create `crates/api/src/handlers/oidc_keys.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Checkpoint 2.2**: Verify all OAuth2 handlers compile

```bash
cargo check -p authenc-api
```

#### Day 7: Rate Limiting Middleware (Priority 2)

**Task 8.5.3: Compare rate_limit.rs implementations**

- [ ] Read `src/middleware/rate_limit.rs`
- [ ] Read `crates/api/src/middleware/rate_limit.rs`
- [ ] Identify missing features
- [ ] Document differences
- [ ] Plan merge strategy

**Task 8.5.4: Merge rate_limit.rs features**

- [ ] Add missing features from src/ to crates/
- [ ] Update imports
- [ ] Test compilation
- [ ] Mark as complete

**Task 8.5.5: Migrate adaptive_rate_limit.rs**

- [ ] Read `src/middleware/adaptive_rate_limit.rs`
- [ ] Create `crates/api/src/middleware/adaptive_rate_limit.rs`
- [ ] Update imports:
  - `crate::services::brute_force_protector` → `authenc_core::services::brute_force_protector`
  - `crate::services::anomaly_detector` → `authenc_core::services::anomaly_detector`
  - `crate::services::risk_engine` → `authenc_core::services::risk_engine`
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.6: Migrate adaptive_rate_limit_integration.rs**

- [ ] Read `src/middleware/adaptive_rate_limit_integration.rs`
- [ ] Create `crates/api/src/middleware/adaptive_rate_limit_integration.rs`
- [ ] Update imports
- [ ] Update helper functions
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.7: Migrate mfa_rate_limit.rs**

- [ ] Read `src/middleware/mfa_rate_limit.rs`
- [ ] Create `crates/api/src/middleware/mfa_rate_limit.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Checkpoint 2.3**: Verify rate limiting middleware compiles

```bash
cargo check -p authenc-api
```

#### Day 8: Security Middleware (Priority 2)

**Task 8.5.8: Migrate security_monitoring.rs**

- [ ] Read `src/middleware/security_monitoring.rs`
- [ ] Create `crates/api/src/middleware/security.rs`
- [ ] Update imports:
  - `crate::services::audit_log_store` → `authenc_core::services::audit_log_store`
  - `crate::services::event_publisher` → `authenc_core::services::event_publisher`
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.9: Migrate input_validation.rs**

- [ ] Read `src/middleware/input_validation.rs`
- [ ] Create `crates/api/src/middleware/validation.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.10: Migrate request_size_limit.rs**

- [ ] Read `src/middleware/request_size_limit.rs`
- [ ] Create `crates/api/src/middleware/size_limit.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.11: Migrate rbac.rs**

- [ ] Read `src/middleware/rbac.rs`
- [ ] Create `crates/api/src/middleware/rbac.rs`
- [ ] Update imports:
  - `crate::services::role_store` → `authenc_core::services::role_store`
  - `crate::services::permission_store` → `authenc_core::services::permission_store`
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Checkpoint 2.4**: Verify security middleware compiles

```bash
cargo check -p authenc-api
```

**Task 8.8.2: Write OAuth2 integration tests**

- [ ] Create `crates/api/tests/oauth2_tests.rs`
- [ ] Test OAuth2 authorization code flow
- [ ] Test OIDC discovery endpoint
- [ ] Test JWKS endpoint
- [ ] Test token endpoint
- [ ] Test userinfo endpoint
- [ ] Run tests: `cargo test -p authenc-api`

**Checkpoint 2.5**: Phase 2 Complete

- [ ] All OAuth2/OIDC handlers migrated (8 handlers)
- [ ] All rate limiting middleware migrated (4 middleware)
- [ ] All security middleware migrated (4 middleware)
- [ ] Integration tests passing
- [ ] Update MIGRATION_ANALYSIS.md

**Phase 2 Success Criteria**:

- ✅ 8 OAuth2/OIDC handlers migrated
- ✅ 8 middleware migrated (rate limiting + security)
- ✅ OAuth2 integration tests passing (>10 tests)
- ✅ No compilation errors

---

### Phase 3: Infrastructure & Integration (Days 9-12)

#### Day 9: Utility Handlers (Priority 3)

**Task 8.4.1: Migrate health.rs**

- [ ] Read `src/handlers/health.rs`
- [ ] Create `crates/api/src/handlers/health.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.4.2: Migrate metrics.rs**

- [ ] Read `src/handlers/metrics.rs`
- [ ] Create `crates/api/src/handlers/metrics.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.4.3: Migrate validation_helper.rs**

- [ ] Read `src/handlers/validation_helper.rs`
- [ ] Create `crates/api/src/handlers/validation_helper.rs`
- [ ] Update imports
- [ ] Update helper functions
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.4.4: Migrate consent_ui.rs**

- [ ] Read `src/handlers/consent_ui.rs`
- [ ] Create `crates/api/src/handlers/consent_ui.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.4.5: Migrate authorization.rs**

- [ ] Read `src/handlers/authorization.rs`
- [ ] Create `crates/api/src/handlers/authorization.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Checkpoint 3.1**: Verify utility handlers compile

```bash
cargo check -p authenc-api
```

#### Day 10: Remaining Middleware (Priority 3)

**Task 8.5.12: Migrate compression.rs**

- [ ] Read `src/middleware/compression.rs`
- [ ] Create `crates/api/src/middleware/compression.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.13: Migrate mtls.rs**

- [ ] Read `src/middleware/mtls.rs`
- [ ] Create `crates/api/src/middleware/mtls.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Task 8.5.14: Migrate mfa_performance_middleware.rs**

- [ ] Read `src/middleware/mfa_performance_middleware.rs`
- [ ] Create `crates/api/src/middleware/mfa_performance.rs`
- [ ] Update imports
- [ ] Update middleware implementation
- [ ] Test compilation
- [ ] Add to middleware/mod.rs
- [ ] Mark as migrated

**Checkpoint 3.2**: Verify remaining middleware compiles

```bash
cargo check -p authenc-api
```

#### Day 11: SSO Handlers & Token Exchange (Priority 3)

**Task 8.3.1: Migrate sso.rs**

- [ ] Read `src/handlers/sso.rs`
- [ ] Create `crates/api/src/handlers/sso.rs`
- [ ] Update imports
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Task 8.2.10: Migrate token_exchange.rs**

- [ ] Read `src/handlers/token_exchange.rs`
- [ ] Create `crates/api/src/handlers/token_exchange.rs`
- [ ] Update imports:
  - `crate::services::token_exchange` → `authenc_core::services::token_exchange`
- [ ] Update handler signatures
- [ ] Test compilation
- [ ] Add to mod.rs
- [ ] Mark as migrated

**Checkpoint 3.3**: Verify SSO and token exchange handlers compile

```bash
cargo check -p authenc-api
```

#### Day 12: Final Router & Integration Tests

**Task 8.6.2: Complete unified router**

- [ ] Update `crates/api/src/router.rs` with all routes
- [ ] Add all migrated handlers
- [ ] Apply middleware layers in correct order:
  1. security_monitoring
  2. csrf_protection
  3. rate_limit
  4. rbac
  5. auth_middleware
- [ ] Test compilation
- [ ] Test router with mock requests

**Task 8.7.2: Complete ApiState**

- [ ] Update `crates/api/src/state.rs` with all services
- [ ] Add OAuth2 services
- [ ] Add federation services (if needed)
- [ ] Add audit services
- [ ] Test compilation

**Task 8.8.3: Write comprehensive integration tests**

- [ ] Create `crates/api/tests/integration_tests.rs`
- [ ] Test complete authentication flow (login → session → logout)
- [ ] Test complete OAuth2 flow (authorize → token → userinfo)
- [ ] Test WebAuthn flow (register → authenticate)
- [ ] Test rate limiting (exceed limits)
- [ ] Test CSRF protection (missing token)
- [ ] Test RBAC (unauthorized access)
- [ ] Run all tests: `cargo test -p authenc-api`

**Task 8.9: Verify API integration**

- [ ] Test authenc-api → authenc-core integration
- [ ] Test authenc-api → authenc-webauthn integration
- [ ] Test authenc-api → authenc-crypto integration
- [ ] Test CORS configuration
- [ ] Test error handling
- [ ] Test logging and metrics

**Task 8.10: Documentation**

- [ ] Update MIGRATION_ANALYSIS.md with complete API migration status
- [ ] Document all migrated handlers
- [ ] Document all migrated middleware
- [ ] Document remaining files in src/
- [ ] Create API migration summary document

**Checkpoint 3.4**: Phase 3 Complete

- [ ] All utility handlers migrated (5 handlers)
- [ ] All remaining middleware migrated (3 middleware)
- [ ] Unified router complete
- [ ] ApiState complete
- [ ] All integration tests passing (>30 tests)
- [ ] Documentation complete
- [ ] Update MIGRATION_ANALYSIS.md

**Phase 3 Success Criteria**:

- ✅ 7 additional handlers migrated (utility + SSO + token exchange)
- ✅ 3 remaining middleware migrated
- ✅ Unified router complete
- ✅ ApiState complete
- ✅ All integration tests passing (>30 tests total)
- ✅ Documentation complete
- ✅ No compilation errors

---

## Final Verification Checklist

### Compilation

- [ ] `cargo check -p authenc-api` - No errors
- [ ] `cargo check --workspace` - No errors
- [ ] `cargo clippy -p authenc-api` - No warnings

### Testing

- [ ] `cargo test -p authenc-api --lib` - All unit tests pass
- [ ] `cargo test -p authenc-api --test authentication_tests` - Pass
- [ ] `cargo test -p authenc-api --test oauth2_tests` - Pass
- [ ] `cargo test -p authenc-api --test integration_tests` - Pass
- [ ] Total test count: >30 tests
- [ ] Test coverage: >70%

### Integration

- [ ] authenc-api → authenc-core integration verified
- [ ] authenc-api → authenc-storage integration verified
- [ ] authenc-api → authenc-crypto integration verified
- [ ] authenc-api → authenc-webauthn integration verified
- [ ] CORS configuration verified
- [ ] Error handling verified
- [ ] Logging verified
- [ ] Metrics verified

### Documentation

- [ ] MIGRATION_ANALYSIS.md updated
- [ ] All migrated handlers documented
- [ ] All migrated middleware documented
- [ ] Remaining files in src/ documented
- [ ] API migration summary created

### Migration Status

- [ ] 22 handlers migrated (100%)
- [ ] 12 middleware migrated (100%)
- [ ] Router complete
- [ ] ApiState complete
- [ ] Integration tests complete
- [ ] Documentation complete

## Success Criteria

Task 8 is considered COMPLETE when:

1. **All handlers migrated**: 22/22 handlers in `crates/api/src/handlers/`
2. **All middleware migrated**: 12/12 middleware in `crates/api/src/middleware/`
3. **Router complete**: Unified router in `crates/api/src/router.rs`
4. **ApiState complete**: Complete state management in `crates/api/src/state.rs`
5. **Tests passing**: >30 integration tests passing (100% pass rate)
6. **No compilation errors**: `cargo check --workspace` succeeds
7. **Documentation complete**: MIGRATION_ANALYSIS.md updated
8. **Integration verified**: All crate integrations tested and working

## Risk Mitigation

### High-Risk Areas

1. **OAuth2 handler merge**: src/ and crates/ versions may have different features
   - **Mitigation**: Careful comparison and feature-by-feature merge
   - **Fallback**: Keep both versions temporarily, test extensively

2. **Rate limiting merge**: src/ and crates/ versions may differ
   - **Mitigation**: Careful comparison and feature-by-feature merge
   - **Fallback**: Use src/ version if crates/ version is incomplete

3. **Middleware ordering**: Incorrect order can break security
   - **Mitigation**: Document correct order, test each layer
   - **Fallback**: Revert to known-good configuration

4. **Import updates**: Many imports need updating
   - **Mitigation**: Use find-replace carefully, test after each change
   - **Fallback**: Use git to revert if imports break

### Rollback Plan

If Task 8 encounters blocking issues:

1. **Preserve src/ handlers**: Do NOT delete until crates/ version is verified
2. **Use feature flags**: Keep both implementations, switch via feature flag
3. **Incremental rollback**: Roll back one handler at a time if needed
4. **Git branches**: Use separate branch for Task 8, merge only when complete

## Timeline

| Day | Phase | Tasks | Deliverables |
|-----|-------|-------|--------------|
| 1 | Phase 1 | 8.1.1-8.1.4 | 4 authentication handlers migrated |
| 2 | Phase 1 | 8.5.1-8.5.2 | 2 authentication middleware migrated |
| 3 | Phase 1 | 8.7.1, 8.6.1 | ApiState + Router (Phase 1) created |
| 4 | Phase 1 | 8.8.1 | Authentication integration tests passing |
| 5 | Phase 2 | 8.2.1-8.2.5 | 5 OAuth2 handlers migrated |
| 6 | Phase 2 | 8.2.6-8.2.9 | 4 more OAuth2 handlers migrated |
| 7 | Phase 2 | 8.5.3-8.5.7 | 5 rate limiting middleware migrated |
| 8 | Phase 2 | 8.5.8-8.5.11, 8.8.2 | 4 security middleware + OAuth2 tests |
| 9 | Phase 3 | 8.4.1-8.4.5 | 5 utility handlers migrated |
| 10 | Phase 3 | 8.5.12-8.5.14 | 3 remaining middleware migrated |
| 11 | Phase 3 | 8.3.1, 8.2.10 | SSO + token exchange handlers migrated |
| 12 | Phase 3 | 8.6.2, 8.7.2, 8.8.3, 8.9, 8.10 | Router + tests + docs complete |

**Total Duration**: 12 days (Week 7-8)

## Next Steps

After Task 8 completion:

1. **Task 9**: Migrate authenc-iam-api (Admin REST API)
2. **Task 10**: Migrate authenc-grpc (Service-to-Service gRPC)
3. **Task 11**: Checkpoint - Verify API implementation

---

**Plan Created by**: Kiro AI Assistant
**Date**: 2026-02-20
**Phase**: Phase 3 - API Migration
**Status**: Ready for execution
