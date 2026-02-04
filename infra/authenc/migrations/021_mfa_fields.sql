SET search_path = authenc, public;
-- Migration: Add MFA fields to users table
-- Description: Adds MFA-related fields to support multi-factor authentication

-- Add MFA fields to users table
ALTER TABLE users
ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS mfa_setup_at TIMESTAMP WITH TIME ZONE NULL,
ADD COLUMN IF NOT EXISTS mfa_last_used TIMESTAMP WITH TIME ZONE NULL;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_mfa_enabled ON users(mfa_enabled);
CREATE INDEX IF NOT EXISTS idx_users_mfa_setup_at ON users(mfa_setup_at);
CREATE INDEX IF NOT EXISTS idx_users_mfa_last_used ON users(mfa_last_used);

-- Add comments for documentation
COMMENT ON COLUMN users.mfa_enabled IS 'Whether multi-factor authentication is enabled for this user';
COMMENT ON COLUMN users.mfa_setup_at IS 'Timestamp when MFA was first set up for this user';
COMMENT ON COLUMN users.mfa_last_used IS 'Timestamp when MFA was last used for authentication';
