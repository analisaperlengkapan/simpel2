-- Migration: MFA Performance Optimization
-- Description: Adds optimized indexes and database improvements for MFA operations

-- Composite indexes for common MFA query patterns
CREATE INDEX IF NOT EXISTS idx_users_mfa_enabled_setup_at
    ON users(mfa_enabled, mfa_setup_at)
    WHERE mfa_enabled = true;

CREATE INDEX IF NOT EXISTS idx_users_mfa_enabled_last_used
    ON users(mfa_enabled, mfa_last_used)
    WHERE mfa_enabled = true;

-- Index for finding users who need MFA setup (enabled but no setup date)
CREATE INDEX IF NOT EXISTS idx_users_mfa_setup_required
    ON users(mfa_enabled, mfa_setup_at)
    WHERE mfa_enabled = true AND mfa_setup_at IS NULL;

-- Index for finding recently active MFA users (for monitoring)
CREATE INDEX IF NOT EXISTS idx_users_mfa_recent_activity
    ON users(mfa_last_used DESC)
    WHERE mfa_enabled = true AND mfa_last_used IS NOT NULL;

-- Composite index for user lookup with MFA status (common in authentication flow)
CREATE INDEX IF NOT EXISTS idx_users_username_mfa_enabled
    ON users(username, mfa_enabled);

CREATE INDEX IF NOT EXISTS idx_users_email_mfa_enabled
    ON users(email, mfa_enabled);

-- Index for satker-based MFA reporting and management
CREATE INDEX IF NOT EXISTS idx_users_satker_mfa_enabled
    ON users(satker_code, mfa_enabled);

-- Optimize MFA admin actions queries
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_target_action_created
    ON mfa_admin_actions(target_user_id, action, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_admin_created
    ON mfa_admin_actions(admin_user_id, created_at DESC);

-- Index for finding recent admin actions by action type
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_action_created
    ON mfa_admin_actions(action, created_at DESC);

-- Create a materialized view for MFA statistics (for reporting)
CREATE MATERIALIZED VIEW IF NOT EXISTS mfa_statistics AS
SELECT
    COUNT(*) as total_users,
    COUNT(*) FILTER (WHERE mfa_enabled = true) as mfa_enabled_users,
    COUNT(*) FILTER (WHERE mfa_enabled = true AND mfa_setup_at IS NOT NULL) as mfa_setup_complete,
    COUNT(*) FILTER (WHERE mfa_enabled = true AND mfa_last_used >= NOW() - INTERVAL '30 days') as mfa_active_30d,
    COUNT(*) FILTER (WHERE mfa_enabled = true AND mfa_last_used >= NOW() - INTERVAL '7 days') as mfa_active_7d,
    COUNT(*) FILTER (WHERE mfa_enabled = true AND mfa_last_used >= NOW() - INTERVAL '1 day') as mfa_active_1d,
    satker_code
FROM users
WHERE deleted_at IS NULL
GROUP BY satker_code;

-- Create unique index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_mfa_statistics_satker
    ON mfa_statistics(satker_code);

-- Create function to refresh MFA statistics
CREATE OR REPLACE FUNCTION refresh_mfa_statistics()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mfa_statistics;
END;
$$ LANGUAGE plpgsql;

-- Create a function for efficient MFA status lookup with caching hints
CREATE OR REPLACE FUNCTION get_user_mfa_status(p_user_id UUID)
RETURNS TABLE(
    mfa_enabled BOOLEAN,
    mfa_setup_at TIMESTAMP WITH TIME ZONE,
    mfa_last_used TIMESTAMP WITH TIME ZONE,
    username VARCHAR,
    satker_code VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used,
        u.username,
        u.satker_code
    FROM users u
    WHERE u.id = p_user_id
      AND u.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql STABLE;

-- Create function for batch MFA status lookup
CREATE OR REPLACE FUNCTION get_batch_mfa_status(p_user_ids UUID[])
RETURNS TABLE(
    user_id UUID,
    mfa_enabled BOOLEAN,
    mfa_setup_at TIMESTAMP WITH TIME ZONE,
    mfa_last_used TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used
    FROM users u
    WHERE u.id = ANY(p_user_ids)
      AND u.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql STABLE;

-- Create function for efficient MFA admin action logging
CREATE OR REPLACE FUNCTION log_mfa_admin_action(
    p_admin_user_id UUID,
    p_target_user_id UUID,
    p_action VARCHAR(50),
    p_reason TEXT
) RETURNS UUID AS $$
DECLARE
    action_id UUID;
BEGIN
    INSERT INTO mfa_admin_actions (admin_user_id, target_user_id, action, reason)
    VALUES (p_admin_user_id, p_target_user_id, p_action, p_reason)
    RETURNING id INTO action_id;

    RETURN action_id;
END;
$$ LANGUAGE plpgsql;

-- Add table statistics for query planner optimization
ANALYZE users;
ANALYZE mfa_admin_actions;

-- Add comments for documentation
COMMENT ON INDEX idx_users_mfa_enabled_setup_at IS 'Composite index for MFA-enabled users with setup timestamp';
COMMENT ON INDEX idx_users_mfa_enabled_last_used IS 'Composite index for MFA-enabled users with last used timestamp';
COMMENT ON INDEX idx_users_mfa_setup_required IS 'Partial index for users requiring MFA setup';
COMMENT ON INDEX idx_users_mfa_recent_activity IS 'Index for finding recently active MFA users';
COMMENT ON INDEX idx_users_username_mfa_enabled IS 'Composite index for authentication flow optimization';
COMMENT ON INDEX idx_users_satker_mfa_enabled IS 'Index for satker-based MFA reporting';

COMMENT ON MATERIALIZED VIEW mfa_statistics IS 'Materialized view for MFA adoption and usage statistics by satker';
COMMENT ON FUNCTION get_user_mfa_status(UUID) IS 'Optimized function for single user MFA status lookup';
COMMENT ON FUNCTION get_batch_mfa_status(UUID[]) IS 'Optimized function for batch MFA status lookup';
COMMENT ON FUNCTION log_mfa_admin_action(UUID, UUID, VARCHAR, TEXT) IS 'Optimized function for MFA admin action logging';
COMMENT ON FUNCTION refresh_mfa_statistics() IS 'Function to refresh MFA statistics materialized view';
