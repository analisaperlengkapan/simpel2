-- Migration: Change api_id from NUMERIC(19,0) to BIGINT for better tokio_postgres compatibility
-- NUMERIC(19,0) causes serialization issues with Option<i64>::None in Rust
-- BIGINT is native PostgreSQL 64-bit integer with direct Option<i64> support

-- ADM module tables
ALTER TABLE integrasi.adm_ref_admin ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.adm_pejabat ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.adm_satker ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- ANG module tables
ALTER TABLE integrasi.ang_anggaran ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ang_pagu ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ang_realisasi ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- PEM module tables
ALTER TABLE integrasi.pem_pegawai ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.pem_jabatan ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.pem_mutasi ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- BEN module tables
ALTER TABLE integrasi.ben_kas_tunai ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_kas_bank ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_spby ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_kuitansi ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_drpp ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_setor_pajak ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_pnbp ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_tup ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ben_pengembalian ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- KOM module tables
ALTER TABLE integrasi.kom_kontrak ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.kom_addendum ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.kom_jaminan ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- AST module tables
ALTER TABLE integrasi.ast_aset ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ast_pemeliharaan ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.ast_pemanfaatan ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- PER module tables
ALTER TABLE integrasi.per_persediaan ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.per_distribusi ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.per_stock_opname ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;

-- GLP module tables (Gaji, Lembur, Perjalanan Dinas)
ALTER TABLE integrasi.glp_gaji ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.glp_lembur ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
ALTER TABLE integrasi.glp_perjalanan_dinas ALTER COLUMN api_id TYPE BIGINT USING api_id::bigint;
