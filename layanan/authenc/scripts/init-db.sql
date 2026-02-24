-- ============================================================================
-- Authenc Database Initialization Script
-- ============================================================================
-- This script initializes the PostgreSQL database for Authenc IAM service

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS authenc;
CREATE SCHEMA IF NOT EXISTS audit;
CREATE SCHEMA IF NOT EXISTS captcha;

-- Set default search path
ALTER DATABASE authenc SET search_path = authenc, public;

-- ============================================================================
-- Authenc Core Tables
-- ============================================================================

-- Users table
CREATE TABLE IF NOT EXISTS authenc.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended', 'deleted')),
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP WITH TIME ZONE,
    created_by UUID,
    updated_by UUID
);

CREATE INDEX idx_users_username ON authenc.users(username);
CREATE INDEX idx_users_email ON authenc.users(email);
CREATE INDEX idx_users_status ON authenc.users(status);
CREATE INDEX idx_users_created_at ON authenc.users(created_at);

-- Roles table
CREATE TABLE IF NOT EXISTS authenc.roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB DEFAULT '[]'::jsonb,
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_roles_name ON authenc.roles(name);

-- User roles mapping
CREATE TABLE IF NOT EXISTS authenc.user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES authenc.users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES authenc.roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    assigned_by UUID,
    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON authenc.user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON authenc.user_roles(role_id);

-- Sessions table
CREATE TABLE IF NOT EXISTS authenc.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES authenc.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_sessions_user_id ON authenc.sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON authenc.sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON authenc.sessions(is_active);

-- MFA Devices table
CREATE TABLE IF NOT EXISTS authenc.mfa_devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES authenc.users(id) ON DELETE CASCADE,
    device_type VARCHAR(50) NOT NULL CHECK (device_type IN ('totp', 'sms', 'email', 'backup')),
    device_name VARCHAR(255),
    secret_hash VARCHAR(255),
    is_verified BOOLEAN DEFAULT FALSE,
    is_primary BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP WITH TIME ZONE,
    backup_codes JSONB
);

CREATE INDEX idx_mfa_devices_user_id ON authenc.mfa_devices(user_id);
CREATE INDEX idx_mfa_devices_device_type ON authenc.mfa_devices(device_type);

-- ============================================================================
-- CAPTCHA Tables
-- ============================================================================

-- CAPTCHA challenges
CREATE TABLE IF NOT EXISTS captcha.challenges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES authenc.users(id) ON DELETE SET NULL,
    challenge_type VARCHAR(50) NOT NULL,
    difficulty_level INT DEFAULT 3,
    challenge_data JSONB NOT NULL,
    answer_hash VARCHAR(255),
    is_solved BOOLEAN DEFAULT FALSE,
    attempts INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    solved_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_challenges_user_id ON captcha.challenges(user_id);
CREATE INDEX idx_challenges_created_at ON captcha.challenges(created_at);
CREATE INDEX idx_challenges_expires_at ON captcha.challenges(expires_at);

-- CAPTCHA metrics
CREATE TABLE IF NOT EXISTS captcha.metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    challenge_id UUID REFERENCES captcha.challenges(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL,
    metric_value JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metrics_challenge_id ON captcha.metrics(challenge_id);
CREATE INDEX idx_metrics_recorded_at ON captcha.metrics(recorded_at);

-- ============================================================================
# Configuration Management Tables
# ============================================================================

-- Application configuration (centralized management)
CREATE TABLE IF NOT EXISTS authenc.configuration (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(255) NOT NULL UNIQUE,
    value JSONB NOT NULL,
    description TEXT,
    is_secret BOOLEAN DEFAULT FALSE,
    is_system BOOLEAN DEFAULT FALSE,
    category VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_by UUID REFERENCES authenc.users(id) ON DELETE SET NULL,
    version INT DEFAULT 1
);

CREATE INDEX idx_config_key ON authenc.configuration(key);
CREATE INDEX idx_config_category ON authenc.configuration(category);
CREATE INDEX idx_config_is_secret ON authenc.configuration(is_secret);
CREATE INDEX idx_config_updated_at ON authenc.configuration(updated_at);

-- Configuration change history (audit trail)
CREATE TABLE IF NOT EXISTS authenc.configuration_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    config_id UUID NOT NULL REFERENCES authenc.configuration(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    changed_by UUID REFERENCES authenc.users(id) ON DELETE SET NULL,
    change_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_config_history_config_id ON authenc.configuration_history(config_id);
CREATE INDEX idx_config_history_changed_by ON authenc.configuration_history(changed_by);
CREATE INDEX idx_config_history_created_at ON authenc.configuration_history(created_at);

-- ============================================================================
-- Audit Tables
-- ============================================================================

-- Audit logs
CREATE TABLE IF NOT EXISTS audit.logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES authenc.users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    status VARCHAR(50) DEFAULT 'success' CHECK (status IN ('success', 'failure', 'warning')),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_logs_user_id ON audit.logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit.logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit.logs(created_at);
CREATE INDEX idx_audit_logs_resource_type ON audit.logs(resource_type);

-- ============================================================================
-- Default Roles
-- ============================================================================

INSERT INTO authenc.roles (name, description, is_system, permissions) VALUES
    ('admin', 'Administrator with full access', TRUE, '["*"]'::jsonb),
    ('user', 'Regular user', TRUE, '["read:profile", "write:profile", "read:sessions"]'::jsonb),
    ('vault-admin', 'Vault administrator', TRUE, '["manage:vault", "read:vault", "write:vault"]'::jsonb),
    ('audit-viewer', 'Audit log viewer', TRUE, '["read:audit"]'::jsonb)
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- Default Configuration Values
-- ============================================================================

-- Server Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('server.host', '"0.0.0.0"'::jsonb, 'Server host address', 'server', TRUE),
    ('server.port', '8088'::jsonb, 'Server port', 'server', TRUE),
    ('server.grpc_port', '9088'::jsonb, 'gRPC port', 'server', TRUE),
    ('server.grpc_enabled', 'true'::jsonb, 'Enable gRPC', 'server', TRUE),
    ('server.tls_enabled', 'false'::jsonb, 'Enable TLS', 'server', TRUE),
    ('server.max_connections', '200'::jsonb, 'Maximum connections', 'server', TRUE)
ON CONFLICT (key) DO NOTHING;

-- Security Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('security.jwt_expiry', '3600'::jsonb, 'JWT expiration in seconds', 'security', TRUE),
    ('security.password_min_length', '12'::jsonb, 'Minimum password length', 'security', TRUE),
    ('security.password_salt_rounds', '12'::jsonb, 'Password salt rounds', 'security', TRUE),
    ('security.brute_force_max_attempts', '5'::jsonb, 'Max brute force attempts', 'security', TRUE),
    ('security.brute_force_window_seconds', '300'::jsonb, 'Brute force window in seconds', 'security', TRUE)
ON CONFLICT (key) DO NOTHING;

-- Rate Limiting Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('rate_limit.enabled', 'true'::jsonb, 'Enable rate limiting', 'rate_limit', TRUE),
    ('rate_limit.requests_per_minute', '100'::jsonb, 'Requests per minute', 'rate_limit', TRUE),
    ('rate_limit.burst_size', '150'::jsonb, 'Burst size', 'rate_limit', TRUE),
    ('adaptive_rate_limit.enabled', 'true'::jsonb, 'Enable adaptive rate limiting', 'rate_limit', TRUE),
    ('adaptive_rate_limit.baseline_requests_per_minute', '100'::jsonb, 'Baseline RPM', 'rate_limit', TRUE),
    ('adaptive_rate_limit.max_requests_per_minute', '200'::jsonb, 'Max RPM', 'rate_limit', TRUE),
    ('adaptive_rate_limit.min_requests_per_minute', '50'::jsonb, 'Min RPM', 'rate_limit', TRUE)
ON CONFLICT (key) DO NOTHING;

-- CAPTCHA Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('captcha.enabled', 'true'::jsonb, 'Enable CAPTCHA', 'captcha', TRUE),
    ('captcha.default_difficulty', '3'::jsonb, 'Default difficulty level', 'captcha', TRUE),
    ('captcha.max_difficulty', '10'::jsonb, 'Maximum difficulty level', 'captcha', TRUE),
    ('captcha.failed_attempts_threshold', '2'::jsonb, 'Failed attempts before CAPTCHA', 'captcha', TRUE),
    ('captcha.max_attempts', '5'::jsonb, 'Maximum CAPTCHA attempts', 'captcha', TRUE),
    ('captcha.rate_limit_duration', '300'::jsonb, 'Rate limit duration in seconds', 'captcha', TRUE),
    ('captcha.challenge_expiration', '300'::jsonb, 'Challenge expiration in seconds', 'captcha', TRUE),
    ('captcha.behavioral_analysis_enabled', 'true'::jsonb, 'Enable behavioral analysis', 'captcha', TRUE),
    ('captcha.accessibility_enabled', 'true'::jsonb, 'Enable accessibility features', 'captcha', TRUE)
ON CONFLICT (key) DO NOTHING;

-- Features Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('features.enable_registration', 'true'::jsonb, 'Enable user registration', 'features', TRUE),
    ('features.enable_password_reset', 'true'::jsonb, 'Enable password reset', 'features', TRUE),
    ('features.enable_email_verification', 'true'::jsonb, 'Enable email verification', 'features', TRUE),
    ('features.enable_multi_factor_auth', 'true'::jsonb, 'Enable MFA', 'features', TRUE),
    ('features.enable_api_docs', 'false'::jsonb, 'Enable API documentation', 'features', TRUE),
    ('features.enable_metrics', 'true'::jsonb, 'Enable metrics', 'features', TRUE),
    ('features.enable_health_checks', 'true'::jsonb, 'Enable health checks', 'features', TRUE),
    ('features.enable_rate_limiting', 'true'::jsonb, 'Enable rate limiting', 'features', TRUE),
    ('features.enable_caching', 'true'::jsonb, 'Enable caching', 'features', TRUE),
    ('features.enable_compression', 'true'::jsonb, 'Enable compression', 'features', TRUE),
    ('features.enable_cors', 'true'::jsonb, 'Enable CORS', 'features', TRUE),
    ('features.enable_input_validation', 'true'::jsonb, 'Enable input validation', 'features', TRUE)
ON CONFLICT (key) DO NOTHING;

-- Observability Configuration
INSERT INTO authenc.configuration (key, value, description, category, is_system) VALUES
    ('observability.log_level', '"info"'::jsonb, 'Log level', 'observability', TRUE),
    ('observability.enable_metrics', 'true'::jsonb, 'Enable metrics', 'observability', TRUE),
    ('observability.enable_tracing', 'true'::jsonb, 'Enable tracing', 'observability', TRUE),
    ('observability.structured_logging', 'true'::jsonb, 'Enable structured logging', 'observability', TRUE)
ON CONFLICT (key) DO NOTHING;

-- ============================================================================
-- Cleanup and Maintenance Functions
-- ============================================================================

-- Function to clean up expired sessions
CREATE OR REPLACE FUNCTION authenc.cleanup_expired_sessions()
RETURNS void AS $$
BEGIN
    DELETE FROM authenc.sessions WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired CAPTCHA challenges
CREATE OR REPLACE FUNCTION captcha.cleanup_expired_challenges()
RETURNS void AS $$
BEGIN
    DELETE FROM captcha.challenges WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- Function to update user updated_at timestamp
CREATE OR REPLACE FUNCTION authenc.update_user_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER trigger_update_user_timestamp
BEFORE UPDATE ON authenc.users
FOR EACH ROW
EXECUTE FUNCTION authenc.update_user_timestamp();

-- ============================================================================
-- Grants and Permissions
-- ============================================================================

-- Grant permissions to authenc user (if different from postgres)
-- Uncomment and modify if needed:
-- GRANT USAGE ON SCHEMA authenc, audit, captcha TO authenc_user;
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA authenc, audit, captcha TO authenc_user;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA authenc, audit, captcha TO authenc_user;
