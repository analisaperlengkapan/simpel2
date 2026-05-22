-- ============================================================================
-- Migration: extend pegawai_pakaian_dinas with reporting/filter fields.
--
-- The persistent profile table `pegawai_pakaian_dinas` already stores sizes
-- and `with_hijab`. Legacy simpel_web pulled `eselon`, `jenis_kelamin`,
-- `jenis` (TU/Jaksa) and `mapped_unit_kerja` from a local materialized
-- view — these fields are NOT exposed by the MySIMKARI API, so simpel2
-- keeps them as part of the perlengkapan-owned profile. They feed:
--   * laporan rekap pakaian dinas filters (status pegawai, eselon, gender)
--   * per-gender / per-variant columns in the rekap table
--   * grouping by unit kerja in the pengajuan wizard
--
-- Values are populated by satker operators during the pakaian-dinas
-- wizard (and from approved pengajuan_pakaian_dinas_satker_pegawai rows),
-- not from MySIMKARI sync.
-- ============================================================================

ALTER TABLE pegawai_pakaian_dinas
    ADD COLUMN IF NOT EXISTS eselon            VARCHAR(20),
    ADD COLUMN IF NOT EXISTS jenis_kelamin     VARCHAR(1)
        CHECK (jenis_kelamin IS NULL OR jenis_kelamin IN ('L', 'P')),
    ADD COLUMN IF NOT EXISTS jenis_pegawai     VARCHAR(32),
    ADD COLUMN IF NOT EXISTS mapped_unit_kerja TEXT,
    ADD COLUMN IF NOT EXISTS kode_satker       VARCHAR(32);

CREATE INDEX IF NOT EXISTS idx_pegawai_pakaian_dinas_kode_satker
    ON pegawai_pakaian_dinas(kode_satker)
    WHERE kode_satker IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_pegawai_pakaian_dinas_eselon
    ON pegawai_pakaian_dinas(eselon)
    WHERE eselon IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_pegawai_pakaian_dinas_jenis_pegawai
    ON pegawai_pakaian_dinas(jenis_pegawai)
    WHERE jenis_pegawai IS NOT NULL;

COMMENT ON COLUMN pegawai_pakaian_dinas.eselon IS
    'Eselon I/II/III/IV or "Non-eselon" — filter for laporan rekap, not sourced from MySIMKARI.';
COMMENT ON COLUMN pegawai_pakaian_dinas.jenis_kelamin IS
    'L/P — drives gender variant of pakaian; not reliably present in MySIMKARI responses.';
COMMENT ON COLUMN pegawai_pakaian_dinas.jenis_pegawai IS
    'Klasifikasi pegawai ("TU" / "Jaksa" / …) — perlengkapan-owned filter.';
COMMENT ON COLUMN pegawai_pakaian_dinas.mapped_unit_kerja IS
    'Normalised unit kerja string used for grouping in laporan output.';
