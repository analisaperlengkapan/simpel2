-- V029: Scope satker = wilayah untuk Pengajuan Kebutuhan BMN (Fase 1.7)
--
-- Sebelumnya `pilihan_satker` hanya menerima 'semua' atau 'sebagian'.
-- Stakeholder minta opsi ketiga: 'wilayah' — Validator Pusat pilih satu
-- wilayah Kejaksaan Tinggi → sistem otomatis melibatkan semua satker
-- di wilayah tsb (resolusi via `integrasi.mysimkari_satker.wilayah`).
--
-- Backward compat: kolom legacy `pilihan_satker` TIDAK di-drop. Kolom
-- baru `scope_satker` di-backfill dari `pilihan_satker` (1:1). Aplikasi
-- baca dari `scope_satker`; INSERT baru mengisi keduanya selama legacy
-- consumers masih ada (drop legacy = follow-up).

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ADD COLUMN IF NOT EXISTS scope_satker TEXT,
    ADD COLUMN IF NOT EXISTS wilayah_id TEXT;

-- Backfill scope_satker dari pilihan_satker (semua/sebagian → sama).
UPDATE perlengkapan.pengajuan_kebutuhan_bmn
SET scope_satker = pilihan_satker
WHERE scope_satker IS NULL
  AND pilihan_satker IS NOT NULL;

-- Default 'semua' untuk row yg keduanya null (defensive).
UPDATE perlengkapan.pengajuan_kebutuhan_bmn
SET scope_satker = 'semua'
WHERE scope_satker IS NULL;

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ALTER COLUMN scope_satker SET NOT NULL,
    ALTER COLUMN scope_satker SET DEFAULT 'semua';

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ADD CONSTRAINT chk_pengajuan_kebutuhan_bmn_scope_satker
    CHECK (scope_satker IN ('semua', 'sebagian', 'wilayah'));

-- Jika scope=wilayah, wilayah_id wajib (consistency check di aplikasi
-- tetapi gating ada di sini juga sbg defense-in-depth).
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ADD CONSTRAINT chk_pengajuan_kebutuhan_bmn_wilayah_required
    CHECK (scope_satker != 'wilayah' OR wilayah_id IS NOT NULL);

COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn.scope_satker IS
    'Cakupan satker: semua | sebagian | wilayah (V029). Menggantikan pilihan_satker (legacy, dipertahankan utk backward compat).';
COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn.wilayah_id IS
    'Nama wilayah Kejaksaan Tinggi (text label, match integrasi.mysimkari_satker.wilayah). WAJIB jika scope_satker = wilayah. Resolver mengisi pengajuan_kebutuhan_bmn_satker_terpilih dgn semua kode_satker di wilayah ini.';
