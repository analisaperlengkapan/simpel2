-- ============================================================================
-- Migration: Create Kebutuhan BMN Tables
-- Description: Tables for BMN (Barang Milik Negara) needs analysis system
-- Author: SIMPEL Team
-- Created: 2026-02-02
-- ============================================================================

-- Create schema if not exists
CREATE SCHEMA IF NOT EXISTS perlengkapan;

-- ============================================================================
-- Master Activity Status Table (shared with pakaian_dinas)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.ms_aktivitas_bmn (
    id SERIAL PRIMARY KEY,
    kode INTEGER UNIQUE NOT NULL,
    nama VARCHAR(100) NOT NULL,
    deskripsi TEXT,
    urutan INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default workflow statuses for BMN
INSERT INTO perlengkapan.ms_aktivitas_bmn (kode, nama, deskripsi, urutan) VALUES
    (2000, 'DRAFT', 'Pengajuan baru dalam tahap penyusunan', 1),
    (2001, 'INPUT_BARANG', 'Pelaksana Satker menginput daftar kebutuhan barang', 2),
    (2002, 'SUBMIT_SATKER', 'Satker mengajukan ke Validator', 3),
    (2003, 'REVISI_SATKER', 'Dikembalikan ke Satker untuk revisi', 4),
    (2004, 'ANALISIS_KELAYAKAN', 'Validator Pusat melakukan analisis kelayakan', 5),
    (2005, 'PENYUSUNAN_PRIORITAS', 'Validator Pusat menyusun prioritas', 6),
    (2006, 'APPROVED', 'Pengajuan disetujui', 7),
    (2007, 'REJECTED', 'Pengajuan ditolak', 8),
    (2008, 'COMPLETED', 'Proses selesai', 9),
    (2009, 'CANCELLED', 'Pengajuan dibatalkan', 10)
ON CONFLICT (kode) DO NOTHING;

-- ============================================================================
-- Pengajuan Kebutuhan BMN (Main Request Entity)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_kebutuhan_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Basic Information
    nama VARCHAR(255) NOT NULL,
    deskripsi TEXT,
    tahun INTEGER NOT NULL,
    tgl_mulai DATE NOT NULL,
    tgl_selesai DATE NOT NULL,
    
    -- Satker Selection
    pilihan_satker VARCHAR(20) DEFAULT 'semua' CHECK (pilihan_satker IN ('semua', 'sebagian')),
    
    -- Asset Types (JSON array of selected asset type IDs)
    id_jenis_asset JSONB DEFAULT '[]'::jsonb,
    
    -- Approval Status
    is_appv_daskrimti BOOLEAN DEFAULT FALSE,
    
    -- Workflow Status (FK to ms_aktivitas_bmn.kode)
    status_kode INTEGER NOT NULL DEFAULT 2000 REFERENCES perlengkapan.ms_aktivitas_bmn(kode),
    
    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Optimistic Locking
    version INTEGER DEFAULT 1,
    
    CONSTRAINT valid_date_range CHECK (tgl_selesai >= tgl_mulai),
    CONSTRAINT valid_tahun CHECK (tahun >= 2020 AND tahun <= 2100)
);

-- ============================================================================
-- Pengajuan Kebutuhan BMN Asset Types (Asset Types per Request)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_kebutuhan_bmn_asset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE,
    
    -- Asset Classification
    kode_barang VARCHAR(50),
    nm_barang VARCHAR(255),
    ms_jenis_asset_id INTEGER,
    keterangan TEXT,
    
    -- Audit Trail
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- Pengajuan Kebutuhan BMN Satker (Per Work Unit Tracking)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_kebutuhan_bmn_satker (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE,
    
    -- Satker Reference (from MySIMKARI)
    ms_satker_id VARCHAR(20) NOT NULL,
    ms_satker_pusat_id VARCHAR(20),
    nm_satker VARCHAR(255),
    
    -- Workflow Status per Satker
    status_kode INTEGER NOT NULL DEFAULT 2001 REFERENCES perlengkapan.ms_aktivitas_bmn(kode),
    
    -- Priority Ranking (set during penyusunan prioritas)
    prioritas INTEGER DEFAULT 0,
    
    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Unique constraint per pengajuan
    CONSTRAINT unique_satker_per_pengajuan UNIQUE (pengajuan_id, ms_satker_id)
);

-- ============================================================================
-- Pengajuan Kebutuhan BMN Satker Barang (Individual Goods per Satker)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_kebutuhan_bmn_satker_barang (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_satker_id UUID NOT NULL REFERENCES perlengkapan.pengajuan_kebutuhan_bmn_satker(id) ON DELETE CASCADE,
    
    -- Goods Information
    nama VARCHAR(255) NOT NULL,
    kode_barang VARCHAR(50),
    jumlah INTEGER NOT NULL DEFAULT 1 CHECK (jumlah > 0),
    satuan VARCHAR(50) DEFAULT 'Unit',
    
    -- Approval Information
    jml_setuju INTEGER DEFAULT 0 CHECK (jml_setuju >= 0),
    alasan TEXT,
    keterangan TEXT,
    
    -- Priority and Scoring
    prioritas INTEGER DEFAULT 0,
    skor NUMERIC(10,2) DEFAULT 0,
    
    -- Supporting Documents
    file_pendukung JSONB DEFAULT '[]'::jsonb,
    
    -- Existing Inventory Reference (from SIMAN)
    existing_count INTEGER DEFAULT 0,
    existing_condition TEXT,
    
    -- Audit Trail
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- Pengajuan Kebutuhan BMN Satker Aktivitas (Workflow History)
-- ============================================================================
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_satker_id UUID NOT NULL REFERENCES perlengkapan.pengajuan_kebutuhan_bmn_satker(id) ON DELETE CASCADE,
    
    -- Workflow Transition
    from_status_kode INTEGER,
    to_status_kode INTEGER NOT NULL REFERENCES perlengkapan.ms_aktivitas_bmn(kode),
    
    -- Actor Information (from Authenc)
    user_id UUID,
    nip VARCHAR(30),
    nama VARCHAR(255),
    pangkat VARCHAR(100),
    jabatan VARCHAR(255),
    role VARCHAR(100),
    
    -- Action Details
    aksi VARCHAR(50) NOT NULL,
    komentar TEXT,
    
    -- Timestamp
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- Indexes for Performance
-- ============================================================================

-- Main table indexes
CREATE INDEX IF NOT EXISTS idx_pkb_tahun ON perlengkapan.pengajuan_kebutuhan_bmn(tahun);
CREATE INDEX IF NOT EXISTS idx_pkb_status ON perlengkapan.pengajuan_kebutuhan_bmn(status_kode);
CREATE INDEX IF NOT EXISTS idx_pkb_created_at ON perlengkapan.pengajuan_kebutuhan_bmn(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_pkb_created_by ON perlengkapan.pengajuan_kebutuhan_bmn(created_by);

-- Asset types indexes
CREATE INDEX IF NOT EXISTS idx_pkb_asset_pengajuan ON perlengkapan.pengajuan_kebutuhan_bmn_asset(pengajuan_id);
CREATE INDEX IF NOT EXISTS idx_pkb_asset_kode ON perlengkapan.pengajuan_kebutuhan_bmn_asset(kode_barang);

-- Satker indexes
CREATE INDEX IF NOT EXISTS idx_pkb_satker_pengajuan ON perlengkapan.pengajuan_kebutuhan_bmn_satker(pengajuan_id);
CREATE INDEX IF NOT EXISTS idx_pkb_satker_id ON perlengkapan.pengajuan_kebutuhan_bmn_satker(ms_satker_id);
CREATE INDEX IF NOT EXISTS idx_pkb_satker_status ON perlengkapan.pengajuan_kebutuhan_bmn_satker(status_kode);
CREATE INDEX IF NOT EXISTS idx_pkb_satker_prioritas ON perlengkapan.pengajuan_kebutuhan_bmn_satker(prioritas);

-- Barang indexes
CREATE INDEX IF NOT EXISTS idx_pkb_barang_satker ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(pengajuan_satker_id);
CREATE INDEX IF NOT EXISTS idx_pkb_barang_kode ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(kode_barang);
CREATE INDEX IF NOT EXISTS idx_pkb_barang_prioritas ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(prioritas);
CREATE INDEX IF NOT EXISTS idx_pkb_barang_skor ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(skor DESC);

-- Aktivitas indexes
CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_satker ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(pengajuan_satker_id);
CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_user ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(user_id);
CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_created ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(created_at DESC);

-- ============================================================================
-- Views for Reporting
-- ============================================================================

-- Summary view for dashboard statistics
CREATE OR REPLACE VIEW perlengkapan.vw_kebutuhan_bmn_summary AS
SELECT 
    p.id,
    p.nama,
    p.tahun,
    p.status_kode,
    m.nama AS status_nama,
    COUNT(DISTINCT ps.id) AS total_satker,
    COUNT(DISTINCT psb.id) AS total_barang,
    COALESCE(SUM(psb.jumlah), 0) AS total_jumlah_diminta,
    COALESCE(SUM(psb.jml_setuju), 0) AS total_jumlah_disetujui,
    p.created_at,
    p.updated_at
FROM perlengkapan.pengajuan_kebutuhan_bmn p
LEFT JOIN perlengkapan.ms_aktivitas_bmn m ON p.status_kode = m.kode
LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON p.id = ps.pengajuan_id
LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang psb ON ps.id = psb.pengajuan_satker_id
GROUP BY p.id, p.nama, p.tahun, p.status_kode, m.nama, p.created_at, p.updated_at;

-- ============================================================================
-- Trigger for Updated At
-- ============================================================================
CREATE OR REPLACE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at()
RETURNS TRIGGER AS $BODY$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$BODY$ LANGUAGE plpgsql;

-- Apply triggers to main tables
CREATE TRIGGER trg_pkb_updated_at
    BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();

CREATE TRIGGER trg_pkb_satker_updated_at
    BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn_satker
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();

CREATE TRIGGER trg_pkb_barang_updated_at
    BEFORE UPDATE ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
    FOR EACH ROW EXECUTE FUNCTION perlengkapan.update_kebutuhan_bmn_updated_at();

-- ============================================================================
-- Comments for Documentation
-- ============================================================================
COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn IS 'Main entity for BMN needs analysis requests';
COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_asset IS 'Asset types included in a BMN request';
COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker IS 'Per-satker tracking for BMN requests';
COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang IS 'Individual goods requested per satker';
COMMENT ON TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas IS 'Workflow history audit trail';
COMMENT ON TABLE perlengkapan.ms_aktivitas_bmn IS 'Master workflow status codes for BMN requests';
