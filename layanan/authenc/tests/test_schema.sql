-- Minimal schema for WebAuthn testing
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Realms table (required for users)
-- NOTE: Keep in sync with 001_initial_schema.sql + 048_realm_soft_delete_unique_index.sql
CREATE TABLE IF NOT EXISTS realms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    ssl_required VARCHAR(50) NOT NULL DEFAULT 'external',
    registration_allowed BOOLEAN NOT NULL DEFAULT false,
    registration_email_as_username BOOLEAN NOT NULL DEFAULT false,
    remember_me BOOLEAN NOT NULL DEFAULT true,
    verify_email BOOLEAN NOT NULL DEFAULT false,
    login_with_email_allowed BOOLEAN NOT NULL DEFAULT true,
    duplicate_emails_allowed BOOLEAN NOT NULL DEFAULT false,
    reset_password_allowed BOOLEAN NOT NULL DEFAULT true,
    edit_username_allowed BOOLEAN NOT NULL DEFAULT false,
    brute_force_protected BOOLEAN NOT NULL DEFAULT true,
    max_failure_wait_seconds INTEGER NOT NULL DEFAULT 900,
    minimum_quick_login_wait_seconds INTEGER NOT NULL DEFAULT 60,
    wait_increment_seconds INTEGER NOT NULL DEFAULT 60,
    quick_login_check_milli_seconds BIGINT NOT NULL DEFAULT 1000,
    max_delta_time_seconds INTEGER NOT NULL DEFAULT 43200,
    failure_factor INTEGER NOT NULL DEFAULT 30,
    default_signature_algorithm VARCHAR(50) NOT NULL DEFAULT 'RS256',
    revoke_refresh_token BOOLEAN NOT NULL DEFAULT false,
    refresh_token_max_reuse INTEGER NOT NULL DEFAULT 0,
    access_token_lifespan INTEGER NOT NULL DEFAULT 300,
    access_token_lifespan_for_implicit_flow INTEGER NOT NULL DEFAULT 900,
    sso_session_idle_timeout INTEGER NOT NULL DEFAULT 1800,
    sso_session_max_lifespan INTEGER NOT NULL DEFAULT 36000,
    sso_session_idle_timeout_remember_me INTEGER NOT NULL DEFAULT 0,
    sso_session_max_lifespan_remember_me INTEGER NOT NULL DEFAULT 0,
    offline_session_idle_timeout INTEGER NOT NULL DEFAULT 2592000,
    offline_session_max_lifespan INTEGER NOT NULL DEFAULT 5184000,
    client_session_idle_timeout INTEGER NOT NULL DEFAULT 0,
    client_session_max_lifespan INTEGER NOT NULL DEFAULT 0,
    access_code_lifespan INTEGER NOT NULL DEFAULT 60,
    access_code_lifespan_user_action INTEGER NOT NULL DEFAULT 300,
    access_code_lifespan_login INTEGER NOT NULL DEFAULT 1800,
    action_token_generated_by_admin_lifespan INTEGER NOT NULL DEFAULT 43200,
    action_token_generated_by_user_lifespan INTEGER NOT NULL DEFAULT 300,
    oauth2_device_code_lifespan INTEGER NOT NULL DEFAULT 600,
    oauth2_device_polling_interval INTEGER NOT NULL DEFAULT 5,
    attributes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Partial unique index: only enforce uniqueness among live (non-deleted) rows
CREATE UNIQUE INDEX IF NOT EXISTS idx_realms_name_unique ON realms (name) WHERE deleted_at IS NULL;

-- Drop and recreate users table with all required columns
DROP TABLE IF EXISTS users CASCADE;

-- Users table with all required columns
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    phone_number VARCHAR(255),
    phone_verified BOOLEAN NOT NULL DEFAULT false,
    password_hash VARCHAR(255),
    totp_secret VARCHAR(255),
    totp_backup_codes TEXT[],
    webauthn_enabled BOOLEAN NOT NULL DEFAULT false,
    account_locked BOOLEAN NOT NULL DEFAULT false,
    account_locked_until TIMESTAMPTZ,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_login_at TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,
    password_expires_at TIMESTAMPTZ,
    require_password_change BOOLEAN NOT NULL DEFAULT false,
    organization_id UUID,
    attributes JSONB,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    realm_id UUID REFERENCES realms(id) ON DELETE RESTRICT,
    federated BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    login_count INTEGER NOT NULL DEFAULT 0
);

-- Partial unique indexes: only enforce uniqueness among live (non-deleted) rows
CREATE UNIQUE INDEX idx_users_username_unique ON users (username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email_unique ON users (email) WHERE deleted_at IS NULL;

-- WebAuthn challenges table
CREATE TABLE IF NOT EXISTS webauthn_challenges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge TEXT NOT NULL,
    challenge_type VARCHAR(50) NOT NULL, -- 'registration' or 'authentication'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false
);

-- WebAuthn credentials table
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    public_key_algorithm INTEGER NOT NULL,
    signature_counter BIGINT NOT NULL DEFAULT 0,
    attestation_object TEXT,
    authenticator_data TEXT,
    user_handle TEXT,
    credential_type VARCHAR(50) NOT NULL DEFAULT 'public-key',
    transports TEXT[], -- Array of transport types
    aaguid UUID,
    attestation_format VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    enabled BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(user_id, credential_id)
);

-- User consents table for GDPR compliance
CREATE TABLE IF NOT EXISTS user_consents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id VARCHAR(255) NOT NULL,
    scopes TEXT[] NOT NULL, -- Array of consented scopes
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- Optional expiration
    metadata JSONB, -- Additional consent metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, client_id) -- One consent record per user-client pair
);

-- Authentication flows table for pluggable authentication flows
CREATE TABLE IF NOT EXISTS authentication_flows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    alias VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    flow_type VARCHAR(50) NOT NULL, -- 'browser', 'direct_grant', 'client_authentication', 'registration', 'reset_credentials', 'docker', 'custom'
    realm_id UUID REFERENCES realms(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Authentication executions table for flow steps
CREATE TABLE IF NOT EXISTS authentication_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    flow_id UUID NOT NULL REFERENCES authentication_flows(id) ON DELETE CASCADE,
    alias VARCHAR(255) NOT NULL,
    description TEXT,
    execution_type VARCHAR(100) NOT NULL, -- authenticator type or 'condition'
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    configuration JSONB, -- Configuration parameters
    requirements TEXT[], -- Conditional requirements
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Authentication sessions table for ongoing authentication
CREATE TABLE IF NOT EXISTS authentication_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_session_id UUID,
    client_id VARCHAR(255) NOT NULL,
    flow_id UUID NOT NULL REFERENCES authentication_flows(id) ON DELETE CASCADE,
    current_execution_id UUID REFERENCES authentication_executions(id) ON DELETE SET NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_data JSONB, -- Temporary session data
    auth_notes JSONB, -- Authentication notes and metadata
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 minutes'),
    completed BOOLEAN NOT NULL DEFAULT false,
    success BOOLEAN,
    error_message TEXT
);

-- Insert a default realm for testing
INSERT INTO realms (id, name, display_name, enabled)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'test-realm', 'Test Realm', true)
ON CONFLICT (id) DO NOTHING;
