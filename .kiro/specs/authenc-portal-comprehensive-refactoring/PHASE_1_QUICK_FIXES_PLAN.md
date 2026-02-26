# Phase 1: Quick Fixes - Execution Plan

**Date**: 2026-02-20
**Goal**: Reduce errors from 34 → ~10 errors
**Time**: 15 minutes

---

## ✅ Step 1: Fix Import Paths (COMPLETED)

**Changes Made**:
```bash
# Fixed import paths from authenc_types::domain::X to authenc_types::X
sed 's/authenc_types::domain::UserId/authenc_types::UserId/g'
sed 's/authenc_types::domain::RoleId/authenc_types::RoleId/g'
sed 's/authenc_types::domain::RealmId/authenc_types::RealmId/g'
sed 's/authenc_types::domain::User/authenc_types::User/g'
sed 's/authenc_types::domain::Role/authenc_types::Role/g'
```

**Result**: 34 → 33 errors (-1)

---

## 🔄 Step 2: Remove Unused Imports (IN PROGRESS)

### Files to Clean:

1. **crates/core/src/services/token_exchange.rs**
   - Remove: `use crate::utils::crypto::jwt::verify_jwt_with_validation;`
   - Remove: `use crate::handlers::oauth2::{...};` (line 632)
   - Remove: `use crate::stores::audit_log_store::AuditLogStore;`
   - Remove: `use crate::services::jwt_validator::JwtValidator;`

2. **crates/core/src/services/enhanced_audit.rs**
   - Remove: `use crate::utils::{...};`

3. **crates/core/src/services/pg_audit_log_store.rs**
   - Remove: `use crate::stores::audit_log_store::AuditLogStore;`

4. **crates/core/src/services/elasticsearch_audit_log_sink.rs**
   - Remove: `use reqwest::Client;`
   - Add to Cargo.toml if actually used, or remove usage

5. **crates/core/src/services/cache_invalidation_listener.rs**
   - Remove: `use crate::events::{EventError, EventListener};`

6. **crates/core/src/services/cache_utils.rs**
   - Remove: `use lib_common::cache::{AsyncLruCache, CacheStats};`
   - Remove: `pub use lib_common::cache::{LruCache, ThreadSafeLruCache};`

7. **crates/core/src/services/key_rotation.rs**
   - Remove: `use crate::secreton_client::secreton_client::SecretonClient;`

8. **crates/core/src/spi/credential/mod.rs**
   - Remove: `use crate::utils::crypto::password::{hash_password, verify_password};`

9. **crates/core/src/config/mfa_fallback.rs**
   - Remove: `use crate::crypto::aes_gcm::AesGcmService;`

10. **crates/core/src/config/mod.rs**
    - Remove: `use crate::middleware::*;`

---

## 🔄 Step 3: Fix Feature-Gated Modules (PENDING)

### Files to Update:

**crates/core/src/services/mod.rs**:
```rust
// Add feature gates for optional modules
#[cfg(feature = "redis-cache")]
pub mod redis_cache;

#[cfg(feature = "kafka")]
pub mod kafka_event_listener;

#[cfg(feature = "kafka")]
pub mod kafka_audit_log_sink;

#[cfg(feature = "kafka")]
pub mod event_publisher;

#[cfg(feature = "aws")]
pub mod event_retention;

#[cfg(feature = "cache")]
pub mod in_memory_cache;

#[cfg(feature = "fips")]
pub mod fips;

#[cfg(feature = "ldap")]
pub mod ldap_federation;
```

**crates/core/src/services/cache/mod.rs**:
```rust
#[cfg(feature = "kafka")]
pub mod event_consumer;

#[cfg(feature = "kafka")]
pub mod invalidation;
```

---

## 📊 Expected Results

| Step | Action | Errors Before | Errors After | Reduction |
|------|--------|---------------|--------------|-----------|
| 1 | Fix import paths | 34 | 33 | -1 |
| 2 | Remove unused imports | 33 | ~15 | -18 |
| 3 | Fix feature-gated modules | ~15 | ~10 | -5 |
| **Total** | | **34** | **~10** | **-24** |

---

## 🎯 Next Steps After Phase 1

Once Phase 1 is complete (errors reduced to ~10), proceed to:

**Phase 2: Implement Storage Operations** (1 hour)
- Create `authenc-storage/src/operations/client_registration.rs`
- Create `authenc-storage/src/operations/protocol_mappers_ops.rs`
- Expected: ~10 → ~5 errors

**Phase 3: Fix Service Dependencies** (30 minutes)
- Update `client_registration.rs` to use new storage ops
- Update `protocol_mapper_service.rs` to use new storage ops
- Expected: ~5 → 0 errors

**Phase 4: Implement Client Management Handlers** (1 hour)
- Create client CRUD handlers in `crates/api`
- Create DCR endpoints in `crates/api`
- Complete client management feature

---

**Status**: Step 1 COMPLETE, Step 2 IN PROGRESS
**Last Updated**: 2026-02-20
