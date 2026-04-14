-- ============================================================================
-- MIGRATION 046: Placeholder (realm soft-delete migration is in 048)
-- ============================================================================
-- The realm soft-delete unique index migration lives in
-- 048_realm_soft_delete_unique_index.sql.
--
-- This file is intentionally a no-op. It exists solely to occupy version 046
-- so that no future migration accidentally reuses this number.
-- ============================================================================
SELECT 1;
