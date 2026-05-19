# authenc-grpc

gRPC service for Authenc identity provider.

## Purpose

This crate provides service-to-service gRPC API for backend services:

- **Authentication RPC**: Authenticate users from backend services
- **Token Validation RPC**: Validate JWT tokens
- **User Management RPCs**: Create, get, update users
- **mTLS Enforcement**: Secure service-to-service communication

## Proto Definition

The gRPC service is defined in `proto/authenc.proto`:

```protobuf
service AuthencService {
  rpc Authenticate(AuthenticateRequest) returns (AuthenticateResponse);
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  // ... and more
}
```

## Usage

### Server Setup

```rust
use authenc_grpc::{
    AuthencGrpcService, GrpcServerBuilder, GrpcServerConfig, TlsConfig,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service dependencies
    let auth_service = Arc::new(create_auth_service());
    let user_service = Arc::new(create_user_service());
    let mfa_service = Arc::new(create_mfa_service());
    let role_service = Arc::new(create_role_service());
    let jwt_service = Arc::new(create_jwt_service());

    // Create gRPC service
    let grpc_service = AuthencGrpcService::new(
        auth_service,
        user_service,
        mfa_service,
        role_service,
        jwt_service,
    );

    // Configure TLS (optional but recommended)
    let tls_config = TlsConfig::from_files(
        "/path/to/server.crt",
        "/path/to/server.key",
        Some("/path/to/ca.crt"), // For mTLS
    )?;

    // Configure server
    let config = GrpcServerConfig {
        bind_address: "0.0.0.0:50051".parse()?,
        tls_config: Some(tls_config),
        enable_logging: true,
        enable_auth: true,
    };

    // Start server
    GrpcServerBuilder::new(config)
        .with_service(grpc_service)
        .serve()
        .await?;

    Ok(())
}
```

### Client Usage (from other services)

```rust
use authenc_grpc::proto::authenc::v1::{
    authenc_service_client::AuthencServiceClient,
    AuthenticateRequest, ValidateTokenRequest,
};
use tonic::transport::{Channel, ClientTlsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure mTLS client
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca_cert))
        .identity(Identity::from_pem(client_cert, client_key));

    // Connect to Authenc gRPC service
    let channel = Channel::from_static("https://authenc.internal:50051")
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = AuthencServiceClient::new(channel);

    // Authenticate user
    let request = tonic::Request::new(AuthenticateRequest {
        username: "user@example.com".to_string(),
        password: "password123".to_string(),
        mfa_code: None,
        device_id: None,
        metadata: Default::default(),
        captcha_token: None,
    });

    let response = client.authenticate(request).await?;
    let auth_response = response.into_inner();

    println!("Access token: {}", auth_response.access_token);

    // Validate token
    let validate_request = tonic::Request::new(ValidateTokenRequest {
        token: auth_response.access_token.clone(),
        required_scopes: vec!["read:users".to_string()],
    });

    let validate_response = client.validate_token(validate_request).await?;
    let validation = validate_response.into_inner();

    if validation.valid {
        println!("Token is valid for user: {:?}", validation.user_id);
    }

    Ok(())
}
```

## Security

### mTLS Configuration

For production deployments, mTLS is **required**:

1. Generate server certificate and key
2. Generate client certificates for each service
3. Configure CA certificate for verification
4. Enable mTLS in server config

```rust
let tls_config = TlsConfig::from_files(
    "/certs/server.crt",
    "/certs/server.key",
    Some("/certs/ca.crt"), // Enables mTLS
)?;
```

### Certificate Rotation

Certificates should be rotated regularly:

1. Generate new certificates
2. Update TLS config
3. Restart gRPC server
4. Update client certificates

## Testing

Run integration tests:

```bash
cargo test --package authenc-grpc --test integration_test
```

Note: Some tests are marked with `#[ignore]` until mock services are implemented.

## Requirements

Implements requirements:

- REQ-API-003 (gRPC service)
- REQ-SEC-004 (mTLS enforcement)
- REQ-TEST-002 (Integration tests)

## Dependencies

- `tonic` 0.14.x - gRPC framework
- `prost` 0.14.x - Protocol Buffers
- `authenc-core` - Business logic
- `authenc-crypto` - Cryptographic operations
- `authenc-types` - Shared types

## Architecture

```
┌─────────────────────────────────────────┐
│         Backend Services                │
│  (layanan-portal, layanan-perlengkapan) │
└────────────────┬────────────────────────┘
                 │ gRPC (mTLS)
                 ▼
┌─────────────────────────────────────────┐
│         authenc-grpc                    │
│  ┌─────────────────────────────────┐   │
│  │  AuthencGrpcService             │   │
│  │  - Authenticate                 │   │
│  │  - ValidateToken                │   │
│  │  - CreateUser                   │   │
│  │  - GetUser                      │   │
│  └─────────────────────────────────┘   │
│                 │                       │
│                 ▼                       │
│  ┌─────────────────────────────────┐   │
│  │  Interceptors                   │   │
│  │  - Authentication               │   │
│  │  - Logging                      │   │
│  │  - Error Mapping                │   │
│  └─────────────────────────────────┘   │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│         authenc-core                    │
│  (Business Logic)                       │
└─────────────────────────────────────────┘
```

## See Also

- [authenc-api](../api/README.md) - REST API for microfrontends
- [authenc-iam-api](../iam-api/README.md) - Admin REST API
- [authenc-core](../core/README.md) - Business logic
- [Proto definitions](../../proto/README.md) - gRPC proto files
