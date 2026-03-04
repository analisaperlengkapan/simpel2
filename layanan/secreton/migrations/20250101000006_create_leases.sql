-- Create leases table for secret lifecycle management
-- Supports TTL, renewal, revocation, and hierarchical lease relationships

CREATE TABLE IF NOT EXISTS leases (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,

    -- User/entity ID who owns the lease
    user_id VARCHAR(255) NOT NULL,

    -- Resource path (e.g., "/secret/data/db/password", "/dynamic/database/creds/readonly")
    resource VARCHAR(1024) NOT NULL,

    -- Resource type: 'kv', 'database', 'aws', 'transit', etc.
    resource_type VARCHAR(50) NOT NULL,

    -- Namespace for multi-tenancy
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',

    -- Lease status: 'active', 'revoked', 'expired'
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked', 'expired')),

    -- Timestamps
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expired_at TIMESTAMPTZ NOT NULL,
    last_renewed_at TIMESTAMPTZ,

    -- Renewal configuration
    renewable BOOLEAN NOT NULL DEFAULT TRUE,
    max_ttl BIGINT NOT NULL DEFAULT 86400, -- Maximum TTL in seconds (default 24 hours)
    renew_count INTEGER NOT NULL DEFAULT 0,
    max_renewals INTEGER, -- NULL means unlimited renewals

    -- Hierarchical lease relationships
    parent_id VARCHAR(255),

    -- Revocation callback (optional function to call on revocation)
    revoke_callback VARCHAR(512),

    -- Additional metadata (JSON object)
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraint for parent lease
    CONSTRAINT fk_parent_lease
        FOREIGN KEY (parent_id)
        REFERENCES leases(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX idx_leases_user_id ON leases(user_id);
CREATE INDEX idx_leases_resource ON leases(resource);
CREATE INDEX idx_leases_resource_type ON leases(resource_type);
CREATE INDEX idx_leases_namespace ON leases(namespace);
CREATE INDEX idx_leases_status ON leases(status);
CREATE INDEX idx_leases_expired_at ON leases(expired_at);
CREATE INDEX idx_leases_parent_id ON leases(parent_id);
CREATE INDEX idx_leases_issued_at ON leases(issued_at);

-- Composite index for common queries
CREATE INDEX idx_leases_status_expired ON leases(status, expired_at);
CREATE INDEX idx_leases_user_status ON leases(user_id, status);
CREATE INDEX idx_leases_namespace_status ON leases(namespace, status);

-- Create GIN index for JSONB metadata
CREATE INDEX idx_leases_metadata ON leases USING gin(metadata);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_leases_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_update_leases_updated_at
    BEFORE UPDATE ON leases
    FOR EACH ROW
    EXECUTE FUNCTION update_leases_updated_at();

-- Create function to automatically expire leases
CREATE OR REPLACE FUNCTION expire_old_leases()
RETURNS INTEGER AS $$
DECLARE
    expired_count INTEGER;
BEGIN
    UPDATE leases
    SET status = 'expired'
    WHERE status = 'active'
      AND expired_at < NOW();

    GET DIAGNOSTICS expired_count = ROW_COUNT;
    RETURN expired_count;
END;
$$ LANGUAGE plpgsql;

-- Create view for active leases with time remaining
CREATE OR REPLACE VIEW active_leases AS
SELECT
    id,
    user_id,
    resource,
    resource_type,
    namespace,
    status,
    issued_at,
    expired_at,
    last_renewed_at,
    renewable,
    max_ttl,
    renew_count,
    max_renewals,
    parent_id,
    EXTRACT(EPOCH FROM (expired_at - NOW())) AS ttl_remaining_seconds,
    CASE
        WHEN expired_at < NOW() THEN 'expired'
        WHEN expired_at < NOW() + INTERVAL '5 minutes' THEN 'expiring_soon'
        ELSE 'active'
    END AS health_status,
    metadata,
    created_at,
    updated_at
FROM leases
WHERE status = 'active';

-- Create view for lease hierarchy
CREATE OR REPLACE VIEW lease_hierarchy AS
WITH RECURSIVE hierarchy AS (
    -- Base case: root leases (no parent)
    SELECT
        id,
        user_id,
        resource,
        resource_type,
        namespace,
        status,
        parent_id,
        0 AS level,
        ARRAY[id]::VARCHAR[] AS ancestors,
        id::VARCHAR AS path
    FROM leases
    WHERE parent_id IS NULL

    UNION ALL

    -- Recursive case: child leases
    SELECT
        l.id,
        l.user_id,
        l.resource,
        l.resource_type,
        l.namespace,
        l.status,
        l.parent_id,
        h.level + 1,
        h.ancestors || l.id::VARCHAR,
        (h.path || ' -> ' || l.id)::VARCHAR
    FROM leases l
    INNER JOIN hierarchy h ON l.parent_id = h.id
)
SELECT
    id,
    user_id,
    resource,
    resource_type,
    namespace,
    status,
    parent_id,
    level,
    ancestors,
    path
FROM hierarchy
ORDER BY level, id;

-- Add comments for documentation
COMMENT ON TABLE leases IS 'Lease management for time-bound credentials and secrets';
COMMENT ON COLUMN leases.id IS 'Unique lease identifier (UUID)';
COMMENT ON COLUMN leases.user_id IS 'User or entity ID who owns this lease';
COMMENT ON COLUMN leases.resource IS 'Full path to the leased resource';
COMMENT ON COLUMN leases.resource_type IS 'Type of resource: kv, database, aws, transit, etc.';
COMMENT ON COLUMN leases.namespace IS 'Namespace for multi-tenancy isolation';
COMMENT ON COLUMN leases.status IS 'Lease status: active, revoked, or expired';
COMMENT ON COLUMN leases.issued_at IS 'When the lease was originally issued';
COMMENT ON COLUMN leases.expired_at IS 'When the lease will expire';
COMMENT ON COLUMN leases.last_renewed_at IS 'Last time the lease was renewed';
COMMENT ON COLUMN leases.renewable IS 'Whether this lease can be renewed';
COMMENT ON COLUMN leases.max_ttl IS 'Maximum TTL in seconds for this lease';
COMMENT ON COLUMN leases.renew_count IS 'Number of times this lease has been renewed';
COMMENT ON COLUMN leases.max_renewals IS 'Maximum number of renewals allowed (NULL = unlimited)';
COMMENT ON COLUMN leases.parent_id IS 'Parent lease ID for hierarchical relationships';
COMMENT ON COLUMN leases.revoke_callback IS 'Optional callback function to execute on revocation';
COMMENT ON COLUMN leases.metadata IS 'Additional metadata as JSON object';

