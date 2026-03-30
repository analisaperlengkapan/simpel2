-- Migration: 046_user_extended_fields.sql
-- Purpose: Add extended user fields for pegawai data, account security, and TOTP/WebAuthn tracking
-- Date: 2026-02-12
--
-- The User domain struct expects these columns but they were missing from previous migrations.
-- Uses IF NOT EXISTS to be idempotent (safe to re-run).

BEGIN;

-- ============================================================================
-- 1. Pegawai identity fields (populated from MySIMKARI via integrasi gRPC)
-- ============================================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS nip VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS nama VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS jabatan VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_number VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_verified BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS organization_id UUID;

-- ============================================================================
-- 2. TOTP / MFA fields (managed by MFA service)
-- ============================================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_backup_codes TEXT[];

-- ============================================================================
-- 3. WebAuthn tracking
-- ============================================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS webauthn_enabled BOOLEAN NOT NULL DEFAULT FALSE;

-- ============================================================================
-- 4. Account lockout fields (brute-force protection)
-- ============================================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_locked_until TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_failed_login_at TIMESTAMPTZ;

-- ============================================================================
-- 5. Extensible attributes (JSONB for custom fields)
-- ============================================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS attributes JSONB;

-- ============================================================================
-- 6. Indexes for new columns
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_users_nip ON users(nip) WHERE nip IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_nama ON users(nama) WHERE nama IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id) WHERE organization_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_account_locked ON users(account_locked) WHERE account_locked = true;

COMMIT;
