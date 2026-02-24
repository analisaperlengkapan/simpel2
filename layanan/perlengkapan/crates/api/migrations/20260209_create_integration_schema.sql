-- ============================================================================
-- Migration: Create Integration Schema and Tables
-- Description: Schema and tables for SIMAN, MySIMKARI, and MonSAKTI integration
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- Requirements: REQ-I001, REQ-I002, REQ-I008
-- ============================================================================

-- ============================================================================
-- CREATE INTEGRATION SCHEMA
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS integrasi;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

COMMENT ON SCHEMA integrasi IS 'Integration schema for external system data (SIMAN, MySIMKARI, MonSAKTI)';

-- ============================================================================
-- SIMAN INTEGRATION TABLES (Asset Management System)
-- Requirement: REQ-I001
-- ============================================================================

-- SIMAN: Tanah (Land Assets)
CREATE TABLE IF NOT EXISTS integrasi.siman_aset_tanah (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Asset Identification
    nup VARCHAR(50) NOT NULL UNIQUE,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,

    -- Land Specific Fields
    luas NUMERIC(15,2),
    satuan VARCHAR(20),
    alamat TEXT,
    provinsi VARCHAR(100),
    kabupaten_kota VARCHAR(100),

    -- Asset Condition
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC(15,2),

    -- Ownership
    satker_code VARCHAR(50),
    satker_nama VARCHAR(255),

    -- Raw Data (for debugging and future fields)
    raw_data JSONB NOT NULL,

    -- Sync Metadata
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for siman_aset_tanah
CREATE INDEX idx_siman_tanah_nup ON integrasi.siman_aset_tanah(nup);
CREATE INDEX idx_siman_tanah_kode ON integrasi.siman_aset_tanah(kode_barang);
CREATE INDEX idx_siman_tanah_satker ON integrasi.siman_aset_tanah(satker_code);
CREATE INDEX idx_siman_tanah_kondisi ON integrasi.siman_aset_tanah(kondisi);
CREATE INDEX idx_siman_tanah_synced ON integrasi.siman_aset_tanah(synced_at DESC);
CREATE INDEX idx_siman_tanah_raw ON integrasi.siman_aset_tanah USING GIN(raw_data);

COMMENT ON TABLE integrasi.siman_aset_tanah IS 'Land assets from SIMAN (Sistem Informasi Manajemen Aset Negara)';
COMMENT ON COLUMN integrasi.siman_aset_tanah.nup IS 'Nomor Urut Pendaftaran (unique asset registration number)';
COMMENT ON COLUMN integrasi.siman_aset_tanah.raw_data IS 'Complete raw JSON data from SIMAN API for debugging';

-- SIMAN: Gedung dan Bangunan (Buildings)
CREATE TABLE IF NOT EXISTS integrasi.siman_aset_gedung_bangunan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Asset Identification
    nup VARCHAR(50) NOT NULL UNIQUE,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,

    -- Building Specific Fields
    luas_lantai NUMERIC(15,2),
    jumlah_lantai INTEGER,
    alamat TEXT,
    provinsi VARCHAR(100),
    kabupaten_kota VARCHAR(100),

    -- Construction Details
    tahun_dibangun INTEGER,
    bahan_bangunan VARCHAR(100),

    -- Asset Condition
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC(15,2),

    -- Ownership
    satker_code VARCHAR(50),
    satker_nama VARCHAR(255),

    -- Raw Data
    raw_data JSONB NOT NULL,

    -- Sync Metadata
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for siman_aset_gedung_bangunan
CREATE INDEX idx_siman_gedung_nup ON integrasi.siman_aset_gedung_bangunan(nup);
CREATE INDEX idx_siman_gedung_kode ON integrasi.siman_aset_gedung_bangunan(kode_barang);
CREATE INDEX idx_siman_gedung_satker ON integrasi.siman_aset_gedung_bangunan(satker_code);
CREATE INDEX idx_siman_gedung_kondisi ON integrasi.siman_aset_gedung_bangunan(kondisi);
CREATE INDEX idx_siman_gedung_synced ON integrasi.siman_aset_gedung_bangunan(synced_at DESC);
CREATE INDEX idx_siman_gedung_raw ON integrasi.siman_aset_gedung_bangunan USING GIN(raw_data);

COMMENT ON TABLE integrasi.siman_aset_gedung_bangunan IS 'Building assets from SIMAN';

-- SIMAN: Alat Besar (Heavy Equipment)
CREATE TABLE IF NOT EXISTS integrasi.siman_aset_alat_besar (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Asset Identification
    nup VARCHAR(50) NOT NULL UNIQUE,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,

    -- Equipment Specific Fields
    merk VARCHAR(100),
    tipe VARCHAR(100),
    ukuran VARCHAR(50),
    bahan VARCHAR(100),

    -- Asset Condition
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC(15,2),

    -- Ownership
    satker_code VARCHAR(50),
    satker_nama VARCHAR(255),

    -- Raw Data
    raw_data JSONB NOT NULL,

    -- Sync Metadata
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for siman_aset_alat_besar
CREATE INDEX idx_siman_alat_nup ON integrasi.siman_aset_alat_besar(nup);
CREATE INDEX idx_siman_alat_kode ON integrasi.siman_aset_alat_besar(kode_barang);
CREATE INDEX idx_siman_alat_satker ON integrasi.siman_aset_alat_besar(satker_code);
CREATE INDEX idx_siman_alat_kondisi ON integrasi.siman_aset_alat_besar(kondisi);
CREATE INDEX idx_siman_alat_synced ON integrasi.siman_aset_alat_besar(synced_at DESC);
CREATE INDEX idx_siman_alat_raw ON integrasi.siman_aset_alat_besar USING GIN(raw_data);

COMMENT ON TABLE integrasi.siman_aset_alat_besar IS 'Heavy equipment assets from SIMAN';

-- SIMAN: Angkutan Bermotor (Motor Vehicles)
CREATE TABLE IF NOT EXISTS integrasi.siman_aset_angkutan_bermotor (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Asset Identification
    nup VARCHAR(50) NOT NULL UNIQUE,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,

    -- Vehicle Specific Fields
    merk VARCHAR(100),
    tipe VARCHAR(100),
    no_polisi VARCHAR(20),
    no_bpkb VARCHAR(50),
    no_rangka VARCHAR(50),
    no_mesin VARCHAR(50),
    warna VARCHAR(50),
    cc INTEGER,

    -- Asset Condition
    kondisi VARCHAR(50),
    tahun_perolehan INTEGER,
    nilai_perolehan NUMERIC(15,2),

    -- Ownership
    satker_code VARCHAR(50),
    satker_nama VARCHAR(255),

    -- Raw Data
    raw_data JSONB NOT NULL,

    -- Sync Metadata
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for siman_aset_angkutan_bermotor
CREATE INDEX idx_siman_kendaraan_nup ON integrasi.siman_aset_angkutan_bermotor(nup);
CREATE INDEX idx_siman_kendaraan_kode ON integrasi.siman_aset_angkutan_bermotor(kode_barang);
CREATE INDEX idx_siman_kendaraan_satker ON integrasi.siman_aset_angkutan_bermotor(satker_code);
CREATE INDEX idx_siman_kendaraan_kondisi ON integrasi.siman_aset_angkutan_bermotor(kondisi);
CREATE INDEX idx_siman_kendaraan_polisi ON integrasi.siman_aset_angkutan_bermotor(no_polisi) WHERE no_polisi IS NOT NULL;
CREATE INDEX idx_siman_kendaraan_synced ON integrasi.siman_aset_angkutan_bermotor(synced_at DESC);
CREATE INDEX idx_siman_kendaraan_raw ON integrasi.siman_aset_angkutan_bermotor USING GIN(raw_data);

COMMENT ON TABLE integrasi.siman_aset_angkutan_bermotor IS 'Motor vehicle assets from SIMAN';
COMMENT ON COLUMN integrasi.siman_aset_angkutan_bermotor.no_polisi IS 'Vehicle registration plate number';

-- ============================================================================
-- MYSIMKARI INTEGRATION TABLES (Personnel Management System)
-- Requirement: REQ-I002
-- ============================================================================

CREATE TABLE IF NOT EXISTS integrasi.mysimkari_pegawai (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Employee Identification
    nip VARCHAR(20) NOT NULL UNIQUE,
    nama VARCHAR(255) NOT NULL,

    -- Organization
    satker_id UUID,
    satker_code VARCHAR(50),
    satker_nama VARCHAR(255),

    -- Position
    jabatan VARCHAR(255),
    golongan VARCHAR(10),
    pangkat VARCHAR(100),
    eselon VARCHAR(10),

    -- Employment Status
    status_pegawai VARCHAR(50),
    jenis_pegawai VARCHAR(50),

    -- Contact Information
    email VARCHAR(255),
    no_hp VARCHAR(20),

    -- Personal Information
    tempat_lahir VARCHAR(100),
    tanggal_lahir DATE,
    jenis_kelamin VARCHAR(10),

    -- Address
    alamat TEXT,
    provinsi VARCHAR(100),
    kabupaten_kota VARCHAR(100),

    -- Employment Dates
    tmt_cpns DATE,
    tmt_pns DATE,
    tmt_jabatan DATE,
    tanggal_pensiun DATE,

    -- Raw Data (for debugging and future fields)
    raw_data JSONB NOT NULL,

    -- Sync Metadata
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for mysimkari_pegawai
CREATE INDEX idx_mysimkari_pegawai_nip ON integrasi.mysimkari_pegawai(nip);
CREATE INDEX idx_mysimkari_pegawai_nama ON integrasi.mysimkari_pegawai(nama);
CREATE INDEX idx_mysimkari_pegawai_satker_id ON integrasi.mysimkari_pegawai(satker_id);
CREATE INDEX idx_mysimkari_pegawai_satker_code ON integrasi.mysimkari_pegawai(satker_code);
CREATE INDEX idx_mysimkari_pegawai_status ON integrasi.mysimkari_pegawai(status_pegawai);
CREATE INDEX idx_mysimkari_pegawai_golongan ON integrasi.mysimkari_pegawai(golongan);
CREATE INDEX idx_mysimkari_pegawai_synced ON integrasi.mysimkari_pegawai(synced_at DESC);
CREATE INDEX idx_mysimkari_pegawai_raw ON integrasi.mysimkari_pegawai USING GIN(raw_data);

-- Full-text search index for employee name
CREATE INDEX idx_mysimkari_pegawai_nama_trgm ON integrasi.mysimkari_pegawai USING gin(nama gin_trgm_ops);

COMMENT ON TABLE integrasi.mysimkari_pegawai IS 'Employee data from MySIMKARI (Kejaksaan RI personnel system)';
COMMENT ON COLUMN integrasi.mysimkari_pegawai.nip IS 'Nomor Induk Pegawai (Employee ID)';
COMMENT ON COLUMN integrasi.mysimkari_pegawai.tmt_cpns IS 'Terhitung Mulai Tanggal CPNS';
COMMENT ON COLUMN integrasi.mysimkari_pegawai.tmt_pns IS 'Terhitung Mulai Tanggal PNS';

-- ============================================================================
-- API CALL LOGGING TABLE
-- Requirement: REQ-I008
-- ============================================================================

CREATE TABLE IF NOT EXISTS integrasi.api_call_log (
    id BIGSERIAL PRIMARY KEY,

    -- Service Information
    service_name VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,

    -- Request Details
    request_params JSONB,
    request_body JSONB,

    -- Response Details
    status_code INTEGER,
    response_body JSONB,

    -- Performance Metrics
    duration_ms INTEGER,

    -- Error Information
    error_message TEXT,
    error_details JSONB,

    -- Retry Information
    retry_count INTEGER DEFAULT 0,
    is_retry BOOLEAN DEFAULT FALSE,

    -- Timestamp
    called_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for api_call_log
CREATE INDEX idx_api_log_service ON integrasi.api_call_log(service_name, called_at DESC);
CREATE INDEX idx_api_log_endpoint ON integrasi.api_call_log(endpoint, called_at DESC);
CREATE INDEX idx_api_log_status ON integrasi.api_call_log(status_code);
CREATE INDEX idx_api_log_error ON integrasi.api_call_log(called_at DESC) WHERE error_message IS NOT NULL;
CREATE INDEX idx_api_log_duration ON integrasi.api_call_log(duration_ms DESC);
CREATE INDEX idx_api_log_called_at ON integrasi.api_call_log(called_at DESC);

COMMENT ON TABLE integrasi.api_call_log IS 'Detailed logging of all external API calls for monitoring and debugging';
COMMENT ON COLUMN integrasi.api_call_log.service_name IS 'External service: SIMAN, MySIMKARI, MonSAKTI';
COMMENT ON COLUMN integrasi.api_call_log.duration_ms IS 'API call duration in milliseconds';

-- ============================================================================
-- SYNC STATUS TABLE
-- Requirement: REQ-I008
-- ============================================================================

CREATE TABLE IF NOT EXISTS integrasi.sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Service Information
    service_name VARCHAR(50) NOT NULL,
    sync_type VARCHAR(20) NOT NULL CHECK (sync_type IN ('FULL', 'INCREMENTAL', 'ON_DEMAND')),

    -- Sync Target
    entity_type VARCHAR(50) NOT NULL,
    satker_code VARCHAR(50),

    -- Sync Status
    status VARCHAR(20) NOT NULL CHECK (status IN ('RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED')),

    -- Sync Statistics
    total_records INTEGER DEFAULT 0,
    processed_records INTEGER DEFAULT 0,
    success_records INTEGER DEFAULT 0,
    failed_records INTEGER DEFAULT 0,

    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_seconds INTEGER,

    -- Error Information
    error_message TEXT,
    error_details JSONB,
    failed_items JSONB DEFAULT '[]'::jsonb,

    -- Metadata
    triggered_by VARCHAR(50),
    triggered_by_user_id UUID,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint for running syncs
    CONSTRAINT unique_running_sync UNIQUE (service_name, entity_type, satker_code, status)
        WHERE status = 'RUNNING'
);

-- Indexes for sync_status
CREATE INDEX idx_sync_status_service ON integrasi.sync_status(service_name, started_at DESC);
CREATE INDEX idx_sync_status_entity ON integrasi.sync_status(entity_type, started_at DESC);
CREATE INDEX idx_sync_status_status ON integrasi.sync_status(status);
CREATE INDEX idx_sync_status_satker ON integrasi.sync_status(satker_code, started_at DESC) WHERE satker_code IS NOT NULL;
CREATE INDEX idx_sync_status_started ON integrasi.sync_status(started_at DESC);
CREATE INDEX idx_sync_status_failed ON integrasi.sync_status(started_at DESC) WHERE status = 'FAILED';

COMMENT ON TABLE integrasi.sync_status IS 'Tracking status and history of data synchronization jobs';
COMMENT ON COLUMN integrasi.sync_status.sync_type IS 'FULL: complete sync, INCREMENTAL: changes only, ON_DEMAND: manual trigger';
COMMENT ON COLUMN integrasi.sync_status.entity_type IS 'Type of data being synced: aset_tanah, aset_gedung, pegawai, etc.';

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION integrasi.update_entity_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to all integration tables
CREATE TRIGGER trg_siman_tanah_updated_at
    BEFORE UPDATE ON integrasi.siman_aset_tanah
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

CREATE TRIGGER trg_siman_gedung_updated_at
    BEFORE UPDATE ON integrasi.siman_aset_gedung_bangunan
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

CREATE TRIGGER trg_siman_alat_updated_at
    BEFORE UPDATE ON integrasi.siman_aset_alat_besar
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

CREATE TRIGGER trg_siman_kendaraan_updated_at
    BEFORE UPDATE ON integrasi.siman_aset_angkutan_bermotor
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

CREATE TRIGGER trg_mysimkari_pegawai_updated_at
    BEFORE UPDATE ON integrasi.mysimkari_pegawai
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

CREATE TRIGGER trg_sync_status_updated_at
    BEFORE UPDATE ON integrasi.sync_status
    FOR EACH ROW EXECUTE FUNCTION integrasi.update_entity_updated_at();

-- ============================================================================
-- VIEWS FOR INTEGRATION MONITORING
-- ============================================================================

-- View: Latest Sync Status per Service
CREATE OR REPLACE VIEW integrasi.v_latest_sync_status AS
SELECT DISTINCT ON (service_name, entity_type, satker_code)
    id,
    service_name,
    sync_type,
    entity_type,
    satker_code,
    status,
    total_records,
    processed_records,
    success_records,
    failed_records,
    started_at,
    completed_at,
    duration_seconds,
    error_message,
    CASE
        WHEN status = 'RUNNING' THEN 'IN_PROGRESS'
        WHEN status = 'COMPLETED' AND failed_records = 0 THEN 'SUCCESS'
        WHEN status = 'COMPLETED' AND failed_records > 0 THEN 'PARTIAL_SUCCESS'
        WHEN status = 'FAILED' THEN 'FAILED'
        ELSE 'UNKNOWN'
    END as sync_result
FROM integrasi.sync_status
ORDER BY service_name, entity_type, satker_code, started_at DESC;

COMMENT ON VIEW integrasi.v_latest_sync_status IS 'Latest synchronization status for each service and entity type';

-- View: API Call Statistics
CREATE OR REPLACE VIEW integrasi.v_api_call_statistics AS
SELECT
    service_name,
    endpoint,
    DATE_TRUNC('hour', called_at) as hour,
    COUNT(*) as total_calls,
    COUNT(*) FILTER (WHERE status_code >= 200 AND status_code < 300) as success_calls,
    COUNT(*) FILTER (WHERE status_code >= 400) as error_calls,
    AVG(duration_ms) as avg_duration_ms,
    MAX(duration_ms) as max_duration_ms,
    MIN(duration_ms) as min_duration_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_duration_ms
FROM integrasi.api_call_log
WHERE called_at >= NOW() - INTERVAL '24 hours'
GROUP BY service_name, endpoint, DATE_TRUNC('hour', called_at)
ORDER BY hour DESC, service_name, endpoint;

COMMENT ON VIEW integrasi.v_api_call_statistics IS 'Hourly API call statistics for the last 24 hours';

-- View: Integration Health Dashboard
CREATE OR REPLACE VIEW integrasi.v_integration_health AS
SELECT
    service_name,
    entity_type,
    MAX(started_at) as last_sync_time,
    MAX(CASE WHEN status = 'COMPLETED' THEN started_at END) as last_successful_sync,
    COUNT(*) FILTER (WHERE status = 'FAILED' AND started_at >= NOW() - INTERVAL '24 hours') as failures_last_24h,
    COUNT(*) FILTER (WHERE status = 'RUNNING') as currently_running,
    CASE
        WHEN MAX(started_at) < NOW() - INTERVAL '1 day' THEN 'STALE'
        WHEN COUNT(*) FILTER (WHERE status = 'FAILED' AND started_at >= NOW() - INTERVAL '24 hours') > 3 THEN 'UNHEALTHY'
        WHEN COUNT(*) FILTER (WHERE status = 'RUNNING') > 0 THEN 'SYNCING'
        ELSE 'HEALTHY'
    END as health_status
FROM integrasi.sync_status
GROUP BY service_name, entity_type
ORDER BY service_name, entity_type;

COMMENT ON VIEW integrasi.v_integration_health IS 'Overall health status of integration services';

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function: Get asset count by satker and condition
CREATE OR REPLACE FUNCTION integrasi.get_asset_count_by_satker(
    p_satker_code VARCHAR,
    p_kode_barang VARCHAR DEFAULT NULL,
    p_kondisi VARCHAR DEFAULT NULL
)
RETURNS TABLE (
    asset_type VARCHAR,
    total_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 'tanah'::VARCHAR, COUNT(*)
    FROM integrasi.siman_aset_tanah
    WHERE satker_code = p_satker_code
      AND (p_kode_barang IS NULL OR kode_barang = p_kode_barang)
      AND (p_kondisi IS NULL OR kondisi = p_kondisi)

    UNION ALL

    SELECT 'gedung_bangunan'::VARCHAR, COUNT(*)
    FROM integrasi.siman_aset_gedung_bangunan
    WHERE satker_code = p_satker_code
      AND (p_kode_barang IS NULL OR kode_barang = p_kode_barang)
      AND (p_kondisi IS NULL OR kondisi = p_kondisi)

    UNION ALL

    SELECT 'alat_besar'::VARCHAR, COUNT(*)
    FROM integrasi.siman_aset_alat_besar
    WHERE satker_code = p_satker_code
      AND (p_kode_barang IS NULL OR kode_barang = p_kode_barang)
      AND (p_kondisi IS NULL OR kondisi = p_kondisi)

    UNION ALL

    SELECT 'angkutan_bermotor'::VARCHAR, COUNT(*)
    FROM integrasi.siman_aset_angkutan_bermotor
    WHERE satker_code = p_satker_code
      AND (p_kode_barang IS NULL OR kode_barang = p_kode_barang)
      AND (p_kondisi IS NULL OR kondisi = p_kondisi);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION integrasi.get_asset_count_by_satker IS 'Get asset count by satker, optionally filtered by kode_barang and kondisi';

-- Function: Get employee count by satker
CREATE OR REPLACE FUNCTION integrasi.get_employee_count_by_satker(
    p_satker_code VARCHAR,
    p_status_pegawai VARCHAR DEFAULT NULL
)
RETURNS BIGINT AS $$
DECLARE
    v_count BIGINT;
BEGIN
    SELECT COUNT(*)
    INTO v_count
    FROM integrasi.mysimkari_pegawai
    WHERE satker_code = p_satker_code
      AND (p_status_pegawai IS NULL OR status_pegawai = p_status_pegawai);

    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION integrasi.get_employee_count_by_satker IS 'Get employee count by satker, optionally filtered by status';

-- ============================================================================
-- DATA RETENTION POLICY
-- ============================================================================

-- Function: Clean old API call logs (keep last 90 days)
CREATE OR REPLACE FUNCTION integrasi.cleanup_old_api_logs()
RETURNS INTEGER AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM integrasi.api_call_log
    WHERE called_at < NOW() - INTERVAL '90 days';

    GET DIAGNOSTICS v_deleted = ROW_COUNT;

    RAISE NOTICE 'Deleted % old API call log records', v_deleted;
    RETURN v_deleted;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION integrasi.cleanup_old_api_logs IS 'Delete API call logs older than 90 days';

-- Function: Clean old sync status records (keep last 180 days)
CREATE OR REPLACE FUNCTION integrasi.cleanup_old_sync_status()
RETURNS INTEGER AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM integrasi.sync_status
    WHERE started_at < NOW() - INTERVAL '180 days'
      AND status IN ('COMPLETED', 'FAILED', 'CANCELLED');

    GET DIAGNOSTICS v_deleted = ROW_COUNT;

    RAISE NOTICE 'Deleted % old sync status records', v_deleted;
    RETURN v_deleted;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION integrasi.cleanup_old_sync_status IS 'Delete completed/failed sync status records older than 180 days';

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '✅ Integration schema and tables created successfully';
    RAISE NOTICE '📊 Created: 7 tables';
    RAISE NOTICE '  - siman_aset_tanah (land assets)';
    RAISE NOTICE '  - siman_aset_gedung_bangunan (building assets)';
    RAISE NOTICE '  - siman_aset_alat_besar (heavy equipment)';
    RAISE NOTICE '  - siman_aset_angkutan_bermotor (motor vehicles)';
    RAISE NOTICE '  - mysimkari_pegawai (employee data)';
    RAISE NOTICE '  - api_call_log (API call logging)';
    RAISE NOTICE '  - sync_status (synchronization tracking)';
    RAISE NOTICE '📈 Created: 50+ indexes (including GIN indexes for JSONB)';
    RAISE NOTICE '👁️ Created: 3 monitoring views';
    RAISE NOTICE '🔧 Created: 6 triggers for updated_at';
    RAISE NOTICE '⚙️ Created: 4 helper functions';
    RAISE NOTICE '🔍 Schema: integrasi';
    RAISE NOTICE '📝 Requirements: REQ-I001, REQ-I002, REQ-I008';
END $$;
