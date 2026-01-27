-- Migration: Satker Hierarchy for Organization-Aware Authorization
-- Description: Create satkers table and related structures for hierarchical organization management

DROP TABLE IF EXISTS satkers CASCADE;
CREATE TABLE IF NOT EXISTS satkers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    parent_code VARCHAR(50),
    level INTEGER NOT NULL DEFAULT 0,
    satker_type JSONB NOT NULL DEFAULT '"KejaksaanNegeri"'::jsonb,
    active BOOLEAN NOT NULL DEFAULT true,
    attributes JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign key to parent satker
    CONSTRAINT fk_parent_satker FOREIGN KEY (parent_code)
        REFERENCES satkers(code) ON DELETE SET NULL,

    -- Check constraints
    CONSTRAINT chk_level_non_negative CHECK (level >= 0),
    CONSTRAINT chk_code_not_empty CHECK (code <> ''),
    CONSTRAINT chk_name_not_empty CHECK (name <> '')
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_satkers_code ON satkers(code) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satkers_parent_code ON satkers(parent_code) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satkers_level ON satkers(level) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satkers_active ON satkers(active);
CREATE INDEX IF NOT EXISTS idx_satkers_name ON satkers USING gin(to_tsvector('indonesian', name));

-- Create index for hierarchy traversal
CREATE INDEX IF NOT EXISTS idx_satkers_hierarchy ON satkers(parent_code, level, code) WHERE active = true;

-- Add satker_code to users table if it doesn't exist (it should already exist)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'satker_code'
    ) THEN
        ALTER TABLE users ADD COLUMN satker_code VARCHAR(50) NOT NULL DEFAULT 'UNKNOWN';
    END IF;
END $$;

-- Create index on users.satker_code for efficient lookups
CREATE INDEX IF NOT EXISTS idx_users_satker_code ON users(satker_code);

DROP TABLE IF EXISTS satker_permissions CASCADE;
CREATE TABLE IF NOT EXISTS satker_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    satker_code VARCHAR(50) NOT NULL,
    permission_type VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    action VARCHAR(50),
    include_children BOOLEAN NOT NULL DEFAULT false,
    include_parents BOOLEAN NOT NULL DEFAULT false,
    granted_by UUID,
    granted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    attributes JSONB,

    -- Foreign keys
    CONSTRAINT fk_satker_perm_user FOREIGN KEY (user_id)
        REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_satker_perm_satker FOREIGN KEY (satker_code)
        REFERENCES satkers(code) ON DELETE CASCADE,
    CONSTRAINT fk_satker_perm_granted_by FOREIGN KEY (granted_by)
        REFERENCES users(id) ON DELETE SET NULL,

    -- Unique constraint to prevent duplicate permissions
    CONSTRAINT uq_satker_permission UNIQUE (user_id, satker_code, permission_type, resource_type, action)
);

-- Create indexes for satker_permissions
CREATE INDEX IF NOT EXISTS idx_satker_perm_user ON satker_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_satker_perm_satker ON satker_permissions(satker_code);
CREATE INDEX IF NOT EXISTS idx_satker_perm_type ON satker_permissions(permission_type);
CREATE INDEX IF NOT EXISTS idx_satker_perm_expires ON satker_permissions(expires_at) WHERE expires_at IS NOT NULL;

-- Create satker_admin_roles table for satker-scoped admin assignments
CREATE TABLE IF NOT EXISTS satker_admin_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    satker_code VARCHAR(50) NOT NULL,
    admin_level VARCHAR(50) NOT NULL, -- 'AdminSatker', 'AdminWilayah', 'AdminEselonI', 'AdminPusat'
    scope_data JSONB, -- Additional scope information
    assigned_by UUID,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    active BOOLEAN NOT NULL DEFAULT true,

    -- Foreign keys
    CONSTRAINT fk_satker_admin_user FOREIGN KEY (user_id)
        REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_satker_admin_satker FOREIGN KEY (satker_code)
        REFERENCES satkers(code) ON DELETE CASCADE,
    CONSTRAINT fk_satker_admin_assigned_by FOREIGN KEY (assigned_by)
        REFERENCES users(id) ON DELETE SET NULL,

    -- Check constraints
    CONSTRAINT chk_admin_level CHECK (admin_level IN ('AdminSatker', 'AdminWilayah', 'AdminEselonI', 'AdminPusat')),

    -- Unique constraint
    CONSTRAINT uq_satker_admin_role UNIQUE (user_id, satker_code, admin_level)
);

-- Create indexes for satker_admin_roles
CREATE INDEX IF NOT EXISTS idx_satker_admin_user ON satker_admin_roles(user_id) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satker_admin_satker ON satker_admin_roles(satker_code) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satker_admin_level ON satker_admin_roles(admin_level) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_satker_admin_expires ON satker_admin_roles(expires_at) WHERE expires_at IS NOT NULL;

-- Create audit log for satker operations
CREATE TABLE IF NOT EXISTS satker_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    satker_code VARCHAR(50) NOT NULL,
    user_id UUID,
    operation VARCHAR(50) NOT NULL, -- 'create', 'update', 'delete', 'access_grant', 'access_revoke'
    operation_details JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_satker_audit_satker FOREIGN KEY (satker_code)
        REFERENCES satkers(code) ON DELETE CASCADE,
    CONSTRAINT fk_satker_audit_user FOREIGN KEY (user_id)
        REFERENCES users(id) ON DELETE SET NULL
);

-- Create indexes for satker_audit_logs
CREATE INDEX IF NOT EXISTS idx_satker_audit_satker ON satker_audit_logs(satker_code);
CREATE INDEX IF NOT EXISTS idx_satker_audit_user ON satker_audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_satker_audit_timestamp ON satker_audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_satker_audit_operation ON satker_audit_logs(operation);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_satker_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for satkers table
DROP TRIGGER IF EXISTS trigger_satkers_updated_at ON satkers;
CREATE TRIGGER trigger_satkers_updated_at
    BEFORE UPDATE ON satkers
    FOR EACH ROW
    EXECUTE FUNCTION update_satker_updated_at();

-- Insert default root satker (Kejaksaan Agung Pusat)
INSERT INTO satkers (code, name, description, parent_code, level, satker_type, active)
VALUES (
    'PUSAT',
    'Kejaksaan Agung Republik Indonesia',
    'Kantor pusat Kejaksaan Agung RI',
    NULL,
    0,
    '"Pusat"'::jsonb,
    true
) ON CONFLICT (code) DO NOTHING;

-- Insert sample regional satkers (Kejaksaan Tinggi)
INSERT INTO satkers (code, name, description, parent_code, level, satker_type, active)
VALUES
    ('KT-DKI', 'Kejaksaan Tinggi DKI Jakarta', 'Kejaksaan Tinggi wilayah DKI Jakarta', 'PUSAT', 1, '"KejaksaanTinggi"'::jsonb, true),
    ('KT-JABAR', 'Kejaksaan Tinggi Jawa Barat', 'Kejaksaan Tinggi wilayah Jawa Barat', 'PUSAT', 1, '"KejaksaanTinggi"'::jsonb, true),
    ('KT-JATENG', 'Kejaksaan Tinggi Jawa Tengah', 'Kejaksaan Tinggi wilayah Jawa Tengah', 'PUSAT', 1, '"KejaksaanTinggi"'::jsonb, true),
    ('KT-JATIM', 'Kejaksaan Tinggi Jawa Timur', 'Kejaksaan Tinggi wilayah Jawa Timur', 'PUSAT', 1, '"KejaksaanTinggi"'::jsonb, true)
ON CONFLICT (code) DO NOTHING;

-- Insert sample district satkers (Kejaksaan Negeri)
INSERT INTO satkers (code, name, description, parent_code, level, satker_type, active)
VALUES
    ('KN-JAKPUS', 'Kejaksaan Negeri Jakarta Pusat', 'Kejaksaan Negeri Jakarta Pusat', 'KT-DKI', 2, '"KejaksaanNegeri"'::jsonb, true),
    ('KN-JAKSEL', 'Kejaksaan Negeri Jakarta Selatan', 'Kejaksaan Negeri Jakarta Selatan', 'KT-DKI', 2, '"KejaksaanNegeri"'::jsonb, true),
    ('KN-BANDUNG', 'Kejaksaan Negeri Bandung', 'Kejaksaan Negeri Bandung', 'KT-JABAR', 2, '"KejaksaanNegeri"'::jsonb, true),
    ('KN-SEMARANG', 'Kejaksaan Negeri Semarang', 'Kejaksaan Negeri Semarang', 'KT-JATENG', 2, '"KejaksaanNegeri"'::jsonb, true),
    ('KN-SURABAYA', 'Kejaksaan Negeri Surabaya', 'Kejaksaan Negeri Surabaya', 'KT-JATIM', 2, '"KejaksaanNegeri"'::jsonb, true)
ON CONFLICT (code) DO NOTHING;

-- Create view for satker hierarchy with full path
CREATE OR REPLACE VIEW satker_hierarchy_view AS
WITH RECURSIVE satker_tree AS (
    -- Base case: root satkers
    SELECT
        id,
        code,
        name,
        parent_code,
        level,
        satker_type,
        active,
        code::TEXT AS path,
        name::TEXT AS full_path
    FROM satkers
    WHERE parent_code IS NULL AND active = true

    UNION ALL

    -- Recursive case: child satkers
    SELECT
        s.id,
        s.code,
        s.name,
        s.parent_code,
        s.level,
        s.satker_type,
        s.active,
        st.path || ' > ' || s.code AS path,
        st.full_path || ' > ' || s.name AS full_path
    FROM satkers s
    INNER JOIN satker_tree st ON s.parent_code = st.code
    WHERE s.active = true
)
SELECT * FROM satker_tree
ORDER BY level, code;

-- Grant permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON satkers TO authenc;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON satker_permissions TO authenc;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON satker_admin_roles TO authenc;
-- GRANT SELECT, INSERT ON satker_audit_logs TO authenc;
-- GRANT SELECT ON satker_hierarchy_view TO authenc;

-- Add comments for documentation
COMMENT ON TABLE satkers IS 'Organizational units (Satuan Kerja) in the Attorney General''s Office hierarchy';
COMMENT ON TABLE satker_permissions IS 'Explicit permissions granted to users for specific satkers';
COMMENT ON TABLE satker_admin_roles IS 'Administrative roles scoped to specific satkers';
COMMENT ON TABLE satker_audit_logs IS 'Audit trail for satker-related operations';
COMMENT ON VIEW satker_hierarchy_view IS 'Hierarchical view of satkers with full path information';

COMMENT ON COLUMN satkers.code IS 'Unique identifier code for the satker';
COMMENT ON COLUMN satkers.parent_code IS 'Code of the parent satker in the hierarchy';
COMMENT ON COLUMN satkers.level IS 'Depth level in the hierarchy (0 = root)';
COMMENT ON COLUMN satkers.satker_type IS 'Type of satker (Pusat, KejaksaanTinggi, KejaksaanNegeri, etc.)';

COMMENT ON COLUMN satker_permissions.include_children IS 'Whether permission extends to child satkers';
COMMENT ON COLUMN satker_permissions.include_parents IS 'Whether permission extends to parent satkers';

COMMENT ON COLUMN satker_admin_roles.admin_level IS 'Level of administrative authority (AdminSatker, AdminWilayah, AdminEselonI, AdminPusat)';
