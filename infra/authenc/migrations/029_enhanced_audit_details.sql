-- Enhanced Audit Log Details Migration
-- Adds comprehensive audit context fields to event_log table

-- Add geolocation data column if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'geolocation_data'
    ) THEN
        ALTER TABLE event_log ADD COLUMN geolocation_data JSONB;
        COMMENT ON COLUMN event_log.geolocation_data IS 'Geolocation data for the IP address (country, city, coordinates)';
    END IF;
END $$;

-- Add request payload column if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'request_payload'
    ) THEN
        ALTER TABLE event_log ADD COLUMN request_payload JSONB;
        COMMENT ON COLUMN event_log.request_payload IS 'Sanitized request payload (PII removed)';
    END IF;
END $$;

-- Add response payload column if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'response_payload'
    ) THEN
        ALTER TABLE event_log ADD COLUMN response_payload JSONB;
        COMMENT ON COLUMN event_log.response_payload IS 'Sanitized response payload (PII removed)';
    END IF;
END $$;

-- Ensure user_agent column exists (should already exist from migration 017)
-- This is a safety check
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'user_agent'
    ) THEN
        ALTE event_log ADD COLUMN user_agent TEXT;
        COMMENT ON COLUMN event_log.user_agent IS 'User agent string from the client';
    END IF;
END $$;

-- Ensure ip_address column exists (should already exist from migration 017)
-- This is a safety check
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'ip_address'
    ) THEN
        ALTER TABLE event_log ADD COLUMN ip_address INET;
        COMMENT ON COLUMN event_log.ip_address IS 'IP address of the client';
    END IF;
END $$;

-- Ensure session_id column exists (should already exist from migration 017)
-- This is a safety check
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'session_id'
    ) THEN
        ALTER TABLE event_log ADD COLUMN session_id UUID;
        COMMENT ON COLUMN event_log.session_id IS 'Session ID for correlation';
    END IF;
END $$;

-- Ensure correlation_id column exists (should already exist from migration 017)
-- This is a safety check
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'event_log' AND column_name = 'correlation_id'
    ) THEN
        ALTERLE event_log ADD COLUMN correlation_id UUID;
        COMMENT ON COLUMN event_log.correlation_id IS 'Correlation ID for tracing related events';
    END IF;
END $$;

-- Create indexes for new columns to improve query performance
CREATE INDEX IF NOT EXISTS idx_event_log_geolocation ON event_log USING GIN (geolocation_data);
CREATE INDEX IF NOT EXISTS idx_event_log_request_payload ON event_log USING GIN (request_payload);
CREATE INDEX IF NOT EXISTS idx_event_log_response_payload ON event_log USING GIN (response_payload);

-- Add comments to the table
COMMENT ON TABLE event_log IS 'Comprehensive audit log with enhanced context including IP, user agent, geolocation, and sanitized payloads';

-- Update admin_audit_log table to ensure it has geolocation support
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'admin_audit_log' AND column_name = 'geolocation_data'
    ) THEN
        ALTER TABLE admin_audit_log ADD COLUMN geolocation_data JSONB;
        COMMENT ON COLUMN admin_audit_log.geolocation_data IS 'Geolocation data for the admin IP address';
    END IF;
END $$;

-- Ensure admin_audit_log has request/response payload columns
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'admin_audit_log' AND column_name = 'request_payload'
    ) THEN
        ALTER TABLE admin_audit_log ADD COLUMN request_payload JSONB;
        COMMENT ON COLUMN admin_audit_log.request_payload IS 'Sanitized request payload for admin operations';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'admin_audit_log' AND column_name = 'response_payload'
    ) THEN
        ALTER TABLE admin_audit_log ADD COLUMN response_payload JSONB;
        COMMENT ON COLUMN admin_audit_log.response_payload IS 'Sanitized response payload for admin operations';
    END IF;
END $$;

-- Create indexes for admin_audit_log
CREATE INDEX IF NOT EXISTS idx_admin_audit_log_geolocation ON admin_audit_log USING GIN (geolocation_data);
CREATE INDEX IF NOT EXISTS idx_admin_audit_log_request_payload ON admin_audit_log USING GIN (request_payload);
CREATE INDEX IF NOT EXISTS idx_admin_audit_log_response_payload ON admin_audit_log USING GIN (response_payload);

-- Update events table (for user events) to ensure enhanced audit fields
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'events' AND column_name = 'user_agent'
    ) THEN
        ALTER TABLE events ADD COLUMN user_agent TEXT;
        COMMENT ON COLUMN events.user_agent IS 'User agent string from the client';
    END IF;
END $$;

-- Update admin_events table to ensure enhanced audit fields
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'admin_events' AND column_name = 'geolocation_data'
    ) THEN
        ALTER TABLE admin_events ADD COLUMN geolocation_data JSONB;
        COMMENT ON COLUMN admin_events.geolocation_data IS 'Geolocation data for admin events';
    END IF;
END $$;

-- Create a view for comprehensive audit trail with all context
CREATE OR REPLACE VIEW comprehensive_audit_trail AS
SELECT
    id,
    created_at,
    event_type,
    event_category,
    resource_type,
    resource_id,
    user_id,
    username,
    ip_address,
    user_agent,
    session_id,
    correlation_id,
    geolocation_data,
    request_payload,
    response_payload,
    success,
    error_message,
    'event_log' as source_table
FROM event_log
UNION ALL
SELECT
    id,
    created_at,
    operation_type as event_type,
    'ADMIN' as event_category,
    resource_type,
    resource_id,
    auth_user_id as user_id,
    auth_username as username,
    auth_ip_address as ip_address,
    auth_user_agent as user_agent,
    NULL as session_id,
    NULL as correlation_id,
    geolocation_data,
    request_payload,
    response_payload,
    (error IS NULL) as success,
    error as error_message,
    'admin_audit_log' as source_table
FROM admin_audit_log
ORDER BY created_at DESC;

COMMENT ON VIEW comprehensive_audit_trail IS 'Unified view of all audit events with enhanced context from both user and admin logs';

