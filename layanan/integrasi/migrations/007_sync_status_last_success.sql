-- Migration 007: when did each source last sync SUCCESSFULLY.
--
-- `integrasi.sync_status` keeps the outcome of the LAST run only. After a failed run it cannot say
-- how old the data is that the application is reading, which is the first thing an administrator
-- needs to know ("the last sync failed" is harmless an hour after a good one and serious a week
-- after). The scheduler writes this column on every successful run (`src/sync_run.rs`).
--
-- Replayed on every deploy (see AGENTS.md §0): ADD COLUMN IF NOT EXISTS converges, and no view
-- selects `*` from this table, so `v_sync_status_dashboard` (002) is unaffected.

ALTER TABLE integrasi.sync_status ADD COLUMN IF NOT EXISTS last_success_at TIMESTAMPTZ;

COMMENT ON COLUMN integrasi.sync_status.last_success_at IS
    'End of the last run of this source that completed; written by the scheduler (src/sync_run.rs)';
