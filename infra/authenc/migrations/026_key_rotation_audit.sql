-- Migration: Key Rotation Audit System
-- Description: Creates tables for tracking automatic key rotation events
-- Version: 026
-- Date: 2024-10-30

-- Date: 2024-10-30
CREATE TABLE IF NOT EXISTS key_rotation_audit (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    key_id VARCHAR(255) NOT NULL,
    key_type VARCHAR(50) NOT NULL,
    old_version INTEGER NOT NULL,
    new_version INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    initiated_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_key_rotation_audit_key_id ON key_rotation_audit(key_id);
CREATE INDEX IF NOT EXISTS idx_key_rotation_audit_timestamp ON key_rotation_audit(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_key_rotation_audit_status ON key_rotation_audit(status);
CREATE INDEX IF NOT EXISTS idx_key_rotation_audit_key_type ON key_rotation_audit(key_type);

-- Create composite index for common queries
CREATE INDEX IF NOT EXISTS idx_key_rotation_audit_key_id_timestamp
    ON key_rotation_audit(key_id, timestamp DESC);

-- Add comments for documentation
COMMENT ON TABLE key_rotation_audit IS 'Audit log for automatic key rotation events';
COMMENT ON COLUMN key_rotation_audit.id IS 'Unique identifier for the rotation event';
COMMENT ON COLUMN key_rotation_audit.timestamp IS 'When the rotation occurred';
COMMENT ON COLUMN key_rotation_audit.key_id IS 'Identifier of the key that was rotated';
COMMENT ON COLUMN key_rotation_audit.key_type IS 'Type of key (jwt_signing, session_encryption, mfa_encryption, etc.)';
COMMENT ON COLUMN key_rotation_audit.old_version IS 'Previous version number of the key';
COMMENT ON COLUMN key_rotation_audit.new_version IS 'New version number after rotation';
COMMENT ON COLUMN key_rotation_audit.status IS 'Status of the rotation (Success, Failed, InProgress, Scheduled)';
COMMENT ON COLUMN key_rotation_audit.error_message IS 'Error message if rotation failed';
COMMENT ON COLUMN key_rotation_audit.initiated_by IS 'User or system that initiated the rotation';

-- Grant permissions (adjust as needed for your security model)
-- GRANT SELECT, INSERT ON key_rotation_audit TO authenc;
-- GRANT SELECT ON key_rotation_audit TO authenc_readonly;

