-- Migration for MFA Administrative Management Features
-- This migration adds tables and functions needed for comprehensive MFA administration

-- Add MFA policy management table
CREATE TABLE IF NOT EXISTS mfa_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_data JSONB NOT NULL,
    created_by UUID REFERENCES users(id),
    active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add index for active policies
CREATE INDEX IF NOT EXISTS idx_mfa_policies_active ON mfa_policies(active, created_at DESC);

-- Add MFA admin actions audit table
CREATE TABLE IF NOT EXISTS mfa_admin_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_user_id UUID REFERENCES users(id),
    target_user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    reason TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add indexes for admin actions
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_admin ON mfa_admin_actions(admin_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_target ON mfa_admin_actions(target_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_action ON mfa_admin_actions(action, created_at DESC);

-- Add require_mfa_setup column to users table if not exists
ALTER TABLE users
ADD COLUMN IF NOT EXISTS require_mfa_setup BOOLEAN NOT NULL DEFAULT false;

-- Add index for MFA setup requirement
CREATE INDEX IF NOT EXISTS idx_users_require_mfa_setup ON users(require_mfa_setup) WHERE require_mfa_setup = true;

-- Create materialized view for MFA statistics by satker
CREATE MATERIALIZED VIEW IF NOT EXISTS mfa_statistics AS
SELECT
    u.satker_code,
    COUNT(*) as total_users,
    COUNT(*) FILTER (WHERE u.mfa_enabled = true) as mfa_enabled_users,
    COUNT(*) FILTER (WHERE u.mfa_enabled = true AND u.mfa_setup_at IS NOT NULL) as mfa_setup_complete,
    COUNT(*) FILTER (WHERE u.mfa_enabled = true AND u.mfa_last_used >= NOW() - INTERVAL '30 days') as mfa_active_30d,
    COUNT(*) FILTER (WHERE u.mfa_enabled = true AND u.mfa_last_used >= NOW() - INTERVAL '7 days') as mfa_active_7d,
    COUNT(*) FILTER (WHERE u.mfa_enabled = true AND u.mfa_last_used >= NOW() - INTERVAL '1 day') as mfa_active_1d,
    MAX(u.updated_at) as last_updated
FROM users u
WHERE u.enabled = true AND u.deleted_at IS NULL
GROUP BY u.satker_code;

-- Create unique index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_mfa_statistics_satker ON mfa_statistics(satker_code);

-- Function to refresh MFA statistics
CREATE OR REPLACE FUNCTION refresh_mfa_statistics()
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mfa_statistics;
END;
$$;

-- Function to get batch MFA status (optimized for performance)
CREATE OR REPLACE FUNCTION get_batch_mfa_status(user_ids UUID[])
RETURNS TABLE(
    user_id UUID,
    mfa_enabled BOOLEAN,
    mfa_setup_at TIMESTAMP WITH TIME ZONE,
    mfa_last_used TIMESTAMP WITH TIME ZONE
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used
    FROM users u
    WHERE u.id = ANY(user_ids)
      AND u.enabled = true
      AND u.deleted_at IS NULL;
END;
$$;

-- Function to get user MFA status (optimized)
CREATE OR REPLACE FUNCTION get_user_mfa_status(p_user_id UUID)
RETURNS TABLE(
    mfa_enabled BOOLEAN,
    mfa_setup_at TIMESTAMP WITH TIME ZONE,
    mfa_last_used TIMESTAMP WITH TIME ZONE
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used
    FROM users u
    WHERE u.id = p_user_id
      AND u.enabled = true
      AND u.deleted_at IS NULL;
END;
$$;

-- Function to log MFA admin actions
CREATE OR REPLACE FUNCTION log_mfa_admin_action(
    p_admin_user_id UUID,
    p_target_user_id UUID,
    p_action VARCHAR(100),
    p_reason TEXT DEFAULT NULL,
    p_metadata JSONB DEFAULT NULL
)
RETURNS UUID
LANGUAGE plpgsql
AS $$
DECLARE
    action_id UUID;
BEGIN
    INSERT INTO mfa_admin_actions (
        admin_user_id,
        target_user_id,
        action,
        reason,
        metadata
    ) VALUES (
        p_admin_user_id,
        p_target_user_id,
        p_action,
        p_reason,
        p_metadata
    ) RETURNING id INTO action_id;

    RETURN action_id;
END;
$$;

-- Function to get MFA compliance summary
CREATE OR REPLACE FUNCTION get_mfa_compliance_summary(
    p_satker_codes TEXT[] DEFAULT NULL,
    p_role_names TEXT[] DEFAULT NULL
)
RETURNS TABLE(
    total_users BIGINT,
    mfa_enabled_users BIGINT,
    compliance_percentage NUMERIC(5,2),
    non_compliant_users BIGINT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    WITH filtered_users AS (
        SELECT u.id, u.mfa_enabled
        FROM users u
        LEFT JOIN user_roles ur ON u.id = ur.user_id
        LEFT JOIN roles r ON ur.role_id = r.id
        WHERE u.enabled = true
          AND u.deleted_at IS NULL
          AND (
              p_satker_codes IS NULL
              OR u.satker_code = ANY(p_satker_codes)
          )
          AND (
              p_role_names IS NULL
              OR r.name = ANY(p_role_names)
          )
    )
    SELECT
        COUNT(*)::BIGINT as total_users,
        COUNT(*) FILTER (WHERE mfa_enabled = true)::BIGINT as mfa_enabled_users,
        CASE
            WHEN COUNT(*) > 0 THEN
                ROUND((COUNT(*) FILTER (WHERE mfa_enabled = true)::NUMERIC / COUNT(*)::NUMERIC) * 100, 2)
            ELSE 0
        END as compliance_percentage,
        COUNT(*) FILTER (WHERE mfa_enabled = false)::BIGINT as non_compliant_users
    FROM filtered_users;
END;
$$;

-- Function to get MFA usage trends
CREATE OR REPLACE FUNCTION get_mfa_usage_trends(
    p_days INTEGER DEFAULT 30
)
RETURNS TABLE(
    date DATE,
    successful_verifications BIGINT,
    failed_verifications BIGINT,
    unique_users BIGINT,
    new_setups BIGINT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        DATE(al.created_at) as date,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_verification_success')::BIGINT as successful_verifications,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_verification_failed')::BIGINT as failed_verifications,
        COUNT(DISTINCT al.user_id) FILTER (WHERE al.event_type IN ('mfa_verification_success', 'mfa_verification_failed'))::BIGINT as unique_users,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_setup_complete')::BIGINT as new_setups
    FROM audit_logs al
    WHERE al.created_at >= NOW() - (p_days || ' days')::INTERVAL
      AND al.event_type IN ('mfa_verification_success', 'mfa_verification_failed', 'mfa_setup_complete')
    GROUP BY DATE(al.created_at)
    ORDER BY date;
END;
$$;

-- Create trigger to automatically refresh statistics periodically
CREATE OR REPLACE FUNCTION trigger_refresh_mfa_statistics()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- Only refresh if it's been more than 5 minutes since last refresh
    IF (
        SELECT EXTRACT(EPOCH FROM (NOW() - MAX(last_updated)))
        FROM mfa_statistics
        WHERE last_updated IS NOT NULL
    ) > 300 OR NOT EXISTS (SELECT 1 FROM mfa_statistics) THEN
        PERFORM refresh_mfa_statistics();
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- Create triggers on users table to refresh statistics
DROP TRIGGER IF EXISTS trigger_users_mfa_stats_refresh ON users;
CREATE TRIGGER trigger_users_mfa_stats_refresh
    AFTER INSERT OR UPDATE OF mfa_enabled, mfa_setup_at, mfa_last_used OR DELETE
    ON users
    FOR EACH STATEMENT
    EXECUTE FUNCTION trigger_refresh_mfa_statistics();

-- Insert default MFA policy if none exists
INSERT INTO mfa_policies (policy_data, active, created_at)
SELECT
    '{
        "enforce_for_all": true,
        "enforce_for_roles": ["admin", "security_admin"],
        "enforce_for_satkers": [],
        "grace_period_days": 30,
        "backup_codes_required": true,
        "max_failed_attempts": 5,
        "lockout_duration_minutes": 30
    }'::jsonb,
    true,
    NOW()
WHERE NOT EXISTS (SELECT 1 FROM mfa_policies WHERE active = true);

-- Create indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_audit_logs_mfa_events ON audit_logs(event_type, created_at)
WHERE event_type IN ('mfa_verification_success', 'mfa_verification_failed', 'mfa_setup_complete');

CREATE INDEX IF NOT EXISTS idx_audit_logs_user_mfa ON audit_logs(user_id, event_type, created_at)
WHERE event_type LIKE 'mfa_%';

-- Add comments for documentation
COMMENT ON TABLE mfa_policies IS 'Stores MFA policy configurations for the organization';
COMMENT ON TABLE mfa_admin_actions IS 'Audit log for all MFA administrative actions';
COMMENT ON MATERIALIZED VIEW mfa_statistics IS 'Aggregated MFA statistics by satker for reporting';
COMMENT ON FUNCTION refresh_mfa_statistics() IS 'Refreshes the MFA statistics materialized view';
COMMENT ON FUNCTION get_batch_mfa_status(UUID[]) IS 'Optimized function to get MFA status for multiple users';
COMMENT ON FUNCTION log_mfa_admin_action(UUID, UUID, VARCHAR, TEXT, JSONB) IS 'Logs MFA administrative actions for audit purposes';
