-- ==================== MODUL ADMINISTRASI (ADM) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

-- Create integrasi schema (all tables will be in this schema)
CREATE SCHEMA IF NOT EXISTS integrasi;

-- Set search path
SET search_path TO integrasi, public;

-- Tabel: adm_ref_admin
-- Endpoint: /API/ADM/refAdmin/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS adm_ref_admin (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kode_unit VARCHAR(255),
    kdsatker VARCHAR(255) NOT NULL,
    deskripsi VARCHAR(4000),
    kode_kab_kota VARCHAR(255),
    uraian_kode_kab_kota VARCHAR(4000),
    kode_kewenangan VARCHAR(255),
    uraian_kewenangan VARCHAR(4000),
    kode_kppn VARCHAR(255),
    nama_kppn VARCHAR(4000),
    kode_uappbw VARCHAR(255), -- Unit Akuntansi Pembantu Pengguna Anggaran Bendahara Wilayah
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kode_kementerian, kdsatker)
);

-- Tabel: adm_pejabat
-- Endpoint: /API/ADM/pejabat/KLxxx (GLOBAL, bukan per satker)
-- Note: Satu pejabat (NIP) bisa berada di multiple satker
CREATE TABLE IF NOT EXISTS adm_pejabat (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kode_unit VARCHAR(2),
    kdsatker VARCHAR(255) NOT NULL,
    nama VARCHAR(100),
    nip VARCHAR(18),
    nik VARCHAR(16),
    no_npwp VARCHAR(20),
    telpon VARCHAR(20),
    email VARCHAR(100),
    jabatan VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (kdsatker, nip) -- Prevent duplicate per satker
);

-- Indexes untuk adm_pejabat
CREATE INDEX IF NOT EXISTS idx_adm_pejabat_kdsatker ON adm_pejabat(kdsatker);
CREATE INDEX IF NOT EXISTS idx_adm_pejabat_nip ON adm_pejabat(nip);

-- Tabel: adm_ref_jns_spp
-- Endpoint: /API/ADM/refJnsSPP/KLxxx/KODE
CREATE TABLE IF NOT EXISTS adm_ref_jns_spp (
    id BIGSERIAL PRIMARY KEY,
    kode VARCHAR(255) NOT NULL,
    deskripsi VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kode)
);

-- Tabel: adm_ref_uraian
-- Endpoint: /API/ADM/refUraian/KLxxx/JENIS/KODE
-- Referensi uraian kode program s.d. komponen
CREATE TABLE IF NOT EXISTS adm_ref_uraian (
    id BIGSERIAL PRIMARY KEY,
    kode VARCHAR(10) NOT NULL,
    deskripsi TEXT,
    thang VARCHAR(4),
    jenis VARCHAR(20),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kode, thang, jenis)
);

-- Tabel: adm_ref_bank
-- Endpoint: /API/ADM/refBank/KLxxx
-- Referensi bank pada supplierBank
CREATE TABLE IF NOT EXISTS adm_ref_bank (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    nama_bank VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kode_kementerian, nama_bank)
);

-- Tabel: adm_ref_aset
-- Endpoint: /API/ADM/refAset/KLxxx/JENIS/KODE
-- Referensi pengkodean pada modul Aset dan Persediaan
CREATE TABLE IF NOT EXISTS adm_ref_aset (
    id BIGSERIAL PRIMARY KEY,
    kode VARCHAR(10) NOT NULL,
    deskripsi VARCHAR(255),
    jenis VARCHAR(20),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kode, jenis)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_kdsatker ON adm_ref_admin(kdsatker);
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_kode_kementerian ON adm_ref_admin(kode_kementerian);
CREATE INDEX IF NOT EXISTS idx_adm_pejabat_kdsatker ON adm_pejabat(kdsatker);
CREATE INDEX IF NOT EXISTS idx_adm_pejabat_nip ON adm_pejabat(nip);
CREATE INDEX IF NOT EXISTS idx_adm_ref_uraian_kode ON adm_ref_uraian(kode);
CREATE INDEX IF NOT EXISTS idx_adm_ref_uraian_jenis ON adm_ref_uraian(jenis);
CREATE INDEX IF NOT EXISTS idx_adm_ref_bank_kode_kementerian ON adm_ref_bank(kode_kementerian);
CREATE INDEX IF NOT EXISTS idx_adm_ref_aset_kode ON adm_ref_aset(kode);
CREATE INDEX IF NOT EXISTS idx_adm_ref_aset_jenis ON adm_ref_aset(jenis);

COMMENT ON TABLE adm_ref_admin IS 'Referensi data satker pada masing-masing kementerian';
COMMENT ON TABLE adm_pejabat IS 'Data pejabat yang telah di input pada menu Pejabat pada aplikasi SAKTI';
COMMENT ON TABLE adm_ref_jns_spp IS 'Referensi uraian dari kode jenis spp (KD_JNS_SPP)';
COMMENT ON TABLE adm_ref_uraian IS 'Referensi uraian kode program s.d. komponen';
COMMENT ON TABLE adm_ref_bank IS 'Referensi bank pada supplierBank';
COMMENT ON TABLE adm_ref_aset IS 'Referensi pengkodean pada modul Aset dan Persediaan';
