-- Create dynamic_roles table for database secrets engine
CREATE TABLE IF NOT EXISTS dynamic_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    db_name VARCHAR(255) NOT NULL,
    default_ttl INTEGER NOT NULL DEFAULT 3600,
    max_ttl INTEGER NOT NULL DEFAULT 86400,
    creation_statements TEXT[] NOT NULL,
    revocation_statements TEXT[] NOT NULL,
    rotation_statements TEXT[],
    renew_statements TEXT[],
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index on name for fast lookups
CREATE INDEX idx_dynamic_roles_name ON dynamic_roles(name);
CREATE INDEX idx_dynamic_roles_namespace ON dynamic_roles(namespace);

-- Create database_connections table for storing connection configurations
CREATE TABLE IF NOT EXISTS database_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    db_type VARCHAR(50) NOT NULL,
    connection_url TEXT NOT NULL,
    max_open_connections INTEGER NOT NULL DEFAULT 4,
    max_idle_connections INTEGER NOT NULL DEFAULT 2,
    max_connection_lifetime INTEGER NOT NULL DEFAULT 3600,
    verify_connection BOOLEAN NOT NULL DEFAULT TRUE,
    root_rotation_statements TEXT[],
    username VARCHAR(255),
    password_encrypted BYTEA,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index on name for fast lookups
CREATE INDEX idx_database_connections_name ON database_connections(name);
CREATE INDEX idx_database_connections_namespace ON database_connections(namespace);

-- Create dynamic_credentials table for tracking active credentials
CREATE TABLE IF NOT EXISTS dynamic_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_id VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL,
    password_encrypted BYTEA NOT NULL,
    connection_url TEXT,
    role_name VARCHAR(255) NOT NULL,
    db_name VARCHAR(255) NOT NULL,
    lease_id VARCHAR(255),
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for fast lookups
CREATE INDEX idx_dynamic_credentials_credential_id ON dynamic_credentials(credential_id);
CREATE INDEX idx_dynamic_credentials_role_name ON dynamic_credentials(role_name);
CREATE INDEX idx_dynamic_credentials_lease_id ON dynamic_credentials(lease_id);
CREATE INDEX idx_dynamic_credentials_expires_at ON dynamic_credentials(expires_at);
CREATE INDEX idx_dynamic_credentials_namespace ON dynamic_credentials(namespace);

-- Add comments
COMMENT ON TABLE dynamic_roles IS 'Database roles for dynamic credential generation';
COMMENT ON TABLE database_connections IS 'Database connection configurations';
COMMENT ON TABLE dynamic_credentials IS 'Active dynamic database credentials';

