-- ============================================================================
-- Migration 003: Rename api_tokens → monsakti_tokens, fix views
-- ============================================================================
-- Reason: api_tokens is specifically for MonSAKTI rotating bearer tokens.
--         The generic name was misleading. MySIMKARI and SIMAN use different
--         auth mechanisms (static bearer / OAuth2 respectively).
-- ============================================================================

SET search_path TO integrasi, public;

-- 1. Rename table
ALTER TABLE IF EXISTS api_tokens RENAME TO monsakti_tokens;

-- 2. Rename constraint (auto-generated from UNIQUE on module)
ALTER INDEX IF EXISTS api_tokens_module_key RENAME TO monsakti_tokens_module_key;

-- 3. Rename PK constraint
ALTER INDEX IF EXISTS api_tokens_pkey RENAME TO monsakti_tokens_pkey;

-- 4. Rename partial index
ALTER INDEX IF EXISTS idx_api_tokens_module RENAME TO idx_monsakti_tokens_active;

-- 5. Fix v_token_health view — was referencing t.last_refreshed_at (wrong column name)
CREATE OR REPLACE VIEW v_token_health AS
SELECT
    t.module,
    t.is_active,
    t.is_expired,
    t.expires_at,
    t.refreshed_at,
    t.refresh_count,
    CASE
        WHEN t.is_expired THEN 'EXPIRED'
        WHEN t.expires_at IS NOT NULL AND t.expires_at < CURRENT_TIMESTAMP THEN 'EXPIRED'
        WHEN t.expires_at IS NOT NULL AND t.expires_at < CURRENT_TIMESTAMP + INTERVAL '1 day' THEN 'EXPIRING_SOON'
        WHEN NOT t.is_active THEN 'INACTIVE'
        ELSE 'HEALTHY'
    END AS health_status
FROM monsakti_tokens t
ORDER BY health_status;

-- 6. Verify
DO $$
BEGIN
    RAISE NOTICE '✅ Migration 003 complete: api_tokens → monsakti_tokens, v_token_health fixed';
END $$;
