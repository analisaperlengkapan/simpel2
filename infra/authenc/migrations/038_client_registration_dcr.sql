-- Migration V009: Dynamic Client Registration (RFC 7591/7592)
--
-- This migration adds support for OAuth 2.0 Dynamic Client Registration
-- including registration access tokens and initial access tokens

-- Add DCR metadata fields to oauth2_clients table
ALTER TABLE oauth2_clients
ADD COLUMN IF NOT EXISTS logo_uri TEXT,
ADD COLUMN IF NOT EXISTS client_uri TEXT,
ADD COLUMN IF NOT EXISTS policy_uri TEXT,
ADD COLUMN IF NOT EXISTS tos_uri TEXT,
ADD COLUMN IF NOT EXISTS jwks_uri TEXT,
ADD COLUMN IF NOT EXISTS jwks JSONB,
ADD COLUMN IF NOT EXISTS sector_identifier_uri TEXT,
ADD COLUMN IF NOT EXISTS subject_type VARCHAR(20) DEFAULT 'public',
ADD COLUMN IF NOT EXISTS id_token_signed_response_alg VARCHAR(10) DEFAULT 'EdDSA',
ADD COLUMN IF NOT EXISTS id_token_encrypted_response_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS id_token_encrypted_response_enc VARCHAR(20),
ADD COLUMN IF NOT EXISTS userinfo_signed_response_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS userinfo_encrypted_response_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS userinfo_encrypted_response_enc VARCHAR(20),
ADD COLUMN IF NOT EXISTS request_object_signing_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS request_object_encryption_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS request_object_encryption_enc VARCHAR(20),
ADD COLUMN IF NOT EXISTS token_endpoint_auth_signing_alg VARCHAR(20),
ADD COLUMN IF NOT EXISTS default_max_age INTEGER,
ADD COLUMN IF NOT EXISTS require_auth_time BOOLEAN DEFAULT false,
ADD COLUMN IF NOT EXISTS default_acr_values TEXT[],
ADD COLUMN IF NOT EXISTS initiate_login_uri TEXT,
ADD COLUMN IF NOT EXISTS request_uris TEXT[],
ADD COLUMN IF NOT EXISTS application_type VARCHAR(20) DEFAULT 'web',
ADD COLUMN IF NOT EXISTS contacts TEXT[],
ADD COLUMN IF NOT EXISTS client_id_issued_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS client_secret_expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS software_id TEXT,
ADD COLUMN IF NOT EXISTS software_version TEXT,
ADD COLUMN IF NOT EXISTS registration_access_token_hash TEXT;

-- Create index for registration access token lookups
CREATE INDEX IF NOT EXISTS idx_oauth2_clients_registration_token
ON oauth2_clients(registration_access_token_hash)
WHERE deleted_at IS NULL;

-- Client registration access tokens table
-- Stores tokens issued for managing dynamically registered clients
CREATE TABLE IF NOT EXISTS client_registration_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash TEXT NOT NULL UNIQUE,
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_client_registration_tokens_client
ON client_registration_tokens(client_id)
WHERE revoked = false;

CREATE INDEX IF NOT EXISTS idx_client_registration_tokens_hash
ON client_registration_tokens(token_hash)
WHERE revoked = false;

-- Initial access tokens table
-- Used to protect the client registration endpoint
CREATE TABLE IF NOT EXISTS initial_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash TEXT NOT NULL UNIQUE,
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    count INTEGER NOT NULL DEFAULT 1,
    remaining_count INTEGER NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_initial_access_tokens_hash
ON initial_access_tokens(token_hash)
WHERE revoked = false AND (expires_at IS NULL OR expires_at > NOW());

CREATE INDEX IF NOT EXISTS idx_initial_access_tokens_realm
ON initial_access_tokens(realm_id)
WHERE revoked = false;

-- Software statements table (trusted issuers)
CREATE TABLE IF NOT EXISTS software_statement_issuers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    issuer TEXT NOT NULL UNIQUE,
    jwks_uri TEXT,
    jwks JSONB,
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_software_statement_issuers_issuer
ON software_statement_issuers(issuer)
WHERE enabled = true;

-- Client registration policy
CREATE TABLE IF NOT EXISTS client_registration_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    allow_dynamic_registration BOOLEAN DEFAULT true,
    require_initial_access_token BOOLEAN DEFAULT false,
    require_software_statement BOOLEAN DEFAULT false,
    allowed_redirect_uri_patterns TEXT[],
    blocked_redirect_uri_patterns TEXT[],
    max_redirect_uris INTEGER DEFAULT 10,
    allowed_scopes TEXT[],
    default_scopes TEXT[],
    allowed_grant_types TEXT[],
    allowed_response_types TEXT[],
    require_https_redirect_uris BOOLEAN DEFAULT true,
    allow_localhost_redirect BOOLEAN DEFAULT false,
    client_secret_expires_in INTEGER, -- seconds
    registration_token_expires_in INTEGER DEFAULT 31536000, -- 1 year
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(realm_id, name)
);

CREATE INDEX IF NOT EXISTS idx_client_registration_policies_realm
ON client_registration_policies(realm_id)
WHERE enabled = true;

-- Audit log for client registration events
CREATE TABLE IF NOT EXISTS client_registration_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL, -- 'REGISTER', 'UPDATE', 'DELETE', 'READ'
    client_id UUID REFERENCES oauth2_clients(id) ON DELETE SET NULL,
    client_identifier TEXT, -- client_id string
    realm_id UUID REFERENCES realms(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    initial_access_token_id UUID REFERENCES initial_access_tokens(id) ON DELETE SET NULL,
    registration_access_token_id UUID REFERENCES client_registration_tokens(id) ON DELETE SET NULL,
    success BOOLEAN NOT NULL,
    error_code VARCHAR(50),
    error_description TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_client_registration_audit_log_client
ON client_registration_audit_log(client_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_client_registration_audit_log_realm
ON client_registration_audit_log(realm_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_client_registration_audit_log_type
ON client_registration_audit_log(event_type, created_at DESC);

-- Comments for documentation
COMMENT ON TABLE client_registration_tokens IS 'RFC 7592: Registration access tokens for managing dynamically registered OAuth2 clients';
COMMENT ON TABLE initial_access_tokens IS 'RFC 7591: Initial access tokens to protect client registration endpoint';
COMMENT ON TABLE software_statement_issuers IS 'RFC 7591: Trusted issuers of software statements (signed JWTs)';
COMMENT ON TABLE client_registration_policies IS 'Policies controlling dynamic client registration per realm';
COMMENT ON TABLE client_registration_audit_log IS 'Audit trail for all client registration operations';
