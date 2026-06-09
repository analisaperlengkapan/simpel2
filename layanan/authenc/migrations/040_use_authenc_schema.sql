-- Migration 040: ensure the dedicated `authenc` schema exists.
--
-- Original intent was to MOVE all authenc tables from public → authenc. That
-- never worked: `ALTER DATABASE CURRENT SET ...` is invalid syntax, so this
-- migration always aborted and tables stayed in `public`. The running app
-- matches that reality — it queries most tables unqualified (public) and only
-- a few are explicitly `authenc.*` (token_revocations [050], totp_secrets &
-- mfa_backup_codes [049]).
--
-- F5-B decision (2026-06-09): keep the public-majority topology (no app change)
-- and use the `authenc` schema only for those explicitly-qualified tables.
-- This migration is therefore reduced to ensuring the schema exists so the
-- later qualified CREATE TABLEs (049/050) succeed.
CREATE SCHEMA IF NOT EXISTS authenc;
COMMENT ON SCHEMA authenc IS 'Authenc-owned tables referenced schema-qualified (token_revocations, totp_secrets, mfa_backup_codes). Most authenc tables live in public.';
