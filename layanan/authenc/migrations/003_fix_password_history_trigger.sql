-- Fix add_password_to_history(): a NULL password_changed_at aborted the UPDATE.
--
-- password_history.created_at is NOT NULL, but users.password_changed_at is
-- NULL for every account that has never changed its password — i.e. every
-- freshly created user. The trigger copied that NULL straight into the history
-- row, so the whole UPDATE was rejected and POST /api/v1/auth/me/password
-- answered 500: a new user could never set their own password. Surfaced by the
-- portal e2e "admin creates a user; that user changes their password" flow.
--
-- Body is otherwise identical to 001_baseline.sql (history count + retention);
-- only the created_at value changed. Expand-only: CREATE OR REPLACE on an
-- existing function, no schema change, safe to re-run and safe to roll back
-- the app over.

CREATE OR REPLACE FUNCTION authenc.add_password_to_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Only add to history if password_hash changed and is not null
    IF NEW.password_hash IS NOT NULL AND
       (OLD.password_hash IS NULL OR NEW.password_hash != OLD.password_hash) THEN

        -- Insert old password into history (if it exists)
        IF OLD.password_hash IS NOT NULL THEN
            INSERT INTO password_history (user_id, password_hash, created_at)
            VALUES (NEW.id, OLD.password_hash, COALESCE(OLD.password_changed_at, NOW()));
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
$$;
