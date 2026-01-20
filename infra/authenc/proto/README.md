# Simpelv2 Protocol Buffers (gRPC)

Protocol Buffer definitions untuk komunikasi gRPC antar microservices Simpelv2.

## 📁 Structure

```
proto/
├── common.proto      # Shared types (HealthCheck, Pagination, etc.)
├── secreton.proto    # Secreton Vault service definitions
├── authenc.proto     # Authenc IAM service definitions
└── build.sh          # Code generation script
```

## 🔧 Proto Files

### 1. `common.proto`

Shared types yang digunakan oleh semua services:

**Types**:
- `HealthCheckRequest/Response` - Health check standard
- `Pagination` - Pagination untuk list operations
- `ErrorDetail` - Error details structure
- `Metadata` - Common metadata fields
- `RequestContext` - Request tracing context
- `HealthStatus` enum - HEALTHY, DEGRADED, UNHEALTHY

**Usage**: Import di service lain untuk menghindari duplikasi.

### 2. `secreton.proto`

Secreton Vault service untuk secret management:

**Services**:
- `SecretonService` - Main vault service

**Operations**:
- Secret Management: `StoreSecret`, `GetSecret`, `DeleteSecret`, `ListSecrets`
- Transit Engine: `Encrypt`, `Decrypt`, `Sign`, `Verify`
- Key Management: `CreateKey`, `RotateKey`
- Raft Cluster: `GetClusterStatus`, `AddNode`, `RemoveNode`
- Health: `HealthCheck`, `GetMetrics`

**Key Types**:
- `SecurityLevel` enum - PUBLIC, INTERNAL, CONFIDENTIAL, SECRET, TOP_SECRET
- `KeyType` enum - AES256_GCM, CHACHA20_POLY1305, ED25519, etc.

### 3. `authenc.proto`

Authenc IAM service untuk authentication & authorization:

**Services**:
- `AuthencService` - Main IAM service

**Operations**:
- Authentication: `Authenticate`, `ValidateToken`, `RefreshToken`, `RevokeToken`
- User Management: `CreateUser`, `GetUser`, `UpdateUser`, `DeleteUser`, `ListUsers`
- MFA: `EnableMFA`, `VerifyMFA`, `DisableMFA`
- Authorization: `CheckPermission`, `AssignRole`, `RevokeRole`
- OAuth2/OIDC: `GetOAuthToken`, `IntrospectToken`, `GetUserInfo`
- Federation: `InitiateFederatedAuth`, `CompleteFederatedAuth`
- Audit: `GetAuditLogs`, `GetComplianceReport`

**Key Types**:
- `TokenType` enum - ACCESS_TOKEN, REFRESH_TOKEN, ID_TOKEN
- `MFAMethod` enum - TOTP, SMS, EMAIL, WEBAUTHN, HARDWARE_KEY

## 🚀 Code Generation

### Prerequisites

```bash
# Install protoc compiler
sudo apt install protobuf-compiler

# Install Rust plugins
cargo install protoc-gen-tonic
```

### Generate Code

```bash
# Run build script
./build.sh

# Or manually for Rust
protoc --proto_path=. \
  --rust_out=generated/rust \
  --tonic_out=generated/rust \
  *.proto
```

### Generated Files

```
generated/
├── rust/
│   ├── common.rs
│   ├── secreton.rs
│   └── authenc.rs
├── go/          # Optional
└── python/      # Optional
```

## 💻 Usage in Rust

### Add Dependencies

```toml
[dependencies]
tonic = "0.10"
prost = "0.12"
tokio = { version = "1", features = ["full"] }
```

### Server Implementation

```rust
use tonic::{transport::Server, Request, Response, Status};
use secreton::v1::secreton_service_server::{SecretonService, SecretonServiceServer};
use secreton::v1::{GetSecretRequest, GetSecretResponse};

pub struct MySecretonService;

#[tonic::async_trait]
impl SecretonService for MySecretonService {
    async fn get_secret(
        &self,
        request: Request<GetSecretRequest>,
    ) -> Result<Response<GetSecretResponse>, Status> {
        let req = request.into_inner();
        
        // Your implementation
        let response = GetSecretResponse {
            id: "secret-123".to_string(),
            path: req.path,
            data: std::collections::HashMap::new(),
            version: 1,
            // ...
        };
        
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:9090".parse()?;
    let service = MySecretonService;
    
    Server::builder()
        .add_service(SecretonServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}
```

### Client Implementation

```rust
use secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton::v1::GetSecretRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SecretonServiceClient::connect("http://secreton:9090").await?;
    
    let request = tonic::Request::new(GetSecretRequest {
        path: "myapp/config".to_string(),
        version: None,
    });
    
    let response = client.get_secret(request).await?;
    println!("Secret: {:?}", response.into_inner());
    
    Ok(())
}
```

## 🔄 Service Communication

### Authenc → Secreton

```rust
// Authenc calls Secreton to get JWT signing key
use secreton::v1::secreton_service_client::SecretonServiceClient;

let mut secreton = SecretonServiceClient::connect("http://secreton-service:9090").await?;
let response = secreton.get_secret(GetSecretRequest {
    path: "auth/jwt-signing-key".to_string(),
    version: None,
}).await?;
```

### Portal → Authenc

```rust
// Portal calls Authenc for authentication
use authenc::v1::authenc_service_client::AuthencServiceClient;

let mut authenc = AuthencServiceClient::connect("http://authenc-service:9090").await?;
let response = authenc.authenticate(AuthenticateRequest {
    username: "user@example.com".to_string(),
    password: "password".to_string(),
    mfa_code: None,
    device_id: None,
    metadata: std::collections::HashMap::new(),
}).await?;
```

## 📊 Performance

### gRPC vs REST

| Metric | REST/JSON | gRPC/Protobuf | Improvement |
|--------|-----------|---------------|-------------|
| Latency | ~10ms | ~1-2ms | 5-10x faster |
| Throughput | ~10K req/s | ~100K req/s | 10x higher |
| Payload | ~500 bytes | ~100 bytes | 5x smaller |
| CPU | 100% | 30% | 70% reduction |

### Benefits

- ⚡ **Faster**: Binary serialization vs JSON
- 📦 **Smaller**: Compact binary format
- 🔒 **Type Safe**: Compile-time type checking
- 🔄 **Streaming**: Bi-directional streaming support
- 🛡️ **Built-in Auth**: mTLS with Istio

## 🔧 Best Practices

### 1. Use Common Types

Always import `common.proto` for shared types:

```protobuf
import "common.proto";

service MyService {
  rpc HealthCheck(common.v1.HealthCheckRequest) 
    returns (common.v1.HealthCheckResponse);
}
```

### 2. Versioning

Use package versioning:

```protobuf
package myservice.v1;  // v1, v2, etc.
```

### 3. Backward Compatibility

- Never change field numbers
- Use `optional` for new fields
- Use `reserved` for removed fields

```protobuf
message MyMessage {
  string name = 1;
  optional string email = 2;  // New field
  reserved 3;  // Removed field
  reserved "old_field_name";
}
```

### 4. Error Handling

Use `common.v1.ErrorDetail` for structured errors:

```rust
use tonic::Status;
use common::v1::ErrorDetail;

return Err(Status::invalid_argument(
    serde_json::to_string(&ErrorDetail {
        code: "INVALID_PATH".to_string(),
        message: "Secret path is invalid".to_string(),
        metadata: HashMap::new(),
    })?
));
```

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_secret() {
        let service = MySecretonService;
        let request = Request::new(GetSecretRequest {
            path: "test/secret".to_string(),
            version: None,
        });
        
        let response = service.get_secret(request).await;
        assert!(response.is_ok());
    }
}
```

### Integration Tests

```bash
# Use grpcurl for testing
grpcurl -plaintext localhost:9090 list
grpcurl -plaintext localhost:9090 secreton.v1.SecretonService/HealthCheck
```

## 📚 References

- [Protocol Buffers](https://developers.google.com/protocol-buffers)
- [gRPC](https://grpc.io/)
- [Tonic (Rust gRPC)](https://github.com/hyperium/tonic)
- [gRPC Best Practices](https://grpc.io/docs/guides/performance/)

## 🔄 Updates

When updating proto files:

1. Update the proto file
2. Run `./build.sh` to regenerate code
3. Update service implementations
4. Test thoroughly
5. Deploy with backward compatibility

## 📝 Changelog

- **2025-10-21**: Initial proto definitions
  - Created `common.proto` with shared types
  - Created `secreton.proto` with vault operations
  - Created `authenc.proto` with IAM operations
  - Removed duplicate HealthCheck definitions
  - Centralized HealthStatus enum in common.proto
