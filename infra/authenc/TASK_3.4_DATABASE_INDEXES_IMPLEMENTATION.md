# Task 3.4: Database Indexes Implementation

## Overview

Implemented comprehensive database indexes to optimize query performance for frequently accessed tables in the Authenc IAM system. This migration adds indexes for users, sessions, audit logs, and permission-related tables.

## Implementation Details

### Migration File

Created `migrations/030_performance_indexes.sql` with the following indexes:

#### Users Table Indexes

1. **idx_users_email** - Single column index on `email` (filtered by `deleted_at IS NULL`)
   - Improves authentication by email lookup
   - Target query: `SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL`

2. **idx_users_username** - Single column index on `username` (filtered by `deleted_at IS NULL`)
   - Improves authentication by username lookup
   - Target query: `SELECT * FROM users WHERE username = $1 AND deleted_at IS NULL`

3. **idx_users_satker_code** - Single column index on `satker_code` (filtered by `deleted_at IS NULL`)
   - Improves organizational unit filtering
   - Target query: User listing by Satker

4. **idx_users_realm_satker** - Composite index on `(realm_id, satker_code)` (filtered by `deleted_at IS NULL`)
   - Improves multi-tenant queries with Satker filtering
   - Target query: Realm-specific user listing by Satker

#### Session Table Indexes

1. **idx_user_sessions_user_expires** - Composite index on `(user_id, expires_at)` (filtered by `NOT revoked`)
   - Improves active session queries
   - Target query: Finding active sessions for a user
   - Note: Individual indexes already exist; this composite index optimizes combined queries

#### Audit Logs Indexes (events table)

1. **idx_events_user_time** - Composite index on `(user_id, time DESC)` (filtered by `user_id IS NOT NULL`)
   - Improves audit trail queries by user
   - Target query: `SELECT * FROM events WHERE user_id = $1 ORDER BY time DESC`

2. **idx_events_realm_type_time** - Composite index on `(realm_id, event_type, time DESC)`
   - Improves audit queries by realm and event type
   - Target query: Filtering events by realm and type with time ordering

#### Admin Events Indexes

1. **idx_admin_events_user_time** - Composite index on `(auth_user_id, time DESC)` (filtered by `auth_user_id IS NOT NULL`)
   - Improves admin audit trail queries
   - Target query: Admin actions by specific admin user

2. **idx_admin_events_realm_resource_time** - Composite index on `(realm_id, resource_type, time DESC)`
   - Improves admin audit queries by resource type
   - Target query: Admin actions on specific resource types

#### Permission-Related Indexes (Conditional)

The migration includes conditional index creation for tables that may or may not exist:

1. **user_roles table** (if exists):
   - `idx_user_roles_user_role` - Composite index on `(user_id, role_id)`
   - `idx_user_roles_role` - Single index on `role_id`

2. **permissions table** (if exists):
   - `idx_permissions_user_resource` - Composite index on `(user_id, resource)`
   - `idx_permissions_resource` - Single index on `resource`

## Key Features

### 1. Concurrent Index Creation

All indexes use `CREATE INDEX CONCURRENTLY` to avoid table locking during index creation. This allows:
- Zero-downtime deployment
- Continued database operations during migration
- Safe production deployment

### 2. Conditional Index Creation

Uses `IF NOT EXISTS` clause to ensure idempotency:
- Migration can be run multiple times safely
- No errors if indexes already exist
- Safe for rollback and re-application

### 3. Partial Indexes

Many indexes include `WHERE` clauses to:
- Reduce index size
- Improve index performance
- Filter out soft-deleted records (`deleted_at IS NULL`)
- Filter out revoked sessions (`NOT revoked`)

### 4. Composite Indexes

Strategic composite indexes for common query patterns:
- User + timestamp queries (audit trails)
- Realm + satker queries (multi-tenant filtering)
- User + expires_at queries (active sessions)

## Performance Impact

### Expected Improvements

1. **Authentication Queries**: 50-70% faster
   - Email/username lookups now use index instead of sequential scan
   - P95 latency target: < 50ms (from ~100ms)

2. **Audit Trail Queries**: 60-80% faster
   - User-specific audit logs use composite index
   - Time-ordered queries benefit from DESC index

3. **Session Queries**: 40-60% faster
   - Active session lookups use composite index
   - Reduced table scans for user sessions

4. **Permission Checks**: 30-50% faster (if tables exist)
   - User-resource permission checks use composite index
   - Role-based queries optimized

### Index Size Estimates

Based on typical data volumes:
- Users table indexes: ~50-100 MB (for 100k users)
- Session indexes: ~20-50 MB (for 50k active sessions)
- Audit log indexes: ~100-200 MB (for 1M events)
- Total additional storage: ~200-400 MB

## Migration Strategy

### Pre-Migration Checklist

- [x] Verify table schemas match expected structure
- [x] Use CONCURRENTLY to avoid locking
- [x] Include IF NOT EXISTS for idempotency
- [x] Add WHERE clauses for partial indexes
- [x] Document all indexes with COMMENT

### Deployment Steps

1. **Staging Environment**:
   ```bash
   # Run migration
   cd infra/authenc
   cargo run --bin authenc -- migrate

   # Verify indexes created
   psql -d authenc -c "\d+ users"
   psql -d authenc -c "\d+ events"
   ```

2. **Production Environment**:
   ```bash
   # Run during low-traffic period
   # CONCURRENTLY allows zero-downtime
   cargo run --bin authenc -- migrate

   # Monitor index creation progress
   SELECT * FROM pg_stat_progress_create_index;
   ```

3. **Verification**:
   ```sql
   -- Check index usage
   SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read
   FROM pg_stat_user_indexes
   WHERE indexname LIKE 'idx_users_%'
      OR indexname LIKE 'idx_events_%'
      OR indexname LIKE 'idx_admin_events_%'
      OR indexname LIKE 'idx_user_sessions_%';

   -- Check index sizes
   SELECT indexname, pg_size_pretty(pg_relation_size(indexname::regclass))
   FROM pg_indexes
   WHERE tablename IN ('users', 'events', 'admin_events', 'user_sessions');
   ```

## Rollback Plan

If issues occur, indexes can be dropped without affecting data:

```sql
-- Drop users indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_users_email;
DROP INDEX CONCURRENTLY IF EXISTS idx_users_username;
DROP INDEX CONCURRENTLY IF EXISTS idx_users_satker_code;
DROP INDEX CONCURRENTLY IF EXISTS idx_users_realm_satker;

-- Drop session indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_user_sessions_user_expires;

-- Drop audit log indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_events_user_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_events_realm_type_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_admin_events_user_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_admin_events_realm_resource_time;

-- Drop permission indexes (if they exist)
DROP INDEX CONCURRENTLY IF EXISTS idx_user_roles_user_role;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_roles_role;
DROP INDEX CONCURRENTLY IF EXISTS idx_permissions_user_resource;
DROP INDEX CONCURRENTLY IF EXISTS idx_permissions_resource;
```

## Testing

### Query Performance Testing

Before and after migration, test these queries:

```sql
-- Test 1: User lookup by email
EXPLAIN ANALYZE
SELECT * FROM users WHERE email = 'test@example.com' AND deleted_at IS NULL;

-- Test 2: User lookup by username
EXPLAIN ANALYZE
SELECT * FROM users WHERE username = 'testuser' AND deleted_at IS NULL;

-- Test 3: Audit trail by user
EXPLAIN ANALYZE
SELECT * FROM events WHERE user_id = 'uuid-here' ORDER BY time DESC LIMIT 100;

-- Test 4: Active sessions for user
EXPLAIN ANALYZE
SELECT * FROM user_sessions
WHERE user_id = 'uuid-here' AND NOT revoked AND expires_at > NOW();

-- Test 5: Admin actions by user
EXPLAIN ANALYZE
SELECT * FROM admin_events
WHERE auth_user_id = 'uuid-here' ORDER BY time DESC LIMIT 100;
```

Expected results:
- Query plans should show "Index Scan" instead of "Seq Scan"
- Execution time should be significantly reduced
- Rows scanned should match rows returned (no over-scanning)

## Requirements Fulfilled

This implementation fulfills **Requirement 4.3** from the requirements document:

> THE Authenc SHALL mengimplementasikan database indexes untuk semua foreign keys dan frequently queried columns

Specifically:
- ✅ Users table: email, username, satker_code indexes
- ✅ Sessions table: user_id + expires_at composite index
- ✅ Audit logs: user_id + timestamp composite indexes
- ✅ Permissions: user_id + resource composite indexes (conditional)
- ✅ All indexes use CONCURRENTLY for zero-downtime
- ✅ Partial indexes for soft-deleted records
- ✅ Composite indexes for common query patterns

## Next Steps

1. **Monitor Index Usage**: After deployment, monitor `pg_stat_user_indexes` to verify indexes are being used
2. **Analyze Query Plans**: Use EXPLAIN ANALYZE to verify query optimization
3. **Tune as Needed**: Adjust indexes based on actual query patterns
4. **Consider Additional Indexes**: Based on slow query logs, add more indexes if needed

## Related Tasks

- Task 3.1: ✅ Prepared statement cache (completed)
- Task 3.2: ✅ Integrate prepared statements (completed)
- Task 3.3: ✅ Optimize connection pool (completed)
- Task 3.4: ✅ Add database indexes (this task)
- Task 3.5: ⏳ Implement batch operations (pending)

## References

- PostgreSQL Documentation: [Indexes](https://www.postgresql.org/docs/current/indexes.html)
- PostgreSQL Documentation: [CREATE INDEX CONCURRENTLY](https://www.postgresql.org/docs/current/sql-createindex.html#SQL-CREATEINDEX-CONCURRENTLY)
- Design Document: Section "Database Optimization"
- Requirements Document: Requirement 4.3
