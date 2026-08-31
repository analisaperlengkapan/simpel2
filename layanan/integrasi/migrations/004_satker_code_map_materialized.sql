-- ============================================================================
-- Migration: materialize the auto satker code-map (task #43 follow-up)
-- Description: `v_satker_code_map_auto` derives the MySIMKARI↔SIMAN mapping by
--   name-matching, and its first step is
--       SELECT DISTINCT kdsatker_keu, nama_satker, satker_nama_norm(nama_satker)
--       FROM integrasi.siman_aset
--   — a full pass over EVERY asset row, calling a function per row, to produce a
--   result of a few hundred satkers. `v_satker_code_map` sits on top of it, and
--   `AsetScope::push_condition` (perlengkapan bank_aset) queries that view on
--   every satker- and wilayah-scoped request. So each such request re-derived the
--   whole national mapping before it could filter to one satker.
-- Author: SIMPEL Team
-- Created: 2026-08-25
--
-- MEASURED on staging (624 533 assets, 543 satkers, 560 mapping rows):
--   scoped COUNT(*) via the plain view ....... 5 518 ms  (4 936 ms in that DISTINCT)
--   scoped COUNT(*) via the matview .......... 0.5 ms
--   end-to-end GET /bank-aset for an operator .. 11.3 s → sub-second
-- The pathology was inverted-looking and that is why it survived review: the
-- NATIONAL query (`AsetScope::All`) took ~1 s because it adds no predicate at
-- all, while an operator reading only their own ~1 700 assets took 11 s. The
-- narrower the scope, the slower the query — so the most common request in the
-- application was also its slowest, and it degrades linearly with SIMAN size.
--
-- Fix: materialize ONLY the expensive auto-derive branch and index it. The
-- verified-override branch (`satker_code_map`, a real table) stays live, so a
-- manual correction still takes effect immediately with no refresh.
--
-- FRESHNESS — read before changing: a matview is a snapshot, and a STALE one
-- here does not merely serve old data, it makes satker-scoped users FAIL CLOSED
-- to zero rows (an unmapped satker sees nothing, by design — see scope.rs). A
-- newly synced satker is therefore invisible until a refresh runs. Two things
-- keep it current, and both must stay:
--   1. this migration REFRESHes on every run, and the Helm pre-install/
--      pre-upgrade hook runs it on every deploy;
--   2. `integrasi-snapshot-refresh` (a Helm CronJob) refreshes on the sync
--      cadence, covering the gap between deploys.
-- Non-CONCURRENTLY is deliberate: REFRESH ... CONCURRENTLY needs a UNIQUE index
-- over plain columns, and this data has neither a unique key (4 kdsatker_keu
-- appear twice, under variant spellings) nor NOT NULL (65 SIMAN-only rows carry
-- kode_satker IS NULL). The refresh takes ~5 s and holds ACCESS EXCLUSIVE for
-- that time; at sync cadence that is a far better trade than 11 s per request.
--
-- Expand/contract: additive only; idempotent (IF NOT EXISTS / CREATE OR REPLACE).
-- ============================================================================

SET search_path TO integrasi, public;

-- ----------------------------------------------------------------------------
-- Materialized snapshot of the auto-derive. Body is deliberately IDENTICAL to
-- `v_satker_code_map_auto` (003) rather than a re-expression of it: the plain
-- view is kept as the readable definition and as the review/seeding tool, and
-- any divergence between the two would be a silent correctness bug.
-- ----------------------------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS integrasi.mv_satker_code_map_auto AS
SELECT * FROM integrasi.v_satker_code_map_auto;

CREATE INDEX IF NOT EXISTS idx_mv_satker_map_kode
    ON integrasi.mv_satker_code_map_auto (kode_satker);
CREATE INDEX IF NOT EXISTS idx_mv_satker_map_kdsatker
    ON integrasi.mv_satker_code_map_auto (kdsatker_keu);
CREATE INDEX IF NOT EXISTS idx_mv_satker_map_wilayah
    ON integrasi.mv_satker_code_map_auto (wilayah_kode);

-- ----------------------------------------------------------------------------
-- Repoint the resolved view at the snapshot. Same columns, same order, same
-- types and same rows as 003 — only the auto branch's SOURCE changes, so
-- CREATE OR REPLACE is valid and no consumer needs to know.
--
-- 003 recreates the pre-materialization definition every time the runner
-- executes (migrations are re-applied in lexical order on each invocation), so
-- this file MUST sort after it. That is load-bearing, not cosmetic.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW integrasi.v_satker_code_map AS
SELECT
    kode_satker,
    kdsatker_keu,
    substring(kdsatker_keu FROM 6 FOR 4) AS wilayah_kode,
    (right(kdsatker_keu, 2) = 'KP')      AS is_pusat,
    nama_satker,
    match_method,
    TRUE AS verified
FROM integrasi.satker_code_map
WHERE kdsatker_keu IS NOT NULL
UNION ALL
SELECT
    a.kode_satker,
    a.kdsatker_keu,
    a.wilayah_kode,
    a.is_pusat,
    COALESCE(a.nama_satker_mysimkari, a.nama_satker_siman) AS nama_satker,
    a.match_method,
    FALSE AS verified
FROM integrasi.mv_satker_code_map_auto a
WHERE a.kdsatker_keu NOT IN (
    SELECT kdsatker_keu FROM integrasi.satker_code_map WHERE kdsatker_keu IS NOT NULL
);

-- Always leave the snapshot current for the code that is about to start. On a
-- fresh install `siman_aset` is still empty, which is correct and harmless: the
-- snapshot is empty, nothing is scoped yet, and the first post-sync refresh
-- fills it.
REFRESH MATERIALIZED VIEW integrasi.mv_satker_code_map_auto;

ANALYZE integrasi.mv_satker_code_map_auto;

COMMENT ON MATERIALIZED VIEW integrasi.mv_satker_code_map_auto IS
    'Snapshot of v_satker_code_map_auto. Exists purely for speed: the underlying '
    'view re-derives the national mapping with a DISTINCT over all of siman_aset, '
    'which made every satker-scoped bank_aset query O(all assets). Refreshed by '
    'the integrasi migrate job on deploy and by the snapshot-refresh CronJob on '
    'the sync cadence. STALE = satker-scoped users fail closed to zero rows.';
