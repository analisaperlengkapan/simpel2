SET search_path = authenc, public;
-- Tamper-proof Audit Logging Migration
-- Adds HMAC-SHA256 signature support for audit trail integrity

-- Create events table if it doesn't exist
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    time TIMESTAMP NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    realm_id VARCHAR(255) NOT NULL,
    realm_name VARCHAR(255),
    client_id VARCHAR(255),
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    ip_address VARCHAR(45),
    error TEXT,
    details JSONB,
    signature VARCHAR(128), -- HMAC-SHA256 signature (64 bytes hex-encoded)
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create admin_events table if it doesn't exist
CREATE TABLE IF NOT EXISTS admin_events (
    id UUID PRIMARY KEY,
    time TIMESTAMP NOT NULL,
    realm_id VARCHAR(255) NOT NULL,
    realm_name VARCHAR(255),
    auth_user_id UUID,
    auth_username VARCHAR(255),
    auth_ip_address VARCHAR(45),
    auth_user_agent TEXT,
    resource_type VARCHAR(100) NOT NULL,
    operation_type VARCHAR(50) NOT NULL,
    resource_path TEXT NOT NULL,
    representation TEXT,
    error TEXT,
    signature VARCHAR(128), -- HMAC-SHA256 signature (64 bytes hex-encoded)
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add signature column to existing events table if it doesn't have it
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'events' AND column_name = 'signature'
    ) THEN
        ALTER TABLE events ADD COLUMN signature VARCHAR(128);
    END IF;
END $$;

-- Add signature column to existing admin_events table if it doesn't have it
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'admin_events' AND column_name = 'signature'
    ) THEN
        ALTER TABLE admin_events ADD COLUMN signature VARCHAR(128);
    END IF;
END $$;

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_events_time ON events(time DESC);
CREATE INDEX IF NOT EXISTS idx_events_realm_id ON events(realm_id);
CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_signature ON events(signature) WHERE signature IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_admin_events_time ON admin_events(time DESC);
CREATE INDEX IF NOT EXISTS idx_admin_events_realm_id ON admin_events(realm_id);
CREATE INDEX IF NOT EXISTS idx_admin_events_auth_user_id ON admin_events(auth_user_id);
CREATE INDEX IF NOT EXISTS idx_admin_events_resource_type ON admin_events(resource_type);
CREATE INDEX IF NOT EXISTS idx_admin_events_signature ON admin_events(signature) WHERE signature IS NOT NULL;

-- Create audit_integrity_checks table for tracking integrity verification runs
CREATE TABLE IF NOT EXISTS audit_integrity_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_time TIMESTAMP NOT NULL DEFAULT NOW(),
    events_checked INTEGER NOT NULL DEFAULT 0,
    admin_events_checked INTEGER NOT NULL DEFAULT 0,
    events_failed INTEGER NOT NULL DEFAULT 0,
    admin_events_failed INTEGER NOT NULL DEFAULT 0,
    check_duration_ms INTEGER,
    status VARCHAR(50) NOT NULL, -- 'success', 'failed', 'partial'
    error_details TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_integrity_checks_time ON audit_integrity_checks(check_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_checks_status ON audit_integrity_checks(status);

-- Create audit_integrity_failures table for tracking specific integrity failures
CREATE TABLE IF NOT EXISTS audit_integrity_failures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_id UUID NOT NULL REFERENCES audit_integrity_checks(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- 'event' or 'admin_event'
    event_id UUID NOT NULL,
    expected_signature VARCHAR(128),
    actual_signature VARCHAR(128),
    event_data JSONB,
    detected_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_integrity_failures_check_id ON audit_integrity_failures(check_id);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_failures_event_id ON audit_integrity_failures(event_id);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_failures_detected_at ON audit_integrity_failures(detected_at DESC);
