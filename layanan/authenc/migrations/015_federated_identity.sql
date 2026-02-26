-- Federated Identity Links Table
-- Links local user accounts to federated identity providers
CREATE TABLE IF NOT EXISTS federated_identity_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    -- Identity provider reference
    identity_provider_alias VARCHAR(255) NOT NULL,
    federated_user_id VARCHAR(255) NOT NULL, -- User ID from the external IdP
    federated_username VARCHAR(255),

    -- Token management
    token TEXT, -- Current access token from IdP
    token_expires_at TIMESTAMP,
    refresh_token TEXT,

    -- User attributes from IdP
    federated_attributes JSONB, -- Full user profile from IdP

    -- Linking metadata
    linked_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_authenticated_at TIMESTAMP,
    authentication_count INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(identity_provider_alias, federated_user_id),
    UNIQUE(user_id, identity_provider_alias)
);

-- Identity Provider Mappers Table Enhancement
-- Maps attributes from external IdPs to local user attributes
-- This enhances the identity_provider_mappers table from 001_initial_schema.sql

-- Add new columns to identity_provider_mappers if they don't exist
DO $$
BEGIN
    -- Add identity_provider_alias column (alternative to identity_provider_id)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'identity_provider_mappers' AND column_name = 'identity_provider_alias') THEN
        ALTER TABLE identity_provider_mappers ADD COLUMN identity_provider_alias VARCHAR(255);
    END IF;

    -- Add sync_mode column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'identity_provider_mappers' AND column_name = 'sync_mode') THEN
        ALTER TABLE identity_provider_mappers ADD COLUMN sync_mode VARCHAR(50) NOT NULL DEFAULT 'IMPORT';
    END IF;

    -- Add realm_id if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'identity_provider_mappers' AND column_name = 'realm_id') THEN
        ALTER TABLE identity_provider_mappers ADD COLUMN realm_id UUID REFERENCES realms(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Identity Broker Configuration Table
-- Configuration for identity brokering
CREATE TABLE IF NOT EXISTS identity_broker_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    -- Broker settings
    alias VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,

    -- Identity provider type
    provider_type VARCHAR(100) NOT NULL, -- saml, oidc, oauth2, ldap, kerberos

    -- Authentication flow
    first_broker_login_flow VARCHAR(255), -- Flow to execute on first login
    post_broker_login_flow VARCHAR(255), -- Flow to execute on subsequent logins

    -- User linking
    trust_email BOOLEAN NOT NULL DEFAULT FALSE,
    store_token BOOLEAN NOT NULL DEFAULT FALSE,
    add_read_token_role_on_create BOOLEAN NOT NULL DEFAULT FALSE,

    -- Account linking
    link_only BOOLEAN NOT NULL DEFAULT FALSE, -- Only allow linking, not account creation

    -- Configuration
    config JSONB NOT NULL, -- Provider-specific configuration

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(realm_id, alias)
);

-- Federated Identity Authentication Log
-- Audit log for federated authentication attempts
CREATE TABLE IF NOT EXISTS federated_auth_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    -- Authentication details
    identity_provider_alias VARCHAR(255) NOT NULL,
    federated_user_id VARCHAR(255),

    -- Result
    success BOOLEAN NOT NULL,
    error_code VARCHAR(100),
    error_message TEXT,

    -- User action
    action VARCHAR(50) NOT NULL, -- AUTHENTICATE, LINK, UNLINK, UPDATE_PROFILE

    -- Context
    ip_address INET,
    user_agent TEXT,
    session_id UUID,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Account Linking Requests Table
-- Pending account linking requests (requires user confirmation)
CREATE TABLE IF NOT EXISTS account_linking_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    -- Linking details
    identity_provider_alias VARCHAR(255) NOT NULL,
    federated_user_id VARCHAR(255) NOT NULL,
    federated_username VARCHAR(255),
    federated_email VARCHAR(255),

    -- Request state
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING', -- PENDING, APPROVED, REJECTED, EXPIRED
    confirmation_token VARCHAR(255) UNIQUE,

    -- User attributes from IdP
    federated_attributes JSONB,

    -- Expiration
    expires_at TIMESTAMP NOT NULL,

    -- Resolution
    resolved_at TIMESTAMP,
    resolved_by VARCHAR(50), -- USER, ADMIN, SYSTEM

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_federated_links_user ON federated_identity_links(user_id);
CREATE INDEX IF NOT EXISTS idx_federated_links_realm ON federated_identity_links(realm_id);
CREATE INDEX IF NOT EXISTS idx_federated_links_provider ON federated_identity_links(identity_provider_alias);
CREATE INDEX IF NOT EXISTS idx_federated_links_federated_user ON federated_identity_links(identity_provider_alias, federated_user_id);

CREATE INDEX IF NOT EXISTS idx_idp_mappers_realm ON identity_provider_mappers(realm_id);
CREATE INDEX IF NOT EXISTS idx_idp_mappers_provider ON identity_provider_mappers(identity_provider_alias);
CREATE INDEX IF NOT EXISTS idx_idp_mappers_type ON identity_provider_mappers(mapper_type);

CREATE INDEX IF NOT EXISTS idx_broker_configs_realm ON identity_broker_configs(realm_id);
CREATE INDEX IF NOT EXISTS idx_broker_configs_alias ON identity_broker_configs(alias);
CREATE INDEX IF NOT EXISTS idx_broker_configs_enabled ON identity_broker_configs(realm_id) WHERE enabled = TRUE;

CREATE INDEX IF NOT EXISTS idx_federated_auth_log_user ON federated_auth_log(user_id);
CREATE INDEX IF NOT EXISTS idx_federated_auth_log_realm ON federated_auth_log(realm_id);
CREATE INDEX IF NOT EXISTS idx_federated_auth_log_provider ON federated_auth_log(identity_provider_alias);
CREATE INDEX IF NOT EXISTS idx_federated_auth_log_created ON federated_auth_log(created_at);

CREATE INDEX IF NOT EXISTS idx_account_linking_user ON account_linking_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_account_linking_status ON account_linking_requests(status) WHERE status = 'PENDING';
CREATE INDEX IF NOT EXISTS idx_account_linking_token ON account_linking_requests(confirmation_token);
CREATE INDEX IF NOT EXISTS idx_account_linking_expires ON account_linking_requests(expires_at) WHERE status = 'PENDING';
