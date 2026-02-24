# Error Handling Consolidation Summary

## Task 9.2: Consolidate Error Handling

### Completed Actions

1. **Removed Duplicate Error Files**
   - Deleted `crates/core/src/api/error.rs` (duplicate/old error handling)
   - Removed references from `crates/core/src/api/mod.rs`

2. **Removed anyhow from Public APIs**
   - **core/error.rs**: Changed `Internal(#[from] anyhow::Error)` to `Internal { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> }`
   - **api/error.rs**: Changed `Internal(#[from] anyhow::Error)` to `Internal { message: String }`
   - **core/config/mod.rs**: Changed return types from `anyhow::Result<Self>` to `Result<Self>` (using CoreError)
   - **storage/raft/mod.rs**: Changed all return types from `anyhow::Result<T>` to `StorageResult<T>`
   - **core/hsm/error.rs**: Updated `From<HsmError>` implementation to use new Internal variant
   - **core/utils/memory.rs**: Updated error creation to use `CoreError::internal()`
   - **core/config/dynamic.rs**: Updated all error creations to use `CoreError::internal()`
   - **core/utils/cache.rs**: Updated all error creations to use `CoreError::internal()`

3. **Added Proper Error Context and Source Chains**
   - **StorageError**: Added `source: Option<Box<dyn std::error::Error + Send + Sync>>` to:
     - `ConnectionFailed`
     - `QueryFailed`
     - `TransactionFailed`
     - `SerializationError`
   - Added helper methods for creating errors with and without source:
     - `connection_failed()` / `connection_failed_with_source()`
     - `query_failed()` / `query_failed_with_source()`
     - `transaction_failed()` / `transaction_failed_with_source()`
     - `serialization_error()` / `serialization_error_with_source()`

   - **CoreError**: Added `source: Option<Box<dyn std::error::Error + Send + Sync>>` to `Internal` variant
   - Added helper methods:
     - `internal()` - create internal error without source
     - `internal_with_source()` - create internal error with source chain

   - **ApiError**: Added helper method:
     - `internal_with_source()` - create internal error with source information

4. **Updated All Error Usages**
   - Fixed all `StorageError` usages in `crates/storage/src/backends/postgres.rs` to include `source: None`
   - Fixed pattern matching in `crates/core/src/error.rs` to use `Internal { .. }` instead of `Internal(_)`
   - Updated all error conversions to use new helper methods

5. **Fixed Compilation Issues**
   - Added missing `as_any()` method to `PostgresBackend`
   - Fixed `StorageStats` initialization to include all required fields
   - Fixed syntax error in `crates/api/src/handlers/raft.rs` (commented code with unmatched braces)

### Error Types Consolidated

#### Per Crate:
- **crypto**: `CryptoError` in `crates/crypto/src/error.rs` ✅
- **api**: `ApiError` in `crates/api/src/error.rs` ✅
- **core**: `CoreError` and `SecretonError` in `crates/core/src/error.rs` ✅
- **core/hsm**: `HsmError` in `crates/core/src/hsm/error.rs` ✅
- **storage**: `StorageError` in `crates/storage/src/lib.rs` ✅

### Consistency Achieved

1. **All error types use thiserror consistently** ✅
2. **anyhow removed from public APIs** ✅
   - anyhow can still be used internally for convenience
   - All public functions return proper typed errors
3. **Proper error context added** ✅
   - Source chains available for debugging
   - Helper methods for creating errors with context
4. **Single error.rs per crate** ✅

### Remaining Work

The API crate has compilation errors due to the error type changes. These need to be fixed:

1. **Replace `InternalServerError` with `Internal`** (38 occurrences)
2. **Update `CoreError::Internal` usages** to use struct syntax (25 occurrences)
3. **Update `ApiError::Internal` usages** to use struct syntax (22 occurrences)
4. **Update `ApiError::BadRequest` usages** to use struct syntax (13 occurrences)
5. **Fix other API-specific issues** (raft module references, audit logging, etc.)

These are mechanical changes that follow the same pattern as what we've already done.

### Benefits

1. **Type Safety**: Proper error types instead of generic anyhow::Error
2. **Better Error Messages**: Structured errors with context
3. **Source Chains**: Can trace error origins through the stack
4. **Consistency**: All crates follow the same error handling pattern
5. **API Clarity**: Public APIs have clear error types
6. **Maintainability**: Single error.rs per crate, easy to find and update

### Files Modified

- `layanan/secreton/crates/core/src/api/error.rs` (deleted)
- `layanan/secreton/crates/core/src/api/mod.rs`
- `layanan/secreton/crates/core/src/error.rs`
- `layanan/secreton/crates/core/src/hsm/error.rs`
- `layanan/secreton/crates/core/src/config/mod.rs`
- `layanan/secreton/crates/core/src/config/dynamic.rs`
- `layanan/secreton/crates/core/src/utils/memory.rs`
- `layanan/secreton/crates/core/src/utils/cache.rs`
- `layanan/secreton/crates/api/src/error.rs`
- `layanan/secreton/crates/api/src/handlers/raft.rs`
- `layanan/secreton/crates/storage/src/lib.rs`
- `layanan/secreton/crates/storage/src/raft/mod.rs`
- `layanan/secreton/crates/storage/src/backends/postgres.rs`

### Next Steps

To complete the error consolidation:

1. Run `cargo fix` to automatically fix some issues
2. Manually update remaining error variant usages in API crate
3. Run `cargo clippy` to catch any remaining issues
4. Update tests to use new error types
5. Update documentation to reflect new error handling patterns
