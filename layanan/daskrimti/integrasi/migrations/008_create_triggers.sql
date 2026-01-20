-- ==================== TRIGGERS & FUNCTIONS ====================
-- Auto-update timestamp untuk semua tabel
-- File: 008_create_triggers.sql

SET search_path TO integrasi, public;

-- Function untuk update timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- ==================== Triggers untuk Modul ADM ====================
CREATE TRIGGER update_adm_ref_admin_updated_at
    BEFORE UPDATE ON adm_ref_admin
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_adm_pejabat_updated_at
    BEFORE UPDATE ON adm_pejabat
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_adm_ref_jns_spp_updated_at
    BEFORE UPDATE ON adm_ref_jns_spp
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul ANG ====================
CREATE TRIGGER update_ang_data_ang_updated_at
    BEFORE UPDATE ON ang_data_ang
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ang_ref_sts_updated_at
    BEFORE UPDATE ON ang_ref_sts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul PEM ====================
CREATE TRIGGER update_pem_realisasi_updated_at
    BEFORE UPDATE ON pem_realisasi
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pem_spp_header_updated_at
    BEFORE UPDATE ON pem_spp_header
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pem_spp_pengeluaran_updated_at
    BEFORE UPDATE ON pem_spp_pengeluaran
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pem_spp_potongan_updated_at
    BEFORE UPDATE ON pem_spp_potongan
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pem_penerima_spm_updated_at
    BEFORE UPDATE ON pem_penerima_spm
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul BEN ====================
CREATE TRIGGER update_ben_kas_tunai_updated_at
    BEFORE UPDATE ON ben_kas_tunai
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_kas_bank_updated_at
    BEFORE UPDATE ON ben_kas_bank
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_spby_updated_at
    BEFORE UPDATE ON ben_spby
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_kuitansi_updated_at
    BEFORE UPDATE ON ben_kuitansi
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_drpp_updated_at
    BEFORE UPDATE ON ben_drpp
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_pungut_pajak_updated_at
    BEFORE UPDATE ON ben_pungut_pajak
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_setor_pajak_updated_at
    BEFORE UPDATE ON ben_setor_pajak
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ben_pnbp_updated_at
    BEFORE UPDATE ON ben_pnbp
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul KOM ====================
CREATE TRIGGER update_kom_capaian_ro_updated_at
    BEFORE UPDATE ON kom_capaian_ro
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_kontrak_header_updated_at
    BEFORE UPDATE ON kom_kontrak_header
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_kontrak_line_updated_at
    BEFORE UPDATE ON kom_kontrak_line
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_kontrak_termin_updated_at
    BEFORE UPDATE ON kom_kontrak_termin
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_kontrak_coa_updated_at
    BEFORE UPDATE ON kom_kontrak_coa
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_kontrak_header_updated_at
    BEFORE UPDATE ON kom_bast_kontrak_header
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_kontrak_detail_barang_updated_at
    BEFORE UPDATE ON kom_bast_kontrak_detail_barang
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_kontrak_coa_updated_at
    BEFORE UPDATE ON kom_bast_kontrak_coa
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_non_kontrak_header_updated_at
    BEFORE UPDATE ON kom_bast_non_kontrak_header
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_non_kontrak_detail_barang_updated_at
    BEFORE UPDATE ON kom_bast_non_kontrak_detail_barang
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_bast_non_kontrak_coa_updated_at
    BEFORE UPDATE ON kom_bast_non_kontrak_coa
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_supplier_header_updated_at
    BEFORE UPDATE ON kom_supplier_header
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_supplier_address_updated_at
    BEFORE UPDATE ON kom_supplier_address
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_kom_supplier_bank_updated_at
    BEFORE UPDATE ON kom_supplier_bank
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul AST ====================
CREATE TRIGGER update_ast_aset_trx_updated_at
    BEFORE UPDATE ON ast_aset_trx
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul PER ====================
CREATE TRIGGER update_per_persedia_trx_updated_at
    BEFORE UPDATE ON per_persedia_trx
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul GLP ====================
CREATE TRIGGER update_glp_buku_besar_updated_at
    BEFORE UPDATE ON glp_buku_besar
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_glp_neraca_sawal_updated_at
    BEFORE UPDATE ON glp_neraca_sawal
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_glp_fa_detail_updated_at
    BEFORE UPDATE ON glp_fa_detail
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== Triggers untuk Modul MySIMKARI ====================
CREATE TRIGGER update_mysimkari_satker_updated_at
    BEFORE UPDATE ON mysimkari_satker
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_mysimkari_pegawai_updated_at
    BEFORE UPDATE ON mysimkari_pegawai
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==================== SELESAI ====================
SELECT 'All triggers have been created successfully.' AS status;
