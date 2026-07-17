-- ============================================================================
-- Migration V004: bantuan support tickets (#98)
-- ============================================================================
-- The /bantuan/helpdesk page previously *faked* its submission: the "Kirim
-- Pesan" button flipped a local signal to render "Pesan Terkirim — Tim helpdesk
-- akan merespons dalam 1×24 jam kerja" while the subject/message were captured
-- into signals and dropped on the floor. Nothing was ever persisted and no
-- helpdesk operator could ever see it — the page told users a falsehood.
--
-- The bantuan service code queried a `bantuan.*` schema that NO migration ever
-- created (verified: the only `bantuan` mentions in V001 are audit_log comments),
-- so it would have failed at runtime had it ever been mounted. This migration
-- creates the schema the ticket path actually needs.
--
-- SCOPE: tickets + comments only. The rest of the former bantuan module
-- (chatbot/knowledge/analytics/gdpr/webhook/export_import/faq) is removed in the
-- same change: it had no UI, no schema, no requirement, and the chatbot required
-- an AI_SERVICE_URL that is configured in no compose file and no Helm values
-- (its config loader `.expect()`s the var — it would panic on boot). FAQ and
-- Panduan are static pages by design and need no backend.
--
-- RBAC: `user_id` is the ticket owner, derived server-side from JWT claims at
-- create — NEVER from client input. The pre-existing handlers read `user_id`
-- from the request payload/query, which would have allowed filing a ticket AS
-- another user and reading anyone's tickets by id. `satker_code` mirrors the
-- V003 pattern (identity-derived MySIMKARI kode_satker) so helpdesk staff can be
-- scoped per satker/wilayah later without a schema change.
--
-- Expand/contract: purely additive (new schema + new tables). No existing object
-- is altered or dropped, so `helm rollback` to the prior app revision leaves
-- these tables orphaned-but-harmless and needs no DB restore.
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS bantuan;

COMMENT ON SCHEMA bantuan IS
    'Helpdesk support tickets raised from the /bantuan/helpdesk page. Owned by '
    'layanan-perlengkapan. Operational module: audit retention 3y (see '
    'perlengkapan.audit_log.retention_until).';

-- ---------------------------------------------------------------------------
-- support_tickets
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS bantuan.support_tickets (
    id           UUID PRIMARY KEY,
    user_id      UUID NOT NULL,
    satker_code  TEXT,
    subject      TEXT NOT NULL,
    description  TEXT,
    priority     TEXT NOT NULL DEFAULT 'normal',
    status       TEXT NOT NULL DEFAULT 'open',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at    TIMESTAMPTZ,
    CONSTRAINT support_tickets_subject_not_blank
        CHECK (length(btrim(subject)) > 0),
    CONSTRAINT support_tickets_status_valid
        CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
    CONSTRAINT support_tickets_priority_valid
        CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    -- A ticket is closed_at exactly when it reached a terminal status. Encoding
    -- it here means a bug in the service can't silently produce a "closed"
    -- ticket with no closure timestamp (or vice versa).
    CONSTRAINT support_tickets_closed_at_matches_status
        CHECK ((status IN ('resolved', 'closed')) = (closed_at IS NOT NULL))
);

ALTER TABLE bantuan.support_tickets
    DROP CONSTRAINT IF EXISTS support_tickets_user_id_fkey;

ALTER TABLE bantuan.support_tickets
    ADD CONSTRAINT support_tickets_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES authenc.users(id);

COMMENT ON TABLE bantuan.support_tickets IS
    'Helpdesk tickets submitted via /bantuan/helpdesk.';
COMMENT ON COLUMN bantuan.support_tickets.user_id IS
    'Ticket owner. Derived from JWT claims server-side at create — never from '
    'client payload (an earlier draft trusted payload.user_id = impersonation).';
COMMENT ON COLUMN bantuan.support_tickets.satker_code IS
    'MySIMKARI kode_satker of the reporter, derived from Claims.satker_code at '
    'create (same provenance rule as V003). Nullable: users without a satker '
    'claim can still raise tickets.';

CREATE INDEX IF NOT EXISTS idx_support_tickets_user_id
    ON bantuan.support_tickets (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_support_tickets_status
    ON bantuan.support_tickets (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_support_tickets_satker_code
    ON bantuan.support_tickets (satker_code);

-- ---------------------------------------------------------------------------
-- ticket_comments
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS bantuan.ticket_comments (
    id         UUID PRIMARY KEY,
    ticket_id  UUID NOT NULL,
    user_id    UUID NOT NULL,
    content    TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ticket_comments_content_not_blank
        CHECK (length(btrim(content)) > 0)
);

ALTER TABLE bantuan.ticket_comments
    DROP CONSTRAINT IF EXISTS ticket_comments_ticket_id_fkey;

-- ON DELETE CASCADE: comments have no meaning without their ticket.
ALTER TABLE bantuan.ticket_comments
    ADD CONSTRAINT ticket_comments_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES bantuan.support_tickets(id) ON DELETE CASCADE;

ALTER TABLE bantuan.ticket_comments
    DROP CONSTRAINT IF EXISTS ticket_comments_user_id_fkey;

ALTER TABLE bantuan.ticket_comments
    ADD CONSTRAINT ticket_comments_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES authenc.users(id);

COMMENT ON TABLE bantuan.ticket_comments IS
    'Threaded replies on a support ticket (reporter and helpdesk staff).';
COMMENT ON COLUMN bantuan.ticket_comments.user_id IS
    'Comment author, derived from JWT claims server-side — never client payload.';

CREATE INDEX IF NOT EXISTS idx_ticket_comments_ticket_id
    ON bantuan.ticket_comments (ticket_id, created_at ASC);
