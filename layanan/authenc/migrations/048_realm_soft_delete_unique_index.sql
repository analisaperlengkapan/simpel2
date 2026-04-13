-- This file was created in error during renumbering.
-- The actual migration lives in 046_realm_soft_delete_unique_index.sql.
-- This file is a no-op; all statements below are idempotent guards that
-- do nothing if 046 has already been applied.

CREATE UNIQUE INDEX IF NOT EXISTS idx_realms_name_unique
    ON realms (name)
    WHERE deleted_at IS NULL;
