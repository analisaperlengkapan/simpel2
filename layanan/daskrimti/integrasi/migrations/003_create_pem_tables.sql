-- ==================== MODUL PEMBAYARAN (PEM) ====================
-- Struktur data sesuai dokumentasi API MonSAKTI v1.4

SET search_path TO integrasi, public;

-- Tabel: pem_realisasi
-- Endpoint: /API/PEM/realisasi/KLxxx/KDSATKER/KD_JNS_SPP/NO_SPP
CREATE TABLE IF NOT EXISTS pem_realisasi (
    id BIGSERIAL PRIMARY KEY,
    kdsatker VARCHAR(6),
    kode_kementerian VARCHAR(3),
    kd_jns_spp VARCHAR(3),
    no_spp VARCHAR(6),
    tgl_spp DATE,
    no_spm VARCHAR(50),
    tgl_spm DATE,
    no_sp2d VARCHAR(50),
    tgl_sp2d DATE,
    uraian VARCHAR(255),
    kode_coa VARCHAR(100),
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(3),
    kode_suboutput VARCHAR(5),
    kode_komponen VARCHAR(5),
    kode_subkomponen VARCHAR(5),
    kode_akun VARCHAR(6),
    kode_item VARCHAR(22),
    mata_uang VARCHAR(3),
    kurs NUMERIC(19,4),
    nilai_valas NUMERIC(21,2),
    nilai_rupiah NUMERIC(21,2),
    status_data VARCHAR(4000),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: pem_spp_header
-- Endpoint: /API/PEM/sppHeader/KLxxx/KDSATKER/KD_JNS_SPP/NO_SPP
CREATE TABLE IF NOT EXISTS pem_spp_header (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    kd_kppn VARCHAR(3),
    thn_ang VARCHAR(4),
    id_spp NUMERIC(19,0) UNIQUE NOT NULL,
    id_supplier NUMERIC(19,0),
    id_bast VARCHAR(1000),
    sts_data VARCHAR(50),
    kd_jns_spp VARCHAR(3),
    no_spp VARCHAR(6),
    no_spp2 VARCHAR(15),
    tgl_spp DATE,
    id_spp_yg_dikoreksi NUMERIC,
    jns_spp_koreksi VARCHAR(255),
    tgl_spp_koreksi TIMESTAMP,
    tgl_spm_koreksi TIMESTAMP,
    tgl_sp2d_koreksi DATE,
    koreksi_flag NUMERIC(1,0),
    no_spm VARCHAR(50),
    tgl_spm DATE,
    tgl_adk_spm DATE,
    nilai_spm NUMERIC(19,0),
    no_sp2d VARCHAR(50),
    tgl_sp2d DATE,
    nilai_sp2d NUMERIC(19,2),
    no_sp2b VARCHAR(255),
    tgl_sp2b TIMESTAMP,
    nilai_sp2b NUMERIC(19,2),
    no_sp3hl_bjs VARCHAR(50),
    tgl_sp3hl_bjs DATE,
    no_gaji VARCHAR(900),
    bulan_gaji NUMERIC(10,0),
    no_reksus VARCHAR(50),
    id_jadwal_byr_kontrak NUMERIC(19,0),
    id_kontrak NUMERIC(19,0),
    no_kontrak VARCHAR(255),
    nilai_kontrak_pdn NUMERIC(19,2),
    nilai_kontrak_pdp NUMERIC(19,2),
    nilai_kontrak_pln NUMERIC(19,2),
    no_aplikasi VARCHAR(50),
    tgl_aplikasi DATE,
    nilai_aplikasi NUMERIC(19,2),
    no_register VARCHAR(50),
    tgl_register DATE,
    no_pengesahan VARCHAR(50),
    tgl_pengesahan DATE,
    jml_pengeluaran NUMERIC(19,2),
    jml_potongan NUMERIC(19,2),
    jml_pembayaran NUMERIC(19,2),
    kd_valas VARCHAR(3),
    tipe_kurs VARCHAR(1),
    tgl_kurs DATE,
    nilai_tukar NUMERIC(19,2),
    nilai_tukar_sp2d NUMERIC(19,4),
    nip_ppk VARCHAR(255),
    nama_ppk VARCHAR(255),
    nip_ppspm VARCHAR(255),
    nama_ppspm VARCHAR(255),
    uraian VARCHAR(255),
    npwp2 VARCHAR(50),
    kode_sumber_dana VARCHAR(1),
    no_nod VARCHAR(255),
    amount_nod NUMERIC(19,2),
    kurs_nod NUMERIC(19,2),
    tgl_nod TIMESTAMP,
    no_wa VARCHAR(255),
    tgl_wa TIMESTAMP,
    pembayaran_pendamping NUMERIC(19,2),
    porsi_setor_saat_ini NUMERIC(19,2),
    porsi_tdk_setor_saat_ini NUMERIC(19,2),
    status_apd VARCHAR(255),
    nilai_kontrak_apd NUMERIC(19,2),
    periode_triwulan VARCHAR(3),
    saldo_awal NUMERIC(19,2),
    belanja NUMERIC(19,2),
    pendapatan NUMERIC(19,2),
    saldo_akhir NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tabel: pem_spp_pengeluaran
-- Endpoint: /API/PEM/sppPengeluaran/KLxxx/ID_SPP
CREATE TABLE IF NOT EXISTS pem_spp_pengeluaran (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    id_spp NUMERIC(19,0) NOT NULL,
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(3),
    kode_akun VARCHAR(6),
    kode_suboutput VARCHAR(5),
    kode_komponen VARCHAR(5),
    kode_subkomponen VARCHAR(5),
    kode_item VARCHAR(22),
    kd_ctarik VARCHAR(1),
    kd_register VARCHAR(8),
    kode_coa VARCHAR(255),
    kode_valas VARCHAR(255),
    nilai_akun_pengeluaran NUMERIC,
    nilai_tukar NUMERIC(19,2),
    nilai_tukar_sp2d NUMERIC(19,4),
    tgl_kur_sp2d DATE,
    nilai_valas NUMERIC,
    nilai_pembayaran_valas_sp2d NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_spp) REFERENCES pem_spp_header(id_spp)
);

-- Tabel: pem_spp_potongan
-- Endpoint: /API/PEM/sppPotongan/KLxxx/ID_SPP
CREATE TABLE IF NOT EXISTS pem_spp_potongan (
    id BIGSERIAL PRIMARY KEY,
    kode_kementerian VARCHAR(3),
    kdsatker VARCHAR(6),
    id_spp NUMERIC(19,0) NOT NULL,
    kode_program VARCHAR(2),
    kode_kegiatan VARCHAR(4),
    kode_output VARCHAR(3),
    kode_akun VARCHAR(6),
    kode_suboutput VARCHAR(5),
    kode_komponen VARCHAR(5),
    kode_subkomponen VARCHAR(5),
    kode_item VARCHAR(22),
    kd_ctarik VARCHAR(1),
    kd_register VARCHAR(8),
    kode_coa VARCHAR(255),
    kode_valas VARCHAR(255),
    nilai_akun_pot NUMERIC,
    nilai_tukar NUMERIC(19,2),
    nilai_tukar_sp2d NUMERIC(19,4),
    nilai_valas NUMERIC,
    nilai_pembayaran_valas_sp2d NUMERIC(19,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_spp) REFERENCES pem_spp_header(id_spp)
);

-- Tabel: pem_penerima_spm
-- Endpoint: /API/PEM/penerimaSPM/KLxxx/ID_SPP
CREATE TABLE IF NOT EXISTS pem_penerima_spm (
    id BIGSERIAL PRIMARY KEY,
    kdsatker VARCHAR(6),
    kode_kementerian VARCHAR(3),
    id_spp NUMERIC(19,0) NOT NULL,
    id_supplier NUMERIC(19,0),
    id_supplier_address NUMERIC(19,0),
    id_supplier_bank NUMERIC(19,0),
    kode_tipe_supplier VARCHAR(255),
    nama_site VARCHAR(255),
    nama VARCHAR(255),
    alamat VARCHAR(255),
    nama_bank VARCHAR(255),
    nama_pegawai VARCHAR(255),
    nilai NUMERIC(19,2),
    npwp VARCHAR(255),
    nrs VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (id_spp) REFERENCES pem_spp_header(id_spp)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_pem_realisasi_kdsatker ON pem_realisasi(kdsatker);
CREATE INDEX IF NOT EXISTS idx_pem_realisasi_kd_jns_spp ON pem_realisasi(kd_jns_spp);
CREATE INDEX IF NOT EXISTS idx_pem_spp_header_kdsatker ON pem_spp_header(kdsatker);
CREATE INDEX IF NOT EXISTS idx_pem_spp_header_id_spp ON pem_spp_header(id_spp);
CREATE INDEX IF NOT EXISTS idx_pem_spp_pengeluaran_id_spp ON pem_spp_pengeluaran(id_spp);
CREATE INDEX IF NOT EXISTS idx_pem_spp_potongan_id_spp ON pem_spp_potongan(id_spp);
CREATE INDEX IF NOT EXISTS idx_pem_penerima_spm_id_spp ON pem_penerima_spm(id_spp);

COMMENT ON TABLE pem_realisasi IS 'Data realisasi SPP dengan 16 segmen kode COA';
COMMENT ON TABLE pem_spp_header IS 'Data realisasi SPP, SPM dan SP2D (1 row per 1 SPP)';
COMMENT ON TABLE pem_spp_pengeluaran IS 'Distribusi COA 16 segmen untuk nilai pengeluaran SPP';
COMMENT ON TABLE pem_spp_potongan IS 'Distribusi COA 16 segmen untuk nilai potongan SPP';
COMMENT ON TABLE pem_penerima_spm IS 'Data penerima/supplier dari SPP';
COMMENT ON COLUMN pem_spp_header.id_spp IS 'Primary Key untuk join dengan child tables';
