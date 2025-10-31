# gRPC Integration Guide

## Overview

Secreton now supports both REST and gRPC APIs running concurrently on the same server instance. Both APIs share the same backend state (Transit engine, KV engine, storage backend), ensuring consistency across all operations.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Secreton API Server                       │
│                                                              │
│  ┌──────────────────┐         ┌──────────────────┐         │
│  │   REST API       │         │   gRPC API       │         │
│  │   Port: 8200     │         │   Port: 9200     │         │
│  └────────┬─────────┘         └────────┬─────────┘         │
│           │                            │                    │
│           └────────────┬───────────────┘                    │
│                        │                                    │
│           ┌────────────▼────────────┐                       │
│           │   Shared State          │                       │
│           │  - Transit Engine       │                       │
│           │  - KV Engine            │                       │
│           │  - Storage Backend      │                       │
│           └─────────────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Configuration

### REST API Configuration

```toml
[server]
host = "0.0.0.0"
port = 8200
tls_enabled = true
tls_cert = "/opt/simpelv2/certs/secreton.crt"
tls_key = "/opt/simpelv2/certs/secreton.key"
```

### gRPC API Configuration

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

## Features

### Concurrent Operation

Both REST and gRPC servers run concurrently using Tokio's async runtime:

- **Independent Failure**: If one server fails, the other continues to operate
- **Shared State**: Both servers access the same Transit engine and storage backend
- **Consistent Data**: All operations are synchronized through shared Arc references

### mTLS Support

gRPC server supports mutual TLS authentication:

- **Server Authentication**: Server presents certificate to clients
- **Client Authentication**: Clients must present valid certificates (optional)
- **CA Verification**: Client certificates verified against configured CA

### Health Checks

Both servers provide health check endpoints:

- **REST**: `GET /health` - Returns JSON health status
- **gRPC**: `HealthCheck` RPC - Returns protobuf health status

Health checks verify:
- Storage backend connectivity
- Transit engine availability
- Overall system status

### Metrics

Prometheus-compatible metrics available at:

- **REST**: `GET /metrics` - Prometheus format
- **gRPC**: `GetMetrics` RPC - Structured metrics

Metrics include:
- Request counts (REST and gRPC)
- Operation counts (transit, KV)
- Active connections
- Uptime

## Kubernetes Deployment

### Service Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: secreton-service
  namespace: simpelv2-infra
spec:
  type: ClusterIP
  ports:
    - port: 8200
      targetPort: 8200
      protocol: TCP
      name: http
    - port: 9200
      targetPort: 9200
      protocol: TCP
      name: grpc
  selector:
    app.kubernetes.io/name: secreton
```

### Deployment Configuration

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secreton
  namespace: simpelv2-infra
spec:
  replicas: 1
  template:
    spec:
      containers:
        - name: secreton
          image: registry.kejaksaan.go.id/simpelv2/secreton:latest
          ports:
            - containerPort: 8200
              name: http
            - containerPort: 9200
              name: grpc
          livenessProbe:
            httpGet:
              path: /health
              port: 8200
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8200
            initialDelaySeconds: 10
            periodSeconds: 5
```

## Usage Examples

### REST API

```bash
# Health check
curl https://secreton-service:8200/health

# Create transit key
curl -X POST https://secreton-service:8200/v1/transit/keys/my-key \
  -H "Content-Type: application/json" \
  -d '{"type": "aes256-gcm"}'

# Encrypt data
curl -X POST https://secreton-service:8200/v1/transit/encrypt/my-key \
  -H "Content-Type: application/json" \
  -d '{"plaintext": "SGVsbG8gV29ybGQ="}'
```

### gRPC API

```rust
use tonic::Request;
use secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton::v1::{CreateKeyRequest, KeyType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SecretonServiceClient::connect("https://secreton-service:9200").await?;

    let request = Request::new(CreateKeyRequest {
        name: "my-key".to_string(),
        key_type: KeyType::Aes256Gcm as i32,
    });

    let response = client.create_key(request).await?;
    println!("Created key: {:?}", response);

    Ok(())
}
```

## Monitoring

### Prometheus Metrics

Add Secreton to Prometheus scrape configuration:

```yaml
scrape_configs:
  - job_name: 'simpelv2-secreton'
    static_configs:
      - targets: ['secreton-service.simpelv2-infra.svc.cluster.local:8200']
    metrics_path: /metrics
    scrape_interval: 30s
```

### Available Metrics

- `rest_requests_total` - Total REST API requests
- `grpc_requests_total` - Total gRPC API requests
- `transit_operations_total` - Total transit engine operations
- `kv_operations_total` - Total KV engine operations
- `active_connections` - Current active connections
- `uptime_seconds` - Server uptime in seconds

## Security Considerations

### TLS Configuration

- **Minimum TLS Version**: TLS 1.3 recommended
- **Certificate Rotation**: Implement automated certificate rotation
- **mTLS**: Enable client certificate authentication for gRPC in production

### Network Policies

Restrict access to Secreton services:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: secreton-network-policy
  namespace: simpelv2-infra
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: secreton
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: simpelv2-backend
      ports:
        - protocol: TCP
          port: 8200
        - protocol: TCP
          port: 9200
```

## Troubleshooting

### Both Servers Not Starting

Check logs for configuration errors:

```bash
kubectl logs -n simpelv2-infra deployment/secreton
```

Common issues:
- Missing TLS certificates
- Port already in use
- Invalid configuration

### gRPC Connection Failures

Verify mTLS configuration:

```bash
# Test gRPC connection
grpcurl -insecure secreton-service:9200 list

# Test with client certificate
grpcurl -cert client.crt -key client.key \
  -cacert ca.crt \
  secreton-service:9200 list
```

### Health Check Failures

Check storage backend connectivity:

```bash
# REST health check
curl https://secreton-service:8200/health

# Check storage backend
kubectl exec -n simpelv2-infra deployment/secreton -- \
  psql -h postgres-service -U simpelv2_secreton -d simpelv2_secreton -c "SELECT 1"
```

## Performance Tuning

### Connection Pooling

Configure storage backend connection pool:

```toml
[storage]
type = "postgresql"
max_connections = 50
connection_timeout = 10
```

### Concurrent Requests

Adjust worker threads for better concurrency:

```toml
[server]
workers = 4  # Number of worker threads
```

### Resource Limits

Set appropriate Kubernetes resource limits:

```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "250m"
  limits:
    memory: "1Gi"
    cpu: "1"
```

## Future Enhancements

- [ ] gRPC streaming support for large data transfers
- [ ] gRPC interceptors for authentication and authorization
- [ ] gRPC load balancing with Envoy
- [ ] gRPC health check probe binary for Kubernetes
- [ ] gRPC reflection for dynamic client discovery
- [ ] Bidirectional streaming for real-time updates

## References

- [gRPC Documentation](https://grpc.io/docs/)
- [Tonic Framework](https://github.com/hyperium/tonic)
- [Kubernetes gRPC Health Checks](https://kubernetes.io/blog/2018/10/01/health-checking-grpc-servers-on-kubernetes/)
- [Prometheus Metrics](https://prometheus.io/docs/concepts/metric_types/)
