-- User Sessions Table Enhancement
-- Enhanced session management with support for tokens, offline access, and security tracking
-- This migration adds new columns to the existing user_sessions table from 001_initial_schema.sql

-- Add new columns to user_sessions if they don't exist
-- Using DO block for idempotent ALTER TABLE operations
DO $$
BEGIN
    -- Add realm_id column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'realm_id') THEN
        ALTER TABLE user_sessions ADD COLUMN realm_id UUID REFERENCES realms(id) ON DELETE CASCADE;
    END IF;

    -- Add client_id column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'client_id') THEN
        ALTER TABLE user_sessions ADD COLUMN client_id UUID REFERENCES oauth2_clients(id) ON DELETE SET NULL;
    END IF;

    -- Add offline_token_hash column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'offline_token_hash') THEN
        ALTER TABLE user_sessions ADD COLUMN offline_token_hash VARCHAR(64) UNIQUE;
    END IF;

    -- Add started_at column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'started_at') THEN
        ALTER TABLE user_sessions ADD COLUMN started_at TIMESTAMP NOT NULL DEFAULT NOW();
    END IF;

    -- Add last_accessed column (if last_activity_at exists, rename it)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'last_accessed') THEN
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'last_activity_at') THEN
            ALTER TABLE user_sessions RENAME COLUMN last_activity_at TO last_accessed;
        ELSE
            ALTER TABLE user_sessions ADD COLUMN last_accessed TIMESTAMP NOT NULL DEFAULT NOW();
        END IF;
    END IF;

    -- Add idle_expires_at column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'idle_expires_at') THEN
        ALTER TABLE user_sessions ADD COLUMN idle_expires_at TIMESTAMP;
    END IF;

    -- Add token rotation tracking columns
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'refresh_count') THEN
        ALTER TABLE user_sessions ADD COLUMN refresh_count INTEGER NOT NULL DEFAULT 0;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'refresh_token_expires_at') THEN
        ALTER TABLE user_sessions ADD COLUMN refresh_token_expires_at TIMESTAMP;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'offline_token_expires_at') THEN
        ALTER TABLE user_sessions ADD COLUMN offline_token_expires_at TIMESTAMP;
    END IF;

    -- Add session state columns
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'revoked') THEN
        ALTER TABLE user_sessions ADD COLUMN revoked BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'revoked_at') THEN
        ALTER TABLE user_sessions ADD COLUMN revoked_at TIMESTAMP;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'revoked_reason') THEN
        ALTER TABLE user_sessions ADD COLUMN revoked_reason TEXT;
    END IF;

    -- Add metadata columns
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'authentication_method') THEN
        ALTER TABLE user_sessions ADD COLUMN authentication_method VARCHAR(50);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'protocol') THEN
        ALTER TABLE user_sessions ADD COLUMN protocol VARCHAR(20);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'updated_at') THEN
        ALTER TABLE user_sessions ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT NOW();
    END IF;

    -- Rename token_hash to use VARCHAR(64) if needed (already compatible)
    -- Rename refresh_token_hash to use VARCHAR(64) if needed (already compatible)
END $$;

-- Device Sessions Table
-- Tracks device-specific session information for security and analytics
CREATE TABLE IF NOT EXISTS device_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_session_id UUID REFERENCES user_sessions(id) ON DELETE CASCADE,

    -- Session information
    session_identifier VARCHAR(255) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_activity TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Location and context
    ip_address INET,
    location JSONB, -- { country, city, latitude, longitude }

    -- Risk assessment
    risk_score DOUBLE PRECISION DEFAULT 0.0,
    risk_factors JSONB, -- Array of detected risk factors

    -- Session state
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ended_at TIMESTAMP,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Offline Tokens Table
-- Stores long-lived offline tokens for background access
CREATE TABLE IF NOT EXISTS offline_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES oauth2_clients(id) ON DELETE CASCADE,

    -- Token information
    token_hash VARCHAR(64) NOT NULL UNIQUE,

    -- Offline token lifecycle
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP,
    last_used_at TIMESTAMP,

    -- Metadata
    scope TEXT,
    data JSONB,

    -- State
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMP,

    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Refresh Token Rotation History
-- Tracks refresh token rotation for security auditing
CREATE TABLE IF NOT EXISTS refresh_token_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,

    old_token_hash VARCHAR(64) NOT NULL,
    new_token_hash VARCHAR(64) NOT NULL,

    rotated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    client_ip INET,
    user_agent TEXT,

    -- Detection flags
    suspicious BOOLEAN NOT NULL DEFAULT FALSE,
    reason TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_realm_id ON user_sessions(realm_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_client_id ON user_sessions(client_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_user_sessions_refresh_token_hash ON user_sessions(refresh_token_hash) WHERE refresh_token_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_sessions_offline_token_hash ON user_sessions(offline_token_hash) WHERE offline_token_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at) WHERE NOT revoked;
CREATE INDEX IF NOT EXISTS idx_user_sessions_idle_expires_at ON user_sessions(idle_expires_at) WHERE NOT revoked AND idle_expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_sessions_active ON user_sessions(user_id, realm_id) WHERE NOT revoked;

CREATE INDEX IF NOT EXISTS idx_device_sessions_device_id ON device_sessions(device_id);
CREATE INDEX IF NOT EXISTS idx_device_sessions_user_id ON device_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_device_sessions_user_session_id ON device_sessions(user_session_id);
CREATE INDEX IF NOT EXISTS idx_device_sessions_active ON device_sessions(user_id) WHERE is_active;
CREATE INDEX IF NOT EXISTS idx_device_sessions_last_activity ON device_sessions(last_activity) WHERE is_active;
CREATE INDEX IF NOT EXISTS idx_device_sessions_risk_score ON device_sessions(risk_score) WHERE is_active AND risk_score > 0.5;

CREATE INDEX IF NOT EXISTS idx_offline_tokens_user_id ON offline_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_offline_tokens_realm_id ON offline_tokens(realm_id);
CREATE INDEX IF NOT EXISTS idx_offline_tokens_client_id ON offline_tokens(client_id);
CREATE INDEX IF NOT EXISTS idx_offline_tokens_token_hash ON offline_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_offline_tokens_active ON offline_tokens(user_id, realm_id) WHERE NOT revoked;

CREATE INDEX IF NOT EXISTS idx_refresh_token_history_session_id ON refresh_token_history(user_session_id);
CREATE INDEX IF NOT EXISTS idx_refresh_token_history_rotated_at ON refresh_token_history(rotated_at);
CREATE INDEX IF NOT EXISTS idx_refresh_token_history_suspicious ON refresh_token_history(user_session_id) WHERE suspicious;
