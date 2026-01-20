-- Secreton Database Initialization Script
-- This script creates the required tables for Secreton storage backend

-- Create extension for UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Secrets storage table
CREATE TABLE IF NOT EXISTS secrets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    path VARCHAR(1024) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    data BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(namespace, path, version)
);

-- Seal status table
CREATE TABLE IF NOT EXISTS seal_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    initialized BOOLEAN NOT NULL DEFAULT FALSE,
    sealed BOOLEAN NOT NULL DEFAULT TRUE,
    threshold INTEGER NOT NULL DEFAULT 3,
    total_shares INTEGER NOT NULL DEFAULT 5,
    master_key_hash BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Transit keys table
CREATE TABLE IF NOT EXISTS transit_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    key_type VARCHAR(50) NOT NULL,
    key_data BYTEA NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    min_decryption_version INTEGER NOT NULL DEFAULT 1,
    min_encryption_version INTEGER NOT NULL DEFAULT 1,
    deletion_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    exportable BOOLEAN NOT NULL DEFAULT FALSE,
    allow_plaintext_backup BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- PKI certificates table
CREATE TABLE IF NOT EXISTS pki_certificates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    serial_number VARCHAR(100) NOT NULL UNIQUE,
    common_name VARCHAR(255) NOT NULL,
    issuer VARCHAR(255) NOT NULL,
    subject VARCHAR(1024) NOT NULL,
    certificate TEXT NOT NULL,
    private_key BYTEA,
    not_before TIMESTAMPTZ NOT NULL,
    not_after TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    revocation_reason VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    action VARCHAR(100) NOT NULL,
    actor VARCHAR(255),
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    namespace VARCHAR(255),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);

-- Leases table
CREATE TABLE IF NOT EXISTS leases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    lease_id VARCHAR(255) NOT NULL UNIQUE,
    secret_path VARCHAR(1024) NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    ttl_seconds INTEGER NOT NULL,
    renewable BOOLEAN NOT NULL DEFAULT TRUE,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'
);

-- Tokens table
CREATE TABLE IF NOT EXISTS tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    accessor VARCHAR(128) NOT NULL UNIQUE,
    policies TEXT[] NOT NULL DEFAULT '{}',
    path VARCHAR(255),
    role VARCHAR(255),
    ttl_seconds INTEGER,
    explicit_max_ttl INTEGER,
    num_uses INTEGER DEFAULT 0,
    max_uses INTEGER,
    renewable BOOLEAN NOT NULL DEFAULT TRUE,
    orphan BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_renewal_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'
);

-- Policies table
CREATE TABLE IF NOT EXISTS policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    policy_type VARCHAR(50) NOT NULL DEFAULT 'acl',
    policy_data TEXT NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Namespaces table
CREATE TABLE IF NOT EXISTS namespaces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    path VARCHAR(1024) NOT NULL UNIQUE,
    parent_path VARCHAR(1024),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_secrets_namespace_path ON secrets(namespace, path);
CREATE INDEX IF NOT EXISTS idx_secrets_deleted_at ON secrets(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_leases_expires_at ON leases(expires_at);
CREATE INDEX IF NOT EXISTS idx_tokens_expires_at ON tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_pki_not_after ON pki_certificates(not_after);

-- Insert default seal status
INSERT INTO seal_status (initialized, sealed, threshold, total_shares)
SELECT FALSE, TRUE, 3, 5
WHERE NOT EXISTS (SELECT 1 FROM seal_status LIMIT 1);

-- Insert default namespace
INSERT INTO namespaces (path, parent_path)
SELECT 'default', NULL
WHERE NOT EXISTS (SELECT 1 FROM namespaces WHERE path = 'default');

-- Insert root policy
INSERT INTO policies (name, policy_type, policy_data, namespace)
SELECT 'root', 'acl', '{"path":{"*":{"capabilities":["create","read","update","delete","list","sudo"]}}}', 'default'
WHERE NOT EXISTS (SELECT 1 FROM policies WHERE name = 'root');

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO secreton;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO secreton;
