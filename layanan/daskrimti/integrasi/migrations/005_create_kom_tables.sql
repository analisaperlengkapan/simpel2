-- ==================== MODUL KOMITMEN (KOM) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: kom_capaian_ro
-- Endpoint: /API/KOM/capaianRO/KLxxx/KDSATKER/KODE_PERIODE
CREATE TABLE IF NOT EXISTS kom_capaian_ro (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kode_unit VARCHAR(2),
    kdsatker VARCHAR(6) NOT NULL,
    sub_output_kode VARCHAR(255),
    kode_periode VARCHAR(7),
    status VARCHAR(255),
    rencana_sub_output NUMERIC(21,4),
    satuan_sub_output VARCHAR(255),
    penambahan_realisasi_volume_ro NUMERIC(21,4),
    total_realisasi_sub_output NUMERIC(21,4),
    penambahan_progress_capaian_ro DOUBLE PRECISION,
    total_progress_capaian_ro DOUBLE PRECISION,
    bukti_dokumen VARCHAR(255),
    referensi_keterangan VARCHAR(365),
    referensi VARCHAR(2),
    keterangan VARCHAR(365),
    ro_strategis NUMERIC(1,0),
    anggaran_belanja NUMERIC(19,2),
    realisasi_belanja NUMERIC(19,2),
    pengembalian_belanja NUMERIC(19,2),
    persen_gap DOUBLE PRECISION,
    revisi_dipa_ke NUMERIC(10,0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_kontrak_header
-- Endpoint: /API/KOM/kontrakHeader/KLxxx/KDSATKER
CREATE TABLE IF NOT EXISTS kom_kontrak_header (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    thn_ang VARCHAR(255),
    id_kontrak NUMERIC(19,0) UNIQUE NOT NULL,
    no_kontrak VARCHAR(150),
    tanggal_kontrak VARCHAR(17),
    tanggal_mulai_pelaksanaan VARCHAR(17),
    tanggal_selesai_pelaksanaan VARCHAR(17),
    nilai_kontrak NUMERIC(21,4),
    mata_uang VARCHAR(255),
    tipe_kontrak VARCHAR(4000),
    nomor_can VARCHAR(100),
    jenis_kontrak VARCHAR(4000),
    uraian_kontrak VARCHAR(240),
    id_supplier NUMERIC(19,0),
    nama_supplier VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_kontrak_line
-- Endpoint: /API/KOM/kontrakLine/KLxxx/KDSATKER/ID_KONTRAK/ID_LINE_KONTRAK
CREATE TABLE IF NOT EXISTS kom_kontrak_line (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    id_kontrak NUMERIC(19,0) NOT NULL,
    id_line_kontrak NUMERIC(19,0) NOT NULL,
    deskripsi_line VARCHAR(240),
    cara_tarik VARCHAR(4000),
    nilai_line NUMERIC(19,2),
    tipe_line VARCHAR(30),
    urutan_line NUMERIC(10,0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_kontrak) REFERENCES kom_kontrak_header(id_kontrak),
    UNIQUE(id_kontrak, id_line_kontrak)
);

-- Tabel: kom_kontrak_termin
-- Endpoint: /API/KOM/kontrakTermin/KLxxx/KDSATKER/ID_KONTRAK/ID_LINE_KONTRAK/ID_JADWAL_PEMBAYARAN
CREATE TABLE IF NOT EXISTS kom_kontrak_termin (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    id_kontrak NUMERIC(19,0) NOT NULL,
    id_line_kontrak NUMERIC(19,0) NOT NULL,
    id_jadwal_pembayaran NUMERIC(19,0) NOT NULL,
    termin_ke VARCHAR(50),
    deskripsi_termin VARCHAR(255),
    tanggal_termin VARCHAR(17),
    nilai_termin NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(id_kontrak, id_line_kontrak, id_jadwal_pembayaran)
);

-- Tabel: kom_kontrak_coa
-- Endpoint: /API/KOM/kontrakCOA/KLxxx/KDSATKER/ID_KONTRAK/ID_LINE_KONTRAK/ID_JADWAL_PEMBAYARAN
CREATE TABLE IF NOT EXISTS kom_kontrak_coa (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    id_kontrak NUMERIC(19,0) NOT NULL,
    id_line_kontrak NUMERIC(19,0) NOT NULL,
    id_jadwal_pembayaran NUMERIC(19,0) NOT NULL,
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(3),
    kode_akun VARCHAR(6),
    kode_suboutput VARCHAR(3),
    kode_komponen VARCHAR(3),
    kode_subkomponen VARCHAR(2),
    kode_item VARCHAR(6),
    kode_coa VARCHAR(100),
    vol_suboutput NUMERIC(21,4),
    nilai_coa_detail NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_bast_kontrak_header
-- Endpoint: /API/KOM/BASTKontrakHeader/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_kontrak_header (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    thn_ang VARCHAR(255),
    id_bast NUMERIC(19,0) UNIQUE NOT NULL,
    no_kontrak VARCHAR(255),
    no_bast VARCHAR(50),
    tanggal_bast VARCHAR(17),
    kategori_bast VARCHAR(4000),
    nilai_bast NUMERIC,
    nomor_dan_status_spp VARCHAR(4000),
    jenis_spp VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_bast_kontrak_detail_barang
-- Endpoint: /API/KOM/BASTKontrakDetailBarang/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_kontrak_detail_barang (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    id_bast NUMERIC(19,0) NOT NULL,
    kode_barang VARCHAR(255),
    nama_barang VARCHAR(4000),
    jumlah_barang NUMERIC(10,0),
    nilai_total_barang NUMERIC,
    status_pendetailan CHAR(17),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_bast) REFERENCES kom_bast_kontrak_header(id_bast)
);

-- Tabel: kom_bast_kontrak_coa
-- Endpoint: /API/KOM/BASTKontrakCOA/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_kontrak_coa (
    id BIGSERIAL PRIMARY KEY,
    id_bast NUMERIC(19,0) NOT NULL,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_akun VARCHAR(6),
    kode_output VARCHAR(3),
    kode_suboutput VARCHAR(3),
    kode_komponen VARCHAR(3),
    kode_subkomponen VARCHAR(2),
    kode_item VARCHAR(6),
    kode_coa VARCHAR(100),
    vol_suboutput NUMERIC(21,4),
    nilai_coa_detail NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_bast) REFERENCES kom_bast_kontrak_header(id_bast)
);

-- Tabel: kom_bast_non_kontrak_header
-- Endpoint: /API/KOM/BASTNonKontrakHeader/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_non_kontrak_header (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    thn_ang VARCHAR(255),
    id_bast NUMERIC(19,0) UNIQUE NOT NULL,
    no_dokumen VARCHAR(50),
    tanggal_bast VARCHAR(17),
    kategori_bast VARCHAR(4000),
    uraian_bast VARCHAR(255),
    nilai_bast NUMERIC(19,2),
    supplier_wp_wb VARCHAR(4000),
    nomor_dan_status_spp_spby VARCHAR(4000),
    jenis_spp_spby VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_bast_non_kontrak_detail_barang
-- Endpoint: /API/KOM/BASTNonKontrakDetailBarang/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_non_kontrak_detail_barang (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255) NOT NULL,
    id_bast NUMERIC(19,0) NOT NULL,
    kode_barang VARCHAR(255),
    nama_barang VARCHAR(4000),
    jumlah_barang NUMERIC(10,0),
    nilai_total_barang NUMERIC,
    status_pendetailan CHAR(17),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_bast) REFERENCES kom_bast_non_kontrak_header(id_bast)
);

-- Tabel: kom_bast_non_kontrak_coa
-- Endpoint: /API/KOM/BASTNonKontrakCOA/KLxxx/KDSATKER/ID_BAST
CREATE TABLE IF NOT EXISTS kom_bast_non_kontrak_coa (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    id_bast NUMERIC(19,0) NOT NULL,
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_akun VARCHAR(6),
    kode_output VARCHAR(3),
    kode_suboutput VARCHAR(3),
    kode_komponen VARCHAR(3),
    kode_subkomponen VARCHAR(2),
    kode_item VARCHAR(6),
    kode_coa VARCHAR(255),
    vol_sub_output NUMERIC,
    nilai_coa_detail NUMERIC(21,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_bast) REFERENCES kom_bast_non_kontrak_header(id_bast)
);

-- Tabel: kom_supplier_header
-- Endpoint: /API/KOM/supplierHeader/KLxxx
CREATE TABLE IF NOT EXISTS kom_supplier_header (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255),
    id_supplier NUMERIC(19,0) UNIQUE NOT NULL,
    nama_supplier VARCHAR(4000),
    npwp VARCHAR(20),
    nrs VARCHAR(255),
    status_data VARCHAR(8),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: kom_supplier_address
-- Endpoint: /API/KOM/supplierAddress/KLxxx/ID_SUPPLIER
CREATE TABLE IF NOT EXISTS kom_supplier_address (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255),
    id_supplier NUMERIC(19,0) NOT NULL,
    id_supplier_address NUMERIC(19,0) NOT NULL,
    nama_site VARCHAR(220),
    kode_kppn VARCHAR(255),
    kode_tipe_supplier VARCHAR(255),
    alamat1 VARCHAR(255),
    alamat2 VARCHAR(255),
    kode_negara VARCHAR(255),
    provinsi VARCHAR(4000),
    kota VARCHAR(4000),
    kecamatan VARCHAR(4000),
    kode_pos VARCHAR(4000),
    no_telp VARCHAR(15),
    no_fax VARCHAR(15),
    email VARCHAR(50),
    status_data VARCHAR(8),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_supplier) REFERENCES kom_supplier_header(id_supplier),
    UNIQUE(id_supplier, id_supplier_address)
);

-- Tabel: kom_supplier_bank
-- Endpoint: /API/KOM/supplierBank/KLxxx/ID_SUPPLIER/ID_SUPPLIER_ADDRESS/NAMA_BANK
CREATE TABLE IF NOT EXISTS kom_supplier_bank (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(255),
    id_supplier NUMERIC(19,0) NOT NULL,
    id_supplier_address NUMERIC(19,0) NOT NULL,
    nama_bank VARCHAR(4000),
    nama_cabang_bank VARCHAR(4000),
    alamat_bank VARCHAR(150),
    nama_pemilik_rekening VARCHAR(100),
    no_rekening VARCHAR(100),
    mata_uang VARCHAR(255),
    detil_nama_cabang_bank VARCHAR(150),
    nama_pegawai_pemda_penerusan_pinjaman VARCHAR(100),
    npwp VARCHAR(15),
    nip VARCHAR(18),
    alamat1 VARCHAR(255),
    alamat2 VARCHAR(255),
    kota VARCHAR(4000),
    propinsi VARCHAR(4000),
    kode_pos VARCHAR(5),
    status_data VARCHAR(8),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_kom_capaian_ro_kdsatker ON kom_capaian_ro(kdsatker);
CREATE INDEX IF NOT EXISTS idx_kom_kontrak_header_kdsatker ON kom_kontrak_header(kdsatker);
CREATE INDEX IF NOT EXISTS idx_kom_kontrak_header_id_kontrak ON kom_kontrak_header(id_kontrak);
CREATE INDEX IF NOT EXISTS idx_kom_kontrak_line_id_kontrak ON kom_kontrak_line(id_kontrak);
CREATE INDEX IF NOT EXISTS idx_kom_kontrak_termin_id_kontrak ON kom_kontrak_termin(id_kontrak);
CREATE INDEX IF NOT EXISTS idx_kom_bast_kontrak_header_id_bast ON kom_bast_kontrak_header(id_bast);
CREATE INDEX IF NOT EXISTS idx_kom_bast_non_kontrak_header_id_bast ON kom_bast_non_kontrak_header(id_bast);
CREATE INDEX IF NOT EXISTS idx_kom_supplier_header_id_supplier ON kom_supplier_header(id_supplier);
CREATE INDEX IF NOT EXISTS idx_kom_supplier_address_id_supplier ON kom_supplier_address(id_supplier);

COMMENT ON TABLE kom_capaian_ro IS 'Data realisasi kinerja satker';
COMMENT ON TABLE kom_kontrak_header IS 'Data kontrak';
COMMENT ON TABLE kom_kontrak_line IS 'Data kontrak per line';
COMMENT ON TABLE kom_kontrak_termin IS 'Data kontrak per termin';
COMMENT ON TABLE kom_kontrak_coa IS 'Rincian 16 segmen coa pada kontrak termin';
COMMENT ON TABLE kom_bast_kontrak_header IS 'Data BAST kontraktual';
COMMENT ON TABLE kom_bast_non_kontrak_header IS 'Data BAST non kontraktual';
COMMENT ON TABLE kom_supplier_header IS 'Data supplier';
