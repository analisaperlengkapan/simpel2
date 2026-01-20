-- ==================== MODUL ASET (AST) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: ast_aset_trx
-- Endpoint: /API/AST/asetTrx/KLxxx/KDSATKER/KDGOL/KDBID/KDKEL/KDSKEL/KDBRG
CREATE TABLE IF NOT EXISTS ast_aset_trx (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6) NOT NULL,
    kduakpb VARCHAR(255),
    kdgol VARCHAR(1),
    kdbid VARCHAR(3),
    kdkel VARCHAR(5),
    kdskel VARCHAR(7),
    kdbrg VARCHAR(255),
    nup VARCHAR(100),              -- Changed from NUMERIC: API returns string
    kond VARCHAR(50),              -- Changed from NUMERIC: API returns string
    kdtrx VARCHAR(255),
    q_ast VARCHAR(50),             -- Changed from NUMERIC: API returns string
    q_prb VARCHAR(50),             -- Changed from NUMERIC: API returns string
    na VARCHAR(100),               -- Changed from NUMERIC: API returns string
    nan VARCHAR(100),              -- Changed from NUMERIC: API returns string
    np VARCHAR(100),               -- Changed from NUMERIC: API returns string
    npn VARCHAR(100),              -- Changed from NUMERIC: API returns string
    sm VARCHAR(50),                -- Changed from NUMERIC: API returns string
    mm VARCHAR(50),                -- Changed from NUMERIC: API returns string
    no_sppa VARCHAR(255),
    sts CHAR(1),
    kode_satker_asal VARCHAR(255),
    kode_register VARCHAR(64),
    ket VARCHAR(255),
    no_dok VARCHAR(255),
    jns_ast VARCHAR(50),           -- Changed from NUMERIC: API returns string
    per VARCHAR(50),               -- Changed from NUMERIC: API returns string
    merek_tipe VARCHAR(255),
    ctt VARCHAR(50),               -- Changed from NUMERIC: API returns string
    thn_ang VARCHAR(10),           -- Changed from NUMERIC: API returns string
    created_date VARCHAR(20),      -- Changed from DATE: API returns "15-JUL-25" format
    created_by VARCHAR(50),
    tgl_buku VARCHAR(20),          -- Changed from DATE: API returns "15-JUL-25" format
    tgl_oleh VARCHAR(20),          -- Changed from DATE: API returns "15-JUL-25" format
    tgl_awal_pakai VARCHAR(20),    -- Changed from DATE: API returns "15-JUL-25" format
    jenis_dokumen VARCHAR(100),    -- Added: Missing field from API response
    kode_bast_kuitansi VARCHAR(100), -- Added: Missing field from API response
    no_bast_kuitansi VARCHAR(100), -- Added: Missing field from API response
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ==================== MODUL PERSEDIAAN (PER) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

-- Tabel: per_persedia_trx
-- Endpoint: /API/PER/persediaTrx/KLxxx/KDSATKER/KDGOL/KDBID/KDKEL/KDSKEL/KDBRG
CREATE TABLE IF NOT EXISTS per_persedia_trx (
    id BIGSERIAL PRIMARY KEY,
    kode_akun VARCHAR(255),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6) NOT NULL,
    kdgol VARCHAR(1),
    kdbid VARCHAR(3),
    kdkel VARCHAR(5),
    kdskel VARCHAR(7),
    kode_kpb VARCHAR(255),
    jenis_transaksi VARCHAR(255),
    kdbrg VARCHAR(255),
    kode_persediaan VARCHAR(255),
    nama_persediaan VARCHAR(4000),
    kuantitas NUMERIC,
    nilai NUMERIC,
    tgl_buku VARCHAR(10),
    keterangan VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ast_aset_trx_kdsatker ON ast_aset_trx(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ast_aset_trx_kdgol ON ast_aset_trx(kdgol);
CREATE INDEX IF NOT EXISTS idx_ast_aset_trx_kdbid ON ast_aset_trx(kdbid);
CREATE INDEX IF NOT EXISTS idx_ast_aset_trx_kdbrg ON ast_aset_trx(kdbrg);
CREATE INDEX IF NOT EXISTS idx_ast_aset_trx_kode_register ON ast_aset_trx(kode_register);
CREATE INDEX IF NOT EXISTS idx_per_persedia_trx_kdsatker ON per_persedia_trx(kdsatker);
CREATE INDEX IF NOT EXISTS idx_per_persedia_trx_kdgol ON per_persedia_trx(kdgol);
CREATE INDEX IF NOT EXISTS idx_per_persedia_trx_kdbid ON per_persedia_trx(kdbid);

COMMENT ON TABLE ast_aset_trx IS 'Data transaksi aset historis untuk masing-masing kode barang';
COMMENT ON TABLE per_persedia_trx IS 'Data transaksi persediaan untuk masing-masing kode barang';
COMMENT ON COLUMN ast_aset_trx.nup IS 'No. Asset';
COMMENT ON COLUMN ast_aset_trx.kond IS 'Kode Kondisi: 1=Baik, 2=Rusak Ringan, 3=Rusak Berat';
COMMENT ON COLUMN ast_aset_trx.q_ast IS 'Kuantitas Asset';
COMMENT ON COLUMN ast_aset_trx.q_prb IS 'Kuantitas Perubahan';
COMMENT ON COLUMN ast_aset_trx.na IS 'Nilai Asset (bruto)';
COMMENT ON COLUMN ast_aset_trx.nan IS 'Nilai Asset Neraca (nilai buku)';
COMMENT ON COLUMN ast_aset_trx.np IS 'Nilai Perubahan';
COMMENT ON COLUMN ast_aset_trx.npn IS 'Nilai Perubahan Neraca';
COMMENT ON COLUMN ast_aset_trx.sm IS 'Sisa Masa Manfaat (Semesteran)';
COMMENT ON COLUMN ast_aset_trx.mm IS 'Masa Manfaat (Tahunan)';
COMMENT ON COLUMN ast_aset_trx.sts IS 'Status Asset: 1=Aktif, 2=Henti guna, 3=Mitra';
COMMENT ON COLUMN ast_aset_trx.kode_register IS 'Identitas unik dari setiap aset';
COMMENT ON COLUMN ast_aset_trx.jns_ast IS 'Jenis Asset: 1=Intrakomptabel, 2=Ekstrakomptabel';
COMMENT ON COLUMN ast_aset_trx.ctt IS 'Kode Tercatat: 1=DBR, 2=DBL, 3=KIB';
