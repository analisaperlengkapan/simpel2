-- V029: Rename semantik kolom Penghapusan BMN
--
-- Field `nilai_residu` di tabel `perlengkapan.penghapusan_bmn` keliru secara
-- semantik — yang dibutuhkan adalah `nilai_perolehan` (harga pembelian/
-- perolehan aset), bukan nilai sisa setelah depresiasi.
--
-- Backward compat: kolom lama `nilai_residu` TIDAK di-drop di migrasi ini —
-- dipertahankan selama minimal 1 siklus fiskal sebelum di-deprecate, agar
-- audit lintas-versi tetap dapat menelusuri data lama. Backfill memuat
-- `nilai_perolehan` dari `nilai_residu` untuk row yang sudah ada; flag
-- `nilai_perolehan_dari_backfill` menandai bahwa nilai tsb perlu diverifikasi
-- user karena semantik aslinya berbeda.

ALTER TABLE perlengkapan.penghapusan_bmn
    ADD COLUMN IF NOT EXISTS nilai_perolehan DECIMAL(15, 2),
    ADD COLUMN IF NOT EXISTS nilai_perolehan_dari_backfill BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE perlengkapan.penghapusan_bmn
SET nilai_perolehan = nilai_residu,
    nilai_perolehan_dari_backfill = TRUE
WHERE nilai_perolehan IS NULL
  AND nilai_residu IS NOT NULL;

COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_perolehan IS
    'Nilai perolehan aset (harga pembelian). Menggantikan nilai_residu yang keliru secara semantik.';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_perolehan_dari_backfill IS
    'TRUE jika nilai_perolehan diisi dari backfill kolom legacy nilai_residu — perlu diverifikasi user.';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.nilai_residu IS
    'DEPRECATED (V029). Gunakan nilai_perolehan. Dipertahankan utk audit data lama; akan di-drop setelah verifikasi.';
