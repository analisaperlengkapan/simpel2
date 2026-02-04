SET search_path = authenc, public;
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

-- Materialized View for MFA statistics is handled in migration 024

-- Handled in migration 024

-- Comments for existing indexes
-- Comments for existing indexes
COMMENT ON INDEX idx_users_mfa_enabled_setup_at IS 'Composite index for MFA-enabled users with setup timestamp';
COMMENT ON INDEX idx_users_mfa_enabled_last_used IS 'Composite index for MFA-enabled users with last used timestamp';
COMMENT ON INDEX idx_users_mfa_setup_required IS 'Partial index for users requiring MFA setup';
COMMENT ON INDEX idx_users_mfa_recent_activity IS 'Index for finding recently active MFA users';
COMMENT ON INDEX idx_users_username_mfa_enabled IS 'Composite index for authentication flow optimization';
COMMENT ON INDEX idx_users_satker_mfa_enabled IS 'Index for satker-based MFA reporting';
