-- ==================== MODUL PELAPORAN (GLP) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: glp_buku_besar
-- Endpoint: /API/GLP/bukuBesar/KLxxx/KDSATKER/PERIODE/OPSI_FILTER
CREATE TABLE IF NOT EXISTS glp_buku_besar (
    id BIGSERIAL PRIMARY KEY,
    kdbaes1 VARCHAR(5),
    kdkanwil VARCHAR(3),
    kdwilayah VARCHAR(4),
    kdkppn VARCHAR(3),
    kdsatker VARCHAR(6) NOT NULL,
    kdfungsi VARCHAR(2),
    kdsfung VARCHAR(2),
    kdprogram VARCHAR(4),
    kdgiat VARCHAR(4),
    kdsgiat VARCHAR(5),
    kdoutput VARCHAR(3),
    kdsoutput VARCHAR(3),
    kdkem VARCHAR(1),
    kdkas VARCHAR(1),
    nkas VARCHAR(1),
    kdval VARCHAR(1),
    kdtrn VARCHAR(1),
    kdmakmap VARCHAR(6),
    kddk VARCHAR(1),
    perksai VARCHAR(6),
    perksai1 VARCHAR(6),
    perkkor VARCHAR(6),
    perkkor1 VARCHAR(6),
    kdkm VARCHAR(1),
    kdsdcp VARCHAR(3),
    thnang VARCHAR(4),
    periode VARCHAR(2),
    rphreal NUMERIC(24,2),
    tglkirim DATE,
    tglterima DATE,
    tglupdate DATE,
    flagrev VARCHAR(1),
    kdbapel VARCHAR(3),
    kdes1pel VARCHAR(2),
    jnsdok1 VARCHAR(10),
    nodok1 VARCHAR(255),
    tgldok1 DATE,
    kddekon VARCHAR(2),
    register VARCHAR(8),
    kdjendok VARCHAR(2),
    kdkanwilk VARCHAR(3),
    tglpost DATE,
    revisike NUMERIC(2,0),
    dipake NUMERIC(3,0),
    kdcrbay VARCHAR(1),
    nokarwas VARCHAR(5),
    kdbeban VARCHAR(1),
    kdjnsban VARCHAR(1),
    kdblu VARCHAR(1),
    noregis VARCHAR(40),
    kdvalas VARCHAR(3),
    nilkurs NUMERIC(20,2),
    tglkurs DATE,
    kdkpknl VARCHAR(5),
    stat_rekon VARCHAR(1),
    kategori VARCHAR(40),
    trn_bmn VARCHAR(3),
    nil_valas NUMERIC(20,2),
    cad1 VARCHAR(10),
    cad2 VARCHAR(10),
    cad3 VARCHAR(10),
    hapus NUMERIC(1,0),
    api_id NUMERIC(10,0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: glp_neraca_sawal
-- Endpoint: /API/GLP/neracaSawal/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS glp_neraca_sawal (
    id BIGSERIAL PRIMARY KEY,
    kdbaes1 VARCHAR(5),
    kdsatker VARCHAR(6) NOT NULL,
    akun VARCHAR(6),
    nilai NUMERIC,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(kdsatker, akun)
);

-- Tabel: glp_fa_detail
-- Endpoint: /API/GLP/faDetail/KLxxx/KDSATKER/PERIODE
CREATE TABLE IF NOT EXISTS glp_fa_detail (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kdsatker VARCHAR(6) NOT NULL,
    kode_kementerian VARCHAR(3),
    deskripsi_trans VARCHAR(255),
    jenis_dokumen VARCHAR(3),
    kode_coa VARCHAR(100),
    kode_mata_uang_trans VARCHAR(10),
    kode_periode VARCHAR(7),
    kode_sdata VARCHAR(4),
    kurs NUMERIC(19,2),
    nilai_rupiah NUMERIC(19,2),
    nilai_trans_valas NUMERIC(19,2),
    no_dok VARCHAR(255),
    nomor_dipa VARCHAR(30),
    tanggal_dipa DATE,
    tgl_dok DATE,
    tgl_jurnal DATE,
    id_trn_modul VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_kdsatker ON glp_buku_besar(kdsatker);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_periode ON glp_buku_besar(periode);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_thnang ON glp_buku_besar(thnang);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_kdprogram ON glp_buku_besar(kdprogram);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_kdgiat ON glp_buku_besar(kdgiat);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_kdmakmap ON glp_buku_besar(kdmakmap);
CREATE INDEX IF NOT EXISTS idx_glp_buku_besar_tglpost ON glp_buku_besar(tglpost);
CREATE INDEX IF NOT EXISTS idx_glp_neraca_sawal_kdsatker ON glp_neraca_sawal(kdsatker);
CREATE INDEX IF NOT EXISTS idx_glp_neraca_sawal_akun ON glp_neraca_sawal(akun);
CREATE INDEX IF NOT EXISTS idx_glp_fa_detail_kdsatker ON glp_fa_detail(kdsatker);
CREATE INDEX IF NOT EXISTS idx_glp_fa_detail_kode_periode ON glp_fa_detail(kode_periode);

COMMENT ON TABLE glp_buku_besar IS 'Data seluruh jurnal-jurnal transaksi pada aplikasi SAKTI';
COMMENT ON TABLE glp_neraca_sawal IS 'Saldo awal neraca atau saldo akhir pada tahun lalu (data statis)';
COMMENT ON TABLE glp_fa_detail IS 'Data seluruh transaksi pada seluruh modul yang berdampak pada perubahan angka di FA akrual';
COMMENT ON COLUMN glp_buku_besar.kdkem IS 'Kode Pengembalian: 0=Realisasi, 1=Pengembalian';
COMMENT ON COLUMN glp_buku_besar.kddk IS 'Kode Debet atau Kredit';
COMMENT ON COLUMN glp_buku_besar.perksai IS 'Kode Perkiraan Kas pada Posisi Debet';
COMMENT ON COLUMN glp_buku_besar.perksai1 IS 'Kode Perkiraan Kas pada Posisi Kredit';
COMMENT ON COLUMN glp_buku_besar.perkkor IS 'Kode Perkiraan Akrual pada Posisi Debet';
COMMENT ON COLUMN glp_buku_besar.perkkor1 IS 'Kode Perkiraan Akrual pada Posisi Kredit';
COMMENT ON COLUMN glp_buku_besar.api_id IS 'ID sebuah transaksi (didapat otomatis dari sistem)';
