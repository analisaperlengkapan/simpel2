-- Migration: Move all Authenc tables to dedicated 'authenc' schema
-- Purpose: Comply with requirement to use dedicated schema instead of public
-- Date: 2026-02-05

-- Create the authenc schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS authenc;

-- Set search_path to use authenc schema first
ALTER DATABASE CURRENT SET search_path TO authenc, public;

-- Move existing tables to authenc schema
-- Note: This is idempotent - will skip if tables are already in authenc schema

DO $$
DECLARE
    tbl TEXT;
    tables_to_move TEXT[] := ARRAY[
        -- Core tables
        'realms',
        'users',
        'roles',
        'user_roles',
        'clients',
        'client_secrets',
        'sessions',
        'refresh_tokens',
        'authorization_codes',
        'scopes',
        'client_scopes',
        'client_default_scopes',
        'client_optional_scopes',
        'client_scope_mappings',
        
        -- SAML tables
        'saml_assertion_cache',
        'saml_messages',
        
        -- Organization tables
        'organization_domains',
        'organization_identity_providers',
        
        -- Session tables
        'user_sessions',
        
        -- Federated identity
        'federated_identity_links',
        'identity_provider_mappers',
        'identity_broker_configs',
        'federated_auth_log',
        'account_linking_requests',
        
        -- OAuth2 providers
        'oauth2_provider_configs',
        'oauth2_states',
        
        -- Admin tables
        'admin_audit_log',
        'admin_dashboard_metrics',
        'admin_console_sessions',
        'admin_notifications',
        'admin_console_preferences',
        
        -- Event system
        'event_listeners',
        'event_log',
        'event_listener_executions',
        'event_webhooks',
        'events',
        'admin_events',
        
        -- Protocol mappers
        'protocol_mappers',
        
        -- Authenticators
        'authenticator_configs',
        'authenticator_executions',
        'authenticator_execution_results',
        
        -- MFA tables
        'mfa_admin_actions',
        'mfa_policies',
        
        -- Key management
        'key_rotation_audit',
        
        -- Audit
        'audit_integrity_checks',
        'audit_integrity_failures',
        
        -- Satker hierarchy
        'satkers',
        'satker_permissions',
        'satker_admin_roles',
        'satker_audit_logs',
        
        -- Service accounts
        'service_accounts',
        'service_account_roles',
        'service_account_audit_log',
        
        -- User consent
        'user_consent_scopes',
        
        -- Token exchange
        'token_exchange_audit',
        
        -- Client registration
        'client_registration_tokens',
        'initial_access_tokens',
        'software_statement_issuers',
        'client_registration_policies',
        'client_registration_audit_log',
        
        -- WebAuthn
        'webauthn_challenges',
        'webauthn_audit_log'
    ];
BEGIN
    FOREACH tbl IN ARRAY tables_to_move
    LOOP
        -- Check if table exists in public schema
        IF EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = tbl
        ) THEN
            -- Move table to authenc schema
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA authenc', tbl);
            RAISE NOTICE 'Moved table % from public to authenc schema', tbl;
        ELSIF EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_schema = 'authenc' AND table_name = tbl
        ) THEN
            RAISE NOTICE 'Table % already in authenc schema', tbl;
        ELSE
            RAISE NOTICE 'Table % does not exist, will be created in authenc schema', tbl;
        END IF;
    END LOOP;
END $$;

-- Grant usage on authenc schema to the application role
-- (Uncomment and adjust role name as needed for deployment)
-- GRANT USAGE ON SCHEMA authenc TO application_role;
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA authenc TO application_role;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA authenc TO application_role;

COMMENT ON SCHEMA authenc IS 'Authenc identity provider - OAuth2/OIDC service tables';
