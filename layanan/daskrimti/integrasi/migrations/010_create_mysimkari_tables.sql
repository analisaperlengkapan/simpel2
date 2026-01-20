-- Migration: 010_create_mysimkari_tables.sql
-- Description: Create tables for MySIMKARI API data
-- Date: 2025-10-30
-- Updated: 2025-11-06 - Changed all data columns to TEXT for API flexibility
-- Note: All columns except UUID are TEXT to handle inconsistent API data types
--       (e.g., kode_satker can be "00", "01", "01.01", or numeric 1.01)

SET search_path TO integrasi, public;

-- =============================================
-- MySIMKARI Satker Table
-- =============================================

CREATE TABLE IF NOT EXISTS mysimkari_satker (
    id UUID PRIMARY KEY,
    parent_id UUID,
    nama_satker TEXT NOT NULL,
    tipe_satker TEXT,
    alamat_satker TEXT,
    kode_satker TEXT,
    telp_satker TEXT,
    website_satker TEXT,
    city TEXT,
    long TEXT,
    lat TEXT,
    provinsi TEXT,
    wilayah TEXT,
    kategori_satker TEXT,
    pulau TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index untuk performa query
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_parent_id ON mysimkari_satker(parent_id);
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_provinsi ON mysimkari_satker(provinsi);
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_kode ON mysimkari_satker(kode_satker);

-- =============================================
-- MySIMKARI Pegawai Table
-- =============================================

CREATE TABLE IF NOT EXISTS mysimkari_pegawai (
    id SERIAL PRIMARY KEY,
    nama TEXT,
    nip TEXT,
    no_hp TEXT,
    email_dinas TEXT,
    bidang TEXT,
    foto TEXT,
    jk TEXT,
    agama TEXT,
    nrp TEXT,
    jabatan TEXT,
    golpang TEXT,
    jenis_jabatan_terakhir TEXT,
    jabat_tmt TEXT,
    eselon TEXT,
    nama_satker TEXT,
    gol_kd TEXT,
    satker_id UUID REFERENCES mysimkari_satker(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index untuk performa query
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_nip ON mysimkari_pegawai(nip);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mysimkari_pegawai_nip_unique ON mysimkari_pegawai(nip) WHERE nip IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_satker_id ON mysimkari_pegawai(satker_id);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_bidang ON mysimkari_pegawai(bidang);
CREATE INDEX IF NOT EXISTS idx_mysimkari_pegawai_gol_kd ON mysimkari_pegawai(gol_kd);

-- =============================================
-- Comments
-- =============================================

COMMENT ON TABLE mysimkari_satker IS 'Tabel untuk menyimpan data satker dari MySIMKARI API';
COMMENT ON TABLE mysimkari_pegawai IS 'Tabel untuk menyimpan data pegawai dari MySIMKARI API';

COMMENT ON COLUMN mysimkari_satker.id IS 'UUID unik dari MySIMKARI';
COMMENT ON COLUMN mysimkari_satker.parent_id IS 'ID satker induk (hierarki)';
COMMENT ON COLUMN mysimkari_satker.long IS 'Koordinat longitude (stored as TEXT untuk flexibility)';
COMMENT ON COLUMN mysimkari_satker.lat IS 'Koordinat latitude (stored as TEXT untuk flexibility)';
COMMENT ON COLUMN mysimkari_satker.kode_satker IS 'Kode satuan kerja (format: 00, 01, 01.01, dll)';

COMMENT ON COLUMN mysimkari_pegawai.nip IS 'Nomor Induk Pegawai';
COMMENT ON COLUMN mysimkari_pegawai.nrp IS 'Nomor Registrasi Pokok';
COMMENT ON COLUMN mysimkari_pegawai.jabat_tmt IS 'Tanggal mulai jabatan (format: YYYY-MM-DD)';
COMMENT ON COLUMN mysimkari_pegawai.gol_kd IS 'Kode golongan';
COMMENT ON COLUMN mysimkari_pegawai.satker_id IS 'Foreign key ke mysimkari_satker.id (injected dari parent satker)';
