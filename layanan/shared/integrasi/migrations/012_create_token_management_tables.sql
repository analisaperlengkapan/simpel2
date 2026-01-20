-- Migration: 012_create_token_management_tables.sql
-- Description: Create tables for token management and history
-- Date: 2025-10-30

SET search_path TO integrasi, public;

-- =============================================
-- Token Storage Table
-- =============================================

CREATE TABLE IF NOT EXISTS api_tokens (
    id SERIAL PRIMARY KEY,

    -- Token Information
    module VARCHAR(20) NOT NULL UNIQUE,    -- ADM, ANG, BEN, MYSIMKARI, etc.
    token_value TEXT NOT NULL,             -- Encrypted token value
    token_hash VARCHAR(64) NOT NULL,       -- SHA256 hash for verification

    -- Token Metadata
    kode_kl VARCHAR(10) DEFAULT '006',     -- KL code this token is for
    token_type VARCHAR(20) DEFAULT 'bearer', -- bearer, api_key, etc.

    -- Status
    is_active BOOLEAN DEFAULT true,
    is_expired BOOLEAN DEFAULT false,

    -- Usage Statistics
    usage_count INTEGER DEFAULT 0,
    last_used_at TIMESTAMP WITH TIME ZONE,
    last_success_at TIMESTAMP WITH TIME ZONE,
    last_failure_at TIMESTAMP WITH TIME ZONE,
    consecutive_failures INTEGER DEFAULT 0,

    -- Lifecycle
    issued_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    refreshed_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT chk_token_not_empty CHECK (length(token_value) > 0),
    CONSTRAINT chk_consecutive_failures CHECK (consecutive_failures >= 0)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_api_tokens_module ON api_tokens(module);
CREATE INDEX IF NOT EXISTS idx_api_tokens_active ON api_tokens(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_tokens_expired ON api_tokens(is_expired) WHERE is_expired = true;
CREATE INDEX IF NOT EXISTS idx_api_tokens_kode_kl ON api_tokens(kode_kl);

-- =============================================
-- Token History Table
-- =============================================

CREATE TABLE IF NOT EXISTS api_token_history (
    id BIGSERIAL PRIMARY KEY,

    -- Reference
    token_id INTEGER REFERENCES api_tokens(id) ON DELETE CASCADE,
    module VARCHAR(20) NOT NULL,

    -- Token Information (partial for security)
    token_prefix VARCHAR(20),              -- First 20 chars for identification
    token_suffix VARCHAR(20),              -- Last 20 chars for identification
    token_hash VARCHAR(64) NOT NULL,       -- Full hash for verification

    -- Event Information
    event_type VARCHAR(30) NOT NULL,       -- created, refreshed, expired, reset, revoked
    event_reason TEXT,                     -- Why this event happened
    triggered_by VARCHAR(50),              -- system, user, api, auto

    -- Context
    kode_kl VARCHAR(10),
    api_call_id BIGINT REFERENCES api_call_log(id),

    -- Old vs New (for refresh/reset)
    old_token_hash VARCHAR(64),
    new_token_hash VARCHAR(64),

    -- Metadata
    metadata JSONB,                        -- Additional context

    -- Timestamps
    event_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT chk_event_type CHECK (event_type IN (
        'created', 'refreshed', 'expired', 'reset', 'revoked',
        'auto_refreshed', 'manual_reset', 'failed_validation'
    ))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_token_history_token_id ON api_token_history(token_id);
CREATE INDEX IF NOT EXISTS idx_token_history_module ON api_token_history(module);
CREATE INDEX IF NOT EXISTS idx_token_history_event_type ON api_token_history(event_type);
CREATE INDEX IF NOT EXISTS idx_token_history_event_at ON api_token_history(event_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_history_api_call ON api_token_history(api_call_id);

-- =============================================
-- Token Reset Log Table
-- =============================================

CREATE TABLE IF NOT EXISTS token_reset_log (
    id BIGSERIAL PRIMARY KEY,

    -- Token Information
    module VARCHAR(20) NOT NULL,
    kode_kl VARCHAR(10) NOT NULL,

    -- Reset Details
    reset_reason VARCHAR(50) NOT NULL,     -- expired, failed, manual, scheduled
    reset_method VARCHAR(30) NOT NULL,     -- auto, manual, api

    -- Request Information
    reset_url TEXT,
    request_status INTEGER,                -- HTTP status of reset request

    -- Result
    success BOOLEAN DEFAULT false,
    error_message TEXT,
    new_token_received BOOLEAN DEFAULT false,

    -- Context
    api_call_id BIGINT REFERENCES api_call_log(id),
    triggered_by_user VARCHAR(100),

    -- Timestamps
    reset_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT chk_reset_reason CHECK (reset_reason IN (
        'expired', 'failed', 'manual', 'scheduled', 'auto', 'unauthorized', 'forbidden'
    )),
    CONSTRAINT chk_reset_method CHECK (reset_method IN ('auto', 'manual', 'api', 'scheduled'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_token_reset_module ON token_reset_log(module);
CREATE INDEX IF NOT EXISTS idx_token_reset_success ON token_reset_log(success);
CREATE INDEX IF NOT EXISTS idx_token_reset_at ON token_reset_log(reset_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_reset_reason ON token_reset_log(reset_reason);

-- =============================================
-- Token Rotation Policy Table
-- =============================================

CREATE TABLE IF NOT EXISTS token_rotation_policy (
    id SERIAL PRIMARY KEY,

    -- Policy Information
    module VARCHAR(20) NOT NULL UNIQUE,

    -- Rotation Settings
    rotation_enabled BOOLEAN DEFAULT true,
    rotation_interval_hours INTEGER DEFAULT 24,  -- Rotate every 24 hours
    max_usage_count INTEGER,                     -- Max uses before rotation
    max_consecutive_failures INTEGER DEFAULT 3,  -- Auto-reset after N failures

    -- Notification Settings
    notify_on_rotation BOOLEAN DEFAULT false,
    notify_on_failure BOOLEAN DEFAULT true,
    notification_email VARCHAR(255),

    -- Last Rotation
    last_rotation_at TIMESTAMP WITH TIME ZONE,
    next_rotation_at TIMESTAMP WITH TIME ZONE,

    -- Status
    is_active BOOLEAN DEFAULT true,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT chk_rotation_interval CHECK (rotation_interval_hours > 0),
    CONSTRAINT chk_max_failures CHECK (max_consecutive_failures > 0)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_rotation_policy_module ON token_rotation_policy(module);
CREATE INDEX IF NOT EXISTS idx_rotation_policy_active ON token_rotation_policy(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_rotation_policy_next_rotation ON token_rotation_policy(next_rotation_at);

-- =============================================
-- Triggers
-- =============================================

-- Trigger: Update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_api_tokens_updated_at
    BEFORE UPDATE ON api_tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_token_rotation_policy_updated_at
    BEFORE UPDATE ON token_rotation_policy
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger: Log token changes to history
CREATE OR REPLACE FUNCTION log_token_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO api_token_history (
            token_id, module, token_hash, event_type,
            event_reason, triggered_by, kode_kl, new_token_hash
        ) VALUES (
            NEW.id, NEW.module, NEW.token_hash, 'created',
            'Initial token creation', 'system', NEW.kode_kl, NEW.token_hash
        );
    ELSIF TG_OP = 'UPDATE' AND OLD.token_hash != NEW.token_hash THEN
        INSERT INTO api_token_history (
            token_id, module, token_hash, event_type,
            event_reason, triggered_by, kode_kl,
            old_token_hash, new_token_hash
        ) VALUES (
            NEW.id, NEW.module, NEW.token_hash, 'refreshed',
            'Token refreshed', 'system', NEW.kode_kl,
            OLD.token_hash, NEW.token_hash
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER log_api_token_changes
    AFTER INSERT OR UPDATE ON api_tokens
    FOR EACH ROW
    EXECUTE FUNCTION log_token_change();

-- =============================================
-- Views
-- =============================================

-- View: Active Tokens Summary
CREATE OR REPLACE VIEW v_active_tokens AS
SELECT
    module,
    kode_kl,
    is_active,
    is_expired,
    usage_count,
    consecutive_failures,
    last_used_at,
    last_success_at,
    last_failure_at,
    refreshed_at,
    expires_at,
    CASE
        WHEN expires_at IS NOT NULL AND expires_at < CURRENT_TIMESTAMP THEN true
        ELSE false
    END as is_actually_expired,
    CASE
        WHEN last_used_at IS NOT NULL THEN
            EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - last_used_at))/3600
        ELSE NULL
    END as hours_since_last_use
FROM api_tokens
WHERE is_active = true
ORDER BY module;

-- View: Token Health Status
CREATE OR REPLACE VIEW v_token_health AS
SELECT
    t.module,
    t.is_active,
    t.is_expired,
    t.usage_count,
    t.consecutive_failures,
    t.last_success_at,
    t.last_failure_at,
    p.max_consecutive_failures,
    p.rotation_enabled,
    p.next_rotation_at,
    CASE
        WHEN t.consecutive_failures >= p.max_consecutive_failures THEN 'critical'
        WHEN t.consecutive_failures > 0 THEN 'warning'
        WHEN t.is_expired THEN 'expired'
        WHEN NOT t.is_active THEN 'inactive'
        ELSE 'healthy'
    END as health_status
FROM api_tokens t
LEFT JOIN token_rotation_policy p ON t.module = p.module
ORDER BY
    CASE
        WHEN t.consecutive_failures >= p.max_consecutive_failures THEN 1
        WHEN t.is_expired THEN 2
        WHEN NOT t.is_active THEN 3
        ELSE 4
    END,
    t.module;

-- View: Recent Token Events
CREATE OR REPLACE VIEW v_recent_token_events AS
SELECT
    h.id,
    h.module,
    h.event_type,
    h.event_reason,
    h.triggered_by,
    h.event_at,
    a.endpoint as related_api_call,
    a.success as api_call_success
FROM api_token_history h
LEFT JOIN api_call_log a ON h.api_call_id = a.id
ORDER BY h.event_at DESC
LIMIT 100;

-- View: Token Reset Statistics
CREATE OR REPLACE VIEW v_token_reset_stats AS
SELECT
    module,
    reset_reason,
    COUNT(*) as reset_count,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_resets,
    SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed_resets,
    DATE(reset_at) as reset_date
FROM token_reset_log
GROUP BY module, reset_reason, DATE(reset_at)
ORDER BY reset_date DESC, module;

-- =============================================
-- Comments
-- =============================================

COMMENT ON TABLE api_tokens IS 'Current active tokens for each API module';
COMMENT ON TABLE api_token_history IS 'Complete history of token changes for audit trail';
COMMENT ON TABLE token_reset_log IS 'Log of all token reset operations';
COMMENT ON TABLE token_rotation_policy IS 'Token rotation policies per module';

COMMENT ON COLUMN api_tokens.token_value IS 'Encrypted token value - should be encrypted at application level';
COMMENT ON COLUMN api_tokens.token_hash IS 'SHA256 hash of token for verification without exposing token';
COMMENT ON COLUMN api_tokens.consecutive_failures IS 'Number of consecutive failures - auto-reset when threshold reached';

COMMENT ON VIEW v_active_tokens AS 'Summary of currently active tokens';
COMMENT ON VIEW v_token_health AS 'Health status of all tokens for monitoring';
COMMENT ON VIEW v_recent_token_events AS 'Recent token-related events for audit';
COMMENT ON VIEW v_token_reset_stats AS 'Statistics on token reset operations';

-- =============================================
-- Initial Data
-- =============================================

-- Insert default rotation policies for all modules
INSERT INTO token_rotation_policy (module, rotation_enabled, rotation_interval_hours, max_consecutive_failures)
VALUES
    ('ADM', true, 24, 3),
    ('ANG', true, 24, 3),
    ('BEN', true, 24, 3),
    ('PEM', true, 24, 3),
    ('KOM', true, 24, 3),
    ('AST', true, 24, 3),
    ('PER', true, 24, 3),
    ('GLP', true, 24, 3),
    ('MYSIMKARI', true, 24, 3)
ON CONFLICT (module) DO NOTHING;
