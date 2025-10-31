# gRPC Server with mTLS Support

This module provides gRPC server implementation for Secreton with mutual TLS (mTLS) support.

## Features

- **mTLS Support**: Mutual TLS authentication with client certificate verification
- **Flexible Configuration**: Support for both TLS and non-TLS modes
- **Metrics Integration**: TLS handshake and client certificate verification metrics
- **Production Ready**: Proper error handling and logging

## Usage

### Basic gRPC Server (No TLS)

```rust
use secreton_api::grpc::SecretonGrpcService;
use std::net::SocketAddr;
use std::sync::Arc;

let service = SecretonGrpcService::new(storage, transit);
let addr: SocketAddr = "0.0.0.0:9200".parse()?;

service.serve(addr).await?;
```

### gRPC Server with TLS

```rust
use secreton_api::grpc::{SecretonGrpcService, GrpcTlsConfig};
use std::path::PathBuf;
use std::net::SocketAddr;

// Create TLS configuration
let tls_config = GrpcTlsConfig::new(
    PathBuf::from("/path/to/server.crt"),
    PathBuf::from("/path/to/server.key"),
);

let service = SecretonGrpcService::new(storage, transit);
let addr: SocketAddr = "0.0.0.0:9200".parse()?;

service.serve_with_tls(addr, tls_config).await?;
```

### gRPC Server with mTLS (Client Certificate Verification)

```rust
use secreton_api::grpc::{SecretonGrpcService, GrpcTlsConfig};
use std::path::PathBuf;

// Create TLS configuration with client authentication
let tls_config = GrpcTlsConfig::new(
    PathBuf::from("/path/to/server.crt"),
    PathBuf::from("/path/to/server.key"),
)
.with_ca_cert(PathBuf::from("/path/to/ca.crt"))
.with_client_auth();

let service = SecretonGrpcService::new(storage, transit);
let addr: SocketAddr = "0.0.0.0:9200".parse()?;

service.serve_with_tls(addr, tls_config).await?;
```

## Configuration

### Config File (config/secreton.production.toml)

```toml
[grpc]
enabled = true
host = "0.0.0.0"
port = 9200
tls_enabled = true
tls_cert = "/opt/simpelv2/certs/secreton-grpc.crt"
tls_key = "/opt/simpelv2/certs/secreton-grpc.key"
tls_ca_cert = "/opt/simpelv2/certs/ca.crt"
require_client_auth = true
```

## Certificate Generation

### Generate Self-Signed Certificates for Testing

```bash
# Generate CA certificate
openssl req -x509 -newkey rsa:4096 -days 365 -nodes \
  -keyout ca.key -out ca.crt \
  -subj "/CN=Secreton CA"

# Generate server certificate
openssl req -newkey rsa:4096 -nodes \
  -keyout server.key -out server.csr \
  -subj "/CN=secreton.example.com"

openssl x509 -req -in server.csr -days 365 \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt

# Generate client certificate
openssl req -newkey rsa:4096 -nodes \
  -keyout client.key -out client.csr \
  -subj "/CN=client.example.com"

openssl x509 -req -in client.csr -days 365 \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out client.crt
```

## Metrics

The gRPC TLS implementation exposes the following Prometheus metrics:

- `grpc_tls_connections_total` - Total number of gRPC TLS connections
- `grpc_tls_handshakes_successful` - Successful TLS handshakes
- `grpc_tls_handshakes_failed` - Failed TLS handshakes
- `grpc_tls_client_cert_verifications_total` - Total client certificate verifications
- `grpc_tls_client_cert_verifications_failed` - Failed client certificate verifications
- `grpc_tls_success_rate` - TLS handshake success rate percentage

## Security Considerations

1. **Certificate Validation**: Always use valid certificates from a trusted CA in production
2. **Client Authentication**: Enable `require_client_auth` for production deployments
3. **Certificate Rotation**: Implement regular certificate rotation
4. **Private Key Protection**: Ensure private keys are properly protected with file permissions
5. **TLS Version**: The implementation uses TLS 1.2+ by default

## Testing

### Test TLS Connection with grpcurl

```bash
# Without client certificate
grpcurl -insecure \
  -d '{"service": "secreton"}' \
  localhost:9200 \
  secreton.v1.SecretonService/HealthCheck

# With client certificate (mTLS)
grpcurl \
  -cacert ca.crt \
  -cert client.crt \
  -key client.key \
  -d '{"service": "secreton"}' \
  localhost:9200 \
  secreton.v1.SecretonService/HealthCheck
```

## Error Handling

The TLS implementation provides detailed error messages for common issues:

- Certificate file not found
- Invalid certificate format
- CA certificate missing when client auth is required
- TLS handshake failures
- Client certificate verification failures

All errors are logged with appropriate context for debugging.

## Integration with Kubernetes

### Kubernetes Deployment with TLS

```yaml
apiVersion: v1
kind: Service
metadata:
  name: secreton-grpc
spec:
  ports:
  - name: grpc
    port: 9200
    targetPort: 9200
  selector:
    app: secreton

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secreton
spec:
  template:
    spec:
      containers:
      - name: secreton
        image: secreton:latest
        ports:
        - containerPort: 9200
          name: grpc
        volumeMounts:
        - name: tls-certs
          mountPath: /opt/simpelv2/certs
          readOnly: true
      volumes:
      - name: tls-certs
        secret:
          secretName: secreton-tls
```

### Create Kubernetes Secret for Certificates

```bash
kubectl create secret generic secreton-tls \
  --from-file=server.crt=server.crt \
  --from-file=server.key=server.key \
  --from-file=ca.crt=ca.crt \
  -n simpelv2-infra
```

## References

- [gRPC Authentication Guide](https://grpc.io/docs/guides/auth/)
- [Tonic TLS Documentation](https://docs.rs/tonic/latest/tonic/transport/struct.ServerTlsConfig.html)
- [Mutual TLS Best Practices](https://www.cloudflare.com/learning/access-management/what-is-mutual-tls/)
