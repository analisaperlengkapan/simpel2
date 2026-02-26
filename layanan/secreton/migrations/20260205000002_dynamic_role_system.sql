-- ============================================================================
-- MIGRATION: Dynamic Role System for Secreton
-- ============================================================================
-- This migration implements a fully dynamic, database-driven role and permission
-- system following HashiCorp Vault best practices. All roles, permissions,
-- and policies are now configurable without code changes.
--
-- Key Principles:
-- 1. Policy-as-Code (HCL/JSON format support)
-- 2. Path-based access control (glob patterns)
-- 3. Capability-based permissions
-- 4. Dynamic secrets engine roles
-- 5. Token types and policies
-- ============================================================================

-- Use secreton schema
SET search_path TO secreton, public;

-- ============================================================================
-- 1. CAPABILITY REGISTRY (Replaces Permission enum in auth.rs)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.capabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL DEFAULT 'general',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_dangerous BOOLEAN NOT NULL DEFAULT FALSE,
    requires_sudo BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secreton_capabilities_code ON secreton.capabilities(code);
CREATE INDEX IF NOT EXISTS idx_secreton_capabilities_category ON secreton.capabilities(category);

COMMENT ON TABLE secreton.capabilities IS 'Dynamic capability registry - replaces Permission enum';
COMMENT ON COLUMN secreton.capabilities.is_dangerous IS 'Dangerous capabilities require extra confirmation';
COMMENT ON COLUMN secreton.capabilities.requires_sudo IS 'Capability requires sudo token';

-- ============================================================================
-- 2. USER ROLE REGISTRY (Replaces UserRole enum in auth.rs)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.user_role_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    default_capabilities JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secreton_user_role_types_code ON secreton.user_role_types(code);

COMMENT ON TABLE secreton.user_role_types IS 'Dynamic user role registry - replaces UserRole enum';
COMMENT ON COLUMN secreton.user_role_types.default_capabilities IS 'JSON array of capability codes granted by default';

-- ============================================================================
-- 3. AUTH METHOD TYPES (Replaces AuthMethodType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.auth_method_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_external BOOLEAN NOT NULL DEFAULT FALSE,
    config_schema JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE secreton.auth_method_types IS 'Dynamic auth method registry - replaces AuthMethodType enum';
COMMENT ON COLUMN secreton.auth_method_types.is_external IS 'Whether this auth method uses external identity provider';

-- ============================================================================
-- 4. TOKEN TYPES (Replaces TokenType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.token_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_renewable BOOLEAN NOT NULL DEFAULT TRUE,
    is_orphan_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    max_ttl_seconds INTEGER,
    default_ttl_seconds INTEGER NOT NULL DEFAULT 3600,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE secreton.token_types IS 'Dynamic token type registry - replaces TokenType enum';

-- ============================================================================
-- 5. POLICY REGISTRY (Enhanced from existing policies table)
-- ============================================================================

-- Add new columns to existing policies table if not exists
DO $$
BEGIN
    -- Add policy_format column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'secreton'
                   AND table_name = 'policies'
                   AND column_name = 'policy_format') THEN
        ALTER TABLE secreton.policies ADD COLUMN policy_format VARCHAR(20) NOT NULL DEFAULT 'json';
    END IF;

    -- Add is_system column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'secreton'
                   AND table_name = 'policies'
                   AND column_name = 'is_system') THEN
        ALTER TABLE secreton.policies ADD COLUMN is_system BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;

    -- Add priority column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'secreton'
                   AND table_name = 'policies'
                   AND column_name = 'priority') THEN
        ALTER TABLE secreton.policies ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
    END IF;

    -- Add enabled column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_schema = 'secreton'
                   AND table_name = 'policies'
                   AND column_name = 'enabled') THEN
        ALTER TABLE secreton.policies ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT TRUE;
    END IF;
END $$;

-- Create policy rules table for structured rules
CREATE TABLE IF NOT EXISTS secreton.policy_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id INTEGER NOT NULL REFERENCES secreton.policies(id) ON DELETE CASCADE,
    effect VARCHAR(20) NOT NULL DEFAULT 'allow' CHECK (effect IN ('allow', 'deny')),
    path_pattern VARCHAR(1000) NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '[]',
    required_parameters JSONB NOT NULL DEFAULT '{}',
    allowed_parameters JSONB NOT NULL DEFAULT '{}',
    denied_parameters JSONB NOT NULL DEFAULT '{}',
    min_wrapping_ttl INTEGER,
    max_wrapping_ttl INTEGER,
    conditions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_rules_policy_id ON secreton.policy_rules(policy_id);
CREATE INDEX IF NOT EXISTS idx_policy_rules_path ON secreton.policy_rules(path_pattern);

COMMENT ON TABLE secreton.policy_rules IS 'Individual rules within a policy (Vault-style)';
COMMENT ON COLUMN secreton.policy_rules.path_pattern IS 'Glob pattern for path matching (e.g., secret/data/*)';
COMMENT ON COLUMN secreton.policy_rules.capabilities IS 'JSON array of capabilities granted on this path';

-- ============================================================================
-- 6. USER-ROLE MAPPING
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_type_id UUID NOT NULL REFERENCES secreton.user_role_types(id) ON DELETE CASCADE,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    granted_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    UNIQUE(user_id, role_type_id, namespace)
);

CREATE INDEX IF NOT EXISTS idx_secreton_user_roles_user_id ON secreton.user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_secreton_user_roles_role_type ON secreton.user_roles(role_type_id);
CREATE INDEX IF NOT EXISTS idx_secreton_user_roles_namespace ON secreton.user_roles(namespace);

COMMENT ON TABLE secreton.user_roles IS 'User to dynamic role mapping';

-- ============================================================================
-- 7. ROLE-CAPABILITY MAPPING
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.role_capabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_type_id UUID NOT NULL REFERENCES secreton.user_role_types(id) ON DELETE CASCADE,
    capability_id UUID NOT NULL REFERENCES secreton.capabilities(id) ON DELETE CASCADE,
    conditions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(role_type_id, capability_id)
);

CREATE INDEX IF NOT EXISTS idx_role_caps_role ON secreton.role_capabilities(role_type_id);
CREATE INDEX IF NOT EXISTS idx_role_caps_cap ON secreton.role_capabilities(capability_id);

-- ============================================================================
-- 8. ENGINE ROLE TYPES (For transit, PKI, SSH engines)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.engine_role_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engine_type VARCHAR(100) NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    default_config JSONB NOT NULL DEFAULT '{}',
    allowed_operations JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(engine_type, code)
);

CREATE INDEX IF NOT EXISTS idx_engine_role_types_engine ON secreton.engine_role_types(engine_type);

COMMENT ON TABLE secreton.engine_role_types IS 'Role types for secret engines (transit, pki, ssh)';

-- ============================================================================
-- 9. SSH KEY TYPES (Replaces SshKeyType enum)
-- ============================================================================

CREATE TABLE IF NOT EXISTS secreton.ssh_key_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    algorithm VARCHAR(100) NOT NULL,
    key_size INTEGER,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    security_level VARCHAR(50) NOT NULL DEFAULT 'standard',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE secreton.ssh_key_types IS 'Dynamic SSH key type registry - replaces SshKeyType enum';

-- ============================================================================
-- 10. BOOTSTRAP DATA (Required defaults only)
-- ============================================================================

-- Bootstrap capabilities (Vault-style)
INSERT INTO secreton.capabilities (code, name, description, category, is_system)
VALUES
    -- Universal
    ('*', 'Superuser', 'All capabilities (root)', 'root', TRUE),
    ('sudo', 'Sudo', 'Sudo access for protected paths', 'root', TRUE),

    -- Basic operations
    ('create', 'Create', 'Create new data', 'basic', TRUE),
    ('read', 'Read', 'Read data', 'basic', TRUE),
    ('update', 'Update', 'Update existing data', 'basic', TRUE),
    ('patch', 'Patch', 'Partial update', 'basic', TRUE),
    ('delete', 'Delete', 'Delete data', 'basic', TRUE),
    ('list', 'List', 'List paths/keys', 'basic', TRUE),

    -- Key management
    ('create-key', 'Create Key', 'Create encryption keys', 'keys', TRUE),
    ('delete-key', 'Delete Key', 'Delete encryption keys', 'keys', TRUE),
    ('rotate-key', 'Rotate Key', 'Rotate encryption keys', 'keys', TRUE),
    ('read-key', 'Read Key', 'Read key metadata', 'keys', TRUE),
    ('list-keys', 'List Keys', 'List all keys', 'keys', TRUE),

    -- Crypto operations
    ('encrypt', 'Encrypt', 'Encrypt data', 'crypto', TRUE),
    ('decrypt', 'Decrypt', 'Decrypt data', 'crypto', TRUE),
    ('sign', 'Sign', 'Create digital signatures', 'crypto', TRUE),
    ('verify', 'Verify', 'Verify signatures', 'crypto', TRUE),
    ('wrap', 'Wrap', 'Wrap secrets', 'crypto', TRUE),
    ('unwrap', 'Unwrap', 'Unwrap secrets', 'crypto', TRUE),

    -- Utility operations
    ('generate-random', 'Generate Random', 'Generate random bytes', 'utility', TRUE),
    ('hash-data', 'Hash Data', 'Hash data', 'utility', TRUE),
    ('derive-key', 'Derive Key', 'Derive keys', 'utility', TRUE),

    -- Administrative
    ('view-metrics', 'View Metrics', 'View system metrics', 'admin', TRUE),
    ('configure-system', 'Configure System', 'Modify system configuration', 'admin', TRUE),
    ('manage-users', 'Manage Users', 'User administration', 'admin', TRUE),
    ('access-audit-logs', 'Access Audit Logs', 'View audit logs', 'admin', TRUE),
    ('manage-policies', 'Manage Policies', 'Policy administration', 'admin', TRUE)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap user role types
INSERT INTO secreton.user_role_types (code, name, description, priority, is_system, default_capabilities)
VALUES
    ('root', 'Root', 'Root access with all capabilities', 1000, TRUE, '["*"]'::jsonb),
    ('admin', 'Administrator', 'Administrative access', 100, TRUE,
     '["create", "read", "update", "delete", "list", "manage-users", "view-metrics", "access-audit-logs"]'::jsonb),
    ('engine-admin', 'Engine Administrator', 'Secret engine administration', 90, TRUE,
     '["create", "read", "update", "delete", "list", "create-key", "delete-key", "rotate-key"]'::jsonb),
    ('key-manager', 'Key Manager', 'Key management', 80, TRUE,
     '["read-key", "list-keys", "rotate-key", "create-key"]'::jsonb),
    ('crypto-user', 'Crypto User', 'Cryptographic operations', 70, TRUE,
     '["encrypt", "decrypt", "sign", "verify", "hash-data"]'::jsonb),
    ('read-only', 'Read Only', 'Read-only access', 10, TRUE,
     '["read", "list"]'::jsonb)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap auth method types
INSERT INTO secreton.auth_method_types (code, name, description, is_system, is_external)
VALUES
    ('token', 'Token', 'Token-based authentication', TRUE, FALSE),
    ('userpass', 'Username/Password', 'Username and password authentication', TRUE, FALSE),
    ('ldap', 'LDAP', 'LDAP authentication', TRUE, TRUE),
    ('oidc', 'OIDC', 'OpenID Connect authentication', TRUE, TRUE),
    ('okta', 'Okta', 'Okta authentication', TRUE, TRUE),
    ('github', 'GitHub', 'GitHub authentication', TRUE, TRUE),
    ('radius', 'RADIUS', 'RADIUS authentication', TRUE, TRUE),
    ('approle', 'AppRole', 'Application role authentication', TRUE, FALSE),
    ('kubernetes', 'Kubernetes', 'Kubernetes service account authentication', TRUE, TRUE),
    ('cert', 'TLS Certificate', 'TLS certificate authentication', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap token types
INSERT INTO secreton.token_types (code, name, description, is_system, is_renewable, is_orphan_allowed, max_ttl_seconds, default_ttl_seconds)
VALUES
    ('service', 'Service', 'Standard service token', TRUE, TRUE, FALSE, 86400, 3600),
    ('batch', 'Batch', 'Batch token (non-renewable)', TRUE, FALSE, TRUE, 86400, 3600),
    ('root', 'Root', 'Root token with no restrictions', TRUE, TRUE, TRUE, NULL, NULL),
    ('recovery', 'Recovery', 'Recovery token for unsealing', TRUE, FALSE, FALSE, 3600, 600),
    ('periodic', 'Periodic', 'Periodic token with TTL refresh', TRUE, TRUE, FALSE, NULL, 86400)
ON CONFLICT (code) DO NOTHING;

-- Bootstrap SSH key types
INSERT INTO secreton.ssh_key_types (code, name, algorithm, key_size, is_system, security_level)
VALUES
    ('rsa-2048', 'RSA 2048', 'RSA', 2048, TRUE, 'standard'),
    ('rsa-4096', 'RSA 4096', 'RSA', 4096, TRUE, 'high'),
    ('ed25519', 'Ed25519', 'Ed25519', NULL, TRUE, 'high'),
    ('ecdsa-256', 'ECDSA P-256', 'ECDSA', 256, TRUE, 'standard'),
    ('ecdsa-384', 'ECDSA P-384', 'ECDSA', 384, TRUE, 'high'),
    ('ecdsa-521', 'ECDSA P-521', 'ECDSA', 521, TRUE, 'high')
ON CONFLICT (code) DO NOTHING;

-- Bootstrap engine role types
INSERT INTO secreton.engine_role_types (engine_type, code, name, description, is_system, allowed_operations)
VALUES
    -- Transit engine roles
    ('transit', 'encrypt-only', 'Encrypt Only', 'Can only encrypt data', TRUE, '["encrypt"]'::jsonb),
    ('transit', 'decrypt-only', 'Decrypt Only', 'Can only decrypt data', TRUE, '["decrypt"]'::jsonb),
    ('transit', 'sign-only', 'Sign Only', 'Can only sign data', TRUE, '["sign"]'::jsonb),
    ('transit', 'verify-only', 'Verify Only', 'Can only verify signatures', TRUE, '["verify"]'::jsonb),
    ('transit', 'full-crypto', 'Full Crypto', 'All cryptographic operations', TRUE,
     '["encrypt", "decrypt", "sign", "verify", "rewrap"]'::jsonb),

    -- PKI engine roles
    ('pki', 'issue-certs', 'Issue Certificates', 'Can issue certificates', TRUE, '["issue"]'::jsonb),
    ('pki', 'sign-csr', 'Sign CSR', 'Can sign certificate requests', TRUE, '["sign"]'::jsonb),
    ('pki', 'read-crl', 'Read CRL', 'Can read certificate revocation list', TRUE, '["read"]'::jsonb),
    ('pki', 'revoke', 'Revoke Certificates', 'Can revoke certificates', TRUE, '["revoke"]'::jsonb),

    -- SSH engine roles
    ('ssh', 'sign-key', 'Sign SSH Key', 'Can sign SSH keys', TRUE, '["sign"]'::jsonb),
    ('ssh', 'create-otp', 'Create OTP', 'Can create one-time passwords', TRUE, '["create"]'::jsonb)
ON CONFLICT (engine_type, code) DO NOTHING;

-- Bootstrap default policies
INSERT INTO secreton.policies (role, path, action, effect, namespace)
VALUES
    ('root', '*', '*', 'allow', 'default'),
    ('default', 'auth/token/lookup-self', 'read', 'allow', 'default'),
    ('default', 'auth/token/renew-self', 'update', 'allow', 'default'),
    ('default', 'auth/token/revoke-self', 'update', 'allow', 'default'),
    ('default', 'sys/capabilities-self', 'read', 'allow', 'default')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 11. VIEWS FOR QUERYING
-- ============================================================================

-- Effective user capabilities view
CREATE OR REPLACE VIEW secreton.v_user_effective_capabilities AS
SELECT DISTINCT
    ur.user_id,
    urt.code AS role_code,
    c.code AS capability_code,
    c.name AS capability_name,
    ur.namespace
FROM secreton.user_roles ur
JOIN secreton.user_role_types urt ON ur.role_type_id = urt.id
JOIN secreton.role_capabilities rc ON urt.id = rc.role_type_id
JOIN secreton.capabilities c ON rc.capability_id = c.id
WHERE (ur.expires_at IS NULL OR ur.expires_at > NOW())
  AND (rc.expires_at IS NULL OR rc.expires_at > NOW());

COMMENT ON VIEW secreton.v_user_effective_capabilities IS 'Consolidated view of user capabilities through roles';

-- ============================================================================
-- 12. HELPER FUNCTIONS
-- ============================================================================

-- Check if user has capability in namespace
CREATE OR REPLACE FUNCTION secreton.user_has_capability(
    p_user_id UUID,
    p_capability_code VARCHAR(100),
    p_namespace VARCHAR(255) DEFAULT 'default'
) RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM secreton.v_user_effective_capabilities
        WHERE user_id = p_user_id
        AND namespace = p_namespace
        AND (capability_code = p_capability_code OR capability_code = '*')
    );
END;
$$ LANGUAGE plpgsql STABLE;

-- Check path access based on policies
CREATE OR REPLACE FUNCTION secreton.check_path_access(
    p_policies TEXT[],
    p_path VARCHAR(1000),
    p_action VARCHAR(100),
    p_namespace VARCHAR(255) DEFAULT 'default'
) RETURNS BOOLEAN AS $$
DECLARE
    v_denied BOOLEAN := FALSE;
    v_allowed BOOLEAN := FALSE;
BEGIN
    -- Check deny first
    SELECT TRUE INTO v_denied
    FROM secreton.policies p
    WHERE p.role = ANY(p_policies)
    AND p.namespace = p_namespace
    AND p.effect = 'deny'
    AND p_path LIKE REPLACE(REPLACE(p.path, '*', '%'), '+', '_')
    AND (p.action = p_action OR p.action = '*')
    LIMIT 1;

    IF v_denied THEN
        RETURN FALSE;
    END IF;

    -- Check allow
    SELECT TRUE INTO v_allowed
    FROM secreton.policies p
    WHERE p.role = ANY(p_policies)
    AND p.namespace = p_namespace
    AND p.effect = 'allow'
    AND p_path LIKE REPLACE(REPLACE(p.path, '*', '%'), '+', '_')
    AND (p.action = p_action OR p.action = '*')
    LIMIT 1;

    RETURN COALESCE(v_allowed, FALSE);
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- 13. TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION secreton.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN SELECT unnest(ARRAY[
        'capabilities', 'user_role_types', 'auth_method_types', 'token_types',
        'ssh_key_types', 'engine_role_types', 'policy_rules'
    ])
    LOOP
        EXECUTE format('
            DROP TRIGGER IF EXISTS update_%s_updated_at ON secreton.%s;
            CREATE TRIGGER update_%s_updated_at
            BEFORE UPDATE ON secreton.%s
            FOR EACH ROW
            EXECUTE FUNCTION secreton.update_updated_at_column();
        ', t, t, t, t);
    END LOOP;
END $$;

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
