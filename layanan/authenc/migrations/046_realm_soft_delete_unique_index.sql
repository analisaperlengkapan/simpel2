-- ============================================================================
-- MIGRATION 046 (duplicate): User extended fields
-- ============================================================================
-- WARNING: This file is a duplicate of 046_user_extended_fields.sql. Both
-- files parse to version 46 in the migration runner. The runner records
-- whichever file it processes first and skips the other. To ensure the
-- correct DDL is applied regardless of filesystem ordering, this file
-- contains the same SQL as 046_user_extended_fields.sql.
--
-- The realm soft-delete migration lives in 048_realm_soft_delete_unique_index.sql.
-- ============================================================================

BEGIN;

ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS nip VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS nama VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS jabatan VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_number VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_verified BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS organization_id UUID;

ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_backup_codes TEXT[];

ALTER TABLE users ADD COLUMN IF NOT EXISTS webauthn_enabled BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE users ADD COLUMN IF NOT EXISTS account_locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_locked_until TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_failed_login_at TIMESTAMPTZ;

ALTER TABLE users ADD COLUMN IF NOT EXISTS attributes JSONB;

CREATE INDEX IF NOT EXISTS idx_users_nip ON users(nip) WHERE nip IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_nama ON users(nama) WHERE nama IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id) WHERE organization_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_account_locked ON users(account_locked) WHERE account_locked = true;

COMMIT;
