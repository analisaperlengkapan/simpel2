# Secreton Deployment Guide

## Quick Start

### Local Development (Docker Compose)

```bash
# 1. Start all services
docker-compose up -d

# 2. Initialize Secreton
docker-compose exec secreton ./secreton init --shares 5 --threshold 3

# 3. Unseal Secreton (provide 3 of 5 keys)
docker-compose exec secreton ./secreton unseal

# 4. Access Secreton
curl http://localhost:8200/health

# 5. View logs
docker-compose logs -f secreton

# 6. Stop services
docker-compose down
```

### Kubernetes Production Deployment

```bash
# 1. Create namespace and secrets
kubectl apply -f deploy/kubernetes/00-namespace.yaml
kubectl apply -f deploy/kubernetes/01-secrets.yaml

# 2. Update secrets with actual credentials
kubectl -n secreton edit secret postgres-credentials
kubectl -n secreton edit secret secreton-config

# 3. Deploy PostgreSQL
kubectl apply -f deploy/kubernetes/02-configmaps.yaml
kubectl apply -f deploy/kubernetes/03-postgres.yaml

# Wait for PostgreSQL to be ready
kubectl -n secreton wait --for=condition=ready pod -l app=postgres --timeout=300s

# 4. Deploy Secreton cluster
kubectl apply -f deploy/kubernetes/04-secreton-statefulset.yaml
kubectl apply -f deploy/kubernetes/05-ingress-rbac.yaml
kubectl apply -f deploy/kubernetes/06-autoscaling-policies.yaml

# 5. Verify deployment
kubectl -n secreton get pods
kubectl -n secreton get svc

# 6. Initialize Secreton (on first pod)
kubectl -n secreton exec -it secreton-0 -- /app/secreton init --shares 5 --threshold 3

# 7. Unseal all pods
for i in 0 1 2; do
  kubectl -n secreton exec -it secreton-$i -- /app/secreton unseal
done
```

## Deployment Architectures

### Development

```
┌─────────────────┐
│ Docker Compose  │
├─────────────────┤
│  Secreton       │
│  PostgreSQL     │
│  Prometheus     │
│  Grafana        │
└─────────────────┘
```

**Use Case**: Local development, testing
**Cost**: Free
**Availability**: Single node

### Production (Single Region)

```
┌─────────────────────────┐
│   Kubernetes Cluster    │
├─────────────────────────┤
│  StatefulSet (3 pods)   │
│  ├─ Secreton-0          │
│  ├─ Secreton-1          │
│  └─ Secreton-2          │
│  PostgreSQL (HA)        │
│  Load Balancer          │
│  Monitoring Stack       │
└─────────────────────────┘
```

**Use Case**: Production workloads
**Cost**: Moderate
**Availability**: 99.9%+ (3-node quorum)

### Production (Multi-Region)

```
Region A                Region B                Region C
┌──────────┐            ┌──────────┐            ┌──────────┐
│ 3-node   │  <──────>  │ 3-node   │  <──────>  │ 3-node   │
│ Cluster  │   Raft     │ Cluster  │   Raft     │ Cluster  │
└──────────┘            └──────────┘            └──────────┘
```

**Use Case**: Global, mission-critical
**Cost**: High
**Availability**: 99.99%+

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (error/warn/info/debug/trace) |
| `SECRETON_CONFIG` | `/app/config/secreton.toml` | Config file path |
| `SECRETON_STORAGE_TYPE` | `postgres` | Storage backend type |
| `SECRETON_POSTGRES_URL` | - | PostgreSQL connection URL |

### Configuration File

See [secreton.toml.example](../secreton.toml.example) for full reference.

Key sections:
- `[server]`: API listener configuration
- `[storage]`: Backend storage settings
- `[seal]`: Seal/unseal mechanism
- `[telemetry]`: Metrics and monitoring
- `[logging]`: Log output configuration

## Resource Requirements

### Docker Compose

| Service | CPU | Memory | Disk |
|---------|-----|--------|------|
| Secreton | 1 core | 512 MB | 1 GB |
| PostgreSQL | 1 core | 256 MB | 10 GB |
| Prometheus | 0.5 core | 512 MB | 20 GB |
| Grafana | 0.5 core | 256 MB | 1 GB |
| **Total** | **3 cores** | **1.5 GB** | **32 GB** |

### Kubernetes Production

| Component | Replicas | CPU (per pod) | Memory (per pod) | Disk (per pod) |
|-----------|----------|---------------|------------------|----------------|
| Secreton | 3-10 | 500m-2 | 512 MB - 2 GB | 10 GB |
| PostgreSQL | 1 | 250m-1 | 256 MB - 1 GB | 10-100 GB |

**Recommended Node Size**: 4 CPU, 16 GB RAM

## Health Checks

### HTTP Health Endpoint

```bash
curl http://localhost:8200/health
```

**Response (healthy)**:
```json
{
  "status": "healthy",
  "sealed": false,
  "initialized": true,
  "version": "0.1.0"
}
```

### Kubernetes Probes

**Liveness Probe**: `/health` (HTTP 200)
**Readiness Probe**: `/health` (HTTP 200)

## Monitoring

### Metrics Endpoints

- **Prometheus**: `http://localhost:8200/metrics`
- **Grafana**: `http://localhost:3000` (admin/admin)

### Key Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `secreton_requests_total` | Counter | Total API requests |
| `secreton_request_duration_seconds` | Histogram | Request latency |
| `secreton_storage_operations_total` | Counter | Storage operations |
| `secreton_seal_status` | Gauge | Seal status (0=sealed, 1=unsealed) |

## Security

### TLS Configuration

1. Generate certificates:
```bash
# Self-signed (development)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Production: Use cert-manager or your CA
```

2. Mount certificates:
```yaml
volumes:
  - name: tls
    secret:
      secretName: secreton-tls
```

### Network Security

**Firewall Rules**:
- Allow: 8200 (HTTP), 8201 (HTTPS) from authorized IPs
- Allow: 8300 (cluster) only within cluster
- Deny: All other traffic

**Kubernetes NetworkPolicy**: Automatically applied

## Backup & Recovery

### PostgreSQL Backup

```bash
# Backup
kubectl -n secreton exec -it postgres-0 -- pg_dump -U secreton secreton > backup.sql

# Restore
kubectl -n secreton exec -i postgres-0 -- psql -U secreton secreton < backup.sql
```

### Secreton Snapshot

```bash
# Create snapshot
kubectl -n secreton exec -it secreton-0 -- /app/secreton snapshot save /app/data/snapshot.json

# Restore snapshot
kubectl -n secreton exec -it secreton-0 -- /app/secreton snapshot restore /app/data/snapshot.json
```

## Troubleshooting

### Common Issues

**1. Secreton is sealed**
```bash
# Unseal with threshold keys
kubectl -n secreton exec -it secreton-0 -- /app/secreton unseal
```

**2. PostgreSQL connection failed**
```bash
# Check PostgreSQL logs
kubectl -n secreton logs postgres-0

# Verify connection
kubectl -n secreton exec -it secreton-0 -- pg_isready -h postgres -U secreton
```

**3. Pod not starting**
```bash
# Check pod status
kubectl -n secreton describe pod secreton-0

# Check logs
kubectl -n secreton logs secreton-0 --previous
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug

# Or in Kubernetes
kubectl -n secreton set env statefulset/secreton RUST_LOG=debug
```

## Performance Tuning

See [PERFORMANCE_ANALYSIS.md](../PERFORMANCE_ANALYSIS.md) for detailed tuning guide.

### Quick Wins

1. **Use SSD storage** for PostgreSQL and Secreton data
2. **Enable connection pooling** (max_connections=50)
3. **Configure HPA** for auto-scaling (3-10 replicas)
4. **Use caching** for frequently accessed secrets

## Upgrading

### Zero-Downtime Upgrade

```bash
# 1. Update image in StatefulSet
kubectl -n secreton set image statefulset/secreton secreton=secreton:v0.2.0

# 2. Kubernetes will perform rolling update automatically
kubectl -n secreton rollout status statefulset/secreton

# 3. Verify each pod after update
for i in 0 1 2; do
  kubectl -n secreton exec secreton-$i -- /app/secreton version
done
```

### Rollback

```bash
# Rollback to previous version
kubectl -n secreton rollout undo statefulset/secreton

# Check rollback status
kubectl -n secreton rollout status statefulset/secreton
```

## Support

- **Documentation**: [README.md](../README.md)
- **Security**: [SECURITY.md](../SECURITY.md)
- **Issues**: GitHub Issues
- **Email**: support@cipherce.io

---

**Last Updated**: 2025-11-25
**Version**: 1.0
