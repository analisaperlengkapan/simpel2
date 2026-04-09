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

-- Step 1: Drop the existing absolute unique constraint on realms.name
ALTER TABLE realms DROP CONSTRAINT IF EXISTS realms_name_key;

-- Step 2: Create a partial unique index that only covers live rows
CREATE UNIQUE INDEX IF NOT EXISTS idx_realms_name_unique
    ON realms (name)
    WHERE deleted_at IS NULL;
