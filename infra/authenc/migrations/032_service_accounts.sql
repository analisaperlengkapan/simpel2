SET search_path = authenc, public;
-- Migration: Service Accounts for Machine-to-Machine Authentication
-- Description: Create service_accounts table and related structures for non-human entity authentication

-- Create service_accounts table
CREATE TABLE IF NOT EXISTS service_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    client_id VARCHAR(255) NOT NULL UNIQUE,
    client_secret_hash TEXT NOT NULL,
    realm_id UUID NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,
    attributes JSONB DEFAULT '{}'::jsonb,

    -- Foreign key to realms table
    CONSTRAINT fk_service_account_realm FOREIGN KEY (realm_id)
        REFERENCES realms(id) ON DELETE CASCADE,

    -- Unique constraint on name within realm
    CONSTRAINT uq_service_account_name_realm UNIQUE (name, realm_id),

    -- Check constraints
    CONSTRAINT chk_service_account_name_not_empty CHECK (name <> ''),
    CONSTRAINT chk_service_account_client_id_not_empty CHECK (client_id <> ''),
    CONSTRAINT chk_service_account_secret_not_empty CHECK (client_secret_hash <> '')
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_service_accounts_realm_id ON service_accounts(realm_id) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_service_accounts_client_id ON service_accounts(client_id) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_service_accounts_enabled ON service_accounts(enabled);
CREATE INDEX IF NOT EXISTS idx_service_accounts_last_used ON service_accounts(last_used_at DESC NULLS LAST) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_service_accounts_name ON service_accounts USING gin(to_tsvector('english', name));

-- Create service_account_roles junction table for role assignments
CREATE TABLE IF NOT EXISTS service_account_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_account_id UUID NOT NULL,
    role_id UUID NOT NULL,
    granted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    granted_by UUID, -- User ID who granted the role
    attributes JSONB DEFAULT '{}'::jsonb,

    -- Foreign keys
    CONSTRAINT fk_service_account_role_sa FOREIGN KEY (service_account_id)
        REFERENCES service_accounts(id) ON DELETE CASCADE,
    CONSTRAINT fk_service_account_role_role FOREIGN KEY (role_id)
        REFERENCES roles(id) ON DELETE CASCADE,

    -- Unique constraint to prevent duplicate role assignments
    CONSTRAINT uq_service_account_role UNIQUE (service_account_id, role_id)
);

-- Create indexes for role lookups
CREATE INDEX IF NOT EXISTS idx_service_account_roles_sa_id ON service_account_roles(service_account_id);
CREATE INDEX IF NOT EXISTS idx_service_account_roles_role_id ON service_account_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_service_account_roles_granted_at ON service_account_roles(granted_at DESC);

-- Create service_account_audit_log table for comprehensive audit trail
CREATE TABLE IF NOT EXISTS service_account_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_account_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL, -- 'created', 'updated', 'deleted', 'authenticated', 'secret_regenerated', 'role_granted', 'role_revoked'
    event_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    performed_by UUID, -- User ID who performed the action (NULL for system actions)
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign key
    CONSTRAINT fk_service_account_audit_sa FOREIGN KEY (service_account_id)
        REFERENCES service_accounts(id) ON DELETE CASCADE,

    -- Check constraint
    CONSTRAINT chk_service_account_audit_event_type CHECK (
        event_type IN ('created', 'updated', 'deleted', 'enabled', 'disabled',
                      'authenticated', 'auth_failed', 'secret_regenerated',
                      'role_granted', 'role_revoked')
    )
);

-- Create indexes for audit log queries
CREATE INDEX IF NOT EXISTS idx_service_account_audit_sa_id ON service_account_audit_log(service_account_id);
CREATE INDEX IF NOT EXISTS idx_service_account_audit_event_type ON service_account_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_service_account_audit_timestamp ON service_account_audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_service_account_audit_performed_by ON service_account_audit_log(performed_by) WHERE performed_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_service_account_audit_success ON service_account_audit_log(success) WHERE success = false;

-- Create function to update service_accounts.updated_at automatically
CREATE OR REPLACE FUNCTION update_service_account_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic updated_at update
CREATE TRIGGER trigger_update_service_account_updated_at
    BEFORE UPDATE ON service_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_service_account_updated_at();

-- Create function to audit service account changes
CREATE OR REPLACE FUNCTION audit_service_account_changes()
RETURNS TRIGGER AS $$
DECLARE
    event_type_var VARCHAR(50);
    event_details_var JSONB;
BEGIN
    -- Determine event type based on operation
    IF TG_OP = 'INSERT' THEN
        event_type_var := 'created';
        event_details_var := jsonb_build_object(
            'name', NEW.name,
            'client_id', NEW.client_id,
            'realm_id', NEW.realm_id,
            'enabled', NEW.enabled
        );

        -- Insert audit log entry
        INSERT INTO service_account_audit_log (
            service_account_id, event_type, event_details, success, timestamp
        ) VALUES (
            NEW.id, event_type_var, event_details_var, true, NOW()
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Determine specific update type
        IF OLD.enabled = true AND NEW.enabled = false THEN
            event_type_var := 'disabled';
        ELSIF OLD.enabled = false AND NEW.enabled = true THEN
            event_type_var := 'enabled';
        ELSIF OLD.client_secret_hash <> NEW.client_secret_hash THEN
            event_type_var := 'secret_regenerated';
        ELSE
            event_type_var := 'updated';
        END IF;

        -- Build event details with changed fields
        event_details_var := jsonb_build_object(
            'changed_fields', jsonb_build_object(
                'name', CASE WHEN OLD.name <> NEW.name THEN jsonb_build_object('old', OLD.name, 'new', NEW.name) ELSE NULL END,
                'description', CASE WHEN OLD.description IS DISTINCT FROM NEW.description THEN jsonb_build_object('old', OLD.description, 'new', NEW.description) ELSE NULL END,
                'enabled', CASE WHEN OLD.enabled <> NEW.enabled THEN jsonb_build_object('old', OLD.enabled, 'new', NEW.enabled) ELSE NULL END
            )
        );

        -- Insert audit log entry
        INSERT INTO service_account_audit_log (
            service_account_id, event_type, event_details, success, timestamp
        ) VALUES (
            NEW.id, event_type_var, event_details_var, true, NOW()
        );

    ELSIF TG_OP = 'DELETE' THEN
        event_type_var := 'deleted';
        event_details_var := jsonb_build_object(
            'name', OLD.name,
            'client_id', OLD.client_id,
            'realm_id', OLD.realm_id
        );

        -- Insert audit log entry (service_account_id still valid due to ON DELETE CASCADE)
        INSERT INTO service_account_audit_log (
            service_account_id, event_type, event_details, success, timestamp
        ) VALUES (
            OLD.id, event_type_var, event_details_var, true, NOW()
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic audit logging
CREATE TRIGGER trigger_audit_service_account_changes
    AFTER INSERT OR UPDATE OR DELETE ON service_accounts
    FOR EACH ROW
    EXECUTE FUNCTION audit_service_account_changes();

-- Create function to audit role assignments
CREATE OR REPLACE FUNCTION audit_service_account_role_changes()
RETURNS TRIGGER AS $$
DECLARE
    event_type_var VARCHAR(50);
    event_details_var JSONB;
BEGIN
    IF TG_OP = 'INSERT' THEN
        event_type_var := 'role_granted';
        event_details_var := jsonb_build_object(
            'role_id', NEW.role_id,
            'granted_by', NEW.granted_by
        );

        INSERT INTO service_account_audit_log (
            service_account_id, event_type, event_details, success, timestamp
        ) VALUES (
            NEW.service_account_id, event_type_var, event_details_var, true, NOW()
        );

    ELSIF TG_OP = 'DELETE' THEN
        event_type_var := 'role_revoked';
        event_details_var := jsonb_build_object(
            'role_id', OLD.role_id
        );

        INSERT INTO service_account_audit_log (
            service_account_id, event_type, event_details, success, timestamp
        ) VALUES (
            OLD.service_account_id, event_type_var, event_details_var, true, NOW()
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for role assignment audit logging
CREATE TRIGGER trigger_audit_service_account_role_changes
    AFTER INSERT OR DELETE ON service_account_roles
    FOR EACH ROW
    EXECUTE FUNCTION audit_service_account_role_changes();

-- Add comments for documentation
COMMENT ON TABLE service_accounts IS 'Service accounts for machine-to-machine authentication using OAuth2 client credentials grant';
COMMENT ON COLUMN service_accounts.client_id IS 'OAuth2 client identifier, must be unique across all realms';
COMMENT ON COLUMN service_accounts.client_secret_hash IS 'Bcrypt-hashed client secret (cost factor 12)';
COMMENT ON COLUMN service_accounts.last_used_at IS 'Timestamp of last successful authentication, used for monitoring and security audits';
COMMENT ON TABLE service_account_roles IS 'Role assignments for service accounts, determines API access permissions';
COMMENT ON TABLE service_account_audit_log IS 'Comprehensive audit trail for all service account operations';

-- Insert initial data (optional - commented out by default)
-- Example: Create a service account for the dashboard microservice in the master realm
/*
DO $$
DECLARE
    master_realm_id UUID;
    dashboard_sa_id UUID;
BEGIN
    -- Get master realm ID
    SELECT id INTO master_realm_id FROM realms WHERE name = 'master' LIMIT 1;

    IF master_realm_id IS NOT NULL THEN
        -- Create dashboard service account
        INSERT INTO service_accounts (
            id, name, description, client_id, client_secret_hash, realm_id, enabled
        ) VALUES (
            gen_random_uuid(),
            'layanan-dasbor',
            'Dashboard microservice authentication',
            'sa-layanan-dasbor',
            '$2b$12$placeholder_hash_here', -- Replace with actual bcrypt hash
            master_realm_id,
            true
        ) RETURNING id INTO dashboard_sa_id;

        -- Grant roles (example)
        -- INSERT INTO service_account_roles (service_account_id, role_id)
        -- SELECT dashboard_sa_id, id FROM roles WHERE name = 'service-role';
    END IF;
END $$;
*/
