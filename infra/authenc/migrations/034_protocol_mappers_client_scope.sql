-- Add client_scope_id column to protocol_mappers table
-- This allows protocol mappers to be associated with client scopes for better scope management

ALTER TABLE protocol_mappers ADD COLUMN IF NOT EXISTS client_scope_id UUID REFERENCES client_scopes(id) ON DELETE CASCADE;

-- Add index for client_scope_id
CREATE INDEX IF NOT EXISTS idx_protocol_mappers_scope ON protocol_mappers(client_scope_id);

-- Update unique constraint to support realm-level mappers (where both client_id and client_scope_id are NULL)
-- Drop old constraint first
ALTER TABLE protocol_mappers DROP CONSTRAINT IF EXISTS protocol_mappers_client_id_name_key;

-- Add new constraint that allows unique names within realm for realm-level mappers
ALTER TABLE protocol_mappers ADD CONSTRAINT protocol_mappers_unique_name
    UNIQUE NULLS NOT DISTINCT (client_id, client_scope_id, realm_id, name);
