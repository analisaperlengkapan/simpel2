# Simpelv2 gRPC Microservices Deployment

Deployment configuration untuk microservices Simpelv2 menggunakan MicroK8s dengan Istio service mesh dan Envoy proxy.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Istio Ingress Gateway                    │
│                  (Load Balancer + TLS)                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    Envoy Proxy                              │
│          (HTTP/gRPC Routing + Load Balancing)               │
└───────┬──────────────────┬──────────────────┬───────────────┘
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Authenc    │  │   Secreton   │  │    Portal    │
│   Service    │  │   Service    │  │   Service    │
│              │  │              │  │              │
│ HTTP: 8080   │  │ HTTP: 8200   │  │ HTTP: 8000   │
│ gRPC: 9090   │  │ gRPC: 9090   │  │              │
└──────────────┘  └──────────────┘  └──────────────┘
```

## 📦 Components

### 1. **Secreton** - Secret Management Service
- **Purpose**: Vault untuk secret management, encryption, key rotation
- **Protocols**: HTTP REST API + gRPC
- **Ports**: 
  - 8200 (HTTP)
  - 9090 (gRPC)
  - 8201 (Raft cluster)
- **Features**:
  - Transit engine (encryption as a service)
  - KV secrets storage
  - Raft consensus for HA
  - Key rotation

### 2. **Authenc** - IAM Service
- **Purpose**: Identity and Access Management
- **Protocols**: HTTP REST API + gRPC
- **Ports**:
  - 8080 (HTTP)
  - 9090 (gRPC)
- **Features**:
  - OAuth2/OIDC/SAML authentication
  - Multi-factor authentication
  - RBAC/ABAC authorization
  - Federation

### 3. **Portal** - Web Frontend
- **Purpose**: User interface (Rust WASM)
- **Protocol**: HTTP
- **Port**: 8000
- **Features**:
  - Admin dashboard
  - User management
  - Secret management UI

## 🚀 Deployment Steps

### Prerequisites

```bash
# Install MicroK8s
sudo snap install microk8s --classic

# Enable required addons
microk8s enable dns storage ingress istio

# Configure kubectl alias
alias kubectl='microk8s kubectl'
```

### 1. Setup Namespace

```bash
kubectl create namespace simpelv2
kubectl label namespace simpelv2 istio-injection=enabled
```

### 2. Deploy Services

```bash
# Deploy all services
kubectl apply -f deployments.yaml

# Verify deployments
kubectl get pods -n simpelv2
kubectl get services -n simpelv2
```

### 3. Configure Istio Gateway

```bash
# Apply Istio configuration
kubectl apply -f istio-gateway.yaml

# Verify gateway
kubectl get gateway -n simpelv2
kubectl get virtualservice -n simpelv2
kubectl get destinationrule -n simpelv2
```

### 4. Deploy Envoy Proxy (Optional)

```bash
# Apply Envoy configuration
kubectl apply -f envoy-config.yaml

# Verify Envoy
kubectl get configmap envoy-config -n simpelv2
```

### 5. Create Secrets

```bash
# Create Authenc secrets
kubectl create secret generic authenc-secrets \
  --from-literal=database-url='postgresql://user:pass@postgres:5432/authenc' \
  --from-literal=redis-url='redis://redis:6379' \
  --from-literal=jwt-secret='your-secret-key-here' \
  -n simpelv2

# Create TLS certificate
kubectl create secret tls simpelv2-tls-cert \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n simpelv2
```

## 🔧 Configuration

### Secreton Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: secreton-config
  namespace: simpelv2
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8200
    
    [storage]
    backend_type = "raft"
    
    [raft]
    node_id = "secreton-node-1"
    data_dir = "/data/raft"
```

### Authenc Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: authenc-config
  namespace: simpelv2
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    
    [auth]
    jwt_expiration = 3600
    refresh_token_expiration = 604800
```

## 🔌 gRPC Communication

### Service-to-Service Communication

Services communicate via gRPC for better performance:

```rust
// Authenc calling Secreton
use secreton::v1::secreton_service_client::SecretonServiceClient;

let mut client = SecretonServiceClient::connect("http://secreton-service:9090").await?;
let response = client.get_secret(GetSecretRequest {
    path: "auth/jwt-key".to_string(),
    version: None,
}).await?;
```

### Proto Files

Proto definitions are in `/infra/proto/`:
- `secreton.proto` - Secreton service definitions
- `authenc.proto` - Authenc service definitions
- `common.proto` - Shared types

### Generate Rust Code

```bash
# Install protoc
sudo apt install protobuf-compiler

# Generate Rust code from proto files
cd /srv/proyek/simpelv2/infra/proto
protoc --rust_out=. --grpc_out=. *.proto
```

## 📊 Monitoring & Observability

### Istio Dashboard

```bash
# Access Kiali dashboard
istioctl dashboard kiali

# Access Grafana
istioctl dashboard grafana

# Access Jaeger tracing
istioctl dashboard jaeger
```

### Prometheus Metrics

All services expose Prometheus metrics on `/metrics` endpoint:

```bash
# Port-forward to access metrics
kubectl port-forward -n simpelv2 svc/secreton-service 8200:8200
curl http://localhost:8200/metrics
```

### Logs

```bash
# View logs
kubectl logs -n simpelv2 -l app=secreton -f
kubectl logs -n simpelv2 -l app=authenc -f
kubectl logs -n simpelv2 -l app=portal -f

# View Istio sidecar logs
kubectl logs -n simpelv2 <pod-name> -c istio-proxy
```

## 🔒 Security

### mTLS

Istio automatically enables mTLS between services:

```bash
# Verify mTLS
kubectl exec -n simpelv2 <pod-name> -c istio-proxy -- \
  curl http://localhost:15000/config_dump | grep -A 10 tls_context
```

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: authenc-policy
  namespace: simpelv2
spec:
  podSelector:
    matchLabels:
      app: authenc
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: portal
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: secreton
    ports:
    - protocol: TCP
      port: 9090
```

## 🧪 Testing

### Health Checks

```bash
# Check service health
kubectl exec -n simpelv2 <pod-name> -- curl http://localhost:8200/health
kubectl exec -n simpelv2 <pod-name> -- curl http://localhost:8080/health
```

### gRPC Testing

```bash
# Install grpcurl
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Test Secreton gRPC
grpcurl -plaintext secreton-service:9090 list
grpcurl -plaintext secreton-service:9090 \
  secreton.v1.SecretonService/HealthCheck

# Test Authenc gRPC
grpcurl -plaintext authenc-service:9090 list
grpcurl -plaintext authenc-service:9090 \
  authenc.v1.AuthencService/HealthCheck
```

## 🔄 Scaling

### Horizontal Pod Autoscaling

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: secreton-hpa
  namespace: simpelv2
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: secreton
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Manual Scaling

```bash
# Scale Secreton
kubectl scale deployment secreton -n simpelv2 --replicas=5

# Scale Authenc
kubectl scale deployment authenc -n simpelv2 --replicas=5
```

## 🐛 Troubleshooting

### Common Issues

1. **Pods not starting**
   ```bash
   kubectl describe pod <pod-name> -n simpelv2
   kubectl logs <pod-name> -n simpelv2 --previous
   ```

2. **Service not reachable**
   ```bash
   kubectl get endpoints -n simpelv2
   kubectl exec -n simpelv2 <pod-name> -- nslookup secreton-service
   ```

3. **Istio sidecar issues**
   ```bash
   kubectl logs <pod-name> -n simpelv2 -c istio-proxy
   istioctl analyze -n simpelv2
   ```

4. **gRPC connection issues**
   ```bash
   # Check if gRPC port is open
   kubectl exec -n simpelv2 <pod-name> -- nc -zv secreton-service 9090
   ```

## 📚 References

- [Istio Documentation](https://istio.io/latest/docs/)
- [Envoy Proxy](https://www.envoyproxy.io/docs/envoy/latest/)
- [MicroK8s](https://microk8s.io/docs)
- [gRPC Rust](https://github.com/hyperium/tonic)
- [Protocol Buffers](https://developers.google.com/protocol-buffers)

## 🎯 Next Steps

1. **Setup CI/CD**: Automate deployment with GitLab CI/CD
2. **Add Monitoring**: Integrate with Prometheus and Grafana
3. **Setup Backup**: Configure backup for Secreton data
4. **Performance Testing**: Load test with k6 or Gatling
5. **Security Hardening**: Implement Pod Security Policies
