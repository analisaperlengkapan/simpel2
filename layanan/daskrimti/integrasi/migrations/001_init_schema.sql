-- ============================================================================
-- SIMPelv2 - Layanan Integrasi Database Schema
-- ============================================================================
-- Purpose: Complete database schema for integration service
-- Modules: MonSAKTI (8 modules), MySIMKARI, SIMAN v2.0
-- ============================================================================

-- Create schema for isolation
CREATE SCHEMA IF NOT EXISTS integrasi;
SET search_path TO integrasi, public;

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm"; -- For text search optimization

-- ============================================================================
-- AUDIT & LOGGING TABLES (Actively Used)
-- ============================================================================

-- API Call Log - Tracks all external API calls
CREATE TABLE IF NOT EXISTS api_call_log (
    id BIGSERIAL PRIMARY KEY,

    -- Request Information
    module VARCHAR(20) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,
    kode_kl VARCHAR(10),
    kdsatker VARCHAR(20),
    full_url TEXT,

    -- Request Details (optional)
    request_method VARCHAR(10) DEFAULT 'GET',
    request_headers JSONB,
    request_params JSONB,

    -- Response Information
    response_status INTEGER,
    response_time_ms INTEGER,
    response_size_bytes INTEGER,
    record_count INTEGER,

    -- Result
    success BOOLEAN DEFAULT false,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,

    -- Token Management
    token_used VARCHAR(50),
    token_refreshed BOOLEAN DEFAULT false,
    new_token_received BOOLEAN DEFAULT false,

    -- Storage Metadata
    storage_strategy VARCHAR(20),
    data_saved BOOLEAN DEFAULT false,

    -- Timestamps
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Batch Processing Log - Tracks batch operations
CREATE TABLE IF NOT EXISTS batch_processing_log (
    id BIGSERIAL PRIMARY KEY,

    module VARCHAR(20) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,
    batch_size INTEGER NOT NULL,

    total_records INTEGER NOT NULL,
    successful_records INTEGER DEFAULT 0,
    failed_records INTEGER DEFAULT 0,

    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_ms INTEGER,

    error_summary TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Data Sync Log - Tracks data synchronization
CREATE TABLE IF NOT EXISTS data_sync_log (
    id BIGSERIAL PRIMARY KEY,

    table_name VARCHAR(100) NOT NULL,
    module VARCHAR(20) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,

    sync_type VARCHAR(20) DEFAULT 'full',  -- full, incremental, delta
    records_synced INTEGER DEFAULT 0,
    records_updated INTEGER DEFAULT 0,
    records_inserted INTEGER DEFAULT 0,
    records_deleted INTEGER DEFAULT 0,

    sync_started_at TIMESTAMPTZ NOT NULL,
    sync_completed_at TIMESTAMPTZ,
    sync_status VARCHAR(20) DEFAULT 'pending', -- pending, running, completed, failed

    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Token Reset Log - Tracks token refresh operations
CREATE TABLE IF NOT EXISTS token_reset_log (
    id BIGSERIAL PRIMARY KEY,

    module VARCHAR(20) NOT NULL,
    old_token_prefix VARCHAR(50),
    new_token_prefix VARCHAR(50),

    reset_reason VARCHAR(100),
    reset_triggered_by VARCHAR(50) DEFAULT 'auto', -- auto, manual, expired

    success BOOLEAN DEFAULT false,
    error_message TEXT,

    reset_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- API Tokens - Stores current tokens for each module
CREATE TABLE IF NOT EXISTS api_tokens (
    id BIGSERIAL PRIMARY KEY,

    module VARCHAR(20) NOT NULL UNIQUE,
    token TEXT NOT NULL,
    token_type VARCHAR(20) DEFAULT 'bearer', -- bearer, oauth2, api_key

    issued_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    last_refreshed_at TIMESTAMPTZ,

    is_active BOOLEAN DEFAULT true,
    refresh_count INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI MODULE TABLES (Schema-only, data inserted dynamically)
-- ============================================================================

-- ADM Module: Reference Admin
CREATE TABLE IF NOT EXISTS adm_ref_admin (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,  -- Original ID from API

    kdsatker TEXT,
    nmsatker TEXT,
    kdunit TEXT,
    nmunit TEXT,
    kode_kl TEXT,

    raw_data JSONB,  -- Full API response

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ADM Module: Banks Reference
CREATE TABLE IF NOT EXISTS adm_ref_bank (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kdbank TEXT,
    nmbank TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ADM Module: SPP Types
CREATE TABLE IF NOT EXISTS adm_ref_jns_spp (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kdjnsspp TEXT,
    nmjnsspp TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Generic pattern for other MonSAKTI endpoints
-- Tables created dynamically based on endpoint pattern: {module}_{endpoint}
-- Examples: ang_data_ang, pem_data_spm, ben_data_setoran, etc.
-- All follow same structure: id, api_id, raw_data, timestamps

-- ============================================================================
-- MYSIMKARI TABLES
-- ============================================================================

-- MySIMKARI: Satker (Work Units)
CREATE TABLE IF NOT EXISTS mysimkari_satker (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kode_satker TEXT,
    nama_satker TEXT,
    alamat TEXT,
    telepon TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- MySIMKARI: Pegawai (Employees)
CREATE TABLE IF NOT EXISTS mysimkari_pegawai (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    nip TEXT,
    nama TEXT,
    satker_id TEXT,
    jabatan TEXT,
    golongan TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- SIMAN v2.0 TABLES (Asset Management)
-- ============================================================================

-- SIMAN: Tanah (Land)
CREATE TABLE IF NOT EXISTS siman_aset_tanah (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kode_barang TEXT,
    nama_barang TEXT,
    luas_m2 NUMERIC,
    alamat TEXT,
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- SIMAN: Gedung dan Bangunan
CREATE TABLE IF NOT EXISTS siman_aset_gedung_bangunan (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kode_barang TEXT,
    nama_barang TEXT,
    luas_m2 NUMERIC,
    alamat TEXT,
    kondisi TEXT,
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- SIMAN: Alat Besar (Heavy Equipment)
CREATE TABLE IF NOT EXISTS siman_aset_alat_besar (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kode_barang TEXT,
    nama_barang TEXT,
    merk TEXT,
    tipe TEXT,
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC,
    kondisi TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- SIMAN: Angkutan Bermotor (Vehicles)
CREATE TABLE IF NOT EXISTS siman_aset_angkutan_bermotor (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT UNIQUE,

    kode_barang TEXT,
    nama_barang TEXT,
    merk TEXT,
    tipe TEXT,
    no_polisi TEXT,
    no_bpkb TEXT,
    no_rangka TEXT,
    no_mesin TEXT,
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC,
    kondisi TEXT,

    raw_data JSONB,

    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Additional SIMAN tables (created as needed by scheduler)
-- Pattern: siman_aset_{category} with raw_data JSONB for flexibility

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- API Call Log Indexes
CREATE INDEX IF NOT EXISTS idx_api_call_log_module_endpoint ON api_call_log(module, endpoint);
CREATE INDEX IF NOT EXISTS idx_api_call_log_created_at ON api_call_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_call_log_success ON api_call_log(success, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_call_log_kdsatker ON api_call_log(kdsatker) WHERE kdsatker IS NOT NULL;

-- Batch Processing Log Indexes
CREATE INDEX IF NOT EXISTS idx_batch_log_module ON batch_processing_log(module, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_batch_log_start_time ON batch_processing_log(start_time DESC);

-- Data Sync Log Indexes
CREATE INDEX IF NOT EXISTS idx_sync_log_table ON data_sync_log(table_name, sync_status);
CREATE INDEX IF NOT EXISTS idx_sync_log_started ON data_sync_log(sync_started_at DESC);

-- API Tokens Indexes
CREATE INDEX IF NOT EXISTS idx_api_tokens_module ON api_tokens(module) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_tokens_expires ON api_tokens(expires_at) WHERE is_active = true;

-- MonSAKTI Indexes
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_kdsatker ON adm_ref_admin(kdsatker);
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_synced ON adm_ref_admin(synced_at DESC);

-- MySIMKARI Indexes
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_kode ON mysimkari_satker(kode_satker);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_nip ON mysimkari_pegawai(nip);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_satker ON mysimkari_pegawai(satker_id);

-- SIMAN Indexes
CREATE INDEX IF NOT EXISTS idx_siman_tanah_kode ON siman_aset_tanah(kode_barang);
CREATE INDEX IF NOT EXISTS idx_siman_gedung_kode ON siman_aset_gedung_bangunan(kode_barang);
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_kode ON siman_aset_alat_besar(kode_barang);
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_nopol ON siman_aset_angkutan_bermotor(no_polisi);

-- JSONB GIN indexes for fast JSON queries
CREATE INDEX IF NOT EXISTS idx_api_call_log_request_params ON api_call_log USING GIN(request_params);
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_raw_data ON adm_ref_admin USING GIN(raw_data);

-- ============================================================================
-- VIEWS FOR MONITORING & ANALYTICS
-- ============================================================================

-- View: Recent Failed API Calls
CREATE OR REPLACE VIEW v_recent_failed_calls AS
SELECT
    module,
    endpoint,
    error_message,
    retry_count,
    created_at,
    response_status,
    full_url
FROM api_call_log
WHERE success = false
ORDER BY created_at DESC
LIMIT 100;

-- View: Token Health Status
CREATE OR REPLACE VIEW v_token_health AS
SELECT
    t.module,
    t.is_active,
    t.expires_at,
    t.last_refreshed_at,
    t.refresh_count,
    CASE
        WHEN t.expires_at < CURRENT_TIMESTAMP THEN 'EXPIRED'
        WHEN t.expires_at < CURRENT_TIMESTAMP + INTERVAL '1 day' THEN 'EXPIRING_SOON'
        WHEN NOT t.is_active THEN 'INACTIVE'
        ELSE 'HEALTHY'
    END as health_status,
    (SELECT COUNT(*) FROM api_call_log WHERE module = t.module AND token_refreshed = true) as total_refreshes,
    (SELECT COUNT(*) FROM api_call_log WHERE module = t.module AND success = false AND created_at > CURRENT_TIMESTAMP - INTERVAL '1 hour') as recent_failures
FROM api_tokens t
ORDER BY
    CASE
        WHEN t.expires_at < CURRENT_TIMESTAMP THEN 1
        WHEN t.expires_at < CURRENT_TIMESTAMP + INTERVAL '1 day' THEN 2
        WHEN NOT t.is_active THEN 3
        ELSE 4
    END;

-- View: API Statistics by Module
CREATE OR REPLACE VIEW v_api_stats_by_module AS
SELECT
    module,
    endpoint,
    COUNT(*) as total_calls,
    COUNT(*) FILTER (WHERE success = true) as successful_calls,
    COUNT(*) FILTER (WHERE success = false) as failed_calls,
    AVG(response_time_ms) FILTER (WHERE response_time_ms IS NOT NULL) as avg_response_time_ms,
    SUM(record_count) FILTER (WHERE record_count IS NOT NULL) as total_records_fetched,
    MAX(created_at) as last_called_at
FROM api_call_log
WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
GROUP BY module, endpoint
ORDER BY total_calls DESC;

-- ============================================================================
-- TRIGGERS FOR AUTO-UPDATE
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to all relevant tables
CREATE TRIGGER update_adm_ref_admin_updated_at BEFORE UPDATE ON adm_ref_admin FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_adm_ref_bank_updated_at BEFORE UPDATE ON adm_ref_bank FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_adm_ref_jns_spp_updated_at BEFORE UPDATE ON adm_ref_jns_spp FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_mysimkari_satker_updated_at BEFORE UPDATE ON mysimkari_satker FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_mysimkari_pegawai_updated_at BEFORE UPDATE ON mysimkari_pegawai FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_siman_tanah_updated_at BEFORE UPDATE ON siman_aset_tanah FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_siman_gedung_updated_at BEFORE UPDATE ON siman_aset_gedung_bangunan FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_siman_alat_besar_updated_at BEFORE UPDATE ON siman_aset_alat_besar FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_siman_angkutan_updated_at BEFORE UPDATE ON siman_aset_angkutan_bermotor FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_api_tokens_updated_at BEFORE UPDATE ON api_tokens FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON SCHEMA integrasi IS 'Integration service database schema for external API data';

COMMENT ON TABLE api_call_log IS 'Audit log of all external API calls with detailed request/response tracking';
COMMENT ON TABLE batch_processing_log IS 'Tracks batch processing operations for bulk data sync';
COMMENT ON TABLE data_sync_log IS 'Synchronization log for data import/export operations';
COMMENT ON TABLE token_reset_log IS 'Audit trail for API token refresh/reset operations';
COMMENT ON TABLE api_tokens IS 'Current active tokens for all integrated APIs';

COMMENT ON TABLE adm_ref_admin IS 'MonSAKTI ADM: Reference data for administrative units';
COMMENT ON TABLE mysimkari_satker IS 'MySIMKARI: Work unit (satker) master data';
COMMENT ON TABLE mysimkari_pegawai IS 'MySIMKARI: Employee master data';

COMMENT ON TABLE siman_aset_tanah IS 'SIMAN: Land assets (Barang Milik Negara - Tanah)';
COMMENT ON TABLE siman_aset_gedung_bangunan IS 'SIMAN: Building and construction assets';
COMMENT ON TABLE siman_aset_alat_besar IS 'SIMAN: Heavy equipment assets';
COMMENT ON TABLE siman_aset_angkutan_bermotor IS 'SIMAN: Vehicle assets';

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '✅ Integration service schema initialized successfully';
    RAISE NOTICE '📊 Created: 13 tables, 20+ indexes, 3 views, 9 triggers';
    RAISE NOTICE '🔍 Schema: integrasi';
END $$;
