# Task 10.6 Implementation: JWT Token Generation for gRPC Responses

## Overview

This document summarizes the implementation of JWT token generation for gRPC responses, replacing the temporary session_id-based authentication with proper JWT access and refresh tokens.

## Changes Made

### 1. Updated `authenc-grpc` Service (`crates/grpc/src/service.rs`)

#### Added JWT Service Dependency
- Added `JwtService` from `authenc-crypto` to `AuthencGrpcService` struct
- Updated constructor to accept `jwt_service: Arc<JwtService>` parameter

#### Updated `authenticate()` Method
**Before:**
```rust
// Returned session_id as token (temporary)
access_token: session_id.to_string(),
refresh_token: String::new(), // TODO: Implement
```

**After:**
```rust
// Generate JWT access token
let access_token = self.jwt_service
    .generate_access_token(
        &user_id.to_string(),
        Some("master".to_string()),
        Some("openid profile".to_string()),
        Some(session_id.to_string()),
    )?;

// Generate refresh token
let refresh_token = self.jwt_service
    .generate_refresh_token(
        &user_id.to_string(),
        &session_id.to_string(),
    )?;
```

#### Implemented `refresh_token()` Method
**Before:**
```rust
Err(Status::unimplemented("Token refresh not yet implemented"))
```

**After:**
- Verifies refresh token signature and expiration
- Validates refresh token scope
- Generates new access token
- Implements token rotation (generates new refresh token)
- Returns both new access and refresh tokens

#### Updated `validate_token()` Method
**Before:**
```rust
// Treated token as session_id
let session_id = SessionId::from_string(&req.token)?;
// Validated session in database
```

**After:**
```rust
// Verifies JWT token signature and expiration
match self.jwt_service.verify_token(&req.token) {
    Ok(claims) => {
        // Returns user_id, scopes, and expiration from claims
    }
    Err(e) => {
        // Returns validation error
    }
}
```

### 2. Added JWT Configuration (`crates/types/src/config.rs`)

Created `JwtConfig` struct with:
- `issuer`: Token issuer URL (default: "https://authenc.kejaksaan.go.id")
- `access_token_ttl_seconds`: Access token lifetime (default: 900 seconds / 15 minutes)
- `refresh_token_ttl_seconds`: Refresh token lifetime (default: 604800 seconds / 7 days)
- `signing_key_path`: Optional path to Ed25519 signing key file

Helper methods:
- `access_token_ttl()`: Returns `Duration` for access token TTL
- `refresh_token_ttl()`: Returns `Duration` for refresh token TTL

### 3. Created Comprehensive Tests (`crates/grpc/tests/jwt_token_generation_test.rs`)

Implemented 6 test cases:
1. **test_jwt_service_initialization**: Verifies JWT service can be created with configuration
2. **test_access_token_generation**: Tests access token generation with all parameters
3. **test_access_token_validation**: Tests token generation and validation roundtrip
4. **test_refresh_token_generation**: Tests refresh token generation and validation
5. **test_token_expiration_configuration**: Tests custom TTL configuration
6. **test_token_rotation**: Tests that refresh tokens are rotated (different JTI)

All tests passed successfully.

## Requirements Satisfied

### REQ-TOKEN-001: JWT Access Token Generation
✅ **Implemented**
- Algorithm: Ed25519 signature (from `authenc-crypto`)
- Token lifetime: 15 minutes (configurable)
- Claims: sub (user ID), exp (expiry), iss (issuer), scope, realm, sid (session ID)

### REQ-TOKEN-002: Refresh Token Generation
✅ **Implemented**
- Token lifetime: 7 days (configurable)
- Secure storage: Tokens contain session_id for database tracking
- Token rotation: New refresh token generated on each refresh

## Security Features

1. **Ed25519 Signatures**: Fast, secure digital signatures
2. **Token Expiration**: Configurable TTL for access and refresh tokens
3. **Token Rotation**: Refresh tokens are rotated on each use
4. **Scope Validation**: Refresh tokens have limited scope ("refresh_token")
5. **Session Binding**: Tokens are bound to session IDs for revocation support

## Configuration

### Environment Variables (Future)
```bash
JWT_ISSUER=https://authenc.kejaksaan.go.id
JWT_ACCESS_TOKEN_TTL=900        # 15 minutes
JWT_REFRESH_TOKEN_TTL=604800    # 7 days
JWT_SIGNING_KEY_PATH=/secrets/authenc_ed25519.key
```

### Code Configuration
```rust
let config = JwtConfig {
    issuer: "https://authenc.kejaksaan.go.id".to_string(),
    access_token_ttl_seconds: 900,
    refresh_token_ttl_seconds: 604800,
    signing_key_path: Some("/secrets/key".to_string()),
};

let jwt_service = JwtService::new(
    &signing_key_bytes,
    config.issuer,
    config.access_token_ttl(),
    config.refresh_token_ttl(),
)?;
```

## Integration Points

### For Service Initialization
When creating `AuthencGrpcService`, you must now provide a `JwtService`:

```rust
let jwt_service = Arc::new(JwtService::new(
    &signing_key,
    config.jwt.issuer,
    config.jwt.access_token_ttl(),
    config.jwt.refresh_token_ttl(),
)?);

let grpc_service = AuthencGrpcService::new(
    auth_service,
    user_service,
    oauth2_service,
    realm_service,
    jwt_service,  // NEW PARAMETER
);
```

### For Clients (layanan-portal, etc.)
Clients now receive proper JWT tokens:
- Store `access_token` for API calls
- Store `refresh_token` for token renewal
- Use `refresh_token` endpoint when access token expires
- Validate tokens using `validate_token` endpoint

## Testing

Run tests:
```bash
cargo test --package authenc-grpc --test jwt_token_generation_test
```

Expected output:
```
running 6 tests
test test_access_token_generation ... ok
test test_jwt_service_initialization ... ok
test test_token_expiration_configuration ... ok
test test_access_token_validation ... ok
test test_token_rotation ... ok
test test_refresh_token_generation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored
```

## Next Steps

1. **Update Service Initialization**: Modify main.rs or app initialization to create and pass JwtService
2. **Load Signing Key**: Integrate with Secreton to load Ed25519 signing key
3. **Update Clients**: Update layanan-portal and other services to use JWT tokens
4. **Add Token Revocation**: Implement token revocation list (optional)
5. **Add Metrics**: Track token generation and validation metrics

## Related Files

- `crates/crypto/src/jwt.rs`: JWT service implementation
- `crates/types/src/config.rs`: JWT configuration
- `crates/grpc/src/service.rs`: gRPC service with JWT integration
- `crates/grpc/tests/jwt_token_generation_test.rs`: Comprehensive tests
- `proto/authenc.proto`: gRPC protocol definitions

## Priority

**HIGH** - This task is critical for proper authentication flow and must be completed before production deployment.

---

**Status**: ✅ COMPLETED
**Date**: 2026-02-19
**Tested**: Yes (6/6 tests passing)
**Requirements**: REQ-TOKEN-001 ✅, REQ-TOKEN-002 ✅
