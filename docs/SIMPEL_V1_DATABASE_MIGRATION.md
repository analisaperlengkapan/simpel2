# SIMPEL v1 Database Migration Guide

## Overview

SIMPEL v1 uses PostgreSQL database initialized from `dbsimpelv1.sql.gz` file. This guide documents the database migration strategy for different environments (local, staging, production).

## Database Structure

- **Database Name**: `dbsimpelv1`
- **Connection**: PostgreSQL 15+
- **Schema**: Public (default)
- **Backup Format**: gzip-compressed SQL dump
- **Backup Location**: See environment-specific sections below

## Local Development Setup

### Prerequisites

```bash
# Install PostgreSQL 15+
brew install postgresql@15  # macOS
sudo apt-get install postgresql-15  # Ubuntu/Debian
```

### Initial Setup

```bash
# Create database
createdb dbsimpelv1

# Restore from backup
./scripts/restore-db.sh dbsimpelv1.sql.gz dbsimpelv1 postgres localhost

# Verify
psql -d dbsimpelv1 -c "\dt"  # List tables
```

### Using Docker Compose

The local Docker Compose environment includes PostgreSQL. Database restoration happens automatically:

```bash
# Start services (includes db restore)
docker compose up -d

# Check logs
docker compose logs postgres

# Access database
docker exec -it postgres psql -U postgres -d dbsimpelv1
```

## Kubernetes Staging Environment

### Database Initialization

The v1 deployment includes an init-container that automatically handles database setup:

1. **Waits for PostgreSQL**: Polls connection until ready
2. **Restores Database**: Restores from `dbsimpelv1.sql.gz` if not already initialized
3. **Idempotent**: Checks if database has tables before restoration

### Deployment Process

```yaml
# infra/k8s/base/backend/simpelv1.yaml contains:

initContainers:
- name: db-migrate
  image: postgres:15-alpine
  command: ["sh", "-c"]
  args:
  - |
    gunzip < /db-backup/dbsimpelv1.sql.gz | \
    psql -h $DB_HOST -U $DB_USERNAME -d $DB_DATABASE
  volumeMounts:
  - name: db-backup
    mountPath: /db-backup
    readOnly: true

volumes:
- name: db-backup
  configMap:
    name: simpelv1-db-backup
```

### Upload Database Backup to Kubernetes

```bash
# Create ConfigMap with database backup
kubectl create configmap simpelv1-db-backup \
  --from-file=dbsimpelv1.sql.gz=/path/to/dbsimpelv1.sql.gz \
  -n simpelv2-staging

# For production (using Secret for size limits > 1MB):
kubectl create secret generic simpelv1-db-backup \
  --from-file=dbsimpelv1.sql.gz=/path/to/dbsimpelv1.sql.gz \
  -n simpelv2-production
```

### Verify Database Initialization

```bash
# Check pod logs
kubectl logs -n simpelv2-staging -l app=simpelv1 -c db-migrate --tail=50

# Access database from pod
kubectl exec -it -n simpelv2-staging pod/simpelv1-xxxxx -- \
  psql -h postgres -U postgres -d dbsimpelv1 -c "\dt"

# Verify tables count
kubectl exec -it -n simpelv2-staging pod/simpelv1-xxxxx -- \
  psql -h postgres -U postgres -d dbsimpelv1 -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';"
```

## Production Environment

### Database Backup Strategy

1. **Regular Backups**: Automated daily snapshots via pg_dump
2. **Retention**: Keep last 30 days of daily backups
3. **Encryption**: Store backups encrypted in S3/cloud storage

### Backup Creation

```bash
# Create full backup
pg_dump -h localhost -U postgres dbsimpelv1 | gzip > dbsimpelv1.sql.gz

# Backup with verbose output
pg_dump -h localhost -U postgres --verbose dbsimpelv1 | gzip > dbsimpelv1.sql.gz

# Backup specific schema
pg_dump -h localhost -U postgres -n public dbsimpelv1 | gzip > dbsimpelv1.sql.gz
```

### Restore from Production Backup

```bash
# Create new database
createdb dbsimpelv1-restore

# Restore backup
./scripts/restore-db.sh dbsimpelv1.sql.gz dbsimpelv1-restore postgres prod-postgres-host

# Verify integrity
psql -h prod-postgres-host -U postgres -d dbsimpelv1-restore -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';"
```

## Database Schema Updates

### Adding New Tables

1. **Development**: Add migration via Laravel artisan
2. **Test**: Verify schema on staging
3. **Production**: Apply migration with backup first

```bash
# Create backup before migration
pg_dump -h localhost -U postgres dbsimpelv1 | gzip > dbsimpelv1.sql.gz.backup

# Run Laravel migration
php artisan migrate

# Rollback if needed
pg_dump dbsimpelv1.sql.gz.backup | psql -h localhost -U postgres -d dbsimpelv1
```

### Monitoring Database

```bash
# Connection count
SELECT count(*) FROM pg_stat_activity;

# Database size
SELECT pg_database.datname, pg_size_pretty(pg_database_size(pg_database.datname)) FROM pg_database;

# Table sizes
SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) FROM pg_tables ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

# Active queries
SELECT pid, usename, application_name, state, query FROM pg_stat_activity WHERE state != 'idle';
```

## Troubleshooting

### Backup File Not Found

```bash
# Check if file exists
ls -lah dbsimpelv1.sql.gz

# Verify it's not corrupted
gunzip -t dbsimpelv1.sql.gz

# Get file info
file dbsimpelv1.sql.gz  # Should show "gzip compressed data"
```

### Connection Issues

```bash
# Test connection
psql -h localhost -U postgres -d dbsimpelv1 -c "SELECT 1;"

# Check PostgreSQL is running
systemctl status postgresql
docker ps | grep postgres

# Check credentials in .env
grep DB_ monolith/simpelv1/.env
```

### Restoration Fails

```bash
# Check database exists
psql -h localhost -U postgres -lc | grep dbsimpelv1

# Drop and recreate
dropdb -h localhost -U postgres dbsimpelv1
createdb -h localhost -U postgres dbsimpelv1

# Retry restoration
./scripts/restore-db.sh dbsimpelv1.sql.gz dbsimpelv1 postgres localhost
```

### Performance Issues

```bash
# Rebuild indexes
psql -h localhost -U postgres -d dbsimpelv1 -c "REINDEX DATABASE dbsimpelv1;"

# Vacuum and analyze
psql -h localhost -U postgres -d dbsimpelv1 -c "VACUUM ANALYZE;"

# Check slow queries
psql -h localhost -U postgres -d dbsimpelv1 -c "\d pg_stat_statements"
```

## Environment Variables

```env
# .env for Laravel v1
DB_CONNECTION=pgsql
DB_HOST=postgres
DB_PORT=5432
DB_DATABASE=dbsimpelv1
DB_USERNAME=postgres
DB_PASSWORD=<secure-password>

# .env for K8s (ConfigMap)
DB_CONNECTION=pgsql
DB_HOST=postgres.simpelv2-staging.svc.cluster.local
DB_PORT=5432
DB_DATABASE=dbsimpelv1
DB_USERNAME=postgres
DB_PASSWORD=<from-secret>
```

## References

- PostgreSQL Documentation: https://www.postgresql.org/docs/
- Laravel Database: https://laravel.com/docs/database
- Kubernetes ConfigMap: https://kubernetes.io/docs/concepts/configuration-storage/configmaps/
- Kubernetes Init Containers: https://kubernetes.io/docs/concepts/workloads/pods/init-containers/
