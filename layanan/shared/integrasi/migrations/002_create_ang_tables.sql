-- ==================== MODUL PENGANGGARAN (ANG) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: ang_data_ang
-- Endpoint: /API/ANG/dataAng/KLxxx/KDSATKER/KODE_STS_HISTORY
CREATE TABLE IF NOT EXISTS ang_data_ang (
    id BIGSERIAL PRIMARY KEY,
    kdsatker VARCHAR(255) NOT NULL,
    kode_program VARCHAR(255),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(255),
    kdib VARCHAR(255),
    volume_output DOUBLE PRECISION,
    kode_suboutput VARCHAR(255),
    volume_suboutput DOUBLE PRECISION,
    kode_komponen VARCHAR(255),
    kode_subkomponen VARCHAR(255),
    uraian_subkomponen VARCHAR(255),
    kode_akun VARCHAR(255),
    kode_jenis_beban VARCHAR(255),
    kode_cara_tarik VARCHAR(255),
    kode_jenis_bantuan VARCHAR(255),
    kode_register VARCHAR(255),
    header1 NUMERIC(1,0),
    header2 NUMERIC(1,0),
    kode_item VARCHAR(255),
    nomor_item NUMERIC(10,0),
    cons_item NUMERIC,
    uraian_item VARCHAR(255),
    sumber_dana VARCHAR(255),
    vol_keg_1 DOUBLE PRECISION,
    sat_keg_1 VARCHAR(255),
    vol_keg_2 DOUBLE PRECISION,
    sat_keg_2 VARCHAR(255),
    vol_keg_3 DOUBLE PRECISION,
    sat_keg_3 VARCHAR(255),
    vol_keg_4 DOUBLE PRECISION,
    sat_keg_4 VARCHAR(255),
    volkeg DOUBLE PRECISION,
    satkeg VARCHAR(255),
    hargasat NUMERIC(19,2),
    total NUMERIC(19,2),
    kode_blokir VARCHAR(255),
    nilai_blokir NUMERIC(19,2),
    kode_sts_history VARCHAR(255),
    pok_nilai_1 NUMERIC(19,2),
    pok_nilai_2 NUMERIC(19,2),
    pok_nilai_3 NUMERIC(19,2),
    pok_nilai_4 NUMERIC(19,2),
    pok_nilai_5 NUMERIC(19,2),
    pok_nilai_6 NUMERIC(19,2),
    pok_nilai_7 NUMERIC(19,2),
    pok_nilai_8 NUMERIC(19,2),
    pok_nilai_9 NUMERIC(19,2),
    pok_nilai_10 NUMERIC(19,2),
    pok_nilai_11 NUMERIC(19,2),
    pok_nilai_12 NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ang_ref_sts
-- Endpoint: /API/ANG/refSts/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ang_ref_sts (
    id BIGSERIAL PRIMARY KEY,
    kdsatker VARCHAR(255) NOT NULL,
    kode_sts_history VARCHAR(255) NOT NULL,
    jenis_revisi VARCHAR(100),
    keterangan TEXT,
    flag_update_coa SMALLINT,
    total_pagu NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kdsatker, kode_sts_history)
);

-- Tabel: ang_pendapatan
-- Endpoint: /API/ANG/pendapatan/KLxxx/KDSATKER/KODE_STS_HISTORY
-- Rencana anggaran pendapatan
CREATE TABLE IF NOT EXISTS ang_pendapatan (
    id BIGSERIAL PRIMARY KEY,
    kdsatker VARCHAR(255) NOT NULL,
    kode_sts_history VARCHAR(255),
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(3),
    kode_akun VARCHAR(6),
    kode_item VARCHAR(22),
    uraian_item VARCHAR(255),
    volume DOUBLE PRECISION,
    satuan VARCHAR(50),
    harga_satuan NUMERIC(19,2),
    total NUMERIC(19,2),
    rpd_nilai_1 NUMERIC(19,2),
    rpd_nilai_2 NUMERIC(19,2),
    rpd_nilai_3 NUMERIC(19,2),
    rpd_nilai_4 NUMERIC(19,2),
    rpd_nilai_5 NUMERIC(19,2),
    rpd_nilai_6 NUMERIC(19,2),
    rpd_nilai_7 NUMERIC(19,2),
    rpd_nilai_8 NUMERIC(19,2),
    rpd_nilai_9 NUMERIC(19,2),
    rpd_nilai_10 NUMERIC(19,2),
    rpd_nilai_11 NUMERIC(19,2),
    rpd_nilai_12 NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ang_data_ang_kdsatker ON ang_data_ang(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ang_data_ang_kode_sts_history ON ang_data_ang(kode_sts_history);
CREATE INDEX IF NOT EXISTS idx_ang_data_ang_cons_item ON ang_data_ang(cons_item);
CREATE INDEX IF NOT EXISTS idx_ang_data_ang_header ON ang_data_ang(header1, header2);
CREATE INDEX IF NOT EXISTS idx_ang_ref_sts_kdsatker ON ang_ref_sts(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ang_pendapatan_kdsatker ON ang_pendapatan(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ang_pendapatan_kode_sts_history ON ang_pendapatan(kode_sts_history);

COMMENT ON TABLE ang_data_ang IS 'Data transaksi penganggaran dari program sampai detail item beserta informasi RPD bulanan (Halaman III DIPA)';
COMMENT ON TABLE ang_ref_sts IS 'Referensi masing-masing satker memiliki status history apa saja';
COMMENT ON TABLE ang_pendapatan IS 'Rencana anggaran pendapatan';
COMMENT ON COLUMN ang_data_ang.header1 IS 'Filter untuk pagu akurat: header1=0 AND header2=0';
COMMENT ON COLUMN ang_data_ang.header2 IS 'Filter untuk pagu akurat: header1=0 AND header2=0';
COMMENT ON COLUMN ang_data_ang.cons_item IS 'Untuk join dengan realisasi (KODE_ITEM)';
