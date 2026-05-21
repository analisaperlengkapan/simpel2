-- Migration: Add Revisi Wilayah workflow statuses
-- Date: 2026-02-12
-- Description: Adds "Return to Wilayah" statuses for Kebutuhan BMN and Pakaian Dinas
--              workflows to allow Validator Pusat to return submissions for corrections
--              at the Wilayah level instead of outright rejecting them.

BEGIN;

-- Add Revisi Wilayah status for Pakaian Dinas (1007)
INSERT INTO perlengkapan.ms_workflow_status (kode, workflow_name, nama, deskripsi, is_terminal, urutan)
VALUES (1007, 'pakaian_dinas', 'Revisi Wilayah', 'Dikembalikan ke Validator Wilayah oleh Validator Pusat untuk perbaikan', false, 7)
ON CONFLICT (kode) DO UPDATE SET
    nama = EXCLUDED.nama,
    deskripsi = EXCLUDED.deskripsi;

-- Add Revisi Wilayah status for Kebutuhan BMN (2010)
INSERT INTO perlengkapan.ms_workflow_status (kode, workflow_name, nama, deskripsi, is_terminal, urutan)
VALUES (2010, 'kebutuhan_bmn', 'Revisi Wilayah', 'Dikembalikan ke Validator Wilayah oleh Validator Pusat untuk perbaikan', false, 11)
ON CONFLICT (kode) DO UPDATE SET
    nama = EXCLUDED.nama,
    deskripsi = EXCLUDED.deskripsi;

COMMIT;
