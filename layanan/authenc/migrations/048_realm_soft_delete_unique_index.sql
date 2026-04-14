-- ============================================================================
-- MIGRATION 048: Add missing realm columns + fix name uniqueness for soft delete
-- ============================================================================
-- 1. The Realm domain struct and all PostgresRealmStore queries expect ~37
--    configuration columns (ssl_required, registration_allowed, token
--    lifespans, etc.) that were never created by 001_initial_schema.sql.
--    This migration adds them with sensible defaults so that existing rows
--    (including the master realm) are back-filled automatically.
--
-- 2. The original schema uses a plain UNIQUE constraint on realms.name which
--    prevents re-creating a realm with the same name after soft-deletion
--    (the soft-deleted row still occupies the unique slot).
--    This migration replaces the absolute UNIQUE constraint with a partial
--    unique index that only enforces uniqueness among live (non-deleted) rows,
--    matching the pattern already used for the users table (see test_schema.sql).
-- ============================================================================

-- ── Step 0: Add missing realm configuration columns ──────────────────────────
-- These columns are expected by Realm::try_from(Row) in authenc_types and by
-- every SELECT in PostgresRealmStore.  Using ADD COLUMN IF NOT EXISTS so the
-- migration is idempotent.

ALTER TABLE realms ADD COLUMN IF NOT EXISTS ssl_required VARCHAR(50) NOT NULL DEFAULT 'external';
ALTER TABLE realms ADD COLUMN IF NOT EXISTS registration_allowed BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS registration_email_as_username BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS remember_me BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS verify_email BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS login_with_email_allowed BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS duplicate_emails_allowed BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS reset_password_allowed BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS edit_username_allowed BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS brute_force_protected BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS max_failure_wait_seconds INTEGER NOT NULL DEFAULT 900;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS minimum_quick_login_wait_seconds INTEGER NOT NULL DEFAULT 60;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS wait_increment_seconds INTEGER NOT NULL DEFAULT 60;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS quick_login_check_milli_seconds BIGINT NOT NULL DEFAULT 1000;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS max_delta_time_seconds INTEGER NOT NULL DEFAULT 43200;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS failure_factor INTEGER NOT NULL DEFAULT 30;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS default_signature_algorithm VARCHAR(50) NOT NULL DEFAULT 'RS256';
ALTER TABLE realms ADD COLUMN IF NOT EXISTS revoke_refresh_token BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS refresh_token_max_reuse INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS access_token_lifespan INTEGER NOT NULL DEFAULT 300;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS access_token_lifespan_for_implicit_flow INTEGER NOT NULL DEFAULT 900;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS sso_session_idle_timeout INTEGER NOT NULL DEFAULT 1800;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS sso_session_max_lifespan INTEGER NOT NULL DEFAULT 36000;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS sso_session_idle_timeout_remember_me INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS sso_session_max_lifespan_remember_me INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS offline_session_idle_timeout INTEGER NOT NULL DEFAULT 2592000;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS offline_session_max_lifespan INTEGER NOT NULL DEFAULT 5184000;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS client_session_idle_timeout INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS client_session_max_lifespan INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS access_code_lifespan INTEGER NOT NULL DEFAULT 60;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS access_code_lifespan_user_action INTEGER NOT NULL DEFAULT 300;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS access_code_lifespan_login INTEGER NOT NULL DEFAULT 1800;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS action_token_generated_by_admin_lifespan INTEGER NOT NULL DEFAULT 43200;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS action_token_generated_by_user_lifespan INTEGER NOT NULL DEFAULT 300;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS oauth2_device_code_lifespan INTEGER NOT NULL DEFAULT 600;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS oauth2_device_polling_interval INTEGER NOT NULL DEFAULT 5;
ALTER TABLE realms ADD COLUMN IF NOT EXISTS attributes TEXT;

-- ── Step 1: Drop the existing absolute unique constraint on realms.name. ─────
-- PostgreSQL names inline UNIQUE constraints as "<table>_<column>_key" by
-- default, but this can vary. We use a DO block to find and drop the actual
-- constraint name from the catalog, falling back to the conventional name.
DO $$
DECLARE
    _constraint_name TEXT;
BEGIN
    -- Find the unique constraint on realms.name from pg_catalog
    SELECT c.conname INTO _constraint_name
    FROM pg_constraint c
    JOIN pg_class t ON c.conrelid = t.oid
    JOIN pg_namespace n ON t.relnamespace = n.oid
    JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(c.conkey)
    WHERE t.relname = 'realms'
      AND a.attname = 'name'
      AND c.contype = 'u'
      AND n.nspname = 'public'
    LIMIT 1;

    IF _constraint_name IS NOT NULL THEN
        EXECUTE format('ALTER TABLE realms DROP CONSTRAINT %I', _constraint_name);
        RAISE NOTICE 'Dropped unique constraint % on realms.name', _constraint_name;
    ELSE
        RAISE NOTICE 'No unique constraint found on realms.name (may already be dropped)';
    END IF;
END $$;

-- Also drop any plain unique index on realms.name (in case it was created as
-- an index rather than a constraint).
DROP INDEX IF EXISTS realms_name_key;

-- Step 2: Create a partial unique index that only covers live rows
CREATE UNIQUE INDEX IF NOT EXISTS idx_realms_name_unique
    ON realms (name)
    WHERE deleted_at IS NULL;

-- Step 3: Change users.realm_id FK from ON DELETE CASCADE to ON DELETE RESTRICT.
-- With soft-delete semantics, a raw DELETE FROM realms should be blocked
-- (not silently cascade-delete all users). The application always uses
-- soft-delete via UPDATE, so the CASCADE is never triggered in normal
-- operation — but RESTRICT prevents accidental data loss from manual SQL.
DO $$
DECLARE
    _fk_name TEXT;
BEGIN
    SELECT c.conname INTO _fk_name
    FROM pg_constraint c
    JOIN pg_class t ON c.conrelid = t.oid
    JOIN pg_namespace n ON t.relnamespace = n.oid
    JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(c.conkey)
    WHERE t.relname = 'users'
      AND a.attname = 'realm_id'
      AND c.contype = 'f'
      AND n.nspname = 'public'
    LIMIT 1;

    IF _fk_name IS NOT NULL THEN
        EXECUTE format('ALTER TABLE users DROP CONSTRAINT %I', _fk_name);
        EXECUTE 'ALTER TABLE users ADD CONSTRAINT users_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE RESTRICT';
        RAISE NOTICE 'Changed users.realm_id FK from CASCADE to RESTRICT (was %)', _fk_name;
    ELSE
        RAISE NOTICE 'No FK found on users.realm_id (may not exist)';
    END IF;
END $$;
