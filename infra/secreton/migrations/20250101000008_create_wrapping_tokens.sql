SET search_path = secreton, public;
-- Create wrapping_tokens table for response wrapping
-- Supports one-time token mechanism for secure secret distribution

CREATE TABLE IF NOT EXISTS wrapping_tokens (
    -- Primary key (wrapping token)
    token VARCHAR(255) PRIMARY KEY,

    -- Encrypted data (AES-256-GCM encrypted)
    encrypted_data BYTEA NOT NULL,

    -- Encryption metadata (algorithm, key, nonce)
    encryption_metadata JSONB NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    -- Namespace for multi-tenancy
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',

    -- Token status: 'Active', 'Unwrapped', 'Expired'
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Unwrapped', 'Expired')),

    -- Original data size (before encryption)
    data_size INTEGER NOT NULL,

    -- Audit fields
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX idx_wrapping_tokens_expires_at ON wrapping_tokens(expires_at);
CREATE INDEX idx_wrapping_tokens_namespace ON wrapping_tokens(namespace);
CREATE INDEX idx_wrapping_tokens_status ON wrapping_tokens(status);
CREATE INDEX idx_wrapping_tokens_created_at ON wrapping_tokens(created_at);

-- Composite index for cleanup queries
CREATE INDEX idx_wrapping_tokens_status_expires ON wrapping_tokens(status, expires_at);
CREATE INDEX idx_wrapping_tokens_namespace_status ON wrapping_tokens(namespace, status);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_wrapping_tokens_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_update_wrapping_tokens_updated_at
    BEFORE UPDATE ON wrapping_tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_wrapping_tokens_updated_at();

-- Create function to automatically expire tokens
CREATE OR REPLACE FUNCTION expire_wrapping_tokens()
RETURNS INTEGER AS $$
DECLARE
    expired_count INTEGER;
BEGIN
    UPDATE wrapping_tokens
    SET status = 'Expired'
    WHERE status = 'Active'
      AND expires_at < NOW();

    GET DIAGNOSTICS expired_count = ROW_COUNT;
    RETURN expired_count;
END;
$$ LANGUAGE plpgsql;

-- Create view for active wrapping tokens with time remaining
CREATE OR REPLACE VIEW active_wrapping_tokens AS
SELECT
    token,
    created_at,
    expires_at,
    namespace,
    status,
    data_size,
    EXTRACT(EPOCH FROM (expires_at - NOW())) AS ttl_remaining_seconds,
    CASE
        WHEN expires_at < NOW() THEN 'expired'
        WHEN expires_at < NOW() + INTERVAL '1 minute' THEN 'expiring_soon'
        ELSE 'active'
    END AS health_status,
    updated_at
FROM wrapping_tokens
WHERE status = 'Active';

-- Create view for wrapping token statistics
CREATE OR REPLACE VIEW wrapping_token_stats AS
SELECT
    namespace,
    status,
    COUNT(*) AS token_count,
    SUM(data_size) AS total_data_size,
    AVG(data_size) AS avg_data_size,
    MIN(created_at) AS oldest_token,
    MAX(created_at) AS newest_token,
    COUNT(CASE WHEN expires_at < NOW() THEN 1 END) AS expired_count,
    COUNT(CASE WHEN expires_at >= NOW() THEN 1 END) AS active_count
FROM wrapping_tokens
GROUP BY namespace, status;

-- Add comments for documentation
COMMENT ON TABLE wrapping_tokens IS 'One-time wrapping tokens for secure secret distribution';
COMMENT ON COLUMN wrapping_tokens.token IS 'Unique wrapping token (UUID with wrap_ prefix)';
COMMENT ON COLUMN wrapping_tokens.encrypted_data IS 'Encrypted wrapped data (AES-256-GCM)';
COMMENT ON COLUMN wrapping_tokens.encryption_metadata IS 'Encryption details (algorithm, key, nonce)';
COMMENT ON COLUMN wrapping_tokens.created_at IS 'When the token was created';
COMMENT ON COLUMN wrapping_tokens.expires_at IS 'When the token will expire';
COMMENT ON COLUMN wrapping_tokens.namespace IS 'Namespace for multi-tenancy isolation';
COMMENT ON COLUMN wrapping_tokens.status IS 'Token status: Active, Unwrapped, or Expired';
COMMENT ON COLUMN wrapping_tokens.data_size IS 'Original data size in bytes (before encryption)';
