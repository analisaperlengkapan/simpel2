-- Migration V028: perlengkapan-wide audit_log table
--
-- Cross-module audit trail surfaced by
-- `lib_perlengkapan::contracts::AuditSink`. Existing per-module audit
-- tables (`dokumen.audit_log` from V021, `bantuan.captcha_logs`, etc.)
-- stay in place for their domain-specific events; this table is the
-- *cross-cutting* audit sink that any module can write to without
-- coupling to another module's schema. Concrete writer:
-- `crate::shared::audit::PgAuditSink`.

CREATE SCHEMA IF NOT EXISTS perlengkapan;

CREATE TABLE IF NOT EXISTS perlengkapan.audit_log (
    id              UUID         PRIMARY KEY,
    occurred_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    actor_user_id   UUID,
    actor_username  VARCHAR(255),
    actor_ip        VARCHAR(64),
    action          VARCHAR(32)  NOT NULL,
    action_name     VARCHAR(128),
    resource_type   VARCHAR(64)  NOT NULL,
    resource_id     VARCHAR(255),
    module          VARCHAR(64)  NOT NULL,
    success         BOOLEAN      NOT NULL DEFAULT TRUE,
    message         TEXT,
    metadata        JSONB
);

CREATE INDEX IF NOT EXISTS idx_audit_log_actor
    ON perlengkapan.audit_log (actor_user_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_module
    ON perlengkapan.audit_log (module, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource
    ON perlengkapan.audit_log (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_occurred
    ON perlengkapan.audit_log (occurred_at DESC);

COMMENT ON TABLE perlengkapan.audit_log IS
    'Cross-module audit trail produced via lib_perlengkapan::contracts::AuditSink';
COMMENT ON COLUMN perlengkapan.audit_log.action IS
    'Stable action verb: create | read | update | delete | login | logout | approve | reject | submit | cancel | export | import | custom';
COMMENT ON COLUMN perlengkapan.audit_log.action_name IS
    'Free-form action name when action = custom (e.g. workflow.delegate)';
COMMENT ON COLUMN perlengkapan.audit_log.module IS
    'Source module: workflow | dokumen | notifikasi | bantuan | pemakaian | penghapusan | …';
