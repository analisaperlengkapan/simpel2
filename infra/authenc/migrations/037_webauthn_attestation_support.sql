-- Migration: WebAuthn Attestation Support
-- Description: Add comprehensive attestation fields to webauthn_credentials table
-- Author: SIMPelv2 Security Team
-- Date: 2025-11-11

-- Add attestation-related columns to webauthn_credentials table if they don't exist
DO $$
BEGIN
    -- Add attestation_certificates column for storing certificate chains
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'attestation_certificates'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN attestation_certificates JSONB;
    END IF;

    -- Add attestation_statement column for storing attestation statement
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'attestation_statement'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN attestation_statement JSONB;
    END IF;

    -- Add authenticator_metadata column for FIDO metadata
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'authenticator_metadata'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN authenticator_metadata JSONB;
    END IF;

    -- Add backup_eligible flag (for backup-capable authenticators)
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'backup_eligible'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN backup_eligible BOOLEAN DEFAULT false NOT NULL;
    END IF;

    -- Add backup_state flag (indicates if credential is backed up)
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'backup_state'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN backup_state BOOLEAN DEFAULT false NOT NULL;
    END IF;

    -- Add attestation_conveyance_preference
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'webauthn_credentials'
        AND column_name = 'attestation_conveyance'
    ) THEN
        ALTER TABLE webauthn_credentials
        ADD COLUMN attestation_conveyance VARCHAR(20) DEFAULT 'none' CHECK (
            attestation_conveyance IN ('none', 'indirect', 'direct', 'enterprise')
        );
    END IF;
END $$;

-- Create index on AAGUID for FIDO metadata lookups
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_aaguid
ON webauthn_credentials(aaguid)
WHERE aaguid IS NOT NULL;

-- Create index on attestation_format for filtering by format
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_attestation_format
ON webauthn_credentials(attestation_format)
WHERE attestation_format IS NOT NULL;

-- Create index on backup_eligible and backup_state for security queries
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_backup_flags
ON webauthn_credentials(backup_eligible, backup_state);

-- Add comment to document the attestation support
COMMENT ON COLUMN webauthn_credentials.attestation_certificates IS
'Certificate chain from direct/enterprise attestation (PEM format, JSON array)';

COMMENT ON COLUMN webauthn_credentials.attestation_statement IS
'Attestation statement from authenticator (format-specific, JSONB)';

COMMENT ON COLUMN webauthn_credentials.authenticator_metadata IS
'Metadata from FIDO Metadata Service including manufacturer, model, certification level';

COMMENT ON COLUMN webauthn_credentials.backup_eligible IS
'Indicates if the credential can be backed up (multi-device credentials)';

COMMENT ON COLUMN webauthn_credentials.backup_state IS
'Indicates if the credential is currently backed up';

COMMENT ON COLUMN webauthn_credentials.attestation_conveyance IS
'Attestation conveyance preference used during registration (none/indirect/direct/enterprise)';

-- Create webauthn_challenges table if it doesn't exist (for storing registration/auth challenges)
CREATE TABLE IF NOT EXISTS webauthn_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge TEXT NOT NULL,
    challenge_type VARCHAR(20) NOT NULL CHECK (challenge_type IN ('registration', 'authentication')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT webauthn_challenges_user_type_unique UNIQUE (user_id, challenge_type, used)
);

-- Index for challenge lookups
CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_user_id_type
ON webauthn_challenges(user_id, challenge_type, used, expires_at);

-- Index for cleanup of expired challenges
CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_expires_at
ON webauthn_challenges(expires_at)
WHERE used = false;

-- Add comment for webauthn_challenges table
COMMENT ON TABLE webauthn_challenges IS
'Stores WebAuthn registration and authentication challenges with TTL';

-- Create audit log table for WebAuthn events
CREATE TABLE IF NOT EXISTS webauthn_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id BYTEA,
    event_type VARCHAR(50) NOT NULL,
    event_details JSONB,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for audit queries
CREATE INDEX IF NOT EXISTS idx_webauthn_audit_log_user_id_created_at
ON webauthn_audit_log(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webauthn_audit_log_credential_id
ON webauthn_audit_log(credential_id)
WHERE credential_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_webauthn_audit_log_event_type
ON webauthn_audit_log(event_type, created_at DESC);

-- Add comment for webauthn_audit_log table
COMMENT ON TABLE webauthn_audit_log IS
'Audit log for all WebAuthn operations (registration, authentication, errors)';

-- Create function to clean up expired challenges
CREATE OR REPLACE FUNCTION cleanup_expired_webauthn_challenges()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM webauthn_challenges
    WHERE expires_at < CURRENT_TIMESTAMP AND used = false;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Add comment for cleanup function
COMMENT ON FUNCTION cleanup_expired_webauthn_challenges() IS
'Cleans up expired WebAuthn challenges. Should be called periodically via cron/scheduler.';

-- Grant necessary permissions (adjust based on your role setup)
GRANT SELECT, INSERT, UPDATE, DELETE ON webauthn_credentials TO authenc;
GRANT SELECT, INSERT, UPDATE, DELETE ON webauthn_challenges TO authenc;
GRANT SELECT, INSERT ON webauthn_audit_log TO authenc;
GRANT EXECUTE ON FUNCTION cleanup_expired_webauthn_challenges() TO authenc;
