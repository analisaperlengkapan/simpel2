SET search_path = secreton, public;
-- Create namespaces table for hierarchical multi-tenancy
-- Supports SIMKARI organizational structure: Pusat -> Wilayah -> Satker

CREATE TABLE IF NOT EXISTS namespaces (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,

    -- Hierarchical path (e.g., "pusat/wilayah-sumut/satker-kja001")
    path VARCHAR(1024) NOT NULL UNIQUE,

    -- Parent namespace ID (NULL for root)
    parent_id VARCHAR(255),

    -- Display name
    name VARCHAR(512) NOT NULL,

    -- Namespace type: 'pusat', 'wilayah', 'satker'
    namespace_type VARCHAR(50) NOT NULL CHECK (namespace_type IN ('pusat', 'wilayah', 'satker')),

    -- Associated policies (JSON array of policy names)
    policies JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- Resource quotas (JSON object)
    quotas JSONB NOT NULL DEFAULT '{
        "max_secrets": 10000,
        "max_storage_bytes": 10737418240,
        "max_leases": 1000,
        "max_policies": 100,
        "current_usage": {
            "secrets_count": 0,
            "storage_bytes": 0,
            "leases_count": 0,
            "policies_count": 0
        }
    }'::jsonb,

    -- Additional metadata (JSON object)
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Created by user ID
    created_by VARCHAR(255) NOT NULL,

    -- Active status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Foreign key constraint for parent
    CONSTRAINT fk_parent_namespace
        FOREIGN KEY (parent_id)
        REFERENCES namespaces(id)
        ON DELETE RESTRICT
);

-- Create indexes for efficient queries
CREATE INDEX idx_namespaces_parent_id ON namespaces(parent_id);
CREATE INDEX idx_namespaces_namespace_type ON namespaces(namespace_type);
CREATE INDEX idx_namespaces_path ON namespaces USING btree(path);
CREATE INDEX idx_namespaces_is_active ON namespaces(is_active);
CREATE INDEX idx_namespaces_created_at ON namespaces(created_at);

-- Create GIN index for JSONB columns for efficient JSON queries
CREATE INDEX idx_namespaces_policies ON namespaces USING gin(policies);
CREATE INDEX idx_namespaces_metadata ON namespaces USING gin(metadata);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_namespaces_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_update_namespaces_updated_at
    BEFORE UPDATE ON namespaces
    FOR EACH ROW
    EXECUTE FUNCTION update_namespaces_updated_at();

-- Insert root namespace (Pusat)
INSERT INTO namespaces (
    id,
    path,
    parent_id,
    name,
    namespace_type,
    created_by,
    is_active
) VALUES (
    'pusat',
    'pusat',
    NULL,
    'Kejaksaan Agung Republik Indonesia',
    'pusat',
    'system',
    TRUE
) ON CONFLICT (id) DO NOTHING;

-- Create view for namespace hierarchy
CREATE OR REPLACE VIEW namespace_hierarchy AS
WITH RECURSIVE hierarchy AS (
    -- Base case: root namespace
    SELECT
        id,
        path,
        parent_id,
        name,
        namespace_type,
        0 AS level,
        ARRAY[id] AS ancestors
    FROM namespaces
    WHERE parent_id IS NULL

    UNION ALL

    -- Recursive case: child namespaces
    SELECT
        n.id,
        n.path,
        n.parent_id,
        n.name,
        n.namespace_type,
        h.level + 1,
        h.ancestors || n.id
    FROM namespaces n
    INNER JOIN hierarchy h ON n.parent_id = h.id
)
SELECT
    id,
    path,
    parent_id,
    name,
    namespace_type,
    level,
    ancestors
FROM hierarchy
ORDER BY level, path;

-- Add comments for documentation
COMMENT ON TABLE namespaces IS 'Hierarchical namespace structure for SIMKARI multi-tenancy';
COMMENT ON COLUMN namespaces.id IS 'Unique namespace identifier (e.g., pusat, wilayah-sumut, satker-kja001)';
COMMENT ON COLUMN namespaces.path IS 'Full hierarchical path from root to this namespace';
COMMENT ON COLUMN namespaces.parent_id IS 'Parent namespace ID (NULL for root)';
COMMENT ON COLUMN namespaces.namespace_type IS 'Type of namespace: pusat (central), wilayah (regional), satker (work unit)';
COMMENT ON COLUMN namespaces.policies IS 'JSON array of policy names associated with this namespace';
COMMENT ON COLUMN namespaces.quotas IS 'JSON object containing resource quotas and current usage';
COMMENT ON COLUMN namespaces.metadata IS 'JSON object for additional namespace metadata';
