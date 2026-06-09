-- ============================================================================
-- MIGRATION 042: Dynamic Role System
-- ============================================================================
-- This migration implements a fully dynamic, database-driven role and permission
-- system following Vault and Keycloak best practices. All roles, permissions,
-- and levels are now configurable without code changes.
--
-- Key Principles:
-- 1. No hardcoded role names in application code
-- 2. Roles are defined via policies stored in database
-- 3. Permissions are granular and composable
-- 4. Hierarchies are flexible and configurable
-- 5. Only bootstrap admin is hardcoded (required for initial setup)
-- ============================================================================

-- ============================================================================
-- 1. ROLE TYPE REGISTRY (Replaces hardcoded enums)
-- ============================================================================

-- F5-B: views/functions below filter `roles.deleted_at IS NULL`, but the roles
-- table (001_initial_schema) has no deleted_at column. Add it (soft-delete
-- support) before those objects are created. Idempotent.
ALTER TABLE roles ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Role types table (replaces OrganizationRole enum)
CREATE TABLE IF NOT EXISTS role_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL DEFAULT 'custom',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_assignable BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_role_types_code ON role_types(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_role_types_category ON role_types(category) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_role_types_realm_id ON role_types(realm_id) WHERE deleted_at IS NULL;

COMMENT ON TABLE role_types IS 'Dynamic role type registry - replaces hardcoded role enums';
COMMENT ON COLUMN role_types.is_system IS 'System roles cannot be deleted but can be modified';
COMMENT ON COLUMN role_types.is_assignable IS 'Whether this role type can be assigned to users';
COMMENT ON COLUMN role_types.priority IS 'Higher priority roles take precedence in conflicts';

-- ============================================================================
-- 2. ACCESS LEVEL REGISTRY (Replaces AccessLevel enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS access_levels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    numeric_level INTEGER NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    capabilities JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(code, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_access_levels_code ON access_levels(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_access_levels_numeric ON access_levels(numeric_level DESC) WHERE deleted_at IS NULL;

COMMENT ON TABLE access_levels IS 'Dynamic access level registry - replaces AccessLevel enum';
COMMENT ON COLUMN access_levels.numeric_level IS 'Higher number = more privileges';
COMMENT ON COLUMN access_levels.capabilities IS 'JSON array of capability codes this level grants';

-- ============================================================================
-- 3. ADMIN LEVEL HIERARCHY (Replaces AdminLevel enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS admin_level_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    hierarchy_level INTEGER NOT NULL,
    scope_type VARCHAR(100) NOT NULL,
    parent_level_id UUID REFERENCES admin_level_types(id) ON DELETE SET NULL,
    can_manage_levels JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(code, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_admin_level_types_code ON admin_level_types(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_admin_level_types_parent ON admin_level_types(parent_level_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_admin_level_types_hierarchy ON admin_level_types(hierarchy_level DESC) WHERE deleted_at IS NULL;

COMMENT ON TABLE admin_level_types IS 'Dynamic admin level hierarchy - replaces AdminLevel enum';
COMMENT ON COLUMN admin_level_types.hierarchy_level IS 'Higher number = higher authority';
COMMENT ON COLUMN admin_level_types.scope_type IS 'Type of scope this level manages (satker, wilayah, pusat, etc)';
COMMENT ON COLUMN admin_level_types.can_manage_levels IS 'JSON array of level codes this level can manage';

-- ============================================================================
-- 4. SCOPE TYPE REGISTRY (Replaces RoleScope enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS scope_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    hierarchy_level INTEGER NOT NULL DEFAULT 0,
    parent_scope_type_id UUID REFERENCES scope_types(id) ON DELETE SET NULL,
    scope_pattern VARCHAR(500),
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(code, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_scope_types_code ON scope_types(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_scope_types_parent ON scope_types(parent_scope_type_id) WHERE deleted_at IS NULL;

COMMENT ON TABLE scope_types IS 'Dynamic scope type registry - replaces RoleScope enum';
COMMENT ON COLUMN scope_types.scope_pattern IS 'Regex pattern for matching scope identifiers';

-- ============================================================================
-- 5. CAPABILITY REGISTRY (Fine-grained permissions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS capabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(100),
    action VARCHAR(100) NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_dangerous BOOLEAN NOT NULL DEFAULT FALSE,
    requires_mfa BOOLEAN NOT NULL DEFAULT FALSE,
    requires_approval BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(code, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_capabilities_code ON capabilities(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_capabilities_resource_type ON capabilities(resource_type) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_capabilities_action ON capabilities(action) WHERE deleted_at IS NULL;

COMMENT ON TABLE capabilities IS 'Fine-grained capabilities/permissions registry';
COMMENT ON COLUMN capabilities.code IS 'Unique capability code (e.g., users:read, secrets:write)';
COMMENT ON COLUMN capabilities.resource_type IS 'Resource type this capability applies to';
COMMENT ON COLUMN capabilities.action IS 'Action type (read, write, delete, admin, etc)';
COMMENT ON COLUMN capabilities.is_dangerous IS 'Dangerous capabilities require extra confirmation';
COMMENT ON COLUMN capabilities.requires_mfa IS 'Capability requires MFA verification';
COMMENT ON COLUMN capabilities.requires_approval IS 'Capability requires approval workflow';

-- ============================================================================
-- 6. ROLE-CAPABILITY MAPPING
-- ============================================================================

CREATE TABLE IF NOT EXISTS role_capabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    capability_id UUID NOT NULL REFERENCES capabilities(id) ON DELETE CASCADE,
    conditions JSONB NOT NULL DEFAULT '{}',
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(role_id, capability_id)
);

CREATE INDEX IF NOT EXISTS idx_role_capabilities_role_id ON role_capabilities(role_id);
CREATE INDEX IF NOT EXISTS idx_role_capabilities_capability_id ON role_capabilities(capability_id);

COMMENT ON TABLE role_capabilities IS 'Mapping of roles to their granted capabilities';
COMMENT ON COLUMN role_capabilities.conditions IS 'JSON conditions for conditional capability grants';

-- ============================================================================
-- 7. POLICY ENGINE (Vault-style policies)
-- ============================================================================

CREATE TABLE IF NOT EXISTS authorization_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    policy_type VARCHAR(50) NOT NULL DEFAULT 'acl',
    effect VARCHAR(20) NOT NULL DEFAULT 'allow' CHECK (effect IN ('allow', 'deny')),
    path_pattern VARCHAR(1000) NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '[]',
    conditions JSONB NOT NULL DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 0,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(name, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_auth_policies_name ON authorization_policies(name) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_auth_policies_path ON authorization_policies(path_pattern) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_auth_policies_priority ON authorization_policies(priority DESC) WHERE deleted_at IS NULL;

COMMENT ON TABLE authorization_policies IS 'Vault-style authorization policies';
COMMENT ON COLUMN authorization_policies.path_pattern IS 'Glob pattern for resource paths';
COMMENT ON COLUMN authorization_policies.capabilities IS 'JSON array of capability codes granted by this policy';
COMMENT ON COLUMN authorization_policies.conditions IS 'JSON conditions for policy evaluation';

-- ============================================================================
-- 8. ROLE-POLICY MAPPING
-- ============================================================================

CREATE TABLE IF NOT EXISTS role_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES authorization_policies(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(role_id, policy_id)
);

CREATE INDEX IF NOT EXISTS idx_role_policies_role_id ON role_policies(role_id);
CREATE INDEX IF NOT EXISTS idx_role_policies_policy_id ON role_policies(policy_id);

COMMENT ON TABLE role_policies IS 'Mapping of roles to authorization policies';

-- ============================================================================
-- 9. USER-POLICY DIRECT MAPPING (for explicit policy grants)
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES authorization_policies(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, policy_id)
);

CREATE INDEX IF NOT EXISTS idx_user_policies_user_id ON user_policies(user_id);
CREATE INDEX IF NOT EXISTS idx_user_policies_policy_id ON user_policies(policy_id);

COMMENT ON TABLE user_policies IS 'Direct user-to-policy mappings for explicit grants';

-- ============================================================================
-- 10. ACTOR TYPES REGISTRY (Replaces hardcoded actor_type checks)
-- ============================================================================

CREATE TABLE IF NOT EXISTS actor_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_human BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE actor_types IS 'Dynamic actor type registry - replaces hardcoded actor_type strings';
COMMENT ON COLUMN actor_types.is_human IS 'Whether this actor type represents human users';

-- ============================================================================
-- 11. CREDENTIAL TYPE REGISTRY (Replaces CredentialType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS credential_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    requires_verification BOOLEAN NOT NULL DEFAULT TRUE,
    config_schema JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(code, realm_id)
);

CREATE INDEX IF NOT EXISTS idx_credential_types_code ON credential_types(code) WHERE deleted_at IS NULL;

COMMENT ON TABLE credential_types IS 'Dynamic credential type registry - replaces CredentialType enum';
COMMENT ON COLUMN credential_types.config_schema IS 'JSON schema for credential configuration';

-- ============================================================================
-- 12. THEME TYPE REGISTRY (Replaces ThemeType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS theme_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    default_template TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE theme_types IS 'Dynamic theme type registry - replaces ThemeType enum';

-- ============================================================================
-- 13. SATKER TYPE REGISTRY (Replaces SatkerType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS satker_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    hierarchy_level INTEGER NOT NULL DEFAULT 0,
    parent_type_id UUID REFERENCES satker_types(id) ON DELETE SET NULL,
    code_pattern VARCHAR(100),
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_satker_types_hierarchy ON satker_types(hierarchy_level DESC);
CREATE INDEX IF NOT EXISTS idx_satker_types_parent ON satker_types(parent_type_id);

COMMENT ON TABLE satker_types IS 'Dynamic Satker type hierarchy - replaces SatkerType enum';
COMMENT ON COLUMN satker_types.code_pattern IS 'Regex pattern for satker codes of this type';

-- ============================================================================
-- 14. UPDATE organization_members TO USE DYNAMIC ROLES
-- ============================================================================

-- Add role_type_id column to organization_members
ALTER TABLE organization_members
    ADD COLUMN IF NOT EXISTS role_type_id UUID REFERENCES role_types(id) ON DELETE SET NULL;

-- Create migration trigger to populate role_type_id from legacy role column
-- This will be handled by application during migration period

-- Remove the hardcoded CHECK constraint and add foreign key
DO $$
BEGIN
    -- Drop the old CHECK constraint if it exists
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'organization_members_role_check'
        AND conrelid = 'organization_members'::regclass
    ) THEN
        ALTER TABLE organization_members DROP CONSTRAINT organization_members_role_check;
    END IF;
EXCEPTION
    WHEN undefined_object THEN NULL;
END $$;

-- ============================================================================
-- 15. ROLE HIERARCHY FOR COMPOSITE ROLES
-- ============================================================================

CREATE TABLE IF NOT EXISTS role_hierarchy (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    child_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(parent_role_id, child_role_id),
    CHECK(parent_role_id != child_role_id)
);

CREATE INDEX IF NOT EXISTS idx_role_hierarchy_parent ON role_hierarchy(parent_role_id);
CREATE INDEX IF NOT EXISTS idx_role_hierarchy_child ON role_hierarchy(child_role_id);

COMMENT ON TABLE role_hierarchy IS 'Role composition - parent roles inherit child role capabilities';

-- ============================================================================
-- 16. BOOTSTRAP SYSTEM DATA (Required defaults only)
-- ============================================================================
-- Following best practices: Only absolutely required bootstrap data is hardcoded.
-- Everything else should be configured via admin console or config files.

-- Bootstrap actor types (required for audit logging)
INSERT INTO actor_types (code, name, description, is_system, is_human)
VALUES
    ('user', 'Human User', 'Regular human user', TRUE, TRUE),
    ('system', 'System', 'System-initiated actions', TRUE, FALSE),
    ('service', 'Service Account', 'Automated service account', TRUE, FALSE),
    ('automated', 'Automated Process', 'Automated background process', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap access levels (minimum required for operation)
INSERT INTO access_levels (code, name, description, numeric_level, is_system, capabilities, realm_id)
VALUES
    ('deny', 'No Access', 'Explicitly denied access', 0, TRUE, '[]'::jsonb, NULL),
    ('read', 'Read Only', 'Read-only access', 10, TRUE, '["read"]'::jsonb, NULL),
    ('write', 'Read/Write', 'Read and write access', 20, TRUE, '["read", "write"]'::jsonb, NULL),
    ('manage', 'Management', 'Full management access', 30, TRUE, '["read", "write", "delete", "manage"]'::jsonb, NULL),
    ('root', 'Root Access', 'Unrestricted root access', 100, TRUE, '["*"]'::jsonb, NULL)
ON CONFLICT (code, realm_id) DO NOTHING;

-- Bootstrap scope types (minimum required for operation)
INSERT INTO scope_types (code, name, description, hierarchy_level, is_system, realm_id)
VALUES
    ('global', 'Global', 'Global scope - applies everywhere', 100, TRUE, NULL),
    ('realm', 'Realm', 'Realm-level scope', 90, TRUE, NULL),
    ('organization', 'Organization', 'Organization-level scope', 80, TRUE, NULL),
    ('group', 'Group', 'Group-level scope', 70, TRUE, NULL),
    ('user', 'User', 'User-level scope', 10, TRUE, NULL)
ON CONFLICT (code, realm_id) DO NOTHING;

-- Bootstrap admin level types (minimum for Kejaksaan hierarchy)
INSERT INTO admin_level_types (code, name, description, hierarchy_level, scope_type, is_system, realm_id)
VALUES
    ('pusat', 'Admin Pusat', 'Central/national level administrator', 100, 'global', TRUE, NULL),
    ('eselon_i', 'Admin Eselon I', 'Eselon I level administrator', 80, 'organization', TRUE, NULL),
    ('wilayah', 'Admin Wilayah', 'Regional area administrator', 60, 'group', TRUE, NULL),
    ('satker', 'Admin Satker', 'Work unit administrator', 40, 'group', TRUE, NULL)
ON CONFLICT (code, realm_id) DO NOTHING;

-- Bootstrap role types (minimum required for operation)
INSERT INTO role_types (code, name, description, category, is_system, priority)
VALUES
    ('owner', 'Owner', 'Full ownership rights', 'ownership', TRUE, 100),
    ('admin', 'Administrator', 'Administrative privileges', 'administrative', TRUE, 90),
    ('manager', 'Manager', 'Management privileges', 'management', TRUE, 80),
    ('member', 'Member', 'Standard member access', 'membership', TRUE, 50),
    ('viewer', 'Viewer', 'Read-only access', 'access', TRUE, 30),
    ('guest', 'Guest', 'Limited guest access', 'access', TRUE, 10)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap credential types
INSERT INTO credential_types (code, name, description, is_system, is_primary, requires_verification, realm_id)
VALUES
    ('password', 'Password', 'Password-based authentication', TRUE, TRUE, FALSE, NULL),
    ('totp', 'TOTP', 'Time-based One-Time Password', TRUE, FALSE, TRUE, NULL),
    ('webauthn', 'WebAuthn', 'WebAuthn/FIDO2 authentication', TRUE, FALSE, TRUE, NULL),
    ('recovery_code', 'Recovery Code', 'Backup recovery codes', TRUE, FALSE, FALSE, NULL),
    ('magic_link', 'Magic Link', 'Email-based passwordless login', TRUE, FALSE, TRUE, NULL)
ON CONFLICT (code, realm_id) DO NOTHING;

-- Bootstrap theme types
INSERT INTO theme_types (code, name, description, is_system)
VALUES
    ('login', 'Login', 'Login page theme', TRUE),
    ('account', 'Account', 'Account management theme', TRUE),
    ('admin', 'Admin', 'Admin console theme', TRUE),
    ('email', 'Email', 'Email template theme', TRUE)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap satker types for Kejaksaan hierarchy
INSERT INTO satker_types (code, name, description, hierarchy_level, code_pattern, is_system)
VALUES
    ('pusat', 'Pusat', 'Central office', 100, '^KP_.*', TRUE),
    ('kejati', 'Kejaksaan Tinggi', 'High Prosecutor Office', 80, '^KT_.*', TRUE),
    ('kejari', 'Kejaksaan Negeri', 'District Prosecutor Office', 60, '^KN_.*', TRUE),
    ('cabang', 'Cabang', 'Branch office', 40, '^CB_.*', TRUE),
    ('unit_khusus', 'Unit Khusus', 'Special unit', 20, '^UK_.*', TRUE)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap core capabilities (Vault-style)
INSERT INTO capabilities (code, name, description, resource_type, action, is_system, realm_id)
VALUES
    -- Universal capabilities
    ('*', 'Superuser', 'All capabilities (root)', NULL, '*', TRUE, NULL),

    -- User management capabilities
    ('users:read', 'Read Users', 'View user information', 'user', 'read', TRUE, NULL),
    ('users:write', 'Write Users', 'Create and update users', 'user', 'write', TRUE, NULL),
    ('users:delete', 'Delete Users', 'Delete users', 'user', 'delete', TRUE, NULL),
    ('users:admin', 'Admin Users', 'Full user administration', 'user', 'admin', TRUE, NULL),

    -- Role management capabilities
    ('roles:read', 'Read Roles', 'View role information', 'role', 'read', TRUE, NULL),
    ('roles:write', 'Write Roles', 'Create and update roles', 'role', 'write', TRUE, NULL),
    ('roles:delete', 'Delete Roles', 'Delete roles', 'role', 'delete', TRUE, NULL),
    ('roles:assign', 'Assign Roles', 'Assign roles to users', 'role', 'assign', TRUE, NULL),

    -- Policy management capabilities
    ('policies:read', 'Read Policies', 'View policies', 'policy', 'read', TRUE, NULL),
    ('policies:write', 'Write Policies', 'Create and update policies', 'policy', 'write', TRUE, NULL),
    ('policies:delete', 'Delete Policies', 'Delete policies', 'policy', 'delete', TRUE, NULL),

    -- Realm management capabilities
    ('realms:read', 'Read Realms', 'View realm configuration', 'realm', 'read', TRUE, NULL),
    ('realms:write', 'Write Realms', 'Modify realm configuration', 'realm', 'write', TRUE, NULL),
    ('realms:delete', 'Delete Realms', 'Delete realms', 'realm', 'delete', TRUE, NULL),

    -- Client management capabilities
    ('clients:read', 'Read Clients', 'View OAuth2 clients', 'client', 'read', TRUE, NULL),
    ('clients:write', 'Write Clients', 'Create and update clients', 'client', 'write', TRUE, NULL),
    ('clients:delete', 'Delete Clients', 'Delete clients', 'client', 'delete', TRUE, NULL),

    -- Audit capabilities
    ('audit:read', 'View Audit Logs', 'View audit logs', 'audit', 'read', TRUE, NULL),
    ('audit:export', 'Export Audit Logs', 'Export audit logs', 'audit', 'export', TRUE, NULL),

    -- Configuration capabilities
    ('config:read', 'Read Config', 'View system configuration', 'config', 'read', TRUE, NULL),
    ('config:write', 'Write Config', 'Modify system configuration', 'config', 'write', TRUE, NULL),
    ('config:delete', 'Delete Config', 'Delete configuration entries', 'config', 'delete', TRUE, NULL),
    ('config:reload', 'Reload Config', 'Hot reload configuration', 'config', 'admin', TRUE, NULL),

    -- MFA capabilities
    ('mfa:manage', 'Manage MFA', 'Manage MFA settings', 'mfa', 'manage', TRUE, NULL),
    ('mfa:admin', 'Admin MFA', 'Full MFA administration', 'mfa', 'admin', TRUE, NULL),
    ('mfa:bypass', 'Bypass MFA', 'Reset and bypass user MFA', 'mfa', 'bypass', TRUE, NULL),
    ('mfa:view', 'View MFA', 'View MFA status', 'mfa', 'read', TRUE, NULL),
    ('mfa:reset', 'Reset MFA', 'Reset user MFA', 'mfa', 'reset', TRUE, NULL),

    -- Secrets capabilities (for Secreton integration)
    ('secrets:read', 'Read Secrets', 'Read secrets from vault', 'secret', 'read', TRUE, NULL),
    ('secrets:write', 'Write Secrets', 'Write secrets to vault', 'secret', 'write', TRUE, NULL),
    ('secrets:delete', 'Delete Secrets', 'Delete secrets from vault', 'secret', 'delete', TRUE, NULL),
    ('secrets:admin', 'Admin Secrets', 'Full secrets administration', 'secret', 'admin', TRUE, NULL),

    -- System capabilities
    ('system:admin', 'System Admin', 'Full system administration (superuser)', 'system', 'admin', TRUE, NULL),
    ('system:health', 'System Health', 'View system health metrics', 'system', 'read', TRUE, NULL),

    -- Session capabilities
    ('sessions:read', 'Read Sessions', 'View active sessions', 'session', 'read', TRUE, NULL),
    ('sessions:revoke', 'Revoke Sessions', 'Terminate user sessions', 'session', 'revoke', TRUE, NULL)
ON CONFLICT (code, realm_id) DO NOTHING;

-- Bootstrap default policies
INSERT INTO authorization_policies (name, description, policy_type, effect, path_pattern, capabilities, priority, is_system, realm_id)
VALUES
    ('root', 'Root policy - full access', 'acl', 'allow', '*', '["*"]'::jsonb, 1000, TRUE, NULL),
    ('default', 'Default read-only policy', 'acl', 'allow', 'self/*', '["read"]'::jsonb, 10, TRUE, NULL),
    ('deny-all', 'Explicit deny-all policy', 'acl', 'deny', '*', '["*"]'::jsonb, 0, TRUE, NULL)
ON CONFLICT (name, realm_id) DO NOTHING;

-- ============================================================================
-- 17. VIEWS FOR EASY QUERYING
-- ============================================================================

-- View to get effective user capabilities
CREATE OR REPLACE VIEW v_user_effective_capabilities AS
SELECT DISTINCT
    u.id AS user_id,
    u.username,
    c.id AS capability_id,
    c.code AS capability_code,
    c.resource_type,
    c.action,
    r.id AS role_id,
    r.name AS role_name,
    'role' AS grant_source
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id
JOIN role_capabilities rc ON r.id = rc.role_id
JOIN capabilities c ON rc.capability_id = c.id
WHERE u.deleted_at IS NULL
    AND r.deleted_at IS NULL
    AND c.deleted_at IS NULL
    AND (rc.expires_at IS NULL OR rc.expires_at > NOW())

UNION

-- Capabilities from direct policy assignments
SELECT DISTINCT
    u.id AS user_id,
    u.username,
    c.id AS capability_id,
    c.code AS capability_code,
    c.resource_type,
    c.action,
    NULL::uuid AS role_id,
    NULL::varchar AS role_name,
    'policy' AS grant_source
FROM users u
JOIN user_policies up ON u.id = up.user_id
JOIN authorization_policies ap ON up.policy_id = ap.id
CROSS JOIN LATERAL jsonb_array_elements_text(ap.capabilities) AS cap_code
JOIN capabilities c ON c.code = cap_code
WHERE u.deleted_at IS NULL
    AND ap.deleted_at IS NULL
    AND ap.enabled = TRUE
    AND c.deleted_at IS NULL
    AND (up.expires_at IS NULL OR up.expires_at > NOW());

COMMENT ON VIEW v_user_effective_capabilities IS 'Consolidated view of all user capabilities from roles and policies';

-- View for role hierarchy with capabilities
CREATE OR REPLACE VIEW v_role_hierarchy_capabilities AS
WITH RECURSIVE role_tree AS (
    -- Base case: direct role capabilities
    SELECT
        r.id AS role_id,
        r.name AS role_name,
        c.code AS capability_code,
        0 AS depth
    FROM roles r
    JOIN role_capabilities rc ON r.id = rc.role_id
    JOIN capabilities c ON rc.capability_id = c.id
    WHERE r.deleted_at IS NULL

    UNION ALL

    -- Recursive case: inherited capabilities from parent roles
    SELECT
        rh.child_role_id AS role_id,
        cr.name AS role_name,
        rt.capability_code,
        rt.depth + 1
    FROM role_hierarchy rh
    JOIN role_tree rt ON rh.parent_role_id = rt.role_id
    JOIN roles cr ON rh.child_role_id = cr.id
    WHERE cr.deleted_at IS NULL
    AND rt.depth < 10  -- Prevent infinite recursion
)
SELECT DISTINCT
    role_id,
    role_name,
    capability_code,
    MIN(depth) AS inheritance_depth
FROM role_tree
GROUP BY role_id, role_name, capability_code;

COMMENT ON VIEW v_role_hierarchy_capabilities IS 'View showing all capabilities for roles including inherited ones';

-- ============================================================================
-- 18. HELPER FUNCTIONS
-- ============================================================================

-- Function to check if user has a specific capability
CREATE OR REPLACE FUNCTION user_has_capability(
    p_user_id UUID,
    p_capability_code VARCHAR(100)
) RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM v_user_effective_capabilities
        WHERE user_id = p_user_id
        AND (capability_code = p_capability_code OR capability_code = '*')
    );
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION user_has_capability IS 'Check if user has a specific capability';

-- Function to check if user can access a resource path (Vault-style)
CREATE OR REPLACE FUNCTION user_can_access_path(
    p_user_id UUID,
    p_path VARCHAR(1000),
    p_action VARCHAR(100)
) RETURNS BOOLEAN AS $$
DECLARE
    v_allowed BOOLEAN := FALSE;
    v_denied BOOLEAN := FALSE;
BEGIN
    -- Check deny policies first (highest priority)
    SELECT TRUE INTO v_denied
    FROM user_policies up
    JOIN authorization_policies ap ON up.policy_id = ap.id
    WHERE up.user_id = p_user_id
    AND ap.effect = 'deny'
    AND ap.enabled = TRUE
    AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
    AND (
        ap.capabilities @> to_jsonb(p_action)::jsonb
        OR ap.capabilities @> '["*"]'::jsonb
    )
    AND (up.expires_at IS NULL OR up.expires_at > NOW())
    LIMIT 1;

    IF v_denied THEN
        RETURN FALSE;
    END IF;

    -- Check allow policies
    SELECT TRUE INTO v_allowed
    FROM (
        -- From role policies
        SELECT ap.id
        FROM user_roles ur
        JOIN role_policies rp ON ur.role_id = rp.role_id
        JOIN authorization_policies ap ON rp.policy_id = ap.id
        WHERE ur.user_id = p_user_id
        AND ap.effect = 'allow'
        AND ap.enabled = TRUE
        AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
        AND (
            ap.capabilities @> to_jsonb(p_action)::jsonb
            OR ap.capabilities @> '["*"]'::jsonb
        )
        AND (rp.expires_at IS NULL OR rp.expires_at > NOW())

        UNION

        -- From direct user policies
        SELECT ap.id
        FROM user_policies up
        JOIN authorization_policies ap ON up.policy_id = ap.id
        WHERE up.user_id = p_user_id
        AND ap.effect = 'allow'
        AND ap.enabled = TRUE
        AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
        AND (
            ap.capabilities @> to_jsonb(p_action)::jsonb
            OR ap.capabilities @> '["*"]'::jsonb
        )
        AND (up.expires_at IS NULL OR up.expires_at > NOW())
    ) allowed_policies
    LIMIT 1;

    RETURN COALESCE(v_allowed, FALSE);
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION user_can_access_path IS 'Check if user can access a path with given action (Vault-style)';

-- ============================================================================
-- 19. TRIGGERS FOR UPDATED_AT
-- ============================================================================

DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN SELECT unnest(ARRAY[
        'role_types', 'access_levels', 'admin_level_types', 'scope_types',
        'capabilities', 'authorization_policies', 'credential_types',
        'theme_types', 'satker_types'
    ])
    LOOP
        EXECUTE format('
            DROP TRIGGER IF EXISTS update_%s_updated_at ON %s;
            CREATE TRIGGER update_%s_updated_at
            BEFORE UPDATE ON %s
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
        ', t, t, t, t);
    END LOOP;
END $$;

-- ============================================================================
-- 20. ASSIGN ROOT POLICY TO BOOTSTRAP ADMIN
-- ============================================================================

-- Ensure bootstrap admin has root policy
INSERT INTO user_policies (user_id, policy_id, reason)
SELECT
    '00000000-0000-0000-0000-000000000001'::UUID,
    ap.id,
    'Bootstrap administrator - required for initial system configuration'
FROM authorization_policies ap
WHERE ap.name = 'root'
ON CONFLICT (user_id, policy_id) DO NOTHING;

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
