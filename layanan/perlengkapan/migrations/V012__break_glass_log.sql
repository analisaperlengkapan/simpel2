-- ============================================================================
-- Migration V012: break-glass log
-- ============================================================================
-- Emergency intervention in a business workflow (an administrator forcing a
-- state change on a stuck usulan/izin) used to be a side effect of `admin`
-- bypassing every role check. It left NO distinct trace: the activity row
-- simply read "admin approved", indistinguishable from a legitimate approval,
-- and there was no reason recorded and nobody to review it.
--
-- The bypass is gone (see `shared::policy`). What replaces it is an explicit,
-- audited, reason-mandatory path: `POST /admin/break-glass/{module}/{id}/transition`.
-- Every attempt — including a refused or failed one — is a row here, written
-- BEFORE the transition runs (outcome `pending`) and settled afterwards, so an
-- action that ran but could not be logged cannot exist: if the insert fails the
-- transition is not attempted.
--
-- Reviewed by Validator Pusat and administrators via `GET /admin/break-glass`.
--
-- Expand/contract: purely additive (one new table + indexes). No existing object
-- is altered or dropped, so `helm rollback` of the app needs no DB restore.
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan.break_glass_log (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- One of: pemakaian_bmn | penghapusan_bmn | kebutuhan_bmn
    module         TEXT NOT NULL,
    entity_id      UUID NOT NULL,
    from_state     TEXT,
    to_state       TEXT NOT NULL,
    -- `pending` is written before the transition runs and settled to
    -- `success` / `failure` after; a row left `pending` means the process died
    -- mid-action and is itself something a reviewer should look at.
    outcome        TEXT NOT NULL DEFAULT 'pending'
                   CHECK (outcome IN ('pending', 'success', 'failure')),
    error          TEXT,
    -- Mandatory written justification (the API enforces >= 20 characters).
    reason         TEXT NOT NULL,
    -- Optional ticket / incident / nota-dinas reference.
    reference      TEXT,
    actor_user_id  UUID NOT NULL,
    actor_username TEXT,
    actor_roles    TEXT[] NOT NULL DEFAULT '{}',
    ip_address     TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    settled_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_break_glass_log_entity
    ON perlengkapan.break_glass_log (module, entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_break_glass_log_created
    ON perlengkapan.break_glass_log (created_at DESC);

COMMENT ON TABLE perlengkapan.break_glass_log IS
    'Audited emergency workflow interventions by an application administrator. '
    'One row per attempt, written before the transition runs. Owned by '
    'layanan-perlengkapan; retained with the workflow audit trail.';
