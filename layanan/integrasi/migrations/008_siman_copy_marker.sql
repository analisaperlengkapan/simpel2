-- Migration 008: where does the last COMPLETE copy of a SIMAN category end.
--
-- Each complete SIMAN run replaces the local copy of a category: it writes the new rows next to the
-- old ones, then deletes the old (`settle_category`, src/siman/endpoints.rs). A run that is killed in
-- between (SIGKILL, an OOM, a node lost) never settles, and the rows it had written stay next to the
-- complete copy: readers see the register twice until the next COMPLETE run. Found on staging
-- 2026-11-20: 311,478 extra rows of one category after the pod was stopped at its deadline.
--
-- `settled_at` is the database's own clock at the moment a category was last settled by a complete
-- run. Every row of that copy has `synced_at <= settled_at`, so a row of the category with
-- `synced_at > settled_at` can only belong to a run that never settled. The next run of the category
-- deletes those rows before it starts (`reclaim_unsettled_rows`). A category with no row here (the
-- state of every deployment before this migration) is left alone: nothing proves what is a leftover.
--
-- Replayed on every deploy (AGENTS.md §0): CREATE TABLE IF NOT EXISTS converges.

CREATE TABLE IF NOT EXISTS integrasi.siman_copy (
    jenis_aset  TEXT        PRIMARY KEY,
    settled_at  TIMESTAMPTZ NOT NULL
);

COMMENT ON TABLE integrasi.siman_copy IS
    'Per SIMAN category: when its last complete copy was settled; rows of the category stamped later belong to a run that never finished (src/siman/endpoints.rs)';
