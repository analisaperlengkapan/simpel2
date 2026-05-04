# Secreton Deployment Guide

> **Status**: Path standalone Kubernetes (`deploy/kubernetes/` & `deploy/staging/`)
> sudah dihapus pada migrasi Helm. Untuk deployment Kubernetes/cluster, gunakan
> chart Helm `infra/helm/simpel/` yang sudah meng-include Secreton sebagai StatefulSet.
> Direktori ini sekarang khusus berisi materi Docker Compose / development.

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

### Kubernetes Deployment

Deployment Kubernetes dilakukan lewat chart `simpel`:

```bash
# Staging
./infra/helm/deploy.sh staging install

# Production (Secret WAJIB lewat Secreton SecretSync CRD)
./infra/helm/deploy.sh production install
```

Knob Secreton di chart (default `secreton.enabled: true`):

| Values key                          | Default                | Catatan                                            |
| ----------------------------------- | ---------------------- | -------------------------------------------------- |
| `secreton.enabled`                  | `true`                 | Set `false` jika ingin pakai instance eksternal.   |
| `secreton.replicas`                 | `1` (staging) / `3` (prod) | Quorum ≥ 3 untuk HA.                              |
| `secreton.storage.size`             | `10Gi`                 | Persistent vault storage (longhorn).               |
| `secreton.image.{name,tag}`         | `secreton` / global tag | Override image per env.                           |
| `secreton.istioInjection`           | `false`                | Secreton butuh konektivitas DB langsung.           |

Init/unseal pertama kali masih manual:

```bash
kubectl -n simpelv2-staging exec -it secreton-0 -- /app/secreton init --shares 5 --threshold 3
for i in 0 1 2; do
  kubectl -n simpelv2-staging exec -it secreton-$i -- /app/secreton unseal
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
| `DATABASE_URL` | (from Secret) | PostgreSQL connection URL |

### Configuration File

See [`../secreton.toml.example`](../secreton.toml.example) for full reference.

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

### Kubernetes Production (chart `simpel`)

| Component | Replicas | CPU (per pod) | Memory (per pod) | Disk (per pod) |
|-----------|----------|---------------|------------------|----------------|
| Secreton | 3-10 | 250m-1 | 512 MB - 1 GB | 10 GB |
| PostgreSQL | 3 | 250m-1 | 512 MB - 2 GB | 100 GB |

## Health Checks

### HTTP Endpoints

| Path     | Purpose                                                     |
| -------- | ----------------------------------------------------------- |
| `/health` | Generic alive check                                         |
| `/live`   | Liveness probe — selalu 200 selama proses hidup             |
| `/ready`  | Readiness probe — 503 saat sealed                           |
| `/metrics`| Prometheus metrics (port 9090)                              |

## Backup & Recovery

### PostgreSQL Backup

```bash
kubectl -n simpelv2-production exec -it postgres-0 -- pg_dump -U secreton secreton > backup.sql
kubectl -n simpelv2-production exec -i postgres-0 -- psql -U secreton secreton < backup.sql
```

### Secreton Snapshot

```bash
kubectl -n simpelv2-production exec -it secreton-0 -- /app/secreton snapshot save /data/snapshot.json
kubectl -n simpelv2-production exec -it secreton-0 -- /app/secreton snapshot restore /data/snapshot.json
```

## Upgrading

```bash
# 1. Bump tag di values
$EDITOR infra/helm/simpel/values-production.yaml   # secreton.image.tag

# 2. Rolling update
./infra/helm/deploy.sh production upgrade

# 3. Verifikasi
kubectl -n simpelv2-production rollout status statefulset/secreton

# Rollback bila perlu
helm rollback simpel -n simpelv2-production
```

## Support

- Helm chart: [`infra/helm/simpel`](../../../infra/helm/simpel/)
- Source: [`../README.md`](../README.md)
- Security: [`../SECURITY.md`](../SECURITY.md)
- Operator: [`../crates/k8s-operator/README.md`](../crates/k8s-operator/README.md)

---

**Last Updated**: 2026-05-03
