-- ============================================================================
-- SIMPEL - Layanan Integrasi Database Schema
-- ============================================================================
-- Purpose: Complete database schema for integration service
-- Sources: MonSAKTI (8 modules: ADM/ANG/AST/BEN/GLP/KOM/PEM/PER),
--          MySIMKARI, SIMAN v2.0 (15 jenis aset)
-- Tables : 34 total (1 audit + 1 tokens + 1 SIMAN + 2 MySIMKARI + 29 MonSAKTI)
-- ============================================================================

-- Create schema for isolation
CREATE SCHEMA IF NOT EXISTS integrasi;
SET search_path TO integrasi, public;

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================================
-- AUDIT & LOGGING (2 tables — consolidated)
-- ============================================================================

-- Unified API log — tracks API calls, batch ops, sync status, token events
CREATE TABLE IF NOT EXISTS api_log (
    id BIGSERIAL PRIMARY KEY,

    -- Classification
    log_type VARCHAR(20) NOT NULL DEFAULT 'api_call',
        -- api_call | batch | sync | token_reset
    module VARCHAR(20) NOT NULL,
    endpoint VARCHAR(100),

    -- Request context
    kode_kl VARCHAR(10),
    kdsatker VARCHAR(20),
    full_url TEXT,
    request_method VARCHAR(10) DEFAULT 'GET',

    -- Response
    response_status INTEGER,
    response_time_ms INTEGER,
    record_count INTEGER,

    -- Result
    success BOOLEAN DEFAULT false,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,

    -- Timestamps
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- MonSAKTI rotating bearer tokens per module
CREATE TABLE IF NOT EXISTS monsakti_tokens (
    id BIGSERIAL PRIMARY KEY,
    module VARCHAR(20) NOT NULL UNIQUE,
    token_value TEXT NOT NULL,
    token_hash TEXT,
    token_type VARCHAR(20) DEFAULT 'bearer',
    is_active BOOLEAN DEFAULT true,
    is_expired BOOLEAN DEFAULT false,
    issued_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    refreshed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    refresh_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- SIMAN v2.0 — SINGLE UNIFIED TABLE (15 jenis aset)
-- ============================================================================
-- Jenis aset: Tanah, Alat Angkutan Bermotor, Peralatan Mesin Non TIK,
--   Peralatan Mesin Khusus TIK, Alat Besar, Alat Persenjataan,
--   Gedung dan Bangunan, Rumah Negara, Jalan dan Jembatan, Bangunan Air,
--   Instalasi dan Jaringan, Aset Tetap Lainnya, Aset Tak Berwujud,
--   Aset Tetap Renovasi, Konstruksi Dalam Pengerjaan

CREATE TABLE IF NOT EXISTS siman_aset (
    id BIGSERIAL PRIMARY KEY,
    jenis_aset TEXT NOT NULL,
    -- Remaining columns are added dynamically from API response
    -- via ensure_table_exists() self-healing schema mechanism
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MYSIMKARI (2 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS mysimkari_satker (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    kode_satker TEXT NOT NULL UNIQUE,
    nama_satker TEXT,
    tipe_satker TEXT,
    alamat_satker TEXT,
    telp_satker TEXT,
    website_satker TEXT,
    city TEXT,
    long TEXT,
    lat TEXT,
    provinsi TEXT,
    wilayah TEXT,
    kategori_satker TEXT,
    pulau TEXT,
    parent_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS mysimkari_pegawai (
    id BIGSERIAL PRIMARY KEY,
    nip TEXT NOT NULL UNIQUE,
    nama TEXT,
    satker_id TEXT,
    nama_satker TEXT,
    jabatan TEXT,
    jenis_jabatan_terakhir TEXT,
    eselon TEXT,
    golpang TEXT,
    gol_kd TEXT,
    jk TEXT,
    agama TEXT,
    email_dinas TEXT,
    no_hp TEXT,
    nrp TEXT,
    foto TEXT,
    bidang TEXT,
    jabat_tmt TEXT,
    status_pegawai TEXT NOT NULL DEFAULT 'aktif',
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — ADM Module (6 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS adm_ref_admin (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    kdsatker TEXT,
    nmsatker TEXT,
    kdunit TEXT,
    nmunit TEXT,
    kode_kl TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS adm_ref_bank (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    kdbank TEXT,
    nmbank TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS adm_ref_jns_spp (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    kdjnsspp TEXT,
    nmjnsspp TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS adm_ref_uraian (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    jenis TEXT,
    kode TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS adm_ref_aset (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    jenis TEXT,
    kode TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS adm_pejabat (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — ANG Module (3 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ang_ref_sts (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ang_data_ang (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ang_pendapatan (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — BEN Module (10 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ben_kas_tunai (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_kas_bank (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_spby (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_kuitansi (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_drpp (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_pungut_pajak (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_setor_pajak (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_pnbp (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_tup (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ben_pengembalian (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — PEM Module (2 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS pem_realisasi (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS pem_spp_header (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — KOM Module (3 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS kom_kontrak_header (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS kom_capaian_ro (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS kom_supplier_header (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — AST Module (1 table)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ast_aset_trx (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — PER Module (1 table)
-- ============================================================================

CREATE TABLE IF NOT EXISTS per_persedia_trx (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MONSAKTI — GLP Module (3 tables)
-- ============================================================================

CREATE TABLE IF NOT EXISTS glp_buku_besar (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS glp_neraca_sawal (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS glp_fa_detail (
    id BIGSERIAL PRIMARY KEY,
    api_id TEXT,
    synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- API Log
CREATE INDEX IF NOT EXISTS idx_api_log_module ON api_log(module, endpoint);
CREATE INDEX IF NOT EXISTS idx_api_log_created ON api_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_log_type ON api_log(log_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_log_success ON api_log(success) WHERE success = false;

-- MonSAKTI Tokens
CREATE INDEX IF NOT EXISTS idx_monsakti_tokens_active ON monsakti_tokens(module) WHERE is_active = true;

-- SIMAN
CREATE INDEX IF NOT EXISTS idx_siman_aset_jenis ON siman_aset(jenis_aset);
CREATE INDEX IF NOT EXISTS idx_siman_aset_synced ON siman_aset(synced_at DESC);

-- MySIMKARI
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_satker ON mysimkari_pegawai(satker_id);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_status ON mysimkari_pegawai(status_pegawai);

-- MonSAKTI ADM
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_kdsatker ON adm_ref_admin(kdsatker);
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_synced ON adm_ref_admin(synced_at DESC);

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Recent failed API calls
CREATE OR REPLACE VIEW v_recent_failed_calls AS
SELECT log_type, module, endpoint, error_message, retry_count,
       response_status, full_url, created_at
FROM api_log
WHERE success = false
ORDER BY created_at DESC
LIMIT 100;

-- Token health
CREATE OR REPLACE VIEW v_token_health AS
SELECT
    t.module, t.is_active, t.is_expired, t.expires_at, t.refreshed_at, t.refresh_count,
    CASE
        WHEN t.is_expired THEN 'EXPIRED'
        WHEN t.expires_at IS NOT NULL AND t.expires_at < CURRENT_TIMESTAMP THEN 'EXPIRED'
        WHEN t.expires_at IS NOT NULL AND t.expires_at < CURRENT_TIMESTAMP + INTERVAL '1 day' THEN 'EXPIRING_SOON'
        WHEN NOT t.is_active THEN 'INACTIVE'
        ELSE 'HEALTHY'
    END AS health_status
FROM monsakti_tokens t
ORDER BY health_status;

-- API stats by module (last 7 days)
CREATE OR REPLACE VIEW v_api_stats AS
SELECT
    module, endpoint,
    COUNT(*) AS total_calls,
    COUNT(*) FILTER (WHERE success) AS ok,
    COUNT(*) FILTER (WHERE NOT success) AS fail,
    AVG(response_time_ms) FILTER (WHERE response_time_ms IS NOT NULL)::INT AS avg_ms,
    SUM(record_count) FILTER (WHERE record_count IS NOT NULL) AS total_records,
    MAX(created_at) AS last_call
FROM api_log
WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
GROUP BY module, endpoint
ORDER BY total_calls DESC;

-- SIMAN ringkasan per jenis aset
CREATE OR REPLACE VIEW v_siman_ringkasan AS
SELECT
    jenis_aset,
    COUNT(*) AS jumlah,
    MAX(synced_at) AS terakhir_sync
FROM siman_aset
GROUP BY jenis_aset
ORDER BY jenis_aset;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables with updated_at column
DO $$
DECLARE
    tbl TEXT;
BEGIN
    FOR tbl IN
        SELECT table_name FROM information_schema.columns
        WHERE table_schema = 'integrasi' AND column_name = 'updated_at'
    LOOP
        EXECUTE format(
            'DROP TRIGGER IF EXISTS trg_updated_at_%I ON %I; '
            'CREATE TRIGGER trg_updated_at_%I BEFORE UPDATE ON %I '
            'FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();',
            tbl, tbl, tbl, tbl
        );
    END LOOP;
END $$;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON SCHEMA integrasi IS 'Integration service: MonSAKTI + MySIMKARI + SIMAN data';
COMMENT ON TABLE api_log IS 'Unified audit log: API calls, batch ops, sync, token events';
COMMENT ON TABLE api_tokens IS 'Active API tokens per module';
COMMENT ON TABLE siman_aset IS 'SIMAN v2.0: All 15 jenis aset in a single table';
COMMENT ON TABLE mysimkari_satker IS 'MySIMKARI: Satuan kerja master data';
COMMENT ON TABLE mysimkari_pegawai IS 'MySIMKARI: Pegawai master data';
COMMENT ON TABLE adm_ref_admin IS 'MonSAKTI ADM: Referensi satuan kerja';
COMMENT ON TABLE adm_ref_bank IS 'MonSAKTI ADM: Referensi bank';
COMMENT ON TABLE adm_ref_jns_spp IS 'MonSAKTI ADM: Referensi jenis SPP';
COMMENT ON TABLE adm_ref_uraian IS 'MonSAKTI ADM: Referensi uraian program/kegiatan/output/akun';
COMMENT ON TABLE adm_ref_aset IS 'MonSAKTI ADM: Referensi pengkodean aset';
COMMENT ON TABLE adm_pejabat IS 'MonSAKTI ADM: Data pejabat';
COMMENT ON TABLE ang_ref_sts IS 'MonSAKTI ANG: Referensi STS history';
COMMENT ON TABLE ang_data_ang IS 'MonSAKTI ANG: Data anggaran';
COMMENT ON TABLE ang_pendapatan IS 'MonSAKTI ANG: Data pendapatan';
COMMENT ON TABLE ben_kas_tunai IS 'MonSAKTI BEN: Kas tunai';
COMMENT ON TABLE ben_kas_bank IS 'MonSAKTI BEN: Kas bank';
COMMENT ON TABLE ben_spby IS 'MonSAKTI BEN: SPBY';
COMMENT ON TABLE ben_kuitansi IS 'MonSAKTI BEN: Kuitansi';
COMMENT ON TABLE ben_drpp IS 'MonSAKTI BEN: DRPP';
COMMENT ON TABLE ben_pungut_pajak IS 'MonSAKTI BEN: Pungut pajak';
COMMENT ON TABLE ben_setor_pajak IS 'MonSAKTI BEN: Setor pajak';
COMMENT ON TABLE ben_pnbp IS 'MonSAKTI BEN: PNBP';
COMMENT ON TABLE ben_tup IS 'MonSAKTI BEN: TUP';
COMMENT ON TABLE ben_pengembalian IS 'MonSAKTI BEN: Pengembalian belanja';
COMMENT ON TABLE pem_realisasi IS 'MonSAKTI PEM: Realisasi belanja';
COMMENT ON TABLE pem_spp_header IS 'MonSAKTI PEM: SPP header';
COMMENT ON TABLE kom_kontrak_header IS 'MonSAKTI KOM: Kontrak header';
COMMENT ON TABLE kom_capaian_ro IS 'MonSAKTI KOM: Capaian RO';
COMMENT ON TABLE kom_supplier_header IS 'MonSAKTI KOM: Supplier header';
COMMENT ON TABLE ast_aset_trx IS 'MonSAKTI AST: Transaksi aset tetap';
COMMENT ON TABLE per_persedia_trx IS 'MonSAKTI PER: Transaksi persediaan';
COMMENT ON TABLE glp_buku_besar IS 'MonSAKTI GLP: Buku besar';
COMMENT ON TABLE glp_neraca_sawal IS 'MonSAKTI GLP: Neraca saldo awal';
COMMENT ON TABLE glp_fa_detail IS 'MonSAKTI GLP: Fixed assets detail';

-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE '✅ Integration schema initialized';
    RAISE NOTICE '📊 34 tables: 2 audit + 1 SIMAN + 2 MySIMKARI + 29 MonSAKTI';
END $$;
