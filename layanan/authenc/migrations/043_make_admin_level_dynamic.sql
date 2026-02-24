-- Migration: Make Admin Level Dynamic
-- Description: Remove hardcoded enum constraints and link to dynamic admin_level_types

-- 1. Drop the hardcoded check constraint
ALTER TABLE satker_admin_roles DROP CONSTRAINT IF EXISTS chk_admin_level;

-- 2. Migrate existing data to match new codes in admin_level_types (from 042 migration)
-- 'AdminPusat' -> 'pusat'
-- 'AdminEselonI' -> 'eselon_i'
-- 'AdminWilayah' -> 'wilayah'
-- 'AdminSatker' -> 'satker'

UPDATE satker_admin_roles SET admin_level = 'pusat' WHERE admin_level = 'AdminPusat';
UPDATE satker_admin_roles SET admin_level = 'eselon_i' WHERE admin_level = 'AdminEselonI';
UPDATE satker_admin_roles SET admin_level = 'wilayah' WHERE admin_level = 'AdminWilayah';
UPDATE satker_admin_roles SET admin_level = 'satker' WHERE admin_level = 'AdminSatker';

-- 3. Add foreign key constraint to ensure validity against dynamic types
-- We reference 'code' in admin_level_types. 042 created a unique constraint on (code, realm_id).
-- Since satker_admin_roles doesn't have realm_id context easily (it might be global or implied),
-- and admin_level_types unique constraint includes realm_id, simple FK might be tricky if realm_id is NULL.
-- However, 042 inserts bootstrap types with realm_id = NULL.
-- Let's try to add FK referencing code where realm_id IS NULL for system types.
-- But standard FK requires a unique constraint on the referenced column(s).
-- admin_level_types has UNIQUE(code, realm_id).
-- If we want to support realm-specific levels, satker_admin_roles should probably have realm_id.
-- For now, let's just index it and rely on application validation to allow flexibility "like Keycloak".
-- Keycloak often relies on logical validation.

CREATE INDEX IF NOT EXISTS idx_satker_admin_roles_level_code ON satker_admin_roles(admin_level);

-- 4. Update satker_permissions if needed (it uses permission_type, resource_type, action - generic strings)
-- No generic changes needed there.

-- 5. Drop constraints on other tables if they exist
-- None found in 032 other than chk_admin_level.
