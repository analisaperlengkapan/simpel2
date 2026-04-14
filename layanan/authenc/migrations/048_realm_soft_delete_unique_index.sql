-- ============================================================================
-- MIGRATION 048: Fix realm name uniqueness for soft delete
-- ============================================================================
-- The original schema uses a plain UNIQUE constraint on realms.name which
-- prevents re-creating a realm with the same name after soft-deletion
-- (the soft-deleted row still occupies the unique slot).
--
-- This migration replaces the absolute UNIQUE constraint with a partial
-- unique index that only enforces uniqueness among live (non-deleted) rows,
-- matching the pattern already used for the users table (see test_schema.sql).
-- ============================================================================

-- Step 1: Drop the existing absolute unique constraint on realms.name.
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
