SET search_path = authenc, public;
-- Migration: Token Exchange Audit and Metadata (RFC 8693)
-- Description: Add support for tracking OAuth 2.0 Token Exchange operations
-- Version: 035
-- Date: 2025-11-11

-- Token Exchange Audit Log
-- Tracks all token exchange operations for security auditing
CREATE TABLE IF NOT EXISTS token_exchange_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Token information
    subject_token_type VARCHAR(255) NOT NULL,  -- URN of subject token type
    requested_token_type VARCHAR(255),         -- URN of requested token type
    issued_token_type VARCHAR(255) NOT NULL,   -- URN of issued token type

    -- Subject information
    subject_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    subject_username VARCHAR(255),
    original_client_id UUID REFERENCES oauth2_clients(id) ON DELETE SET NULL,

    -- Actor information (for delegation scenarios)
    actor_id UUID,
    actor_type VARCHAR(50),                    -- 'user', 'service', 'application'
    delegation_enabled BOOLEAN DEFAULT false,

    -- Target information
    target_client_id UUID REFERENCES oauth2_clients(id) ON DELETE CASCADE,
    audience VARCHAR(500),                     -- Target audience
    resource VARCHAR(500),                     -- Target resource

    -- Scopes
    original_scopes TEXT[],                    -- Scopes from subject token
    requested_scopes TEXT[],                   -- Scopes requested in exchange
    granted_scopes TEXT[],                     -- Scopes in issued token

    -- Token lifecycle
    issued_token_id UUID REFERENCES oauth2_access_tokens(id) ON DELETE SET NULL,
    expires_in INTEGER,                        -- Token lifetime in seconds

    -- Operation status
    success BOOLEAN NOT NULL DEFAULT false,
    error_code VARCHAR(100),
    error_description TEXT,

    -- Metadata
    ip_address INET,                           -- Client IP address
    user_agent TEXT,                           -- Client user agent
    metadata JSONB,                            -- Additional context

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_token_exchange_subject_user ON token_exchange_audit(subject_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_exchange_actor ON token_exchange_audit(actor_id, created_at DESC) WHERE actor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_token_exchange_client ON token_exchange_audit(target_client_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_exchange_success ON token_exchange_audit(success, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_exchange_created_at ON token_exchange_audit(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_exchange_audience ON token_exchange_audit(audience) WHERE audience IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_token_exchange_metadata ON token_exchange_audit USING gin(metadata) WHERE metadata IS NOT NULL;

-- Add token exchange capability flag to clients table
ALTER TABLE oauth2_clients ADD COLUMN IF NOT EXISTS token_exchange_enabled BOOLEAN DEFAULT false;

-- Add actor/delegation claims support to oauth2_access_tokens
ALTER TABLE oauth2_access_tokens ADD COLUMN IF NOT EXISTS actor_id UUID;
ALTER TABLE oauth2_access_tokens ADD COLUMN IF NOT EXISTS delegation_enabled BOOLEAN DEFAULT false;
ALTER TABLE oauth2_access_tokens ADD COLUMN IF NOT EXISTS audience VARCHAR(500);
ALTER TABLE oauth2_access_tokens ADD COLUMN IF NOT EXISTS resource VARCHAR(500);
ALTER TABLE oauth2_access_tokens ADD COLUMN IF NOT EXISTS token_exchange_source VARCHAR(255); -- 'original' or 'exchanged'

-- Index for actor-related queries
CREATE INDEX IF NOT EXISTS idx_oauth2_tokens_actor ON oauth2_access_tokens(actor_id) WHERE actor_id IS NOT NULL;

-- Add comment for documentation
COMMENT ON TABLE token_exchange_audit IS 'Audit log for OAuth 2.0 Token Exchange (RFC 8693) operations';
COMMENT ON COLUMN token_exchange_audit.subject_token_type IS 'URN identifier of the subject token type (e.g., urn:ietf:params:oauth:token-type:access_token)';
COMMENT ON COLUMN token_exchange_audit.delegation_enabled IS 'True if this was a delegation scenario with actor token';
COMMENT ON COLUMN token_exchange_audit.audience IS 'Target audience for the issued token (RFC 8693 audience parameter)';
COMMENT ON COLUMN token_exchange_audit.resource IS 'Target resource for the issued token (RFC 8693 resource parameter)';

-- Grant appropriate permissions
-- GRANT SELECT ON token_exchange_audit TO authenc_reader;
-- GRANT INSERT ON token_exchange_audit TO authenc_writer;
-- GRANT SELECT, UPDATE ON clients TO authenc_writer;
