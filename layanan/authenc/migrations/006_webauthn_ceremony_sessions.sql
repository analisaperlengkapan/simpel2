-- ============================================================================
-- Migration 006: WebAuthn ceremony state shared between replicas
-- ============================================================================
-- A passkey registration or login is two requests, `start` and `finish`, and the
-- state webauthn-rs returns from `start` is needed again at `finish`. It was kept
-- in the memory of the replica that answered `start`. authenc runs two replicas
-- and requests are not pinned to one, so about half of all ceremonies finished
-- on the other replica and failed with "session not found or expired" (measured
-- on staging: 4 of 8 registrations).
--
-- The rows are short lived (10 minutes) and consumed exactly once, by a
-- `DELETE ... RETURNING`; expired rows are removed by the next write. Nothing
-- here is a secret that outlives a ceremony: the payload is the challenge and
-- the verification state of one in-flight ceremony.
--
-- Expand-only: a new table nothing else reads. An older binary simply does not
-- know it exists, so rolling the app back over this migration is safe (the
-- authenc runner applies the migration files it has and ignores history rows it
-- has no file for).

CREATE TABLE IF NOT EXISTS authenc.webauthn_ceremony_sessions (
    id         TEXT        PRIMARY KEY,
    kind       TEXT        NOT NULL CHECK (kind IN ('registration', 'authentication')),
    payload    TEXT        NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_webauthn_ceremony_sessions_expires
    ON authenc.webauthn_ceremony_sessions (expires_at);
