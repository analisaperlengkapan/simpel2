//! Comprehensive Integration Tests for authenc-api (Task 8.5)
//!
//! **BLOCKER STATUS**: These tests are currently blocked by compilation errors in authenc-core.
//! Once authenc-core compiles successfully, these test stubs should be implemented.
//!
//! **Requirements Coverage**:
//! - REQ-AUTH-001: User authentication
//! - REQ-AUTH-002: Multi-factor authentication
//! - REQ-AUTH-005: Passwordless authentication (WebAuthn/Passkeys)
//! - REQ-WEBAUTHN-001 through REQ-WEBAUTHN-010: WebAuthn requirements
//! - REQ-OAUTH-001, REQ-OAUTH-002: OAuth2 flows
//! - REQ-OIDC-001: OpenID Connect
//! - REQ-TOKEN-001 through REQ-TOKEN-004: Token management
//! - REQ-SEC-006: Rate limiting
//! - REQ-SEC-007: CORS
//! - REQ-PERF-001: Authentication latency <100ms (p99)
//! - REQ-TEST-001: Unit tests
//! - REQ-TEST-002: Integration tests

#![allow(unused_imports, dead_code)]

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// SECTION 1: AUTHENTICATION FLOW TESTS (REQ-AUTH-001, REQ-AUTH-003)
// ============================================================================

/// Test 1.1: End-to-end authentication flow with username/password
///
/// **Requirements**: REQ-AUTH-001 (username/password authentication)
///
/// **Test Flow**:
/// 1. Create test app with mock authentication service
/// 2. Send POST /api/v1/auth/login with valid credentials
/// 3. Verify response contains access_token and refresh_token
/// 4. Verify tokens are valid JWTs with correct claims
/// 5. Verify session is created in database
///
/// **Expected Behavior**:
/// - Status: 200 OK
/// - Response body: { "access_token": "...", "refresh_token": "...", "token_type": "Bearer", "expires_in": 900 }
/// - Access token expires in 15 minutes (REQ-TOKEN-001)
/// - Refresh token expires in 7 days (REQ-TOKEN-002)
///
/// **Mock Data**:
/// ```rust
/// let credentials = LoginRequest {
///     username: "testuser".to_string(),
///     password: "SecurePass123!".to_string(),
/// };
/// ```
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_authentication_flow_end_to_end() {
    // TODO: Implement once authenc-core compiles
    // 1. Create mock AuthenticationServiceImpl
    // 2. Create test router with ApiState
    // 3. Send login request
    // 4. Verify JWT structure and claims
    // 5. Verify session in database
}

/// Test 1.2: Authentication with invalid credentials
///
/// **Requirements**: REQ-AUTH-001, REQ-AUTH-003 (brute force protection)
///
/// **Test Flow**:
/// 1. Send POST /api/v1/auth/login with invalid password
/// 2. Verify response is 401 Unauthorized
/// 3. Verify brute force counter increments
/// 4. After 3 failures, verify CAPTCHA is required
/// 5. After 5 failures, verify account is locked
///
/// **Expected Behavior**:
/// - Status: 401 Unauthorized
/// - Response: { "error": "invalid_credentials" }
/// - After 3 failures: { "error": "captcha_required", "captcha_challenge": "..." }
/// - After 5 failures: { "error": "account_locked", "lockout_duration": 900 }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_authentication_invalid_credentials() {
    // TODO: Test brute force protection
}
/// Test 1.3: Authentication with MFA required
///
/// **Requirements**: REQ-AUTH-002 (MFA), REQ-MFA-001, REQ-MFA-002
///
/// **Test Flow**:
/// 1. Create user with MFA enabled
/// 2. Send POST /api/v1/auth/login with valid credentials
/// 3. Verify response indicates MFA required
/// 4. Send POST /api/v1/auth/mfa/verify with TOTP code
/// 5. Verify response contains access_token and refresh_token
///
/// **Expected Behavior**:
/// - First response: { "mfa_required": true, "mfa_token": "...", "mfa_methods": ["totp", "webauthn"] }
/// - Second response: { "access_token": "...", "refresh_token": "..." }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_authentication_with_mfa() {
    // TODO: Test MFA flow
}

// ============================================================================
// SECTION 2: WEBAUTHN/PASSKEYS TESTS (REQ-AUTH-005, REQ-WEBAUTHN-*)
// ============================================================================

/// Test 2.1: WebAuthn passkey registration flow
///
/// **Requirements**: REQ-AUTH-005, REQ-WEBAUTHN-001
///
/// **Test Flow**:
/// 1. Authenticate user (get JWT)
/// 2. Send POST /api/v1/auth/webauthn/register/start
/// 3. Verify response contains WebAuthn challenge
/// 4. Mock browser WebAuthn API response
/// 5. Send POST /api/v1/auth/webauthn/register/finish with credential
/// 6. Verify credential is stored in database
/// 7. Verify credential counter is initialized to 0
///
/// **Expected Behavior**:
/// - Start response: { "challenge": "...", "rp": {...}, "user": {...}, "pubKeyCredParams": [...] }
/// - Finish response: { "credential_id": "...", "created_at": "..." }
/// - Database: credential stored with counter=0
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_registration_flow() {
    // TODO: Test passkey registration
}
/// Test 2.2: WebAuthn passkey authentication flow
///
/// **Requirements**: REQ-AUTH-005, REQ-WEBAUTHN-002
///
/// **Test Flow**:
/// 1. Create user with registered passkey
/// 2. Send POST /api/v1/auth/webauthn/authenticate/start with user_id
/// 3. Verify response contains WebAuthn challenge
/// 4. Mock browser WebAuthn API response (assertion)
/// 5. Send POST /api/v1/auth/webauthn/authenticate/finish
/// 6. Verify response contains access_token and refresh_token
/// 7. Verify credential counter incremented
/// 8. Verify last_used timestamp updated
///
/// **Expected Behavior**:
/// - Start response: { "challenge": "...", "allowCredentials": [...] }
/// - Finish response: { "access_token": "...", "refresh_token": "..." }
/// - Database: counter incremented, last_used updated
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_authentication_flow() {
    // TODO: Test passkey authentication
}

/// Test 2.3: WebAuthn usernameless authentication (discoverable credentials)
///
/// **Requirements**: REQ-AUTH-005, REQ-WEBAUTHN-002
///
/// **Test Flow**:
/// 1. Create user with discoverable credential
/// 2. Send POST /api/v1/auth/webauthn/authenticate/start WITHOUT user_id
/// 3. Verify response allows discoverable credentials
/// 4. Mock browser WebAuthn API response with user handle
/// 5. Send POST /api/v1/auth/webauthn/authenticate/finish
/// 6. Verify user is identified from credential
/// 7. Verify authentication succeeds
///
/// **Expected Behavior**:
/// - Start response: { "challenge": "...", "allowCredentials": [] } (empty for discoverable)
/// - Finish response: { "access_token": "...", "user_id": "..." }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_usernameless_authentication() {
    // TODO: Test usernameless authentication
}
/// Test 2.4: WebAuthn replay attack prevention
///
/// **Requirements**: REQ-WEBAUTHN-004, REQ-SEC-012
///
/// **Test Flow**:
/// 1. Authenticate with passkey (counter=1)
/// 2. Capture the assertion response
/// 3. Try to replay the same assertion
/// 4. Verify authentication fails
/// 5. Try to authenticate with counter=0 (rollback attack)
/// 6. Verify authentication fails
///
/// **Expected Behavior**:
/// - Replay attempt: 401 Unauthorized, { "error": "replay_detected" }
/// - Counter rollback: 401 Unauthorized, { "error": "counter_rollback_detected" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_replay_attack_prevention() {
    // TODO: Test replay attack prevention
}

/// Test 2.5: WebAuthn origin binding enforcement
///
/// **Requirements**: REQ-WEBAUTHN-008, REQ-SEC-011
///
/// **Test Flow**:
/// 1. Register passkey for origin "https://simpel.kejaksaan.go.id"
/// 2. Try to authenticate from different origin "https://evil.com"
/// 3. Verify authentication fails
/// 4. Verify error indicates origin mismatch
///
/// **Expected Behavior**:
/// - Status: 401 Unauthorized
/// - Response: { "error": "origin_mismatch", "expected": "https://simpel.kejaksaan.go.id", "actual": "https://evil.com" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_origin_binding() {
    // TODO: Test origin binding
}

/// Test 2.6: WebAuthn credential management
///
/// **Requirements**: REQ-WEBAUTHN-003
///
/// **Test Flow**:
/// 1. Authenticate user
/// 2. Send GET /api/v1/auth/webauthn/credentials
/// 3. Verify response lists all user's passkeys
/// 4. Send PATCH /api/v1/auth/webauthn/credentials/{id} to update nickname
/// 5. Verify nickname updated
/// 6. Send DELETE /api/v1/auth/webauthn/credentials/{id}
/// 7. Verify credential deleted
///
/// **Expected Behavior**:
/// - List: [{ "id": "...", "nickname": "...", "created_at": "...", "last_used": "..." }]
/// - Update: { "id": "...", "nickname": "My YubiKey" }
/// - Delete: 204 No Content
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_webauthn_credential_management() {
    // TODO: Test credential management
}
// ============================================================================
// SECTION 3: OAUTH2 FLOW TESTS (REQ-OAUTH-001, REQ-OAUTH-002)
// ============================================================================

/// Test 3.1: OAuth2 Authorization Code flow with PKCE
///
/// **Requirements**: REQ-OAUTH-001 (Authorization Code flow with PKCE)
///
/// **Test Flow**:
/// 1. Generate code_verifier and code_challenge
/// 2. Send GET /api/v1/oauth2/authorize with code_challenge
/// 3. Verify redirect to login (if not authenticated)
/// 4. Authenticate user
/// 5. Verify redirect to redirect_uri with authorization code
/// 6. Send POST /api/v1/oauth2/token with code and code_verifier
/// 7. Verify response contains access_token and refresh_token
/// 8. Verify authorization code is single-use
///
/// **Expected Behavior**:
/// - Authorize: 302 redirect to redirect_uri?code=...&state=...
/// - Token: { "access_token": "...", "refresh_token": "...", "token_type": "Bearer", "expires_in": 900 }
/// - Second token request with same code: 400 Bad Request
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_oauth2_authorization_code_flow_with_pkce() {
    // TODO: Test OAuth2 authorization code flow
}

/// Test 3.2: OAuth2 Client Credentials flow
///
/// **Requirements**: REQ-OAUTH-002 (Client Credentials flow)
///
/// **Test Flow**:
/// 1. Create confidential client with client_secret
/// 2. Send POST /api/v1/oauth2/token with grant_type=client_credentials
/// 3. Verify response contains access_token (no refresh_token)
/// 4. Verify token has client_id in claims
///
/// **Expected Behavior**:
/// - Status: 200 OK
/// - Response: { "access_token": "...", "token_type": "Bearer", "expires_in": 900 }
/// - No refresh_token (service-to-service)
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_oauth2_client_credentials_flow() {
    // TODO: Test client credentials flow
}
/// Test 3.3: OAuth2 Refresh Token flow with rotation
///
/// **Requirements**: REQ-OAUTH-003 (Refresh Token flow with rotation)
///
/// **Test Flow**:
/// 1. Authenticate and get access_token + refresh_token
/// 2. Wait for access_token to expire (or mock expiry)
/// 3. Send POST /api/v1/oauth2/token with grant_type=refresh_token
/// 4. Verify response contains NEW access_token and NEW refresh_token
/// 5. Verify old refresh_token is invalidated
/// 6. Try to use old refresh_token again
/// 7. Verify request fails
///
/// **Expected Behavior**:
/// - First refresh: { "access_token": "new_token", "refresh_token": "new_refresh" }
/// - Second refresh with old token: 401 Unauthorized, { "error": "invalid_grant" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_oauth2_refresh_token_rotation() {
    // TODO: Test refresh token rotation
}

// ============================================================================
// SECTION 4: TOKEN VALIDATION TESTS (REQ-TOKEN-003, REQ-TOKEN-004)
// ============================================================================

/// Test 4.1: JWT token validation (valid token)
///
/// **Requirements**: REQ-TOKEN-003 (token validation)
///
/// **Test Flow**:
/// 1. Generate valid JWT with JwtService
/// 2. Send POST /api/v1/auth/validate with token
/// 3. Verify response indicates token is valid
/// 4. Verify claims are returned
///
/// **Expected Behavior**:
/// - Status: 200 OK
/// - Response: { "valid": true, "user_id": "...", "scope": "...", "exp": ... }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_token_validation_valid() {
    // TODO: Test valid token validation
}

/// Test 4.2: JWT token validation (expired token)
///
/// **Requirements**: REQ-TOKEN-003
///
/// **Test Flow**:
/// 1. Generate expired JWT
/// 2. Send POST /api/v1/auth/validate
/// 3. Verify response indicates token is invalid
///
/// **Expected Behavior**:
/// - Status: 401 Unauthorized
/// - Response: { "valid": false, "error": "token_expired" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_token_validation_expired() {
    // TODO: Test expired token
}
/// Test 4.3: JWT token validation (invalid signature)
///
/// **Requirements**: REQ-TOKEN-003
///
/// **Test Flow**:
/// 1. Generate JWT with wrong signing key
/// 2. Send POST /api/v1/auth/validate
/// 3. Verify response indicates token is invalid
///
/// **Expected Behavior**:
/// - Status: 401 Unauthorized
/// - Response: { "valid": false, "error": "invalid_signature" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_token_validation_invalid_signature() {
    // TODO: Test invalid signature
}

/// Test 4.4: Token revocation
///
/// **Requirements**: REQ-TOKEN-004 (token revocation)
///
/// **Test Flow**:
/// 1. Generate valid JWT
/// 2. Send POST /api/v1/auth/revoke with token
/// 3. Verify token is added to revocation list
/// 4. Try to use revoked token
/// 5. Verify request fails
///
/// **Expected Behavior**:
/// - Revoke: 200 OK
/// - Use revoked token: 401 Unauthorized, { "error": "token_revoked" }
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_token_revocation() {
    // TODO: Test token revocation
}

// ============================================================================
// SECTION 5: RATE LIMITING TESTS (REQ-SEC-006)
// ============================================================================

/// Test 5.1: Rate limiting enforcement
///
/// **Requirements**: REQ-SEC-006 (rate limiting)
///
/// **Test Flow**:
/// 1. Send 100 requests from same IP within 1 minute
/// 2. Verify first 100 requests succeed
/// 3. Verify 101st request fails with 429 Too Many Requests
/// 4. Verify Retry-After header is present
/// 5. Wait for rate limit window to reset
/// 6. Verify requests succeed again
///
/// **Expected Behavior**:
/// - First 100 requests: 200 OK
/// - 101st request: 429 Too Many Requests, { "error": "rate_limit_exceeded", "retry_after": 60 }
/// - Headers: X-RateLimit-Limit: 100, X-RateLimit-Remaining: 0, Retry-After: 60
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_rate_limiting() {
    // TODO: Test rate limiting
}
/// Test 5.2: Adaptive rate limiting based on risk score
///
/// **Requirements**: REQ-SEC-006
///
/// **Test Flow**:
/// 1. Send requests with suspicious patterns (rapid failures)
/// 2. Verify rate limit is reduced dynamically
/// 3. Send requests with normal patterns
/// 4. Verify rate limit returns to normal
///
/// **Expected Behavior**:
/// - Suspicious activity: rate limit reduced to 10 req/min
/// - Normal activity: rate limit restored to 100 req/min
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_adaptive_rate_limiting() {
    // TODO: Test adaptive rate limiting
}

// ============================================================================
// SECTION 6: CORS TESTS (REQ-SEC-007)
// ============================================================================

/// Test 6.1: CORS preflight request
///
/// **Requirements**: REQ-SEC-007 (CORS)
///
/// **Test Flow**:
/// 1. Send OPTIONS request with Origin header
/// 2. Verify CORS headers are present
/// 3. Verify allowed origins are enforced
/// 4. Try with unauthorized origin
/// 5. Verify CORS headers are NOT present
///
/// **Expected Behavior**:
/// - Authorized origin: Access-Control-Allow-Origin: https://portal.simpel.kejaksaan.go.id
/// - Unauthorized origin: No CORS headers
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_cors_preflight() {
    // TODO: Test CORS preflight
}

/// Test 6.2: CORS with credentials
///
/// **Requirements**: REQ-SEC-007
///
/// **Test Flow**:
/// 1. Send request with Origin and credentials
/// 2. Verify Access-Control-Allow-Credentials: true
/// 3. Verify cookies are allowed
///
/// **Expected Behavior**:
/// - Headers: Access-Control-Allow-Credentials: true
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_cors_with_credentials() {
    // TODO: Test CORS with credentials
}
// ============================================================================
// SECTION 7: INTEGRATION TESTS (REQ-TEST-002)
// ============================================================================

/// Test 7.1: authenc-api → authenc-core integration
///
/// **Requirements**: REQ-TEST-002, REQ-ARCH-002
///
/// **Test Flow**:
/// 1. Create real ApiState with authenc-core services
/// 2. Test all handlers call correct services
/// 3. Verify service methods are invoked
/// 4. Verify responses match service outputs
///
/// **Expected Behavior**:
/// - All handlers successfully call core services
/// - No panics or errors
/// - Correct data flow from handler → service → storage
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_api_core_integration() {
    // TODO: Test authenc-api → authenc-core integration
    // This requires authenc-core to compile successfully
}

/// Test 7.2: authenc-api → authenc-webauthn integration
///
/// **Requirements**: REQ-TEST-002, REQ-ARCH-002
///
/// **Test Flow**:
/// 1. Create ApiState with WebAuthnService
/// 2. Test WebAuthn handlers call WebAuthnService
/// 3. Verify credential storage
/// 4. Verify authentication flow
///
/// **Expected Behavior**:
/// - WebAuthn handlers successfully call WebAuthnService
/// - Credentials stored in database
/// - Authentication succeeds with valid credentials
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_api_webauthn_integration() {
    // TODO: Test authenc-api → authenc-webauthn integration
}

/// Test 7.3: authenc-api → authenc-crypto integration
///
/// **Requirements**: REQ-TEST-002, REQ-ARCH-002
///
/// **Test Flow**:
/// 1. Create ApiState with JwtService
/// 2. Test JWT middleware validates tokens
/// 3. Test JWT generation in login handler
/// 4. Verify signature validation
///
/// **Expected Behavior**:
/// - JWT middleware correctly validates tokens
/// - Login handler generates valid JWTs
/// - Invalid tokens are rejected
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_api_crypto_integration() {
    // TODO: Test authenc-api → authenc-crypto integration
}
/// Test 7.4: End-to-end flow (Frontend → API → Core → Storage → Database)
///
/// **Requirements**: REQ-TEST-002, REQ-PERF-001
///
/// **Test Flow**:
/// 1. Set up complete stack: PostgreSQL → authenc-storage → authenc-core → authenc-api
/// 2. Mock frontend request (login)
/// 3. Trace request through all layers
/// 4. Verify data is stored in PostgreSQL
/// 5. Verify response is returned to frontend
/// 6. Measure latency (<100ms p99)
///
/// **Expected Behavior**:
/// - Request flows through all layers successfully
/// - User data stored in database
/// - JWT returned to frontend
/// - Latency <100ms (REQ-PERF-001)
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_end_to_end_flow() {
    // TODO: Test complete end-to-end flow
    // This is the most comprehensive integration test
}

/// Test 7.5: CORS integration with Portal microfrontend origin
///
/// **Requirements**: REQ-TEST-002, REQ-SEC-007, REQ-PORTAL-ARCH-002
///
/// **Test Flow**:
/// 1. Configure CORS for Portal origin (https://portal.simpel.kejaksaan.go.id)
/// 2. Send request from Portal origin
/// 3. Verify CORS headers allow the request
/// 4. Verify credentials are allowed
/// 5. Test with other microfrontend origins
///
/// **Expected Behavior**:
/// - Portal origin: CORS allowed
/// - Other authorized origins: CORS allowed
/// - Unauthorized origins: CORS blocked
#[tokio::test]
#[ignore = "Blocked by authenc-core compilation errors"]
async fn test_cors_portal_integration() {
    // TODO: Test CORS with Portal microfrontend
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a test router with mock services
///
/// **Blocked by**: authenc-core compilation errors
///
/// **Implementation Plan**:
/// 1. Create mock AuthenticationServiceImpl
/// 2. Create mock UserManagementServiceImpl
/// 3. Create mock OAuth2ServiceImpl
/// 4. Create mock WebAuthnService
/// 5. Create mock JwtService
/// 6. Create ApiState with all mocks
/// 7. Create router with ApiState
///
/// **Returns**: Axum Router ready for testing
#[allow(dead_code)]
fn create_test_router() -> axum::Router {
    todo!("Blocked by authenc-core compilation errors")
}
/// Create a mock JWT token for testing
///
/// **Blocked by**: authenc-crypto compilation
///
/// **Implementation Plan**:
/// 1. Load test Ed25519 key pair
/// 2. Create TokenClaims with test data
/// 3. Sign with JwtService
/// 4. Return JWT string
///
/// **Returns**: Valid JWT string
#[allow(dead_code)]
fn create_mock_jwt() -> String {
    todo!("Blocked by authenc-crypto compilation")
}

/// Create a mock WebAuthn credential for testing
///
/// **Blocked by**: authenc-webauthn compilation
///
/// **Implementation Plan**:
/// 1. Generate mock credential ID
/// 2. Generate mock public key
/// 3. Create credential with counter=0
/// 4. Store in test database
/// 5. Return credential ID
///
/// **Returns**: Credential ID string
#[allow(dead_code)]
fn create_mock_webauthn_credential() -> String {
    todo!("Blocked by authenc-webauthn compilation")
}

/// Create a mock OAuth2 client for testing
///
/// **Blocked by**: authenc-core compilation
///
/// **Implementation Plan**:
/// 1. Create OidcClient with test data
/// 2. Store in test database
/// 3. Return client_id and client_secret
///
/// **Returns**: (client_id, client_secret)
#[allow(dead_code)]
fn create_mock_oauth2_client() -> (String, String) {
    todo!("Blocked by authenc-core compilation")
}

/// Create a test database connection
///
/// **Blocked by**: authenc-storage compilation
///
/// **Implementation Plan**:
/// 1. Create in-memory PostgreSQL (or use testcontainers)
/// 2. Run migrations
/// 3. Return Database instance
///
/// **Returns**: Database connection
#[allow(dead_code)]
async fn create_test_database() -> authenc_storage::Database {
    todo!("Blocked by authenc-storage compilation")
}
