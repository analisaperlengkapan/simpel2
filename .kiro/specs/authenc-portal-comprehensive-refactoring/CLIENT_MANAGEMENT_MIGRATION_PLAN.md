# Client Management Migration Plan

**Date**: 2026-02-20
**Goal**: Migrate Client Management feature from OLD (`src/`) to NEW (`crates/`) architecture
**Priority**: CRITICAL (User confirmed this is essential like Keycloak)

---

## Overview

Client Management adalah fitur CRITICAL yang memungkinkan:

1. **Client CRUD** - Create, Read, Update, Delete OAuth2/OIDC clients
2. **Dynamic Client Registration (DCR)** - RFC 7591/7592 compliance
3. **Client Policies** - Authorization policies untuk client registration
4. **Client Scopes** - Scope management per client

---

## Current State Analysis

### OLD Architecture (`src/`)

**Files Found**:

1. `src/handlers/oidc_client.rs` - Client CRUD endpoints (Actix-web)
2. `src/handlers/client_registration.rs` - DCR endpoints (Axum) ✅
3. `src/handlers/dcr_admin.rs` - DCR admin operations
4. `src/services/oidc_client_store.rs` - Client storage service
5. `src/services/client_registration.rs` - DCR business logic
6. `src/models/oidc_client.rs` - Client data model
7. `src/models/client_registration.rs` - DCR request/response models

**Status**:

- ✅ DCR handlers already use Axum (modern)
- ❌ OIDC client handlers use Actix-web (legacy)
- ✅ Business logic exists in services
- ❌ Models not in authenc-types yet

### NEW Architecture (`crates/`)

**Current State**:

- ✅ `crates/core/src/services/client_registration.rs` - EXISTS (needs storage ops)
- ✅ `crates/storage/src/operations/client_registration_ops.rs` - EXISTS (commented out)
- ❌ `crates/api/src/handlers/client.rs` - MISSING
- ❌ `crates/api/src/handlers/client_registration.rs` - MISSING
- ❌ `crates/types/src/domain/oauth2.rs` - INCOMPLETE (missing types)

---

## Migration Strategy

### Phase 1: Types Migration (30 minutes)

**Goal**: Migrate all Client Management types to `authenc-types`

**Files to Create/Update**:

1. **`crates/types/src/domain/oauth2.rs`**
   - Add `OAuth2Client` struct
   - Add `ClientMetadata` struct
   - Add `ClientType` enum (confidential, public, bearer-only)
   - Add `GrantType` enum
   - Add `ResponseType` enum

2. **`crates/types/src/domain/client_registration.rs`**
   - Add `ClientRegistrationRequest`
   - Add `ClientRegistrationResponse`
   - Add `ClientUpdateRequest`
   - Add `ClientRegistrationError`
   - Add `SoftwareStatement`
   - Add `InitialAccessToken`
   - Add `RegistrationAccessToken`

3. **`crates/types/src/domain/mod.rs`**
   - Export `oauth2` module
   - Export `client_registration` module

**Source Files** (to copy from):

- `src/models/oidc_client.rs`
- `src/models/client_registration.rs`

---

### Phase 2: Storage Operations Migration (45 minutes)

**Goal**: Enable and fix storage operations for Client Management

**Files to Fix**:

1. **`crates/storage/src/operations/client_registration_ops.rs`**
   - Fix imports to use `authenc_types` instead of `crate::models`
   - Update `OAuth2Client` references
   - Update `ClientRegistrationToken` references
   - Add missing CRUD operations

2. **`crates/storage/src/operations/mod.rs`**
   - Uncomment `pub mod client_registration_ops;`
   - Uncomment `pub use client_registration_ops as client_registration;`

3. **`crates/storage/src/operations/protocol_mappers_ops.rs`**
   - Fix imports to use `authenc_types`
   - Update `ProtocolMapper` references

**Operations Needed**:

- `create_client()`
- `get_client_by_id()`
- `get_client_by_client_id()`
- `update_client()`
- `delete_client()`
- `list_clients()`
- `create_registration_token()`
- `validate_registration_token()`
- `revoke_registration_token()`

---

### Phase 3: Core Services Migration (30 minutes)

**Goal**: Fix and complete core services for Client Management

**Files to Fix**:

1. **`crates/core/src/services/client_registration.rs`**
   - Uncomment storage operations import
   - Fix type references to use `authenc_types`
   - Ensure all methods compile

2. **Create `crates/core/src/services/client_service.rs`**
   - Client CRUD business logic
   - Client validation
   - Client policy enforcement
   - Scope management

3. **`crates/core/src/services/mod.rs`**
   - Add `pub mod client_service;`
   - Export `ClientService`

---

### Phase 4: API Handlers Migration (1 hour)

**Goal**: Create Axum handlers for Client Management endpoints

**Files to Create**:

1. **`crates/api/src/handlers/client.rs`**

   ```rust
   // Client CRUD endpoints
   GET    /api/v1/clients          - List all clients
   POST   /api/v1/clients          - Create new client
   GET    /api/v1/clients/{id}     - Get client by ID
   PUT    /api/v1/clients/{id}     - Update client
   DELETE /api/v1/clients/{id}     - Delete client
   GET    /api/v1/clients/{id}/secret - Get client secret
   POST   /api/v1/clients/{id}/secret - Regenerate client secret
   ```

2. **`crates/api/src/handlers/client_registration.rs`**

   ```rust
   // DCR endpoints (RFC 7591/7592)
   POST   /register                - Register new client (RFC 7591)
   GET    /register/{client_id}    - Get client config (RFC 7592)
   PUT    /register/{client_id}    - Update client config (RFC 7592)
   DELETE /register/{client_id}    - Delete client (RFC 7592)
   ```

3. **`crates/api/src/handlers/mod.rs`**
   - Add `pub mod client;`
   - Add `pub mod client_registration;`

4. **`crates/api/src/lib.rs`**
   - Add routes to main router
   - Add authentication middleware

**Source Files** (to migrate from):

- `src/handlers/oidc_client.rs` → `crates/api/src/handlers/client.rs`
- `src/handlers/client_registration.rs` → `crates/api/src/handlers/client_registration.rs`

---

### Phase 5: Integration & Testing (30 minutes)

**Goal**: Ensure everything compiles and integrates properly

**Tasks**:

1. ✅ Verify `cargo check -p authenc-types` passes
2. ✅ Verify `cargo check -p authenc-storage` passes
3. ✅ Verify `cargo check -p authenc-core` passes
4. ✅ Verify `cargo check -p authenc-api` passes
5. ✅ Verify `cargo check --workspace` passes
6. ✅ Update `crates/api/src/lib.rs` to include new routes
7. ✅ Test compilation of main binary

---

## Detailed File Mapping

### Types Migration

| OLD File | NEW File | Status |
|----------|----------|--------|
| `src/models/oidc_client.rs` | `crates/types/src/domain/oauth2.rs` | ❌ To migrate |
| `src/models/client_registration.rs` | `crates/types/src/domain/client_registration.rs` | ❌ To migrate |

### Storage Migration

| OLD File | NEW File | Status |
|----------|----------|--------|
| `src/database/operations/client_registration.rs` | `crates/storage/src/operations/client_registration_ops.rs` | ⚠️ Exists, needs fix |
| `src/database/operations/protocol_mappers.rs` | `crates/storage/src/operations/protocol_mappers_ops.rs` | ⚠️ Exists, needs fix |

### Services Migration

| OLD File | NEW File | Status |
|----------|----------|--------|
| `src/services/oidc_client_store.rs` | `crates/core/src/services/client_service.rs` | ❌ To create |
| `src/services/client_registration.rs` | `crates/core/src/services/client_registration.rs` | ⚠️ Exists, needs fix |

### Handlers Migration

| OLD File | NEW File | Status |
|----------|----------|--------|
| `src/handlers/oidc_client.rs` | `crates/api/src/handlers/client.rs` | ❌ To create |
| `src/handlers/client_registration.rs` | `crates/api/src/handlers/client_registration.rs` | ❌ To create |
| `src/handlers/dcr_admin.rs` | `crates/api/src/handlers/client_admin.rs` | ❌ To create (optional) |

---

## Success Criteria

### Must Have (MVP)

1. ✅ All types migrated to `authenc-types`
2. ✅ Storage operations working
3. ✅ Core services compiling
4. ✅ API handlers created
5. ✅ Client CRUD endpoints working
6. ✅ DCR endpoints working (RFC 7591/7592)
7. ✅ Zero compilation errors in workspace

### Nice to Have (Future)

1. ⏳ Client policies implementation
2. ⏳ Client scopes management
3. ⏳ Client templates
4. ⏳ Client import/export
5. ⏳ Client analytics

---

## Execution Order

1. **Phase 1**: Types Migration (START HERE)
   - Migrate `OAuth2Client` and related types
   - Migrate `ClientRegistration` types
   - Export from `authenc-types`

2. **Phase 2**: Storage Operations
   - Fix `client_registration_ops.rs`
   - Fix `protocol_mappers_ops.rs`
   - Enable in `mod.rs`

3. **Phase 3**: Core Services
   - Fix `client_registration.rs`
   - Create `client_service.rs`
   - Export from `mod.rs`

4. **Phase 4**: API Handlers
   - Create `client.rs` handlers
   - Create `client_registration.rs` handlers
   - Add routes to router

5. **Phase 5**: Integration
   - Test compilation
   - Verify all endpoints
   - Update documentation

---

## Estimated Time

| Phase | Time | Cumulative |
|-------|------|------------|
| Phase 1: Types | 30 min | 30 min |
| Phase 2: Storage | 45 min | 1h 15min |
| Phase 3: Services | 30 min | 1h 45min |
| Phase 4: Handlers | 1 hour | 2h 45min |
| Phase 5: Integration | 30 min | 3h 15min |
| **Total** | **3h 15min** | - |

---

## Next Steps

**IMMEDIATE**: Start with Phase 1 - Types Migration

```bash
# Command to start
cd layanan/authenc
# Read OLD models
cat src/models/oidc_client.rs
cat src/models/client_registration.rs
# Create NEW types
# ... (detailed in Phase 1)
```

---

**Status**: 📋 READY TO START
**Priority**: 🔴 CRITICAL
**Assigned**: AI Agent
**Estimated Completion**: 3h 15min
