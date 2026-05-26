-- V030: File upload untuk Usulan SK Penghapusan BMN (Fase 0.6)
--
-- Sebelumnya `lampiran_persyaratan` di `perlengkapan.penghapusan_bmn`
-- hanya menyimpan URL string — operator harus upload file di luar sistem
-- lalu paste URL, sehingga jejak hilang & tidak ada validasi.
--
-- V030 menambahkan:
-- 1. Kolom `surat_usulan_file_url` di tabel utama untuk URL Surat Usulan
--    (1 file wajib).
-- 2. Tabel `penghapusan_bmn_lampiran` untuk Lampiran[] (multi-file
--    opsional, beragam tipe). Setiap baris menyimpan satu file dgn
--    metadata.
--
-- Backward compat: kolom legacy `lampiran_persyaratan` & `lampiran_pendukung`
-- TIDAK di-drop — tetap dipertahankan utk data lama.

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS surat_usulan_file_url TEXT,
    ADD COLUMN IF NOT EXISTS surat_usulan_uploaded_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS perlengkapan.penghapusan_bmn_lampiran (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    penghapusan_id UUID NOT NULL REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE,
    nama TEXT NOT NULL,
    file_url TEXT NOT NULL,
    content_type TEXT,
    size_bytes BIGINT,
    uploaded_by UUID,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_lampiran_penghapusan_id
    ON perlengkapan.penghapusan_bmn_lampiran(penghapusan_id);

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.surat_usulan_file_url IS
    'URL Surat Usulan (1 file wajib) yg di-upload via /penghapusan-bmn/{id}/lampiran.';
COMMENT ON TABLE perlengkapan.penghapusan_bmn_lampiran IS
    'Lampiran pendukung Usulan SK Penghapusan BMN (multi-file, opsional). File disimpan via DocumentStorage; baris ini hanya metadata.';
