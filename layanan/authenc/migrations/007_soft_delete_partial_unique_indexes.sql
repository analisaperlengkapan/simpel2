-- Migration: Convert unconditional UNIQUE constraints on users.username and
-- users.email to partial unique indexes filtered on `deleted_at IS NULL`.
--
-- This is required because the application now performs soft-delete by setting
-- `deleted_at` (instead of hard-deleting rows).  Without partial indexes the
-- database would reject re-provisioning a username/email that belongs to a
-- soft-deleted record, even though `username_exists` / `email_exists` queries
-- (which filter on `deleted_at IS NULL`) report the name as available.

-- Drop the old unconditional constraints
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_username_key;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_key;

-- Create partial unique indexes that only enforce uniqueness among live rows
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_unique
    ON users (username) WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique
    ON users (email) WHERE deleted_at IS NULL;
