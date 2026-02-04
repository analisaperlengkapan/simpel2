SET search_path = authenc, public;
-- Client Policies & Profiles Implementation
-- Migration: 036_client_policies_and_profiles.sql
--
-- Implements comprehensive client policy framework for OAuth2/OIDC clients.
-- Supports conditional policy execution, reusable profiles, and FAPI compliance.

-- Client Policies Table
CREATE TABLE IF NOT EXISTS client_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,

    -- Policy conditions and executors (stored as JSON arrays)
    conditions TEXT[] NOT NULL DEFAULT '{}',
    condition_config JSONB NOT NULL DEFAULT '{}',
    executors TEXT[] NOT NULL DEFAULT '{}',
    executor_config JSONB NOT NULL DEFAULT '{}',

    -- Priority and type
    priority INTEGER NOT NULL DEFAULT 0,
    policy_type VARCHAR(100) NOT NULL DEFAULT 'custom',

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,

    -- Constraints
    UNIQUE(realm_id, name),
    CHECK (priority >= 0),
    CHECK (policy_type IN ('security', 'compliance', 'fapi-baseline', 'fapi-advanced', 'custom'))
);

CREATE INDEX IF NOT EXISTS idx_client_policies_realm ON client_policies(realm_id);
CREATE INDEX IF NOT EXISTS idx_client_policies_enabled ON client_policies(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_client_policies_priority ON client_policies(priority DESC);
CREATE INDEX IF NOT EXISTS idx_client_policies_type ON client_policies(policy_type);

COMMENT ON TABLE client_policies IS 'Client security policies for OAuth2/OIDC enforcement';
COMMENT ON COLUMN client_policies.conditions IS 'Array of condition identifiers that must be met';
COMMENT ON COLUMN client_policies.condition_config IS 'JSON configuration for conditions';
COMMENT ON COLUMN client_policies.executors IS 'Array of executor identifiers to run';
COMMENT ON COLUMN client_policies.executor_config IS 'JSON configuration for executors';
COMMENT ON COLUMN client_policies.priority IS 'Higher priority policies execute first';

-- Client Profiles Table
CREATE TABLE IF NOT EXISTS client_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,

    -- Policy references
    policy_ids UUID[] NOT NULL DEFAULT '{}',

    -- Profile metadata
    profile_type VARCHAR(100) NOT NULL DEFAULT 'custom',
    is_builtin BOOLEAN NOT NULL DEFAULT false,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,

    -- Constraints
    UNIQUE(realm_id, name),
    CHECK (profile_type IN ('fapi-1-baseline', 'fapi-1-advanced', 'fapi-2-security', 'fapi-2-message-signing', 'custom'))
);

CREATE INDEX IF NOT EXISTS idx_client_profiles_realm ON client_profiles(realm_id);
CREATE INDEX IF NOT EXISTS idx_client_profiles_enabled ON client_profiles(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_client_profiles_type ON client_profiles(profile_type);
CREATE INDEX IF NOT EXISTS idx_client_profiles_builtin ON client_profiles(is_builtin) WHERE is_builtin = true;

COMMENT ON TABLE client_profiles IS 'Reusable collections of client policies';
COMMENT ON COLUMN client_profiles.policy_ids IS 'Array of policy UUIDs included in this profile';
COMMENT ON COLUMN client_profiles.is_builtin IS 'Built-in profiles cannot be deleted';

-- Client Policy Assignments Table
CREATE TABLE IF NOT EXISTS client_policy_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,

    -- Assignment can be either direct policy or via profile
    policy_id UUID REFERENCES client_policies(id) ON DELETE CASCADE,
    profile_id UUID REFERENCES client_profiles(id) ON DELETE CASCADE,

    assignment_type VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority_override INTEGER,

    -- Audit fields
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID,

    -- Constraints
    CHECK (assignment_type IN ('direct', 'profile')),
    CHECK (
        (assignment_type = 'direct' AND policy_id IS NOT NULL AND profile_id IS NULL) OR
        (assignment_type = 'profile' AND profile_id IS NOT NULL AND policy_id IS NULL)
    ),
    -- Prevent duplicate assignments
    UNIQUE(client_id, policy_id, profile_id)
);

CREATE INDEX IF NOT EXISTS idx_client_policy_assignments_client ON client_policy_assignments(client_id);
CREATE INDEX IF NOT EXISTS idx_client_policy_assignments_policy ON client_policy_assignments(policy_id) WHERE policy_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_client_policy_assignments_profile ON client_policy_assignments(profile_id) WHERE profile_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_client_policy_assignments_enabled ON client_policy_assignments(enabled) WHERE enabled = true;

COMMENT ON TABLE client_policy_assignments IS 'Associates policies and profiles with clients';
COMMENT ON COLUMN client_policy_assignments.assignment_type IS 'Either "direct" for policy or "profile" for profile assignment';
COMMENT ON COLUMN client_policy_assignments.priority_override IS 'Overrides policy priority for this specific assignment';

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_client_policy_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_client_policies_updated
BEFORE UPDATE ON client_policies
FOR EACH ROW
EXECUTE FUNCTION update_client_policy_timestamp();

CREATE TRIGGER trigger_client_profiles_updated
BEFORE UPDATE ON client_profiles
FOR EACH ROW
EXECUTE FUNCTION update_client_policy_timestamp();

-- Insert built-in FAPI 1.0 Baseline Profile
INSERT INTO client_profiles (id, realm_id, name, description, enabled, profile_type, is_builtin, policy_ids)
SELECT
    gen_random_uuid(),
    id,
    'FAPI 1.0 Baseline',
    'Financial-grade API 1.0 Baseline Security Profile - enforces PKCE, secure redirects, and rejects implicit grants',
    true,
    'fapi-1-baseline',
    true,
    '{}' -- Will be populated after policies are created
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Insert built-in FAPI 1.0 Advanced Profile
INSERT INTO client_profiles (id, realm_id, name, description, enabled, profile_type, is_builtin, policy_ids)
SELECT
    gen_random_uuid(),
    id,
    'FAPI 1.0 Advanced',
    'Financial-grade API 1.0 Advanced Security Profile - adds DPoP binding and enhanced security measures',
    true,
    'fapi-1-advanced',
    true,
    '{}' -- Will be populated after policies are created
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create PKCE Enforcement Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'PKCE Enforcement',
    'Enforces Proof Key for Code Exchange (PKCE) for authorization code flow',
    true,
    ARRAY['grant-type-condition'],
    ARRAY['pkce-enforcer-executor'],
    100,
    'security',
    '{"grant-type-condition": {"allowed_grant_types": ["authorization_code"]}}'::jsonb,
    '{"pkce-enforcer-executor": {"enforce_pkce": true}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create Secure Redirect URIs Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'Secure Redirect URIs',
    'Enforces HTTPS for all redirect URIs',
    true,
    ARRAY['any-client-condition'],
    ARRAY['secure-redirect-uris-enforcer-executor'],
    90,
    'security',
    '{}'::jsonb,
    '{"secure-redirect-uris-enforcer-executor": {"enforce_https": true}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create Reject Implicit Grant Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'Reject Implicit Grant',
    'Rejects insecure implicit grant flow',
    true,
    ARRAY['any-client-condition'],
    ARRAY['reject-implicit-grant-executor'],
    80,
    'security',
    '{}'::jsonb,
    '{}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create DPoP Binding Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'DPoP Binding',
    'Enforces Demonstrating Proof-of-Possession (DPoP) for token binding',
    false, -- Disabled by default, enable for FAPI Advanced
    ARRAY['any-client-condition'],
    ARRAY['dpop-bind-enforcer-executor'],
    110,
    'security',
    '{}'::jsonb,
    '{"dpop-bind-enforcer-executor": {"enforce_dpop": true}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create Confidential Client Only Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'Confidential Clients Only',
    'Only allows confidential clients to authenticate',
    false, -- Disabled by default
    ARRAY['any-client-condition'],
    ARRAY['confidential-client-accept-executor'],
    70,
    'security',
    '{}'::jsonb,
    '{"confidential-client-accept-executor": {"accept_confidential_only": true}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create Consent Required Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'Consent Required',
    'Requires explicit user consent for all authorizations',
    false, -- Disabled by default
    ARRAY['any-client-condition'],
    ARRAY['consent-required-executor'],
    60,
    'compliance',
    '{}'::jsonb,
    '{"consent-required-executor": {"require_consent": true}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Create Secure Signing Algorithm Policy for each realm
INSERT INTO client_policies (realm_id, name, description, enabled, conditions, executors, priority, policy_type, condition_config, executor_config)
SELECT
    id,
    'Secure Signing Algorithm',
    'Enforces secure signing algorithms (Ed25519, ES256, RS256)',
    false, -- Disabled by default
    ARRAY['any-client-condition'],
    ARRAY['secure-signing-algorithm-executor'],
    85,
    'security',
    '{}'::jsonb,
    '{"secure-signing-algorithm-executor": {"enforce_secure_algorithm": true, "allowed_algorithms": ["EdDSA", "ES256", "RS256"]}}'::jsonb
FROM realms
ON CONFLICT (realm_id, name) DO NOTHING;

-- Update FAPI 1.0 Baseline Profile with policy IDs
UPDATE client_profiles cp
SET policy_ids = (
    SELECT array_agg(id)
    FROM client_policies
    WHERE realm_id = cp.realm_id
    AND name IN ('PKCE Enforcement', 'Secure Redirect URIs', 'Reject Implicit Grant')
)
WHERE profile_type = 'fapi-1-baseline';

-- Update FAPI 1.0 Advanced Profile with policy IDs
UPDATE client_profiles cp
SET policy_ids = (
    SELECT array_agg(id)
    FROM client_policies
    WHERE realm_id = cp.realm_id
    AND name IN ('PKCE Enforcement', 'Secure Redirect URIs', 'Reject Implicit Grant', 'DPoP Binding')
)
WHERE profile_type = 'fapi-1-advanced';

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON client_policies TO authenc;
GRANT SELECT, INSERT, UPDATE, DELETE ON client_profiles TO authenc;
GRANT SELECT, INSERT, UPDATE, DELETE ON client_policy_assignments TO authenc;

-- Create view for client policies with full details
CREATE OR REPLACE VIEW v_client_policies_with_profiles AS
SELECT
    cp.*,
    COALESCE(
        json_agg(
            json_build_object(
                'profile_id', prof.id,
                'profile_name', prof.name,
                'profile_type', prof.profile_type
            )
        ) FILTER (WHERE prof.id IS NOT NULL),
        '[]'::json
    ) as applied_profiles
FROM client_policies cp
LEFT JOIN client_profiles prof ON cp.id = ANY(prof.policy_ids)
GROUP BY cp.id;

GRANT SELECT ON v_client_policies_with_profiles TO authenc;

-- Create view for client profile details with policies
CREATE OR REPLACE VIEW v_client_profiles_with_policies AS
SELECT
    prof.*,
    COALESCE(
        json_agg(
            json_build_object(
                'policy_id', cp.id,
                'policy_name', cp.name,
                'policy_type', cp.policy_type,
                'priority', cp.priority,
                'enabled', cp.enabled
            )
            ORDER BY cp.priority DESC
        ) FILTER (WHERE cp.id IS NOT NULL),
        '[]'::json
    ) as policies
FROM client_profiles prof
LEFT JOIN client_policies cp ON cp.id = ANY(prof.policy_ids)
GROUP BY prof.id;

GRANT SELECT ON v_client_profiles_with_policies TO authenc;

-- Create view for client assignments with full details
CREATE OR REPLACE VIEW v_client_policy_assignments_detail AS
SELECT
    cpa.*,
    CASE
        WHEN cpa.assignment_type = 'direct' THEN
            json_build_object(
                'type', 'policy',
                'id', cp.id,
                'name', cp.name,
                'description', cp.description,
                'priority', COALESCE(cpa.priority_override, cp.priority)
            )
        WHEN cpa.assignment_type = 'profile' THEN
            json_build_object(
                'type', 'profile',
                'id', prof.id,
                'name', prof.name,
                'description', prof.description,
                'policy_count', array_length(prof.policy_ids, 1)
            )
    END as assignment_details
FROM client_policy_assignments cpa
LEFT JOIN client_policies cp ON cpa.policy_id = cp.id
LEFT JOIN client_profiles prof ON cpa.profile_id = prof.id;

GRANT SELECT ON v_client_policy_assignments_detail TO authenc;

-- Analytics: Count policies by type
CREATE OR REPLACE VIEW v_client_policy_stats AS
SELECT
    realm_id,
    policy_type,
    COUNT(*) as total_policies,
    COUNT(*) FILTER (WHERE enabled = true) as enabled_policies,
    COUNT(*) FILTER (WHERE enabled = false) as disabled_policies,
    AVG(priority) as avg_priority
FROM client_policies
GROUP BY realm_id, policy_type;

GRANT SELECT ON v_client_policy_stats TO authenc;

-- Analytics: Profile usage statistics
CREATE OR REPLACE VIEW v_client_profile_usage AS
SELECT
    prof.id as profile_id,
    prof.realm_id,
    prof.name as profile_name,
    prof.profile_type,
    prof.is_builtin,
    COUNT(DISTINCT cpa.client_id) as assigned_clients,
    array_length(prof.policy_ids, 1) as policy_count
FROM client_profiles prof
LEFT JOIN client_policy_assignments cpa ON cpa.profile_id = prof.id AND cpa.enabled = true
GROUP BY prof.id;

GRANT SELECT ON v_client_profile_usage TO authenc;
