-- Migration: 011_create_audit_log_tables.sql
-- Description: Create tables for API call logging and audit trail
-- Date: 2025-10-30

SET search_path TO integrasi, public;

-- =============================================
-- API Call Log Table
-- =============================================

CREATE TABLE IF NOT EXISTS api_call_log (
    id BIGSERIAL PRIMARY KEY,

    -- Request Information
    module VARCHAR(20) NOT NULL,           -- ADM, ANG, BEN, etc.
    endpoint VARCHAR(100) NOT NULL,        -- refAdmin, dataAng, etc.
    kode_kl VARCHAR(10),                   -- KL code (e.g., "006")
    kdsatker VARCHAR(20),                  -- Satker code
    full_url TEXT,                         -- Complete URL called

    -- Request Details
    request_method VARCHAR(10) DEFAULT 'GET',
    request_headers JSONB,                 -- Request headers (sanitized)
    request_params JSONB,                  -- Request parameters

    -- Response Information
    response_status INTEGER,               -- HTTP status code
    response_time_ms INTEGER,              -- Response time in milliseconds
    response_size_bytes INTEGER,           -- Response size
    record_count INTEGER,                  -- Number of records returned

    -- Result
    success BOOLEAN DEFAULT false,
    error_message TEXT,                    -- Error message if failed
    retry_count INTEGER DEFAULT 0,         -- Number of retries

    -- Token Information
    token_used VARCHAR(50),                -- First 50 chars of token (for tracking)
    token_refreshed BOOLEAN DEFAULT false, -- Was token refreshed?
    new_token_received BOOLEAN DEFAULT false, -- Did we receive new token?

    -- Metadata
    storage_strategy VARCHAR(20),          -- database, json, csv, both
    data_saved BOOLEAN DEFAULT false,      -- Was data successfully saved?

    -- Timestamps
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Indexes for common queries
    CONSTRAINT chk_response_status CHECK (response_status >= 100 AND response_status < 600)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_api_call_log_module ON api_call_log(module);
CREATE INDEX IF NOT EXISTS idx_api_call_log_endpoint ON api_call_log(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_call_log_kode_kl ON api_call_log(kode_kl);
CREATE INDEX IF NOT EXISTS idx_api_call_log_kdsatker ON api_call_log(kdsatker);
CREATE INDEX IF NOT EXISTS idx_api_call_log_success ON api_call_log(success);
CREATE INDEX IF NOT EXISTS idx_api_call_log_started_at ON api_call_log(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_call_log_module_endpoint ON api_call_log(module, endpoint);
CREATE INDEX IF NOT EXISTS idx_api_call_log_error ON api_call_log(success, error_message) WHERE success = false;

-- Partial index for failed calls (for quick error analysis)
CREATE INDEX IF NOT EXISTS idx_api_call_log_failed ON api_call_log(module, endpoint, started_at DESC)
WHERE success = false;

-- =============================================
-- Batch Processing Log Table
-- =============================================

CREATE TABLE IF NOT EXISTS batch_processing_log (
    id BIGSERIAL PRIMARY KEY,

    -- Batch Information
    batch_type VARCHAR(50) NOT NULL,       -- complete, satker, global, mysimkari
    kode_kl VARCHAR(10),
    kdsatker VARCHAR(20),                  -- NULL for batch operations

    -- Processing Details
    total_satker INTEGER,                  -- Total satker to process
    processed_satker INTEGER DEFAULT 0,    -- Successfully processed
    failed_satker INTEGER DEFAULT 0,       -- Failed to process

    -- Statistics
    total_api_calls INTEGER DEFAULT 0,
    successful_calls INTEGER DEFAULT 0,
    failed_calls INTEGER DEFAULT 0,
    total_records INTEGER DEFAULT 0,       -- Total records fetched

    -- Performance
    duration_seconds INTEGER,
    avg_response_time_ms INTEGER,

    -- Status
    status VARCHAR(20) NOT NULL,           -- running, completed, failed, cancelled
    error_message TEXT,

    -- Configuration
    storage_strategy VARCHAR(20),
    parallel_mode BOOLEAN DEFAULT false,

    -- Timestamps
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT chk_batch_status CHECK (status IN ('running', 'completed', 'failed', 'cancelled'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_batch_log_status ON batch_processing_log(status);
CREATE INDEX IF NOT EXISTS idx_batch_log_started_at ON batch_processing_log(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_batch_log_kode_kl ON batch_processing_log(kode_kl);
CREATE INDEX IF NOT EXISTS idx_batch_log_type ON batch_processing_log(batch_type);

-- =============================================
-- Data Sync Log Table
-- =============================================

CREATE TABLE IF NOT EXISTS data_sync_log (
    id BIGSERIAL PRIMARY KEY,

    -- Sync Information
    table_name VARCHAR(100) NOT NULL,
    module VARCHAR(20) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,

    -- Data Information
    records_fetched INTEGER DEFAULT 0,
    records_inserted INTEGER DEFAULT 0,
    records_updated INTEGER DEFAULT 0,
    records_failed INTEGER DEFAULT 0,

    -- Context
    kode_kl VARCHAR(10),
    kdsatker VARCHAR(20),
    batch_id BIGINT REFERENCES batch_processing_log(id),
    api_call_id BIGINT REFERENCES api_call_log(id),

    -- Status
    success BOOLEAN DEFAULT false,
    error_message TEXT,

    -- Timestamps
    synced_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_data_sync_log_table ON data_sync_log(table_name);
CREATE INDEX IF NOT EXISTS idx_data_sync_log_module ON data_sync_log(module, endpoint);
CREATE INDEX IF NOT EXISTS idx_data_sync_log_batch ON data_sync_log(batch_id);
CREATE INDEX IF NOT EXISTS idx_data_sync_log_api_call ON data_sync_log(api_call_id);
CREATE INDEX IF NOT EXISTS idx_data_sync_log_synced_at ON data_sync_log(synced_at DESC);

-- =============================================
-- Comments
-- =============================================

COMMENT ON TABLE api_call_log IS 'Log semua API calls ke MonSAKTI dan MySIMKARI untuk audit dan monitoring';
COMMENT ON TABLE batch_processing_log IS 'Log batch processing operations untuk tracking dan analytics';
COMMENT ON TABLE data_sync_log IS 'Log data synchronization ke database untuk audit trail';

COMMENT ON COLUMN api_call_log.token_used IS 'First 50 characters of token for tracking (not full token for security)';
COMMENT ON COLUMN api_call_log.response_time_ms IS 'Response time in milliseconds for performance monitoring';
COMMENT ON COLUMN api_call_log.retry_count IS 'Number of retries attempted (for token refresh, etc)';

COMMENT ON COLUMN batch_processing_log.status IS 'Status: running, completed, failed, cancelled';
COMMENT ON COLUMN batch_processing_log.parallel_mode IS 'Whether parallel processing was used';

COMMENT ON COLUMN data_sync_log.batch_id IS 'Reference to batch_processing_log if part of batch operation';
COMMENT ON COLUMN data_sync_log.api_call_id IS 'Reference to api_call_log for traceability';

-- =============================================
-- Views for Common Queries
-- =============================================

-- View: API Call Statistics by Module
CREATE OR REPLACE VIEW v_api_stats_by_module AS
SELECT
    module,
    COUNT(*) as total_calls,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_calls,
    SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed_calls,
    ROUND(AVG(response_time_ms), 2) as avg_response_time_ms,
    SUM(record_count) as total_records,
    DATE(started_at) as call_date
FROM api_call_log
GROUP BY module, DATE(started_at)
ORDER BY call_date DESC, module;

-- View: Recent Failed API Calls
CREATE OR REPLACE VIEW v_recent_failed_calls AS
SELECT
    id,
    module,
    endpoint,
    kode_kl,
    kdsatker,
    response_status,
    error_message,
    retry_count,
    started_at
FROM api_call_log
WHERE success = false
ORDER BY started_at DESC
LIMIT 100;

-- View: Batch Processing Summary
CREATE OR REPLACE VIEW v_batch_summary AS
SELECT
    id,
    batch_type,
    kode_kl,
    status,
    total_satker,
    processed_satker,
    failed_satker,
    total_api_calls,
    successful_calls,
    failed_calls,
    duration_seconds,
    started_at,
    completed_at
FROM batch_processing_log
ORDER BY started_at DESC;

-- View: Daily Sync Statistics
CREATE OR REPLACE VIEW v_daily_sync_stats AS
SELECT
    DATE(synced_at) as sync_date,
    table_name,
    module,
    COUNT(*) as sync_count,
    SUM(records_inserted) as total_inserted,
    SUM(records_updated) as total_updated,
    SUM(records_failed) as total_failed,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs
FROM data_sync_log
GROUP BY DATE(synced_at), table_name, module
ORDER BY sync_date DESC, table_name;

COMMENT ON VIEW v_api_stats_by_module IS 'API call statistics grouped by module and date';
COMMENT ON VIEW v_recent_failed_calls IS 'Most recent 100 failed API calls for quick debugging';
COMMENT ON VIEW v_batch_summary IS 'Summary of batch processing operations';
COMMENT ON VIEW v_daily_sync_stats IS 'Daily data synchronization statistics';
