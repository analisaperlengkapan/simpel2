-- Add api_id column to all MonSAKTI tables
-- This stores the ID from external API while keeping auto-increment id for internal use

SET search_path TO integrasi, public;

-- ADM Module
ALTER TABLE adm_ref_admin ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE adm_pejabat ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE adm_ref_jns_spp ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE adm_ref_jns_dokumen ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE adm_ref_jns_blokir ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE adm_ref_uraian_akun ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- ANG Module
ALTER TABLE ang_anggaran ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ang_kewenangan ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ang_revisi ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ang_dana_blokir ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- PEM Module
ALTER TABLE pem_pembayaran ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE pem_spm ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE pem_sp2d ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- BEN Module
ALTER TABLE ben_kas_tunai ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ben_kas_bank ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ben_spby ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- KOM Module
ALTER TABLE kom_pengembalian ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE kom_tagihan ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- AST Module
ALTER TABLE ast_aset ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE ast_mutasi ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- PER Module
ALTER TABLE per_persediaan ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE per_mutasi ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- GLP Module
ALTER TABLE glp_realisasi ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);
ALTER TABLE glp_blokir ADD COLUMN IF NOT EXISTS api_id NUMERIC(19,0);

-- Create indexes on api_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_adm_ref_admin_api_id ON adm_ref_admin(api_id);
CREATE INDEX IF NOT EXISTS idx_adm_pejabat_api_id ON adm_pejabat(api_id);
CREATE INDEX IF NOT EXISTS idx_ang_anggaran_api_id ON ang_anggaran(api_id);
CREATE INDEX IF NOT EXISTS idx_pem_pembayaran_api_id ON pem_pembayaran(api_id);
CREATE INDEX IF NOT EXISTS idx_ben_kas_tunai_api_id ON ben_kas_tunai(api_id);
CREATE INDEX IF NOT EXISTS idx_ben_kas_bank_api_id ON ben_kas_bank(api_id);
CREATE INDEX IF NOT EXISTS idx_kom_pengembalian_api_id ON kom_pengembalian(api_id);
CREATE INDEX IF NOT EXISTS idx_ast_aset_api_id ON ast_aset(api_id);
CREATE INDEX IF NOT EXISTS idx_per_persediaan_api_id ON per_persediaan(api_id);
CREATE INDEX IF NOT EXISTS idx_glp_realisasi_api_id ON glp_realisasi(api_id);
