-- Create sealed_master_keys table for auto-unseal capability
-- This table stores master keys encrypted by external KMS providers (AWS KMS, GCP KMS, Azure Key Vault, Transit)
-- Only one sealed master key should be active at a time

CREATE TABLE IF NOT EXISTS sealed_master_keys (
    -- Unique identifier for the sealed key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Auto-unseal provider type (aws-kms, gcp-kms, azure-kv, transit)
    provider_type VARCHAR(50) NOT NULL,

    -- Provider-specific key identifier (KMS key ARN, key name, etc.)
    provider_key_id VARCHAR(500) NOT NULL,

    -- Provider region (for AWS/GCP/Azure)
    provider_region VARCHAR(100),

    -- Provider endpoint (for custom endpoints or Transit)
    provider_endpoint VARCHAR(500),

    -- Encrypted master key (encrypted by the KMS provider)
    encrypted_master_key BYTEA NOT NULL,

    -- SHA-256 checksum of encrypted data for integrity verification
    checksum VARCHAR(64) NOT NULL,

    -- Whether this is the active sealed key
    is_active BOOLEAN NOT NULL DEFAULT false,

    -- Version number for key rotation
    version INTEGER NOT NULL DEFAULT 1,

    -- Creation timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Last updated timestamp
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Additional metadata in JSON format
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Constraints
    CONSTRAINT sealed_master_keys_provider_type_check CHECK (
        provider_type IN ('aws-kms', 'gcp-kms', 'azure-kv', 'transit')
    ),
    CONSTRAINT sealed_master_keys_version_check CHECK (version > 0)
);

-- Unique constraint: only one active sealed key at a time
CREATE UNIQUE INDEX idx_sealed_master_keys_active
    ON sealed_master_keys(is_active)
    WHERE is_active = true;

-- Index for querying by provider type
CREATE INDEX idx_sealed_master_keys_provider_type
    ON sealed_master_keys(provider_type);

-- Index for querying by version (for key rotation)
CREATE INDEX idx_sealed_master_keys_version
    ON sealed_master_keys(version DESC);

-- Index for querying by creation time
CREATE INDEX idx_sealed_master_keys_created_at
    ON sealed_master_keys(created_at DESC);

-- Comments for documentation
COMMENT ON TABLE sealed_master_keys IS 'Stores master keys encrypted by external KMS providers for auto-unseal capability';
COMMENT ON COLUMN sealed_master_keys.id IS 'Unique identifier for the sealed key entry';
COMMENT ON COLUMN sealed_master_keys.provider_type IS 'Auto-unseal provider type: aws-kms, gcp-kms, azure-kv, or transit';
COMMENT ON COLUMN sealed_master_keys.provider_key_id IS 'Provider-specific key identifier (e.g., AWS KMS key ARN, GCP key name)';
COMMENT ON COLUMN sealed_master_keys.provider_region IS 'Cloud provider region (for AWS/GCP/Azure)';
COMMENT ON COLUMN sealed_master_keys.provider_endpoint IS 'Custom endpoint URL (for Transit or custom KMS endpoints)';
COMMENT ON COLUMN sealed_master_keys.encrypted_master_key IS 'Master key encrypted by the KMS provider';
COMMENT ON COLUMN sealed_master_keys.checksum IS 'SHA-256 checksum of encrypted_master_key for integrity verification';
COMMENT ON COLUMN sealed_master_keys.is_active IS 'Whether this is the currently active sealed key (only one can be active)';
COMMENT ON COLUMN sealed_master_keys.version IS 'Version number for tracking key rotation';
COMMENT ON COLUMN sealed_master_keys.created_at IS 'Timestamp when the sealed key was created';
COMMENT ON COLUMN sealed_master_keys.updated_at IS 'Timestamp when the sealed key was last updated';
COMMENT ON COLUMN sealed_master_keys.metadata IS 'Additional provider-specific metadata in JSON format';
