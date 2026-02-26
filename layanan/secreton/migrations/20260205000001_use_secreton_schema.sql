-- Migration: Move all Secreton tables to dedicated 'secreton' schema
-- Purpose: Comply with requirement to use dedicated schema instead of public
-- Date: 2025-02-05

-- Create the secreton schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS secreton;

-- Set search_path to use secreton schema first
ALTER DATABASE CURRENT SET search_path TO secreton, public;

-- Move existing tables to secreton schema
-- Note: This is idempotent - will skip if tables are already in secreton schema

DO $$
DECLARE
    tbl TEXT;
    -- Only tables that actually exist in other migrations
    tables_to_move TEXT[] := ARRAY[
        'audit_logs',
        'secret_manager_state',
        'namespaces',
        'dynamic_roles',
        'leases',
        'policies',
        'wrapping_tokens',
        'raft_snapshots'
    ];
BEGIN
    FOREACH tbl IN ARRAY tables_to_move
    LOOP
        -- Check if table exists in public schema
        IF EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = tbl
        ) THEN
            -- Move table to secreton schema
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA secreton', tbl);
            RAISE NOTICE 'Moved table % from public to secreton schema', tbl;
        ELSIF EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'secreton' AND table_name = tbl
        ) THEN
            RAISE NOTICE 'Table % already in secreton schema', tbl;
        ELSE
            RAISE NOTICE 'Table % does not exist, will be created in secreton schema', tbl;
        END IF;
    END LOOP;
END $$;

-- Grant usage on secreton schema to the application role
-- Adjust role name as needed for your deployment
-- GRANT USAGE ON SCHEMA secreton TO secreton_app;
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA secreton TO secreton_app;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA secreton TO secreton_app;

-- Comment for documentation
COMMENT ON SCHEMA secreton IS 'Secreton secrets management - dedicated schema for isolation from other services';
