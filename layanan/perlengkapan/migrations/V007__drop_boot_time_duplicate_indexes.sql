-- Contract step for the removal of the boot-time `add_essential_indexes` step.
--
-- That function ran 19 DDL statements on every start of layanan-perlengkapan.
-- Measured against this baseline on a clean PostgreSQL 15:
--
--   * 10 statements ERRORED — they targeted `perlengkapan.kebutuhan_bmn` and
--     `perlengkapan.pakaian_dinas` (tables no migration has ever created; see
--     #118) or columns that do not exist (`..._satker_aktivitas.pengajuan_id`,
--     the real column is `pengajuan_satker_id`; `izin_pemakaian_bmn.bmn_id`).
--     The loop only `warn!`ed, so this was invisible in every environment.
--
--   * 9 "succeeded" but added nothing of value. `CREATE INDEX IF NOT EXISTS`
--     matches on the index NAME, not its definition, so a differently-named
--     copy of an index the migrations already create looks like a success.
--     Three were exact-name no-ops; the five dropped below were real duplicate
--     indexes under new names.
--
-- Each DROP is paired with the migration-created index that already covers it.
-- Every one is IF EXISTS, so this is a no-op on a database that never ran the
-- boot-time step (including any fresh install) and idempotent on one that did.

-- (pegawai_nip) — duplicate of idx_izin_pegawai_nip.
DROP INDEX IF EXISTS perlengkapan.idx_pemakaian_bmn_pegawai;

-- (status) — duplicate of idx_izin_status.
DROP INDEX IF EXISTS perlengkapan.idx_pemakaian_bmn_status;

-- (tanggal_mulai, tanggal_selesai) — not a byte-identical duplicate, but its
-- leading column is served by idx_izin_tanggal_mulai and no query in the
-- service filters on the pair. Dropped rather than kept "just in case": an
-- index with no query behind it is pure write amplification. If a range query
-- over both columns appears later, it comes back through a migration with the
-- query that justifies it.
DROP INDEX IF EXISTS perlengkapan.idx_pemakaian_bmn_tanggal;

-- (created_at) — duplicate of idx_pkb_aktivitas_created (created_at DESC);
-- btree indexes scan in both directions, so the sort order does not make this
-- a distinct index.
DROP INDEX IF EXISTS perlengkapan.idx_workflow_aktivitas_created;

-- (user_id) — the only genuinely new index the boot step ever created, and
-- nothing queries `pengajuan_kebutuhan_bmn_satker_aktivitas.user_id`
-- (verified by grep across layanan/perlengkapan/src/).
DROP INDEX IF EXISTS perlengkapan.idx_workflow_aktivitas_user;
