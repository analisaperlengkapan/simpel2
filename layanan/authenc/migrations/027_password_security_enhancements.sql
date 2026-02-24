-- Migration: Password Security Enhancements
-- Description: Add password history tracking and expiration policy support
-- Version: 027
-- Date: 2024-10-30

-- Create password history table for tracking previous passwords
CREATE TABLE IF NOT EXISTS password_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign key to users table
    CONSTRAINT fk_password_history_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Create index for efficient history lookups
CREATE INDEX IF NOT EXISTS idx_password_history_user_id_created
    ON password_history(user_id, created_at DESC);

-- Add password expiration columns to users table if they don't exist
DO $$
BEGIN
    -- Add password_changed_at if not exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'password_changed_at'
    ) THEN
        ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMP WITH TIME ZONE;
        -- Set initial value to created_at for existing users
        UPDATE users SET password_changed_at = created_at WHERE password_changed_at IS NULL;
    END IF;

    -- Add password_expires_at if not exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'password_expires_at'
    ) THEN
        ALTER TABLE users ADD COLUMN password_expires_at TIMESTAMP WITH TIME ZONE;
    END IF;

    -- Add require_password_change if not exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'require_password_change'
    ) THEN
        ALTER TABLE users ADD COLUMN require_password_change BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;

    -- Add password_history_count if not exists (for tracking history size)
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'password_history_count'
    ) THEN
        ALTER TABLE users ADD COLUMN password_history_count INTEGER NOT NULL DEFAULT 0;
    END IF;
END $$;

-- Create function to automatically add password to history on change
CREATE OR REPLACE FUNCTION add_password_to_history()
RETURNS TRIGGER AS $$
BEGIN
    -- Only add to history if password_hash changed and is not null
    IF NEW.password_hash IS NOT NULL AND
       (OLD.password_hash IS NULL OR NEW.password_hash != OLD.password_hash) THEN

        -- Insert old password into history (if it exists)
        IF OLD.password_hash IS NOT NULL THEN
            INSERT INTO password_history (user_id, password_hash, created_at)
            VALUES (NEW.id, OLD.password_hash, OLD.password_changed_at);
        END IF;

        -- Update password_changed_at to current time
        NEW.password_changed_at = NOW();

        -- Increment history count
        NEW.password_history_count = NEW.password_history_count + 1;

        -- Clean up old history entries (keep only last 10)
        DELETE FROM password_history
        WHERE user_id = NEW.id
        AND id NOT IN (
            SELECT id FROM password_history
            WHERE user_id = NEW.id
            ORDER BY created_at DESC
            LIMIT 10
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger forword history tracking
DROP TRIGGER IF EXISTS trigger_password_history ON users;
CREATE TRIGGER trigger_password_history
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION add_password_to_history();

-- Create function to check password expiration
CREATE OR REPLACE FUNCTION check_password_expiration(
    p_user_id UUID,
    p_expiration_days INTEGER DEFAULT 90
)
RETURNS TABLE (
    is_expired BOOLEAN,
    days_until_expiration INTEGER,
    should_warn BOOLEAN
) AS $$
DECLARE
    v_password_changed_at TIMESTAMP WITH TIME ZONE;
    v_password_expires_at TIMESTAMP WITH TIME ZONE;
    v_days_left INTEGER;
BEGIN
    -- Get password change date
    SELECT password_changed_at, password_expires_at
    INTO v_password_changed_at, v_password_expires_at
    FROM users
    WHERE id = p_user_id;

    -- If no expiration set, return not expired
    IF v_password_expires_at IS NULL THEN
        RETURN QUERY SELECT FALSE, NULL::INTEGER, FALSE;
        RETURN;
    END IF;

    -- Calculate days until expiration
    v_days_left := EXTRACT(DAY FROM (v_password_expires_at - NOW()));

    -- Return expiration status
    RETURN QUERY SELECT
        v_days_left <= 0,  -- is_expired
        v_days_left,       -- days_until_expiration
        v_days_left <= 7;  -- should_warn (7 days grace period)
END;
$$ LANGUAGE plpgsql;

-- Create function to get password history for a user
CREATE OR REPLACE FUNCTION get_password_history(
    p_user_id UUID,
    p_limit INTEGER DEFAULT 5
)
RETURNS TABLE (
    password_hash TEXT,
    created_at TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT ph.password_hash, ph.created_at
    FROM password_history ph
    WHERE ph.user_id = p_user_id
    ORDER BY ph.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON TABLE password_history IS 'Stores password history for users to prevent password reuse';
COMMENT ON COLUMN password_history.user_id IS 'Reference to the user who owns this password history entry';
COMMENT ON COLUMN password_history.password_hash IS 'Argon2 hash of the previous password';
COMMENT ON COLUMN password_history.created_at IS 'Timestamp when this password was changed';

COMMENT ON COLUMN users.password_changed_at IS 'Timestamp when the password was last changed';
COMMENT ON COLUMN users.password_expires_at IS 'Timestamp when the password will expire';
COMMENT ON COLUMN users.require_password_change IS 'Flag indicating user must change password on next login';
COMMENT ON COLUMN users.password_history_count IS 'Number of password changes for this user';

COMMENT ON FUNCTION add_password_to_history() IS 'Trigger function to automatically track password changes';
COMMENT ON FUNCTION check_password_expiration(UUID, INTEGER) IS 'Check if a user password has expired or will expire soon';
COMMENT ON FUNCTION get_password_history(UUID, INTEGER) IS 'Retrieve password history for a user';

