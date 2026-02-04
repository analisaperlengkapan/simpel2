SET search_path = authenc, public;
-- Migration: Add MFA admin actions table for audit logging
-- This table tracks administrative actions related to MFA management

CREATE TABLE IF NOT EXISTS mfa_admin_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_user_id UUID,
    target_user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    actor_type VARCHAR(50) NOT NULL DEFAULT 'user' CHECK (actor_type IN ('user', 'system', 'automated')),
    reason TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign key constraints
    CONSTRAINT fk_mfa_admin_actions_admin_user
        FOREIGN KEY (admin_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_mfa_admin_actions_target_user
        FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE CASCADE,

    -- Ensure admin_user_id is present for user actions
    CONSTRAINT check_admin_user_accountability
        CHECK ((actor_type = 'user' AND admin_user_id IS NOT NULL) OR (actor_type != 'user'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_admin_user_id
    ON mfa_admin_actions(admin_user_id);
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_target_user_id
    ON mfa_admin_actions(target_user_id);
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_created_at
    ON mfa_admin_actions(created_at);
CREATE INDEX IF NOT EXISTS idx_mfa_admin_actions_action
    ON mfa_admin_actions(action);

-- Add comments for documentation
COMMENT ON TABLE mfa_admin_actions IS 'Audit log for MFA administrative actions';
COMMENT ON COLUMN mfa_admin_actions.admin_user_id IS 'ID of the administrator performing the action (NULL for system actions)';
COMMENT ON COLUMN mfa_admin_actions.target_user_id IS 'ID of the user being affected by the action';
COMMENT ON COLUMN mfa_admin_actions.action IS 'Type of action performed (account_unlock, mfa_reset, etc.)';
COMMENT ON COLUMN mfa_admin_actions.reason IS 'Reason provided by the administrator for the action';
