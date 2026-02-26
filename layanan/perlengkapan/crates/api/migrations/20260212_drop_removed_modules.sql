-- Migration: Drop removed modules (Pengadaan, Hibah, Mutasi, Pengalihan, Pemeliharaan)
-- Date: 2026-02-12
-- Description: These modules have been removed from the application as part of the
--              major perlengkapan refactoring. All backend code (models, handlers,
--              services, repository, routes, database DDL/DML) has been deleted.

BEGIN;

-- Drop pengadaan sub-tables first (foreign key dependencies)
DROP TABLE IF EXISTS perlengkapan.pengadaan_nodis CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_bast CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_kontrak CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_ringkasan CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_spk CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_skppbj CASCADE;
DROP TABLE IF EXISTS perlengkapan.pengadaan_hps CASCADE;

-- Drop pengadaan main table
DROP TABLE IF EXISTS perlengkapan.pengadaan CASCADE;

-- Drop hibah table
DROP TABLE IF EXISTS perlengkapan.hibah CASCADE;

-- Drop mutasi table
DROP TABLE IF EXISTS perlengkapan.mutasi CASCADE;

-- Drop pengalihan table
DROP TABLE IF EXISTS perlengkapan.pengalihan CASCADE;

-- Drop pemeliharaan table
DROP TABLE IF EXISTS perlengkapan.pemeliharaan CASCADE;

COMMIT;
