-- ============================================================================
-- Migration: Enhance Integration Schema
-- Description: Add missing tables and enhance existing integration schema
-- Author: SIMPelv2 Team
-- Created: 2026-02-09
-- Requirements: REQ-I001, REQ-I002, REQ-I008
-- ============================================================================

-- Ensure schema exists
CREATE SCHEMA IF NOT EXISTS integrasi;
SET search_path TO integrasi, public;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================================
-- SYNC STATUS TABLE (NEW)
-- Tracks synchronization status per module/endpoint
-- ============================================================================

CREATE TABLE IF NOT EXISTS integrasi.sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Module Information
    module VARCHAR(50) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,

    -- Sync Configuration
    sync_type VARCHAR(20) NOT NULL DEFAULT 'full' CHECK (sync_type IN ('full', 'incremental', 'delta')),
    sync_frequency VARCHAR(50), -- e.g., 'daily', 'hourly', 'every_6_hours'

    -- Sync Status
    last_sync_started_at TIMESTAMPTZ,
    last_sync_completed_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20) DEFAULT 'pending' CHECK (last_sync_status IN ('pending', 'running', 'completed', 'failed')),
    last_sync_records_count INTEGER DEFAULT 0,
    last_sync_error TEXT,

    -- Next Sync Schedule
    next_sync_scheduled_at TIMESTAMPTZ,

    -- Statistics
    total_syncs_completed INTEGER DEFAULT 0,
    total_syncs_failed INTEGER DEFAULT 0,
    total_records_synced BIGINT DEFAULT 0,

    -- Metadata
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- Unique constraint per module/endpoint
    CONSTRAINT unique_module_endpoint UNIQUE (module, endpoint)
);

CREATE INDEX idx_sync_status_module ON integrasi.sync_status(module);
CREATE INDEX idx_sync_status_last_sync ON integrasi.sync_status(last_sync_completed_at DESC);
CREATE INDEX idx_sync_status_next_sync ON integrasi.sync_status(next_sync_scheduled_at) WHERE is_enabled = TRUE;
CREATE INDEX idx_sync_status_failed ON integrasi.sync_status(last_sync_status) WHERE last_sync_status = 'failed';

COMMENT ON TABLE integrasi.sync_status IS 'Tracks synchronization status and schedule for each integration module';

-- ============================================================================
-- ENHANCE EXISTING SIMAN TABLES
-- Add missing fields and improve structure
-- ============================================================================

-- Add satker_id to SIMAN tables for better filtering
ALTER TABLE integrasi.siman_aset_tanah
ADD COLUMN IF NOT EXISTS satker_id VARCHAR(20),
ADD COLUMN IF NOT EXISTS nup VARCHAR(50),
ADD COLUMN IF NOT EXISTS kondisi VARCHAR(20),
ADD COLUMN IF NOT EXISTS tahun_perolehan INTEGER;

ALTER TABLE integrasi.siman_aset_gedung_bangunan
ADD COLUMN IF NOT EXISTS satker_id VARCHAR(20),
ADD COLUMN IF NOT EXISTS nup VARCHAR(50);

ALTER TABLE integrasi.siman_aset_alat_besar
ADD COLUMN IF NOT EXISTS satker_id VARCHAR(20),
ADD COLUMN IF NOT EXISTS nup VARCHAR(50);

ALTER TABLE integrasi.siman_aset_angkutan_bermotor
ADD COLUMN IF NOT EXISTS satker_id VARCHAR(20),
ADD COLUMN IF NOT EXISTS nup VARCHAR(50);

-- Add indexes for satker_id filtering
CREATE INDEX IF NOT EXISTS idx_siman_tanah_satker ON integrasi.siman_aset_tanah(satker_id) WHERE satker_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_gedung_satker ON integrasi.siman_aset_gedung_bangunan(satker_id) WHERE satker_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_satker ON integrasi.siman_aset_alat_besar(satker_id) WHERE satker_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_satker ON integrasi.siman_aset_angkutan_bermotor(satker_id) WHERE satker_id IS NOT NULL;

-- Add indexes for NUP (Nomor Urut Pendaftaran)
CREATE INDEX IF NOT EXISTS idx_siman_tanah_nup ON integrasi.siman_aset_tanah(nup) WHERE nup IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_gedung_nup ON integrasi.siman_aset_gedung_bangunan(nup) WHERE nup IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_nup ON integrasi.siman_aset_alat_besar(nup) WHERE nup IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_nup ON integrasi.siman_aset_angkutan_bermotor(nup) WHERE nup IS NOT NULL;

-- Add indexes for kondisi (condition) filtering
CREATE INDEX IF NOT EXISTS idx_siman_tanah_kondisi ON integrasi.siman_aset_tanah(kondisi) WHERE kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_gedung_kondisi ON integrasi.siman_aset_gedung_bangunan(kondisi) WHERE kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_kondisi ON integrasi.siman_aset_alat_besar(kondisi) WHERE kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_kondisi ON integrasi.siman_aset_angkutan_bermotor(kondisi) WHERE kondisi IS NOT NULL;

-- ============================================================================
-- ENHANCE MYSIMKARI TABLES
-- Add missing fields for better integration
-- ============================================================================

-- Add status and additional fields to pegawai table
ALTER TABLE integrasi.mysimkari_pegawai
ADD COLUMN IF NOT EXISTS status_pegawai VARCHAR(50),
ADD COLUMN IF NOT EXISTS eselon VARCHAR(20),
ADD COLUMN IF NOT EXISTS jenis_kelamin VARCHAR(1),
ADD COLUMN IF NOT EXISTS tanggal_lahir DATE,
ADD COLUMN IF NOT EXISTS email VARCHAR(255),
ADD COLUMN IF NOT EXISTS no_hp VARCHAR(20);

-- Add indexes for common queries
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_status ON integrasi.mysimkari_pegawai(status_pegawai) WHERE status_pegawai IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_eselon ON integrasi.mysimkari_pegawai(eselon) WHERE eselon IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_jk ON integrasi.mysimkari_pegawai(jenis_kelamin) WHERE jenis_kelamin IS NOT NULL;

-- Add full-text search index for pegawai nama
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_nama_trgm ON integrasi.mysimkari_pegawai USING gin(nama gin_trgm_ops);

-- ============================================================================
-- ADD MISSING JSONB GIN INDEXES
-- For fast JSON queries on raw_data columns
-- ============================================================================

-- SIMAN raw_data indexes
CREATE INDEX IF NOT EXISTS idx_siman_tanah_raw_gin ON integrasi.siman_aset_tanah USING gin(raw_data);
CREATE INDEX IF NOT EXISTS idx_siman_gedung_raw_gin ON integrasi.siman_aset_gedung_bangunan USING gin(raw_data);
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_raw_gin ON integrasi.siman_aset_alat_besar USING gin(raw_data);
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_raw_gin ON integrasi.siman_aset_angkutan_bermotor USING gin(raw_data);

-- MySIMKARI raw_data indexes
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_raw_gin ON integrasi.mysimkari_satker USING gin(raw_data);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_raw_gin ON integrasi.mysimkari_pegawai USING gin(raw_data);

-- MonSAKTI raw_data indexes
CREATE INDEX IF NOT EXISTS idx_adm_ref_bank_raw_gin ON integrasi.adm_ref_bank USING gin(raw_data);
CREATE INDEX IF NOT EXISTS idx_adm_ref_jns_spp_raw_gin ON integrasi.adm_ref_jns_spp USING gin(raw_data);

-- ============================================================================
-- CREATE COMPOSITE INDEXES FOR COMMON QUERY PATTERNS
-- ============================================================================

-- SIMAN: Filter by satker + kondisi (for gap analysis)
CREATE INDEX IF NOT EXISTS idx_siman_tanah_satker_kondisi ON integrasi.siman_aset_tanah(satker_id, kondisi) WHERE satker_id IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_gedung_satker_kondisi ON integrasi.siman_aset_gedung_bangunan(satker_id, kondisi) WHERE satker_id IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_satker_kondisi ON integrasi.siman_aset_alat_besar(satker_id, kondisi) WHERE satker_id IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_satker_kondisi ON integrasi.siman_aset_angkutan_bermotor(satker_id, kondisi) WHERE satker_id IS NOT NULL AND kondisi IS NOT NULL;

-- SIMAN: Filter by kode_barang + kondisi (for specific asset queries)
CREATE INDEX IF NOT EXISTS idx_siman_tanah_kode_kondisi ON integrasi.siman_aset_tanah(kode_barang, kondisi) WHERE kode_barang IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_gedung_kode_kondisi ON integrasi.siman_aset_gedung_bangunan(kode_barang, kondisi) WHERE kode_barang IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_alat_besar_kode_kondisi ON integrasi.siman_aset_alat_besar(kode_barang, kondisi) WHERE kode_barang IS NOT NULL AND kondisi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_siman_angkutan_kode_kondisi ON integrasi.siman_aset_angkutan_bermotor(kode_barang, kondisi) WHERE kode_barang IS NOT NULL AND kondisi IS NOT NULL;

-- MySIMKARI: Filter by satker + status
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_satker_status ON integrasi.mysimkari_pegawai(satker_id, status_pegawai) WHERE satker_id IS NOT NULL AND status_pegawai IS NOT NULL;

-- API Call Log: Filter by module + success + date
CREATE INDEX IF NOT EXISTS idx_api_call_log_module_success_date ON integrasi.api_call_log(module, success, created_at DESC);

-- ============================================================================
-- CREATE ENHANCED VIEWS FOR MONITORING
-- ============================================================================

-- View: Sync Status Dashboard
CREATE OR REPLACE VIEW integrasi.v_sync_status_dashboard AS
SELECT
    ss.module,
    ss.endpoint,
    ss.sync_type,
    ss.last_sync_status,
    ss.last_sync_completed_at,
    ss.last_sync_records_count,
    ss.next_sync_scheduled_at,
    ss.total_syncs_completed,
    ss.total_syncs_failed,
    ss.total_records_synced,
    ss.is_enabled,
    CASE
        WHEN NOT ss.is_enabled THEN 'DISABLED'
        WHEN ss.last_sync_status = 'failed' THEN 'FAILED'
        WHEN ss.last_sync_status = 'running' THEN 'RUNNING'
        WHEN ss.next_sync_scheduled_at < CURRENT_TIMESTAMP THEN 'OVERDUE'
        WHEN ss.last_sync_completed_at < CURRENT_TIMESTAMP - INTERVAL '24 hours' THEN 'STALE'
        ELSE 'HEALTHY'
    END as health_status,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - ss.last_sync_completed_at)) / 3600 as hours_since_last_sync
FROM integrasi.sync_status ss
ORDER BY
    CASE
        WHEN NOT ss.is_enabled THEN 5
        WHEN ss.last_sync_status = 'failed' THEN 1
        WHEN ss.last_sync_status = 'running' THEN 2
        WHEN ss.next_sync_scheduled_at < CURRENT_TIMESTAMP THEN 3
        ELSE 4
    END,
    ss.module, ss.endpoint;

COMMENT ON VIEW integrasi.v_sync_status_dashboard IS 'Dashboard view showing sync health status for all integration modules';

-- View: SIMAN Asset Summary by Satker
CREATE OR REPLACE VIEW integrasi.v_siman_asset_summary_by_satker AS
SELECT
    satker_id,
    'tanah' as asset_type,
    COUNT(*) as total_count,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.siman_aset_tanah
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'gedung_bangunan' as asset_type,
    COUNT(*) as total_count,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.siman_aset_gedung_bangunan
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'alat_besar' as asset_type,
    COUNT(*) as total_count,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.siman_aset_alat_besar
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'angkutan_bermotor' as asset_type,
    COUNT(*) as total_count,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.siman_aset_angkutan_bermotor
WHERE satker_id IS NOT NULL
GROUP BY satker_id

ORDER BY satker_id, asset_type;

COMMENT ON VIEW integrasi.v_siman_asset_summary_by_satker IS 'Summary of SIMAN assets by satker and condition';

-- View: MySIMKARI Pegawai Summary by Satker
CREATE OR REPLACE VIEW integrasi.v_mysimkari_pegawai_summary AS
SELECT
    satker_id,
    COUNT(*) as total_pegawai,
    COUNT(*) FILTER (WHERE status_pegawai = 'AKTIF') as aktif_count,
    COUNT(*) FILTER (WHERE status_pegawai = 'PENSIUN') as pensiun_count,
    COUNT(*) FILTER (WHERE jenis_kelamin = 'L') as laki_laki_count,
    COUNT(*) FILTER (WHERE jenis_kelamin = 'P') as perempuan_count,
    COUNT(*) FILTER (WHERE eselon IS NOT NULL) as eselon_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.mysimkari_pegawai
WHERE satker_id IS NOT NULL
GROUP BY satker_id
ORDER BY satker_id;

COMMENT ON VIEW integrasi.v_mysimkari_pegawai_summary IS 'Summary of MySIMKARI employees by satker';

-- ============================================================================
-- TRIGGERS FOR AUTO-UPDATE
-- ============================================================================

-- Trigger for sync_status updated_at
CREATE TRIGGER update_sync_status_updated_at
    BEFORE UPDATE ON integrasi.sync_status
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- INSERT DEFAULT SYNC STATUS RECORDS
-- ============================================================================

-- SIMAN sync status
INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('SIMAN', 'aset_tanah', 'full', 'daily', TRUE),
    ('SIMAN', 'aset_gedung_bangunan', 'full', 'daily', TRUE),
    ('SIMAN', 'aset_alat_besar', 'full', 'daily', TRUE),
    ('SIMAN', 'aset_angkutan_bermotor', 'full', 'daily', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;

-- MySIMKARI sync status
INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('MySIMKARI', 'satker', 'full', 'daily', TRUE),
    ('MySIMKARI', 'pegawai', 'full', 'daily', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;

-- MonSAKTI sync status (examples)
INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('MonSAKTI_ADM', 'ref_admin', 'full', 'weekly', TRUE),
    ('MonSAKTI_ADM', 'ref_bank', 'full', 'weekly', TRUE),
    ('MonSAKTI_ADM', 'ref_jns_spp', 'full', 'weekly', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $
BEGIN
    RAISE NOTICE '✅ Integration schema enhancement completed successfully';
    RAISE NOTICE '📊 Added: 1 new table (sync_status)';
    RAISE NOTICE '🔧 Enhanced: SIMAN and MySIMKARI tables with additional fields';
    RAISE NOTICE '📈 Created: 30+ new indexes (FK, composite, JSONB GIN, full-text)';
    RAISE NOTICE '👁️ Created: 3 new views for monitoring';
    RAISE NOTICE '🔍 Schema: integrasi';
END $;
