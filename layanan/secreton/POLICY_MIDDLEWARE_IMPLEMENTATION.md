# Policy Check Middleware Implementation

## Overview

This document describes the implementation of task 6.2: "Integrate policies with authentication" for the Secreton comprehensive refactor.

## Implementation Summary

### 1. Policy Service Integration

**File**: `layanan/secreton/crates/api/src/services/mod.rs`

Added a policy service to the `ServiceContainer` to enable policy-based authorization across all API operations:

```rust
pub struct ServiceContainer {
    // ... existing fields ...

    /// Policy service for authorization
    pub policy: Arc<RwLock<PolicySet>>,
}
```

The policy service is initialized with default policies (empty by default) and can be loaded from storage on startup.

### 2. Request Context Enhancement

**File**: `layanan/secreton/crates/api/src/middleware.rs`

Enhanced the `RequestContext` struct to include policy names extracted from JWT claims:

```rust
pub struct RequestContext {
    // ... existing fields ...

    /// Policy names from JWT claims
    pub policy_names: Vec<String>,
}
```

### 3. Policy Name Extraction

**Function**: `extract_policy_names_from_claims()`

Extracts policy names from JWT claims metadata. Supports multiple field names:
- `policy_names` (comma-separated list)
- `policies` (comma-separated list)
- Falls back to role-based policies if no explicit policies are defined

### 4. Policy Check Middleware

**Function**: `policy_check_middleware()`

Core middleware that enforces policy-based authorization on all operations:

**Features**:
- Skips policy checks for system endpoints (health, version, metrics, seal-status, unseal, init)
- Maps HTTP methods to policy actions (GET→read, POST→create, PUT/PATCH→update, DELETE→delete)
- Evaluates policies using the PolicySet service
- Records policy evaluation metrics (evaluation time, decision)
- Logs policy decisions to audit log with full context
- Returns 403 Forbidden if policy denies access
- Returns 401 Unauthorized if no authentication context exists

**Policy Evaluation Context**:
- User ID and email
- User roles
- Client IP address
- Request ID
- MFA status (TODO: integrate with actual MFA verification)
- Timestamp

### 5. Helper Functions

**`map_method_to_action()`**: Maps HTTP methods to policy actions/capabilities
- GET → read
- POST → create
- PUT/PATCH → update
- DELETE → delete
- HEAD/OPTIONS → list

**`build_policy_context()`**: Builds policy evaluation context from request

**`record_policy_evaluation_metrics()`**: Records metrics for policy evaluations (placeholder for Prometheus integration)

**`log_policy_decision_to_audit()`**: Logs policy decisions to audit log with:
- Action performed
- Actor (user)
- Resource type and ID
- Decision (allow/deny)
- Policy names evaluated
- Request ID
- Status (Success/Denied)

### 6. JWT Claims Integration

**Function**: `extract_jwt_claims_from_token()`

Converts auth service token claims into JwtClaims for namespace validation:
- Extracts satker_code and wilayah_code from metadata
- Determines admin level from roles or metadata
- Preserves all original claims

**Function**: `determine_admin_level()`

Determines admin level from roles and metadata:
- Checks metadata for explicit admin_level
- Infers from role names (pusat, eselon, wilayah, satker)
- Defaults to Satker (most restrictive)

### 7. Integration Tests

**File**: `layanan/secreton/crates/api/tests/policy_middleware_tests.rs`

Comprehensive test suite covering:
- Policy allows matching paths
- Policy denies non-matching paths
- Deny overrides allow (policy precedence)
- Policy caching functionality
- Single wildcard matching (*)
- Double wildcard matching (**)

## Usage

### 1. Apply Middleware to Routes

```rust
use axum::{Router, middleware};
use secreton_api::middleware::policy_check_middleware;

let app = Router::new()
    .route("/v1/secret/data/*path", get(get_secret))
    .layer(middleware::from_fn_with_state(state.clone(), policy_check_middleware))
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
```

**Note**: Policy check middleware should be applied AFTER authentication middleware to ensure RequestContext is available.

### 2. Configure Policies in JWT

Include policy names in JWT claims metadata:

```json
{
  "sub": "user123",
  "name": "John Doe",
  "email": "john@example.com",
  "roles": ["user", "admin"],
  "metadata": {
    "policy_names": "read-secrets,write-secrets",
    "satker_code": "KJA001",
    "wilayah_code": "SUMUT",
    "admin_level": "satker"
  }
}
```

### 3. Define Policies

Policies are defined using the PolicySet structure:

```rust
use secreton_core::models::PolicyRule;
use secreton_core::services::policy::PolicySet;

let rules = vec![
    PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    },
    PolicyRule {
        effect: "deny".to_string(),
        action: "delete".to_string(),
        path: "secret/protected/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    },
];

let policy_set = PolicySet::new(rules);
```

## Audit Logging

All policy evaluations are logged to the audit system with:
- **Action**: "policy_evaluation"
- **Actor**: User ID
- **Resource Type**: "policy"
- **Resource ID**: Request path
- **Status**: Success (allowed) or Denied
- **Metadata**:
  - path: Request path
  - action: Policy action (read, create, update, delete)
  - decision: allow or deny
  - policy_names: Comma-separated list of evaluated policies
  - request_id: Unique request identifier

## Metrics

Policy evaluation metrics are recorded (placeholder for Prometheus integration):
- `secreton_policy_evaluations_total{decision="allowed|denied"}` - Total policy evaluations
- `secreton_policy_evaluation_duration_seconds` - Policy evaluation duration
- `secreton_policy_cache_hits_total` - Policy cache hits
- `secreton_policy_cache_misses_total` - Policy cache misses

## Security Considerations

1. **Default Deny**: If no policy matches, access is denied by default
2. **Deny Overrides Allow**: Explicit deny rules override allow rules (policy precedence)
3. **System Endpoints Whitelisted**: Health, version, metrics, and seal-related endpoints bypass policy checks
4. **Authentication Required**: Policy check requires valid authentication context
5. **Audit Trail**: All policy decisions are logged for compliance and security auditing
6. **Caching**: Policy evaluations are cached for 60 seconds to improve performance

## TODO

1. **Load Policies from Storage**: Currently policies are initialized empty. Need to load from storage on startup.
2. **Prometheus Integration**: Replace placeholder metrics with actual Prometheus metrics.
3. **MFA Status Integration**: Integrate actual MFA verification status in policy context.
4. **Policy Management API**: Implement REST/gRPC endpoints for policy CRUD operations (task 6.3).
5. **Policy Validation**: Add validation for policy syntax and path patterns.
6. **Policy Versioning**: Implement policy versioning for rollback capability.
7. **Policy Testing Endpoint**: Add endpoint to test policy evaluation without applying changes.

## Requirements Met

- ✅ **6.5**: Policy-based authorization implemented
- ✅ **7.1**: Audit logging for policy decisions
- ✅ **9.1**: Metrics for policy evaluations (placeholder)

## Files Modified

1. `layanan/secreton/crates/api/src/services/mod.rs` - Added policy service to ServiceContainer
2. `layanan/secreton/crates/api/src/middleware.rs` - Implemented policy check middleware
3. `layanan/secreton/crates/api/tests/policy_middleware_tests.rs` - Added integration tests

## Testing

Run the integration tests:

```bash
cd layanan/secreton
cargo test --package secreton-api --test policy_middleware_tests
```

## Next Steps

1. Complete task 6.1 (Policy evaluation engine enhancement) if not already done
2. Implement task 6.3 (Policy management API endpoints)
3. Implement task 6.4 (Policy tests)
4. Integrate with actual Prometheus metrics
5. Load policies from storage on startup
6. Add policy validation and testing endpoints
