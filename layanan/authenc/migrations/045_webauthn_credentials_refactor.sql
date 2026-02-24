-- Migration: WebAuthn Credentials Refactor for webauthn-rs 0.5.x
-- Description: Update webauthn_credentials table to store full Passkey objects
-- Author: SIMPelv2 Security Team
-- Date: 2026-02-19

-- Rename old table to preserve data
ALTER TABLE IF EXISTS webauthn_credentials RENAME TO webauthn_credentials_old;

-- Create new webauthn_credentials table with updated schema
CREATE TABLE webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cred_id BYTEA NOT NULL UNIQUE,  -- WebAuthn credential ID (binary)
    cred JSONB NOT NULL,             -- Full Passkey object from webauthn-rs
    nickname VARCHAR(255),           -- User-assigned nickname for this credential
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMPTZ,           -- Last authentication timestamp
    CONSTRAINT webauthn_credentials_user_cred_unique UNIQUE (user_id, cred_id)
);

-- Create indexes for efficient queries
CREATE INDEX idx_webauthn_credentials_user_id ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_credentials_cred_id ON webauthn_credentials(cred_id);
CREATE INDEX idx_webauthn_credentials_last_used ON webauthn_credentials(last_used DESC NULLS LAST);

-- Add comments
COMMENT ON TABLE webauthn_credentials IS
'Stores WebAuthn/FIDO2 passkey credentials for passwordless authentication';

COMMENT ON COLUMN webauthn_credentials.id IS
'Unique credential ID (database primary key)';

COMMENT ON COLUMN webauthn_credentials.user_id IS
'User ID this credential belongs to';

COMMENT ON COLUMN webauthn_credentials.cred_id IS
'WebAuthn credential ID (binary, from authenticator)';

COMMENT ON COLUMN webauthn_credentials.cred IS
'Full Passkey object from webauthn-rs (includes public key, counter, etc.)';

COMMENT ON COLUMN webauthn_credentials.nickname IS
'Optional user-assigned nickname (e.g., "My YubiKey", "iPhone Touch ID")';

COMMENT ON COLUMN webauthn_credentials.created_at IS
'When this credential was registered';

COMMENT ON COLUMN webauthn_credentials.last_used IS
'When this credential was last used for authentication';

-- Migrate data from old table (if it exists and has data)
-- Note: This is a best-effort migration. Manual verification may be needed.
DO $
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'webauthn_credentials_old') THEN
        -- Log migration start
        RAISE NOTICE 'Migrating data from webauthn_credentials_old to webauthn_credentials';

        -- Note: Full migration requires reconstructing Passkey objects from old schema
        -- This is complex and may require application-level migration
        -- For now, we'll just preserve the table for manual migration
        RAISE NOTICE 'Old webauthn_credentials data preserved in webauthn_credentials_old';
        RAISE NOTICE 'Manual migration may be required for existing credentials';
    END IF;
END $;

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON webauthn_credentials TO authenc;

-- Add trigger to update last_used automatically (optional)
CREATE OR REPLACE FUNCTION update_webauthn_credential_last_used()
RETURNS TRIGGER AS $
BEGIN
    -- This trigger can be used by application to auto-update last_used
    -- Currently, application handles this explicitly
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

-- Note: Trigger not created by default, application handles last_used updates
-- CREATE TRIGGER trigger_update_webauthn_credential_last_used
-- BEFORE UPDATE ON webauthn_credentials
-- FOR EACH ROW
-- EXECUTE FUNCTION update_webauthn_credential_last_used();
