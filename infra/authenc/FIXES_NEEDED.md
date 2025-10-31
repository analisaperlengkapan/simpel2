# Authenc Compilation Fixes Needed

## Fixed Issues ✅
1. **Prost version conflict** - Fixed by downgrading prost from 0.14.1 to 0.13.5 to match tonic 0.12
2. **Duplicate ValidationResult** - Fixed by renaming to CaptchaValidationResult and JwtValidationResult
3. **UserStoreTrait not exported** - Fixed by adding re-export in stores/mod.rs

## Remaining Issues (61 errors)

### 1. Schemars API Issues (E0433)
- **Location**: `src/services/kubernetes.rs:16`
- **Problem**: schemars 0.8 doesn't have `transform` and `generate` modules
- **Solution**: Either upgrade to schemars 1.0 or remove/refactor Kubernetes CRD code

### 2. Cache Trait Not in Scope (E0599)
- **Locations**: Multiple files using `redis_cache.get()`, `redis_cache.set()`, `redis_cache.delete()`
- **Problem**: Cache trait methods not in scope
- **Solution**: Add `use crate::services::cache::Cache;` to affected files:
  - `src/grpc/authenc_service.rs` (lines 119, 150, 206, 651, 706)

### 3. RwLock Pattern Mismatch (E0308)
- **Locations**: Multiple event_manager.write().await usages
- **Problem**: Tokio RwLock doesn't return Result, but code expects Result pattern
- **Solution**: Change from `if let Ok(mut em) = event_manager.write().await` to `let mut em = event_manager.write().await`
- **Affected files**:
  - `src/grpc/authenc_service.rs` (lines 157, 214, 286, 383, 486, 531, 546, 616, 716)

### 4. Type Annotation Issues (E0282)
- **Locations**: Various Vec type annotations needed
- **Files**: key_rotation.rs, captcha modules, clustering, compliance
- **Solution**: Add explicit type annotations where compiler can't infer

### 5. Health Service State Type Mismatch (E0308)
- **Location**: `src/grpc/health.rs:255`
- **Problem**: Passing `Arc<impl Future>` instead of `Arc<AppState>`
- **Solution**: Await the future before wrapping in Arc

### 6. gRPC Router Type Mismatch (E0308)
- **Location**: `src/grpc/mod.rs:118`
- **Problem**: Router with layers doesn't match expected Router<Identity> type
- **Solution**: Update return type to match actual type with layers

### 7. Serde Trait Bounds (E0277)
- **Location**: `src/grpc/authenc_service.rs:652, 704`
- **Problem**: CheckPermissionResponse doesn't implement Serialize/Deserialize
- **Solution**: Add `#[derive(Serialize, Deserialize)]` to proto-generated types or create wrapper types

## Quick Fix Priority

### High Priority (Blocks Compilation)
1. Fix Cache trait imports (simple, affects many files)
2. Fix RwLock pattern (simple, affects many files)
3. Fix Health Service state type
4. Fix gRPC Router type

### Medium Priority
5. Fix serde traits for proto types
6. Add type annotations

### Low Priority (Can be feature-gated)
7. Fix or disable Kubernetes operator code

## Recommended Approach

1. **Batch fix Cache imports**: Add `use crate::services::cache::Cache;` to all affected files
2. **Batch fix RwLock patterns**: Replace all `if let Ok(mut em) =` with `let mut em =`
3. **Fix proto serde**: Create wrapper types or add serde derives
4. **Fix type mismatches**: Update function signatures and return types
5. **Consider feature-gating**: Put Kubernetes operator behind a feature flag if not critical

## Commands to Run After Fixes

```bash
# Clean build
cargo clean

# Check compilation
cargo check --all-targets

# Run tests
cargo test --lib

# Build release
cargo build --release
```

## Notes

- The project uses Rust edition 2024 which may have stricter requirements
- Many warnings about unused variables can be fixed by prefixing with `_`
- Consider running `cargo clippy` after compilation succeeds for additional improvements
