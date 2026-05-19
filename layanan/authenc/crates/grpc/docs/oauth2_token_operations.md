# OAuth2 Token Operations - gRPC Implementation

## Overview

This document describes the implementation of OAuth2 token operations in the Authenc gRPC service. These operations provide OAuth 2.1 compliant token management and OIDC UserInfo endpoint support.

## Implemented RPCs

### 1. GetOAuthToken

**Purpose**: Exchange authorization codes, refresh tokens, or client credentials for access tokens.

**Proto Definition**:

```protobuf
rpc GetOAuthToken(OAuthTokenRequest) returns (OAuthTokenResponse);

message OAuthTokenRequest {
  string grant_type = 1;
  optional string code = 2;
  optional string refresh_token = 3;
  optional string client_id = 4;
  optional string client_secret = 5;
  optional string redirect_uri = 6;
  repeated string scopes = 7;
}

message OAuthTokenResponse {
  string access_token = 1;
  string token_type = 2;
  int64 expires_in = 3;
  optional string refresh_token = 4;
  optional string id_token = 5;
  repeated string scopes = 6;
}
```

**Supported Grant Types**:

- `authorization_code` - Authorization Code flow with PKCE
- `refresh_token` - Refresh Token flow
- `client_credentials` - Client Credentials flow

**Implementation Details**:

- Integrates with `OAuth2ServiceImpl` from `authenc-core`
- Validates client credentials for confidential clients
- Enforces PKCE for authorization code flow (OAuth 2.1 requirement)
- Implements refresh token rotation for security
- Returns JWT access tokens with configurable expiration (default: 15 minutes)

**Example Usage**:

```rust
// Authorization Code flow
let request = OAuthTokenRequest {
    grant_type: "authorization_code".to_string(),
    code: Some("auth_code_123".to_string()),
    client_id: Some("my_client".to_string()),
    client_secret: Some("client_secret".to_string()),
    redirect_uri: Some("https://app.example.com/callback".to_string()),
    scopes: vec!["openid".to_string(), "profile".to_string()],
    ..Default::default()
};

let response = client.get_o_auth_token(request).await?;
// response.access_token: JWT access token
// response.refresh_token: Refresh token for token renewal
// response.expires_in: 900 (15 minutes)
```

**Error Handling**:

- `INVALID_ARGUMENT`: Invalid grant_type or missing required fields
- `UNAUTHENTICATED`: Invalid client credentials
- `PERMISSION_DENIED`: Client not authorized for requested scopes
- `NOT_FOUND`: Authorization code not found or expired
- `INTERNAL`: Token generation failed

---

### 2. IntrospectToken

**Purpose**: Validate and inspect access tokens (RFC 7662 - OAuth 2.0 Token Introspection).

**Proto Definition**:

```protobuf
rpc IntrospectToken(IntrospectTokenRequest) returns (IntrospectTokenResponse);

message IntrospectTokenRequest {
  string token = 1;
}

message IntrospectTokenResponse {
  bool active = 1;
  optional string user_id = 2;
  optional string client_id = 3;
  repeated string scopes = 4;
  optional int64 exp = 5;
  optional int64 iat = 6;
}
```

**Implementation Details**:

- Verifies JWT signature using Ed25519 public key
- Checks token expiration timestamp
- Extracts user_id, scopes, and timestamps from JWT claims
- Returns `active: false` for invalid/expired tokens (not an error)
- Does NOT revoke tokens (use `RevokeToken` RPC for that)

**Example Usage**:

```rust
let request = IntrospectTokenRequest {
    token: "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...".to_string(),
};

let response = client.introspect_token(request).await?;
if response.active {
    println!("Token is valid for user: {}", response.user_id.unwrap());
    println!("Scopes: {:?}", response.scopes);
    println!("Expires at: {}", response.exp.unwrap());
} else {
    println!("Token is invalid or expired");
}
```

**Security Considerations**:

- Token introspection does NOT require authentication (public endpoint)
- For production, consider adding client authentication
- Rate limiting should be applied to prevent abuse
- Tokens are validated cryptographically (signature verification)

---

### 3. GetUserInfo

**Purpose**: Retrieve user profile information using an access token (OIDC UserInfo endpoint).

**Proto Definition**:

```protobuf
rpc GetUserInfo(UserInfoRequest) returns (UserInfoResponse);

message UserInfoRequest {
  string access_token = 1;
}

message UserInfoResponse {
  string sub = 1;
  string email = 2;
  bool email_verified = 3;
  optional string name = 4;
  optional string picture = 5;
  map<string, string> additional_claims = 6;
}
```

**Implementation Details**:

- Validates access token signature and expiration
- Extracts user_id from token claims
- Fetches user profile from database
- Returns OIDC standard claims (sub, email, email_verified)
- Includes additional claims (realm, session_id) in `additional_claims` map

**Example Usage**:

```rust
let request = UserInfoRequest {
    access_token: "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...".to_string(),
};

let response = client.get_user_info(request).await?;
println!("User ID: {}", response.sub);
println!("Email: {}", response.email);
println!("Email verified: {}", response.email_verified);
println!("Realm: {}", response.additional_claims.get("realm").unwrap());
```

**OIDC Standard Claims**:

- `sub` (subject): User ID (UUID)
- `email`: User's email address
- `email_verified`: Whether email has been verified
- `name`: Full name (optional, not yet implemented)
- `picture`: Profile picture URL (optional, not yet implemented)

**Additional Claims**:

- `realm`: Realm ID from token
- `sid`: Session ID from token

**Error Handling**:

- `UNAUTHENTICATED`: Invalid or expired access token
- `NOT_FOUND`: User not found in database
- `INTERNAL`: Database query failed

---

## Integration with OAuth2ServiceImpl

The gRPC service delegates OAuth2 operations to `OAuth2ServiceImpl` from `authenc-core`:

```rust
pub struct AuthencGrpcService {
    oauth2_service: Arc<OAuth2ServiceImpl>,
    jwt_service: Arc<JwtService>,
    user_service: Arc<UserManagementServiceImpl>,
    // ...
}
```

**OAuth2ServiceImpl Methods Used**:

- `token(TokenRequest) -> TokenResponse` - Handle token requests
- `validate_redirect_uri()` - Validate redirect URIs
- `validate_scopes()` - Validate requested scopes
- `verify_pkce()` - Verify PKCE code challenge

**JwtService Methods Used**:

- `verify_token(token: &str) -> TokenClaims` - Verify JWT signature and expiration
- `generate_access_token()` - Generate JWT access tokens (used by OAuth2Service)

---

## Security Features

### OAuth 2.1 Compliance

- **PKCE Enforcement**: All authorization code flows require PKCE (code_challenge)
- **Refresh Token Rotation**: Old refresh tokens are revoked when new ones are issued
- **Single-Use Authorization Codes**: Codes are marked as used after exchange
- **Strict Redirect URI Matching**: Exact match required (no wildcards)

### Token Security

- **Ed25519 Signatures**: Cryptographically secure JWT signatures
- **Short-Lived Access Tokens**: 15-minute expiration (configurable)
- **Long-Lived Refresh Tokens**: 7-day expiration (configurable)
- **Token Revocation**: Refresh tokens can be revoked via `RevokeToken` RPC

### Client Authentication

- **Confidential Clients**: Require client_secret for token requests
- **Public Clients**: No client_secret required (PKCE mandatory)
- **Client Secret Hashing**: Secrets stored as Argon2id hashes (TODO: implement)

---

## Testing

### Unit Tests

Located in `tests/oauth2_token_test.rs`:

- `test_oauth_token_request_creation` - Authorization code flow
- `test_oauth_token_request_with_refresh_token` - Refresh token flow
- `test_oauth_token_request_with_client_credentials` - Client credentials flow
- `test_introspect_token_request_creation` - Token introspection
- `test_user_info_request_creation` - UserInfo endpoint

### Integration Tests

TODO: Add integration tests with real OAuth2Service and database

---

## Future Enhancements

### ID Token Generation (OIDC)

Currently, `id_token` field in `OAuthTokenResponse` is always `None`. Future implementation should:

- Generate ID tokens for `openid` scope
- Include standard OIDC claims (iss, aud, exp, iat, sub)
- Sign with Ed25519 (same as access tokens)

### Token Exchange (RFC 8693)

Support token exchange for delegation and impersonation:

- `grant_type: urn:ietf:params:oauth:grant-type:token-exchange`
- Exchange access tokens for different scopes or audiences
- Support delegation and impersonation use cases

### Pushed Authorization Requests (PAR)

Support PAR for enhanced security:

- `POST /oauth2/par` endpoint
- Return `request_uri` for use in authorization request
- Prevent authorization request tampering

### Client Secret Verification

Currently uses simple string comparison. Should implement:

- Argon2id hashing for client secrets
- Proper password hasher integration
- Secret rotation support

---

## Requirements Satisfied

This implementation satisfies the following requirements from the spec:

- **REQ-OAUTH-001**: Authorization Code flow with PKCE
- **REQ-OAUTH-002**: Client Credentials flow
- **REQ-OAUTH-003**: Refresh Token flow with rotation
- **REQ-OIDC-001**: UserInfo endpoint with standard claims

---

## Related Documentation

- [OAuth 2.1 Specification](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-10)
- [RFC 7662 - Token Introspection](https://datatracker.ietf.org/doc/html/rfc7662)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [RFC 7636 - PKCE](https://datatracker.ietf.org/doc/html/rfc7636)
- [RFC 8693 - Token Exchange](https://datatracker.ietf.org/doc/html/rfc8693)

---

**Last Updated**: 2026-02-19
**Author**: SIMPEL Team
**Status**: Implemented (Medium Priority)
