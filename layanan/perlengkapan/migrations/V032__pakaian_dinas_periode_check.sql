-- V029: CHECK constraint periode pakaian dinas (Fase 1.8)
--
-- Field `tgl_mulai`/`tgl_selesai` di `perlengkapan.pengajuan_pakaian_dinas`
-- sudah ada sejak V002, tapi tidak ada constraint yg memastikan
-- `tgl_selesai >= tgl_mulai`. Service layer baru menegakkan validasi
-- ini untuk submit baru (Fase 1.8), tapi defense-in-depth di DB.
--
-- Untuk data lama yg pernah ter-insert dgn periode terbalik (jika ada),
-- migration ini akan FAIL — itu disengaja: data tidak konsisten harus
-- diperbaiki manual sebelum migration berjalan.
--
-- Tidak menegakkan NOT NULL — data lama (sebelum periode jadi required
-- di service) boleh punya NULL; aplikasi treat sbg "periode tidak
-- terstruktur" dan jatuh balik ke `tahun` saat menampilkan.

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
    ADD CONSTRAINT chk_pengajuan_pakaian_dinas_periode_order
    CHECK (tgl_mulai IS NULL OR tgl_selesai IS NULL OR tgl_mulai <= tgl_selesai);

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.tgl_mulai IS
    'Periode pengajuan: tanggal mulai. Wajib utk pengajuan baru (V029, Fase 1.8); legacy row boleh NULL.';
COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.tgl_selesai IS
    'Periode pengajuan: tanggal selesai (≥ tgl_mulai, enforced via CHECK constraint).';
