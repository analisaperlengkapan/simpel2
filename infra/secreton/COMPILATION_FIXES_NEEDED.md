# Secreton Compilation Fixes Summary

## Critical Issues Found (264 errors, 84 warnings)

### 1. Metrics API Issues (Multiple locations)
**Problem**: Using old metrics API with integer literals instead of proper increment/set methods
**Locations**:
- `handlers/raft.rs:1149` - counter with literal `1`
- `grpc/server.rs:419, 488, 530, 531, 583, 584` - multiple counter calls

**Fix**: Replace `metrics::counter!("name", 1)` with `metrics::counter!("name").increment(1)`

### 2. Mutex Lock Unwrapping Issues
**Problem**: Not unwrapping Result from `.lock()` calls
**Locations**:
- `middleware.rs:41, 45, 53, 63, 354, 465` - cache and limiter guards
- Multiple other locations

**Fix**: Add `.expect("Lock poisoned")` or proper error handling

### 3. Missing Method `log_event` on AuditLogger
**Problem**: `state.audit.log_event()` method doesn't exist
**Location**: `handlers/raft.rs:1153`

**Fix**: Comment out or implement proper audit logging

### 4. HeaderValue Parse Issues
**Problem**: `.parse()` returns Result but code expects HeaderValue directly
**Locations**: `middleware.rs:364, 374, 524-538`

**Fix**: Add `.unwrap()` or `.expect()` to parse calls

### 5. Type Mismatches in Auth Service
**Problem**: Various type mismatches in auth.rs
- `roles` field expects `HashSet<String>` but gets `Vec<String>` (line 249)
- Missing `InternalError` variant in AuthError enum (lines 376, 386)

**Fix**: Convert Vec to HashSet, add missing error variant

### 6. Missing Fields in Structs
**Problem**: JwtClaims missing `permissions` and `roles` fields (line 165)
**Location**: `handlers/namespace.rs:165`

**Fix**: Add missing fields to struct initialization

### 7. Database Configuration Access
**Problem**: `config.database` field doesn't exist on ApiConfig
**Locations**: `services/mod.rs:288-292`

**Fix**: Update to use correct config structure

### 8. GRPC Server Issues
**Problem**: `ServiceContainer::new_mock` expects 2 arguments but only 1 provided
**Location**: `grpc/server.rs:36`

**Fix**: Add missing `pool` parameter

### 9. Response Type Mismatches
**Problem**: Response builder returns Result but code expects Response directly
**Locations**: `middleware.rs:618-630, 710`

**Fix**: Add `.unwrap()` or proper error handling

### 10. Claims Metadata Field Missing
**Problem**: `claims.metadata` field doesn't exist
**Locations**: `middleware.rs:792, 795, 796, 810, 858, 868`

**Fix**: Remove or update to use correct Claims structure

## Recommended Fix Strategy

1. **Phase 1**: Fix metrics API calls (quick wins)
2. **Phase 2**: Fix Mutex unwrapping issues
3. **Phase 3**: Fix type mismatches and missing fields
4. **Phase 4**: Fix database and config access
5. **Phase 5**: Address remaining warnings

## Estimated Effort
- High priority fixes: ~2-3 hours
- Medium priority: ~1-2 hours
- Low priority (warnings): ~30 minutes

Total: ~4-6 hours of careful, systematic fixes
