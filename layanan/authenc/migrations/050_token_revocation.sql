-- Migration: Token revocation list (F2H token revocation)
-- Purpose: Server-side revocation so access tokens can be invalidated before
--          their `exp`. Supports three granularities:
--   - kind='jti'  → revoke a single token (by JWT ID)
--   - kind='sid'  → revoke a whole session (logout: all tokens sharing the sid)
--   - kind='user' → revoke ALL tokens for a user issued before the cutoff
--                    (revoked_at), e.g. on role change / account deactivation.
--
-- A token presented to validate/verify is revoked iff a non-expired row matches
-- its jti, its sid, or (kind='user' for its sub AND its iat < revoked_at cutoff).

CREATE SCHEMA IF NOT EXISTS authenc;

CREATE TABLE IF NOT EXISTS authenc.token_revocations (
    kind       TEXT        NOT NULL CHECK (kind IN ('jti', 'sid', 'user')),
    value      TEXT        NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Row may be purged after this instant (set to token/session exp, or for
    -- 'user' cutoffs to revoked_at + max refresh-token lifetime).
    expires_at TIMESTAMPTZ NOT NULL,
    reason     TEXT,
    PRIMARY KEY (kind, value)
);

-- Supports the cleanup sweep (DELETE WHERE expires_at <= NOW()).
CREATE INDEX IF NOT EXISTS idx_token_revocations_expires_at
    ON authenc.token_revocations (expires_at);
