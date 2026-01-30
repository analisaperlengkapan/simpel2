# PostgreSQL High Availability Setup

Production-ready PostgreSQL HA cluster using Zalando Postgres Operator with Longhorn storage.

## 📋 Overview

### Current Setup (Single Instance)
- ❌ **Single replica** - not HA
- ❌ **Single point of failure**
- ✅ **Longhorn storage** - 50Gi

### Target Setup (HA Cluster)
- ✅ **3 replicas** (1 primary + 2 standby)
- ✅ **Automatic failover** via Patroni
- ✅ **Streaming replication**
- ✅ **Connection pooling** via PgBouncer
- ✅ **Longhorn storage** - 150Gi total (50Gi per pod)

## 🚀 Quick Start

### Prerequisites

```bash
# Ensure kubectl is configured
kubectl cluster-info

# Ensure Helm is installed
helm version

# Ensure Longhorn is running
kubectl get storageclass longhorn
```

### Installation Steps

#### 1. Install Postgres Operator

```bash
cd /srv/proyek/simpelv2/k8s/postgresql-ha

# Install operator
./00-operator-install.sh

# Verify operator is running
kubectl get pods -n postgres-operator
```

#### 2. Deploy HA Cluster

```bash
# Deploy PostgreSQL HA cluster
kubectl apply -f 01-postgresql-ha-cluster.yaml

# Wait for cluster to be ready (this takes 2-5 minutes)
kubectl get postgresql -n simpelv2 -w

# Expected output when ready:
# NAME                    TEAM       VERSION   PODS   VOLUME   CPU-REQUEST   MEMORY-REQUEST   AGE   STATUS
# simpelv2-postgres-ha    simpelv2   15        3      50Gi     500m          1Gi              5m    Running
```

#### 3. Verify Cluster Health

```bash
# Check pods
kubectl get pods -n simpelv2 -l cluster-name=simpelv2-postgres-ha

# Expected: 3 postgres pods + 2 pooler pods
# simpelv2-postgres-ha-0          2/2     Running   0          5m
# simpelv2-postgres-ha-1          2/2     Running   0          4m
# simpelv2-postgres-ha-2          2/2     Running   0          3m
# simpelv2-postgres-ha-pooler-... 1/1     Running   0          5m
# simpelv2-postgres-ha-pooler-... 1/1     Running   0          5m

# Check cluster status
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- patronictl list

# Expected output:
# + Cluster: simpelv2-postgres-ha (7123...) -----+----+-----------+
# | Member                 | Host      | Role    | State   | TL | Lag in MB |
# +------------------------+-----------+---------+---------+----+-----------+
# | simpelv2-postgres-ha-0 | 10.1.x.x  | Leader  | running |  1 |           |
# | simpelv2-postgres-ha-1 | 10.1.x.x  | Replica | running |  1 |         0 |
# | simpelv2-postgres-ha-2 | 10.1.x.x  | Replica | running |  1 |         0 |
# +------------------------+-----------+---------+---------+----+-----------+
```

#### 4. Migrate Data

```bash
# Run migration script (dry-run first)
./02-migration-script.sh --dry-run

# If dry-run looks good, run actual migration
./02-migration-script.sh

# This script will:
# 1. Backup old PostgreSQL data
# 2. Restore to new HA cluster
# 3. Verify data integrity
# 4. Guide you through connection string updates
```

## 📊 Services Created

The operator creates these Kubernetes services:

```bash
kubectl get svc -n simpelv2 | grep simpelv2-postgres-ha
```

| Service | Purpose | Connection String |
|---------|---------|-------------------|
| `simpelv2-postgres-ha` | Primary (read-write) | `postgres://user:pass@simpelv2-postgres-ha:5432/db` |
| `simpelv2-postgres-ha-repl` | Replicas (read-only) | `postgres://user:pass@simpelv2-postgres-ha-repl:5432/db` |
| `simpelv2-postgres-ha-pooler` | Connection pool | `postgres://user:pass@simpelv2-postgres-ha-pooler:5432/db` |
| `simpelv2-postgres-ha-config` | Patroni config | Internal use only |

### Recommended Connection Strings

**For application workloads** (use connection pooler):
```
DATABASE_URL=postgres://simpelv2_owner:<password>@simpelv2-postgres-ha-pooler:5432/simpelv2
```

**For read-heavy queries** (use read replicas):
```
DATABASE_REPLICA_URL=postgres://simpelv2_owner:<password>@simpelv2-postgres-ha-repl:5432/simpelv2
```

**For admin operations** (direct to primary):
```
DATABASE_ADMIN_URL=postgres://postgres:<password>@simpelv2-postgres-ha:5432/simpelv2
```

## 🔐 Credentials

The operator creates secrets automatically:

```bash
# View generated secrets
kubectl get secrets -n simpelv2 | grep simpelv2-postgres-ha

# Get postgres user password
kubectl get secret postgres.simpelv2-postgres-ha.credentials.postgresql.acid.zalan.do \
  -n simpelv2 -o jsonpath='{.data.password}' | base64 -d

# Get application user password
kubectl get secret simpelv2-owner.simpelv2-postgres-ha.credentials.postgresql.acid.zalan.do \
  -n simpelv2 -o jsonpath='{.data.password}' | base64 -d
```

## 🧪 Testing

### Test Database Connectivity

```bash
# Connect to primary
kubectl exec -it simpelv2-postgres-ha-0 -n simpelv2 -- \
  psql -U postgres -d simpelv2

# Run test query
SELECT version();
SELECT current_database();
\dt
```

### Test Replication

```bash
# Check replication status
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  psql -U postgres -c "SELECT * FROM pg_stat_replication;"

# Expected: 2 replicas streaming
```

### Test Failover

```bash
# Delete primary pod to trigger failover
kubectl delete pod simpelv2-postgres-ha-0 -n simpelv2

# Watch automatic failover (should complete in < 30 seconds)
kubectl get pods -n simpelv2 -l cluster-name=simpelv2-postgres-ha -w

# Verify new leader elected
kubectl exec -n simpelv2 simpelv2-postgres-ha-1 -- patronictl list

# Application connections should NOT be interrupted (handled by pooler)
```

### Test Connection Pooling

```bash
# Connect via pooler
kubectl run psql-test --rm -it --image=postgres:15-alpine -- \
  psql postgres://simpelv2_owner:<password>@simpelv2-postgres-ha-pooler.simpelv2:5432/simpelv2

# Check pooler stats
kubectl exec -n simpelv2 simpelv2-postgres-ha-pooler-xxxx -- \
  psql -U pooler -p 6432 pgbouncer -c "SHOW POOLS;"
```

## 📈 Monitoring

### Prometheus Metrics

Metrics are exposed on port 9187 (postgres_exporter):

```bash
# Port-forward to view metrics
kubectl port-forward simpelv2-postgres-ha-0 9187:9187 -n simpelv2

# Access metrics
curl http://localhost:9187/metrics
```

### Key Metrics to Monitor

- `pg_up` - Database availability (should be 1)
- `pg_stat_replication_pg_wal_lsn_diff` - Replication lag in bytes
- `pg_stat_database_numbackends` - Active connections
- `pg_settings_max_connections` - Connection limit
- `patroni_postgres_running` - Patroni status

### Grafana Dashboard

Import Grafana dashboard ID: 9628 (PostgreSQL Database)

## 🔧 Operations

### Scale Cluster

```bash
# Scale to 5 replicas
kubectl patch postgresql simpelv2-postgres-ha -n simpelv2 \
  --type merge -p '{"spec":{"numberOfInstances":5}}'

# Scale pooler
kubectl patch postgresql simpelv2-postgres-ha -n simpelv2 \
  --type merge -p '{"spec":{"connectionPooler":{"numberOfInstances":3}}}'
```

### Increase Storage

```bash
# Increase volume size to 100Gi
kubectl patch postgresql simpelv2-postgres-ha -n simpelv2 \
  --type merge -p '{"spec":{"volume":{"size":"100Gi"}}}'

# Operator will resize PVCs automatically (if storageClass supports it)
```

### Manual Switchover

```bash
# Switchover to replica-1
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  patronictl switchover --candidate simpelv2-postgres-ha-1 --force
```

### Restart Cluster

```bash
# Restart all pods (rolling)
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  patronictl restart simpelv2-postgres-ha --force
```

## 💾 Backup & Recovery

### Manual Backup

```bash
# Logical backup (pg_dump)
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  pg_dumpall -U postgres > backup-$(date +%Y%m%d).sql

# Physical backup (pg_basebackup)
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  pg_basebackup -D /tmp/backup -F tar -z -P -U postgres
```

### Automated Backups (S3/MinIO)

Uncomment backup section in `01-postgresql-ha-cluster.yaml`:

```yaml
spec:
  enableLogicalBackup: true
  logicalBackupSchedule: "30 00 * * *"  # Daily at 00:30 UTC
  env:
    - name: LOGICAL_BACKUP_S3_BUCKET
      value: "postgres-backups"
    # ... (see manifest for full config)
```

### Point-in-Time Recovery (PITR)

```yaml
# Create new cluster from backup
apiVersion: "acid.zalan.do/v1"
kind: postgresql
metadata:
  name: simpelv2-postgres-restored
spec:
  clone:
    cluster: "simpelv2-postgres-ha"
    timestamp: "2026-01-30T10:00:00Z"
```

## 🚨 Troubleshooting

### Pods Not Starting

```bash
# Check operator logs
kubectl logs -n postgres-operator deployment/postgres-operator -f

# Check pod events
kubectl describe pod simpelv2-postgres-ha-0 -n simpelv2

# Check PVC status
kubectl get pvc -n simpelv2 | grep simpelv2-postgres-ha
```

### Replication Lag

```bash
# Check lag
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  psql -U postgres -c "SELECT
    client_addr,
    state,
    sync_state,
    pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS lag_bytes,
    replay_lag
  FROM pg_stat_replication;"

# If lag > 100MB, investigate:
# - Network issues
# - Disk I/O bottleneck
# - Long-running queries on replica
```

### Split-Brain Prevention

Patroni prevents split-brain via DCS (etcd/consul/kubernetes). To verify:

```bash
# Check DCS endpoints
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  patronictl show-config

# Verify only one leader
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  patronictl list | grep Leader
```

### Connection Issues

```bash
# Test from inside cluster
kubectl run psql-debug --rm -it --image=postgres:15-alpine -- \
  psql postgres://simpelv2_owner:PASSWORD@simpelv2-postgres-ha-pooler.simpelv2:5432/simpelv2

# Check pg_hba.conf
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  cat /home/postgres/pgdata/pgroot/data/pg_hba.conf

# Check connection limits
kubectl exec -n simpelv2 simpelv2-postgres-ha-0 -- \
  psql -U postgres -c "SHOW max_connections;"
```

## 📚 Additional Resources

- [Zalando Postgres Operator Docs](https://postgres-operator.readthedocs.io/)
- [Patroni Documentation](https://patroni.readthedocs.io/)
- [PostgreSQL HA Best Practices](https://www.postgresql.org/docs/15/high-availability.html)
- [PgBouncer Documentation](https://www.pgbouncer.org/)

## 🤝 Support

For issues or questions:
1. Check operator logs: `kubectl logs -n postgres-operator deployment/postgres-operator`
2. Check cluster events: `kubectl describe postgresql simpelv2-postgres-ha -n simpelv2`
3. Check detailed setup guide: `/tmp/claude-1000/.../scratchpad/postgresql-ha-setup.md`

---

**Last Updated:** 2026-01-30
**Operator Version:** Zalando Postgres Operator latest
**PostgreSQL Version:** 15
