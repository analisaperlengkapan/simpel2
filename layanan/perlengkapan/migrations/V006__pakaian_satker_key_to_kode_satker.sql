-- ============================================================================
-- Migration V006: pakaian-dinas satker references become MySIMKARI kode_satker
-- ============================================================================
-- #94. `pengajuan_pakaian_dinas_satker.satker_id` and
-- `pengajuan_pakaian_dinas_satker_terpilih.satker_id` are `uuid`, but the
-- satker they point at lives in `integrasi.mysimkari_satker`, whose primary key
-- is `BIGSERIAL` (bigint). So every join the code writes —
--   ON ps.satker_id = s.id          (repository/satker.rs x2, laporan.rs x3)
--   JOIN ... ms ON ms.id = pt.satker_id   (scope.rs x2)
-- is `uuid = bigint`, which Postgres rejects outright. The satker workflow
-- (list / forward / rekap), the wilayah campaign resolver and the RBAC scoping
-- subquery therefore fail at runtime in EVERY environment. This was never
-- caught because the pakaian satker workflow had no e2e (F-E2E E-2 descoped it
-- precisely because of this bug — see tests/fixtures/e2e/seed-perlengkapan-workflow.sql).
--
-- WHY kode_satker AND NOT bigint. Making these columns `bigint` would compile
-- and would look like the smaller change, but it is the WRONG key:
--   * `mysimkari_satker.id` is a BIGSERIAL surrogate assigned at sync time. A
--     re-sync that truncates and reloads (which is how the integrasi sync
--     works) reassigns those ids, silently repointing every stored reference at
--     a DIFFERENT satker. That is worse than the current loud failure.
--   * `kode_satker` is `TEXT NOT NULL UNIQUE` — the stable Kejaksaan business
--     key, and the SoT column every other consumer already uses.
-- The repo convention agrees and pakaian-dinas is the sole outlier:
--   * `perlengkapan.pengajuan_kebutuhan_bmn_satker.satker_id` = varchar(20)
--     holding kode_satker (V001 baseline);
--   * `izin_pemakaian_bmn` / `penghapusan_bmn` carry `satker_code` (V003);
--   * even integrasi's own child table `mysimkari_pegawai.satker_id` is TEXT
--     holding kode_satker, not the bigint surrogate.
-- Aligning here also lets `shared::satker_scope::SatkerScope` compare directly
-- against the column instead of joining through the surrogate.
--
-- NO DATA TO CONVERT — this is a plain type change, not a backfill. Both tables
-- are empty in every environment: `V002__seed.sql` ships zero rows for them
-- (the pg_dump sections are blank), the e2e fixture explicitly seeds none, and
-- no code path could ever have inserted a row because the wilayah resolver
-- itself errors first (`SELECT id ... ` read as Uuid against a bigint column).
-- So USING is a formality that can never see an existing value.
--
-- EXPAND/CONTRACT (#52): normally a type change would be split add-new-column →
-- backfill → drop-old. That ceremony exists to keep the OLD reader working
-- during a rollback window. Here there is no working old reader to preserve —
-- every query against these columns is invalid SQL — and no rows to lose, so a
-- rollback has nothing to roll back to. Doing it in one step is both safe and
-- honest about the state.
--
-- WHY A NEW MIGRATION RATHER THAN EDITING THE BASELINE: V001__baseline.sql is
-- already applied and refinery validates checksums of applied migrations;
-- editing it in place would make every deployed environment fail to start.
-- ============================================================================

-- The PK includes satker_id, so it is dropped and rebuilt around the new type.
ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
    DROP CONSTRAINT IF EXISTS pengajuan_pakaian_dinas_satker_terpilih_pkey;

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
    ALTER COLUMN satker_id TYPE character varying(20) USING satker_id::text,
    ALTER COLUMN satker_pusat_id TYPE character varying(20) USING satker_pusat_id::text;

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_terpilih
    ADD CONSTRAINT pengajuan_pakaian_dinas_satker_terpilih_pkey
    PRIMARY KEY (pengajuan_id, satker_id);

-- idx_ppd_satker_id_status (satker_id, aktivitas_id) is rebuilt automatically by
-- ALTER COLUMN TYPE; no explicit DROP/CREATE needed.
ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker
    ALTER COLUMN satker_id TYPE character varying(20) USING satker_id::text,
    ALTER COLUMN satker_pusat_id TYPE character varying(20) USING satker_pusat_id::text;

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas_satker.satker_id IS
    'MySIMKARI kode_satker (SoT: integrasi.mysimkari_satker.kode_satker). NOT the bigint surrogate id — that is reassigned on re-sync. #94';
COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas_satker_terpilih.satker_id IS
    'MySIMKARI kode_satker (SoT: integrasi.mysimkari_satker.kode_satker). #94';

-- ---------------------------------------------------------------------------
-- Drop three dead uuid satker columns while we are correcting this table's
-- satker typing. They are the same "satker reference typed as uuid" mistake and
-- are provably unused: `grep -rn 'id_kejati\|id_kejari\|id_cabjari'` over the
-- whole repo finds ONLY this DDL plus a `row.try_get(...)` in
-- models/entities.rs. There is no INSERT, no UPDATE, no WHERE, no index — they
-- have never held a value in any environment. Leaving them would keep a
-- uuid-shaped satker reference in the table for the next reader to copy.
-- ---------------------------------------------------------------------------
ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker
    DROP COLUMN IF EXISTS id_kejati,
    DROP COLUMN IF EXISTS id_kejari,
    DROP COLUMN IF EXISTS id_cabjari;
