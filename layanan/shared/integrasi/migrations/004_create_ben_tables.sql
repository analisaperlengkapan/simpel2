-- ==================== MODUL BENDAHARA (BEN) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: ben_kas_tunai
-- Endpoint: /API/BEN/kasTunai/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_kas_tunai (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    thang VARCHAR(255),
    kategori_kas VARCHAR(3),
    jumlah NUMERIC(19,2),
    no_referensi VARCHAR(50),
    tgl_transaksi DATE,
    kode_sumber_dana VARCHAR(1),
    uraian VARCHAR(2000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_kas_bank
-- Endpoint: /API/BEN/kasBank/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_kas_bank (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    kategori_kas VARCHAR(3),
    jumlah NUMERIC(19,2),
    no_referensi VARCHAR(50),
    tgl_transaksi DATE,
    uraian VARCHAR(2000),
    kode_sumber_dana VARCHAR(1),
    thang VARCHAR(255),
    id_rek_bank NUMERIC(19,0),
    nama_bank VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_spby
-- Endpoint: /API/BEN/spby/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_spby (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    id_bast VARCHAR(255),
    wajib_pajak_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    thang VARCHAR(255),
    jenis_perintah_bayar VARCHAR(1),
    no_perintah_bayar VARCHAR(50),
    ref_kwitansi_supplier VARCHAR(50),
    no_uang_muka VARCHAR(50),
    is_returned NUMERIC(1,0),
    tgl_perintah_bayar DATE,
    cara_tarik VARCHAR(255),
    no_register VARCHAR(255),
    status_kwitansi NUMERIC(10,0),
    status_pungutan NUMERIC(10,0),
    um_returned NUMERIC(1,0),
    status_spp NUMERIC(10,0),
    status_sptb NUMERIC(10,0),
    kode_unit_teknis VARCHAR(10),
    uraian_perintah_bayar VARCHAR(255),
    uraian VARCHAR(225),
    keterangan_validasi VARCHAR(255),
    ada_bast NUMERIC(1,0),
    kategori_pengeluaran NUMERIC(1,0),
    tahun_anggaran_hibah VARCHAR(1),
    kategori_pengeluaran_hibah VARCHAR(1),
    kode_akun VARCHAR(255),
    jumlah NUMERIC(19,2),
    status_validasi NUMERIC(1,0),
    kode_coa VARCHAR(255),
    nama_wp VARCHAR(100),
    npwp_wp VARCHAR(20),
    nip_ppk VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_kuitansi
-- Endpoint: /API/BEN/kuitansi/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_kuitansi (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    non_barang NUMERIC(1,0),
    cara_bayar VARCHAR(1),
    jabatan_penerima VARCHAR(100),
    jns_kuitansi VARCHAR(1),
    jumlah NUMERIC(19,2),
    keterangan VARCHAR(2000),
    nama_penerima VARCHAR(100),
    no_kwitansi VARCHAR(50),
    status_barang NUMERIC(1,0),
    status_kwitansi VARCHAR(1),
    thang VARCHAR(255),
    tgl_bayar DATE,
    tgl_kwitansi DATE,
    perintah_bayar_id NUMERIC(19,0),
    kode_coa VARCHAR(255),
    no_drpp_id NUMERIC(19,0),
    nip_ppk VARCHAR(255),
    id_rek_bank NUMERIC(19,0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_drpp
-- Endpoint: /API/BEN/drpp/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_drpp (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(255),
    kode_coa VARCHAR(255),
    jenis_kuitansi VARCHAR(1),
    total_jumlah NUMERIC(19,2),
    no_drpp VARCHAR(50),
    status_drpp VARCHAR(1),
    thang VARCHAR(255),
    tgl_drpp DATE,
    tahun_ang_hibah VARCHAR(255),
    id_spp NUMERIC(19,0),
    jumlah NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_pungut_pajak
-- Endpoint: /API/BEN/pungutPajak/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_pungut_pajak (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    thang VARCHAR(255),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    dasar_pungut VARCHAR(1),
    keterangan VARCHAR(2000),
    no_bukti_pungut VARCHAR(50),
    tgl_pungut DATE,
    perintah_bayar_id NUMERIC(19,0),
    wajib_pajak_id NUMERIC(19,0),
    nama_wajib_pajak VARCHAR(100),
    no_rekening VARCHAR(20),
    kode_bank VARCHAR(255),
    nama_bank VARCHAR(4000),
    alamat VARCHAR(255),
    jenis_pem_kas VARCHAR(1),
    kode_akun VARCHAR(6),
    jumlah NUMERIC(19,2),
    status VARCHAR(1),
    setoran_pajak_id NUMERIC(19,0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_setor_pajak
-- Endpoint: /API/BEN/setorPajak/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_setor_pajak (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    thang VARCHAR(255),
    kode_akun VARCHAR(6),
    alamat_object_pajak VARCHAR(255),
    kode_bank VARCHAR(255),
    cabang VARCHAR(50),
    cara_setor VARCHAR(1),
    jenis_setoran VARCHAR(50),
    jumlah_setor_pajak NUMERIC(19,2),
    keterangan VARCHAR(2000),
    masa_pajak VARCHAR(6),
    no_ketetapan VARCHAR(50),
    no_object_pajak VARCHAR(50),
    no_ref_gl VARCHAR(50),
    no_ssp VARCHAR(50),
    ntpn VARCHAR(20),
    tgl_setoran DATE,
    tgl_terima_bank DATE,
    nama_wajib_pajak VARCHAR(100),
    ntb VARCHAR(20),
    tgl_buku DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_pnbp
-- Endpoint: /API/BEN/pnbp/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS ben_pnbp (
    id BIGSERIAL PRIMARY KEY,
    api_id NUMERIC(19,0),
    kode_kementerian VARCHAR(255),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(50),
    kode_program VARCHAR(255),
    kode_kegiatan VARCHAR(255),
    kode_output VARCHAR(255),
    kode_kppn VARCHAR(255),
    kode_akun VARCHAR(255),
    kode_bank VARCHAR(255),
    kode_sub_fungsi VARCHAR(255),
    thang VARCHAR(255),
    cabang VARCHAR(50),
    ntpn VARCHAR(20),
    cara_setor VARCHAR(1),
    jenis_pengembalian VARCHAR(2),
    jenis_pungut VARCHAR(1),
    jumlah NUMERIC(19,2),
    keterangan VARCHAR(2000),
    no_spn VARCHAR(50),
    tgl_spn DATE,
    no_ssbp VARCHAR(50),
    status VARCHAR(1),
    tgl_ssbp DATE,
    tgl_terima_bank DATE,
    wajib_pajak_id NUMERIC(19,0),
    flag_owner NUMERIC(1,0),
    ntb VARCHAR(20),
    tgl_buku DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_tup
-- Endpoint: /API/BEN/tup/KLxxx/KDSATKER
-- Data pengajuan TUP satker
CREATE TABLE IF NOT EXISTS ben_tup (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    thang VARCHAR(4),
    no_tup VARCHAR(50),
    tgl_tup DATE,
    nilai_tup NUMERIC(19,2),
    jenis_tup VARCHAR(1),
    status_tup VARCHAR(1),
    no_spm VARCHAR(50),
    tgl_spm DATE,
    no_sp2d VARCHAR(50),
    tgl_sp2d DATE,
    nilai_sp2d NUMERIC(19,2),
    keterangan VARCHAR(2000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: ben_pengembalian
-- Endpoint: /API/BEN/pengembalian/KLxxx/KDSATKER
-- Pengembalian belanja pada modul bendahara (SSPB)
CREATE TABLE IF NOT EXISTS ben_pengembalian (
    id BIGSERIAL PRIMARY KEY,
    api_id BIGINT,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    kode_unit_teknis VARCHAR(10),
    thang VARCHAR(4),
    no_sspb VARCHAR(50),
    tgl_sspb DATE,
    no_ntb VARCHAR(30),
    tgl_ntb DATE,
    kode_akun VARCHAR(6),
    nilai_pengembalian NUMERIC(19,2),
    keterangan VARCHAR(2000),
    status VARCHAR(1),
    jenis_pengembalian VARCHAR(2),
    kode_coa VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ben_kas_tunai_kdsatker ON ben_kas_tunai(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_kas_bank_kdsatker ON ben_kas_bank(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_spby_kdsatker ON ben_spby(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_kuitansi_kdsatker ON ben_kuitansi(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_drpp_kdsatker ON ben_drpp(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_drpp_id_spp ON ben_drpp(id_spp);
CREATE INDEX IF NOT EXISTS idx_ben_kuitansi_no_drpp_id ON ben_kuitansi(no_drpp_id);
CREATE INDEX IF NOT EXISTS idx_ben_tup_kdsatker ON ben_tup(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_tup_api_id ON ben_tup(api_id);
CREATE INDEX IF NOT EXISTS idx_ben_pengembalian_kdsatker ON ben_pengembalian(kdsatker);
CREATE INDEX IF NOT EXISTS idx_ben_pengembalian_api_id ON ben_pengembalian(api_id);

COMMENT ON TABLE ben_kas_tunai IS 'Transaksi bendahara dengan jenis pemindahan kas tunai';
COMMENT ON TABLE ben_kas_bank IS 'Transaksi bendahara dengan jenis pemindahan non tunai';
COMMENT ON TABLE ben_spby IS 'Transaksi Surat Perintah Bayar (SPBy)';
COMMENT ON TABLE ben_kuitansi IS 'Kuitansi bukti pembayaran yang dilakukan oleh bendahara';
COMMENT ON TABLE ben_drpp IS 'Daftar Rincian Permintaan Pembayaran yang berisi kumpulan kuitansi yang akan di-SPJ-kan';
COMMENT ON TABLE ben_pungut_pajak IS 'Pungutan Pajak atas transaksi bendahara';
COMMENT ON TABLE ben_setor_pajak IS 'Setoran Pajak atas transaksi bendahara';
COMMENT ON TABLE ben_pnbp IS 'Setoran Penerimaan Negara Bukan Pajak yang dicatat Bendahara';
COMMENT ON TABLE ben_tup IS 'Data pengajuan TUP (Tambah Uang Persediaan) satker';
COMMENT ON TABLE ben_pengembalian IS 'Pengembalian belanja pada modul bendahara (SSPB)';
