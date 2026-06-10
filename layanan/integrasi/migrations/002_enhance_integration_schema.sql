-- ============================================================================
-- Migration: Enhance Integration Schema
-- Description: sync_status tracking, MySIMKARI enhancements, monitoring views.
-- Author: SIMPEL Team
-- Created: 2026-02-09 (rebuilt 2026-06-09: unified siman_aset model)
-- Requirements: REQ-I001, REQ-I002, REQ-I008
--
-- NOTE (F5-B rebuild): the legacy SPLIT siman_aset_{tanah,gedung_bangunan,
-- alat_besar,angkutan_bermotor} tables were removed. SIMAN is a SINGLE unified
-- `siman_aset` table (jenis_aset discriminator) — see 001_init_schema.sql and
-- src/grpc/service.rs / src/siman/endpoints.rs (writer `.save("siman","aset")`).
-- All split-table ALTER/index/view statements that previously lived here were
-- dead (the tables never existed under the unified model) and have been dropped.
-- ============================================================================

-- Ensure schema exists
CREATE SCHEMA IF NOT EXISTS integrasi;
SET search_path TO integrasi, public;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================================
-- SYNC STATUS TABLE
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

    CONSTRAINT unique_module_endpoint UNIQUE (module, endpoint)
);

CREATE INDEX IF NOT EXISTS idx_sync_status_module ON integrasi.sync_status(module);
CREATE INDEX IF NOT EXISTS idx_sync_status_last_sync ON integrasi.sync_status(last_sync_completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_sync_status_next_sync ON integrasi.sync_status(next_sync_scheduled_at) WHERE is_enabled = TRUE;
CREATE INDEX IF NOT EXISTS idx_sync_status_failed ON integrasi.sync_status(last_sync_status) WHERE last_sync_status = 'failed';

COMMENT ON TABLE integrasi.sync_status IS 'Tracks synchronization status and schedule for each integration module';

-- ============================================================================
-- ENHANCE MYSIMKARI PEGAWAI
-- ============================================================================

ALTER TABLE integrasi.mysimkari_pegawai
ADD COLUMN IF NOT EXISTS status_pegawai VARCHAR(50),
ADD COLUMN IF NOT EXISTS eselon VARCHAR(20),
ADD COLUMN IF NOT EXISTS jenis_kelamin VARCHAR(1),
ADD COLUMN IF NOT EXISTS tanggal_lahir DATE,
ADD COLUMN IF NOT EXISTS email VARCHAR(255),
ADD COLUMN IF NOT EXISTS no_hp VARCHAR(20);

CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_status ON integrasi.mysimkari_pegawai(status_pegawai) WHERE status_pegawai IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_eselon ON integrasi.mysimkari_pegawai(eselon) WHERE eselon IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_jk ON integrasi.mysimkari_pegawai(jenis_kelamin) WHERE jenis_kelamin IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_nama_trgm ON integrasi.mysimkari_pegawai USING gin(nama gin_trgm_ops);

-- NOTE: raw_data JSONB GIN indexes only apply to siman_aset (the only table
-- that stores raw_data; created in 001). The structured tables (mysimkari_*,
-- adm_ref_*) store typed columns, not raw_data, so no GIN index here.

-- ============================================================================
-- COMPOSITE INDEXES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_satker_status ON integrasi.mysimkari_pegawai(satker_id, status_pegawai) WHERE satker_id IS NOT NULL AND status_pegawai IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_api_log_module_success_date ON integrasi.api_log(module, success, created_at DESC);

-- ============================================================================
-- MONITORING VIEWS
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

-- View: SIMAN Asset Summary by Satker (unified siman_aset, grouped by jenis_aset)
CREATE OR REPLACE VIEW integrasi.v_siman_asset_summary_by_satker AS
SELECT
    kdsatker_keu AS satker_id,
    jenis_aset AS asset_type,
    COUNT(*) as total_count,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    MAX(synced_at) as last_synced_at
FROM integrasi.siman_aset
WHERE kdsatker_keu IS NOT NULL
GROUP BY kdsatker_keu, jenis_aset
ORDER BY kdsatker_keu, jenis_aset;

COMMENT ON VIEW integrasi.v_siman_asset_summary_by_satker IS 'Summary of unified SIMAN assets by satker (kdsatker_keu) and jenis_aset/condition';

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
-- TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS update_sync_status_updated_at ON integrasi.sync_status;
CREATE TRIGGER update_sync_status_updated_at
    BEFORE UPDATE ON integrasi.sync_status
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- DEFAULT SYNC STATUS RECORDS
-- ============================================================================

INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('SIMAN', 'aset', 'full', 'daily', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;

INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('MySIMKARI', 'satker', 'full', 'daily', TRUE),
    ('MySIMKARI', 'pegawai', 'full', 'daily', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;

INSERT INTO integrasi.sync_status (module, endpoint, sync_type, sync_frequency, is_enabled) VALUES
    ('MonSAKTI_ADM', 'ref_admin', 'full', 'weekly', TRUE),
    ('MonSAKTI_ADM', 'ref_bank', 'full', 'weekly', TRUE),
    ('MonSAKTI_ADM', 'ref_jns_spp', 'full', 'weekly', TRUE)
ON CONFLICT (module, endpoint) DO NOTHING;
