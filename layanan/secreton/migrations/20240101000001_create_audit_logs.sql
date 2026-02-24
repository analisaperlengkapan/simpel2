-- Create audit_logs table (PostgreSQL)
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    action VARCHAR(100) NOT NULL,
    actor_id UUID,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    ip INET,
    user_agent TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_status ON audit_logs(status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_metadata ON audit_logs USING GIN(metadata);
