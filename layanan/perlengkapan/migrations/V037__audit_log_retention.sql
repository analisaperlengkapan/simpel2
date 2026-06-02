-- Migration V037: BPK-ready retention metadata for perlengkapan.audit_log
--
-- Fase 2.9 — the cross-module audit sink (V028) already captures every
-- workflow transition, document action, login, export, etc. via
-- `lib_perlengkapan::contracts::AuditSink`. What it lacked was an explicit
-- *retention horizon* so an archival/purge job (and BPK reviewers) can tell
-- how long each row must be kept.
--
-- Per PMK pengelolaan BMN, audit records tied to Barang Milik Negara must be
-- retained for at least 10 years. Operational chatter (bantuan tickets,
-- in-app notifications) does not carry the same statutory weight, so it gets
-- a shorter 3-year horizon. The column default is the conservative 10-year
-- value: any new event without an explicit horizon is kept the longest, so a
-- module that forgets to classify itself never under-retains.
--
-- Fully additive & backward compatible: no column dropped, existing rows
-- backfilled via UPDATE, re-runnable (IF NOT EXISTS + idempotent backfill).

ALTER TABLE perlengkapan.audit_log
    ADD COLUMN IF NOT EXISTS retention_until TIMESTAMPTZ
    NOT NULL DEFAULT (NOW() + INTERVAL '10 years');

COMMENT ON COLUMN perlengkapan.audit_log.retention_until IS
    'Statutory retention horizon. BMN modules: occurred_at + 10y (PMK); '
    'operational modules (bantuan/notifikasi): occurred_at + 3y. Rows must '
    'not be purged before this instant.';

-- Backfill existing rows by module class. Idempotent: re-running re-derives
-- the same value from the immutable occurred_at, so it is safe to apply more
-- than once (refinery guards against that anyway).
UPDATE perlengkapan.audit_log
SET retention_until = CASE
        WHEN module IN ('bantuan', 'notifikasi')
            THEN occurred_at + INTERVAL '3 years'
        ELSE occurred_at + INTERVAL '10 years'
    END;

-- Supports the eventual archival sweep ("rows whose retention_until < now()")
-- without scanning the whole table.
CREATE INDEX IF NOT EXISTS idx_audit_log_retention
    ON perlengkapan.audit_log (retention_until);
