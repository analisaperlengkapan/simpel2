# Task 9.1: Admin Handlers Migration Plan

## Overview

Migrating 15 admin handler files from `src/handlers/` to `crates/iam-api/src/handlers/`

## Migration Status

### ✅ Completed (1/15)

1. **admin.rs** - Migrated with IamApiState integration
   - Location: `crates/iam-api/src/handlers/admin.rs`
   - Status: Skeleton implementation with proper state management
   - Routes: System stats, dashboard, users, roles, sessions, audit logs

### 🔄 In Progress (14/15)

2. **client_registration.rs** - Dynamic Client Registration (RFC 7591/7592)
   - Source: `src/handlers/client_registration.rs`
   - Target: `crates/iam-api/src/handlers/client_registration.rs`
   - Dependencies: AppState → IamApiState, client_service
   - Routes: POST/GET/PUT/DELETE /register/{client_id}

3. **dcr_admin.rs** - DCR Admin Management
   - Source: `src/handlers/dcr_admin.rs`
   - Target: `crates/iam-api/src/handlers/dcr_admin.rs`
   - Dependencies: AppState → IamApiState, client_service
   - Routes: Initial access tokens, policies, software statement issuers

4. **client_policy.rs** - Client Policy Management
   - Source: `src/handlers/client_policy.rs`
   - Target: `crates/iam-api/src/handlers/client_policy.rs`
   - Dependencies: PolicyHandlerState → IamApiState
   - Routes: Client policies, profiles, assignments

5. **federation_admin.rs** - Federation Admin
   - Source: `src/handlers/federation_admin.rs`
   - Target: `crates/iam-api/src/handlers/federation_admin.rs`
   - Dependencies: AppState → IamApiState, federation_service (TODO)
   - Routes: Identity providers, sync management, statistics

6. **jit_admin_service.rs** - Just-In-Time Admin
   - Source: `src/handlers/jit_admin_service.rs`
   - Target: `crates/iam-api/src/handlers/jit_admin.rs`
   - Dependencies: AppState → IamApiState
   - Routes: JIT provisioning configuration

7. **group.rs** - Group Management
   - Source: `src/handlers/group.rs`
   - Target: `crates/iam-api/src/handlers/groups.rs`
   - Dependencies: Database → IamApiState
   - Routes: Group CRUD, members, subgroups

8. **organization.rs** - Organization Management
   - Source: `src/handlers/organization.rs`
   - Target: `crates/iam-api/src/handlers/organizations.rs`
   - Dependencies: Database → IamApiState
   - Routes: Organization CRUD, hierarchy

9. **satker.rs** - Satker (Government Hierarchy)
   - Source: `src/handlers/satker.rs`
   - Target: `crates/iam-api/src/handlers/satker.rs`
   - Dependencies: Database → IamApiState
   - Routes: Satker CRUD, hierarchy, authorization

10. **audit.rs** - Audit Log Management
    - Source: `src/handlers/audit.rs`
    - Target: Update existing `crates/iam-api/src/handlers/audit.rs`
    - Dependencies: Database → IamApiState
    - Routes: Audit log queries, export

11. **spi_management.rs** - SPI Management
    - Source: `src/handlers/spi_management.rs`
    - Target: `crates/iam-api/src/handlers/spi_management.rs`
    - Dependencies: AppState → IamApiState
    - Routes: SPI plugin management

12. **spi_federation.rs** - SPI Federation
    - Source: `src/handlers/spi_federation.rs`
    - Target: `crates/iam-api/src/handlers/spi_federation.rs`
    - Dependencies: AppState → IamApiState
    - Routes: Federation SPI configuration

13. **uma.rs** - UMA 2.0 Management
    - Source: `src/handlers/uma.rs`
    - Target: `crates/iam-api/src/handlers/uma.rs`
    - Dependencies: AppState → IamApiState
    - Routes: UMA resources, policies, permissions

14. **zero_trust.rs** - Zero Trust Management
    - Source: `src/handlers/zero_trust.rs`
    - Target: `crates/iam-api/src/handlers/zero_trust.rs`
    - Dependencies: AppState → IamApiState
    - Routes: Zero trust policies, device trust

15. **oid4vc.rs** - OpenID for Verifiable Credentials
    - Source: `src/handlers/oid4vc.rs`
    - Target: `crates/iam-api/src/handlers/oid4vc.rs`
    - Dependencies: AppState → IamApiState
    - Routes: Verifiable credentials issuance

## Migration Strategy

### Phase 1: Simple Migrations (Handlers 6-10)

These handlers have straightforward dependencies and can be migrated quickly:

- jit_admin_service.rs
- group.rs
- organization.rs
- satker.rs
- audit.rs (update existing)

**Approach:**

1. Copy handler file to target location
2. Update imports: `crate::` → `authenc_types::`, `authenc_core::`, `authenc_storage::`
3. Replace `State<Arc<Database>>` with `State<Arc<IamApiState>>`
4. Update database access: `db.query()` → `state.user_service.method()`
5. Add to `handlers/mod.rs`

### Phase 2: Complex Migrations (Handlers 2-5, 11-15)

These handlers have complex dependencies or require service implementations:

- client_registration.rs (needs client_service methods)
- dcr_admin.rs (needs client_service methods)
- client_policy.rs (needs policy_service)
- federation_admin.rs (needs federation_service - TODO in IamApiState)
- spi_management.rs (needs SPI service)
- spi_federation.rs (needs SPI service)
- uma.rs (needs UMA service)
- zero_trust.rs (needs zero trust service)
- oid4vc.rs (needs OID4VC service)

**Approach:**

1. Identify missing services in IamApiState
2. Add service fields to IamApiState (with TODO comments if not implemented)
3. Migrate handler with service calls
4. Mark as NOT_IMPLEMENTED if service doesn't exist yet

## Import Mapping

### Old Imports → New Imports

```rust
// Old
use crate::database::Database;
use crate::app::AppState;
use crate::error::AuthencError;
use crate::models::user::User;
use crate::services::user_store::UserStore;

// New
use crate::state::IamApiState;
use authenc_types::error::{AuthencError, Result};
use authenc_types::domain::user::User;
use authenc_core::services::UserManagementServiceImpl;
```

## State Mapping

### Old State → New State

```rust
// Old
State(db): State<Arc<Database>>
State(state): State<Arc<AppState>>

// New
State(state): State<Arc<IamApiState>>
```

### Service Access

```rust
// Old
let user = db.query_one("SELECT * FROM users WHERE id = $1", &[&id]).await?;

// New
let user = state.user_service.get_user(id).await?;
```

## handlers/mod.rs Updates

After migration, update `crates/iam-api/src/handlers/mod.rs`:

```rust
pub mod admin;
pub mod audit;
pub mod client_policy;
pub mod client_registration;
pub mod clients;
pub mod dcr_admin;
pub mod federation;
pub mod federation_admin;
pub mod groups;
pub mod jit_admin;
pub mod oid4vc;
pub mod organizations;
pub mod realms;
pub mod roles;
pub mod satker;
pub mod spi_federation;
pub mod spi_management;
pub mod uma;
pub mod users;
pub mod zero_trust;

// Re-exports
pub use admin::*;
pub use audit::*;
pub use client_policy::*;
pub use client_registration::*;
pub use clients::*;
pub use dcr_admin::*;
pub use federation::*;
pub use federation_admin::*;
pub use groups::*;
pub use jit_admin::*;
pub use oid4vc::*;
pub use organizations::*;
pub use realms::*;
pub use roles::*;
pub use satker::*;
pub use spi_federation::*;
pub use spi_management::*;
pub use uma::*;
pub use users::*;
pub use zero_trust::*;
```

## Compilation Verification

After each handler migration:

```bash
cargo check --package authenc-iam-api
```

After all migrations:

```bash
cargo build --package authenc-iam-api
cargo test --package authenc-iam-api
cargo clippy --package authenc-iam-api
```

## Next Steps

1. **User Decision Required**: Due to the large scope (15 handlers), recommend:
   - Option A: Continue with automated migration (all 15 handlers)
   - Option B: Migrate in phases (Phase 1 first, then Phase 2)
   - Option C: Migrate only critical handlers (admin, groups, organizations, satker, audit)

2. **Missing Services**: Some handlers require services not yet in IamApiState:
   - federation_service (for federation_admin.rs)
   - policy_service (for client_policy.rs)
   - spi_service (for spi_management.rs, spi_federation.rs)
   - uma_service (for uma.rs)
   - zero_trust_service (for zero_trust.rs)
   - oid4vc_service (for oid4vc.rs)

3. **Recommendation**: Migrate Phase 1 handlers first (simple migrations), then assess if Phase 2 services need implementation before migration.

## Estimated Time

- Phase 1 (5 handlers): ~1 hour
- Phase 2 (9 handlers): ~2-3 hours
- Total: ~3-4 hours

## Success Criteria

- ✅ All 15 handler files migrated to `crates/iam-api/src/handlers/`
- ✅ All imports updated to use new crate structure
- ✅ All handlers use IamApiState instead of Database/AppState
- ✅ `handlers/mod.rs` updated with all exports
- ✅ `cargo check --package authenc-iam-api` passes
- ⚠️ Some handlers may return NOT_IMPLEMENTED if services don't exist yet
