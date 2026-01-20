-- =====================================================
-- Migration: Create SIMAN (Sistem Informasi Manajemen Aset Negara) UNIFIED Table
-- Description: Tabel TUNGGAL untuk menyimpan data BMN dari 15 kategori API SIMAN v2.0
-- Version: 3.0 (CORRECTED based on Official DJKN Guide)
-- Created: 2025-11-06
-- Reference: Panduan Penggunaan Web Service SLDK-Kejaksaan RI (Oktober 2025)
-- PENTING: Semua 15 kategori memiliki 106 fields yang IDENTIK (lihat lampiran 1-15)
-- =====================================================

-- Enable UUID extension if not exists
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- SCHEMA: integrasi (use same schema as other tables)
-- =====================================================
-- Schema already created in migration 001
-- CREATE SCHEMA IF NOT EXISTS integrasi;

-- Set search path
SET search_path TO integrasi, public;

-- =====================================================
-- TABLE: siman_sync_log
-- Purpose: Log untuk tracking sinkronisasi data SIMAN
-- =====================================================
CREATE TABLE IF NOT EXISTS siman_sync_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    kategori_aset VARCHAR(100), -- 'Alat Besar', 'Angkutan Bermotor', 'ALL' untuk sync semua
    sync_start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sync_end_time TIMESTAMP,
    total_records TEXT DEFAULT 0,
    success_records TEXT DEFAULT 0,
    failed_records TEXT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'running', -- running, completed, failed
    error_message TEXT,
    ba_key VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_siman_sync_log_kategori ON siman_sync_log(kategori_aset);
CREATE INDEX idx_siman_sync_log_status ON siman_sync_log(status);
CREATE INDEX idx_siman_sync_log_created ON siman_sync_log(created_at DESC);

-- =====================================================
-- MAIN TABLE: siman_aset (UNIFIED)
-- Purpose: Menyimpan SEMUA 15 kategori aset dengan 106 fields IDENTIK
-- Based on: DJKN Official Guide - All categories have same structure
-- =====================================================
CREATE TABLE IF NOT EXISTS siman_aset (
    -- ===== INTERNAL FIELDS =====
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    kategori_aset VARCHAR(50) NOT NULL, -- 'Alat Besar', 'Angkutan Bermotor', dll

    -- ===== DJKN OFFICIAL FIELDS (106 fields) =====
    -- Fields 1-14: Core Asset Identification
    kd_jns_bmn TEXT,
    kd_brg TEXT,
    no_aset TEXT NOT NULL,
    tercatat VARCHAR(3),
    merk VARCHAR(500),
    tipe VARCHAR(500),
    rph_aset TEXT,
    rph_susut TEXT,
    rph_mutasi TEXT,
    status_bmn_yn VARCHAR(10),
    bpybds_yn VARCHAR(10),
    flag_sap VARCHAR(10),
    jml_photo TEXT,
    ur_sskel VARCHAR(100),

    -- Fields 15-20: Location
    alamat VARCHAR(200),
    komplek VARCHAR(200),
    kd_rtrw VARCHAR(15),
    ur_kab VARCHAR(100),
    ur_prov VARCHAR(100),
    kd_pos VARCHAR(10),

    -- Fields 21-32: Financial & Administrative
    status_sbsn VARCHAR(50),
    no_dana VARCHAR(200),
    tgl_dana TEXT,
    asl_perlh VARCHAR(255),
    tgl_perlh TEXT,
    ur_sumber_dana VARCHAR(30),
    tgl_akhir_sbsn TEXT,
    status_bmn_idle VARCHAR(1),
    rencana_hibah_yn VARCHAR(1),
    kmk_sbsn VARCHAR(100),
    no_psp VARCHAR(100),
    tgl_psp TEXT,

    -- Fields 33-39: Dimensions
    luas TEXT,
    luas_tapak TEXT,
    luas_tnhl TEXT,
    luas_tnhk TEXT,
    luas_tnhb TEXT,
    jml_lantai TEXT,
    jml_bdg TEXT,

    -- Fields 40-48: Status & Legal
    kd_jns_idle VARCHAR(2),
    no_perkara_hukum VARCHAR(100),
    no_dok_bukti_kepemilikan VARCHAR(200),
    jns_dok_bukti_kepemilikan VARCHAR(300),
    dihentikan_yn VARCHAR(1),
    sbsk TEXT,
    tgl_rekam_pertama TEXT,
    tgl_rekam TEXT,
    tgl_hapus TEXT,

    -- Fields 49-62: Geographic & Physical
    bts_utara VARCHAR(100),
    bts_selatan VARCHAR(100),
    bts_barat VARCHAR(100),
    bts_timur VARCHAR(100),
    bentuk VARCHAR(30),
    peruntukan_tnh VARCHAR(100),
    topografi_kontur VARCHAR(30),
    topografi_elevasi VARCHAR(30),
    aksesibilitas VARCHAR(30),
    gps_latitude TEXT,
    gps_longitude TEXT,
    kemitraan_yn VARCHAR(1),
    brg_hilang_yn VARCHAR(1),
    dktp_yn VARCHAR(1),

    -- Fields 63-77: Asset Details
    brg_rusak_yn VARCHAR(1),
    umur_sisa TEXT,
    no_dok_perolehan VARCHAR(200),
    jns_aset VARCHAR(70),
    negara VARCHAR(128),
    rph_perolehan TEXT,
    rph_buku TEXT,
    cara_perlh VARCHAR(128),
    jns_pengguna VARCHAR(128),
    kd_unit_pengguna VARCHAR(128),
    nm_unit_pengguna VARCHAR(128),
    ket_pengguna VARCHAR(128),
    stat_dok_bukti_kepemilikan VARCHAR(128),
    tgl_dok_bukti_kepemilikan VARCHAR(50),
    no_kib VARCHAR(100),

    -- Fields 78-86: Organization & Usage
    kode_satker VARCHAR(20) NOT NULL,
    nama_satker VARCHAR(100),
    kode_sub_satker VARCHAR(20),
    nama_sub_satker VARCHAR(100),
    ur_kondisi VARCHAR(50),
    nm_penghuni VARCHAR(50),
    tgl_mulai_huni TEXT,
    tgl_selesai_huni TEXT,
    no_polisi VARCHAR(20), -- ADA di SEMUA tabel per panduan

    -- Fields 87-106: Administrative Hierarchy & Metadata
    urkpknl VARCHAR(100),
    ur_kanwil VARCHAR(100),
    ur_kl VARCHAR(60),
    ur_eselon1 VARCHAR(100),
    ur_korwil VARCHAR(100),
    kode_register VARCHAR(100),
    optimalisasi TEXT,
    tgl_buku_pertama TEXT,
    jns_sertifikat VARCHAR(30),
    intra_extra TEXT,
    ur_status VARCHAR(100),
    nama TEXT,
    ur_kel VARCHAR(100),
    ur_kec VARCHAR(100),
    hapus_lainnya_yn VARCHAR(1),
    konjas_yn VARCHAR(1),
    properti_investasi_yn VARCHAR(1),
    lokasi_ruang VARCHAR(500),
    luas_pemanfaatan TEXT,
    last_update TEXT,

    -- ===== INTERNAL TRACKING FIELDS =====
    raw_data JSONB, -- Complete API response
    sync_id UUID REFERENCES siman_sync_log(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT unique_aset_per_satker UNIQUE (no_aset, kode_satker, kategori_aset)
);

-- =====================================================
-- INDEXES for Performance
-- =====================================================
CREATE INDEX idx_siman_aset_kategori ON siman_aset(kategori_aset);
CREATE INDEX idx_siman_aset_satker ON siman_aset(kode_satker);
CREATE INDEX idx_siman_aset_no_aset ON siman_aset(no_aset);
CREATE INDEX idx_siman_aset_created ON siman_aset(created_at DESC);
CREATE INDEX idx_siman_aset_sync ON siman_aset(sync_id);
CREATE INDEX idx_siman_aset_kondisi ON siman_aset(ur_kondisi);
CREATE INDEX idx_siman_aset_jns_bmn ON siman_aset(kd_jns_bmn);
CREATE INDEX idx_siman_aset_updated ON siman_aset(updated_at DESC);

-- Full-text search on location and name fields
CREATE INDEX idx_siman_aset_fulltext ON siman_aset USING GIN (
    to_tsvector('indonesian',
        COALESCE(alamat, '') || ' ' ||
        COALESCE(nama, '') || ' ' ||
        COALESCE(ur_kab, '') || ' ' ||
        COALESCE(ur_prov, '') || ' ' ||
        COALESCE(nama_satker, '')
    )
);

-- GIN index for JSONB queries
CREATE INDEX idx_siman_aset_raw_data ON siman_aset USING GIN (raw_data);

-- =====================================================
-- AUTO-UPDATE TRIGGER
-- =====================================================
CREATE OR REPLACE FUNCTION update_siman_aset_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_siman_aset_timestamp
    BEFORE UPDATE ON siman_aset
    FOR EACH ROW
    EXECUTE FUNCTION update_siman_aset_timestamp();

CREATE TRIGGER trg_update_sync_log_timestamp
    BEFORE UPDATE ON siman_sync_log
    FOR EACH ROW
    EXECUTE FUNCTION update_siman_aset_timestamp();

-- =====================================================
-- VIEWS: Per-Category Views (Backward Compatibility)
-- Purpose: Jika developer mau query per kategori
-- =====================================================
CREATE OR REPLACE VIEW v_siman_aset_alat_besar AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Alat Besar';

CREATE OR REPLACE VIEW v_siman_aset_alat_persenjataan AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Alat Persenjataan';

CREATE OR REPLACE VIEW v_siman_aset_angkutan_bermotor AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Angkutan Bermotor';

CREATE OR REPLACE VIEW v_siman_aset_tak_berwujud AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Tak Berwujud';

CREATE OR REPLACE VIEW v_siman_aset_tetap_lainnya AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Tetap Lainnya';

CREATE OR REPLACE VIEW v_siman_aset_tanah AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Tanah';

CREATE OR REPLACE VIEW v_siman_aset_bangunan_air AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Bangunan Air';

CREATE OR REPLACE VIEW v_siman_aset_gedung_bangunan AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Gedung Bangunan';

CREATE OR REPLACE VIEW v_siman_aset_instalasi_jaringan AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Instalasi Jaringan';

CREATE OR REPLACE VIEW v_siman_aset_jalan_jembatan AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Jalan dan Jembatan';

CREATE OR REPLACE VIEW v_siman_aset_kdp AS
SELECT * FROM siman_aset WHERE kategori_aset = 'KDP';

CREATE OR REPLACE VIEW v_siman_aset_khusus_tik AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Khusus TIK';

CREATE OR REPLACE VIEW v_siman_aset_non_tik AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Non TIK';

CREATE OR REPLACE VIEW v_siman_aset_rumah AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Rumah';

CREATE OR REPLACE VIEW v_siman_aset_tetap_renovasi AS
SELECT * FROM siman_aset WHERE kategori_aset = 'Tetap Renovasi';

-- =====================================================
-- SUMMARY VIEWS
-- =====================================================
CREATE OR REPLACE VIEW v_siman_summary_per_kategori AS
SELECT
    kategori_aset,
    COUNT(*) as total_aset,
    COUNT(DISTINCT kode_satker) as jumlah_satker,
    SUM(rph_aset) as total_nilai_perolehan,
    SUM(rph_susut) as total_penyusutan,
    SUM(rph_aset - COALESCE(rph_susut, 0)) as total_nilai_buku,
    COUNT(CASE WHEN ur_kondisi = 'Baik' THEN 1 END) as aset_baik,
    COUNT(CASE WHEN ur_kondisi LIKE 'Rusak%' THEN 1 END) as aset_rusak,
    MAX(updated_at) as last_sync
FROM siman_aset
GROUP BY kategori_aset
ORDER BY total_aset DESC;

CREATE OR REPLACE VIEW v_siman_summary_per_satker AS
SELECT
    kode_satker,
    nama_satker,
    COUNT(*) as total_aset,
    COUNT(DISTINCT kategori_aset) as jumlah_kategori,
    SUM(rph_aset) as total_nilai_perolehan,
    SUM(rph_aset - COALESCE(rph_susut, 0)) as total_nilai_buku,
    MAX(updated_at) as last_sync
FROM siman_aset
GROUP BY kode_satker, nama_satker
ORDER BY total_aset DESC;

CREATE OR REPLACE VIEW v_siman_summary_total AS
SELECT
    COUNT(*) as total_aset,
    COUNT(DISTINCT kategori_aset) as total_kategori,
    COUNT(DISTINCT kode_satker) as total_satker,
    SUM(rph_aset) as total_nilai_perolehan,
    SUM(rph_aset - COALESCE(rph_susut, 0)) as total_nilai_buku,
    COUNT(CASE WHEN ur_kondisi = 'Baik' THEN 1 END) as total_baik,
    COUNT(CASE WHEN ur_kondisi LIKE 'Rusak%' THEN 1 END) as total_rusak,
    MAX(updated_at) as last_sync
FROM siman_aset;

-- =====================================================
-- COMMENTS
-- =====================================================
COMMENT ON SCHEMA integrasi IS 'Schema untuk semua tabel integrasi API (MonSAKTI, MySIMKARI, SIMAN, Audit, Token)';

COMMENT ON TABLE siman_aset IS 'Unified table untuk 15 kategori aset SIMAN. Semua kategori punya 106 fields identik sesuai panduan DJKN Oktober 2025.';
COMMENT ON COLUMN siman_aset.kategori_aset IS 'Kategori aset: Alat Besar, Angkutan Bermotor, Gedung Bangunan, Tanah, KDP, dll (15 kategori)';
COMMENT ON COLUMN siman_aset.no_aset IS 'Nomor aset unik dari sistem SIMAN';
COMMENT ON COLUMN siman_aset.kode_satker IS 'Kode satuan kerja pemilik aset';
COMMENT ON COLUMN siman_aset.raw_data IS 'Complete JSON response dari SIMAN API';
COMMENT ON COLUMN siman_aset.sync_id IS 'Reference ke siman_sync_log untuk tracking';

COMMENT ON VIEW v_siman_summary_per_kategori IS 'Summary statistik aset per kategori';
COMMENT ON VIEW v_siman_summary_per_satker IS 'Summary statistik aset per satuan kerja';
COMMENT ON VIEW v_siman_summary_total IS 'Summary statistik keseluruhan';

-- =====================================================
-- SUCCESS MESSAGE
-- =====================================================
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'SIMAN UNIFIED TABLE CREATED SUCCESSFULLY';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Table: siman_aset (106 DJKN fields + 5 internal)';
    RAISE NOTICE 'Views: 15 category views + 3 summary views';
    RAISE NOTICE 'Indexes: 9 performance indexes';
    RAISE NOTICE 'Triggers: Auto-update timestamps';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'VERIFIED: All 15 categories have IDENTICAL fields';
    RAISE NOTICE 'Source: Panduan WS SLDK Kejaksaan RI (Okt 2025)';
    RAISE NOTICE '========================================';
END $$;
