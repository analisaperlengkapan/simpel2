SET search_path = secreton, public;
-- Create policies table for policy management
CREATE TABLE IF NOT EXISTS policies (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',
    description TEXT,
    rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255) NOT NULL,
    updated_by VARCHAR(255),

    -- Unique constraint on name and namespace
    CONSTRAINT policies_name_namespace_unique UNIQUE (name, namespace)
);

-- Create index on namespace for faster lookups
CREATE INDEX IF NOT EXISTS idx_policies_namespace ON policies(namespace);

-- Create index on name for faster lookups
CREATE INDEX IF NOT EXISTS idx_policies_name ON policies(name);

-- Create index on is_active for filtering
CREATE INDEX IF NOT EXISTS idx_policies_is_active ON policies(is_active);

-- Create index on created_at for sorting
CREATE INDEX IF NOT EXISTS idx_policies_created_at ON policies(created_at DESC);

-- Create policy evaluation statistics table
CREATE TABLE IF NOT EXISTS policy_stats (
    id BIGSERIAL PRIMARY KEY,
    policy_id BIGINT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    evaluations_total BIGINT NOT NULL DEFAULT 0,
    evaluations_allowed BIGINT NOT NULL DEFAULT 0,
    evaluations_denied BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    last_evaluated_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Unique constraint on policy_id
    CONSTRAINT policy_stats_policy_id_unique UNIQUE (policy_id)
);

-- Create index on policy_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_policy_stats_policy_id ON policy_stats(policy_id);

-- Create policy version history table for audit trail
CREATE TABLE IF NOT EXISTS policy_versions (
    id BIGSERIAL PRIMARY KEY,
    policy_id BIGINT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    rules JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255) NOT NULL,

    -- Unique constraint on policy_id and version
    CONSTRAINT policy_versions_policy_version_unique UNIQUE (policy_id, version)
);

-- Create index on policy_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_policy_versions_policy_id ON policy_versions(policy_id);

-- Create index on version for sorting
CREATE INDEX IF NOT EXISTS idx_policy_versions_version ON policy_versions(version DESC);

-- Create policy dependencies table for circular dependency detection
CREATE TABLE IF NOT EXISTS policy_dependencies (
    id BIGSERIAL PRIMARY KEY,
    policy_id BIGINT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    depends_on_policy_id BIGINT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Unique constraint on policy_id and depends_on_policy_id
    CONSTRAINT policy_dependencies_unique UNIQUE (policy_id, depends_on_policy_id),

    -- Prevent self-dependency
    CONSTRAINT policy_dependencies_no_self CHECK (policy_id != depends_on_policy_id)
);

-- Creat policy_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_policy_dependencies_policy_id ON policy_dependencies(policy_id);

-- Create index on depends_on_policy_id for reverse lookups
CREATE INDEX IF NOT EXISTS idx_policy_dependencies_depends_on ON policy_dependencies(depends_on_policy_id);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_policies_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER policies_updated_at_trigger
    BEFORE UPDATE ON policies
    FOR EACH ROW
    EXECUTE FUNCTION update_policies_updated_at();

-- Create trigger for policy_stats
CREATE TRIGGER policy_stats_updated_at_trigger
    BEFORE UPDATE ON policy_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_policies_updated_at();
