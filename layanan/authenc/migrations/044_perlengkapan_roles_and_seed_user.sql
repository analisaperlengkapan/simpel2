-- Migration: 044_perlengkapan_roles_and_seed_user.sql
-- Purpose: Create perlengkapan-specific roles and seed test user NIP 199203142014031001
-- Date: 2026-02-11
--
-- Role hierarchy for Perlengkapan domain:
--   1. operator_satker    - Data entry at satker level (create/edit kebutuhan, pemakaian, etc.)
--   2. validator_wilayah  - Validate/review at wilayah/region level
--   3. validator_pusat    - Validate/approve at central level
--   4. admin - Full admin access (user management, config, reports)
--
-- These roles are stored in authenc but scoped to the 'perlengkapan' realm/application.
-- The perlengkapan backend reads roles from JWT claims via authenc gRPC validation.

BEGIN;

-- ============================================================================
-- 1. Create perlengkapan application/client registration
-- ============================================================================

INSERT INTO clients (id, realm_id, client_id, name, description, enabled, client_type, protocol, base_url, redirect_uris, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    'perlengkapan',
    'SIMPEL Perlengkapan',
    'Sistem Informasi Manajemen Perlengkapan - BMN lifecycle management',
    true,
    'public',
    'openid-connect',
    '/perlengkapan',
    ARRAY['/perlengkapan/*', '/dashboard/*'],
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT (client_id) DO NOTHING;

-- ============================================================================
-- 2. Create perlengkapan-specific roles
-- ============================================================================

-- Role: operator_satker
INSERT INTO roles (id, realm_id, name, description, composite, client_role, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    'operator_satker',
    'Operator Satker Perlengkapan - Input data kebutuhan BMN, pemakaian, pemeliharaan, pakaian dinas di tingkat satuan kerja',
    false,
    true,
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT DO NOTHING;

-- Role: validator_wilayah
INSERT INTO roles (id, realm_id, name, description, composite, client_role, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    'validator_wilayah',
    'Validator Wilayah Perlengkapan - Verifikasi dan validasi data dari operator satker di tingkat wilayah/Kejaksaan Tinggi',
    false,
    true,
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT DO NOTHING;

-- Role: validator_pusat  
INSERT INTO roles (id, realm_id, name, description, composite, client_role, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    'validator_pusat',
    'Validator Pusat Perlengkapan - Persetujuan akhir di tingkat Kejaksaan Agung/pusat, termasuk penerbitan SK',
    false,
    true,
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT DO NOTHING;

-- Role: admin
INSERT INTO roles (id, realm_id, name, description, composite, client_role, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    'admin',
    'Admin - Pengelolaan pengguna, konfigurasi sistem, master data, laporan komprehensif',
    true,  -- composite: includes all other perlengkapan roles
    true,
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 3. Define role permissions/scopes for each role
-- ============================================================================

-- Create a role_permissions table if not exists (for fine-grained permissions)
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255),  -- e.g., 'kebutuhan_bmn', 'pemakaian_bmn', etc.
    actions TEXT[] DEFAULT '{}',  -- e.g., '{create,read,update,delete,approve}'
    conditions JSONB DEFAULT '{}',  -- e.g., {"satker_scope": "own"} or {"level": "wilayah"}
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(role_id, permission, resource)
);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_resource ON role_permissions(resource);

-- Operator Satker permissions
INSERT INTO role_permissions (role_id, permission, resource, actions, conditions)
SELECT r.id, perm.permission, perm.resource, perm.actions, perm.conditions
FROM roles r
CROSS JOIN (VALUES
    ('manage', 'kebutuhan_bmn', '{create,read,update,delete,submit}', '{"scope": "own_satker"}'::jsonb),
    ('manage', 'pemakaian_bmn', '{create,read,update,delete,submit}', '{"scope": "own_satker"}'::jsonb),
    ('manage', 'penghapusan_bmn', '{create,read,update,submit}', '{"scope": "own_satker"}'::jsonb),
    ('manage', 'pemeliharaan', '{create,read,update,delete}', '{"scope": "own_satker"}'::jsonb),
    ('manage', 'pakaian_dinas', '{create,read,update}', '{"scope": "own_satker"}'::jsonb),
    ('view', 'bank_aset', '{read}', '{"scope": "own_satker"}'::jsonb),
    ('view', 'dashboard', '{read}', '{"scope": "own_satker"}'::jsonb),
    ('manage', 'roadmap_sarpras', '{create,read,update}', '{"scope": "own_satker"}'::jsonb),
    ('view', 'laporan', '{read,export}', '{"scope": "own_satker"}'::jsonb)
) AS perm(permission, resource, actions, conditions)
WHERE r.name = 'operator_satker'
ON CONFLICT (role_id, permission, resource) DO NOTHING;

-- Validator Wilayah permissions
INSERT INTO role_permissions (role_id, permission, resource, actions, conditions)
SELECT r.id, perm.permission, perm.resource, perm.actions, perm.conditions
FROM roles r
CROSS JOIN (VALUES
    ('validate', 'kebutuhan_bmn', '{read,validate,return,forward}', '{"scope": "wilayah"}'::jsonb),
    ('validate', 'pemakaian_bmn', '{read,approve,reject,return}', '{"scope": "wilayah"}'::jsonb),
    ('validate', 'penghapusan_bmn', '{read,validate,return,forward}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'pemeliharaan', '{read}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'pakaian_dinas', '{read,validate}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'bank_aset', '{read}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'dashboard', '{read}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'roadmap_sarpras', '{read}', '{"scope": "wilayah"}'::jsonb),
    ('view', 'laporan', '{read,export}', '{"scope": "wilayah"}'::jsonb)
) AS perm(permission, resource, actions, conditions)
WHERE r.name = 'validator_wilayah'
ON CONFLICT (role_id, permission, resource) DO NOTHING;

-- Validator Pusat permissions  
INSERT INTO role_permissions (role_id, permission, resource, actions, conditions)
SELECT r.id, perm.permission, perm.resource, perm.actions, perm.conditions
FROM roles r
CROSS JOIN (VALUES
    ('approve', 'kebutuhan_bmn', '{read,approve,reject,return}', '{"scope": "pusat"}'::jsonb),
    ('approve', 'pemakaian_bmn', '{read,approve,reject}', '{"scope": "pusat"}'::jsonb),
    ('approve', 'penghapusan_bmn', '{read,approve,reject,generate_sk,sign_sk}', '{"scope": "pusat"}'::jsonb),
    ('view', 'pemeliharaan', '{read}', '{"scope": "all"}'::jsonb),
    ('approve', 'pakaian_dinas', '{read,approve,reject}', '{"scope": "pusat"}'::jsonb),
    ('view', 'bank_aset', '{read}', '{"scope": "all"}'::jsonb),
    ('view', 'dashboard', '{read}', '{"scope": "all"}'::jsonb),
    ('manage', 'roadmap_sarpras', '{read,approve}', '{"scope": "pusat"}'::jsonb),
    ('manage', 'mapping_kodefikasi', '{read,approve,reject}', '{"scope": "pusat"}'::jsonb),
    ('view', 'laporan', '{read,export}', '{"scope": "all"}'::jsonb)
) AS perm(permission, resource, actions, conditions)
WHERE r.name = 'validator_pusat'
ON CONFLICT (role_id, permission, resource) DO NOTHING;

-- Admin permissions (full access)
INSERT INTO role_permissions (role_id, permission, resource, actions, conditions)
SELECT r.id, perm.permission, perm.resource, perm.actions, perm.conditions
FROM roles r
CROSS JOIN (VALUES
    ('admin', 'users', '{create,read,update,delete,assign_role}', '{"scope": "all"}'::jsonb),
    ('admin', 'roles', '{create,read,update,delete}', '{"scope": "all"}'::jsonb),
    ('admin', 'kebutuhan_bmn', '{create,read,update,delete,approve,reject}', '{"scope": "all"}'::jsonb),
    ('admin', 'pemakaian_bmn', '{create,read,update,delete,approve,reject}', '{"scope": "all"}'::jsonb),
    ('admin', 'penghapusan_bmn', '{create,read,update,delete,approve,reject,generate_sk,sign_sk}', '{"scope": "all"}'::jsonb),
    ('admin', 'pemeliharaan', '{create,read,update,delete}', '{"scope": "all"}'::jsonb),
    ('admin', 'pakaian_dinas', '{create,read,update,delete,approve}', '{"scope": "all"}'::jsonb),
    ('admin', 'bank_aset', '{create,read,update,delete}', '{"scope": "all"}'::jsonb),
    ('admin', 'dashboard', '{read,configure}', '{"scope": "all"}'::jsonb),
    ('admin', 'roadmap_sarpras', '{create,read,update,delete,approve}', '{"scope": "all"}'::jsonb),
    ('admin', 'mapping_kodefikasi', '{create,read,update,delete,approve}', '{"scope": "all"}'::jsonb),
    ('admin', 'laporan', '{read,export,generate}', '{"scope": "all"}'::jsonb),
    ('admin', 'konfigurasi', '{read,update}', '{"scope": "all"}'::jsonb),
    ('admin', 'audit_log', '{read}', '{"scope": "all"}'::jsonb),
    ('admin', 'master_data', '{create,read,update,delete}', '{"scope": "all"}'::jsonb)
) AS perm(permission, resource, actions, conditions)
WHERE r.name = 'admin'
ON CONFLICT (role_id, permission, resource) DO NOTHING;

-- ============================================================================
-- 4. Seed test user: NIP 199203142014031001
-- ============================================================================

-- Create the user (password: hashed NIP as initial password)
-- The password_hash is a real Argon2id hash of '199203142014031001'
INSERT INTO users (id, realm_id, username, email, first_name, last_name, enabled, email_verified, password_hash, created_at, updated_at)
SELECT
    gen_random_uuid(),
    r.id,
    '199203142014031001',
    '199203142014031001@kejaksaan.go.id',
    'Admin',
    'Perlengkapan',
    true,
    true,
    '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M',
    NOW(),
    NOW()
FROM realms r
WHERE r.name = 'master'
ON CONFLICT (username) DO UPDATE SET
    enabled = true,
    password_hash = EXCLUDED.password_hash,
    updated_at = NOW();

-- Set password credential (Argon2id hash of '199203142014031001')
-- In production, this would be properly hashed by the auth service
INSERT INTO credentials (id, user_id, credential_type, secret_data, credential_data, created_at)
SELECT
    gen_random_uuid(),
    u.id,
    'password',
    -- Real Argon2id hash of '199203142014031001' (m=65536, t=3, p=4)
    '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M',
    '{"algorithm": "argon2id", "hash_iterations": 3}',
    NOW()
FROM users u
WHERE u.username = '199203142014031001'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 5. Assign ALL perlengkapan roles to the seed user
-- ============================================================================

-- Assign operator_satker
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = '199203142014031001'
  AND r.name = 'operator_satker'
ON CONFLICT DO NOTHING;

-- Assign validator_wilayah
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = '199203142014031001'
  AND r.name = 'validator_wilayah'
ON CONFLICT DO NOTHING;

-- Assign validator_pusat
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = '199203142014031001'
  AND r.name = 'validator_pusat'
ON CONFLICT DO NOTHING;

-- Assign admin
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = '199203142014031001'
  AND r.name = 'admin'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 6. Create user attributes for MySIMKARI integration
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_attributes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    value TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_user_attributes_user ON user_attributes(user_id);
CREATE INDEX IF NOT EXISTS idx_user_attributes_name ON user_attributes(name);

INSERT INTO user_attributes (user_id, name, value)
SELECT u.id, attr.name, attr.value
FROM users u
CROSS JOIN (VALUES
    ('nip', '199203142014031001'),
    ('satker_code', '0100000'),  -- Kejaksaan Agung
    ('satker_name', 'Kejaksaan Agung Republik Indonesia'),
    ('jabatan', 'Kasubag Perlengkapan'),
    ('golongan', 'III/c'),
    ('eselon', '4'),
    ('pangkat', 'Penata'),
    ('unit_kerja', 'Biro Umum - Bagian Perlengkapan'),
    ('active_role', 'admin'),  -- Current active role
    ('available_roles', 'operator_satker,validator_wilayah,validator_pusat,admin')
) AS attr(name, value)
WHERE u.username = '199203142014031001'
ON CONFLICT (user_id, name) DO UPDATE SET value = EXCLUDED.value;

-- ============================================================================
-- 7. Create view for user role resolution
-- ============================================================================

CREATE OR REPLACE VIEW v_user_perlengkapan_roles AS
SELECT
    u.id AS user_id,
    u.username,
    u.email,
    u.first_name,
    u.last_name,
    r.name AS role_name,
    r.description AS role_description,
    ua_satker.value AS satker_code,
    ua_satker_name.value AS satker_name,
    ua_active.value AS active_role
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id
LEFT JOIN user_attributes ua_satker ON u.id = ua_satker.user_id AND ua_satker.name = 'satker_code'
LEFT JOIN user_attributes ua_satker_name ON u.id = ua_satker_name.user_id AND ua_satker_name.name = 'satker_name'
LEFT JOIN user_attributes ua_active ON u.id = ua_active.user_id AND ua_active.name = 'active_role'
WHERE r.name IN ('operator_satker', 'validator_wilayah', 'validator_pusat', 'admin');

COMMIT;
