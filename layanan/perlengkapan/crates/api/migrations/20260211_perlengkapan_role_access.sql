-- Migration: 20260211_perlengkapan_role_access.sql
-- Purpose: Add role-based access control tables for perlengkapan domain
-- Date: 2026-02-11
--
-- Integrates with authenc roles via JWT claims.
-- Perlengkapan-local tables for caching user roles and tracking role switches.

BEGIN;

-- ============================================================================
-- 1. User role cache (synced from authenc via gRPC)
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan_user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    username VARCHAR(50) NOT NULL,
    nip VARCHAR(18) NOT NULL,
    role_name VARCHAR(50) NOT NULL CHECK (role_name IN (
        'operator_satker', 'validator_wilayah', 'validator_pusat', 'admin'
    )),
    satker_code VARCHAR(20),
    satker_name VARCHAR(255),
    is_active BOOLEAN DEFAULT false,  -- Current active role for this user
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, role_name)
);

CREATE INDEX idx_perlengkapan_user_roles_user ON perlengkapan_user_roles(user_id);
CREATE INDEX idx_perlengkapan_user_roles_username ON perlengkapan_user_roles(username);
CREATE INDEX idx_perlengkapan_user_roles_nip ON perlengkapan_user_roles(nip);
CREATE INDEX idx_perlengkapan_user_roles_active ON perlengkapan_user_roles(user_id, is_active) WHERE is_active = true;

-- ============================================================================
-- 2. Role switch audit log
-- ============================================================================

CREATE TABLE IF NOT EXISTS role_switch_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    username VARCHAR(50) NOT NULL,
    from_role VARCHAR(50),
    to_role VARCHAR(50) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    switched_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_role_switch_log_user ON role_switch_log(user_id);
CREATE INDEX idx_role_switch_log_time ON role_switch_log(switched_at DESC);

-- ============================================================================
-- 3. User management (admin panel) - cached from MySIMKARI
-- ============================================================================

CREATE TABLE IF NOT EXISTS perlengkapan_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nip VARCHAR(18) UNIQUE NOT NULL,
    nama VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    jabatan VARCHAR(255),
    golongan VARCHAR(10),
    pangkat VARCHAR(100),
    eselon VARCHAR(5),
    satker_code VARCHAR(20),
    satker_name VARCHAR(255),
    unit_kerja VARCHAR(255),
    jenis_kelamin VARCHAR(1),  -- L/P
    photo_url TEXT,
    status VARCHAR(20) DEFAULT 'active',
    last_login_at TIMESTAMPTZ,
    synced_from_mysimkari_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_perlengkapan_users_satker ON perlengkapan_users(satker_code);
CREATE INDEX idx_perlengkapan_users_status ON perlengkapan_users(status);

-- ============================================================================
-- 4. Role assignment view for admin panel
-- ============================================================================

CREATE OR REPLACE VIEW v_user_role_summary AS
SELECT
    pu.nip,
    pu.nama,
    pu.jabatan,
    pu.golongan,
    pu.satker_code,
    pu.satker_name,
    pu.status,
    pu.last_login_at,
    COALESCE(
        array_agg(DISTINCT pur.role_name) FILTER (WHERE pur.role_name IS NOT NULL),
        '{}'
    ) AS assigned_roles,
    (SELECT pur2.role_name FROM perlengkapan_user_roles pur2
     WHERE pur2.nip = pu.nip AND pur2.is_active = true LIMIT 1) AS active_role,
    COUNT(DISTINCT pur.role_name) AS role_count
FROM perlengkapan_users pu
LEFT JOIN perlengkapan_user_roles pur ON pu.nip = pur.nip
GROUP BY pu.nip, pu.nama, pu.jabatan, pu.golongan, pu.satker_code, pu.satker_name, pu.status, pu.last_login_at;

-- ============================================================================
-- 5. Seed the test user into perlengkapan
-- ============================================================================

INSERT INTO perlengkapan_users (nip, nama, email, jabatan, golongan, pangkat, eselon, satker_code, satker_name, unit_kerja, jenis_kelamin, status)
VALUES (
    '199203142014031001',
    'Admin',
    '199203142014031001@kejaksaan.go.id',
    'Kasubag Perlengkapan',
    'III/c',
    'Penata',
    '4',
    '0100000',
    'Kejaksaan Agung Republik Indonesia',
    'Biro Umum - Bagian Perlengkapan',
    'L',
    'active'
)
ON CONFLICT (nip) DO UPDATE SET
    nama = EXCLUDED.nama,
    updated_at = NOW();

-- Assign all roles to seed user
INSERT INTO perlengkapan_user_roles (user_id, username, nip, role_name, satker_code, satker_name, is_active)
SELECT
    gen_random_uuid(),
    '199203142014031001',
    '199203142014031001',
    role_name,
    '0100000',
    'Kejaksaan Agung Republik Indonesia',
    (role_name = 'admin')  -- admin is default active role
FROM unnest(ARRAY['operator_satker', 'validator_wilayah', 'validator_pusat', 'admin']) AS role_name
ON CONFLICT (user_id, role_name) DO NOTHING;

-- ============================================================================
-- 6. Role-scoped menu configuration
-- ============================================================================

CREATE TABLE IF NOT EXISTS role_menu_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_name VARCHAR(50) NOT NULL,
    menu_key VARCHAR(100) NOT NULL,
    menu_label VARCHAR(255) NOT NULL,
    menu_icon VARCHAR(100),
    menu_path VARCHAR(255) NOT NULL,
    parent_key VARCHAR(100),
    sort_order INT DEFAULT 0,
    is_visible BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(role_name, menu_key)
);

-- Admin-only menus
INSERT INTO role_menu_config (role_name, menu_key, menu_label, menu_icon, menu_path, sort_order) VALUES
    ('admin', 'admin_users', 'Manajemen Pengguna', 'fas fa-users-cog', '/dashboard/admin/users', 100),
    ('admin', 'admin_roles', 'Pengaturan Role', 'fas fa-user-shield', '/dashboard/admin/roles', 101),
    ('admin', 'admin_config', 'Konfigurasi Sistem', 'fas fa-cogs', '/dashboard/admin/config', 102),
    ('admin', 'admin_audit', 'Audit Log', 'fas fa-clipboard-list', '/dashboard/admin/audit', 103),
    ('admin', 'admin_master', 'Master Data', 'fas fa-database', '/dashboard/admin/master', 104)
ON CONFLICT DO NOTHING;

COMMIT;
