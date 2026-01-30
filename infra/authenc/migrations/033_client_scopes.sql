-- Client Scopes System
-- Implements OAuth2/OIDC client scopes with reusable scope definitions,
-- default/optional scope assignments, and consent management

-- =====================================================================
-- TABLE: client_scopes
-- Reusable scope definitions with metadata
-- =====================================================================
CREATE TABLE IF NOT EXISTS client_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    -- Scope identification
    name VARCHAR(255) NOT NULL, -- e.g., "read:aset", "write:aset", "openid", "profile"
    display_name VARCHAR(500), -- Human-readable name for consent UI
    description TEXT, -- Detailed description for users

    -- Scope type
    protocol VARCHAR(50) NOT NULL DEFAULT 'openid-connect', -- openid-connect, saml

    -- Consent behavior
    consent_required BOOLEAN NOT NULL DEFAULT TRUE, -- Whether user consent is needed
    display_on_consent_screen BOOLEAN NOT NULL DEFAULT TRUE, -- Show in consent UI
    consent_screen_text TEXT, -- Custom text for consent screen

    -- Token inclusion
    include_in_token_scope BOOLEAN NOT NULL DEFAULT TRUE, -- Include in token scope claim
    gui_order INTEGER DEFAULT 0, -- Display order in UI

    -- Icon/branding
    icon_uri VARCHAR(1000), -- Icon URL for UI

    -- Attributes
    attributes JSONB DEFAULT '{}', -- Custom attributes (e.g., audience, resources)

    -- State
    enabled BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(realm_id, name)
);

CREATE INDEX IF NOT EXISTS idx_client_scopes_realm ON client_scopes(realm_id);
CREATE INDEX IF NOT EXISTS idx_client_scopes_name ON client_scopes(realm_id, name);
CREATE INDEX IF NOT EXISTS idx_client_scopes_protocol ON client_scopes(protocol);
CREATE INDEX IF NOT EXISTS idx_client_scopes_enabled ON client_scopes(realm_id, enabled) WHERE enabled = TRUE;

-- =====================================================================
-- TABLE: client_default_scopes
-- Default scopes automatically granted to clients
-- =====================================================================
CREATE TABLE IF NOT EXISTS client_default_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,
    scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(client_id, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_client_default_scopes_client ON client_default_scopes(client_id);
CREATE INDEX IF NOT EXISTS idx_client_default_scopes_scope ON client_default_scopes(scope_id);

-- =====================================================================
-- TABLE: client_optional_scopes
-- Optional scopes that can be requested by clients (require consent)
-- =====================================================================
CREATE TABLE IF NOT EXISTS client_optional_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,
    scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(client_id, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_client_optional_scopes_client ON client_optional_scopes(client_id);
CREATE INDEX IF NOT EXISTS idx_client_optional_scopes_scope ON client_optional_scopes(scope_id);

-- =====================================================================
-- TABLE: client_scope_mappings
-- Maps scopes to protocol mappers for claim generation
-- =====================================================================
CREATE TABLE IF NOT EXISTS client_scope_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,
    protocol_mapper_id UUID NOT NULL REFERENCES protocol_mappers(id) ON DELETE CASCADE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(scope_id, protocol_mapper_id)
);

CREATE INDEX IF NOT EXISTS idx_client_scope_mappings_scope ON client_scope_mappings(scope_id);
CREATE INDEX IF NOT EXISTS idx_client_scope_mappings_mapper ON client_scope_mappings(protocol_mapper_id);

-- =====================================================================
-- UPDATE: user_consents table to reference client_scopes
-- Add foreign key relationship for better data integrity
-- =====================================================================
-- Note: user_consents.scopes is currently TEXT[] (array of scope names)
-- We'll keep this for backward compatibility but add a new table for structured consent

CREATE TABLE IF NOT EXISTS user_consent_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,
    scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,

    -- Consent metadata
    granted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP,

    -- Source of consent
    consent_source VARCHAR(50) DEFAULT 'explicit', -- explicit, implicit, pre-authorized

    UNIQUE(user_id, client_id, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_user_consent_scopes_user ON user_consent_scopes(user_id);
CREATE INDEX IF NOT EXISTS idx_user_consent_scopes_client ON user_consent_scopes(client_id);
CREATE INDEX IF NOT EXISTS idx_user_consent_scopes_scope ON user_consent_scopes(scope_id);
CREATE INDEX IF NOT EXISTS idx_user_consent_scopes_expires ON user_consent_scopes(expires_at) WHERE expires_at IS NOT NULL;

-- =====================================================================
-- SEED: Standard OIDC Scopes
-- Pre-populate common OpenID Connect scopes
-- =====================================================================
DO $$
DECLARE
    default_realm_id UUID;
BEGIN
    -- Get default realm (assuming one exists)
    SELECT id INTO default_realm_id FROM realms LIMIT 1;

    IF default_realm_id IS NOT NULL THEN
        -- Standard OIDC scopes
        INSERT INTO client_scopes (realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, include_in_token_scope)
        VALUES
            (default_realm_id, 'openid', 'OpenID Connect', 'Access to OpenID Connect authentication', 'openid-connect', FALSE, FALSE, TRUE),
            (default_realm_id, 'profile', 'User Profile', 'Access to user profile information (name, username, etc.)', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'email', 'Email Address', 'Access to user email address', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'address', 'Physical Address', 'Access to user physical address', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'phone', 'Phone Number', 'Access to user phone number', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'offline_access', 'Offline Access', 'Access to refresh tokens for offline access', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'roles', 'User Roles', 'Access to user role information', 'openid-connect', FALSE, FALSE, TRUE),
            (default_realm_id, 'groups', 'User Groups', 'Access to user group membership', 'openid-connect', FALSE, FALSE, TRUE)
        ON CONFLICT (realm_id, name) DO NOTHING;

        -- SIMPelv2-specific scopes for asset management
        INSERT INTO client_scopes (realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, include_in_token_scope)
        VALUES
            (default_realm_id, 'read:aset', 'Read Assets', 'View asset information', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'write:aset', 'Manage Assets', 'Create, update, and delete assets', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'read:laporan', 'Read Reports', 'View reports and analytics', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'write:laporan', 'Manage Reports', 'Create and manage reports', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'admin:satker', 'Satker Administration', 'Administrative access to satker (work unit)', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'admin:wilayah', 'Regional Administration', 'Administrative access to regional level', 'openid-connect', TRUE, TRUE, TRUE),
            (default_realm_id, 'admin:pusat', 'Central Administration', 'Administrative access to central level', 'openid-connect', TRUE, TRUE, TRUE)
        ON CONFLICT (realm_id, name) DO NOTHING;
    END IF;
END $$;

-- =====================================================================
-- VIEWS: Convenience views for scope queries
-- =====================================================================

-- View: client_all_scopes
-- Shows all scopes available to a client (default + optional)
CREATE VIEW client_all_scopes AS
SELECT
    c.id AS client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name,
    cs.description,
    cs.consent_required,
    cs.display_on_consent_screen,
    'default' AS scope_type
FROM oauth2_clients c
INNER JOIN client_default_scopes cds ON c.id = cds.client_id
INNER JOIN client_scopes cs ON cds.scope_id = cs.id
WHERE cs.enabled = TRUE

UNION ALL

SELECT
    c.id AS client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name,
    cs.description,
    cs.consent_required,
    cs.display_on_consent_screen,
    'optional' AS scope_type
FROM oauth2_clients c
INNER JOIN client_optional_scopes cos ON c.id = cos.client_id
INNER JOIN client_scopes cs ON cos.scope_id = cs.id
WHERE cs.enabled = TRUE;

-- View: user_active_consents
-- Shows active user consents with scope details
CREATE VIEW user_active_consents AS
SELECT
    ucs.user_id,
    ucs.client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name AS scope_display_name,
    ucs.granted_at,
    ucs.expires_at,
    ucs.consent_source
FROM user_consent_scopes ucs
INNER JOIN oauth2_clients c ON ucs.client_id = c.id
INNER JOIN client_scopes cs ON ucs.scope_id = cs.id
WHERE ucs.expires_at IS NULL OR ucs.expires_at > NOW();

-- =====================================================================
-- FUNCTIONS: Scope validation and consent checking
-- =====================================================================

-- Function: validate_client_scopes
-- Validates if requested scopes are allowed for a client
CREATE OR REPLACE FUNCTION validate_client_scopes(
    p_client_id UUID,
    p_requested_scopes TEXT[]
) RETURNS TABLE(valid BOOLEAN, invalid_scopes TEXT[]) AS $$
DECLARE
    v_allowed_scopes TEXT[];
    v_invalid_scopes TEXT[];
BEGIN
    -- Get all allowed scopes for client (default + optional)
    SELECT ARRAY_AGG(scope_name) INTO v_allowed_scopes
    FROM client_all_scopes
    WHERE client_id = p_client_id;

    -- Find invalid scopes
    SELECT ARRAY_AGG(scope) INTO v_invalid_scopes
    FROM UNNEST(p_requested_scopes) AS scope
    WHERE scope != ALL(v_allowed_scopes);

    -- Return validation result
    IF v_invalid_scopes IS NULL OR ARRAY_LENGTH(v_invalid_scopes, 1) = 0 THEN
        RETURN QUERY SELECT TRUE, NULL::TEXT[];
    ELSE
        RETURN QUERY SELECT FALSE, v_invalid_scopes;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function: check_consent_required
-- Checks if user consent is required for requested scopes
CREATE OR REPLACE FUNCTION check_consent_required(
    p_user_id UUID,
    p_client_id UUID,
    p_requested_scopes TEXT[]
) RETURNS TABLE(consent_needed BOOLEAN, missing_scopes TEXT[]) AS $$
DECLARE
    v_consented_scopes TEXT[];
    v_missing_scopes TEXT[];
    v_consent_required_scopes TEXT[];
BEGIN
    -- Get scopes that require consent
    SELECT ARRAY_AGG(cs.name) INTO v_consent_required_scopes
    FROM client_scopes cs
    INNER JOIN client_all_scopes cas ON cs.id = cas.scope_id
    WHERE cas.client_id = p_client_id
      AND cs.consent_required = TRUE
      AND cs.name = ANY(p_requested_scopes);

    -- Get already consented scopes
    SELECT ARRAY_AGG(cs.name) INTO v_consented_scopes
    FROM user_consent_scopes ucs
    INNER JOIN client_scopes cs ON ucs.scope_id = cs.id
    WHERE ucs.user_id = p_user_id
      AND ucs.client_id = p_client_id
      AND (ucs.expires_at IS NULL OR ucs.expires_at > NOW())
      AND cs.name = ANY(v_consent_required_scopes);

    -- Find missing consents
    SELECT ARRAY_AGG(scope) INTO v_missing_scopes
    FROM UNNEST(v_consent_required_scopes) AS scope
    WHERE scope != ALL(COALESCE(v_consented_scopes, ARRAY[]::TEXT[]));

    -- Return result
    IF v_missing_scopes IS NULL OR ARRAY_LENGTH(v_missing_scopes, 1) = 0 THEN
        RETURN QUERY SELECT FALSE, NULL::TEXT[];
    ELSE
        RETURN QUERY SELECT TRUE, v_missing_scopes;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- =====================================================================
-- TRIGGERS: Automatic timestamp updates
-- =====================================================================

CREATE OR REPLACE FUNCTION update_client_scopes_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_client_scopes_timestamp
    BEFORE UPDATE ON client_scopes
    FOR EACH ROW
    EXECUTE FUNCTION update_client_scopes_timestamp();

-- =====================================================================
-- COMMENTS: Table and column documentation
-- =====================================================================

COMMENT ON TABLE client_scopes IS 'Reusable OAuth2/OIDC scope definitions with consent metadata';
COMMENT ON TABLE client_default_scopes IS 'Default scopes automatically granted to clients without consent';
COMMENT ON TABLE client_optional_scopes IS 'Optional scopes that clients can request (may require user consent)';
COMMENT ON TABLE client_scope_mappings IS 'Maps scopes to protocol mappers for claim generation';
COMMENT ON TABLE user_consent_scopes IS 'Structured user consent tracking per scope';

COMMENT ON COLUMN client_scopes.name IS 'OAuth2 scope name (e.g., "openid", "read:aset")';
COMMENT ON COLUMN client_scopes.consent_required IS 'Whether user must explicitly consent to this scope';
COMMENT ON COLUMN client_scopes.display_on_consent_screen IS 'Whether to show this scope in consent UI';
COMMENT ON COLUMN client_scopes.include_in_token_scope IS 'Whether to include scope in token scope claim';
COMMENT ON COLUMN client_scopes.attributes IS 'Custom attributes for scope (audience, resources, etc.)';
