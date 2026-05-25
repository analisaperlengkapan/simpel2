-- Migration: Persistent storage for TOTP secrets and MFA backup codes
-- Description: Backs `authenc_mfa::TotpStore` and `BackupCodesStore` with
--              Postgres tables so the REST `MfaApiService` survives pod
--              restarts. Required by Phase 1.3 of the stabilization plan
--              (tolong-bantu-saya-saya-tender-flamingo) — without these
--              tables the MFA REST adapter could only run in-memory.
--
-- Security note: TOTP secrets are written as base32 plaintext here. The
-- authenc DB is treated as a secret-grade store (`mfa_secrets` access is
-- gated by the same Secreton-bootstrapped credentials as JWT material),
-- but a future enhancement should wrap these columns with the existing
-- `authenc-crypto` envelope encryption helpers.

CREATE TABLE IF NOT EXISTS authenc.totp_secrets (
    user_id UUID PRIMARY KEY REFERENCES authenc.users(id) ON DELETE CASCADE,
    secret TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE  authenc.totp_secrets         IS 'Per-user TOTP secret (base32) backing authenc_mfa::TotpStore';
COMMENT ON COLUMN authenc.totp_secrets.secret  IS 'Base32-encoded TOTP secret; rotate on disable/re-enroll';

CREATE TABLE IF NOT EXISTS authenc.mfa_backup_codes (
    user_id UUID NOT NULL REFERENCES authenc.users(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, code)
);

COMMENT ON TABLE  authenc.mfa_backup_codes        IS 'One-time MFA recovery codes; pruned on regenerate';
COMMENT ON COLUMN authenc.mfa_backup_codes.used   IS 'Once true, never reused — enforced at the application layer';

-- Hot-path index: when verifying a code we look it up by (user_id, code)
-- so the PK alone is sufficient. Add a secondary index on (user_id, used)
-- for the "how many recovery codes do I have left" admin queries.
CREATE INDEX IF NOT EXISTS idx_mfa_backup_codes_user_unused
    ON authenc.mfa_backup_codes (user_id)
    WHERE used = FALSE;
