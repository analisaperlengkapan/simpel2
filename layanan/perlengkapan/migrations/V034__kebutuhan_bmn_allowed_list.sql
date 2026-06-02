-- V029: Allowed-list BMN per Pengajuan (Fase 1.6)
--
-- Sebelumnya Operator Satker dapat input barang apa saja saat usulan
-- kebutuhan BMN. Stakeholder eksplisit (plan §3.1): "Operator Satker
-- HANYA boleh menginput barang dari allowed-list" yg ditetapkan
-- Validator Pusat saat membuat periode.
--
-- Tabel baru `pengajuan_bmn_referensi_diizinkan` menyimpan whitelist
-- per pengajuan (one pengajuan → many allowed BMN). Service validate
-- `kode_barang ∈ allowed-list` di create_barang.

CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_bmn_referensi_diizinkan (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL
        REFERENCES perlengkapan.pengajuan_kebutuhan_bmn(id) ON DELETE CASCADE,
    kode_barang TEXT NOT NULL,
    nama_barang TEXT NOT NULL,
    keterangan TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(pengajuan_id, kode_barang)
);

CREATE INDEX IF NOT EXISTS idx_pengajuan_bmn_referensi_pengajuan
    ON perlengkapan.pengajuan_bmn_referensi_diizinkan(pengajuan_id);

-- Snapshot SIMAN per-satker (pre-fetch saat satker join; freeze saat submit)
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker
    ADD COLUMN IF NOT EXISTS data_eksisting_siman_per_barang JSONB DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS analisis_snapshot_at_submit JSONB;

COMMENT ON TABLE perlengkapan.pengajuan_bmn_referensi_diizinkan IS
    'V029 (Fase 1.6): Daftar kode_barang yg boleh diusulkan operator per pengajuan. Validator Pusat tetapkan saat create periode; backend validate di create_barang.';
COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker.data_eksisting_siman_per_barang IS
    'V029 (Fase 1.6): Pre-fetch SIMAN existing assets per kode_barang dlm allowed-list. Diisi async saat satker join; ditampilkan side-by-side dgn usulan operator (transparansi dari hulu).';
COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker.analisis_snapshot_at_submit IS
    'V029 (Fase 1.6): Snapshot lengkap data analisis (usulan + eksisting + kondisi) di-freeze saat operator submit ke wilayah. Validator Wilayah & Pusat melihat data konsisten.';
