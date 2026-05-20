# Federation gRPC Operations

This document describes the federation gRPC RPCs for SSO and external identity provider integration.

## Overview

The federation RPCs enable:

- **SSO Initiation**: Start federated authentication flow with external IdP
- **SSO Callback Handling**: Complete authentication and link/create user accounts
- **External IdP Integration**: Support for OIDC, SAML, and LDAP providers
- **Identity Brokering**: Link external identities to internal user accounts
- **Just-in-Time Provisioning**: Automatically create user accounts on first login

## Requirements

Implements the following requirements from the spec:

- **REQ-FED-001**: External identity provider support (OIDC, SAML, LDAP)
- **REQ-FED-002**: Identity brokering with account linking
- **REQ-FED-003**: SSO flows (SP-initiated, IdP-initiated, SLO)

## RPCs

### 1. InitiateFederatedAuth

Initiates federated authentication flow by generating an authorization URL for the specified identity provider.

**Request:**

```protobuf
message FederatedAuthRequest {
  string provider = 1;                  // Provider name (e.g., "google", "microsoft")
  optional string redirect_uri = 2;     // Custom redirect URI (optional)
  repeated string scopes = 3;           // Scopes to request (optional)
}
```

**Response:**

```protobuf
message FederatedAuthResponse {
  string auth_url = 1;  // Authorization URL to redirect user to
  string state = 2;     // State parameter for CSRF protection
}
```

**Example Usage:**

```rust
use authenc_grpc::proto::authenc::v1::authenc_service_client::AuthencServiceClient;
use authenc_grpc::proto::authenc::v1::FederatedAuthRequest;

let mut client = AuthencServiceClient::connect("https://authenc:50051").await?;

let request = FederatedAuthRequest {
    provider: "google".to_string(),
    redirect_uri: None,
    scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
};

let response = client.initiate_federated_auth(request).await?;

// Redirect user to response.auth_url
println!("Redirect to: {}", response.into_inner().auth_url);
```

**Flow:**

1. Client calls `InitiateFederatedAuth` with provider name
2. Authenc generates authorization URL with state parameter
3. Client redirects user to authorization URL
4. User authenticates with external IdP
5. IdP redirects back to callback URL with authorization code

**Error Cases:**

- `NOT_FOUND`: Provider not found
- `INVALID_ARGUMENT`: Invalid provider configuration
- `FAILED_PRECONDITION`: Provider is disabled

---

### 2. CompleteFederatedAuth

Completes federated authentication by exchanging authorization code for tokens and linking/creating user account.

**Request:**

```protobuf
message CompleteFederatedAuthRequest {
  string provider = 1;  // Provider name
  string code = 2;      // Authorization code from IdP
  string state = 3;     // State parameter for verification
}
```

**Response:**

```protobuf
message CompleteFederatedAuthResponse {
  string access_token = 1;   // Internal access token
  string refresh_token = 2;  // Internal refresh token
  UserInfo user = 3;         // User information
}
```

**Example Usage:**

```rust
use authenc_grpc::proto::authenc::v1::CompleteFederatedAuthRequest;

let request = CompleteFederatedAuthRequest {
    provider: "google".to_string(),
    code: "authorization_code_from_idp".to_string(),
    state: "state_from_initiate_call".to_string(),
};

let response = client.complete_federated_auth(request).await?;
let inner = response.into_inner();

// Use access_token for subsequent API calls
println!("Access token: {}", inner.access_token);
println!("User: {:?}", inner.user);
```

**Flow:**

1. Client receives authorization code from IdP callback
2. Client calls `CompleteFederatedAuth` with code and state
3. Authenc verifies state parameter (CSRF protection)
4. Authenc exchanges code for tokens with external IdP
5. Authenc retrieves user info from external IdP
6. Authenc links or creates user account (just-in-time provisioning)
7. Authenc generates internal access and refresh tokens
8. Client receives tokens and user info

**Error Cases:**

- `NOT_FOUND`: Provider not found
- `UNAUTHENTICATED`: Invalid authorization code
- `INVALID_ARGUMENT`: State verification failed
- `INTERNAL`: Token exchange failed

---

## Identity Provider Configuration

### OIDC Provider

```rust
use authenc_federation::service::{
    IdentityProviderConfig, IdentityProviderType,
    ProviderConfig, OidcProviderConfig,
};

let config = IdentityProviderConfig {
    id: Uuid::new_v4(),
    name: "google".to_string(),
    provider_type: IdentityProviderType::OIDC,
    config: ProviderConfig::OIDC(OidcProviderConfig {
        authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
        userinfo_endpoint: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
        client_id: "your_client_id".to_string(),
        client_secret: "your_client_secret".to_string(),
        redirect_uri: "https://authenc.example.com/callback".to_string(),
        scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
    }),
    enabled: true,
};

federation_service.register_provider(config).await?;
```

### SAML Provider

```rust
let config = IdentityProviderConfig {
    id: Uuid::new_v4(),
    name: "corporate-saml".to_string(),
    provider_type: IdentityProviderType::SAML,
    config: ProviderConfig::SAML(SamlProviderConfig {
        entity_id: "https://idp.example.com/saml".to_string(),
        sso_url: "https://idp.example.com/saml/sso".to_string(),
        slo_url: Some("https://idp.example.com/saml/slo".to_string()),
        certificate: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".to_string(),
        acs_url: "https://authenc.example.com/saml/acs".to_string(),
    }),
    enabled: true,
};

federation_service.register_provider(config).await?;
```

---

## Security Considerations

### CSRF Protection

The federation flow uses state parameters to prevent CSRF attacks:

1. `InitiateFederatedAuth` generates a random 32-character state
2. State is included in authorization URL
3. IdP includes state in callback
4. `CompleteFederatedAuth` verifies state matches

**Implementation:**

```rust
// State generation (in FederationService)
fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .nth(idx)
                .unwrap()
        })
        .collect()
}
```

### Account Linking

When completing federated authentication, Authenc performs account linking:

1. **Check external_id**: If user with external_id exists, use that account
2. **Check email**: If user with email exists, link accounts
3. **Create new user**: If no match, create new user (just-in-time provisioning)

**Security:**

- Email verification required before linking
- User consent required for account linking
- Audit log for all account linking operations

### Token Security

- External tokens (from IdP) are NOT stored permanently
- Internal tokens (JWT) are generated with short expiration
- Refresh tokens enable long-lived sessions without storing external tokens

---

## Testing

### Unit Tests

```bash
cd crates/federation
cargo test
```

### Integration Tests

```bash
cd crates/grpc
cargo test federation_test
```

### Test Coverage

- ✅ OIDC provider registration
- ✅ SAML provider registration
- ✅ Initiate federated auth with OIDC
- ✅ Initiate federated auth with SAML
- ✅ Complete federated auth with authorization code
- ✅ Error handling for invalid providers
- ✅ Error handling for disabled providers
- ✅ State parameter generation and uniqueness
- ✅ URL encoding for special characters
- ✅ Custom scopes and redirect URIs

---

## Future Enhancements

### Phase 1 (Current)

- ✅ OIDC provider support
- ✅ SAML provider support (basic)
- ✅ Account linking
- ✅ Just-in-time provisioning

### Phase 2 (Future)

- [ ] LDAP/Active Directory integration
- [ ] Attribute mapping configuration
- [ ] Identity provider chaining
- [ ] Single Logout (SLO) support
- [ ] IdP-initiated SSO
- [ ] Advanced SAML features (encryption, signing)

### Phase 3 (Future)

- [ ] Social login providers (Facebook, Twitter, GitHub)
- [ ] Multi-factor authentication with external IdP
- [ ] Account unlinking
- [ ] Provider discovery (OpenID Connect Discovery)

---

## Related Documentation

- [Design Document](../../../.kiro/specs/authenc-portal-comprehensive-refactoring/design.md) - Federation architecture
- [Requirements Document](../../../.kiro/specs/authenc-portal-comprehensive-refactoring/requirements.md) - REQ-FED-001, REQ-FED-002, REQ-FED-003
- [AGENTS.md](../../../AGENTS.md) - Authenc conventions

---

**Last Updated:** 2026-02-19
**Status:** Implemented (LOW priority task 10.10)
**Requirements:** REQ-FED-001, REQ-FED-002, REQ-FED-003
