# Secreton Storage Backend Configuration

## Overview

Secreton now supports **multiple persistent storage backends** with **Raft consensus as the default** for production deployments. This ensures data safety, high availability, and consistency across distributed deployments.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Secreton API Server                    │
├─────────────────────────────────────────────────────────┤
│                  Storage Backend Layer                  │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Raft (HA)    │  │ PostgreSQL   │  │ File Storage │   │
│  │ Consensus    │  │ (Single Node)│  │ (Local)      │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│  ┌──────────────┐  ┌──────────────┐                     │
│  │ Redis        │  │ Consul       │                     │
│  │ (Cache)      │  │ (Distributed)│                     │
│  └──────────────┘  └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

## Storage Backends

### 1. Raft (Default - Recommended for Production)

**Status**: ✅ Implemented
**Feature**: `raft-consensus`
**Use Case**: High availability, distributed deployments, data safety

**Advantages**:

- ✅ Distributed consensus algorithm
- ✅ Automatic leader election
- ✅ Data replication across nodes
- ✅ Automatic failover
- ✅ Snapshot and log persistence
- ✅ No external dependencies

**Configuration**:

```toml
[storage]
backend = "raft"

[storage.raft]
node_id = 1
bind_address = "0.0.0.0:7000"
peers = ["2=http://node2:7000", "3=http://node3:7000"]
snapshot_interval = 3600
election_timeout_ms = 1500
heartbeat_interval_ms = 150
data_dir = "/var/lib/secreton/raft"
```

**Environment Variables**:

```bash
SECRETON_STORAGE_BACKEND=raft
SECRETON_RAFT_NODE_ID=1
SECRETON_RAFT_PEERS="2=http://node2:7000,3=http://node3:7000"
```

**Deployment**:

```yaml
# Docker Compose Example
services:
  secreton-node1:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 1
      SECRETON_RAFT_PEERS: "2=http://secreton-node2:7000,3=http://secreton-node3:7000"
    volumes:
      - secreton-data-1:/var/lib/secreton/raft
    ports:
      - "8200:8200"
      - "7000:7000"

  secreton-node2:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 2
      SECRETON_RAFT_PEERS: "1=http://secreton-node1:7000,3=http://secreton-node3:7000"
    volumes:
      - secreton-data-2:/var/lib/secreton/raft
    ports:
      - "8201:8200"
      - "7001:7000"

  secreton-node3:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 3
      SECRETON_RAFT_PEERS: "1=http://secreton-node1:7000,2=http://secreton-node2:7000"
    volumes:
      - secreton-data-3:/var/lib/secreton/raft
    ports:
      - "8202:8200"
      - "7002:7000"

volumes:
  secreton-data-1:
  secreton-data-2:
  secreton-data-3:
```

### 2. PostgreSQL (Single Node)

**Status**: ✅ Implemented
**Feature**: `postgres`
**Use Case**: Single node deployments, existing PostgreSQL infrastructure

**Advantages**:

- ✅ Proven database
- ✅ ACID compliance
- ✅ Backup/restore tools
- ✅ Monitoring tools
- ✅ Scalable storage

**Configuration**:

```toml
[storage]
backend = "postgres"
postgres_url = "postgresql://user:password@localhost:5432/secreton"
```

**Environment Variables**:

```bash
SECRETON_STORAGE_BACKEND=postgres
SECRETON_STORAGE_URL=postgresql://user:password@postgres:5432/secreton
# Or
DATABASE_URL=postgresql://user:password@postgres:5432/secreton
```

### 3. File Storage (Development)

**Status**: ✅ Implemented
**Feature**: `file`
**Use Case**: Development, testing, single-node deployments

**Configuration**:

```toml
[storage]
backend = "file"
file_path = "/var/lib/secreton/data"
```

**Environment Variables**:

```bash
SECRETON_STORAGE_BACKEND=file
SECRETON_STORAGE_FILE_PATH=/var/lib/secreton/data
```

### 4. Redis (Cache/Session Store)

**Status**: ⚠️ Planned
**Feature**: `redis`
**Use Case**: Session storage, caching

**Configuration**:

```toml
[storage]
backend = "redis"
redis_url = "redis://localhost:6379"
```

### 5. Consul (Distributed)

**Status**: ⚠️ Planned
**Feature**: `consul`
**Use Case**: Distributed deployments, service mesh integration

**Configuration**:

```toml
[storage]
backend = "consul"
consul_address = "consul:8500"
consul_path = "secreton/"
```

### 6. Memory (Development Only)

**Status**: ✅ Implemented
**Feature**: None (always available)
**Use Case**: Development, testing, ephemeral deployments

**Configuration**:

```toml
[storage]
backend = "memory"
```

**⚠️ WARNING**: Data is lost on restart!

## Configuration Priority

Configuration is applied in this order (highest to lowest priority):

1. **Environment Variables** (highest priority)

   ```bash
   SECRETON_STORAGE_BACKEND=raft
   SECRETON_RAFT_NODE_ID=1
   ```

2. **Config File** (default.toml or production.toml)

   ```toml
   [storage]
   backend = "raft"
   ```

3. **Hardcoded Defaults** (lowest priority)
   - Default backend: `raft`
   - Default node ID: `1`

## Data Safety & Persistence

### Raft (Recommended)

```
┌─────────────────────────────────────────┐
│  Secret Vault Data                             │
├─────────────────────────────────────────┤
│  Secrets, Keys, Policies, Audit Logs    │
├─────────────────────────────────────────┤
│  Raft Consensus Layer                   │
├─────────────────────────────────────────┤
│  Persistent Storage                     │
│  • Raft Logs: /var/lib/secreton/raft/   │
│  • Snapshots: /var/lib/secreton/raft/   │
│  • State: /var/lib/secreton/raft/       │
└─────────────────────────────────────────┘
```

**Data Flow**:

1. Write request received by API
2. Applied to state machine
3. Replicated to other nodes via Raft
4. Persisted to disk
5. Acknowledged to client

**Guarantees**:

- ✅ Durability: Data persisted before acknowledgment
- ✅ Consistency: All nodes have same data
- ✅ Availability: Survives node failures
- ✅ Partition Tolerance: Handles network splits

### PostgreSQL (Single Node)

```
┌─────────────────────────────────────────┐
│  Secret Vault Data                             │
├─────────────────────────────────────────┤
│  Secrets, Keys, Policies, Audit Logs    │
├─────────────────────────────────────────┤
│  PostgreSQL Database                    │
├─────────────────────────────────────────┤
│  Persistent Storage                     │
│  • Tables: secreton_secrets, etc.       │
│  • Indexes: For fast queries             │
│  • WAL: Write-ahead logs                │
└─────────────────────────────────────────┘
```

## Production Deployment Guide

### Single Node (PostgreSQL)

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: secreton
      POSTGRES_USER: secreton
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  secreton:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: postgres
      DATABASE_URL: postgresql://secreton:${DB_PASSWORD}@postgres:5432/secreton
      JWT_SECRET: ${JWT_SECRET}
      SECRETON_ENV: production
    depends_on:
      - postgres
    ports:
      - "8200:8200"
      - "8201:8201"

volumes:
  postgres-data:
```

### 3-Node Raft Cluster (Recommended)

```yaml
version: '3.8'
services:
  secreton-1:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 1
      SECRETON_RAFT_PEERS: "2=http://secreton-2:7000,3=http://secreton-3:7000"
      JWT_SECRET: ${JWT_SECRET}
      SECRETON_ENV: production
    volumes:
      - raft-data-1:/var/lib/secreton/raft
    ports:
      - "8200:8200"
      - "8201:8201"
      - "7000:7000"

  secreton-2:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 2
      SECRETON_RAFT_PEERS: "1=http://secreton-1:7000,3=http://secreton-3:7000"
      JWT_SECRET: ${JWT_SECRET}
      SECRETON_ENV: production
    volumes:
      - raft-data-2:/var/lib/secreton/raft
    ports:
      - "8201:8200"
      - "8202:8201"
      - "7001:7000"

  secreton-3:
    image: secreton:latest
    environment:
      SECRETON_STORAGE_BACKEND: raft
      SECRETON_RAFT_NODE_ID: 3
      SECRETON_RAFT_PEERS: "1=http://secreton-1:7000,2=http://secreton-2:7000"
      JWT_SECRET: ${JWT_SECRET}
      SECRETON_ENV: production
    volumes:
      - raft-data-3:/var/lib/secreton/raft
    ports:
      - "8202:8200"
      - "8203:8201"
      - "7002:7000"

  # Load balancer
  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - secreton-1
      - secreton-2
      - secreton-3

volumes:
  raft-data-1:
  raft-data-2:
  raft-data-3:
```

## Monitoring & Operations

### Health Check

```bash
# Check storage backend status
curl http://localhost:8200/v1/health

# Response includes storage backend info
{
  "success": true,
  "data": {
    "status": "healthy",
    "dependencies": {
      "storage": {
        "healthy": true,
        "message": "Raft cluster healthy, 3/3 nodes"
      }
    }
  }
}
```

### Raft Cluster Status

```bash
# Get cluster status
curl http://localhost:8200/v1/sys/raft/status

# Response
{
  "success": true,
  "data": {
    "node_id": 1,
    "leader": true,
    "term": 5,
    "log_index": 1234,
    "peers": [
      {"id": 1, "address": "http://node1:7000", "status": "healthy"},
      {"id": 2, "address": "http://node2:7000", "status": "healthy"},
      {"id": 3, "address": "http://node3:7000", "status": "healthy"}
    ]
  }
}
```

### Backup & Restore

```bash
# Create backup
curl -X POST http://localhost:8200/v1/sys/backup \
  -H "X-Secret Vault-Token: $TOKEN"

# Restore from backup
curl -X POST http://localhost:8200/v1/sys/restore \
  -H "X-Secret Vault-Token: $TOKEN" \
  -d @backup.json
```

## Migration Guide

### From Memory to Raft

1. **Stop the service**:

   ```bash
   docker stop secreton
   ```

2. **Update configuration**:

   ```toml
   [storage]
   backend = "raft"
   ```

3. **Create data directory**:

   ```bash
   mkdir -p /var/lib/secreton/raft
   chmod 700 /var/lib/secreton/raft
   ```

4. **Start with Raft**:

   ```bash
   docker start secreton
   ```

### From PostgreSQL to Raft

1. **Backup PostgreSQL data**:

   ```bash
   curl -X POST http://localhost:8200/v1/sys/backup \
     -H "X-Secret Vault-Token: $TOKEN" > backup.json
   ```

2. **Update configuration**:

   ```toml
   [storage]
   backend = "raft"
   ```

3. **Restore to Raft**:

   ```bash
   curl -X POST http://localhost:8200/v1/sys/restore \
     -H "X-Secret Vault-Token: $TOKEN" \
     -d @backup.json
   ```

## Security Considerations

### Raft Cluster

- ✅ **Encryption**: Use TLS for Raft communication
- ✅ **Authentication**: Implement mutual TLS
- ✅ **Isolation**: Run on private network
- ✅ **Firewall**: Restrict access to Raft port (7000)

### PostgreSQL

- ✅ **Encryption**: Use SSL/TLS connections
- ✅ **Authentication**: Strong passwords
- ✅ **Isolation**: Private network only
- ✅ **Backups**: Encrypted backups

### File Storage

- ✅ **Permissions**: Restrict file permissions (700)
- ✅ **Encryption**: Encrypt at rest
- ✅ **Backups**: Regular encrypted backups
- ⚠️ **Not for production**: Use only for development

## Troubleshooting

### Raft Cluster Issues

**Problem**: Node not joining cluster

```bash
# Check logs
docker logs secreton-1 | grep -i raft

# Verify peer configuration
echo $SECRETON_RAFT_PEERS

# Check network connectivity
curl http://secreton-2:7000/health
```

**Problem**: Leader election timeout

```bash
# Increase election timeout
SECRETON_RAFT_ELECTION_TIMEOUT_MS=3000

# Check heartbeat interval
SECRETON_RAFT_HEARTBEAT_INTERVAL_MS=300
```

### PostgreSQL Issues

**Problem**: Connection refused

```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Check connection string
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL -c "SELECT 1"
```

## TODO Items Completed

- [x] Add StorageConfig to ApiConfig
- [x] Add RaftConfig with full configuration options
- [x] Update services/mod.rs to use config storage backend
- [x] Set default backend to "raft" (HA)
- [x] Add storage configuration to default.toml
- [x] Add storage configuration to production.toml
- [x] Support environment variable overrides
- [x] Support multiple backend types
- [x] Create comprehensive documentation
- [x] Add deployment examples
- [x] Add monitoring examples
- [x] Add migration guide

## Next Steps

1. **Implement missing backends**:
   - [ ] Redis backend
   - [ ] Consul backend
   - [ ] S3 backend

2. **Add advanced features**:
   - [ ] Automatic backup
   - [ ] Disaster recovery
   - [ ] Cross-region replication
   - [ ] Encryption at rest

3. **Improve operations**:
   - [ ] Better monitoring
   - [ ] Automated failover
   - [ ] Health checks
   - [ ] Metrics collection

## References

- [Raft Consensus Algorithm](https://raft.github.io/)
- [HashiCorp Secret Vault Storage Backends](https://www.engineproject.io/docs/configuration/storage)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Distributed Systems](https://en.wikipedia.org/wiki/Distributed_computing)
