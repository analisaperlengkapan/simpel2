# SIMPEL Database Migration Guide

## Overview

This guide provides step-by-step instructions for migrating the SIMPEL database from the current state to the standardized schema with all performance optimizations.

**Migration Duration:** Approximately 2-4 hours (depending on data volume)
**Downtime Required:** Yes (maintenance window required)
**Rollback Available:** Yes (via backup restore)

## Prerequisites

- [ ] PostgreSQL 14+ installed
- [ ] Database backup tools available (`pg_dump`, `pg_restore`)
- [ ] Sufficient disk space (at least 3x current database size)
- [ ] Database credentials with superuser privileges
- [ ] Maintenance window scheduled
- [ ] All stakeholders notified

## Migration Phases

### Phase 1: Pre-Migration Preparation

#### 1.1 Environment Setup

```bash
# Set environment variables
export DB_NAME=simpelv2
export DB_USER=simpelv2
export DB_HOST=localhost
export DB_PORT=5432
export BACKUP_DIR=/var/backups/postgresql/simpelv2

# Create backup directory
mkdir -p $BACKUP_DIR
chmod 700 $BACKUP_DIR
```

#### 1.2 Pre-Migration Checks

```bash
# Check database connectivity
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT version();"

# Check current database size
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT pg_size_pretty(pg_database_size('$DB_NAME'));"

# Check available disk space
df -h $BACKUP_DIR

# Check current table count
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema');"
```

#### 1.3 Record Current State

```bash
# Export current record counts
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME > /tmp/pre_migration_counts.txt <<EOF
SELECT 'pengajuan_kebutuhan_bmn' as table_name, COUNT(*) as count FROM pengajuan_kebutuhan_bmn
UNION ALL
SELECT 'pengajuan_pakaian_dinas', COUNT(*) FROM pengajuan_pakaian_dinas;
EOF

cat /tmp/pre_migration_counts.txt
```

### Phase 2: Backup

#### 2.1 Full Database Backup

```bash
# Run backup script
cd /path/to/simpelv2/layanan/perlengkapan/crates/api/scripts
chmod +x backup_database.sh
./backup_database.sh

# Verify backup was created
ls -lh $BACKUP_DIR/simpelv2_full_*.sql.gz

# Verify checksum
sha256sum -c $BACKUP_DIR/simpelv2_full_*.sql.gz.sha256
```

#### 2.2 Test Backup Restore (Optional but Recommended)

```bash
# Create test database
createdb -h $DB_HOST -p $DB_PORT -U $DB_USER simpelv2_test

# Restore to test database
export DB_NAME=simpelv2_test
./restore_database.sh $BACKUP_DIR/simpelv2_full_*.sql.gz

# Verify test restore
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d simpelv2_test -c "SELECT COUNT(*) FROM information_schema.tables;"

# Drop test database
dropdb -h $DB_HOST -p $DB_PORT -U $DB_USER simpelv2_test

# Reset DB_NAME
export DB_NAME=simpelv2
```

### Phase 3: Migration Execution

#### 3.1 Stop Application Services

```bash
# Stop all application services that connect to the database
systemctl stop layanan-perlengkapan
systemctl stop antarmuka-perlengkapan

# Verify no active connections
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME' AND pid <> pg_backend_pid();"
```

#### 3.2 Run Migration Scripts

```bash
# Navigate to migrations directory
cd /path/to/simpelv2/layanan/perlengkapan/crates/api/migrations

# Run migrations in order
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f 20260209_create_integration_schema.sql
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f 20260209_create_new_entity_tables.sql
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f 20260209_migrate_schema_standardization.sql
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f 20260209_create_dashboard_views.sql
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f 20260209_add_performance_indexes.sql
```

#### 3.3 Verify Migration

```bash
# Run verification script
cd ../scripts
chmod +x verify_migration.sh
./verify_migration.sh

# Check report
cat /tmp/simpelv2_migration_report_*.txt
```

### Phase 4: Post-Migration Tasks

#### 4.1 Update Statistics

```bash
# Analyze all tables for query planner
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "ANALYZE;"

# Vacuum to reclaim space
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "VACUUM ANALYZE;"
```

#### 4.2 Refresh Materialized Views

```bash
# Initial refresh of dashboard metrics
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "REFRESH MATERIALIZED VIEW perlengkapan.mv_dashboard_metrics;"
```

#### 4.3 Set Up Cron Jobs

```bash
# Add cron job for materialized view refresh (every 5 minutes)
crontab -e

# Add this line:
# */5 * * * * psql -h localhost -U simpelv2 -d simpelv2 -c "SELECT perlengkapan.refresh_dashboard_metrics();" >> /var/log/simpelv2/mv_refresh.log 2>&1
```

#### 4.4 Verify Data Integrity

```bash
# Compare record counts
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME > /tmp/post_migration_counts.txt <<EOF
SELECT 'pengajuan_kebutuhan_bmn' as table_name, COUNT(*) as count FROM perlengkapan.pengajuan_kebutuhan_bmn
UNION ALL
SELECT 'pengajuan_pakaian_dinas', COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas;
EOF

# Compare with pre-migration counts
diff /tmp/pre_migration_counts.txt /tmp/post_migration_counts.txt
```

### Phase 5: Application Restart

#### 5.1 Update Application Configuration

```bash
# Update database connection strings to use schema-qualified names
# Example: perlengkapan.pengajuan_kebutuhan_bmn instead of pengajuan_kebutuhan_bmn
```

#### 5.2 Start Application Services

```bash
# Start backend services
systemctl start layanan-perlengkapan

# Verify service is running
systemctl status layanan-perlengkapan

# Check logs for errors
journalctl -u layanan-perlengkapan -f

# Start frontend services
systemctl start antarmuka-perlengkapan
```

#### 5.3 Smoke Testing

```bash
# Test basic functionality
curl http://localhost:3020/api/v1/health

# Test database queries
curl http://localhost:3020/api/v1/kebutuhan-bmn?limit=10

# Test dashboard
curl http://localhost:3020/api/v1/dashboard/metrics
```

### Phase 6: Monitoring

#### 6.1 Monitor Performance

```bash
# Check query performance
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;"

# Check index usage
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT schemaname, tablename, indexname, idx_scan FROM pg_stat_user_indexes WHERE schemaname = 'perlengkapan' ORDER BY idx_scan DESC LIMIT 20;"

# Check cache hit ratio
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT sum(heap_blks_read) as heap_read, sum(heap_blks_hit) as heap_hit, sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) as ratio FROM pg_statio_user_tables;"
```

#### 6.2 Monitor Application Logs

```bash
# Watch for errors
tail -f /var/log/simpelv2/application.log | grep ERROR

# Watch for slow queries
tail -f /var/log/postgresql/postgresql-14-main.log | grep "duration:"
```

## Rollback Procedure

If migration fails or issues are discovered:

### Step 1: Stop Application

```bash
systemctl stop layanan-perlengkapan
systemctl stop antarmuka-perlengkapan
```

### Step 2: Restore from Backup

```bash
cd /path/to/simpelv2/layanan/perlengkapan/crates/api/scripts
./restore_database.sh $BACKUP_DIR/simpelv2_full_YYYYMMDD_HHMMSS.sql.gz
```

### Step 3: Restart Application

```bash
systemctl start layanan-perlengkapan
systemctl start antarmuka-perlengkapan
```

## Troubleshooting

### Issue: Migration script fails

**Solution:**
1. Check PostgreSQL logs: `tail -f /var/log/postgresql/postgresql-14-main.log`
2. Verify database connectivity
3. Check for insufficient permissions
4. Restore from backup and retry

### Issue: Performance degradation after migration

**Solution:**
1. Run `ANALYZE` on all tables
2. Check index usage with `pg_stat_user_indexes`
3. Review slow query log
4. Consider adding additional indexes

### Issue: Application cannot connect after migration

**Solution:**
1. Verify schema names in connection strings
2. Check search_path configuration
3. Verify user permissions on new schemas
4. Review application logs for specific errors

## Post-Migration Checklist

- [ ] All migration scripts executed successfully
- [ ] Verification script passed all checks
- [ ] Record counts match pre-migration
- [ ] No orphaned records found
- [ ] All foreign key constraints valid
- [ ] Indexes created and being used
- [ ] Materialized views refreshing automatically
- [ ] Application services running
- [ ] Smoke tests passed
- [ ] Performance metrics acceptable
- [ ] Backup retained for 30 days
- [ ] Documentation updated
- [ ] Stakeholders notified of completion

## Performance Benchmarks

Expected improvements after migration:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Dashboard load time | 8-10s | 2-3s | 70-75% |
| Search query time | 2-3s | 200-300ms | 85-90% |
| Gap analysis query | 15-20s | 1-2s | 90% |
| Workflow metrics | 5-7s | 500ms-1s | 85% |

## Support

For issues or questions:
- Check logs: `/var/log/simpelv2/`
- Review PostgreSQL logs: `/var/log/postgresql/`
- Contact: SIMPelv2 Team

## References

- Migration scripts: `layanan/perlengkapan/crates/api/migrations/`
- Backup scripts: `layanan/perlengkapan/crates/api/scripts/`
- Requirements: `.kiro/specs/simpel-completion/requirements.md`
- Design: `.kiro/specs/simpel-completion/design.md`
