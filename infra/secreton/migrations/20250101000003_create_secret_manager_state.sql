SET search_path = secreton, public;
-- Create vault_state table for storing encrypted master key and seal configuration
-- This table stores the vault's seal state and encrypted master key

CREATE TABLE IF NOT EXISTS vault_state (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Encrypted master key (encrypted with seal key derived from Shamir shares)
    encrypted_master_key BYTEA NOT NULL,

    -- Seal configuration (JSON)
    seal_config JSONB NOT NULL,

    -- Shamir commitments for Feldman VSS verification (JSON array)
    shamir_commitments JSONB NOT NULL,

    -- Encryption metadata for the master key
    encryption_metadata JSONB NOT NULL,

    -- Version for key rotation tracking
    version INTEGER NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure only one vault state exists
    CONSTRAINT single_vault_state CHECK (id = '00000000-0000-0000-0000-000000000000'::UUID)
);

-- Index for quick lookups (though there should only be one row)
CREATE INDEX IF NOT EXISTS idx_vault_state_version ON vault_state(version);

-- Add comment for documentation
COMMENT ON TABLE vault_state IS 'Stores the encrypted master key and seal configuration for the vault';
COMMENT ON COLUMN vault_state.encrypted_master_key IS 'Master key encrypted with seal key derived from Shamir shares';
COMMENT ON COLUMN vault_state.seal_config IS 'Seal configuration including threshold and share count';
COMMENT ON COLUMN vault_state.shamir_commitments IS 'Feldman VSS commitments for share verification';
COMMENT ON COLUMN vault_state.encryption_metadata IS 'Metadata about the encryption algorithm and parameters used';
COMMENT ON COLUMN vault_state.version IS 'Version number for key rotation tracking';
