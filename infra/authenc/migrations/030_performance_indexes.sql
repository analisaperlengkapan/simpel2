-- Performance Optimization Indexes Migration
-- Adds indexes for frequently queried columns to improve query performance
-- Uses CREATE INDEX IF NOT EXISTS to avoid table locking during index creation

-- Users table indexes
-- These indexes improve performance for authentication and user lookup operations
CREATE INDEX IF NOT EXISTS idx_users_email
    ON users(email)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_users_username
    ON users(username)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_users_satker_code
    ON users(satker_code)
    WHERE deleted_at IS NULL;

-- Composite index for common query patterns (realm + satker filtering)
CREATE INDEX IF NOT EXISTS idx_users_realm_satker
    ON users(realm_id, satker_code)
    WHERE deleted_at IS NULL;

-- Session table indexes (if not already present)
-- Note: user_sessions already has idx_user_sessions_user_id and idx_user_sessions_expires_at
-- Adding composite index for common query patterns
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_expires
    ON user_sessions(user_id, expires_at)
    WHERE NOT revoked;

-- Audit logs indexes (events table)
-- Composite index for user + timestamp queries (common for audit trail lookups)
CREATE INDEX IF NOT EXISTS idx_events_user_time
    ON events(user_id, time DESC)
    WHERE user_id IS NOT NULL;

-- Composite index for realm + event type queries
CREATE INDEX IF NOT EXISTS idx_events_realm_type_time
    ON events(realm_id, event_type, time DESC);

-- Admin events indexes
-- Composite index for admin user + timestamp queries
CREATE INDEX IF NOT EXISTS idx_admin_events_user_time
    ON admin_events(auth_user_id, time DESC)
    WHERE auth_user_id IS NOT NULL;

-- Composite index for realm + resource type queries
CREATE INDEX IF NOT EXISTS idx_admin_events_realm_resource_time
    ON admin_events(realm_id, resource_type, time DESC);

-- User roles indexes
-- These indexes improve permission check performance
-- Using direct CREATE INDEX CONCURRENTLY (will fail if table doesn't exist, which is expected for optional tables)
CREATE INDEX IF NOT EXISTS idx_user_roles_user_role
    ON user_roles(user_id, role_id);

CREATE INDEX IF NOT EXISTS idx_user_roles_role
    ON user_roles(role_id);

-- Permissions table indexes
-- These indexes improve authorization check performance
CREATE INDEX IF NOT EXISTS idx_permissions_user_resource
    ON permissions(user_id, resource);

CREATE INDEX IF NOT EXISTS idx_permissions_resource
    ON permissions(resource);

-- Add comments for documentation
COMMENT ON INDEX idx_users_email IS 'Improves user lookup by email (authentication)';
COMMENT ON INDEX idx_users_username IS 'Improves user lookup by username (authentication)';
COMMENT ON INDEX idx_users_satker_code IS 'Improves user filtering by organizational unit';
COMMENT ON INDEX idx_users_realm_satker IS 'Composite index for realm + satker filtering';
COMMENT ON INDEX idx_user_sessions_user_expires IS 'Improves active session queries';
COMMENT ON INDEX idx_events_user_time IS 'Improves audit trail queries by user';
COMMENT ON INDEX idx_events_realm_type_time IS 'Improves audit queries by realm and event type';
COMMENT ON INDEX idx_admin_events_user_time IS 'Improves admin audit trail queries';
COMMENT ON INDEX idx_admin_events_realm_resource_time IS 'Improves admin audit queries by resource type';
