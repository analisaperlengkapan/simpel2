-- F5-B squashed baseline (pg_dump --schema-only of the repaired migration set,
-- 2026-06-09). Replaces the 44 accreted NNN_*.sql. ALL authenc tables live in the
-- dedicated `authenc` schema (clear ownership + per-schema least-privilege in the
-- shared dbsimpelv2). Only uuid_generate_v4 (extension) and update_updated_at_column
-- (shared trigger fn) stay in public. App connects with search_path=authenc,public.

--
-- PostgreSQL database dump
--


-- Dumped from database version 15.18
-- Dumped by pg_dump version 15.18

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', 'authenc, public', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: authenc; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA IF NOT EXISTS authenc;


--
-- Name: SCHEMA authenc; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA authenc IS 'Authenc-owned schema: ALL authenc identity/IAM tables live here (shared dbsimpelv2). Consumers (perlengkapan) read authenc.* cross-schema. App search_path=authenc,public.';


--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: add_password_to_history(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.add_password_to_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION add_password_to_history(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.add_password_to_history() IS 'Trigger function to automatically track password changes';


--
-- Name: audit_service_account_changes(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.audit_service_account_changes() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: audit_service_account_role_changes(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.audit_service_account_role_changes() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: check_consent_required(uuid, uuid, text[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.check_consent_required(p_user_id uuid, p_client_id uuid, p_requested_scopes text[]) RETURNS TABLE(consent_needed boolean, missing_scopes text[])
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_consented_scopes TEXT[];
    v_missing_scopes TEXT[];
    v_consent_required_scopes TEXT[];
BEGIN
    -- Get scopes that require consent
    SELECT ARRAY_AGG(cs.name) INTO v_consent_required_scopes
    FROM client_scopes cs
    INNER JOIN client_all_scopes cas ON cs.id = cas.scope_id
    WHERE cas.client_id = p_client_id
      AND cs.consent_required = TRUE
      AND cs.name = ANY(p_requested_scopes);

    -- Get already consented scopes
    SELECT ARRAY_AGG(cs.name) INTO v_consented_scopes
    FROM user_consent_scopes ucs
    INNER JOIN client_scopes cs ON ucs.scope_id = cs.id
    WHERE ucs.user_id = p_user_id
      AND ucs.client_id = p_client_id
      AND (ucs.expires_at IS NULL OR ucs.expires_at > NOW())
      AND cs.name = ANY(v_consent_required_scopes);

    -- Find missing consents
    SELECT ARRAY_AGG(scope) INTO v_missing_scopes
    FROM UNNEST(v_consent_required_scopes) AS scope
    WHERE scope != ALL(COALESCE(v_consented_scopes, ARRAY[]::TEXT[]));

    -- Return result
    IF v_missing_scopes IS NULL OR ARRAY_LENGTH(v_missing_scopes, 1) = 0 THEN
        RETURN QUERY SELECT FALSE, NULL::TEXT[];
    ELSE
        RETURN QUERY SELECT TRUE, v_missing_scopes;
    END IF;
END;
$$;


--
-- Name: check_password_expiration(uuid, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.check_password_expiration(p_user_id uuid, p_expiration_days integer DEFAULT 90) RETURNS TABLE(is_expired boolean, days_until_expiration integer, should_warn boolean)
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: FUNCTION check_password_expiration(p_user_id uuid, p_expiration_days integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.check_password_expiration(p_user_id uuid, p_expiration_days integer) IS 'Check if a user password has expired or will expire soon';


--
-- Name: cleanup_expired_captcha_challenges(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.cleanup_expired_captcha_challenges() RETURNS bigint
    LANGUAGE plpgsql
    AS $$
DECLARE
    deleted_count BIGINT;
BEGIN
    -- Delete challenges that expired more than 1 hour ago
    DELETE FROM captcha_challenges
    WHERE expires_at < NOW() - INTERVAL '1 hour'
      AND solved = false;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    -- Also clean up orphaned behavioral metrics (shouldn't happen due to CASCADE, but safety check)
    DELETE FROM captcha_behavioral_metrics
    WHERE challenge_id NOT IN (SELECT id FROM captcha_challenges);

    -- Clean up old validation attempts (keep for 30 days for analytics)
    DELETE FROM captcha_validation_attempts
    WHERE created_at < NOW() - INTERVAL '30 days';

    RETURN deleted_count;
END;
$$;


--
-- Name: FUNCTION cleanup_expired_captcha_challenges(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.cleanup_expired_captcha_challenges() IS 'Cleans up expired CAPTCHA challenges and related data';


--
-- Name: cleanup_expired_data(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.cleanup_expired_data() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM oauth2_authorization_codes WHERE expires_at < NOW();
    UPDATE oauth2_access_tokens SET revoked = true, revoked_at = NOW()
    WHERE expires_at < NOW() AND revoked = false;
    UPDATE user_sessions SET terminated = true, terminated_at = NOW(), terminated_reason = 'expired'
    WHERE expires_at < NOW() AND terminated = false;
    DELETE FROM saml_sessions WHERE expires_at < NOW();
    DELETE FROM audit_logs WHERE timestamp < NOW() - INTERVAL '90 days';
    DELETE FROM uma_permission_requests WHERE expires_at < NOW() AND status = 'pending';
END;
$$;


--
-- Name: cleanup_expired_webauthn_challenges(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.cleanup_expired_webauthn_challenges() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM webauthn_challenges
    WHERE expires_at < CURRENT_TIMESTAMP AND used = false;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$;


--
-- Name: FUNCTION cleanup_expired_webauthn_challenges(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.cleanup_expired_webauthn_challenges() IS 'Cleans up expired WebAuthn challenges. Should be called periodically via cron/scheduler.';


--
-- Name: get_batch_mfa_status(uuid[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_batch_mfa_status(user_ids uuid[]) RETURNS TABLE(user_id uuid, mfa_enabled boolean, mfa_setup_at timestamp with time zone, mfa_last_used timestamp with time zone)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used
    FROM users u
    WHERE u.id = ANY(user_ids)
      AND u.enabled = true
      AND u.deleted_at IS NULL;
END;
$$;


--
-- Name: FUNCTION get_batch_mfa_status(user_ids uuid[]); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_batch_mfa_status(user_ids uuid[]) IS 'Optimized function to get MFA status for multiple users';


--
-- Name: get_captcha_analytics_summary(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_captcha_analytics_summary(p_days integer DEFAULT 7) RETURNS TABLE(total_challenges bigint, solved_challenges bigint, success_rate numeric, avg_difficulty numeric, bot_detection_rate numeric, unique_ips bigint, high_risk_attempts bigint)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(c.id)::BIGINT as total_challenges,
        COUNT(*) FILTER (WHERE c.solved = true)::BIGINT as solved_challenges,
        CASE
            WHEN COUNT(c.id) > 0 THEN
                ROUND((COUNT(*) FILTER (WHERE c.solved = true)::NUMERIC / COUNT(c.id)::NUMERIC) * 100, 2)
            ELSE 0
        END as success_rate,
        ROUND(AVG(c.difficulty_level), 1) as avg_difficulty,
        CASE
            WHEN COUNT(bm.id) > 0 THEN
                ROUND((COUNT(*) FILTER (WHERE bm.classification = 'Bot')::NUMERIC / COUNT(bm.id)::NUMERIC) * 100, 2)
            ELSE 0
        END as bot_detection_rate,
        COUNT(DISTINCT c.ip_address)::BIGINT as unique_ips,
        COUNT(*) FILTER (WHERE va.risk_assessment IN ('High', 'Critical'))::BIGINT as high_risk_attempts
    FROM captcha_challenges c
    LEFT JOIN captcha_behavioral_metrics bm ON c.id = bm.challenge_id
    LEFT JOIN captcha_validation_attempts va ON c.id = va.challenge_id
    WHERE c.created_at >= NOW() - (p_days || ' days')::INTERVAL;
END;
$$;


--
-- Name: FUNCTION get_captcha_analytics_summary(p_days integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_captcha_analytics_summary(p_days integer) IS 'Returns summary analytics for CAPTCHA system performance';


--
-- Name: get_captcha_difficulty(inet, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_captcha_difficulty(p_ip_address inet, p_session_id character varying DEFAULT NULL::character varying) RETURNS smallint
    LANGUAGE plpgsql
    AS $$
DECLARE
    difficulty SMALLINT := 3; -- Default difficulty
    adjustment_record RECORD;
BEGIN
    -- Check for IP-based difficulty adjustments
    SELECT difficulty_level INTO difficulty
    FROM captcha_difficulty_adjustments
    WHERE ip_pattern >>= p_ip_address
      AND active = true
      AND (expires_at IS NULL OR expires_at > NOW())
    ORDER BY masklen(ip_pattern) DESC, created_at DESC
    LIMIT 1;

    -- Check for session-based difficulty adjustments (higher priority)
    IF p_session_id IS NOT NULL THEN
        SELECT difficulty_level INTO difficulty
        FROM captcha_difficulty_adjustments
        WHERE session_pattern = p_session_id
          AND active = true
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY created_at DESC
        LIMIT 1;
    END IF;

    -- Adaptive difficulty based on recent failures
    SELECT
        CASE
            WHEN failure_rate > 0.8 THEN LEAST(difficulty + 3, 10)
            WHEN failure_rate > 0.6 THEN LEAST(difficulty + 2, 10)
            WHEN failure_rate > 0.4 THEN LEAST(difficulty + 1, 10)
            ELSE difficulty
        END INTO difficulty
    FROM (
        SELECT
            COALESCE(
                COUNT(*) FILTER (WHERE va.success = false)::NUMERIC /
                NULLIF(COUNT(*)::NUMERIC, 0),
                0
            ) as failure_rate
        FROM captcha_challenges c
        LEFT JOIN captcha_validation_attempts va ON c.id = va.challenge_id
        WHERE c.ip_address = p_ip_address
          AND c.created_at > NOW() - INTERVAL '1 hour'
    ) recent_activity;

    RETURN difficulty;
END;
$$;


--
-- Name: FUNCTION get_captcha_difficulty(p_ip_address inet, p_session_id character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_captcha_difficulty(p_ip_address inet, p_session_id character varying) IS 'Determines appropriate CAPTCHA difficulty for IP/session';


--
-- Name: get_mfa_compliance_summary(text[], text[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_mfa_compliance_summary(p_satker_codes text[] DEFAULT NULL::text[], p_role_names text[] DEFAULT NULL::text[]) RETURNS TABLE(total_users bigint, mfa_enabled_users bigint, compliance_percentage numeric, non_compliant_users bigint)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    WITH filtered_users AS (
        SELECT u.id, u.mfa_enabled
        FROM users u
        LEFT JOIN user_roles ur ON u.id = ur.user_id
        LEFT JOIN roles r ON ur.role_id = r.id
        WHERE u.enabled = true
          AND u.deleted_at IS NULL
          AND (
              p_satker_codes IS NULL
              OR u.satker_code = ANY(p_satker_codes)
          )
          AND (
              p_role_names IS NULL
              OR r.name = ANY(p_role_names)
          )
    )
    SELECT
        COUNT(*)::BIGINT as total_users,
        COUNT(*) FILTER (WHERE mfa_enabled = true)::BIGINT as mfa_enabled_users,
        CASE
            WHEN COUNT(*) > 0 THEN
                ROUND((COUNT(*) FILTER (WHERE mfa_enabled = true)::NUMERIC / COUNT(*)::NUMERIC) * 100, 2)
            ELSE 0
        END as compliance_percentage,
        COUNT(*) FILTER (WHERE mfa_enabled = false)::BIGINT as non_compliant_users
    FROM filtered_users;
END;
$$;


--
-- Name: get_mfa_usage_trends(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_mfa_usage_trends(p_days integer DEFAULT 30) RETURNS TABLE(date date, successful_verifications bigint, failed_verifications bigint, unique_users bigint, new_setups bigint)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        DATE(al.created_at) as date,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_verification_success')::BIGINT as successful_verifications,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_verification_failed')::BIGINT as failed_verifications,
        COUNT(DISTINCT al.user_id) FILTER (WHERE al.event_type IN ('mfa_verification_success', 'mfa_verification_failed'))::BIGINT as unique_users,
        COUNT(*) FILTER (WHERE al.event_type = 'mfa_setup_complete')::BIGINT as new_setups
    FROM audit_logs al
    WHERE al.created_at >= NOW() - (p_days || ' days')::INTERVAL
      AND al.event_type IN ('mfa_verification_success', 'mfa_verification_failed', 'mfa_setup_complete')
    GROUP BY DATE(al.created_at)
    ORDER BY date;
END;
$$;


--
-- Name: get_password_history(uuid, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_password_history(p_user_id uuid, p_limit integer DEFAULT 5) RETURNS TABLE(password_hash text, created_at timestamp with time zone)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT ph.password_hash, ph.created_at
    FROM password_history ph
    WHERE ph.user_id = p_user_id
    ORDER BY ph.created_at DESC
    LIMIT p_limit;
END;
$$;


--
-- Name: FUNCTION get_password_history(p_user_id uuid, p_limit integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_password_history(p_user_id uuid, p_limit integer) IS 'Retrieve password history for a user';


--
-- Name: get_recommended_challenge_type(character varying, numeric); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_recommended_challenge_type(p_user_id character varying, p_risk_score numeric DEFAULT 0.5) RETURNS character varying
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_preferred_types TEXT[];
    v_recommended_type VARCHAR(100);
BEGIN
    -- Get user's preferred types
    SELECT preferred_challenge_types INTO v_preferred_types
    FROM captcha_user_history
    WHERE user_id = p_user_id;

    -- If user has preferred types, return the first one
    IF v_preferred_types IS NOT NULL AND array_length(v_preferred_types, 1) > 0 THEN
        RETURN v_preferred_types[1];
    END IF;

    -- If high risk score, return Hybrid or Behavioral
    IF p_risk_score > 0.7 THEN
        RETURN 'Hybrid';
    END IF;

    -- Otherwise, return most effective challenge type
    SELECT challenge_type INTO v_recommended_type
    FROM captcha_type_effectiveness
    ORDER BY effectiveness_score DESC
    LIMIT 1;

    RETURN COALESCE(v_recommended_type, 'Visual');
END;
$$;


--
-- Name: FUNCTION get_recommended_challenge_type(p_user_id character varying, p_risk_score numeric); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_recommended_challenge_type(p_user_id character varying, p_risk_score numeric) IS 'Returns recommended challenge type based on user history and risk score';


--
-- Name: get_recommended_difficulty(character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_recommended_difficulty(p_user_id character varying) RETURNS smallint
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_difficulty SMALLINT;
BEGIN
    SELECT current_difficulty INTO v_difficulty
    FROM captcha_user_history
    WHERE user_id = p_user_id;

    RETURN COALESCE(v_difficulty, 3);
END;
$$;


--
-- Name: FUNCTION get_recommended_difficulty(p_user_id character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_recommended_difficulty(p_user_id character varying) IS 'Returns recommended difficulty level for user';


--
-- Name: get_user_mfa_status(uuid); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.get_user_mfa_status(p_user_id uuid) RETURNS TABLE(mfa_enabled boolean, mfa_setup_at timestamp with time zone, mfa_last_used timestamp with time zone)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.mfa_enabled,
        u.mfa_setup_at,
        u.mfa_last_used
    FROM users u
    WHERE u.id = p_user_id
      AND u.enabled = true
      AND u.deleted_at IS NULL;
END;
$$;


--
-- Name: FUNCTION get_user_mfa_status(p_user_id uuid); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.get_user_mfa_status(p_user_id uuid) IS 'Optimized function for single user MFA status lookup';


--
-- Name: log_mfa_admin_action(uuid, uuid, character varying, text, jsonb); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.log_mfa_admin_action(p_admin_user_id uuid, p_target_user_id uuid, p_action character varying, p_reason text DEFAULT NULL::text, p_metadata jsonb DEFAULT NULL::jsonb) RETURNS uuid
    LANGUAGE plpgsql
    AS $$
DECLARE
    action_id UUID;
BEGIN
    INSERT INTO mfa_admin_actions (
        admin_user_id,
        target_user_id,
        action,
        reason,
        metadata
    ) VALUES (
        p_admin_user_id,
        p_target_user_id,
        p_action,
        p_reason,
        p_metadata
    ) RETURNING id INTO action_id;

    RETURN action_id;
END;
$$;


--
-- Name: FUNCTION log_mfa_admin_action(p_admin_user_id uuid, p_target_user_id uuid, p_action character varying, p_reason text, p_metadata jsonb); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.log_mfa_admin_action(p_admin_user_id uuid, p_target_user_id uuid, p_action character varying, p_reason text, p_metadata jsonb) IS 'Logs MFA administrative actions for audit purposes';


--
-- Name: prevent_circular_groups(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.prevent_circular_groups() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    ancestor_id UUID;
BEGIN
    IF NEW.parent_id IS NULL THEN
        RETURN NEW;
    END IF;

    -- Check if the new parent is a descendant of this group
    ancestor_id := NEW.parent_id;
    WHILE ancestor_id IS NOT NULL LOOP
        IF ancestor_id = NEW.id THEN
            RAISE EXCEPTION 'Circular group hierarchy detected';
        END IF;
        SELECT parent_id INTO ancestor_id FROM groups WHERE id = ancestor_id;
    END LOOP;

    RETURN NEW;
END;
$$;


--
-- Name: record_captcha_validation(uuid, inet, text, text, boolean, double precision, character varying, uuid); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.record_captcha_validation(p_challenge_id uuid, p_ip_address inet, p_user_agent text, p_answer_provided text, p_success boolean, p_confidence_score double precision DEFAULT NULL::double precision, p_risk_assessment character varying DEFAULT 'Medium'::character varying, p_behavioral_metrics_id uuid DEFAULT NULL::uuid) RETURNS uuid
    LANGUAGE plpgsql
    AS $$
DECLARE
    attempt_id UUID;
    current_attempts INTEGER;
BEGIN
    -- Insert validation attempt
    INSERT INTO captcha_validation_attempts (
        challenge_id,
        ip_address,
        user_agent,
        answer_provided,
        success,
        confidence_score,
        risk_assessment,
        behavioral_metrics_id
    ) VALUES (
        p_challenge_id,
        p_ip_address,
        p_user_agent,
        p_answer_provided,
        p_success,
        p_confidence_score,
        p_risk_assessment,
        p_behavioral_metrics_id
    ) RETURNING id INTO attempt_id;

    -- Update challenge attempts counter and solved status
    UPDATE captcha_challenges
    SET
        attempts = attempts + 1,
        solved = CASE WHEN p_success THEN true ELSE solved END,
        solved_at = CASE WHEN p_success THEN NOW() ELSE solved_at END
    WHERE id = p_challenge_id;

    RETURN attempt_id;
END;
$$;


--
-- Name: FUNCTION record_captcha_validation(p_challenge_id uuid, p_ip_address inet, p_user_agent text, p_answer_provided text, p_success boolean, p_confidence_score double precision, p_risk_assessment character varying, p_behavioral_metrics_id uuid); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.record_captcha_validation(p_challenge_id uuid, p_ip_address inet, p_user_agent text, p_answer_provided text, p_success boolean, p_confidence_score double precision, p_risk_assessment character varying, p_behavioral_metrics_id uuid) IS 'Records a CAPTCHA validation attempt with all metadata';


--
-- Name: refresh_captcha_analytics(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.refresh_captcha_analytics() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY captcha_analytics;
END;
$$;


--
-- Name: FUNCTION refresh_captcha_analytics(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.refresh_captcha_analytics() IS 'Refreshes the CAPTCHA analytics materialized view';


--
-- Name: refresh_mfa_statistics(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.refresh_mfa_statistics() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW mfa_statistics;
END;
$$;


--
-- Name: FUNCTION refresh_mfa_statistics(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.refresh_mfa_statistics() IS 'Refreshes the MFA statistics materialized view';


--
-- Name: trigger_refresh_captcha_analytics(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.trigger_refresh_captcha_analytics() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Only refresh if it's been more than 10 minutes since last refresh
    IF (
        SELECT EXTRACT(EPOCH FROM (NOW() - MAX(last_updated)))
        FROM captcha_analytics
        WHERE last_updated IS NOT NULL
    ) > 600 OR NOT EXISTS (SELECT 1 FROM captcha_analytics) THEN
        PERFORM refresh_captcha_analytics();
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;


--
-- Name: trigger_refresh_mfa_statistics(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.trigger_refresh_mfa_statistics() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Only refresh if it's been more than 5 minutes since last refresh
    IF (
        SELECT EXTRACT(EPOCH FROM (NOW() - MAX(last_updated)))
        FROM mfa_statistics
        WHERE last_updated IS NOT NULL
    ) > 300 OR NOT EXISTS (SELECT 1 FROM mfa_statistics) THEN
        PERFORM refresh_mfa_statistics();
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;


--
-- Name: update_captcha_type_effectiveness(character varying, boolean, numeric, boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_captcha_type_effectiveness(p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric, p_bot_detected boolean) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_success_value NUMERIC(5,4);
    v_bot_value NUMERIC(5,4);
    v_time_factor NUMERIC(5,4);
    v_satisfaction NUMERIC(5,4);
BEGIN
    v_success_value := CASE WHEN p_success THEN 1.0 ELSE 0.0 END;
    v_bot_value := CASE WHEN p_bot_detected THEN 1.0 ELSE 0.0 END;
    v_time_factor := CASE WHEN p_completion_time_secs < 60.0 THEN 1.0 ELSE 0.5 END;
    v_satisfaction := CASE WHEN p_success THEN v_time_factor ELSE 0.3 END;

    UPDATE captcha_type_effectiveness
    SET
        usage_count = usage_count + 1,
        success_rate = (success_rate * 0.9) + (v_success_value * 0.1),
        avg_completion_time_secs = (avg_completion_time_secs * 0.9) + (p_completion_time_secs * 0.1),
        bot_detection_rate = (bot_detection_rate * 0.9) + (v_bot_value * 0.1),
        user_satisfaction = (user_satisfaction * 0.9) + (v_satisfaction * 0.1),
        effectiveness_score = ((success_rate * 0.9 + v_success_value * 0.1) * 0.3) +
                             ((bot_detection_rate * 0.9 + v_bot_value * 0.1) * 0.4) +
                             ((user_satisfaction * 0.9 + v_satisfaction * 0.1) * 0.3),
        last_updated = NOW()
    WHERE challenge_type = p_challenge_type;
END;
$$;


--
-- Name: FUNCTION update_captcha_type_effectiveness(p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric, p_bot_detected boolean); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.update_captcha_type_effectiveness(p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric, p_bot_detected boolean) IS 'Updates global effectiveness metrics for challenge types';


--
-- Name: update_captcha_user_history(character varying, character varying, boolean, numeric); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_captcha_user_history(p_user_id character varying, p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_current_difficulty SMALLINT;
    v_success_rate NUMERIC(5,4);
BEGIN
    -- Insert or update user history
    INSERT INTO captcha_user_history (user_id, total_attempts, successful_completions, failed_attempts, last_challenge_time)
    VALUES (
        p_user_id,
        1,
        CASE WHEN p_success THEN 1 ELSE 0 END,
        CASE WHEN p_success THEN 0 ELSE 1 END,
        NOW()
    )
    ON CONFLICT (user_id) DO UPDATE SET
        total_attempts = captcha_user_history.total_a+ 1,
        successful_completions = captcha_user_history.successful_completions + CASE WHEN p_success THEN 1 ELSE 0 END,
        failed_attempts = captcha_user_history.failed_attempts + CASE WHEN NOT p_success THEN 1 ELSE 0 END,
        last_challenge_time = NOW(),
        updated_at = NOW();

    -- Insert or update user type performance
    INSERT INTO captcha_user_type_performance (user_id, challenge_type, attempts, successes, avg_completion_time_secs, last_attempt_time)
    VALUES (
        p_user_id,
        p_challenge_type,
        1,
        CASE WHEN p_success THEN 1 ELSE 0 END,
        p_completion_time_secs,
        NOW()
    )
    ON CONFLICT (user_id, challenge_type) DO UPDATE SET
        attempts = captcha_user_type_performance.attempts + 1,
        successes = captcha_user_type_performance.successes + CASE WHEN p_success THEN 1 ELSE 0 END,
        success_rate = (captcha_user_type_performance.successes + CASE WHEN p_success THEN 1 ELSE 0 END)::NUMERIC /
                      (captcha_user_type_performance.attempts + 1)::NUMERIC,
        avg_completion_time_secs = (captcha_user_type_performance.avg_completion_time_secs * 0.7) + (p_completion_time_secs * 0.3),
        last_attempt_time = NOW(),
        updated_at = NOW();

    -- Update preferred and avoid types based on performance
    WITH type_performance AS (
        SELECT challenge_type, success_rate
        FROM captcha_user_type_performance
        WHERE user_id = p_user_id
    )
    UPDATE captcha_user_history
    SET
        preferred_challenge_types = ARRAY(
            SELECT challenge_type FROM type_performance WHERE success_rate >= 0.7
        ),
        avoid_challenge_types = ARRAY(
            SELECT challenge_type FROM type_performance WHERE success_rate < 0.3
        )
    WHERE user_id = p_user_id;

    -- Adjust difficulty based on overall success rate
    SELECT
        CASE
            WHEN successful_completions::NUMERIC / NULLIF(total_attempts, 0) >= 0.8
                THEN LEAST(current_difficulty + 1, 10)
            WHEN successful_completions::NUMERIC / NULLIF(total_attempts, 0) < 0.4
                THEN GREATEST(current_difficulty - 1, 1)
            ELSE current_difficulty
        END
    INTO v_current_difficulty
    FROM captcha_user_history
    WHERE user_id = p_user_id;

    UPDATE captcha_user_history
    SET current_difficulty = v_current_difficulty
    WHERE user_id = p_user_id;
END;
$$;


--
-- Name: FUNCTION update_captcha_user_history(p_user_id character varying, p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.update_captcha_user_history(p_user_id character varying, p_challenge_type character varying, p_success boolean, p_completion_time_secs numeric) IS 'Updates user history and adjusts difficulty after challenge completion';


--
-- Name: update_child_group_paths(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_child_group_paths() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.path != NEW.path THEN
        UPDATE groups
        SET path = REPLACE(path, OLD.path, NEW.path)
        WHERE path LIKE OLD.path || '/%';
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: update_client_policy_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_client_policy_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_client_scopes_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_client_scopes_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_group_path(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_group_path() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    parent_path TEXT;
BEGIN
    IF NEW.parent_id IS NULL THEN
        NEW.path := '/' || NEW.name;
    ELSE
        SELECT path INTO parent_path FROM groups WHERE id = NEW.parent_id;
        NEW.path := parent_path || '/' || NEW.name;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: update_satker_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_satker_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_service_account_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_service_account_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


--
-- Name: update_webauthn_credential_last_used(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.update_webauthn_credential_last_used() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- This trigger can be used by application to auto-update last_used
    -- Currently, application handles this explicitly
    RETURN NEW;
END;
$$;


--
-- Name: user_can_access_path(uuid, character varying, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.user_can_access_path(p_user_id uuid, p_path character varying, p_action character varying) RETURNS boolean
    LANGUAGE plpgsql STABLE
    AS $$
DECLARE
    v_allowed BOOLEAN := FALSE;
    v_denied BOOLEAN := FALSE;
BEGIN
    -- Check deny policies first (highest priority)
    SELECT TRUE INTO v_denied
    FROM user_policies up
    JOIN authorization_policies ap ON up.policy_id = ap.id
    WHERE up.user_id = p_user_id
    AND ap.effect = 'deny'
    AND ap.enabled = TRUE
    AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
    AND (
        ap.capabilities @> to_jsonb(p_action)::jsonb
        OR ap.capabilities @> '["*"]'::jsonb
    )
    AND (up.expires_at IS NULL OR up.expires_at > NOW())
    LIMIT 1;

    IF v_denied THEN
        RETURN FALSE;
    END IF;

    -- Check allow policies
    SELECT TRUE INTO v_allowed
    FROM (
        -- From role policies
        SELECT ap.id
        FROM user_roles ur
        JOIN role_policies rp ON ur.role_id = rp.role_id
        JOIN authorization_policies ap ON rp.policy_id = ap.id
        WHERE ur.user_id = p_user_id
        AND ap.effect = 'allow'
        AND ap.enabled = TRUE
        AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
        AND (
            ap.capabilities @> to_jsonb(p_action)::jsonb
            OR ap.capabilities @> '["*"]'::jsonb
        )
        AND (rp.expires_at IS NULL OR rp.expires_at > NOW())

        UNION

        -- From direct user policies
        SELECT ap.id
        FROM user_policies up
        JOIN authorization_policies ap ON up.policy_id = ap.id
        WHERE up.user_id = p_user_id
        AND ap.effect = 'allow'
        AND ap.enabled = TRUE
        AND p_path LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
        AND (
            ap.capabilities @> to_jsonb(p_action)::jsonb
            OR ap.capabilities @> '["*"]'::jsonb
        )
        AND (up.expires_at IS NULL OR up.expires_at > NOW())
    ) allowed_policies
    LIMIT 1;

    RETURN COALESCE(v_allowed, FALSE);
END;
$$;


--
-- Name: FUNCTION user_can_access_path(p_user_id uuid, p_path character varying, p_action character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.user_can_access_path(p_user_id uuid, p_path character varying, p_action character varying) IS 'Check if user can access a path with given action (Vault-style)';


--
-- Name: user_has_capability(uuid, character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.user_has_capability(p_user_id uuid, p_capability_code character varying) RETURNS boolean
    LANGUAGE plpgsql STABLE
    AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM v_user_effective_capabilities
        WHERE user_id = p_user_id
        AND (capability_code = p_capability_code OR capability_code = '*')
    );
END;
$$;


--
-- Name: FUNCTION user_has_capability(p_user_id uuid, p_capability_code character varying); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION authenc.user_has_capability(p_user_id uuid, p_capability_code character varying) IS 'Check if user has a specific capability';


--
-- Name: validate_client_scopes(uuid, text[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION authenc.validate_client_scopes(p_client_id uuid, p_requested_scopes text[]) RETURNS TABLE(valid boolean, invalid_scopes text[])
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_allowed_scopes TEXT[];
    v_invalid_scopes TEXT[];
BEGIN
    -- Get all allowed scopes for client (default + optional)
    SELECT ARRAY_AGG(scope_name) INTO v_allowed_scopes
    FROM client_all_scopes
    WHERE client_id = p_client_id;

    -- Find invalid scopes
    SELECT ARRAY_AGG(scope) INTO v_invalid_scopes
    FROM UNNEST(p_requested_scopes) AS scope
    WHERE scope != ALL(v_allowed_scopes);

    -- Return validation result
    IF v_invalid_scopes IS NULL OR ARRAY_LENGTH(v_invalid_scopes, 1) = 0 THEN
        RETURN QUERY SELECT TRUE, NULL::TEXT[];
    ELSE
        RETURN QUERY SELECT FALSE, v_invalid_scopes;
    END IF;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: mfa_backup_codes; Type: TABLE; Schema: authenc; Owner: -
--

CREATE TABLE authenc.mfa_backup_codes (
    user_id uuid NOT NULL,
    code text NOT NULL,
    used boolean DEFAULT false NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE mfa_backup_codes; Type: COMMENT; Schema: authenc; Owner: -
--

COMMENT ON TABLE authenc.mfa_backup_codes IS 'One-time MFA recovery codes; pruned on regenerate';


--
-- Name: COLUMN mfa_backup_codes.used; Type: COMMENT; Schema: authenc; Owner: -
--

COMMENT ON COLUMN authenc.mfa_backup_codes.used IS 'Once true, never reused — enforced at the application layer';


--
-- Name: token_revocations; Type: TABLE; Schema: authenc; Owner: -
--

CREATE TABLE authenc.token_revocations (
    kind text NOT NULL,
    value text NOT NULL,
    revoked_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    reason text,
    CONSTRAINT token_revocations_kind_check CHECK ((kind = ANY (ARRAY['jti'::text, 'sid'::text, 'user'::text])))
);


--
-- Name: totp_secrets; Type: TABLE; Schema: authenc; Owner: -
--

CREATE TABLE authenc.totp_secrets (
    user_id uuid NOT NULL,
    secret text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE totp_secrets; Type: COMMENT; Schema: authenc; Owner: -
--

COMMENT ON TABLE authenc.totp_secrets IS 'Per-user TOTP secret (base32) backing authenc_mfa::TotpStore';


--
-- Name: COLUMN totp_secrets.secret; Type: COMMENT; Schema: authenc; Owner: -
--

COMMENT ON COLUMN authenc.totp_secrets.secret IS 'Base32-encoded TOTP secret; rotate on disable/re-enroll';


--
-- Name: access_levels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.access_levels (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    numeric_level integer NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    capabilities jsonb DEFAULT '[]'::jsonb NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE access_levels; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.access_levels IS 'Dynamic access level registry - replaces AccessLevel enum';


--
-- Name: COLUMN access_levels.numeric_level; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.access_levels.numeric_level IS 'Higher number = more privileges';


--
-- Name: COLUMN access_levels.capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.access_levels.capabilities IS 'JSON array of capability codes this level grants';


--
-- Name: account_linking_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.account_linking_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    realm_id uuid NOT NULL,
    identity_provider_alias character varying(255) NOT NULL,
    federated_user_id character varying(255) NOT NULL,
    federated_username character varying(255),
    federated_email character varying(255),
    status character varying(50) DEFAULT 'PENDING'::character varying NOT NULL,
    confirmation_token character varying(255),
    federated_attributes jsonb,
    expires_at timestamp without time zone NOT NULL,
    resolved_at timestamp without time zone,
    resolved_by character varying(50),
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: actor_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.actor_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    is_human boolean DEFAULT true NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE actor_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.actor_types IS 'Dynamic actor type registry - replaces hardcoded actor_type strings';


--
-- Name: COLUMN actor_types.is_human; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.actor_types.is_human IS 'Whether this actor type represents human users';


--
-- Name: admin_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_audit_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    admin_user_id uuid,
    admin_username character varying(255),
    admin_ip_address inet,
    operation_type character varying(100) NOT NULL,
    resource_type character varying(100) NOT NULL,
    resource_id character varying(255),
    resource_name character varying(255),
    action character varying(255) NOT NULL,
    status character varying(50) DEFAULT 'SUCCESS'::character varying NOT NULL,
    error_message text,
    request_method character varying(10),
    request_path text,
    request_body jsonb,
    response_status integer,
    response_body jsonb,
    duration_ms integer,
    user_agent text,
    session_id uuid,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    geolocation_data jsonb,
    request_payload jsonb,
    response_payload jsonb
);


--
-- Name: COLUMN admin_audit_log.geolocation_data; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_audit_log.geolocation_data IS 'Geolocation data for the admin IP address';


--
-- Name: COLUMN admin_audit_log.request_payload; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_audit_log.request_payload IS 'Sanitized request payload for admin operations';


--
-- Name: COLUMN admin_audit_log.response_payload; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_audit_log.response_payload IS 'Sanitized response payload for admin operations';


--
-- Name: admin_console_preferences; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_console_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    admin_user_id uuid NOT NULL,
    realm_id uuid NOT NULL,
    theme character varying(50) DEFAULT 'light'::character varying,
    language character varying(10) DEFAULT 'en'::character varying,
    timezone character varying(100) DEFAULT 'UTC'::character varying,
    items_per_page integer DEFAULT 25,
    compact_mode boolean DEFAULT false,
    sidebar_collapsed boolean DEFAULT false,
    email_notifications boolean DEFAULT true,
    desktop_notifications boolean DEFAULT true,
    notification_frequency character varying(50) DEFAULT 'realtime'::character varying,
    dashboard_layout jsonb,
    favorite_pages text[],
    developer_mode boolean DEFAULT false,
    show_advanced_options boolean DEFAULT false,
    preferences jsonb,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: admin_console_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_console_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    admin_user_id uuid NOT NULL,
    username character varying(255) NOT NULL,
    session_token character varying(512) NOT NULL,
    ip_address inet,
    user_agent text,
    is_active boolean DEFAULT true NOT NULL,
    last_activity_at timestamp without time zone DEFAULT now() NOT NULL,
    login_method character varying(50),
    mfa_verified boolean DEFAULT false NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    logout_reason character varying(100),
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    terminated_at timestamp without time zone
);


--
-- Name: admin_dashboard_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_dashboard_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    metric_type character varying(100) NOT NULL,
    metric_name character varying(255) NOT NULL,
    metric_value numeric NOT NULL,
    metric_unit character varying(50),
    aggregation_period character varying(50) NOT NULL,
    period_start timestamp without time zone NOT NULL,
    period_end timestamp without time zone NOT NULL,
    metadata jsonb,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: admin_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_events (
    id character varying(36) NOT NULL,
    "time" timestamp with time zone DEFAULT now() NOT NULL,
    realm_id character varying(36) NOT NULL,
    realm_name character varying(255),
    auth_user_id character varying(36),
    auth_username character varying(255),
    auth_realm character varying(255),
    auth_client character varying(255),
    auth_ip_address inet,
    auth_user_agent text,
    resource_type character varying(50) NOT NULL,
    operation_type character varying(50) NOT NULL,
    resource_path text NOT NULL,
    representation text,
    error text,
    signature character varying(128),
    geolocation_data jsonb
);


--
-- Name: COLUMN admin_events.geolocation_data; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_events.geolocation_data IS 'Geolocation data for admin events';


--
-- Name: admin_level_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_level_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    hierarchy_level integer NOT NULL,
    scope_type character varying(100) NOT NULL,
    parent_level_id uuid,
    can_manage_levels jsonb DEFAULT '[]'::jsonb NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE admin_level_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.admin_level_types IS 'Dynamic admin level hierarchy - replaces AdminLevel enum';


--
-- Name: COLUMN admin_level_types.hierarchy_level; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_level_types.hierarchy_level IS 'Higher number = higher authority';


--
-- Name: COLUMN admin_level_types.scope_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_level_types.scope_type IS 'Type of scope this level manages (satker, wilayah, pusat, etc)';


--
-- Name: COLUMN admin_level_types.can_manage_levels; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.admin_level_types.can_manage_levels IS 'JSON array of level codes this level can manage';


--
-- Name: admin_notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.admin_notifications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    notification_type character varying(100) NOT NULL,
    title character varying(255) NOT NULL,
    message text NOT NULL,
    target_admin_user_id uuid,
    target_role character varying(255),
    is_read boolean DEFAULT false NOT NULL,
    read_at timestamp without time zone,
    read_by_user_id uuid,
    action_url text,
    action_label character varying(100),
    priority integer DEFAULT 5 NOT NULL,
    expires_at timestamp without time zone,
    metadata jsonb,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: audit_integrity_checks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.audit_integrity_checks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    check_time timestamp without time zone DEFAULT now() NOT NULL,
    events_checked integer DEFAULT 0 NOT NULL,
    admin_events_checked integer DEFAULT 0 NOT NULL,
    events_failed integer DEFAULT 0 NOT NULL,
    admin_events_failed integer DEFAULT 0 NOT NULL,
    check_duration_ms integer,
    status character varying(50) NOT NULL,
    error_details text,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: audit_integrity_failures; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.audit_integrity_failures (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    check_id uuid NOT NULL,
    event_type character varying(50) NOT NULL,
    event_id uuid NOT NULL,
    expected_signature character varying(128),
    actual_signature character varying(128),
    event_data jsonb,
    detected_at timestamp without time zone DEFAULT now() NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.audit_logs (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    event_type character varying(100) NOT NULL,
    user_id uuid,
    session_id uuid,
    client_id uuid,
    resource_type character varying(50),
    resource_id uuid,
    action character varying(50) NOT NULL,
    status character varying(20) DEFAULT 'success'::character varying NOT NULL,
    details jsonb,
    ip_address inet,
    user_agent text,
    location_data jsonb,
    error_message text,
    request_id character varying(100),
    correlation_id character varying(100),
    CONSTRAINT audit_logs_status_check CHECK (((status)::text = ANY ((ARRAY['success'::character varying, 'failure'::character varying, 'warning'::character varying])::text[])))
);


--
-- Name: authentication_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authentication_executions (
    id uuid NOT NULL,
    flow_id uuid NOT NULL,
    authenticator character varying(255),
    authenticator_config uuid,
    authenticator_flow boolean DEFAULT false NOT NULL,
    requirement character varying(50) DEFAULT 'DISABLED'::character varying NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    parent_flow uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authentication_flows; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authentication_flows (
    id uuid NOT NULL,
    realm_id uuid NOT NULL,
    alias character varying(255) NOT NULL,
    description text,
    provider_id character varying(255) NOT NULL,
    top_level boolean DEFAULT false NOT NULL,
    built_in boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authentication_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authentication_sessions (
    id uuid NOT NULL,
    realm_id uuid NOT NULL,
    user_id uuid,
    client_id character varying(255),
    flow_id uuid,
    auth_state character varying(50) DEFAULT 'STARTED'::character varying NOT NULL,
    protocol character varying(50) DEFAULT 'openid-connect'::character varying NOT NULL,
    redirect_uri text,
    current_execution uuid,
    execution_status jsonb DEFAULT '{}'::jsonb,
    authentication_notes jsonb DEFAULT '{}'::jsonb,
    client_notes jsonb DEFAULT '{}'::jsonb,
    required_actions jsonb DEFAULT '[]'::jsonb,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    expires_at timestamp with time zone NOT NULL,
    success boolean,
    error_message text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authenticator_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authenticator_configs (
    id uuid NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    alias character varying(255) NOT NULL,
    authenticator_type character varying(100) NOT NULL,
    config jsonb DEFAULT '{}'::jsonb NOT NULL,
    priority integer DEFAULT 100 NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authenticator_execution_results; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authenticator_execution_results (
    id uuid NOT NULL,
    execution_id uuid NOT NULL,
    session_id uuid,
    user_id uuid,
    status character varying(50) NOT NULL,
    error_message text,
    duration_ms integer,
    attempt_count integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authenticator_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authenticator_executions (
    id uuid NOT NULL,
    realm_id uuid NOT NULL,
    flow_id uuid NOT NULL,
    authenticator_id uuid,
    requirement character varying(50) DEFAULT 'REQUIRED'::character varying NOT NULL,
    priority integer DEFAULT 100 NOT NULL,
    parent_flow_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: authorization_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.authorization_policies (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    policy_type character varying(50) DEFAULT 'acl'::character varying NOT NULL,
    effect character varying(20) DEFAULT 'allow'::character varying NOT NULL,
    path_pattern character varying(1000) NOT NULL,
    capabilities jsonb DEFAULT '[]'::jsonb NOT NULL,
    conditions jsonb DEFAULT '{}'::jsonb NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT authorization_policies_effect_check CHECK (((effect)::text = ANY ((ARRAY['allow'::character varying, 'deny'::character varying])::text[])))
);


--
-- Name: TABLE authorization_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.authorization_policies IS 'Vault-style authorization policies';


--
-- Name: COLUMN authorization_policies.path_pattern; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.authorization_policies.path_pattern IS 'Glob pattern for resource paths';


--
-- Name: COLUMN authorization_policies.capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.authorization_policies.capabilities IS 'JSON array of capability codes granted by this policy';


--
-- Name: COLUMN authorization_policies.conditions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.authorization_policies.conditions IS 'JSON conditions for policy evaluation';


--
-- Name: capabilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.capabilities (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    resource_type character varying(100),
    action character varying(100) NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    is_dangerous boolean DEFAULT false NOT NULL,
    requires_mfa boolean DEFAULT false NOT NULL,
    requires_approval boolean DEFAULT false NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.capabilities IS 'Fine-grained capabilities/permissions registry';


--
-- Name: COLUMN capabilities.code; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.code IS 'Unique capability code (e.g., users:read, secrets:write)';


--
-- Name: COLUMN capabilities.resource_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.resource_type IS 'Resource type this capability applies to';


--
-- Name: COLUMN capabilities.action; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.action IS 'Action type (read, write, delete, admin, etc)';


--
-- Name: COLUMN capabilities.is_dangerous; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.is_dangerous IS 'Dangerous capabilities require extra confirmation';


--
-- Name: COLUMN capabilities.requires_mfa; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.requires_mfa IS 'Capability requires MFA verification';


--
-- Name: COLUMN capabilities.requires_approval; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.capabilities.requires_approval IS 'Capability requires approval workflow';


--
-- Name: captcha_behavioral_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_behavioral_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    challenge_id uuid NOT NULL,
    session_id character varying(255) NOT NULL,
    mouse_movements jsonb,
    keystroke_dynamics jsonb,
    timing_patterns jsonb NOT NULL,
    browser_fingerprint jsonb NOT NULL,
    risk_score double precision NOT NULL,
    classification character varying(20) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT captcha_behavioral_metrics_classification_check CHECK (((classification)::text = ANY ((ARRAY['Human'::character varying, 'Suspicious'::character varying, 'Bot'::character varying, 'Unknown'::character varying])::text[]))),
    CONSTRAINT captcha_behavioral_metrics_risk_score_check CHECK (((risk_score >= (0.0)::double precision) AND (risk_score <= (1.0)::double precision)))
);


--
-- Name: TABLE captcha_behavioral_metrics; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_behavioral_metrics IS 'Stores behavioral analysis data for bot detection';


--
-- Name: captcha_challenges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_challenges (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    challenge_type character varying(50) NOT NULL,
    difficulty_level smallint NOT NULL,
    encrypted_data text NOT NULL,
    expected_answer_hash character varying(256) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    session_id character varying(255),
    ip_address inet NOT NULL,
    solved boolean DEFAULT false NOT NULL,
    solved_at timestamp with time zone,
    attempts integer DEFAULT 0 NOT NULL,
    max_attempts integer DEFAULT 3 NOT NULL,
    CONSTRAINT captcha_challenges_challenge_type_check CHECK (((challenge_type)::text = ANY ((ARRAY['Visual'::character varying, 'Audio'::character varying, 'Behavioral'::character varying, 'Logical'::character varying, 'Hybrid'::character varying])::text[]))),
    CONSTRAINT captcha_challenges_difficulty_level_check CHECK (((difficulty_level >= 1) AND (difficulty_level <= 10)))
);


--
-- Name: TABLE captcha_challenges; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_challenges IS 'Stores CAPTCHA challenges with encrypted data and metadata';


--
-- Name: captcha_validation_attempts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_validation_attempts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    challenge_id uuid NOT NULL,
    ip_address inet NOT NULL,
    user_agent text,
    answer_provided text NOT NULL,
    success boolean NOT NULL,
    confidence_score double precision,
    risk_assessment character varying(20) NOT NULL,
    behavioral_metrics_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT captcha_validation_attempts_confidence_score_check CHECK (((confidence_score >= (0.0)::double precision) AND (confidence_score <= (1.0)::double precision))),
    CONSTRAINT captcha_validation_attempts_risk_assessment_check CHECK (((risk_assessment)::text = ANY ((ARRAY['Low'::character varying, 'Medium'::character varying, 'High'::character varying, 'Critical'::character varying])::text[])))
);


--
-- Name: TABLE captcha_validation_attempts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_validation_attempts IS 'Audit log of all CAPTCHA validation attempts';


--
-- Name: captcha_analytics; Type: MATERIALIZED VIEW; Schema: public; Owner: -
--

CREATE MATERIALIZED VIEW authenc.captcha_analytics AS
 SELECT date(c.created_at) AS date,
    c.challenge_type,
    c.difficulty_level,
    count(*) AS total_challenges,
    count(*) FILTER (WHERE (c.solved = true)) AS solved_challenges,
    count(*) FILTER (WHERE ((c.solved = false) AND (c.expires_at < now()))) AS expired_challenges,
    avg(c.attempts) AS avg_attempts,
    count(DISTINCT c.ip_address) AS unique_ips,
    count(DISTINCT c.session_id) AS unique_sessions,
    avg(bm.risk_score) AS avg_risk_score,
    count(*) FILTER (WHERE ((bm.classification)::text = 'Bot'::text)) AS bot_detections,
    count(*) FILTER (WHERE ((bm.classification)::text = 'Suspicious'::text)) AS suspicious_detections,
    count(*) FILTER (WHERE ((bm.classification)::text = 'Human'::text)) AS human_detections,
    count(va.id) AS total_validation_attempts,
    count(*) FILTER (WHERE (va.success = true)) AS successful_validations,
    avg(va.confidence_score) AS avg_confidence_score,
    count(*) FILTER (WHERE ((va.risk_assessment)::text = 'High'::text)) AS high_risk_attempts,
    count(*) FILTER (WHERE ((va.risk_assessment)::text = 'Critical'::text)) AS critical_risk_attempts,
    max(c.created_at) AS last_updated
   FROM ((authenc.captcha_challenges c
     LEFT JOIN authenc.captcha_behavioral_metrics bm ON ((c.id = bm.challenge_id)))
     LEFT JOIN authenc.captcha_validation_attempts va ON ((c.id = va.challenge_id)))
  WHERE (c.created_at >= (now() - '90 days'::interval))
  GROUP BY (date(c.created_at)), c.challenge_type, c.difficulty_level
  WITH NO DATA;


--
-- Name: MATERIALIZED VIEW captcha_analytics; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON MATERIALIZED VIEW authenc.captcha_analytics IS 'Aggregated CAPTCHA analytics for monitoring and reporting';


--
-- Name: captcha_bot_detection_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_bot_detection_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    total_detections bigint NOT NULL,
    true_positives bigint NOT NULL,
    false_positives bigint NOT NULL,
    true_negatives bigint NOT NULL,
    false_negatives bigint NOT NULL,
    accuracy_rate double precision NOT NULL,
    precision_rate double precision NOT NULL,
    recall_rate double precision NOT NULL,
    f1_score double precision NOT NULL,
    risk_distribution jsonb NOT NULL,
    CONSTRAINT captcha_bot_detection_metrics_accuracy_rate_check CHECK (((accuracy_rate >= (0.0)::double precision) AND (accuracy_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_bot_detection_metrics_f1_score_check CHECK (((f1_score >= (0.0)::double precision) AND (f1_score <= (1.0)::double precision))),
    CONSTRAINT captcha_bot_detection_metrics_precision_rate_check CHECK (((precision_rate >= (0.0)::double precision) AND (precision_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_bot_detection_metrics_recall_rate_check CHECK (((recall_rate >= (0.0)::double precision) AND (recall_rate <= (1.0)::double precision)))
);


--
-- Name: captcha_difficulty_adjustments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_difficulty_adjustments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    ip_pattern cidr,
    session_pattern character varying(255),
    difficulty_level smallint NOT NULL,
    reason text,
    created_by uuid,
    active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone,
    CONSTRAINT captcha_difficulty_adjustments_difficulty_level_check CHECK (((difficulty_level >= 1) AND (difficulty_level <= 10)))
);


--
-- Name: TABLE captcha_difficulty_adjustments; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_difficulty_adjustments IS 'Configuration for adaptive difficulty adjustments';


--
-- Name: captcha_performance_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_performance_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    challenge_generation_latency_ms bigint NOT NULL,
    validation_latency_ms bigint NOT NULL,
    success_rate double precision NOT NULL,
    failure_rate double precision NOT NULL,
    average_difficulty double precision NOT NULL,
    concurrent_challenges bigint NOT NULL,
    memory_usage_mb double precision NOT NULL,
    cpu_usage_percent double precision NOT NULL,
    CONSTRAINT captcha_performance_metrics_cpu_usage_percent_check CHECK (((cpu_usage_percent >= (0.0)::double precision) AND (cpu_usage_percent <= (100.0)::double precision))),
    CONSTRAINT captcha_performance_metrics_failure_rate_check CHECK (((failure_rate >= (0.0)::double precision) AND (failure_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_performance_metrics_success_rate_check CHECK (((success_rate >= (0.0)::double precision) AND (success_rate <= (1.0)::double precision)))
);


--
-- Name: captcha_security_event_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_security_event_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    attack_attempts bigint NOT NULL,
    blocked_ips bigint NOT NULL,
    rate_limit_triggers bigint NOT NULL,
    lockout_events bigint NOT NULL,
    suspicious_behavior_count bigint NOT NULL,
    threat_level_distribution jsonb NOT NULL,
    geographic_distribution jsonb NOT NULL
);


--
-- Name: captcha_type_effectiveness; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_type_effectiveness (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    challenge_type character varying(100) NOT NULL,
    usage_count integer DEFAULT 0 NOT NULL,
    success_rate numeric(5,4) DEFAULT 0.5 NOT NULL,
    avg_completion_time_secs numeric(8,2) DEFAULT 30.0 NOT NULL,
    bot_detection_rate numeric(5,4) DEFAULT 0.5 NOT NULL,
    user_satisfaction numeric(5,4) DEFAULT 0.5 NOT NULL,
    effectiveness_score numeric(5,4) DEFAULT 0.5 NOT NULL,
    last_updated timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT captcha_type_effectiveness_bot_detection_rate_check CHECK (((bot_detection_rate >= 0.0) AND (bot_detection_rate <= 1.0))),
    CONSTRAINT captcha_type_effectiveness_effectiveness_score_check CHECK (((effectiveness_score >= 0.0) AND (effectiveness_score <= 1.0))),
    CONSTRAINT captcha_type_effectiveness_success_rate_check CHECK (((success_rate >= 0.0) AND (success_rate <= 1.0))),
    CONSTRAINT captcha_type_effectiveness_user_satisfaction_check CHECK (((user_satisfaction >= 0.0) AND (user_satisfaction <= 1.0)))
);


--
-- Name: TABLE captcha_type_effectiveness; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_type_effectiveness IS 'Global effectiveness metrics for each challenge type';


--
-- Name: captcha_user_type_performance; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_user_type_performance (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id character varying(255) NOT NULL,
    challenge_type character varying(100) NOT NULL,
    attempts integer DEFAULT 0 NOT NULL,
    successes integer DEFAULT 0 NOT NULL,
    success_rate numeric(5,4) DEFAULT 0.0 NOT NULL,
    avg_completion_time_secs numeric(8,2) DEFAULT 30.0 NOT NULL,
    last_attempt_time timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT captcha_user_type_performance_success_rate_check CHECK (((success_rate >= 0.0) AND (success_rate <= 1.0)))
);


--
-- Name: TABLE captcha_user_type_performance; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_user_type_performance IS 'Tracks user performance by challenge type';


--
-- Name: captcha_type_analytics; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.captcha_type_analytics AS
 SELECT cte.challenge_type,
    cte.usage_count,
    cte.success_rate,
    cte.avg_completion_time_secs,
    cte.bot_detection_rate,
    cte.user_satisfaction,
    cte.effectiveness_score,
    count(DISTINCT cutp.user_id) AS unique_users,
    avg(cutp.success_rate) AS avg_user_success_rate,
    cte.last_updated
   FROM (authenc.captcha_type_effectiveness cte
     LEFT JOIN authenc.captcha_user_type_performance cutp ON (((cte.challenge_type)::text = (cutp.challenge_type)::text)))
  GROUP BY cte.challenge_type, cte.usage_count, cte.success_rate, cte.avg_completion_time_secs, cte.bot_detection_rate, cte.user_satisfaction, cte.effectiveness_score, cte.last_updated
  ORDER BY cte.effectiveness_score DESC;


--
-- Name: captcha_user_experience_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_user_experience_metrics (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    average_completion_time_ms bigint NOT NULL,
    abandonment_rate double precision NOT NULL,
    retry_rate double precision NOT NULL,
    accessibility_usage_rate double precision NOT NULL,
    user_satisfaction_score double precision NOT NULL,
    challenge_type_preferences jsonb NOT NULL,
    difficulty_distribution jsonb NOT NULL,
    CONSTRAINT captcha_user_experience_metrics_abandonment_rate_check CHECK (((abandonment_rate >= (0.0)::double precision) AND (abandonment_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_user_experience_metrics_accessibility_usage_rate_check CHECK (((accessibility_usage_rate >= (0.0)::double precision) AND (accessibility_usage_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_user_experience_metrics_retry_rate_check CHECK (((retry_rate >= (0.0)::double precision) AND (retry_rate <= (1.0)::double precision))),
    CONSTRAINT captcha_user_experience_metrics_user_satisfaction_score_check CHECK (((user_satisfaction_score >= (0.0)::double precision) AND (user_satisfaction_score <= (1.0)::double precision)))
);


--
-- Name: captcha_user_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.captcha_user_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id character varying(255) NOT NULL,
    session_id character varying(255),
    total_attempts integer DEFAULT 0 NOT NULL,
    successful_completions integer DEFAULT 0 NOT NULL,
    failed_attempts integer DEFAULT 0 NOT NULL,
    current_difficulty smallint DEFAULT 3 NOT NULL,
    preferred_challenge_types text[],
    avoid_challenge_types text[],
    last_challenge_time timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT captcha_user_history_current_difficulty_check CHECK (((current_difficulty >= 1) AND (current_difficulty <= 10)))
);


--
-- Name: TABLE captcha_user_history; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.captcha_user_history IS 'Stores user challenge history and preferences for personalization';


--
-- Name: client_default_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_default_scopes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    scope_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_default_scopes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_default_scopes IS 'Default scopes automatically granted to clients without consent';


--
-- Name: client_optional_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_optional_scopes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    scope_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_optional_scopes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_optional_scopes IS 'Optional scopes that clients can request (may require user consent)';


--
-- Name: client_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_scopes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(500),
    description text,
    protocol character varying(50) DEFAULT 'openid-connect'::character varying NOT NULL,
    consent_required boolean DEFAULT true NOT NULL,
    display_on_consent_screen boolean DEFAULT true NOT NULL,
    consent_screen_text text,
    include_in_token_scope boolean DEFAULT true NOT NULL,
    gui_order integer DEFAULT 0,
    icon_uri character varying(1000),
    attributes jsonb DEFAULT '{}'::jsonb,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_scopes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_scopes IS 'Reusable OAuth2/OIDC scope definitions with consent metadata';


--
-- Name: COLUMN client_scopes.name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_scopes.name IS 'OAuth2 scope name (e.g., "openid", "read:aset")';


--
-- Name: COLUMN client_scopes.consent_required; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_scopes.consent_required IS 'Whether user must explicitly consent to this scope';


--
-- Name: COLUMN client_scopes.display_on_consent_screen; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_scopes.display_on_consent_screen IS 'Whether to show this scope in consent UI';


--
-- Name: COLUMN client_scopes.include_in_token_scope; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_scopes.include_in_token_scope IS 'Whether to include scope in token scope claim';


--
-- Name: COLUMN client_scopes.attributes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_scopes.attributes IS 'Custom attributes for scope (audience, resources, etc.)';


--
-- Name: oauth2_clients; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_clients (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    client_id character varying(255) NOT NULL,
    client_secret_hash character varying(255) NOT NULL,
    client_name character varying(255) NOT NULL,
    client_type character varying(20) DEFAULT 'confidential'::character varying NOT NULL,
    redirect_uris text[] DEFAULT '{}'::text[] NOT NULL,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    grant_types text[] DEFAULT '{}'::text[] NOT NULL,
    response_types text[] DEFAULT '{}'::text[] NOT NULL,
    token_endpoint_auth_method character varying(50) DEFAULT 'client_secret_basic'::character varying NOT NULL,
    owner_id uuid,
    realm_id uuid,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    token_exchange_enabled boolean DEFAULT false,
    logo_uri text,
    client_uri text,
    policy_uri text,
    tos_uri text,
    jwks_uri text,
    jwks jsonb,
    sector_identifier_uri text,
    subject_type character varying(20) DEFAULT 'public'::character varying,
    id_token_signed_response_alg character varying(10) DEFAULT 'EdDSA'::character varying,
    id_token_encrypted_response_alg character varying(20),
    id_token_encrypted_response_enc character varying(20),
    userinfo_signed_response_alg character varying(20),
    userinfo_encrypted_response_alg character varying(20),
    userinfo_encrypted_response_enc character varying(20),
    request_object_signing_alg character varying(20),
    request_object_encryption_alg character varying(20),
    request_object_encryption_enc character varying(20),
    token_endpoint_auth_signing_alg character varying(20),
    default_max_age integer,
    require_auth_time boolean DEFAULT false,
    default_acr_values text[],
    initiate_login_uri text,
    request_uris text[],
    application_type character varying(20) DEFAULT 'web'::character varying,
    contacts text[],
    client_id_issued_at timestamp with time zone,
    client_secret_expires_at timestamp with time zone,
    software_id text,
    software_version text,
    registration_access_token_hash text,
    CONSTRAINT oauth2_clients_client_type_check CHECK (((client_type)::text = ANY ((ARRAY['confidential'::character varying, 'public'::character varying])::text[])))
);


--
-- Name: client_all_scopes; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.client_all_scopes AS
 SELECT c.id AS client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name,
    cs.description,
    cs.consent_required,
    cs.display_on_consent_screen,
    'default'::text AS scope_type
   FROM ((authenc.oauth2_clients c
     JOIN authenc.client_default_scopes cds ON ((c.id = cds.client_id)))
     JOIN authenc.client_scopes cs ON ((cds.scope_id = cs.id)))
  WHERE (cs.enabled = true)
UNION ALL
 SELECT c.id AS client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name,
    cs.description,
    cs.consent_required,
    cs.display_on_consent_screen,
    'optional'::text AS scope_type
   FROM ((authenc.oauth2_clients c
     JOIN authenc.client_optional_scopes cos ON ((c.id = cos.client_id)))
     JOIN authenc.client_scopes cs ON ((cos.scope_id = cs.id)))
  WHERE (cs.enabled = true);


--
-- Name: client_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    conditions text[] DEFAULT '{}'::text[] NOT NULL,
    condition_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    executors text[] DEFAULT '{}'::text[] NOT NULL,
    executor_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    policy_type character varying(100) DEFAULT 'custom'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid,
    CONSTRAINT client_policies_policy_type_check CHECK (((policy_type)::text = ANY ((ARRAY['security'::character varying, 'compliance'::character varying, 'fapi-baseline'::character varying, 'fapi-advanced'::character varying, 'custom'::character varying])::text[]))),
    CONSTRAINT client_policies_priority_check CHECK ((priority >= 0))
);


--
-- Name: TABLE client_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_policies IS 'Client security policies for OAuth2/OIDC enforcement';


--
-- Name: COLUMN client_policies.conditions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policies.conditions IS 'Array of condition identifiers that must be met';


--
-- Name: COLUMN client_policies.condition_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policies.condition_config IS 'JSON configuration for conditions';


--
-- Name: COLUMN client_policies.executors; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policies.executors IS 'Array of executor identifiers to run';


--
-- Name: COLUMN client_policies.executor_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policies.executor_config IS 'JSON configuration for executors';


--
-- Name: COLUMN client_policies.priority; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policies.priority IS 'Higher priority policies execute first';


--
-- Name: client_policy_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_policy_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    policy_id uuid,
    profile_id uuid,
    assignment_type character varying(50) NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    priority_override integer,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    assigned_by uuid,
    CONSTRAINT client_policy_assignments_assignment_type_check CHECK (((assignment_type)::text = ANY ((ARRAY['direct'::character varying, 'profile'::character varying])::text[]))),
    CONSTRAINT client_policy_assignments_check CHECK (((((assignment_type)::text = 'direct'::text) AND (policy_id IS NOT NULL) AND (profile_id IS NULL)) OR (((assignment_type)::text = 'profile'::text) AND (profile_id IS NOT NULL) AND (policy_id IS NULL))))
);


--
-- Name: TABLE client_policy_assignments; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_policy_assignments IS 'Associates policies and profiles with clients';


--
-- Name: COLUMN client_policy_assignments.assignment_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policy_assignments.assignment_type IS 'Either "direct" for policy or "profile" for profile assignment';


--
-- Name: COLUMN client_policy_assignments.priority_override; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_policy_assignments.priority_override IS 'Overrides policy priority for this specific assignment';


--
-- Name: client_profiles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_profiles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    policy_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    profile_type character varying(100) DEFAULT 'custom'::character varying NOT NULL,
    is_builtin boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid,
    CONSTRAINT client_profiles_profile_type_check CHECK (((profile_type)::text = ANY ((ARRAY['fapi-1-baseline'::character varying, 'fapi-1-advanced'::character varying, 'fapi-2-security'::character varying, 'fapi-2-message-signing'::character varying, 'custom'::character varying])::text[])))
);


--
-- Name: TABLE client_profiles; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_profiles IS 'Reusable collections of client policies';


--
-- Name: COLUMN client_profiles.policy_ids; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_profiles.policy_ids IS 'Array of policy UUIDs included in this profile';


--
-- Name: COLUMN client_profiles.is_builtin; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.client_profiles.is_builtin IS 'Built-in profiles cannot be deleted';


--
-- Name: client_registration_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_registration_audit_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    event_type character varying(50) NOT NULL,
    client_id uuid,
    client_identifier text,
    realm_id uuid,
    ip_address inet,
    user_agent text,
    initial_access_token_id uuid,
    registration_access_token_id uuid,
    success boolean NOT NULL,
    error_code character varying(50),
    error_description text,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_registration_audit_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_registration_audit_log IS 'Audit trail for all client registration operations';


--
-- Name: client_registration_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_registration_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name text NOT NULL,
    allow_dynamic_registration boolean DEFAULT true,
    require_initial_access_token boolean DEFAULT false,
    require_software_statement boolean DEFAULT false,
    allowed_redirect_uri_patterns text[],
    blocked_redirect_uri_patterns text[],
    max_redirect_uris integer DEFAULT 10,
    allowed_scopes text[],
    default_scopes text[],
    allowed_grant_types text[],
    allowed_response_types text[],
    require_https_redirect_uris boolean DEFAULT true,
    allow_localhost_redirect boolean DEFAULT false,
    client_secret_expires_in integer,
    registration_token_expires_in integer DEFAULT 31536000,
    enabled boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_registration_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_registration_policies IS 'Policies controlling dynamic client registration per realm';


--
-- Name: client_registration_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_registration_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    token_hash text NOT NULL,
    client_id uuid NOT NULL,
    realm_id uuid,
    expires_at timestamp with time zone,
    revoked boolean DEFAULT false,
    revoked_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone
);


--
-- Name: TABLE client_registration_tokens; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_registration_tokens IS 'RFC 7592: Registration access tokens for managing dynamically registered OAuth2 clients';


--
-- Name: client_scope_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.client_scope_mappings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    scope_id uuid NOT NULL,
    protocol_mapper_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE client_scope_mappings; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.client_scope_mappings IS 'Maps scopes to protocol mappers for claim generation';


--
-- Name: event_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.event_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    event_type character varying(100) NOT NULL,
    event_category character varying(50) NOT NULL,
    resource_type character varying(100),
    resource_id character varying(255),
    resource_name character varying(255),
    user_id uuid,
    username character varying(255),
    event_data jsonb,
    old_value jsonb,
    new_value jsonb,
    ip_address inet,
    user_agent text,
    session_id uuid,
    success boolean DEFAULT true NOT NULL,
    error_message text,
    operation_id uuid,
    correlation_id uuid,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    geolocation_data jsonb,
    request_payload jsonb,
    response_payload jsonb
);


--
-- Name: TABLE event_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.event_log IS 'Comprehensive audit log with enhanced context including IP, user agent, geolocation, and sanitized payloads';


--
-- Name: COLUMN event_log.geolocation_data; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.event_log.geolocation_data IS 'Geolocation data for the IP address (country, city, coordinates)';


--
-- Name: COLUMN event_log.request_payload; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.event_log.request_payload IS 'Sanitized request payload (PII removed)';


--
-- Name: COLUMN event_log.response_payload; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.event_log.response_payload IS 'Sanitized response payload (PII removed)';


--
-- Name: comprehensive_audit_trail; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.comprehensive_audit_trail AS
 SELECT event_log.id,
    event_log.created_at,
    event_log.event_type,
    event_log.event_category,
    event_log.resource_type,
    event_log.resource_id,
    event_log.user_id,
    event_log.username,
    event_log.ip_address,
    event_log.user_agent,
    event_log.session_id,
    event_log.correlation_id,
    event_log.geolocation_data,
    event_log.request_payload,
    event_log.response_payload,
    event_log.success,
    event_log.error_message,
    'event_log'::text AS source_table
   FROM authenc.event_log
UNION ALL
 SELECT admin_audit_log.id,
    admin_audit_log.created_at,
    admin_audit_log.operation_type AS event_type,
    'ADMIN'::character varying AS event_category,
    admin_audit_log.resource_type,
    admin_audit_log.resource_id,
    admin_audit_log.admin_user_id AS user_id,
    admin_audit_log.admin_username AS username,
    admin_audit_log.admin_ip_address AS ip_address,
    NULL::text AS user_agent,
    NULL::uuid AS session_id,
    NULL::uuid AS correlation_id,
    admin_audit_log.geolocation_data,
    admin_audit_log.request_payload,
    admin_audit_log.response_payload,
    (admin_audit_log.error_message IS NULL) AS success,
    admin_audit_log.error_message,
    'admin_audit_log'::text AS source_table
   FROM authenc.admin_audit_log
  ORDER BY 2 DESC;


--
-- Name: VIEW comprehensive_audit_trail; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW authenc.comprehensive_audit_trail IS 'Unified view of all audit events with enhanced context from both user and admin logs';


--
-- Name: credential_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.credential_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    is_primary boolean DEFAULT false NOT NULL,
    requires_verification boolean DEFAULT true NOT NULL,
    config_schema jsonb DEFAULT '{}'::jsonb NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE credential_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.credential_types IS 'Dynamic credential type registry - replaces CredentialType enum';


--
-- Name: COLUMN credential_types.config_schema; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.credential_types.config_schema IS 'JSON schema for credential configuration';


--
-- Name: custom_themes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.custom_themes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    theme_type character varying(50) NOT NULL,
    parent_theme character varying(255),
    css_content text,
    css_variables jsonb,
    templates jsonb,
    resources jsonb,
    messages jsonb,
    description text,
    version character varying(50),
    author character varying(255),
    is_active boolean DEFAULT false NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: device_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.device_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    device_id uuid,
    user_id uuid NOT NULL,
    user_session_id uuid,
    session_identifier character varying(255) NOT NULL,
    started_at timestamp without time zone DEFAULT now() NOT NULL,
    last_activity timestamp without time zone DEFAULT now() NOT NULL,
    ip_address inet,
    location jsonb,
    risk_score double precision DEFAULT 0.0,
    risk_factors jsonb,
    is_active boolean DEFAULT true NOT NULL,
    ended_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: device_trust_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.device_trust_history (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    device_id uuid NOT NULL,
    previous_score numeric(3,2),
    new_score numeric(3,2) NOT NULL,
    factors jsonb NOT NULL,
    changed_by uuid,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: devices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.devices (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    device_name character varying(255),
    device_fingerprint text NOT NULL,
    trust_score double precision DEFAULT 0.5 NOT NULL,
    risk_level character varying(20) DEFAULT 'medium'::character varying NOT NULL,
    os character varying(100),
    os_version character varying(100),
    browser character varying(100),
    browser_version character varying(100),
    ip_address inet,
    user_agent text,
    location_data jsonb,
    last_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    first_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT devices_risk_level_check CHECK (((risk_level)::text = ANY ((ARRAY['low'::character varying, 'medium'::character varying, 'high'::character varying, 'critical'::character varying])::text[]))),
    CONSTRAINT devices_trust_score_check CHECK (((trust_score >= (0)::double precision) AND (trust_score <= (1)::double precision)))
);


--
-- Name: event_listener_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.event_listener_executions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    event_log_id uuid NOT NULL,
    listener_id uuid NOT NULL,
    executed_at timestamp without time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    error_message text,
    duration_ms integer,
    retry_count integer DEFAULT 0,
    next_retry_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: event_listeners; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.event_listeners (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    listener_type character varying(100) NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    config jsonb,
    event_types text[],
    priority integer DEFAULT 100 NOT NULL,
    is_async boolean DEFAULT true NOT NULL,
    retry_on_failure boolean DEFAULT false NOT NULL,
    max_retries integer DEFAULT 3,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: event_webhooks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.event_webhooks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    listener_id uuid NOT NULL,
    realm_id uuid NOT NULL,
    url text NOT NULL,
    http_method character varying(10) DEFAULT 'POST'::character varying NOT NULL,
    auth_type character varying(50),
    auth_credentials jsonb,
    custom_headers jsonb,
    payload_template text,
    secret_key character varying(255),
    verify_ssl boolean DEFAULT true NOT NULL,
    timeout_seconds integer DEFAULT 30,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.events (
    id character varying(36) NOT NULL,
    "time" timestamp with time zone DEFAULT now() NOT NULL,
    event_type character varying(100) NOT NULL,
    realm_id character varying(36) NOT NULL,
    realm_name character varying(255),
    client_id character varying(36),
    user_id character varying(36),
    session_id character varying(36),
    ip_address inet,
    error text,
    details jsonb,
    signature character varying(128),
    user_agent text
);


--
-- Name: COLUMN events.user_agent; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.events.user_agent IS 'User agent string from the client';


--
-- Name: federated_auth_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.federated_auth_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    realm_id uuid NOT NULL,
    identity_provider_alias character varying(255) NOT NULL,
    federated_user_id character varying(255),
    success boolean NOT NULL,
    error_code character varying(100),
    error_message text,
    action character varying(50) NOT NULL,
    ip_address inet,
    user_agent text,
    session_id uuid,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: federated_identities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.federated_identities (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    identity_provider_id uuid NOT NULL,
    external_id character varying(255) NOT NULL,
    external_username character varying(255),
    external_email character varying(255),
    external_attributes jsonb,
    last_login_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: federated_identity_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.federated_identity_links (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    realm_id uuid NOT NULL,
    identity_provider_alias character varying(255) NOT NULL,
    federated_user_id character varying(255) NOT NULL,
    federated_username character varying(255),
    token text,
    token_expires_at timestamp without time zone,
    refresh_token text,
    federated_attributes jsonb,
    linked_at timestamp without time zone DEFAULT now() NOT NULL,
    last_authenticated_at timestamp without time zone,
    authentication_count integer DEFAULT 0 NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: group_attributes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.group_attributes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    group_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    value text,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: group_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.group_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    group_id uuid NOT NULL,
    role_id uuid NOT NULL,
    assigned_at timestamp without time zone DEFAULT now() NOT NULL,
    assigned_by uuid
);


--
-- Name: groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    parent_id uuid,
    name character varying(255) NOT NULL,
    path text NOT NULL,
    description text,
    attributes jsonb DEFAULT '{}'::jsonb,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: identity_broker_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.identity_broker_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    alias character varying(255) NOT NULL,
    display_name character varying(255),
    enabled boolean DEFAULT true NOT NULL,
    provider_type character varying(100) NOT NULL,
    first_broker_login_flow character varying(255),
    post_broker_login_flow character varying(255),
    trust_email boolean DEFAULT false NOT NULL,
    store_token boolean DEFAULT false NOT NULL,
    add_read_token_role_on_create boolean DEFAULT false NOT NULL,
    link_only boolean DEFAULT false NOT NULL,
    config jsonb NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: identity_provider_mappers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.identity_provider_mappers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    identity_provider_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    mapper_type character varying(50) NOT NULL,
    config jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    identity_provider_alias character varying(255),
    sync_mode character varying(50) DEFAULT 'IMPORT'::character varying NOT NULL,
    realm_id uuid
);


--
-- Name: identity_providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.identity_providers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255) NOT NULL,
    provider_type character varying(50) NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid NOT NULL,
    config jsonb DEFAULT '{}'::jsonb NOT NULL,
    truststore_path text,
    keystore_path text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT identity_providers_provider_type_check CHECK (((provider_type)::text = ANY ((ARRAY['SAML'::character varying, 'OIDC'::character varying, 'OAuth2'::character varying, 'LDAP'::character varying, 'Kerberos'::character varying, 'SocialLogin'::character varying, 'Custom'::character varying])::text[])))
);


--
-- Name: initial_access_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.initial_access_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    token_hash text NOT NULL,
    realm_id uuid,
    count integer DEFAULT 1 NOT NULL,
    remaining_count integer DEFAULT 1 NOT NULL,
    expires_at timestamp with time zone,
    revoked boolean DEFAULT false,
    revoked_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid,
    last_used_at timestamp with time zone
);


--
-- Name: TABLE initial_access_tokens; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.initial_access_tokens IS 'RFC 7591: Initial access tokens to protect client registration endpoint';


--
-- Name: key_rotation_audit; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.key_rotation_audit (
    id uuid NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    key_id character varying(255) NOT NULL,
    key_type character varying(50) NOT NULL,
    old_version integer NOT NULL,
    new_version integer NOT NULL,
    status character varying(20) NOT NULL,
    error_message text,
    initiated_by character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE key_rotation_audit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.key_rotation_audit IS 'Audit log for automatic key rotation events';


--
-- Name: COLUMN key_rotation_audit.id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.id IS 'Unique identifier for the rotation event';


--
-- Name: COLUMN key_rotation_audit."timestamp"; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit."timestamp" IS 'When the rotation occurred';


--
-- Name: COLUMN key_rotation_audit.key_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.key_id IS 'Identifier of the key that was rotated';


--
-- Name: COLUMN key_rotation_audit.key_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.key_type IS 'Type of key (jwt_signing, session_encryption, mfa_encryption, etc.)';


--
-- Name: COLUMN key_rotation_audit.old_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.old_version IS 'Previous version number of the key';


--
-- Name: COLUMN key_rotation_audit.new_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.new_version IS 'New version number after rotation';


--
-- Name: COLUMN key_rotation_audit.status; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.status IS 'Status of the rotation (Success, Failed, InProgress, Scheduled)';


--
-- Name: COLUMN key_rotation_audit.error_message; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.error_message IS 'Error message if rotation failed';


--
-- Name: COLUMN key_rotation_audit.initiated_by; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.key_rotation_audit.initiated_by IS 'User or system that initiated the rotation';


--
-- Name: mfa_admin_actions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.mfa_admin_actions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    admin_user_id uuid,
    target_user_id uuid NOT NULL,
    action character varying(50) NOT NULL,
    actor_type character varying(50) DEFAULT 'user'::character varying NOT NULL,
    reason text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    metadata jsonb,
    CONSTRAINT check_admin_user_accountability CHECK (((((actor_type)::text = 'user'::text) AND (admin_user_id IS NOT NULL)) OR ((actor_type)::text <> 'user'::text))),
    CONSTRAINT mfa_admin_actions_actor_type_check CHECK (((actor_type)::text = ANY ((ARRAY['user'::character varying, 'system'::character varying, 'automated'::character varying])::text[])))
);


--
-- Name: TABLE mfa_admin_actions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.mfa_admin_actions IS 'Audit log for all MFA administrative actions';


--
-- Name: COLUMN mfa_admin_actions.admin_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.mfa_admin_actions.admin_user_id IS 'ID of the administrator performing the action (NULL for system actions)';


--
-- Name: COLUMN mfa_admin_actions.target_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.mfa_admin_actions.target_user_id IS 'ID of the user being affected by the action';


--
-- Name: COLUMN mfa_admin_actions.action; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.mfa_admin_actions.action IS 'Type of action performed (account_unlock, mfa_reset, etc.)';


--
-- Name: COLUMN mfa_admin_actions.reason; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.mfa_admin_actions.reason IS 'Reason provided by the administrator for the action';


--
-- Name: mfa_devices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.mfa_devices (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    device_type character varying(50) NOT NULL,
    device_name character varying(255),
    secret_key_encrypted text,
    is_primary boolean DEFAULT false NOT NULL,
    is_verified boolean DEFAULT false NOT NULL,
    last_used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: mfa_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.mfa_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_data jsonb NOT NULL,
    created_by uuid,
    active boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE mfa_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.mfa_policies IS 'Stores MFA policy configurations for the organization';


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.users (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    username character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255),
    email_verified boolean DEFAULT false NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid,
    federated boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    last_login_at timestamp with time zone,
    login_count integer DEFAULT 0 NOT NULL,
    mfa_enabled boolean DEFAULT false NOT NULL,
    mfa_setup_at timestamp with time zone,
    mfa_last_used timestamp with time zone,
    satker_code character varying(50) DEFAULT 'UNKNOWN'::character varying NOT NULL,
    require_mfa_setup boolean DEFAULT false NOT NULL,
    password_changed_at timestamp with time zone,
    password_expires_at timestamp with time zone,
    require_password_change boolean DEFAULT false NOT NULL,
    password_history_count integer DEFAULT 0 NOT NULL,
    first_name character varying(255),
    last_name character varying(255),
    nip character varying(50),
    nama character varying(255),
    jabatan character varying(255),
    phone_number character varying(50),
    phone_verified boolean DEFAULT false NOT NULL,
    organization_id uuid,
    totp_secret character varying(255),
    totp_backup_codes text[],
    webauthn_enabled boolean DEFAULT false NOT NULL,
    account_locked boolean DEFAULT false NOT NULL,
    account_locked_until timestamp with time zone,
    failed_login_attempts integer DEFAULT 0 NOT NULL,
    last_failed_login_at timestamp with time zone,
    attributes jsonb
);


--
-- Name: COLUMN users.mfa_enabled; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.mfa_enabled IS 'Whether multi-factor authentication is enabled for this user';


--
-- Name: COLUMN users.mfa_setup_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.mfa_setup_at IS 'Timestamp when MFA was first set up for this user';


--
-- Name: COLUMN users.mfa_last_used; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.mfa_last_used IS 'Timestamp when MFA was last used for authentication';


--
-- Name: COLUMN users.password_changed_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.password_changed_at IS 'Timestamp when the password was last changed';


--
-- Name: COLUMN users.password_expires_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.password_expires_at IS 'Timestamp when the password will expire';


--
-- Name: COLUMN users.require_password_change; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.require_password_change IS 'Flag indicating user must change password on next login';


--
-- Name: COLUMN users.password_history_count; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.users.password_history_count IS 'Number of password changes for this user';


--
-- Name: mfa_statistics; Type: MATERIALIZED VIEW; Schema: public; Owner: -
--

CREATE MATERIALIZED VIEW authenc.mfa_statistics AS
 SELECT u.satker_code,
    count(*) AS total_users,
    count(*) FILTER (WHERE (u.mfa_enabled = true)) AS mfa_enabled_users,
    count(*) FILTER (WHERE ((u.mfa_enabled = true) AND (u.mfa_setup_at IS NOT NULL))) AS mfa_setup_complete,
    count(*) FILTER (WHERE ((u.mfa_enabled = true) AND (u.mfa_last_used >= (now() - '30 days'::interval)))) AS mfa_active_30d,
    count(*) FILTER (WHERE ((u.mfa_enabled = true) AND (u.mfa_last_used >= (now() - '7 days'::interval)))) AS mfa_active_7d,
    count(*) FILTER (WHERE ((u.mfa_enabled = true) AND (u.mfa_last_used >= (now() - '1 day'::interval)))) AS mfa_active_1d,
    max(u.updated_at) AS last_updated
   FROM authenc.users u
  WHERE ((u.enabled = true) AND (u.deleted_at IS NULL))
  GROUP BY u.satker_code
  WITH NO DATA;


--
-- Name: MATERIALIZED VIEW mfa_statistics; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON MATERIALIZED VIEW authenc.mfa_statistics IS 'Aggregated MFA statistics by satker for reporting';


--
-- Name: oauth2_access_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_access_tokens (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    token_hash character varying(255) NOT NULL,
    refresh_token_hash character varying(255),
    client_id uuid NOT NULL,
    user_id uuid,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    refresh_expires_at timestamp with time zone,
    revoked boolean DEFAULT false NOT NULL,
    revoked_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    actor_id uuid,
    delegation_enabled boolean DEFAULT false,
    audience character varying(500),
    resource character varying(500),
    token_exchange_source character varying(255)
);


--
-- Name: oauth2_authorization_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_authorization_codes (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(255) NOT NULL,
    client_id uuid NOT NULL,
    user_id uuid NOT NULL,
    redirect_uri text NOT NULL,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    code_challenge text,
    code_challenge_method character varying(10),
    expires_at timestamp with time zone NOT NULL,
    used boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT oauth2_authorization_codes_code_challenge_method_check CHECK (((code_challenge_method)::text = ANY ((ARRAY['plain'::character varying, 'S256'::character varying])::text[])))
);


--
-- Name: oauth2_provider_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_provider_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    provider_name character varying(100) NOT NULL,
    alias character varying(100) NOT NULL,
    display_name character varying(255),
    authorization_url text NOT NULL,
    token_url text NOT NULL,
    user_info_url text,
    jwks_url text,
    issuer text,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    scopes text DEFAULT 'openid email profile'::text NOT NULL,
    response_type character varying(50) DEFAULT 'code'::character varying,
    response_mode character varying(50) DEFAULT 'query'::character varying,
    grant_type character varying(50) DEFAULT 'authorization_code'::character varying,
    pkce_enabled boolean DEFAULT true NOT NULL,
    pkce_method character varying(10) DEFAULT 'S256'::character varying,
    additional_parameters jsonb,
    user_info_mapping jsonb,
    trust_email boolean DEFAULT false NOT NULL,
    link_only boolean DEFAULT false NOT NULL,
    store_tokens boolean DEFAULT true NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: oauth2_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_states (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    state_token character varying(255) NOT NULL,
    provider_config_id uuid NOT NULL,
    code_verifier character varying(128),
    code_challenge character varying(128),
    code_challenge_method character varying(10),
    redirect_uri text NOT NULL,
    nonce character varying(255),
    realm_id uuid NOT NULL,
    client_session_id uuid,
    ip_address inet,
    user_agent text,
    expires_at timestamp without time zone NOT NULL,
    used boolean DEFAULT false NOT NULL,
    used_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: oauth2_token_exchanges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.oauth2_token_exchanges (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    provider_config_id uuid NOT NULL,
    user_id uuid,
    authorization_code text,
    access_token_hash character varying(64),
    refresh_token_hash character varying(64),
    token_type character varying(50),
    expires_in integer,
    scope text,
    provider_user_id text,
    provider_email text,
    provider_name text,
    user_info_raw jsonb,
    success boolean NOT NULL,
    error_message text,
    ip_address inet,
    user_agent text,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: offline_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.offline_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    realm_id uuid NOT NULL,
    client_id uuid NOT NULL,
    token_hash character varying(64) NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    expires_at timestamp without time zone,
    last_used_at timestamp without time zone,
    scope text,
    data jsonb,
    revoked boolean DEFAULT false NOT NULL,
    revoked_at timestamp without time zone,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: organization_domains; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.organization_domains (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    organization_id uuid NOT NULL,
    domain character varying(255) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    verification_token character varying(255),
    verification_method character varying(50) DEFAULT 'dns'::character varying NOT NULL,
    verified_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: organization_identity_providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.organization_identity_providers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    organization_id uuid NOT NULL,
    identity_provider_id uuid NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: organization_invitations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.organization_invitations (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    organization_id uuid NOT NULL,
    email character varying(255) NOT NULL,
    role character varying(50) DEFAULT 'member'::character varying NOT NULL,
    invited_by uuid NOT NULL,
    token_hash character varying(255) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    accepted_at timestamp with time zone,
    accepted_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT organization_invitations_role_check CHECK (((role)::text = ANY ((ARRAY['owner'::character varying, 'admin'::character varying, 'member'::character varying])::text[])))
);


--
-- Name: organization_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.organization_members (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    organization_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role character varying(50) DEFAULT 'member'::character varying NOT NULL,
    invited_by uuid,
    invited_at timestamp with time zone,
    joined_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    role_type_id uuid
);


--
-- Name: organizations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.organizations (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    description text,
    domain character varying(255),
    logo_url text,
    website_url text,
    owner_id uuid NOT NULL,
    realm_id uuid,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: password_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.password_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    password_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE password_history; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.password_history IS 'Stores password history for users to prevent password reuse';


--
-- Name: COLUMN password_history.user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.password_history.user_id IS 'Reference to the user who owns this password history entry';


--
-- Name: COLUMN password_history.password_hash; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.password_history.password_hash IS 'Argon2 hash of the previous password';


--
-- Name: COLUMN password_history.created_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.password_history.created_at IS 'Timestamp when this password was changed';


--
-- Name: permission_tickets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.permission_tickets (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    resource_id uuid,
    scope_id uuid,
    owner character varying(255) NOT NULL,
    requester character varying(255) NOT NULL,
    granted boolean DEFAULT false NOT NULL,
    granted_timestamp timestamp with time zone,
    realm_id uuid,
    resource_server_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: protocol_mappers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.protocol_mappers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid,
    realm_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    protocol character varying(50) NOT NULL,
    mapper_type character varying(100) NOT NULL,
    config jsonb NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    client_scope_id uuid
);


--
-- Name: realm_theme_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.realm_theme_settings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    realm_id uuid NOT NULL,
    login_theme_id uuid,
    account_theme_id uuid,
    admin_theme_id uuid,
    email_theme_id uuid,
    use_default_on_error boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: realms; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.realms (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    description text,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    ssl_required character varying(50) DEFAULT 'external'::character varying NOT NULL,
    registration_allowed boolean DEFAULT false NOT NULL,
    registration_email_as_username boolean DEFAULT false NOT NULL,
    remember_me boolean DEFAULT true NOT NULL,
    verify_email boolean DEFAULT false NOT NULL,
    login_with_email_allowed boolean DEFAULT true NOT NULL,
    duplicate_emails_allowed boolean DEFAULT false NOT NULL,
    reset_password_allowed boolean DEFAULT true NOT NULL,
    edit_username_allowed boolean DEFAULT false NOT NULL,
    brute_force_protected boolean DEFAULT true NOT NULL,
    max_failure_wait_seconds integer DEFAULT 900 NOT NULL,
    minimum_quick_login_wait_seconds integer DEFAULT 60 NOT NULL,
    wait_increment_seconds integer DEFAULT 60 NOT NULL,
    quick_login_check_milli_seconds bigint DEFAULT 1000 NOT NULL,
    max_delta_time_seconds integer DEFAULT 43200 NOT NULL,
    failure_factor integer DEFAULT 30 NOT NULL,
    default_signature_algorithm character varying(50) DEFAULT 'RS256'::character varying NOT NULL,
    revoke_refresh_token boolean DEFAULT false NOT NULL,
    refresh_token_max_reuse integer DEFAULT 0 NOT NULL,
    access_token_lifespan integer DEFAULT 300 NOT NULL,
    access_token_lifespan_for_implicit_flow integer DEFAULT 900 NOT NULL,
    sso_session_idle_timeout integer DEFAULT 1800 NOT NULL,
    sso_session_max_lifespan integer DEFAULT 36000 NOT NULL,
    sso_session_idle_timeout_remember_me integer DEFAULT 0 NOT NULL,
    sso_session_max_lifespan_remember_me integer DEFAULT 0 NOT NULL,
    offline_session_idle_timeout integer DEFAULT 2592000 NOT NULL,
    offline_session_max_lifespan integer DEFAULT 5184000 NOT NULL,
    client_session_idle_timeout integer DEFAULT 0 NOT NULL,
    client_session_max_lifespan integer DEFAULT 0 NOT NULL,
    access_code_lifespan integer DEFAULT 60 NOT NULL,
    access_code_lifespan_user_action integer DEFAULT 300 NOT NULL,
    access_code_lifespan_login integer DEFAULT 1800 NOT NULL,
    action_token_generated_by_admin_lifespan integer DEFAULT 43200 NOT NULL,
    action_token_generated_by_user_lifespan integer DEFAULT 300 NOT NULL,
    oauth2_device_code_lifespan integer DEFAULT 600 NOT NULL,
    oauth2_device_polling_interval integer DEFAULT 5 NOT NULL,
    attributes text
);


--
-- Name: refresh_token_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.refresh_token_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_session_id uuid NOT NULL,
    old_token_hash character varying(64) NOT NULL,
    new_token_hash character varying(64) NOT NULL,
    rotated_at timestamp without time zone DEFAULT now() NOT NULL,
    client_ip inet,
    user_agent text,
    suspicious boolean DEFAULT false NOT NULL,
    reason text
);


--
-- Name: resource_servers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.resource_servers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    client_id character varying(255) NOT NULL,
    name character varying(255),
    description text,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid,
    policy_enforcement_mode character varying(50) DEFAULT 'enforcing'::character varying NOT NULL,
    decision_strategy character varying(50) DEFAULT 'unanimous'::character varying NOT NULL,
    allow_remote_resource_management boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.resources (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    uris text[] DEFAULT '{}'::text[] NOT NULL,
    icon_uri character varying(1000),
    resource_type character varying(255),
    owner character varying(255) NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid,
    resource_server_id uuid,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    attributes jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: role_capabilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.role_capabilities (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    role_id uuid NOT NULL,
    capability_id uuid NOT NULL,
    conditions jsonb DEFAULT '{}'::jsonb NOT NULL,
    granted_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: TABLE role_capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.role_capabilities IS 'Mapping of roles to their granted capabilities';


--
-- Name: COLUMN role_capabilities.conditions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.role_capabilities.conditions IS 'JSON conditions for conditional capability grants';


--
-- Name: role_hierarchy; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.role_hierarchy (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    parent_role_id uuid NOT NULL,
    child_role_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT role_hierarchy_check CHECK ((parent_role_id <> child_role_id))
);


--
-- Name: TABLE role_hierarchy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.role_hierarchy IS 'Role composition - parent roles inherit child role capabilities';


--
-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.role_permissions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    role_id uuid NOT NULL,
    permission character varying(255) NOT NULL,
    resource character varying(255),
    actions text[] DEFAULT '{}'::text[],
    conditions jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: role_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.role_policies (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    role_id uuid NOT NULL,
    policy_id uuid NOT NULL,
    granted_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: TABLE role_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.role_policies IS 'Mapping of roles to authorization policies';


--
-- Name: role_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.role_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    category character varying(100) DEFAULT 'custom'::character varying NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    is_assignable boolean DEFAULT true NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE role_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.role_types IS 'Dynamic role type registry - replaces hardcoded role enums';


--
-- Name: COLUMN role_types.is_system; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.role_types.is_system IS 'System roles cannot be deleted but can be modified';


--
-- Name: COLUMN role_types.is_assignable; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.role_types.is_assignable IS 'Whether this role type can be assigned to users';


--
-- Name: COLUMN role_types.priority; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.role_types.priority IS 'Higher priority roles take precedence in conflicts';


--
-- Name: roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.roles (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    realm_id uuid,
    composite boolean DEFAULT false NOT NULL,
    client_role boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: saml_assertion_cache; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.saml_assertion_cache (
    assertion_id character varying(255) NOT NULL,
    used_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


--
-- Name: TABLE saml_assertion_cache; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.saml_assertion_cache IS 'Cache of used SAML assertion IDs to prevent replay attacks';


--
-- Name: COLUMN saml_assertion_cache.assertion_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.saml_assertion_cache.assertion_id IS 'Unique SAML assertion ID from the AssertionID attribute';


--
-- Name: COLUMN saml_assertion_cache.used_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.saml_assertion_cache.used_at IS 'Timestamp when the assertion was first used';


--
-- Name: COLUMN saml_assertion_cache.expires_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.saml_assertion_cache.expires_at IS 'Timestamp when the cache entry expires (typically 5 minutes after use)';


--
-- Name: saml_identity_providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.saml_identity_providers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    entity_id text NOT NULL,
    metadata_url text,
    metadata_xml text,
    sso_url text NOT NULL,
    slo_url text,
    signing_certificate text NOT NULL,
    encryption_certificate text,
    name_id_format text DEFAULT 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress'::text,
    realm_id uuid,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: saml_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.saml_messages (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    saml_id character varying(255) NOT NULL,
    message_type character varying(50) NOT NULL,
    issuer character varying(500) NOT NULL,
    destination character varying(500) NOT NULL,
    xml_content text NOT NULL,
    signature text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    session_id character varying(255),
    relay_state text
);


--
-- Name: saml_service_providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.saml_service_providers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    entity_id text NOT NULL,
    metadata_url text,
    metadata_xml text,
    signing_certificate text,
    encryption_certificate text,
    assertion_consumer_service_url text NOT NULL,
    single_logout_service_url text,
    name_id_format text DEFAULT 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress'::text,
    realm_id uuid,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: saml_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.saml_sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    session_id text NOT NULL,
    user_id uuid NOT NULL,
    identity_provider_id uuid NOT NULL,
    service_provider_id uuid,
    name_id text NOT NULL,
    name_id_format text NOT NULL,
    session_index text,
    authn_instant timestamp with time zone NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: satker_admin_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.satker_admin_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    satker_code character varying(50) NOT NULL,
    admin_level character varying(50) NOT NULL,
    scope_data jsonb,
    assigned_by uuid,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone,
    active boolean DEFAULT true NOT NULL
);


--
-- Name: TABLE satker_admin_roles; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.satker_admin_roles IS 'Administrative roles scoped to specific satkers';


--
-- Name: COLUMN satker_admin_roles.admin_level; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satker_admin_roles.admin_level IS 'Level of administrative authority (AdminSatker, AdminWilayah, AdminEselonI, AdminPusat)';


--
-- Name: satker_audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.satker_audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    satker_code character varying(50) NOT NULL,
    user_id uuid,
    operation character varying(50) NOT NULL,
    operation_details jsonb,
    ip_address inet,
    user_agent text,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE satker_audit_logs; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.satker_audit_logs IS 'Audit trail for satker-related operations';


--
-- Name: satkers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.satkers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    description text,
    parent_code character varying(50),
    level integer DEFAULT 0 NOT NULL,
    satker_type jsonb DEFAULT '"KejaksaanNegeri"'::jsonb NOT NULL,
    active boolean DEFAULT true NOT NULL,
    attributes jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_code_not_empty CHECK (((code)::text <> ''::text)),
    CONSTRAINT chk_level_non_negative CHECK ((level >= 0)),
    CONSTRAINT chk_name_not_empty CHECK (((name)::text <> ''::text))
);


--
-- Name: TABLE satkers; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.satkers IS 'Organizational units (Satuan Kerja) in the Attorney General''s Office hierarchy';


--
-- Name: COLUMN satkers.code; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satkers.code IS 'Unique identifier code for the satker';


--
-- Name: COLUMN satkers.parent_code; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satkers.parent_code IS 'Code of the parent satker in the hierarchy';


--
-- Name: COLUMN satkers.level; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satkers.level IS 'Depth level in the hierarchy (0 = root)';


--
-- Name: COLUMN satkers.satker_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satkers.satker_type IS 'Type of satker (Pusat, KejaksaanTinggi, KejaksaanNegeri, etc.)';


--
-- Name: satker_hierarchy_view; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.satker_hierarchy_view AS
 WITH RECURSIVE satker_tree AS (
         SELECT satkers.id,
            satkers.code,
            satkers.name,
            satkers.parent_code,
            satkers.level,
            satkers.satker_type,
            satkers.active,
            (satkers.code)::text AS path,
            (satkers.name)::text AS full_path
           FROM authenc.satkers
          WHERE ((satkers.parent_code IS NULL) AND (satkers.active = true))
        UNION ALL
         SELECT s.id,
            s.code,
            s.name,
            s.parent_code,
            s.level,
            s.satker_type,
            s.active,
            ((st.path || ' > '::text) || (s.code)::text) AS path,
            ((st.full_path || ' > '::text) || (s.name)::text) AS full_path
           FROM (authenc.satkers s
             JOIN satker_tree st ON (((s.parent_code)::text = (st.code)::text)))
          WHERE (s.active = true)
        )
 SELECT satker_tree.id,
    satker_tree.code,
    satker_tree.name,
    satker_tree.parent_code,
    satker_tree.level,
    satker_tree.satker_type,
    satker_tree.active,
    satker_tree.path,
    satker_tree.full_path
   FROM satker_tree
  ORDER BY satker_tree.level, satker_tree.code;


--
-- Name: VIEW satker_hierarchy_view; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW authenc.satker_hierarchy_view IS 'Hierarchical view of satkers with full path information';


--
-- Name: satker_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.satker_permissions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    satker_code character varying(50) NOT NULL,
    permission_type character varying(100) NOT NULL,
    resource_type character varying(100),
    action character varying(50),
    include_children boolean DEFAULT false NOT NULL,
    include_parents boolean DEFAULT false NOT NULL,
    granted_by uuid,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone,
    attributes jsonb
);


--
-- Name: TABLE satker_permissions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.satker_permissions IS 'Explicit permissions granted to users for specific satkers';


--
-- Name: COLUMN satker_permissions.include_children; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satker_permissions.include_children IS 'Whether permission extends to child satkers';


--
-- Name: COLUMN satker_permissions.include_parents; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satker_permissions.include_parents IS 'Whether permission extends to parent satkers';


--
-- Name: satker_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.satker_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    hierarchy_level integer DEFAULT 0 NOT NULL,
    parent_type_id uuid,
    code_pattern character varying(100),
    is_system boolean DEFAULT false NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE satker_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.satker_types IS 'Dynamic Satker type hierarchy - replaces SatkerType enum';


--
-- Name: COLUMN satker_types.code_pattern; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.satker_types.code_pattern IS 'Regex pattern for satker codes of this type';


--
-- Name: scope_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.scope_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    hierarchy_level integer DEFAULT 0 NOT NULL,
    parent_scope_type_id uuid,
    scope_pattern character varying(500),
    is_system boolean DEFAULT false NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    realm_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: TABLE scope_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.scope_types IS 'Dynamic scope type registry - replaces RoleScope enum';


--
-- Name: COLUMN scope_types.scope_pattern; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.scope_types.scope_pattern IS 'Regex pattern for matching scope identifiers';


--
-- Name: scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.scopes (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    icon_uri character varying(1000),
    realm_id uuid,
    resource_server_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: service_account_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.service_account_audit_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    service_account_id uuid NOT NULL,
    event_type character varying(50) NOT NULL,
    event_details jsonb DEFAULT '{}'::jsonb NOT NULL,
    performed_by uuid,
    ip_address inet,
    user_agent text,
    success boolean DEFAULT true NOT NULL,
    error_message text,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_service_account_audit_event_type CHECK (((event_type)::text = ANY ((ARRAY['created'::character varying, 'updated'::character varying, 'deleted'::character varying, 'enabled'::character varying, 'disabled'::character varying, 'authenticated'::character varying, 'auth_failed'::character varying, 'secret_regenerated'::character varying, 'role_granted'::character varying, 'role_revoked'::character varying])::text[])))
);


--
-- Name: TABLE service_account_audit_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.service_account_audit_log IS 'Comprehensive audit trail for all service account operations';


--
-- Name: service_account_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.service_account_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    service_account_id uuid NOT NULL,
    role_id uuid NOT NULL,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    granted_by uuid,
    attributes jsonb DEFAULT '{}'::jsonb
);


--
-- Name: TABLE service_account_roles; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.service_account_roles IS 'Role assignments for service accounts, determines API access permissions';


--
-- Name: service_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.service_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(200) NOT NULL,
    description text,
    client_id character varying(255) NOT NULL,
    client_secret_hash text NOT NULL,
    realm_id uuid NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    attributes jsonb DEFAULT '{}'::jsonb,
    CONSTRAINT chk_service_account_client_id_not_empty CHECK (((client_id)::text <> ''::text)),
    CONSTRAINT chk_service_account_name_not_empty CHECK (((name)::text <> ''::text)),
    CONSTRAINT chk_service_account_secret_not_empty CHECK ((client_secret_hash <> ''::text))
);


--
-- Name: TABLE service_accounts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.service_accounts IS 'Service accounts for machine-to-machine authentication using OAuth2 client credentials grant';


--
-- Name: COLUMN service_accounts.client_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.service_accounts.client_id IS 'OAuth2 client identifier, must be unique across all realms';


--
-- Name: COLUMN service_accounts.client_secret_hash; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.service_accounts.client_secret_hash IS 'Bcrypt-hashed client secret (cost factor 12)';


--
-- Name: COLUMN service_accounts.last_used_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.service_accounts.last_used_at IS 'Timestamp of last successful authentication, used for monitoring and security audits';


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    token_hash character varying(255) NOT NULL,
    refresh_token_hash character varying(255),
    ip_address inet,
    user_agent text,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_activity_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: social_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.social_accounts (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    provider character varying(50) NOT NULL,
    provider_user_id character varying(255) NOT NULL,
    display_name character varying(255),
    email character varying(255),
    profile_picture_url text,
    access_token text,
    refresh_token text,
    token_expires_at timestamp with time zone,
    linked_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: social_login_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.social_login_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    oauth2_config_id uuid NOT NULL,
    provider_type character varying(50) NOT NULL,
    google_hosted_domain text,
    google_prompt character varying(50),
    github_allow_signup boolean DEFAULT true,
    github_allowed_organizations text[],
    facebook_fields text,
    facebook_graph_api_version character varying(20) DEFAULT 'v18.0'::character varying,
    microsoft_tenant_id text,
    microsoft_admin_consent boolean DEFAULT false,
    apple_team_id text,
    apple_key_id text,
    apple_private_key text,
    button_text character varying(255),
    button_icon_url text,
    button_class character varying(100),
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: software_statement_issuers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.software_statement_issuers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    issuer text NOT NULL,
    jwks_uri text,
    jwks jsonb,
    realm_id uuid,
    enabled boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE software_statement_issuers; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.software_statement_issuers IS 'RFC 7591: Trusted issuers of software statements (signed JWTs)';


--
-- Name: theme_inheritance; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.theme_inheritance (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    theme_id uuid NOT NULL,
    parent_theme_id uuid,
    inheritance_order integer DEFAULT 0 NOT NULL,
    override_css boolean DEFAULT true NOT NULL,
    override_templates boolean DEFAULT true NOT NULL,
    override_messages boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: theme_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.theme_resources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    theme_id uuid NOT NULL,
    resource_name character varying(255) NOT NULL,
    resource_type character varying(50) NOT NULL,
    mime_type character varying(100),
    content_url text,
    content_data bytea,
    content_size bigint,
    cache_key character varying(255),
    etag character varying(255),
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: theme_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.theme_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    theme_id uuid NOT NULL,
    template_name character varying(255) NOT NULL,
    template_type character varying(50) NOT NULL,
    content text NOT NULL,
    is_valid boolean DEFAULT true NOT NULL,
    validation_errors jsonb,
    version integer DEFAULT 1 NOT NULL,
    previous_version_id uuid,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- Name: theme_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.theme_types (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    default_template text,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE theme_types; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.theme_types IS 'Dynamic theme type registry - replaces ThemeType enum';


--
-- Name: token_exchange_audit; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.token_exchange_audit (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    subject_token_type character varying(255) NOT NULL,
    requested_token_type character varying(255),
    issued_token_type character varying(255) NOT NULL,
    subject_user_id uuid,
    subject_username character varying(255),
    original_client_id uuid,
    actor_id uuid,
    actor_type character varying(50),
    delegation_enabled boolean DEFAULT false,
    target_client_id uuid,
    audience character varying(500),
    resource character varying(500),
    original_scopes text[],
    requested_scopes text[],
    granted_scopes text[],
    issued_token_id uuid,
    expires_in integer,
    success boolean DEFAULT false NOT NULL,
    error_code character varying(100),
    error_description text,
    ip_address inet,
    user_agent text,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE token_exchange_audit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.token_exchange_audit IS 'Audit log for OAuth 2.0 Token Exchange (RFC 8693) operations';


--
-- Name: COLUMN token_exchange_audit.subject_token_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.token_exchange_audit.subject_token_type IS 'URN identifier of the subject token type (e.g., urn:ietf:params:oauth:token-type:access_token)';


--
-- Name: COLUMN token_exchange_audit.delegation_enabled; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.token_exchange_audit.delegation_enabled IS 'True if this was a delegation scenario with actor token';


--
-- Name: COLUMN token_exchange_audit.audience; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.token_exchange_audit.audience IS 'Target audience for the issued token (RFC 8693 audience parameter)';


--
-- Name: COLUMN token_exchange_audit.resource; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.token_exchange_audit.resource IS 'Target resource for the issued token (RFC 8693 resource parameter)';


--
-- Name: uma_permission_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.uma_permission_requests (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    ticket character varying(255) NOT NULL,
    resource_id uuid,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    requester character varying(255) NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT uma_permission_requests_status_check CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'approved'::character varying, 'denied'::character varying, 'expired'::character varying])::text[])))
);


--
-- Name: uma_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.uma_policies (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    policy_type character varying(50) NOT NULL,
    logic character varying(20) DEFAULT 'POSITIVE'::character varying NOT NULL,
    decision_strategy character varying(20) DEFAULT 'UNANIMOUS'::character varying NOT NULL,
    config jsonb DEFAULT '{}'::jsonb NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    realm_id uuid NOT NULL,
    resource_server_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT uma_policies_decision_strategy_check CHECK (((decision_strategy)::text = ANY ((ARRAY['UNANIMOUS'::character varying, 'AFFIRMATIVE'::character varying, 'CONSENSUS'::character varying])::text[]))),
    CONSTRAINT uma_policies_logic_check CHECK (((logic)::text = ANY ((ARRAY['POSITIVE'::character varying, 'NEGATIVE'::character varying])::text[]))),
    CONSTRAINT uma_policies_policy_type_check CHECK (((policy_type)::text = ANY ((ARRAY['role'::character varying, 'user'::character varying, 'group'::character varying, 'time'::character varying, 'attribute'::character varying, 'javascript'::character varying, 'aggregate'::character varying, 'client'::character varying])::text[])))
);


--
-- Name: user_consent_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_consent_scopes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    client_id uuid NOT NULL,
    scope_id uuid NOT NULL,
    granted_at timestamp without time zone DEFAULT now() NOT NULL,
    expires_at timestamp without time zone,
    consent_source character varying(50) DEFAULT 'explicit'::character varying
);


--
-- Name: TABLE user_consent_scopes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.user_consent_scopes IS 'Structured user consent tracking per scope';


--
-- Name: user_active_consents; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.user_active_consents AS
 SELECT ucs.user_id,
    ucs.client_id,
    c.client_id AS client_identifier,
    cs.id AS scope_id,
    cs.name AS scope_name,
    cs.display_name AS scope_display_name,
    ucs.granted_at,
    ucs.expires_at,
    ucs.consent_source
   FROM ((authenc.user_consent_scopes ucs
     JOIN authenc.oauth2_clients c ON ((ucs.client_id = c.id)))
     JOIN authenc.client_scopes cs ON ((ucs.scope_id = cs.id)))
  WHERE ((ucs.expires_at IS NULL) OR (ucs.expires_at > now()));


--
-- Name: user_attributes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_attributes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    value text,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: user_consents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_consents (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    client_id character varying(255) NOT NULL,
    scopes text[] DEFAULT '{}'::text[] NOT NULL,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone,
    metadata jsonb
);


--
-- Name: user_groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    group_id uuid NOT NULL,
    joined_at timestamp without time zone DEFAULT now() NOT NULL,
    expires_at timestamp without time zone,
    attributes jsonb DEFAULT '{}'::jsonb
);


--
-- Name: user_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_policies (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    policy_id uuid NOT NULL,
    granted_by uuid,
    reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: TABLE user_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.user_policies IS 'Direct user-to-policy mappings for explicit grants';


--
-- Name: user_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_roles (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.user_sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    session_id character varying(255) NOT NULL,
    user_id uuid NOT NULL,
    device_id uuid,
    ip_address inet,
    user_agent text,
    location_data jsonb,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    last_accessed timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    terminated boolean DEFAULT false NOT NULL,
    terminated_at timestamp with time zone,
    terminated_reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    realm_id uuid,
    client_id uuid,
    offline_token_hash character varying(64),
    idle_expires_at timestamp without time zone,
    refresh_count integer DEFAULT 0 NOT NULL,
    refresh_token_expires_at timestamp without time zone,
    offline_token_expires_at timestamp without time zone,
    revoked boolean DEFAULT false NOT NULL,
    revoked_at timestamp without time zone,
    revoked_reason text,
    authentication_method character varying(50),
    protocol character varying(20),
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    token_hash character varying(64),
    refresh_token_hash character varying(64)
);


--
-- Name: v_client_policies_with_profiles; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_client_policies_with_profiles AS
SELECT
    NULL::uuid AS id,
    NULL::uuid AS realm_id,
    NULL::character varying(255) AS name,
    NULL::text AS description,
    NULL::boolean AS enabled,
    NULL::text[] AS conditions,
    NULL::jsonb AS condition_config,
    NULL::text[] AS executors,
    NULL::jsonb AS executor_config,
    NULL::integer AS priority,
    NULL::character varying(100) AS policy_type,
    NULL::timestamp with time zone AS created_at,
    NULL::timestamp with time zone AS updated_at,
    NULL::uuid AS created_by,
    NULL::json AS applied_profiles;


--
-- Name: v_client_policy_assignments_detail; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_client_policy_assignments_detail AS
 SELECT cpa.id,
    cpa.client_id,
    cpa.policy_id,
    cpa.profile_id,
    cpa.assignment_type,
    cpa.enabled,
    cpa.priority_override,
    cpa.assigned_at,
    cpa.assigned_by,
        CASE
            WHEN ((cpa.assignment_type)::text = 'direct'::text) THEN json_build_object('type', 'policy', 'id', cp.id, 'name', cp.name, 'description', cp.description, 'priority', COALESCE(cpa.priority_override, cp.priority))
            WHEN ((cpa.assignment_type)::text = 'profile'::text) THEN json_build_object('type', 'profile', 'id', prof.id, 'name', prof.name, 'description', prof.description, 'policy_count', array_length(prof.policy_ids, 1))
            ELSE NULL::json
        END AS assignment_details
   FROM ((authenc.client_policy_assignments cpa
     LEFT JOIN authenc.client_policies cp ON ((cpa.policy_id = cp.id)))
     LEFT JOIN authenc.client_profiles prof ON ((cpa.profile_id = prof.id)));


--
-- Name: v_client_policy_stats; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_client_policy_stats AS
 SELECT client_policies.realm_id,
    client_policies.policy_type,
    count(*) AS total_policies,
    count(*) FILTER (WHERE (client_policies.enabled = true)) AS enabled_policies,
    count(*) FILTER (WHERE (client_policies.enabled = false)) AS disabled_policies,
    avg(client_policies.priority) AS avg_priority
   FROM authenc.client_policies
  GROUP BY client_policies.realm_id, client_policies.policy_type;


--
-- Name: v_client_profile_usage; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_client_profile_usage AS
SELECT
    NULL::uuid AS profile_id,
    NULL::uuid AS realm_id,
    NULL::character varying(255) AS profile_name,
    NULL::character varying(100) AS profile_type,
    NULL::boolean AS is_builtin,
    NULL::bigint AS assigned_clients,
    NULL::integer AS policy_count;


--
-- Name: v_client_profiles_with_policies; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_client_profiles_with_policies AS
SELECT
    NULL::uuid AS id,
    NULL::uuid AS realm_id,
    NULL::character varying(255) AS name,
    NULL::text AS description,
    NULL::boolean AS enabled,
    NULL::uuid[] AS policy_ids,
    NULL::character varying(100) AS profile_type,
    NULL::boolean AS is_builtin,
    NULL::timestamp with time zone AS created_at,
    NULL::timestamp with time zone AS updated_at,
    NULL::uuid AS created_by,
    NULL::json AS policies;


--
-- Name: v_role_hierarchy_capabilities; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_role_hierarchy_capabilities AS
 WITH RECURSIVE role_tree AS (
         SELECT r.id AS role_id,
            r.name AS role_name,
            c.code AS capability_code,
            0 AS depth
           FROM ((authenc.roles r
             JOIN authenc.role_capabilities rc ON ((r.id = rc.role_id)))
             JOIN authenc.capabilities c ON ((rc.capability_id = c.id)))
          WHERE (r.deleted_at IS NULL)
        UNION ALL
         SELECT rh.child_role_id AS role_id,
            cr.name AS role_name,
            rt.capability_code,
            (rt.depth + 1)
           FROM ((authenc.role_hierarchy rh
             JOIN role_tree rt ON ((rh.parent_role_id = rt.role_id)))
             JOIN authenc.roles cr ON ((rh.child_role_id = cr.id)))
          WHERE ((cr.deleted_at IS NULL) AND (rt.depth < 10))
        )
 SELECT DISTINCT role_tree.role_id,
    role_tree.role_name,
    role_tree.capability_code,
    min(role_tree.depth) AS inheritance_depth
   FROM role_tree
  GROUP BY role_tree.role_id, role_tree.role_name, role_tree.capability_code;


--
-- Name: VIEW v_role_hierarchy_capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW authenc.v_role_hierarchy_capabilities IS 'View showing all capabilities for roles including inherited ones';


--
-- Name: v_user_effective_capabilities; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_user_effective_capabilities AS
 SELECT DISTINCT u.id AS user_id,
    u.username,
    c.id AS capability_id,
    c.code AS capability_code,
    c.resource_type,
    c.action,
    r.id AS role_id,
    r.name AS role_name,
    'role'::text AS grant_source
   FROM ((((authenc.users u
     JOIN authenc.user_roles ur ON ((u.id = ur.user_id)))
     JOIN authenc.roles r ON ((ur.role_id = r.id)))
     JOIN authenc.role_capabilities rc ON ((r.id = rc.role_id)))
     JOIN authenc.capabilities c ON ((rc.capability_id = c.id)))
  WHERE ((u.deleted_at IS NULL) AND (r.deleted_at IS NULL) AND (c.deleted_at IS NULL) AND ((rc.expires_at IS NULL) OR (rc.expires_at > now())))
UNION
 SELECT DISTINCT u.id AS user_id,
    u.username,
    c.id AS capability_id,
    c.code AS capability_code,
    c.resource_type,
    c.action,
    NULL::uuid AS role_id,
    NULL::character varying AS role_name,
    'policy'::text AS grant_source
   FROM ((((authenc.users u
     JOIN authenc.user_policies up ON ((u.id = up.user_id)))
     JOIN authenc.authorization_policies ap ON ((up.policy_id = ap.id)))
     CROSS JOIN LATERAL jsonb_array_elements_text(ap.capabilities) cap_code(value))
     JOIN authenc.capabilities c ON (((c.code)::text = cap_code.value)))
  WHERE ((u.deleted_at IS NULL) AND (ap.deleted_at IS NULL) AND (ap.enabled = true) AND (c.deleted_at IS NULL) AND ((up.expires_at IS NULL) OR (up.expires_at > now())));


--
-- Name: VIEW v_user_effective_capabilities; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON VIEW authenc.v_user_effective_capabilities IS 'Consolidated view of all user capabilities from roles and policies';


--
-- Name: v_user_perlengkapan_roles; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW authenc.v_user_perlengkapan_roles AS
 SELECT u.id AS user_id,
    u.username,
    u.email,
    r.name AS role_name,
    r.description AS role_description,
    ua_satker.value AS satker_code,
    ua_satker_name.value AS satker_name,
    ua_active.value AS active_role
   FROM (((((authenc.users u
     JOIN authenc.user_roles ur ON ((u.id = ur.user_id)))
     JOIN authenc.roles r ON ((ur.role_id = r.id)))
     LEFT JOIN authenc.user_attributes ua_satker ON (((u.id = ua_satker.user_id) AND ((ua_satker.name)::text = 'satker_code'::text))))
     LEFT JOIN authenc.user_attributes ua_satker_name ON (((u.id = ua_satker_name.user_id) AND ((ua_satker_name.name)::text = 'satker_name'::text))))
     LEFT JOIN authenc.user_attributes ua_active ON (((u.id = ua_active.user_id) AND ((ua_active.name)::text = 'active_role'::text))))
  WHERE ((r.name)::text = ANY ((ARRAY['operator_satker'::character varying, 'validator_wilayah'::character varying, 'validator_pusat'::character varying, 'admin'::character varying])::text[]));


--
-- Name: webauthn_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.webauthn_audit_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    credential_id bytea,
    event_type character varying(50) NOT NULL,
    event_details jsonb,
    ip_address inet,
    user_agent text,
    success boolean NOT NULL,
    error_message text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: TABLE webauthn_audit_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.webauthn_audit_log IS 'Audit log for all WebAuthn operations (registration, authentication, errors)';


--
-- Name: webauthn_challenges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.webauthn_challenges (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    challenge text NOT NULL,
    challenge_type character varying(20) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used boolean DEFAULT false NOT NULL,
    CONSTRAINT webauthn_challenges_challenge_type_check CHECK (((challenge_type)::text = ANY ((ARRAY['registration'::character varying, 'authentication'::character varying])::text[])))
);


--
-- Name: TABLE webauthn_challenges; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.webauthn_challenges IS 'Stores WebAuthn registration and authentication challenges with TTL';


--
-- Name: webauthn_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.webauthn_credentials (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    cred_id bytea NOT NULL,
    cred jsonb NOT NULL,
    nickname character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_used timestamp with time zone
);


--
-- Name: TABLE webauthn_credentials; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE authenc.webauthn_credentials IS 'Stores WebAuthn/FIDO2 passkey credentials for passwordless authentication';


--
-- Name: COLUMN webauthn_credentials.id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.id IS 'Unique credential ID (database primary key)';


--
-- Name: COLUMN webauthn_credentials.user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.user_id IS 'User ID this credential belongs to';


--
-- Name: COLUMN webauthn_credentials.cred_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.cred_id IS 'WebAuthn credential ID (binary, from authenticator)';


--
-- Name: COLUMN webauthn_credentials.cred; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.cred IS 'Full Passkey object from webauthn-rs (includes public key, counter, etc.)';


--
-- Name: COLUMN webauthn_credentials.nickname; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.nickname IS 'Optional user-assigned nickname (e.g., "My YubiKey", "iPhone Touch ID")';


--
-- Name: COLUMN webauthn_credentials.created_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.created_at IS 'When this credential was registered';


--
-- Name: COLUMN webauthn_credentials.last_used; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials.last_used IS 'When this credential was last used for authentication';


--
-- Name: webauthn_credentials_old; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE authenc.webauthn_credentials_old (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    credential_id text NOT NULL,
    public_key text NOT NULL,
    public_key_algorithm integer NOT NULL,
    signature_counter bigint DEFAULT 0 NOT NULL,
    attestation_object text,
    authenticator_data text,
    user_handle text,
    credential_type character varying(50) DEFAULT 'public-key'::character varying NOT NULL,
    transports text[],
    aaguid uuid,
    attestation_format character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    enabled boolean DEFAULT true NOT NULL,
    attestation_certificates jsonb,
    attestation_statement jsonb,
    authenticator_metadata jsonb,
    backup_eligible boolean DEFAULT false NOT NULL,
    backup_state boolean DEFAULT false NOT NULL,
    attestation_conveyance character varying(20) DEFAULT 'none'::character varying,
    CONSTRAINT webauthn_credentials_attestation_conveyance_check CHECK (((attestation_conveyance)::text = ANY ((ARRAY['none'::character varying, 'indirect'::character varying, 'direct'::character varying, 'enterprise'::character varying])::text[])))
);


--
-- Name: COLUMN webauthn_credentials_old.attestation_certificates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.attestation_certificates IS 'Certificate chain from direct/enterprise attestation (PEM format, JSON array)';


--
-- Name: COLUMN webauthn_credentials_old.attestation_statement; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.attestation_statement IS 'Attestation statement from authenticator (format-specific, JSONB)';


--
-- Name: COLUMN webauthn_credentials_old.authenticator_metadata; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.authenticator_metadata IS 'Metadata from FIDO Metadata Service including manufacturer, model, certification level';


--
-- Name: COLUMN webauthn_credentials_old.backup_eligible; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.backup_eligible IS 'Indicates if the credential can be backed up (multi-device credentials)';


--
-- Name: COLUMN webauthn_credentials_old.backup_state; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.backup_state IS 'Indicates if the credential is currently backed up';


--
-- Name: COLUMN webauthn_credentials_old.attestation_conveyance; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN authenc.webauthn_credentials_old.attestation_conveyance IS 'Attestation conveyance preference used during registration (none/indirect/direct/enterprise)';


--
-- Name: mfa_backup_codes mfa_backup_codes_pkey; Type: CONSTRAINT; Schema: authenc; Owner: -
--

ALTER TABLE ONLY authenc.mfa_backup_codes
    ADD CONSTRAINT mfa_backup_codes_pkey PRIMARY KEY (user_id, code);


--
-- Name: token_revocations token_revocations_pkey; Type: CONSTRAINT; Schema: authenc; Owner: -
--

ALTER TABLE ONLY authenc.token_revocations
    ADD CONSTRAINT token_revocations_pkey PRIMARY KEY (kind, value);


--
-- Name: totp_secrets totp_secrets_pkey; Type: CONSTRAINT; Schema: authenc; Owner: -
--

ALTER TABLE ONLY authenc.totp_secrets
    ADD CONSTRAINT totp_secrets_pkey PRIMARY KEY (user_id);


--
-- Name: access_levels access_levels_code_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.access_levels
    ADD CONSTRAINT access_levels_code_realm_id_key UNIQUE (code, realm_id);


--
-- Name: access_levels access_levels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.access_levels
    ADD CONSTRAINT access_levels_pkey PRIMARY KEY (id);


--
-- Name: account_linking_requests account_linking_requests_confirmation_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.account_linking_requests
    ADD CONSTRAINT account_linking_requests_confirmation_token_key UNIQUE (confirmation_token);


--
-- Name: account_linking_requests account_linking_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.account_linking_requests
    ADD CONSTRAINT account_linking_requests_pkey PRIMARY KEY (id);


--
-- Name: actor_types actor_types_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.actor_types
    ADD CONSTRAINT actor_types_code_key UNIQUE (code);


--
-- Name: actor_types actor_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.actor_types
    ADD CONSTRAINT actor_types_pkey PRIMARY KEY (id);


--
-- Name: admin_audit_log admin_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_audit_log
    ADD CONSTRAINT admin_audit_log_pkey PRIMARY KEY (id);


--
-- Name: admin_console_preferences admin_console_preferences_admin_user_id_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_preferences
    ADD CONSTRAINT admin_console_preferences_admin_user_id_realm_id_key UNIQUE (admin_user_id, realm_id);


--
-- Name: admin_console_preferences admin_console_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_preferences
    ADD CONSTRAINT admin_console_preferences_pkey PRIMARY KEY (id);


--
-- Name: admin_console_sessions admin_console_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_sessions
    ADD CONSTRAINT admin_console_sessions_pkey PRIMARY KEY (id);


--
-- Name: admin_console_sessions admin_console_sessions_session_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_sessions
    ADD CONSTRAINT admin_console_sessions_session_token_key UNIQUE (session_token);


--
-- Name: admin_dashboard_metrics admin_dashboard_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_dashboard_metrics
    ADD CONSTRAINT admin_dashboard_metrics_pkey PRIMARY KEY (id);


--
-- Name: admin_dashboard_metrics admin_dashboard_metrics_realm_id_metric_type_metric_name_pe_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_dashboard_metrics
    ADD CONSTRAINT admin_dashboard_metrics_realm_id_metric_type_metric_name_pe_key UNIQUE (realm_id, metric_type, metric_name, period_start);


--
-- Name: admin_events admin_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_events
    ADD CONSTRAINT admin_events_pkey PRIMARY KEY (id);


--
-- Name: admin_level_types admin_level_types_code_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_level_types
    ADD CONSTRAINT admin_level_types_code_realm_id_key UNIQUE (code, realm_id);


--
-- Name: admin_level_types admin_level_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_level_types
    ADD CONSTRAINT admin_level_types_pkey PRIMARY KEY (id);


--
-- Name: admin_notifications admin_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_notifications
    ADD CONSTRAINT admin_notifications_pkey PRIMARY KEY (id);


--
-- Name: audit_integrity_checks audit_integrity_checks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_integrity_checks
    ADD CONSTRAINT audit_integrity_checks_pkey PRIMARY KEY (id);


--
-- Name: audit_integrity_failures audit_integrity_failures_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_integrity_failures
    ADD CONSTRAINT audit_integrity_failures_pkey PRIMARY KEY (id);


--
-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_logs
    ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (id);


--
-- Name: authentication_executions authentication_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_executions
    ADD CONSTRAINT authentication_executions_pkey PRIMARY KEY (id);


--
-- Name: authentication_flows authentication_flows_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_flows
    ADD CONSTRAINT authentication_flows_pkey PRIMARY KEY (id);


--
-- Name: authentication_flows authentication_flows_realm_id_alias_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_flows
    ADD CONSTRAINT authentication_flows_realm_id_alias_key UNIQUE (realm_id, alias);


--
-- Name: authentication_sessions authentication_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_sessions
    ADD CONSTRAINT authentication_sessions_pkey PRIMARY KEY (id);


--
-- Name: authenticator_configs authenticator_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_configs
    ADD CONSTRAINT authenticator_configs_pkey PRIMARY KEY (id);


--
-- Name: authenticator_configs authenticator_configs_realm_id_alias_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_configs
    ADD CONSTRAINT authenticator_configs_realm_id_alias_key UNIQUE (realm_id, alias);


--
-- Name: authenticator_execution_results authenticator_execution_results_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_execution_results
    ADD CONSTRAINT authenticator_execution_results_pkey PRIMARY KEY (id);


--
-- Name: authenticator_executions authenticator_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_executions
    ADD CONSTRAINT authenticator_executions_pkey PRIMARY KEY (id);


--
-- Name: authorization_policies authorization_policies_name_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authorization_policies
    ADD CONSTRAINT authorization_policies_name_realm_id_key UNIQUE (name, realm_id);


--
-- Name: authorization_policies authorization_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authorization_policies
    ADD CONSTRAINT authorization_policies_pkey PRIMARY KEY (id);


--
-- Name: capabilities capabilities_code_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.capabilities
    ADD CONSTRAINT capabilities_code_realm_id_key UNIQUE (code, realm_id);


--
-- Name: capabilities capabilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.capabilities
    ADD CONSTRAINT capabilities_pkey PRIMARY KEY (id);


--
-- Name: captcha_behavioral_metrics captcha_behavioral_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_behavioral_metrics
    ADD CONSTRAINT captcha_behavioral_metrics_pkey PRIMARY KEY (id);


--
-- Name: captcha_bot_detection_metrics captcha_bot_detection_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_bot_detection_metrics
    ADD CONSTRAINT captcha_bot_detection_metrics_pkey PRIMARY KEY (id);


--
-- Name: captcha_challenges captcha_challenges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_challenges
    ADD CONSTRAINT captcha_challenges_pkey PRIMARY KEY (id);


--
-- Name: captcha_difficulty_adjustments captcha_difficulty_adjustments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_difficulty_adjustments
    ADD CONSTRAINT captcha_difficulty_adjustments_pkey PRIMARY KEY (id);


--
-- Name: captcha_performance_metrics captcha_performance_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_performance_metrics
    ADD CONSTRAINT captcha_performance_metrics_pkey PRIMARY KEY (id);


--
-- Name: captcha_security_event_metrics captcha_security_event_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_security_event_metrics
    ADD CONSTRAINT captcha_security_event_metrics_pkey PRIMARY KEY (id);


--
-- Name: captcha_type_effectiveness captcha_type_effectiveness_challenge_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_type_effectiveness
    ADD CONSTRAINT captcha_type_effectiveness_challenge_type_key UNIQUE (challenge_type);


--
-- Name: captcha_type_effectiveness captcha_type_effectiveness_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_type_effectiveness
    ADD CONSTRAINT captcha_type_effectiveness_pkey PRIMARY KEY (id);


--
-- Name: captcha_user_experience_metrics captcha_user_experience_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_user_experience_metrics
    ADD CONSTRAINT captcha_user_experience_metrics_pkey PRIMARY KEY (id);


--
-- Name: captcha_user_history captcha_user_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_user_history
    ADD CONSTRAINT captcha_user_history_pkey PRIMARY KEY (id);


--
-- Name: captcha_user_type_performance captcha_user_type_performance_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_user_type_performance
    ADD CONSTRAINT captcha_user_type_performance_pkey PRIMARY KEY (id);


--
-- Name: captcha_validation_attempts captcha_validation_attempts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_validation_attempts
    ADD CONSTRAINT captcha_validation_attempts_pkey PRIMARY KEY (id);


--
-- Name: client_default_scopes client_default_scopes_client_id_scope_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_default_scopes
    ADD CONSTRAINT client_default_scopes_client_id_scope_id_key UNIQUE (client_id, scope_id);


--
-- Name: client_default_scopes client_default_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_default_scopes
    ADD CONSTRAINT client_default_scopes_pkey PRIMARY KEY (id);


--
-- Name: client_optional_scopes client_optional_scopes_client_id_scope_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_optional_scopes
    ADD CONSTRAINT client_optional_scopes_client_id_scope_id_key UNIQUE (client_id, scope_id);


--
-- Name: client_optional_scopes client_optional_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_optional_scopes
    ADD CONSTRAINT client_optional_scopes_pkey PRIMARY KEY (id);


--
-- Name: client_policies client_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policies
    ADD CONSTRAINT client_policies_pkey PRIMARY KEY (id);


--
-- Name: client_policies client_policies_realm_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policies
    ADD CONSTRAINT client_policies_realm_id_name_key UNIQUE (realm_id, name);


--
-- Name: client_policy_assignments client_policy_assignments_client_id_policy_id_profile_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policy_assignments
    ADD CONSTRAINT client_policy_assignments_client_id_policy_id_profile_id_key UNIQUE (client_id, policy_id, profile_id);


--
-- Name: client_policy_assignments client_policy_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policy_assignments
    ADD CONSTRAINT client_policy_assignments_pkey PRIMARY KEY (id);


--
-- Name: client_profiles client_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_profiles
    ADD CONSTRAINT client_profiles_pkey PRIMARY KEY (id);


--
-- Name: client_profiles client_profiles_realm_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_profiles
    ADD CONSTRAINT client_profiles_realm_id_name_key UNIQUE (realm_id, name);


--
-- Name: client_registration_audit_log client_registration_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_audit_log
    ADD CONSTRAINT client_registration_audit_log_pkey PRIMARY KEY (id);


--
-- Name: client_registration_policies client_registration_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_policies
    ADD CONSTRAINT client_registration_policies_pkey PRIMARY KEY (id);


--
-- Name: client_registration_policies client_registration_policies_realm_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_policies
    ADD CONSTRAINT client_registration_policies_realm_id_name_key UNIQUE (realm_id, name);


--
-- Name: client_registration_tokens client_registration_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_tokens
    ADD CONSTRAINT client_registration_tokens_pkey PRIMARY KEY (id);


--
-- Name: client_registration_tokens client_registration_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_tokens
    ADD CONSTRAINT client_registration_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: client_scope_mappings client_scope_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scope_mappings
    ADD CONSTRAINT client_scope_mappings_pkey PRIMARY KEY (id);


--
-- Name: client_scope_mappings client_scope_mappings_scope_id_protocol_mapper_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scope_mappings
    ADD CONSTRAINT client_scope_mappings_scope_id_protocol_mapper_id_key UNIQUE (scope_id, protocol_mapper_id);


--
-- Name: client_scopes client_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scopes
    ADD CONSTRAINT client_scopes_pkey PRIMARY KEY (id);


--
-- Name: client_scopes client_scopes_realm_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scopes
    ADD CONSTRAINT client_scopes_realm_id_name_key UNIQUE (realm_id, name);


--
-- Name: credential_types credential_types_code_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.credential_types
    ADD CONSTRAINT credential_types_code_realm_id_key UNIQUE (code, realm_id);


--
-- Name: credential_types credential_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.credential_types
    ADD CONSTRAINT credential_types_pkey PRIMARY KEY (id);


--
-- Name: custom_themes custom_themes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.custom_themes
    ADD CONSTRAINT custom_themes_pkey PRIMARY KEY (id);


--
-- Name: device_sessions device_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_sessions
    ADD CONSTRAINT device_sessions_pkey PRIMARY KEY (id);


--
-- Name: device_trust_history device_trust_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_trust_history
    ADD CONSTRAINT device_trust_history_pkey PRIMARY KEY (id);


--
-- Name: devices devices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);


--
-- Name: devices devices_user_id_device_fingerprint_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.devices
    ADD CONSTRAINT devices_user_id_device_fingerprint_key UNIQUE (user_id, device_fingerprint);


--
-- Name: event_listener_executions event_listener_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listener_executions
    ADD CONSTRAINT event_listener_executions_pkey PRIMARY KEY (id);


--
-- Name: event_listeners event_listeners_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listeners
    ADD CONSTRAINT event_listeners_pkey PRIMARY KEY (id);


--
-- Name: event_listeners event_listeners_realm_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listeners
    ADD CONSTRAINT event_listeners_realm_id_name_key UNIQUE (realm_id, name);


--
-- Name: event_log event_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_log
    ADD CONSTRAINT event_log_pkey PRIMARY KEY (id);


--
-- Name: event_webhooks event_webhooks_listener_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_webhooks
    ADD CONSTRAINT event_webhooks_listener_id_key UNIQUE (listener_id);


--
-- Name: event_webhooks event_webhooks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_webhooks
    ADD CONSTRAINT event_webhooks_pkey PRIMARY KEY (id);


--
-- Name: events events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (id);


--
-- Name: federated_auth_log federated_auth_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_auth_log
    ADD CONSTRAINT federated_auth_log_pkey PRIMARY KEY (id);


--
-- Name: federated_identities federated_identities_identity_provider_id_external_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identities
    ADD CONSTRAINT federated_identities_identity_provider_id_external_id_key UNIQUE (identity_provider_id, external_id);


--
-- Name: federated_identities federated_identities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identities
    ADD CONSTRAINT federated_identities_pkey PRIMARY KEY (id);


--
-- Name: federated_identity_links federated_identity_links_identity_provider_alias_federated__key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identity_links
    ADD CONSTRAINT federated_identity_links_identity_provider_alias_federated__key UNIQUE (identity_provider_alias, federated_user_id);


--
-- Name: federated_identity_links federated_identity_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identity_links
    ADD CONSTRAINT federated_identity_links_pkey PRIMARY KEY (id);


--
-- Name: federated_identity_links federated_identity_links_user_id_identity_provider_alias_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identity_links
    ADD CONSTRAINT federated_identity_links_user_id_identity_provider_alias_key UNIQUE (user_id, identity_provider_alias);


--
-- Name: group_attributes group_attributes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_attributes
    ADD CONSTRAINT group_attributes_pkey PRIMARY KEY (id);


--
-- Name: group_roles group_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_roles
    ADD CONSTRAINT group_roles_pkey PRIMARY KEY (id);


--
-- Name: groups groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.groups
    ADD CONSTRAINT groups_pkey PRIMARY KEY (id);


--
-- Name: identity_broker_configs identity_broker_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_broker_configs
    ADD CONSTRAINT identity_broker_configs_pkey PRIMARY KEY (id);


--
-- Name: identity_broker_configs identity_broker_configs_realm_id_alias_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_broker_configs
    ADD CONSTRAINT identity_broker_configs_realm_id_alias_key UNIQUE (realm_id, alias);


--
-- Name: identity_provider_mappers identity_provider_mappers_identity_provider_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_provider_mappers
    ADD CONSTRAINT identity_provider_mappers_identity_provider_id_name_key UNIQUE (identity_provider_id, name);


--
-- Name: identity_provider_mappers identity_provider_mappers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_provider_mappers
    ADD CONSTRAINT identity_provider_mappers_pkey PRIMARY KEY (id);


--
-- Name: identity_providers identity_providers_name_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_providers
    ADD CONSTRAINT identity_providers_name_realm_id_key UNIQUE (name, realm_id);


--
-- Name: identity_providers identity_providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_providers
    ADD CONSTRAINT identity_providers_pkey PRIMARY KEY (id);


--
-- Name: initial_access_tokens initial_access_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.initial_access_tokens
    ADD CONSTRAINT initial_access_tokens_pkey PRIMARY KEY (id);


--
-- Name: initial_access_tokens initial_access_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.initial_access_tokens
    ADD CONSTRAINT initial_access_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: key_rotation_audit key_rotation_audit_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.key_rotation_audit
    ADD CONSTRAINT key_rotation_audit_pkey PRIMARY KEY (id);


--
-- Name: mfa_admin_actions mfa_admin_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_admin_actions
    ADD CONSTRAINT mfa_admin_actions_pkey PRIMARY KEY (id);


--
-- Name: mfa_devices mfa_devices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_devices
    ADD CONSTRAINT mfa_devices_pkey PRIMARY KEY (id);


--
-- Name: mfa_policies mfa_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_policies
    ADD CONSTRAINT mfa_policies_pkey PRIMARY KEY (id);


--
-- Name: oauth2_access_tokens oauth2_access_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_access_tokens
    ADD CONSTRAINT oauth2_access_tokens_pkey PRIMARY KEY (id);


--
-- Name: oauth2_access_tokens oauth2_access_tokens_refresh_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_access_tokens
    ADD CONSTRAINT oauth2_access_tokens_refresh_token_hash_key UNIQUE (refresh_token_hash);


--
-- Name: oauth2_access_tokens oauth2_access_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_access_tokens
    ADD CONSTRAINT oauth2_access_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: oauth2_authorization_codes oauth2_authorization_codes_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_authorization_codes
    ADD CONSTRAINT oauth2_authorization_codes_code_key UNIQUE (code);


--
-- Name: oauth2_authorization_codes oauth2_authorization_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_authorization_codes
    ADD CONSTRAINT oauth2_authorization_codes_pkey PRIMARY KEY (id);


--
-- Name: oauth2_clients oauth2_clients_client_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_clients
    ADD CONSTRAINT oauth2_clients_client_id_key UNIQUE (client_id);


--
-- Name: oauth2_clients oauth2_clients_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_clients
    ADD CONSTRAINT oauth2_clients_pkey PRIMARY KEY (id);


--
-- Name: oauth2_provider_configs oauth2_provider_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_provider_configs
    ADD CONSTRAINT oauth2_provider_configs_pkey PRIMARY KEY (id);


--
-- Name: oauth2_provider_configs oauth2_provider_configs_realm_id_alias_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_provider_configs
    ADD CONSTRAINT oauth2_provider_configs_realm_id_alias_key UNIQUE (realm_id, alias);


--
-- Name: oauth2_states oauth2_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_states
    ADD CONSTRAINT oauth2_states_pkey PRIMARY KEY (id);


--
-- Name: oauth2_states oauth2_states_state_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_states
    ADD CONSTRAINT oauth2_states_state_token_key UNIQUE (state_token);


--
-- Name: oauth2_token_exchanges oauth2_token_exchanges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_token_exchanges
    ADD CONSTRAINT oauth2_token_exchanges_pkey PRIMARY KEY (id);


--
-- Name: offline_tokens offline_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.offline_tokens
    ADD CONSTRAINT offline_tokens_pkey PRIMARY KEY (id);


--
-- Name: offline_tokens offline_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.offline_tokens
    ADD CONSTRAINT offline_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: organization_domains organization_domains_domain_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_domains
    ADD CONSTRAINT organization_domains_domain_key UNIQUE (domain);


--
-- Name: organization_domains organization_domains_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_domains
    ADD CONSTRAINT organization_domains_pkey PRIMARY KEY (id);


--
-- Name: organization_domains organization_domains_verification_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_domains
    ADD CONSTRAINT organization_domains_verification_token_key UNIQUE (verification_token);


--
-- Name: organization_identity_providers organization_identity_provide_organization_id_identity_prov_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_identity_providers
    ADD CONSTRAINT organization_identity_provide_organization_id_identity_prov_key UNIQUE (organization_id, identity_provider_id);


--
-- Name: organization_identity_providers organization_identity_providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_identity_providers
    ADD CONSTRAINT organization_identity_providers_pkey PRIMARY KEY (id);


--
-- Name: organization_invitations organization_invitations_organization_id_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_organization_id_email_key UNIQUE (organization_id, email);


--
-- Name: organization_invitations organization_invitations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_pkey PRIMARY KEY (id);


--
-- Name: organization_invitations organization_invitations_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_token_hash_key UNIQUE (token_hash);


--
-- Name: organization_members organization_members_organization_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_organization_id_user_id_key UNIQUE (organization_id, user_id);


--
-- Name: organization_members organization_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_pkey PRIMARY KEY (id);


--
-- Name: organizations organizations_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organizations
    ADD CONSTRAINT organizations_name_key UNIQUE (name);


--
-- Name: organizations organizations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organizations
    ADD CONSTRAINT organizations_pkey PRIMARY KEY (id);


--
-- Name: password_history password_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.password_history
    ADD CONSTRAINT password_history_pkey PRIMARY KEY (id);


--
-- Name: permission_tickets permission_tickets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.permission_tickets
    ADD CONSTRAINT permission_tickets_pkey PRIMARY KEY (id);


--
-- Name: protocol_mappers protocol_mappers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.protocol_mappers
    ADD CONSTRAINT protocol_mappers_pkey PRIMARY KEY (id);


--
-- Name: protocol_mappers protocol_mappers_unique_name; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.protocol_mappers
    ADD CONSTRAINT protocol_mappers_unique_name UNIQUE NULLS NOT DISTINCT (client_id, client_scope_id, realm_id, name);


--
-- Name: realm_theme_settings realm_theme_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_pkey PRIMARY KEY (id);


--
-- Name: realm_theme_settings realm_theme_settings_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_realm_id_key UNIQUE (realm_id);


--
-- Name: realms realms_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realms
    ADD CONSTRAINT realms_pkey PRIMARY KEY (id);


--
-- Name: refresh_token_history refresh_token_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.refresh_token_history
    ADD CONSTRAINT refresh_token_history_pkey PRIMARY KEY (id);


--
-- Name: resource_servers resource_servers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.resource_servers
    ADD CONSTRAINT resource_servers_pkey PRIMARY KEY (id);


--
-- Name: resources resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.resources
    ADD CONSTRAINT resources_pkey PRIMARY KEY (id);


--
-- Name: role_capabilities role_capabilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_capabilities
    ADD CONSTRAINT role_capabilities_pkey PRIMARY KEY (id);


--
-- Name: role_capabilities role_capabilities_role_id_capability_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_capabilities
    ADD CONSTRAINT role_capabilities_role_id_capability_id_key UNIQUE (role_id, capability_id);


--
-- Name: role_hierarchy role_hierarchy_parent_role_id_child_role_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_hierarchy
    ADD CONSTRAINT role_hierarchy_parent_role_id_child_role_id_key UNIQUE (parent_role_id, child_role_id);


--
-- Name: role_hierarchy role_hierarchy_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_hierarchy
    ADD CONSTRAINT role_hierarchy_pkey PRIMARY KEY (id);


--
-- Name: role_permissions role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_permissions
    ADD CONSTRAINT role_permissions_pkey PRIMARY KEY (id);


--
-- Name: role_permissions role_permissions_role_id_permission_resource_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_permissions
    ADD CONSTRAINT role_permissions_role_id_permission_resource_key UNIQUE (role_id, permission, resource);


--
-- Name: role_policies role_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_policies
    ADD CONSTRAINT role_policies_pkey PRIMARY KEY (id);


--
-- Name: role_policies role_policies_role_id_policy_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_policies
    ADD CONSTRAINT role_policies_role_id_policy_id_key UNIQUE (role_id, policy_id);


--
-- Name: role_types role_types_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_types
    ADD CONSTRAINT role_types_code_key UNIQUE (code);


--
-- Name: role_types role_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_types
    ADD CONSTRAINT role_types_pkey PRIMARY KEY (id);


--
-- Name: roles roles_name_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.roles
    ADD CONSTRAINT roles_name_realm_id_key UNIQUE (name, realm_id);


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- Name: saml_assertion_cache saml_assertion_cache_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_assertion_cache
    ADD CONSTRAINT saml_assertion_cache_pkey PRIMARY KEY (assertion_id);


--
-- Name: saml_identity_providers saml_identity_providers_entity_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_identity_providers
    ADD CONSTRAINT saml_identity_providers_entity_id_key UNIQUE (entity_id);


--
-- Name: saml_identity_providers saml_identity_providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_identity_providers
    ADD CONSTRAINT saml_identity_providers_pkey PRIMARY KEY (id);


--
-- Name: saml_messages saml_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_messages
    ADD CONSTRAINT saml_messages_pkey PRIMARY KEY (id);


--
-- Name: saml_messages saml_messages_saml_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_messages
    ADD CONSTRAINT saml_messages_saml_id_key UNIQUE (saml_id);


--
-- Name: saml_service_providers saml_service_providers_entity_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_service_providers
    ADD CONSTRAINT saml_service_providers_entity_id_key UNIQUE (entity_id);


--
-- Name: saml_service_providers saml_service_providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_service_providers
    ADD CONSTRAINT saml_service_providers_pkey PRIMARY KEY (id);


--
-- Name: saml_sessions saml_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_sessions
    ADD CONSTRAINT saml_sessions_pkey PRIMARY KEY (id);


--
-- Name: saml_sessions saml_sessions_session_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_sessions
    ADD CONSTRAINT saml_sessions_session_id_key UNIQUE (session_id);


--
-- Name: satker_admin_roles satker_admin_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_admin_roles
    ADD CONSTRAINT satker_admin_roles_pkey PRIMARY KEY (id);


--
-- Name: satker_audit_logs satker_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_audit_logs
    ADD CONSTRAINT satker_audit_logs_pkey PRIMARY KEY (id);


--
-- Name: satker_permissions satker_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_permissions
    ADD CONSTRAINT satker_permissions_pkey PRIMARY KEY (id);


--
-- Name: satker_types satker_types_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_types
    ADD CONSTRAINT satker_types_code_key UNIQUE (code);


--
-- Name: satker_types satker_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_types
    ADD CONSTRAINT satker_types_pkey PRIMARY KEY (id);


--
-- Name: satkers satkers_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satkers
    ADD CONSTRAINT satkers_code_key UNIQUE (code);


--
-- Name: satkers satkers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satkers
    ADD CONSTRAINT satkers_pkey PRIMARY KEY (id);


--
-- Name: scope_types scope_types_code_realm_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scope_types
    ADD CONSTRAINT scope_types_code_realm_id_key UNIQUE (code, realm_id);


--
-- Name: scope_types scope_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scope_types
    ADD CONSTRAINT scope_types_pkey PRIMARY KEY (id);


--
-- Name: scopes scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scopes
    ADD CONSTRAINT scopes_pkey PRIMARY KEY (id);


--
-- Name: service_account_audit_log service_account_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_audit_log
    ADD CONSTRAINT service_account_audit_log_pkey PRIMARY KEY (id);


--
-- Name: service_account_roles service_account_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_roles
    ADD CONSTRAINT service_account_roles_pkey PRIMARY KEY (id);


--
-- Name: service_accounts service_accounts_client_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_accounts
    ADD CONSTRAINT service_accounts_client_id_key UNIQUE (client_id);


--
-- Name: service_accounts service_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_accounts
    ADD CONSTRAINT service_accounts_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.sessions
    ADD CONSTRAINT sessions_token_hash_key UNIQUE (token_hash);


--
-- Name: social_accounts social_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_accounts
    ADD CONSTRAINT social_accounts_pkey PRIMARY KEY (id);


--
-- Name: social_accounts social_accounts_provider_provider_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_accounts
    ADD CONSTRAINT social_accounts_provider_provider_user_id_key UNIQUE (provider, provider_user_id);


--
-- Name: social_accounts social_accounts_user_id_provider_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_accounts
    ADD CONSTRAINT social_accounts_user_id_provider_key UNIQUE (user_id, provider);


--
-- Name: social_login_configs social_login_configs_oauth2_config_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_login_configs
    ADD CONSTRAINT social_login_configs_oauth2_config_id_key UNIQUE (oauth2_config_id);


--
-- Name: social_login_configs social_login_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_login_configs
    ADD CONSTRAINT social_login_configs_pkey PRIMARY KEY (id);


--
-- Name: software_statement_issuers software_statement_issuers_issuer_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.software_statement_issuers
    ADD CONSTRAINT software_statement_issuers_issuer_key UNIQUE (issuer);


--
-- Name: software_statement_issuers software_statement_issuers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.software_statement_issuers
    ADD CONSTRAINT software_statement_issuers_pkey PRIMARY KEY (id);


--
-- Name: theme_inheritance theme_inheritance_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_inheritance
    ADD CONSTRAINT theme_inheritance_pkey PRIMARY KEY (id);


--
-- Name: theme_resources theme_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_resources
    ADD CONSTRAINT theme_resources_pkey PRIMARY KEY (id);


--
-- Name: theme_resources theme_resources_theme_id_resource_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_resources
    ADD CONSTRAINT theme_resources_theme_id_resource_name_key UNIQUE (theme_id, resource_name);


--
-- Name: theme_templates theme_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_templates
    ADD CONSTRAINT theme_templates_pkey PRIMARY KEY (id);


--
-- Name: theme_templates theme_templates_theme_id_template_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_templates
    ADD CONSTRAINT theme_templates_theme_id_template_name_key UNIQUE (theme_id, template_name);


--
-- Name: theme_types theme_types_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_types
    ADD CONSTRAINT theme_types_code_key UNIQUE (code);


--
-- Name: theme_types theme_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_types
    ADD CONSTRAINT theme_types_pkey PRIMARY KEY (id);


--
-- Name: token_exchange_audit token_exchange_audit_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.token_exchange_audit
    ADD CONSTRAINT token_exchange_audit_pkey PRIMARY KEY (id);


--
-- Name: uma_permission_requests uma_permission_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_permission_requests
    ADD CONSTRAINT uma_permission_requests_pkey PRIMARY KEY (id);


--
-- Name: uma_permission_requests uma_permission_requests_ticket_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_permission_requests
    ADD CONSTRAINT uma_permission_requests_ticket_key UNIQUE (ticket);


--
-- Name: uma_policies uma_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_policies
    ADD CONSTRAINT uma_policies_pkey PRIMARY KEY (id);


--
-- Name: captcha_user_history unique_user_history; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_user_history
    ADD CONSTRAINT unique_user_history UNIQUE (user_id);


--
-- Name: captcha_user_type_performance unique_user_type_performance; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_user_type_performance
    ADD CONSTRAINT unique_user_type_performance UNIQUE (user_id, challenge_type);


--
-- Name: group_attributes uq_group_attributes; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_attributes
    ADD CONSTRAINT uq_group_attributes UNIQUE (group_id, name, value);


--
-- Name: group_roles uq_group_roles; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_roles
    ADD CONSTRAINT uq_group_roles UNIQUE (group_id, role_id);


--
-- Name: groups uq_groups_name_parent; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.groups
    ADD CONSTRAINT uq_groups_name_parent UNIQUE (realm_id, parent_id, name);


--
-- Name: groups uq_groups_path; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.groups
    ADD CONSTRAINT uq_groups_path UNIQUE (realm_id, path);


--
-- Name: satker_admin_roles uq_satker_admin_role; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_admin_roles
    ADD CONSTRAINT uq_satker_admin_role UNIQUE (user_id, satker_code, admin_level);


--
-- Name: satker_permissions uq_satker_permission; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_permissions
    ADD CONSTRAINT uq_satker_permission UNIQUE (user_id, satker_code, permission_type, resource_type, action);


--
-- Name: service_accounts uq_service_account_name_realm; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_accounts
    ADD CONSTRAINT uq_service_account_name_realm UNIQUE (name, realm_id);


--
-- Name: service_account_roles uq_service_account_role; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_roles
    ADD CONSTRAINT uq_service_account_role UNIQUE (service_account_id, role_id);


--
-- Name: user_groups uq_user_groups; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_groups
    ADD CONSTRAINT uq_user_groups UNIQUE (user_id, group_id);


--
-- Name: user_attributes user_attributes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_attributes
    ADD CONSTRAINT user_attributes_pkey PRIMARY KEY (id);


--
-- Name: user_attributes user_attributes_user_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_attributes
    ADD CONSTRAINT user_attributes_user_id_name_key UNIQUE (user_id, name);


--
-- Name: user_consent_scopes user_consent_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consent_scopes
    ADD CONSTRAINT user_consent_scopes_pkey PRIMARY KEY (id);


--
-- Name: user_consent_scopes user_consent_scopes_user_id_client_id_scope_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consent_scopes
    ADD CONSTRAINT user_consent_scopes_user_id_client_id_scope_id_key UNIQUE (user_id, client_id, scope_id);


--
-- Name: user_consents user_consents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consents
    ADD CONSTRAINT user_consents_pkey PRIMARY KEY (id);


--
-- Name: user_consents user_consents_user_id_client_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consents
    ADD CONSTRAINT user_consents_user_id_client_id_key UNIQUE (user_id, client_id);


--
-- Name: user_groups user_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_groups
    ADD CONSTRAINT user_groups_pkey PRIMARY KEY (id);


--
-- Name: user_policies user_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_policies
    ADD CONSTRAINT user_policies_pkey PRIMARY KEY (id);


--
-- Name: user_policies user_policies_user_id_policy_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_policies
    ADD CONSTRAINT user_policies_user_id_policy_id_key UNIQUE (user_id, policy_id);


--
-- Name: user_roles user_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_roles
    ADD CONSTRAINT user_roles_pkey PRIMARY KEY (id);


--
-- Name: user_roles user_roles_user_id_role_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_roles
    ADD CONSTRAINT user_roles_user_id_role_id_key UNIQUE (user_id, role_id);


--
-- Name: user_sessions user_sessions_offline_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_offline_token_hash_key UNIQUE (offline_token_hash);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: user_sessions user_sessions_session_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_session_id_key UNIQUE (session_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: webauthn_audit_log webauthn_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_audit_log
    ADD CONSTRAINT webauthn_audit_log_pkey PRIMARY KEY (id);


--
-- Name: webauthn_challenges webauthn_challenges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_challenges
    ADD CONSTRAINT webauthn_challenges_pkey PRIMARY KEY (id);


--
-- Name: webauthn_challenges webauthn_challenges_user_type_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_challenges
    ADD CONSTRAINT webauthn_challenges_user_type_unique UNIQUE (user_id, challenge_type, used);


--
-- Name: webauthn_credentials webauthn_credentials_cred_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_cred_id_key UNIQUE (cred_id);


--
-- Name: webauthn_credentials_old webauthn_credentials_credential_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials_old
    ADD CONSTRAINT webauthn_credentials_credential_id_key UNIQUE (credential_id);


--
-- Name: webauthn_credentials_old webauthn_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials_old
    ADD CONSTRAINT webauthn_credentials_pkey PRIMARY KEY (id);


--
-- Name: webauthn_credentials webauthn_credentials_pkey1; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_pkey1 PRIMARY KEY (id);


--
-- Name: webauthn_credentials webauthn_credentials_user_cred_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_user_cred_unique UNIQUE (user_id, cred_id);


--
-- Name: webauthn_credentials_old webauthn_credentials_user_id_credential_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials_old
    ADD CONSTRAINT webauthn_credentials_user_id_credential_id_key UNIQUE (user_id, credential_id);


--
-- Name: idx_mfa_backup_codes_user_unused; Type: INDEX; Schema: authenc; Owner: -
--

CREATE INDEX idx_mfa_backup_codes_user_unused ON authenc.mfa_backup_codes USING btree (user_id) WHERE (used = false);


--
-- Name: idx_token_revocations_expires_at; Type: INDEX; Schema: authenc; Owner: -
--

CREATE INDEX idx_token_revocations_expires_at ON authenc.token_revocations USING btree (expires_at);


--
-- Name: idx_access_levels_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_levels_code ON authenc.access_levels USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_access_levels_numeric; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_levels_numeric ON authenc.access_levels USING btree (numeric_level DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_account_linking_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_account_linking_expires ON authenc.account_linking_requests USING btree (expires_at) WHERE ((status)::text = 'PENDING'::text);


--
-- Name: idx_account_linking_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_account_linking_status ON authenc.account_linking_requests USING btree (status) WHERE ((status)::text = 'PENDING'::text);


--
-- Name: idx_account_linking_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_account_linking_token ON authenc.account_linking_requests USING btree (confirmation_token);


--
-- Name: idx_account_linking_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_account_linking_user ON authenc.account_linking_requests USING btree (user_id);


--
-- Name: idx_admin_audit_log_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_action ON authenc.admin_audit_log USING btree (action);


--
-- Name: idx_admin_audit_log_admin_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_admin_user ON authenc.admin_audit_log USING btree (admin_user_id);


--
-- Name: idx_admin_audit_log_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_created ON authenc.admin_audit_log USING btree (created_at DESC);


--
-- Name: idx_admin_audit_log_geolocation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_geolocation ON authenc.admin_audit_log USING gin (geolocation_data);


--
-- Name: idx_admin_audit_log_operation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_operation ON authenc.admin_audit_log USING btree (operation_type, status);


--
-- Name: idx_admin_audit_log_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_realm ON authenc.admin_audit_log USING btree (realm_id);


--
-- Name: idx_admin_audit_log_request_payload; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_request_payload ON authenc.admin_audit_log USING gin (request_payload);


--
-- Name: idx_admin_audit_log_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_resource ON authenc.admin_audit_log USING btree (resource_type, resource_id);


--
-- Name: idx_admin_audit_log_response_payload; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_audit_log_response_payload ON authenc.admin_audit_log USING gin (response_payload);


--
-- Name: idx_admin_events_auth_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_auth_user_id ON authenc.admin_events USING btree (auth_user_id);


--
-- Name: idx_admin_events_operation_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_operation_type ON authenc.admin_events USING btree (operation_type);


--
-- Name: idx_admin_events_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_realm_id ON authenc.admin_events USING btree (realm_id);


--
-- Name: idx_admin_events_realm_resource_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_realm_resource_time ON authenc.admin_events USING btree (realm_id, resource_type, "time" DESC);


--
-- Name: INDEX idx_admin_events_realm_resource_time; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_admin_events_realm_resource_time IS 'Improves admin audit queries by resource type';


--
-- Name: idx_admin_events_resource_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_resource_type ON authenc.admin_events USING btree (resource_type);


--
-- Name: idx_admin_events_signature; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_signature ON authenc.admin_events USING btree (signature) WHERE (signature IS NOT NULL);


--
-- Name: idx_admin_events_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_time ON authenc.admin_events USING btree ("time" DESC);


--
-- Name: idx_admin_events_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_events_user_time ON authenc.admin_events USING btree (auth_user_id, "time" DESC) WHERE (auth_user_id IS NOT NULL);


--
-- Name: INDEX idx_admin_events_user_time; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_admin_events_user_time IS 'Improves admin audit trail queries';


--
-- Name: idx_admin_level_types_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_level_types_code ON authenc.admin_level_types USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_admin_level_types_hierarchy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_level_types_hierarchy ON authenc.admin_level_types USING btree (hierarchy_level DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_admin_level_types_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_level_types_parent ON authenc.admin_level_types USING btree (parent_level_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_admin_notifications_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_notifications_created ON authenc.admin_notifications USING btree (created_at DESC);


--
-- Name: idx_admin_notifications_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_notifications_realm ON authenc.admin_notifications USING btree (realm_id);


--
-- Name: idx_admin_notifications_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_notifications_target ON authenc.admin_notifications USING btree (target_admin_user_id, is_read);


--
-- Name: idx_admin_notifications_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_notifications_type ON authenc.admin_notifications USING btree (notification_type, priority);


--
-- Name: idx_admin_notifications_unread; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_notifications_unread ON authenc.admin_notifications USING btree (target_admin_user_id) WHERE (is_read = false);


--
-- Name: idx_admin_preferences_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_preferences_realm ON authenc.admin_console_preferences USING btree (realm_id);


--
-- Name: idx_admin_preferences_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_preferences_user ON authenc.admin_console_preferences USING btree (admin_user_id);


--
-- Name: idx_admin_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_sessions_active ON authenc.admin_console_sessions USING btree (is_active, last_activity_at) WHERE (is_active = true);


--
-- Name: idx_admin_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_sessions_expires ON authenc.admin_console_sessions USING btree (expires_at) WHERE (is_active = true);


--
-- Name: idx_admin_sessions_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_sessions_realm ON authenc.admin_console_sessions USING btree (realm_id);


--
-- Name: idx_admin_sessions_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_sessions_token ON authenc.admin_console_sessions USING btree (session_token);


--
-- Name: idx_admin_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_sessions_user ON authenc.admin_console_sessions USING btree (admin_user_id);


--
-- Name: idx_audit_integrity_checks_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_integrity_checks_status ON authenc.audit_integrity_checks USING btree (status);


--
-- Name: idx_audit_integrity_checks_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_integrity_checks_time ON authenc.audit_integrity_checks USING btree (check_time DESC);


--
-- Name: idx_audit_integrity_failures_check_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_integrity_failures_check_id ON authenc.audit_integrity_failures USING btree (check_id);


--
-- Name: idx_audit_integrity_failures_detected_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_integrity_failures_detected_at ON authenc.audit_integrity_failures USING btree (detected_at DESC);


--
-- Name: idx_audit_integrity_failures_event_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_integrity_failures_event_id ON authenc.audit_integrity_failures USING btree (event_id);


--
-- Name: idx_audit_logs_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_event_type ON authenc.audit_logs USING btree (event_type);


--
-- Name: idx_audit_logs_mfa_events; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_mfa_events ON authenc.audit_logs USING btree (event_type, "timestamp") WHERE ((event_type)::text = ANY ((ARRAY['mfa_verification_success'::character varying, 'mfa_verification_failed'::character varying, 'mfa_setup_complete'::character varying])::text[]));


--
-- Name: idx_audit_logs_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_request_id ON authenc.audit_logs USING btree (request_id);


--
-- Name: idx_audit_logs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_status ON authenc.audit_logs USING btree (status);


--
-- Name: idx_audit_logs_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_timestamp ON authenc.audit_logs USING btree ("timestamp" DESC);


--
-- Name: idx_audit_logs_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_user_id ON authenc.audit_logs USING btree (user_id);


--
-- Name: idx_audit_logs_user_mfa; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_user_mfa ON authenc.audit_logs USING btree (user_id, event_type, "timestamp") WHERE ((event_type)::text ~~ 'mfa_%'::text);


--
-- Name: idx_auth_executions_flow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_executions_flow ON authenc.authentication_executions USING btree (flow_id);


--
-- Name: idx_auth_executions_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_executions_priority ON authenc.authentication_executions USING btree (flow_id, priority);


--
-- Name: idx_auth_flows_alias; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_flows_alias ON authenc.authentication_flows USING btree (alias);


--
-- Name: idx_auth_flows_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_flows_provider ON authenc.authentication_flows USING btree (provider_id);


--
-- Name: idx_auth_flows_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_flows_realm ON authenc.authentication_flows USING btree (realm_id);


--
-- Name: idx_auth_policies_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_policies_name ON authenc.authorization_policies USING btree (name) WHERE (deleted_at IS NULL);


--
-- Name: idx_auth_policies_path; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_policies_path ON authenc.authorization_policies USING btree (path_pattern) WHERE (deleted_at IS NULL);


--
-- Name: idx_auth_policies_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_policies_priority ON authenc.authorization_policies USING btree (priority DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_auth_sessions_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_client ON authenc.authentication_sessions USING btree (client_id);


--
-- Name: idx_auth_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_expires ON authenc.authentication_sessions USING btree (expires_at);


--
-- Name: idx_auth_sessions_flow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_flow ON authenc.authentication_sessions USING btree (flow_id);


--
-- Name: idx_auth_sessions_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_realm ON authenc.authentication_sessions USING btree (realm_id);


--
-- Name: idx_auth_sessions_state; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_state ON authenc.authentication_sessions USING btree (auth_state);


--
-- Name: idx_auth_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_sessions_user ON authenc.authentication_sessions USING btree (user_id);


--
-- Name: idx_authenticator_configs_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_configs_enabled ON authenc.authenticator_configs USING btree (realm_id, enabled) WHERE (enabled = true);


--
-- Name: idx_authenticator_configs_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_configs_priority ON authenc.authenticator_configs USING btree (realm_id, priority);


--
-- Name: idx_authenticator_configs_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_configs_realm ON authenc.authenticator_configs USING btree (realm_id);


--
-- Name: idx_authenticator_configs_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_configs_type ON authenc.authenticator_configs USING btree (authenticator_type);


--
-- Name: idx_authenticator_executions_authenticator; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_executions_authenticator ON authenc.authenticator_executions USING btree (authenticator_id);


--
-- Name: idx_authenticator_executions_flow; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_executions_flow ON authenc.authenticator_executions USING btree (flow_id);


--
-- Name: idx_authenticator_executions_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_executions_priority ON authenc.authenticator_executions USING btree (flow_id, priority);


--
-- Name: idx_authenticator_executions_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_executions_realm ON authenc.authenticator_executions USING btree (realm_id);


--
-- Name: idx_authenticator_executions_requirement; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_executions_requirement ON authenc.authenticator_executions USING btree (requirement);


--
-- Name: idx_authenticator_results_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_results_created ON authenc.authenticator_execution_results USING btree (created_at);


--
-- Name: idx_authenticator_results_execution; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_results_execution ON authenc.authenticator_execution_results USING btree (execution_id);


--
-- Name: idx_authenticator_results_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_results_session ON authenc.authenticator_execution_results USING btree (session_id);


--
-- Name: idx_authenticator_results_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_results_status ON authenc.authenticator_execution_results USING btree (status);


--
-- Name: idx_authenticator_results_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authenticator_results_user ON authenc.authenticator_execution_results USING btree (user_id);


--
-- Name: idx_behavioral_metrics_challenge; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_behavioral_metrics_challenge ON authenc.captcha_behavioral_metrics USING btree (challenge_id);


--
-- Name: idx_behavioral_metrics_classification; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_behavioral_metrics_classification ON authenc.captcha_behavioral_metrics USING btree (classification, created_at DESC);


--
-- Name: idx_behavioral_metrics_risk_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_behavioral_metrics_risk_score ON authenc.captcha_behavioral_metrics USING btree (risk_score DESC, created_at DESC);


--
-- Name: idx_behavioral_metrics_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_behavioral_metrics_session ON authenc.captcha_behavioral_metrics USING btree (session_id, created_at DESC);


--
-- Name: idx_bot_detection_metrics_accuracy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bot_detection_metrics_accuracy ON authenc.captcha_bot_detection_metrics USING btree (accuracy_rate DESC, "timestamp" DESC);


--
-- Name: idx_bot_detection_metrics_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bot_detection_metrics_timestamp ON authenc.captcha_bot_detection_metrics USING btree ("timestamp" DESC);


--
-- Name: idx_broker_configs_alias; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_broker_configs_alias ON authenc.identity_broker_configs USING btree (alias);


--
-- Name: idx_broker_configs_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_broker_configs_enabled ON authenc.identity_broker_configs USING btree (realm_id) WHERE (enabled = true);


--
-- Name: idx_broker_configs_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_broker_configs_realm ON authenc.identity_broker_configs USING btree (realm_id);


--
-- Name: idx_capabilities_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_capabilities_action ON authenc.capabilities USING btree (action) WHERE (deleted_at IS NULL);


--
-- Name: idx_capabilities_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_capabilities_code ON authenc.capabilities USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_capabilities_resource_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_capabilities_resource_type ON authenc.capabilities USING btree (resource_type) WHERE (deleted_at IS NULL);


--
-- Name: idx_captcha_analytics_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_captcha_analytics_unique ON authenc.captcha_analytics USING btree (date, challenge_type, difficulty_level);


--
-- Name: idx_captcha_challenges_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_captcha_challenges_expires_at ON authenc.captcha_challenges USING btree (expires_at);


--
-- Name: idx_captcha_challenges_ip_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_captcha_challenges_ip_created ON authenc.captcha_challenges USING btree (ip_address, created_at DESC);


--
-- Name: idx_captcha_challenges_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_captcha_challenges_session ON authenc.captcha_challenges USING btree (session_id, created_at DESC) WHERE (session_id IS NOT NULL);


--
-- Name: idx_captcha_challenges_unsolved; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_captcha_challenges_unsolved ON authenc.captcha_challenges USING btree (created_at DESC) WHERE (solved = false);


--
-- Name: idx_client_default_scopes_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_default_scopes_client ON authenc.client_default_scopes USING btree (client_id);


--
-- Name: idx_client_default_scopes_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_default_scopes_scope ON authenc.client_default_scopes USING btree (scope_id);


--
-- Name: idx_client_optional_scopes_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_optional_scopes_client ON authenc.client_optional_scopes USING btree (client_id);


--
-- Name: idx_client_optional_scopes_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_optional_scopes_scope ON authenc.client_optional_scopes USING btree (scope_id);


--
-- Name: idx_client_policies_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policies_enabled ON authenc.client_policies USING btree (enabled) WHERE (enabled = true);


--
-- Name: idx_client_policies_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policies_priority ON authenc.client_policies USING btree (priority DESC);


--
-- Name: idx_client_policies_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policies_realm ON authenc.client_policies USING btree (realm_id);


--
-- Name: idx_client_policies_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policies_type ON authenc.client_policies USING btree (policy_type);


--
-- Name: idx_client_policy_assignments_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policy_assignments_client ON authenc.client_policy_assignments USING btree (client_id);


--
-- Name: idx_client_policy_assignments_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policy_assignments_enabled ON authenc.client_policy_assignments USING btree (enabled) WHERE (enabled = true);


--
-- Name: idx_client_policy_assignments_policy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policy_assignments_policy ON authenc.client_policy_assignments USING btree (policy_id) WHERE (policy_id IS NOT NULL);


--
-- Name: idx_client_policy_assignments_profile; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_policy_assignments_profile ON authenc.client_policy_assignments USING btree (profile_id) WHERE (profile_id IS NOT NULL);


--
-- Name: idx_client_profiles_builtin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_profiles_builtin ON authenc.client_profiles USING btree (is_builtin) WHERE (is_builtin = true);


--
-- Name: idx_client_profiles_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_profiles_enabled ON authenc.client_profiles USING btree (enabled) WHERE (enabled = true);


--
-- Name: idx_client_profiles_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_profiles_realm ON authenc.client_profiles USING btree (realm_id);


--
-- Name: idx_client_profiles_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_profiles_type ON authenc.client_profiles USING btree (profile_type);


--
-- Name: idx_client_registration_audit_log_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_audit_log_client ON authenc.client_registration_audit_log USING btree (client_id, created_at DESC);


--
-- Name: idx_client_registration_audit_log_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_audit_log_realm ON authenc.client_registration_audit_log USING btree (realm_id, created_at DESC);


--
-- Name: idx_client_registration_audit_log_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_audit_log_type ON authenc.client_registration_audit_log USING btree (event_type, created_at DESC);


--
-- Name: idx_client_registration_policies_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_policies_realm ON authenc.client_registration_policies USING btree (realm_id) WHERE (enabled = true);


--
-- Name: idx_client_registration_tokens_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_tokens_client ON authenc.client_registration_tokens USING btree (client_id) WHERE (revoked = false);


--
-- Name: idx_client_registration_tokens_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_registration_tokens_hash ON authenc.client_registration_tokens USING btree (token_hash) WHERE (revoked = false);


--
-- Name: idx_client_scope_mappings_mapper; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scope_mappings_mapper ON authenc.client_scope_mappings USING btree (protocol_mapper_id);


--
-- Name: idx_client_scope_mappings_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scope_mappings_scope ON authenc.client_scope_mappings USING btree (scope_id);


--
-- Name: idx_client_scopes_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scopes_enabled ON authenc.client_scopes USING btree (realm_id, enabled) WHERE (enabled = true);


--
-- Name: idx_client_scopes_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scopes_name ON authenc.client_scopes USING btree (realm_id, name);


--
-- Name: idx_client_scopes_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scopes_protocol ON authenc.client_scopes USING btree (protocol);


--
-- Name: idx_client_scopes_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_client_scopes_realm ON authenc.client_scopes USING btree (realm_id);


--
-- Name: idx_credential_types_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_credential_types_code ON authenc.credential_types USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_custom_themes_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_custom_themes_active ON authenc.custom_themes USING btree (realm_id, theme_type) WHERE (is_active = true);


--
-- Name: idx_custom_themes_default; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_custom_themes_default ON authenc.custom_themes USING btree (theme_type) WHERE (is_default = true);


--
-- Name: idx_custom_themes_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_custom_themes_realm_id ON authenc.custom_themes USING btree (realm_id);


--
-- Name: idx_custom_themes_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_custom_themes_type ON authenc.custom_themes USING btree (theme_type);


--
-- Name: idx_dashboard_metrics_aggregation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dashboard_metrics_aggregation ON authenc.admin_dashboard_metrics USING btree (aggregation_period);


--
-- Name: idx_dashboard_metrics_period; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dashboard_metrics_period ON authenc.admin_dashboard_metrics USING btree (period_start, period_end);


--
-- Name: idx_dashboard_metrics_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dashboard_metrics_realm ON authenc.admin_dashboard_metrics USING btree (realm_id);


--
-- Name: idx_dashboard_metrics_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dashboard_metrics_type ON authenc.admin_dashboard_metrics USING btree (metric_type, metric_name);


--
-- Name: idx_device_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_active ON authenc.device_sessions USING btree (user_id) WHERE is_active;


--
-- Name: idx_device_sessions_device_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_device_id ON authenc.device_sessions USING btree (device_id);


--
-- Name: idx_device_sessions_last_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_last_activity ON authenc.device_sessions USING btree (last_activity) WHERE is_active;


--
-- Name: idx_device_sessions_risk_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_risk_score ON authenc.device_sessions USING btree (risk_score) WHERE (is_active AND (risk_score > (0.5)::double precision));


--
-- Name: idx_device_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_user_id ON authenc.device_sessions USING btree (user_id);


--
-- Name: idx_device_sessions_user_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_sessions_user_session_id ON authenc.device_sessions USING btree (user_session_id);


--
-- Name: idx_device_trust_history_device_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_trust_history_device_id ON authenc.device_trust_history USING btree (device_id);


--
-- Name: idx_devices_last_seen; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_devices_last_seen ON authenc.devices USING btree (last_seen_at);


--
-- Name: idx_devices_trust_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_devices_trust_score ON authenc.devices USING btree (trust_score);


--
-- Name: idx_devices_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_devices_user_id ON authenc.devices USING btree (user_id);


--
-- Name: idx_difficulty_adjustments_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_difficulty_adjustments_active ON authenc.captcha_difficulty_adjustments USING btree (active, created_at DESC);


--
-- Name: idx_difficulty_adjustments_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_difficulty_adjustments_ip ON authenc.captcha_difficulty_adjustments USING btree (ip_pattern) WHERE ((ip_pattern IS NOT NULL) AND (active = true));


--
-- Name: idx_difficulty_adjustments_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_difficulty_adjustments_session ON authenc.captcha_difficulty_adjustments USING btree (session_pattern) WHERE ((session_pattern IS NOT NULL) AND (active = true));


--
-- Name: idx_event_listeners_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_listeners_enabled ON authenc.event_listeners USING btree (realm_id, enabled) WHERE (enabled = true);


--
-- Name: idx_event_listeners_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_listeners_priority ON authenc.event_listeners USING btree (priority);


--
-- Name: idx_event_listeners_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_listeners_realm ON authenc.event_listeners USING btree (realm_id);


--
-- Name: idx_event_listeners_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_listeners_type ON authenc.event_listeners USING btree (listener_type);


--
-- Name: idx_event_log_category_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_category_time ON authenc.event_log USING btree (event_category, created_at DESC);


--
-- Name: idx_event_log_correlation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_correlation ON authenc.event_log USING btree (correlation_id);


--
-- Name: idx_event_log_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_created ON authenc.event_log USING btree (created_at DESC);


--
-- Name: idx_event_log_geolocation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_geolocation ON authenc.event_log USING gin (geolocation_data);


--
-- Name: idx_event_log_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_realm ON authenc.event_log USING btree (realm_id);


--
-- Name: idx_event_log_request_payload; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_request_payload ON authenc.event_log USING gin (request_payload);


--
-- Name: idx_event_log_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_resource ON authenc.event_log USING btree (resource_type, resource_id);


--
-- Name: idx_event_log_response_payload; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_response_payload ON authenc.event_log USING gin (response_payload);


--
-- Name: idx_event_log_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_type ON authenc.event_log USING btree (event_type, event_category);


--
-- Name: idx_event_log_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_log_user ON authenc.event_log USING btree (user_id);


--
-- Name: idx_events_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_client_id ON authenc.events USING btree (client_id);


--
-- Name: idx_events_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_event_type ON authenc.events USING btree (event_type);


--
-- Name: idx_events_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_realm_id ON authenc.events USING btree (realm_id);


--
-- Name: idx_events_realm_type_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_realm_type_time ON authenc.events USING btree (realm_id, event_type, "time" DESC);


--
-- Name: INDEX idx_events_realm_type_time; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_events_realm_type_time IS 'Improves audit queries by realm and event type';


--
-- Name: idx_events_signature; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_signature ON authenc.events USING btree (signature) WHERE (signature IS NOT NULL);


--
-- Name: idx_events_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_time ON authenc.events USING btree ("time" DESC);


--
-- Name: idx_events_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_user_id ON authenc.events USING btree (user_id);


--
-- Name: idx_events_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_events_user_time ON authenc.events USING btree (user_id, "time" DESC) WHERE (user_id IS NOT NULL);


--
-- Name: INDEX idx_events_user_time; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_events_user_time IS 'Improves audit trail queries by user';


--
-- Name: idx_federated_auth_log_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_auth_log_created ON authenc.federated_auth_log USING btree (created_at);


--
-- Name: idx_federated_auth_log_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_auth_log_provider ON authenc.federated_auth_log USING btree (identity_provider_alias);


--
-- Name: idx_federated_auth_log_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_auth_log_realm ON authenc.federated_auth_log USING btree (realm_id);


--
-- Name: idx_federated_auth_log_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_auth_log_user ON authenc.federated_auth_log USING btree (user_id);


--
-- Name: idx_federated_identities_provider_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_identities_provider_id ON authenc.federated_identities USING btree (identity_provider_id);


--
-- Name: idx_federated_identities_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_identities_user_id ON authenc.federated_identities USING btree (user_id);


--
-- Name: idx_federated_links_federated_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_links_federated_user ON authenc.federated_identity_links USING btree (identity_provider_alias, federated_user_id);


--
-- Name: idx_federated_links_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_links_provider ON authenc.federated_identity_links USING btree (identity_provider_alias);


--
-- Name: idx_federated_links_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_links_realm ON authenc.federated_identity_links USING btree (realm_id);


--
-- Name: idx_federated_links_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_links_user ON authenc.federated_identity_links USING btree (user_id);


--
-- Name: idx_group_attributes_group_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_attributes_group_id ON authenc.group_attributes USING btree (group_id);


--
-- Name: idx_group_attributes_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_attributes_name ON authenc.group_attributes USING btree (name);


--
-- Name: idx_group_attributes_value; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_attributes_value ON authenc.group_attributes USING btree (value);


--
-- Name: idx_group_roles_group_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_roles_group_id ON authenc.group_roles USING btree (group_id);


--
-- Name: idx_group_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_roles_role_id ON authenc.group_roles USING btree (role_id);


--
-- Name: idx_groups_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_groups_name ON authenc.groups USING btree (name);


--
-- Name: idx_groups_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_groups_parent_id ON authenc.groups USING btree (parent_id) WHERE (parent_id IS NOT NULL);


--
-- Name: idx_groups_path; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_groups_path ON authenc.groups USING btree (path);


--
-- Name: idx_groups_path_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_groups_path_gin ON authenc.groups USING gin (to_tsvector('english'::regconfig, path));


--
-- Name: idx_groups_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_groups_realm_id ON authenc.groups USING btree (realm_id);


--
-- Name: idx_identity_provider_mappers_provider_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_identity_provider_mappers_provider_id ON authenc.identity_provider_mappers USING btree (identity_provider_id);


--
-- Name: idx_identity_providers_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_identity_providers_enabled ON authenc.identity_providers USING btree (enabled) WHERE (deleted_at IS NULL);


--
-- Name: idx_identity_providers_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_identity_providers_realm_id ON authenc.identity_providers USING btree (realm_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_identity_providers_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_identity_providers_type ON authenc.identity_providers USING btree (provider_type) WHERE (deleted_at IS NULL);


--
-- Name: idx_idp_mappers_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_idp_mappers_provider ON authenc.identity_provider_mappers USING btree (identity_provider_alias);


--
-- Name: idx_idp_mappers_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_idp_mappers_realm ON authenc.identity_provider_mappers USING btree (realm_id);


--
-- Name: idx_idp_mappers_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_idp_mappers_type ON authenc.identity_provider_mappers USING btree (mapper_type);


--
-- Name: idx_initial_access_tokens_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_initial_access_tokens_hash ON authenc.initial_access_tokens USING btree (token_hash) WHERE (revoked = false);


--
-- Name: idx_initial_access_tokens_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_initial_access_tokens_realm ON authenc.initial_access_tokens USING btree (realm_id) WHERE (revoked = false);


--
-- Name: idx_key_rotation_audit_key_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_key_rotation_audit_key_id ON authenc.key_rotation_audit USING btree (key_id);


--
-- Name: idx_key_rotation_audit_key_id_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_key_rotation_audit_key_id_timestamp ON authenc.key_rotation_audit USING btree (key_id, "timestamp" DESC);


--
-- Name: idx_key_rotation_audit_key_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_key_rotation_audit_key_type ON authenc.key_rotation_audit USING btree (key_type);


--
-- Name: idx_key_rotation_audit_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_key_rotation_audit_status ON authenc.key_rotation_audit USING btree (status);


--
-- Name: idx_key_rotation_audit_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_key_rotation_audit_timestamp ON authenc.key_rotation_audit USING btree ("timestamp" DESC);


--
-- Name: idx_listener_executions_event; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_listener_executions_event ON authenc.event_listener_executions USING btree (event_log_id);


--
-- Name: idx_listener_executions_listener; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_listener_executions_listener ON authenc.event_listener_executions USING btree (listener_id);


--
-- Name: idx_listener_executions_retry; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_listener_executions_retry ON authenc.event_listener_executions USING btree (next_retry_at) WHERE ((next_retry_at IS NOT NULL) AND (success = false));


--
-- Name: idx_listener_executions_success; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_listener_executions_success ON authenc.event_listener_executions USING btree (success);


--
-- Name: idx_mfa_admin_actions_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_action ON authenc.mfa_admin_actions USING btree (action);


--
-- Name: idx_mfa_admin_actions_action_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_action_created ON authenc.mfa_admin_actions USING btree (action, created_at DESC);


--
-- Name: idx_mfa_admin_actions_admin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_admin ON authenc.mfa_admin_actions USING btree (admin_user_id, created_at DESC);


--
-- Name: idx_mfa_admin_actions_admin_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_admin_created ON authenc.mfa_admin_actions USING btree (admin_user_id, created_at DESC);


--
-- Name: idx_mfa_admin_actions_admin_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_admin_user_id ON authenc.mfa_admin_actions USING btree (admin_user_id);


--
-- Name: idx_mfa_admin_actions_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_created_at ON authenc.mfa_admin_actions USING btree (created_at);


--
-- Name: idx_mfa_admin_actions_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_target ON authenc.mfa_admin_actions USING btree (target_user_id, created_at DESC);


--
-- Name: idx_mfa_admin_actions_target_action_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_target_action_created ON authenc.mfa_admin_actions USING btree (target_user_id, action, created_at DESC);


--
-- Name: idx_mfa_admin_actions_target_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_admin_actions_target_user_id ON authenc.mfa_admin_actions USING btree (target_user_id);


--
-- Name: idx_mfa_policies_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_policies_active ON authenc.mfa_policies USING btree (active, created_at DESC);


--
-- Name: idx_mfa_statistics_satker; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_mfa_statistics_satker ON authenc.mfa_statistics USING btree (satker_code);


--
-- Name: idx_oauth2_access_tokens_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_access_tokens_expires ON authenc.oauth2_access_tokens USING btree (expires_at);


--
-- Name: idx_oauth2_access_tokens_refresh; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_access_tokens_refresh ON authenc.oauth2_access_tokens USING btree (refresh_token_hash);


--
-- Name: idx_oauth2_access_tokens_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_access_tokens_token ON authenc.oauth2_access_tokens USING btree (token_hash);


--
-- Name: idx_oauth2_authorization_codes_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_authorization_codes_code ON authenc.oauth2_authorization_codes USING btree (code);


--
-- Name: idx_oauth2_authorization_codes_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_authorization_codes_expires ON authenc.oauth2_authorization_codes USING btree (expires_at);


--
-- Name: idx_oauth2_clients_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_clients_client_id ON authenc.oauth2_clients USING btree (client_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_oauth2_clients_registration_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_clients_registration_token ON authenc.oauth2_clients USING btree (registration_access_token_hash) WHERE (deleted_at IS NULL);


--
-- Name: idx_oauth2_configs_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_configs_enabled ON authenc.oauth2_provider_configs USING btree (realm_id, enabled) WHERE (enabled = true);


--
-- Name: idx_oauth2_configs_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_configs_provider ON authenc.oauth2_provider_configs USING btree (provider_name);


--
-- Name: idx_oauth2_configs_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_configs_realm ON authenc.oauth2_provider_configs USING btree (realm_id);


--
-- Name: idx_oauth2_exchanges_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_exchanges_created ON authenc.oauth2_token_exchanges USING btree (created_at);


--
-- Name: idx_oauth2_exchanges_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_exchanges_provider ON authenc.oauth2_token_exchanges USING btree (provider_config_id);


--
-- Name: idx_oauth2_exchanges_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_exchanges_user ON authenc.oauth2_token_exchanges USING btree (user_id);


--
-- Name: idx_oauth2_states_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_states_expires ON authenc.oauth2_states USING btree (expires_at) WHERE (NOT used);


--
-- Name: idx_oauth2_states_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_states_provider ON authenc.oauth2_states USING btree (provider_config_id);


--
-- Name: idx_oauth2_states_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_states_token ON authenc.oauth2_states USING btree (state_token);


--
-- Name: idx_oauth2_tokens_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth2_tokens_actor ON authenc.oauth2_access_tokens USING btree (actor_id) WHERE (actor_id IS NOT NULL);


--
-- Name: idx_offline_tokens_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_offline_tokens_active ON authenc.offline_tokens USING btree (user_id, realm_id) WHERE (NOT revoked);


--
-- Name: idx_offline_tokens_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_offline_tokens_client_id ON authenc.offline_tokens USING btree (client_id);


--
-- Name: idx_offline_tokens_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_offline_tokens_realm_id ON authenc.offline_tokens USING btree (realm_id);


--
-- Name: idx_offline_tokens_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_offline_tokens_token_hash ON authenc.offline_tokens USING btree (token_hash);


--
-- Name: idx_offline_tokens_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_offline_tokens_user_id ON authenc.offline_tokens USING btree (user_id);


--
-- Name: idx_org_domains_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_domains_domain ON authenc.organization_domains USING btree (domain);


--
-- Name: idx_org_domains_org_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_domains_org_id ON authenc.organization_domains USING btree (organization_id);


--
-- Name: idx_org_domains_verified; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_domains_verified ON authenc.organization_domains USING btree (verified);


--
-- Name: idx_org_idps_org_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_idps_org_id ON authenc.organization_identity_providers USING btree (organization_id);


--
-- Name: idx_org_invitations_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_invitations_email ON authenc.organization_invitations USING btree (email);


--
-- Name: idx_org_invitations_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_invitations_token ON authenc.organization_invitations USING btree (token_hash);


--
-- Name: idx_org_members_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_org_members_user_id ON authenc.organization_members USING btree (user_id);


--
-- Name: idx_organization_invitations_org_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_invitations_org_id ON authenc.organization_invitations USING btree (organization_id);


--
-- Name: idx_organization_invitations_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_invitations_token ON authenc.organization_invitations USING btree (token_hash);


--
-- Name: idx_organization_members_org_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_members_org_id ON authenc.organization_members USING btree (organization_id);


--
-- Name: idx_organization_members_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_members_user_id ON authenc.organization_members USING btree (user_id);


--
-- Name: idx_organizations_owner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organizations_owner_id ON authenc.organizations USING btree (owner_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_password_history_user_id_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_history_user_id_created ON authenc.password_history USING btree (user_id, created_at DESC);


--
-- Name: idx_performance_metrics_latency; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_performance_metrics_latency ON authenc.captcha_performance_metrics USING btree (challenge_generation_latency_ms, validation_latency_ms);


--
-- Name: idx_performance_metrics_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_performance_metrics_timestamp ON authenc.captcha_performance_metrics USING btree ("timestamp" DESC);


--
-- Name: idx_permission_tickets_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_permission_tickets_owner ON authenc.permission_tickets USING btree (owner);


--
-- Name: idx_permission_tickets_requester; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_permission_tickets_requester ON authenc.permission_tickets USING btree (requester);


--
-- Name: idx_permission_tickets_resource_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_permission_tickets_resource_id ON authenc.permission_tickets USING btree (resource_id);


--
-- Name: idx_protocol_mappers_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_client ON authenc.protocol_mappers USING btree (client_id);


--
-- Name: idx_protocol_mappers_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_enabled ON authenc.protocol_mappers USING btree (client_id, enabled) WHERE (enabled = true);


--
-- Name: idx_protocol_mappers_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_protocol ON authenc.protocol_mappers USING btree (protocol);


--
-- Name: idx_protocol_mappers_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_realm ON authenc.protocol_mappers USING btree (realm_id);


--
-- Name: idx_protocol_mappers_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_scope ON authenc.protocol_mappers USING btree (client_scope_id);


--
-- Name: idx_protocol_mappers_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_mappers_type ON authenc.protocol_mappers USING btree (mapper_type);


--
-- Name: idx_realm_theme_settings_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_realm_theme_settings_realm_id ON authenc.realm_theme_settings USING btree (realm_id);


--
-- Name: idx_realms_name_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_realms_name_unique ON authenc.realms USING btree (name) WHERE (deleted_at IS NULL);


--
-- Name: idx_refresh_token_history_rotated_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_token_history_rotated_at ON authenc.refresh_token_history USING btree (rotated_at);


--
-- Name: idx_refresh_token_history_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_token_history_session_id ON authenc.refresh_token_history USING btree (user_session_id);


--
-- Name: idx_refresh_token_history_suspicious; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_token_history_suspicious ON authenc.refresh_token_history USING btree (user_session_id) WHERE suspicious;


--
-- Name: idx_resource_servers_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_servers_client_id ON authenc.resource_servers USING btree (client_id);


--
-- Name: idx_resource_servers_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resource_servers_realm_id ON authenc.resource_servers USING btree (realm_id);


--
-- Name: idx_resources_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_name ON authenc.resources USING btree (name);


--
-- Name: idx_resources_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_owner ON authenc.resources USING btree (owner);


--
-- Name: idx_resources_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_realm_id ON authenc.resources USING btree (realm_id);


--
-- Name: idx_resources_resource_server_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_resources_resource_server_id ON authenc.resources USING btree (resource_server_id);


--
-- Name: idx_role_capabilities_capability_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_capabilities_capability_id ON authenc.role_capabilities USING btree (capability_id);


--
-- Name: idx_role_capabilities_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_capabilities_role_id ON authenc.role_capabilities USING btree (role_id);


--
-- Name: idx_role_hierarchy_child; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_hierarchy_child ON authenc.role_hierarchy USING btree (child_role_id);


--
-- Name: idx_role_hierarchy_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_hierarchy_parent ON authenc.role_hierarchy USING btree (parent_role_id);


--
-- Name: idx_role_permissions_resource; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_permissions_resource ON authenc.role_permissions USING btree (resource);


--
-- Name: idx_role_permissions_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_permissions_role ON authenc.role_permissions USING btree (role_id);


--
-- Name: idx_role_policies_policy_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_policies_policy_id ON authenc.role_policies USING btree (policy_id);


--
-- Name: idx_role_policies_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_policies_role_id ON authenc.role_policies USING btree (role_id);


--
-- Name: idx_role_types_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_types_category ON authenc.role_types USING btree (category) WHERE (deleted_at IS NULL);


--
-- Name: idx_role_types_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_types_code ON authenc.role_types USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_role_types_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_role_types_realm_id ON authenc.role_types USING btree (realm_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_roles_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_roles_realm_id ON authenc.roles USING btree (realm_id);


--
-- Name: idx_saml_assertion_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_assertion_expires ON authenc.saml_assertion_cache USING btree (expires_at);


--
-- Name: idx_saml_idp_entity_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_idp_entity_id ON authenc.saml_identity_providers USING btree (entity_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_saml_messages_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_messages_expires_at ON authenc.saml_messages USING btree (expires_at);


--
-- Name: idx_saml_messages_saml_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_messages_saml_id ON authenc.saml_messages USING btree (saml_id);


--
-- Name: idx_saml_messages_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_messages_session_id ON authenc.saml_messages USING btree (session_id) WHERE (session_id IS NOT NULL);


--
-- Name: idx_saml_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_sessions_expires ON authenc.saml_sessions USING btree (expires_at);


--
-- Name: idx_saml_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_sessions_user_id ON authenc.saml_sessions USING btree (user_id);


--
-- Name: idx_saml_sp_entity_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_saml_sp_entity_id ON authenc.saml_service_providers USING btree (entity_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_satker_admin_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_admin_expires ON authenc.satker_admin_roles USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_satker_admin_level; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_admin_level ON authenc.satker_admin_roles USING btree (admin_level) WHERE (active = true);


--
-- Name: idx_satker_admin_roles_level_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_admin_roles_level_code ON authenc.satker_admin_roles USING btree (admin_level);


--
-- Name: idx_satker_admin_satker; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_admin_satker ON authenc.satker_admin_roles USING btree (satker_code) WHERE (active = true);


--
-- Name: idx_satker_admin_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_admin_user ON authenc.satker_admin_roles USING btree (user_id) WHERE (active = true);


--
-- Name: idx_satker_audit_operation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_audit_operation ON authenc.satker_audit_logs USING btree (operation);


--
-- Name: idx_satker_audit_satker; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_audit_satker ON authenc.satker_audit_logs USING btree (satker_code);


--
-- Name: idx_satker_audit_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_audit_timestamp ON authenc.satker_audit_logs USING btree ("timestamp" DESC);


--
-- Name: idx_satker_audit_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_audit_user ON authenc.satker_audit_logs USING btree (user_id);


--
-- Name: idx_satker_perm_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_perm_expires ON authenc.satker_permissions USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_satker_perm_satker; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_perm_satker ON authenc.satker_permissions USING btree (satker_code);


--
-- Name: idx_satker_perm_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_perm_type ON authenc.satker_permissions USING btree (permission_type);


--
-- Name: idx_satker_perm_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_perm_user ON authenc.satker_permissions USING btree (user_id);


--
-- Name: idx_satker_types_hierarchy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_types_hierarchy ON authenc.satker_types USING btree (hierarchy_level DESC);


--
-- Name: idx_satker_types_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satker_types_parent ON authenc.satker_types USING btree (parent_type_id);


--
-- Name: idx_satkers_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_active ON authenc.satkers USING btree (active);


--
-- Name: idx_satkers_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_code ON authenc.satkers USING btree (code) WHERE (active = true);


--
-- Name: idx_satkers_hierarchy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_hierarchy ON authenc.satkers USING btree (parent_code, level, code) WHERE (active = true);


--
-- Name: idx_satkers_level; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_level ON authenc.satkers USING btree (level) WHERE (active = true);


--
-- Name: idx_satkers_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_name ON authenc.satkers USING gin (to_tsvector('indonesian'::regconfig, (name)::text));


--
-- Name: idx_satkers_parent_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_satkers_parent_code ON authenc.satkers USING btree (parent_code) WHERE (active = true);


--
-- Name: idx_scope_types_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scope_types_code ON authenc.scope_types USING btree (code) WHERE (deleted_at IS NULL);


--
-- Name: idx_scope_types_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scope_types_parent ON authenc.scope_types USING btree (parent_scope_type_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_scopes_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scopes_name ON authenc.scopes USING btree (name);


--
-- Name: idx_scopes_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scopes_realm_id ON authenc.scopes USING btree (realm_id);


--
-- Name: idx_scopes_resource_server_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scopes_resource_server_id ON authenc.scopes USING btree (resource_server_id);


--
-- Name: idx_security_event_metrics_attacks; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_event_metrics_attacks ON authenc.captcha_security_event_metrics USING btree (attack_attempts DESC, "timestamp" DESC);


--
-- Name: idx_security_event_metrics_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_event_metrics_timestamp ON authenc.captcha_security_event_metrics USING btree ("timestamp" DESC);


--
-- Name: idx_service_account_audit_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_audit_event_type ON authenc.service_account_audit_log USING btree (event_type);


--
-- Name: idx_service_account_audit_performed_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_audit_performed_by ON authenc.service_account_audit_log USING btree (performed_by) WHERE (performed_by IS NOT NULL);


--
-- Name: idx_service_account_audit_sa_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_audit_sa_id ON authenc.service_account_audit_log USING btree (service_account_id);


--
-- Name: idx_service_account_audit_success; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_audit_success ON authenc.service_account_audit_log USING btree (success) WHERE (success = false);


--
-- Name: idx_service_account_audit_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_audit_timestamp ON authenc.service_account_audit_log USING btree ("timestamp" DESC);


--
-- Name: idx_service_account_roles_granted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_roles_granted_at ON authenc.service_account_roles USING btree (granted_at DESC);


--
-- Name: idx_service_account_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_roles_role_id ON authenc.service_account_roles USING btree (role_id);


--
-- Name: idx_service_account_roles_sa_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_account_roles_sa_id ON authenc.service_account_roles USING btree (service_account_id);


--
-- Name: idx_service_accounts_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_accounts_client_id ON authenc.service_accounts USING btree (client_id) WHERE (enabled = true);


--
-- Name: idx_service_accounts_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_accounts_enabled ON authenc.service_accounts USING btree (enabled);


--
-- Name: idx_service_accounts_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_accounts_last_used ON authenc.service_accounts USING btree (last_used_at DESC NULLS LAST) WHERE (enabled = true);


--
-- Name: idx_service_accounts_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_accounts_name ON authenc.service_accounts USING gin (to_tsvector('english'::regconfig, (name)::text));


--
-- Name: idx_service_accounts_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_accounts_realm_id ON authenc.service_accounts USING btree (realm_id) WHERE (enabled = true);


--
-- Name: idx_sessions_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_expires_at ON authenc.sessions USING btree (expires_at);


--
-- Name: idx_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_user_id ON authenc.sessions USING btree (user_id);


--
-- Name: idx_social_login_configs_oauth; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_social_login_configs_oauth ON authenc.social_login_configs USING btree (oauth2_config_id);


--
-- Name: idx_social_login_configs_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_social_login_configs_type ON authenc.social_login_configs USING btree (provider_type);


--
-- Name: idx_software_statement_issuers_issuer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_software_statement_issuers_issuer ON authenc.software_statement_issuers USING btree (issuer) WHERE (enabled = true);


--
-- Name: idx_theme_inheritance_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_inheritance_parent_id ON authenc.theme_inheritance USING btree (parent_theme_id);


--
-- Name: idx_theme_inheritance_theme_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_inheritance_theme_id ON authenc.theme_inheritance USING btree (theme_id);


--
-- Name: idx_theme_resources_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_resources_name ON authenc.theme_resources USING btree (theme_id, resource_name);


--
-- Name: idx_theme_resources_theme_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_resources_theme_id ON authenc.theme_resources USING btree (theme_id);


--
-- Name: idx_theme_resources_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_resources_type ON authenc.theme_resources USING btree (resource_type);


--
-- Name: idx_theme_templates_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_templates_name ON authenc.theme_templates USING btree (theme_id, template_name);


--
-- Name: idx_theme_templates_theme_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_templates_theme_id ON authenc.theme_templates USING btree (theme_id);


--
-- Name: idx_theme_templates_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_theme_templates_type ON authenc.theme_templates USING btree (template_type);


--
-- Name: idx_token_exchange_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_actor ON authenc.token_exchange_audit USING btree (actor_id, created_at DESC) WHERE (actor_id IS NOT NULL);


--
-- Name: idx_token_exchange_audience; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_audience ON authenc.token_exchange_audit USING btree (audience) WHERE (audience IS NOT NULL);


--
-- Name: idx_token_exchange_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_client ON authenc.token_exchange_audit USING btree (target_client_id, created_at DESC);


--
-- Name: idx_token_exchange_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_created_at ON authenc.token_exchange_audit USING btree (created_at DESC);


--
-- Name: idx_token_exchange_metadata; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_metadata ON authenc.token_exchange_audit USING gin (metadata) WHERE (metadata IS NOT NULL);


--
-- Name: idx_token_exchange_subject_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_subject_user ON authenc.token_exchange_audit USING btree (subject_user_id, created_at DESC);


--
-- Name: idx_token_exchange_success; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_token_exchange_success ON authenc.token_exchange_audit USING btree (success, created_at DESC);


--
-- Name: idx_type_effectiveness_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_type_effectiveness_score ON authenc.captcha_type_effectiveness USING btree (effectiveness_score DESC);


--
-- Name: idx_type_effectiveness_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_type_effectiveness_type ON authenc.captcha_type_effectiveness USING btree (challenge_type);


--
-- Name: idx_type_effectiveness_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_type_effectiveness_updated ON authenc.captcha_type_effectiveness USING btree (last_updated DESC);


--
-- Name: idx_uma_permission_requests_resource_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_uma_permission_requests_resource_id ON authenc.uma_permission_requests USING btree (resource_id);


--
-- Name: idx_uma_permission_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_uma_permission_requests_status ON authenc.uma_permission_requests USING btree (status);


--
-- Name: idx_uma_permission_requests_ticket; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_uma_permission_requests_ticket ON authenc.uma_permission_requests USING btree (ticket);


--
-- Name: idx_uma_policies_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_uma_policies_realm_id ON authenc.uma_policies USING btree (realm_id);


--
-- Name: idx_uma_policies_resource_server_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_uma_policies_resource_server_id ON authenc.uma_policies USING btree (resource_server_id);


--
-- Name: idx_user_attributes_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_attributes_name ON authenc.user_attributes USING btree (name);


--
-- Name: idx_user_attributes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_attributes_user ON authenc.user_attributes USING btree (user_id);


--
-- Name: idx_user_consent_scopes_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_consent_scopes_client ON authenc.user_consent_scopes USING btree (client_id);


--
-- Name: idx_user_consent_scopes_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_consent_scopes_expires ON authenc.user_consent_scopes USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_user_consent_scopes_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_consent_scopes_scope ON authenc.user_consent_scopes USING btree (scope_id);


--
-- Name: idx_user_consent_scopes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_consent_scopes_user ON authenc.user_consent_scopes USING btree (user_id);


--
-- Name: idx_user_experience_metrics_satisfaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_experience_metrics_satisfaction ON authenc.captcha_user_experience_metrics USING btree (user_satisfaction_score DESC, "timestamp" DESC);


--
-- Name: idx_user_experience_metrics_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_experience_metrics_timestamp ON authenc.captcha_user_experience_metrics USING btree ("timestamp" DESC);


--
-- Name: idx_user_groups_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_groups_active ON authenc.user_groups USING btree (user_id, group_id) WHERE (expires_at IS NULL);


--
-- Name: idx_user_groups_group_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_groups_group_id ON authenc.user_groups USING btree (group_id);


--
-- Name: idx_user_groups_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_groups_user_id ON authenc.user_groups USING btree (user_id);


--
-- Name: idx_user_history_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_history_session_id ON authenc.captcha_user_history USING btree (session_id) WHERE (session_id IS NOT NULL);


--
-- Name: idx_user_history_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_history_updated ON authenc.captcha_user_history USING btree (updated_at DESC);


--
-- Name: idx_user_history_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_history_user_id ON authenc.captcha_user_history USING btree (user_id);


--
-- Name: idx_user_policies_policy_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_policies_policy_id ON authenc.user_policies USING btree (policy_id);


--
-- Name: idx_user_policies_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_policies_user_id ON authenc.user_policies USING btree (user_id);


--
-- Name: idx_user_roles_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_roles_role ON authenc.user_roles USING btree (role_id);


--
-- Name: idx_user_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_roles_role_id ON authenc.user_roles USING btree (role_id);


--
-- Name: idx_user_roles_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_roles_user_id ON authenc.user_roles USING btree (user_id);


--
-- Name: idx_user_roles_user_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_roles_user_role ON authenc.user_roles USING btree (user_id, role_id);


--
-- Name: idx_user_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_active ON authenc.user_sessions USING btree (user_id, realm_id) WHERE (NOT revoked);


--
-- Name: idx_user_sessions_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_client_id ON authenc.user_sessions USING btree (client_id);


--
-- Name: idx_user_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_expires ON authenc.user_sessions USING btree (expires_at);


--
-- Name: idx_user_sessions_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_expires_at ON authenc.user_sessions USING btree (expires_at) WHERE (NOT revoked);


--
-- Name: idx_user_sessions_idle_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_idle_expires_at ON authenc.user_sessions USING btree (idle_expires_at) WHERE ((NOT revoked) AND (idle_expires_at IS NOT NULL));


--
-- Name: idx_user_sessions_offline_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_offline_token_hash ON authenc.user_sessions USING btree (offline_token_hash) WHERE (offline_token_hash IS NOT NULL);


--
-- Name: idx_user_sessions_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_realm_id ON authenc.user_sessions USING btree (realm_id);


--
-- Name: idx_user_sessions_refresh_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_refresh_token_hash ON authenc.user_sessions USING btree (refresh_token_hash) WHERE (refresh_token_hash IS NOT NULL);


--
-- Name: idx_user_sessions_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_session_id ON authenc.user_sessions USING btree (session_id);


--
-- Name: idx_user_sessions_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_token_hash ON authenc.user_sessions USING btree (token_hash);


--
-- Name: idx_user_sessions_user_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_user_expires ON authenc.user_sessions USING btree (user_id, expires_at) WHERE (NOT revoked);


--
-- Name: INDEX idx_user_sessions_user_expires; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_user_sessions_user_expires IS 'Improves active session queries';


--
-- Name: idx_user_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_user_id ON authenc.user_sessions USING btree (user_id);


--
-- Name: idx_user_type_perf_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_type_perf_type ON authenc.captcha_user_type_performance USING btree (challenge_type, success_rate DESC);


--
-- Name: idx_user_type_perf_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_type_perf_updated ON authenc.captcha_user_type_performance USING btree (updated_at DESC);


--
-- Name: idx_user_type_perf_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_type_perf_user ON authenc.captcha_user_type_performance USING btree (user_id, success_rate DESC);


--
-- Name: idx_users_account_locked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_account_locked ON authenc.users USING btree (account_locked) WHERE (account_locked = true);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email ON authenc.users USING btree (email) WHERE (deleted_at IS NULL);


--
-- Name: INDEX idx_users_email; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_email IS 'Improves user lookup by email (authentication)';


--
-- Name: idx_users_email_mfa_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email_mfa_enabled ON authenc.users USING btree (email, mfa_enabled);


--
-- Name: idx_users_email_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_users_email_unique ON authenc.users USING btree (email) WHERE (deleted_at IS NULL);


--
-- Name: idx_users_mfa_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_enabled ON authenc.users USING btree (mfa_enabled);


--
-- Name: idx_users_mfa_enabled_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_enabled_last_used ON authenc.users USING btree (mfa_enabled, mfa_last_used) WHERE (mfa_enabled = true);


--
-- Name: INDEX idx_users_mfa_enabled_last_used; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_mfa_enabled_last_used IS 'Composite index for MFA-enabled users with last used timestamp';


--
-- Name: idx_users_mfa_enabled_setup_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_enabled_setup_at ON authenc.users USING btree (mfa_enabled, mfa_setup_at) WHERE (mfa_enabled = true);


--
-- Name: INDEX idx_users_mfa_enabled_setup_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_mfa_enabled_setup_at IS 'Composite index for MFA-enabled users with setup timestamp';


--
-- Name: idx_users_mfa_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_last_used ON authenc.users USING btree (mfa_last_used);


--
-- Name: idx_users_mfa_recent_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_recent_activity ON authenc.users USING btree (mfa_last_used DESC) WHERE ((mfa_enabled = true) AND (mfa_last_used IS NOT NULL));


--
-- Name: INDEX idx_users_mfa_recent_activity; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_mfa_recent_activity IS 'Index for finding recently active MFA users';


--
-- Name: idx_users_mfa_setup_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_setup_at ON authenc.users USING btree (mfa_setup_at);


--
-- Name: idx_users_mfa_setup_required; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_mfa_setup_required ON authenc.users USING btree (mfa_enabled, mfa_setup_at) WHERE ((mfa_enabled = true) AND (mfa_setup_at IS NULL));


--
-- Name: INDEX idx_users_mfa_setup_required; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_mfa_setup_required IS 'Partial index for users requiring MFA setup';


--
-- Name: idx_users_nama; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_nama ON authenc.users USING btree (nama) WHERE ((nama IS NOT NULL) AND (deleted_at IS NULL));


--
-- Name: idx_users_nip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_nip ON authenc.users USING btree (nip) WHERE ((nip IS NOT NULL) AND (deleted_at IS NULL));


--
-- Name: idx_users_organization_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_organization_id ON authenc.users USING btree (organization_id) WHERE ((organization_id IS NOT NULL) AND (deleted_at IS NULL));


--
-- Name: idx_users_realm_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_realm_id ON authenc.users USING btree (realm_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_users_realm_satker; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_realm_satker ON authenc.users USING btree (realm_id, satker_code) WHERE (deleted_at IS NULL);


--
-- Name: INDEX idx_users_realm_satker; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_realm_satker IS 'Composite index for realm + satker filtering';


--
-- Name: idx_users_require_mfa_setup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_require_mfa_setup ON authenc.users USING btree (require_mfa_setup) WHERE (require_mfa_setup = true);


--
-- Name: idx_users_satker_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_satker_code ON authenc.users USING btree (satker_code) WHERE (deleted_at IS NULL);


--
-- Name: INDEX idx_users_satker_code; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_satker_code IS 'Improves user filtering by organizational unit';


--
-- Name: idx_users_satker_mfa_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_satker_mfa_enabled ON authenc.users USING btree (satker_code, mfa_enabled);


--
-- Name: INDEX idx_users_satker_mfa_enabled; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_satker_mfa_enabled IS 'Index for satker-based MFA reporting';


--
-- Name: idx_users_username; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_username ON authenc.users USING btree (username) WHERE (deleted_at IS NULL);


--
-- Name: INDEX idx_users_username; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_username IS 'Improves user lookup by username (authentication)';


--
-- Name: idx_users_username_mfa_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_username_mfa_enabled ON authenc.users USING btree (username, mfa_enabled);


--
-- Name: INDEX idx_users_username_mfa_enabled; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX authenc.idx_users_username_mfa_enabled IS 'Composite index for authentication flow optimization';


--
-- Name: idx_users_username_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_users_username_unique ON authenc.users USING btree (username) WHERE (deleted_at IS NULL);


--
-- Name: idx_validation_attempts_challenge; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_validation_attempts_challenge ON authenc.captcha_validation_attempts USING btree (challenge_id, created_at DESC);


--
-- Name: idx_validation_attempts_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_validation_attempts_ip ON authenc.captcha_validation_attempts USING btree (ip_address, created_at DESC);


--
-- Name: idx_validation_attempts_risk; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_validation_attempts_risk ON authenc.captcha_validation_attempts USING btree (risk_assessment, created_at DESC);


--
-- Name: idx_validation_attempts_success; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_validation_attempts_success ON authenc.captcha_validation_attempts USING btree (success, created_at DESC);


--
-- Name: idx_webauthn_audit_log_credential_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_audit_log_credential_id ON authenc.webauthn_audit_log USING btree (credential_id) WHERE (credential_id IS NOT NULL);


--
-- Name: idx_webauthn_audit_log_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_audit_log_event_type ON authenc.webauthn_audit_log USING btree (event_type, created_at DESC);


--
-- Name: idx_webauthn_audit_log_user_id_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_audit_log_user_id_created_at ON authenc.webauthn_audit_log USING btree (user_id, created_at DESC);


--
-- Name: idx_webauthn_challenges_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_challenges_expires_at ON authenc.webauthn_challenges USING btree (expires_at) WHERE (used = false);


--
-- Name: idx_webauthn_challenges_user_id_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_challenges_user_id_type ON authenc.webauthn_challenges USING btree (user_id, challenge_type, used, expires_at);


--
-- Name: idx_webauthn_credentials_aaguid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_aaguid ON authenc.webauthn_credentials_old USING btree (aaguid) WHERE (aaguid IS NOT NULL);


--
-- Name: idx_webauthn_credentials_attestation_format; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_attestation_format ON authenc.webauthn_credentials_old USING btree (attestation_format) WHERE (attestation_format IS NOT NULL);


--
-- Name: idx_webauthn_credentials_backup_flags; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_backup_flags ON authenc.webauthn_credentials_old USING btree (backup_eligible, backup_state);


--
-- Name: idx_webauthn_credentials_cred_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_cred_id ON authenc.webauthn_credentials USING btree (cred_id);


--
-- Name: idx_webauthn_credentials_credential_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_credential_id ON authenc.webauthn_credentials_old USING btree (credential_id);


--
-- Name: idx_webauthn_credentials_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_last_used ON authenc.webauthn_credentials USING btree (last_used DESC NULLS LAST);


--
-- Name: idx_webauthn_credentials_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_user_id ON authenc.webauthn_credentials USING btree (user_id);


--
-- Name: idx_webhooks_listener; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webhooks_listener ON authenc.event_webhooks USING btree (listener_id);


--
-- Name: idx_webhooks_realm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webhooks_realm ON authenc.event_webhooks USING btree (realm_id);


--
-- Name: v_client_policies_with_profiles _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW authenc.v_client_policies_with_profiles AS
 SELECT cp.id,
    cp.realm_id,
    cp.name,
    cp.description,
    cp.enabled,
    cp.conditions,
    cp.condition_config,
    cp.executors,
    cp.executor_config,
    cp.priority,
    cp.policy_type,
    cp.created_at,
    cp.updated_at,
    cp.created_by,
    COALESCE(json_agg(json_build_object('profile_id', prof.id, 'profile_name', prof.name, 'profile_type', prof.profile_type)) FILTER (WHERE (prof.id IS NOT NULL)), '[]'::json) AS applied_profiles
   FROM (authenc.client_policies cp
     LEFT JOIN authenc.client_profiles prof ON ((cp.id = ANY (prof.policy_ids))))
  GROUP BY cp.id;


--
-- Name: v_client_profile_usage _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW authenc.v_client_profile_usage AS
 SELECT prof.id AS profile_id,
    prof.realm_id,
    prof.name AS profile_name,
    prof.profile_type,
    prof.is_builtin,
    count(DISTINCT cpa.client_id) AS assigned_clients,
    array_length(prof.policy_ids, 1) AS policy_count
   FROM (authenc.client_profiles prof
     LEFT JOIN authenc.client_policy_assignments cpa ON (((cpa.profile_id = prof.id) AND (cpa.enabled = true))))
  GROUP BY prof.id;


--
-- Name: v_client_profiles_with_policies _RETURN; Type: RULE; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW authenc.v_client_profiles_with_policies AS
 SELECT prof.id,
    prof.realm_id,
    prof.name,
    prof.description,
    prof.enabled,
    prof.policy_ids,
    prof.profile_type,
    prof.is_builtin,
    prof.created_at,
    prof.updated_at,
    prof.created_by,
    COALESCE(json_agg(json_build_object('policy_id', cp.id, 'policy_name', cp.name, 'policy_type', cp.policy_type, 'priority', cp.priority, 'enabled', cp.enabled) ORDER BY cp.priority DESC) FILTER (WHERE (cp.id IS NOT NULL)), '[]'::json) AS policies
   FROM (authenc.client_profiles prof
     LEFT JOIN authenc.client_policies cp ON ((cp.id = ANY (prof.policy_ids))))
  GROUP BY prof.id;


--
-- Name: groups trg_prevent_circular_groups; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_prevent_circular_groups BEFORE INSERT OR UPDATE OF parent_id ON authenc.groups FOR EACH ROW EXECUTE FUNCTION authenc.prevent_circular_groups();


--
-- Name: groups trg_update_child_group_paths; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_update_child_group_paths AFTER UPDATE OF path ON authenc.groups FOR EACH ROW WHEN ((old.path IS DISTINCT FROM new.path)) EXECUTE FUNCTION authenc.update_child_group_paths();


--
-- Name: groups trg_update_group_path; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_update_group_path BEFORE INSERT OR UPDATE OF name, parent_id ON authenc.groups FOR EACH ROW EXECUTE FUNCTION authenc.update_group_path();


--
-- Name: service_accounts trigger_audit_service_account_changes; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_audit_service_account_changes AFTER INSERT OR DELETE OR UPDATE ON authenc.service_accounts FOR EACH ROW EXECUTE FUNCTION authenc.audit_service_account_changes();


--
-- Name: service_account_roles trigger_audit_service_account_role_changes; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_audit_service_account_role_changes AFTER INSERT OR DELETE ON authenc.service_account_roles FOR EACH ROW EXECUTE FUNCTION authenc.audit_service_account_role_changes();


--
-- Name: captcha_challenges trigger_captcha_analytics_refresh; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_captcha_analytics_refresh AFTER INSERT OR DELETE OR UPDATE ON authenc.captcha_challenges FOR EACH STATEMENT EXECUTE FUNCTION authenc.trigger_refresh_captcha_analytics();


--
-- Name: client_policies trigger_client_policies_updated; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_client_policies_updated BEFORE UPDATE ON authenc.client_policies FOR EACH ROW EXECUTE FUNCTION authenc.update_client_policy_timestamp();


--
-- Name: client_profiles trigger_client_profiles_updated; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_client_profiles_updated BEFORE UPDATE ON authenc.client_profiles FOR EACH ROW EXECUTE FUNCTION authenc.update_client_policy_timestamp();


--
-- Name: users trigger_password_history; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_password_history BEFORE UPDATE ON authenc.users FOR EACH ROW EXECUTE FUNCTION authenc.add_password_to_history();


--
-- Name: satkers trigger_satkers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_satkers_updated_at BEFORE UPDATE ON authenc.satkers FOR EACH ROW EXECUTE FUNCTION authenc.update_satker_updated_at();


--
-- Name: client_scopes trigger_update_client_scopes_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_update_client_scopes_timestamp BEFORE UPDATE ON authenc.client_scopes FOR EACH ROW EXECUTE FUNCTION authenc.update_client_scopes_timestamp();


--
-- Name: service_accounts trigger_update_service_account_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_update_service_account_updated_at BEFORE UPDATE ON authenc.service_accounts FOR EACH ROW EXECUTE FUNCTION authenc.update_service_account_updated_at();


--
-- Name: users trigger_users_mfa_stats_refresh; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_users_mfa_stats_refresh AFTER INSERT OR DELETE OR UPDATE OF mfa_enabled, mfa_setup_at, mfa_last_used ON authenc.users FOR EACH STATEMENT EXECUTE FUNCTION authenc.trigger_refresh_mfa_statistics();


--
-- Name: access_levels update_access_levels_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_access_levels_updated_at BEFORE UPDATE ON authenc.access_levels FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: admin_level_types update_admin_level_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_admin_level_types_updated_at BEFORE UPDATE ON authenc.admin_level_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: authorization_policies update_authorization_policies_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_authorization_policies_updated_at BEFORE UPDATE ON authenc.authorization_policies FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: capabilities update_capabilities_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_capabilities_updated_at BEFORE UPDATE ON authenc.capabilities FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: credential_types update_credential_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_credential_types_updated_at BEFORE UPDATE ON authenc.credential_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: devices update_devices_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_devices_updated_at BEFORE UPDATE ON authenc.devices FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: federated_identities update_federated_identities_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_federated_identities_updated_at BEFORE UPDATE ON authenc.federated_identities FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: identity_provider_mappers update_identity_provider_mappers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_identity_provider_mappers_updated_at BEFORE UPDATE ON authenc.identity_provider_mappers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: identity_providers update_identity_providers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_identity_providers_updated_at BEFORE UPDATE ON authenc.identity_providers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: oauth2_clients update_oauth2_clients_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_oauth2_clients_updated_at BEFORE UPDATE ON authenc.oauth2_clients FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: organization_members update_organization_members_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_organization_members_updated_at BEFORE UPDATE ON authenc.organization_members FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: organizations update_organizations_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON authenc.organizations FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: permission_tickets update_permission_tickets_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_permission_tickets_updated_at BEFORE UPDATE ON authenc.permission_tickets FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: realms update_realms_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_realms_updated_at BEFORE UPDATE ON authenc.realms FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resource_servers update_resource_servers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resource_servers_updated_at BEFORE UPDATE ON authenc.resource_servers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: resources update_resources_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_resources_updated_at BEFORE UPDATE ON authenc.resources FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: role_types update_role_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_role_types_updated_at BEFORE UPDATE ON authenc.role_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: roles update_roles_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON authenc.roles FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: saml_identity_providers update_saml_identity_providers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_saml_identity_providers_updated_at BEFORE UPDATE ON authenc.saml_identity_providers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: saml_service_providers update_saml_service_providers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_saml_service_providers_updated_at BEFORE UPDATE ON authenc.saml_service_providers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: satker_types update_satker_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_satker_types_updated_at BEFORE UPDATE ON authenc.satker_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: scope_types update_scope_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_scope_types_updated_at BEFORE UPDATE ON authenc.scope_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: scopes update_scopes_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_scopes_updated_at BEFORE UPDATE ON authenc.scopes FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: social_accounts update_social_accounts_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_social_accounts_updated_at BEFORE UPDATE ON authenc.social_accounts FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: theme_types update_theme_types_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_theme_types_updated_at BEFORE UPDATE ON authenc.theme_types FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: uma_permission_requests update_uma_permission_requests_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_uma_permission_requests_updated_at BEFORE UPDATE ON authenc.uma_permission_requests FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: uma_policies update_uma_policies_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_uma_policies_updated_at BEFORE UPDATE ON authenc.uma_policies FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: users update_users_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON authenc.users FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: webauthn_credentials_old update_webauthn_credentials_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_webauthn_credentials_updated_at BEFORE UPDATE ON authenc.webauthn_credentials_old FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: mfa_backup_codes mfa_backup_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: authenc; Owner: -
--

ALTER TABLE ONLY authenc.mfa_backup_codes
    ADD CONSTRAINT mfa_backup_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: totp_secrets totp_secrets_user_id_fkey; Type: FK CONSTRAINT; Schema: authenc; Owner: -
--

ALTER TABLE ONLY authenc.totp_secrets
    ADD CONSTRAINT totp_secrets_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: access_levels access_levels_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.access_levels
    ADD CONSTRAINT access_levels_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: account_linking_requests account_linking_requests_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.account_linking_requests
    ADD CONSTRAINT account_linking_requests_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: account_linking_requests account_linking_requests_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.account_linking_requests
    ADD CONSTRAINT account_linking_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: admin_audit_log admin_audit_log_admin_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_audit_log
    ADD CONSTRAINT admin_audit_log_admin_user_id_fkey FOREIGN KEY (admin_user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: admin_audit_log admin_audit_log_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_audit_log
    ADD CONSTRAINT admin_audit_log_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_console_preferences admin_console_preferences_admin_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_preferences
    ADD CONSTRAINT admin_console_preferences_admin_user_id_fkey FOREIGN KEY (admin_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: admin_console_preferences admin_console_preferences_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_preferences
    ADD CONSTRAINT admin_console_preferences_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_console_sessions admin_console_sessions_admin_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_sessions
    ADD CONSTRAINT admin_console_sessions_admin_user_id_fkey FOREIGN KEY (admin_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: admin_console_sessions admin_console_sessions_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_console_sessions
    ADD CONSTRAINT admin_console_sessions_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_dashboard_metrics admin_dashboard_metrics_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_dashboard_metrics
    ADD CONSTRAINT admin_dashboard_metrics_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_level_types admin_level_types_parent_level_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_level_types
    ADD CONSTRAINT admin_level_types_parent_level_id_fkey FOREIGN KEY (parent_level_id) REFERENCES authenc.admin_level_types(id) ON DELETE SET NULL;


--
-- Name: admin_level_types admin_level_types_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_level_types
    ADD CONSTRAINT admin_level_types_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_notifications admin_notifications_read_by_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_notifications
    ADD CONSTRAINT admin_notifications_read_by_user_id_fkey FOREIGN KEY (read_by_user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: admin_notifications admin_notifications_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_notifications
    ADD CONSTRAINT admin_notifications_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: admin_notifications admin_notifications_target_admin_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.admin_notifications
    ADD CONSTRAINT admin_notifications_target_admin_user_id_fkey FOREIGN KEY (target_admin_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: audit_integrity_failures audit_integrity_failures_check_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_integrity_failures
    ADD CONSTRAINT audit_integrity_failures_check_id_fkey FOREIGN KEY (check_id) REFERENCES authenc.audit_integrity_checks(id) ON DELETE CASCADE;


--
-- Name: audit_logs audit_logs_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_logs
    ADD CONSTRAINT audit_logs_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE SET NULL;


--
-- Name: audit_logs audit_logs_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_logs
    ADD CONSTRAINT audit_logs_session_id_fkey FOREIGN KEY (session_id) REFERENCES authenc.user_sessions(id) ON DELETE SET NULL;


--
-- Name: audit_logs audit_logs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.audit_logs
    ADD CONSTRAINT audit_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: authentication_executions authentication_executions_flow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_executions
    ADD CONSTRAINT authentication_executions_flow_id_fkey FOREIGN KEY (flow_id) REFERENCES authenc.authentication_flows(id) ON DELETE CASCADE;


--
-- Name: authentication_sessions authentication_sessions_flow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authentication_sessions
    ADD CONSTRAINT authentication_sessions_flow_id_fkey FOREIGN KEY (flow_id) REFERENCES authenc.authentication_flows(id) ON DELETE SET NULL;


--
-- Name: authenticator_configs authenticator_configs_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_configs
    ADD CONSTRAINT authenticator_configs_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: authenticator_execution_results authenticator_execution_results_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_execution_results
    ADD CONSTRAINT authenticator_execution_results_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES authenc.authenticator_executions(id) ON DELETE CASCADE;


--
-- Name: authenticator_execution_results authenticator_execution_results_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_execution_results
    ADD CONSTRAINT authenticator_execution_results_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: authenticator_executions authenticator_executions_authenticator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_executions
    ADD CONSTRAINT authenticator_executions_authenticator_id_fkey FOREIGN KEY (authenticator_id) REFERENCES authenc.authenticator_configs(id) ON DELETE CASCADE;


--
-- Name: authenticator_executions authenticator_executions_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authenticator_executions
    ADD CONSTRAINT authenticator_executions_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: authorization_policies authorization_policies_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.authorization_policies
    ADD CONSTRAINT authorization_policies_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: capabilities capabilities_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.capabilities
    ADD CONSTRAINT capabilities_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: captcha_behavioral_metrics captcha_behavioral_metrics_challenge_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_behavioral_metrics
    ADD CONSTRAINT captcha_behavioral_metrics_challenge_id_fkey FOREIGN KEY (challenge_id) REFERENCES authenc.captcha_challenges(id) ON DELETE CASCADE;


--
-- Name: captcha_difficulty_adjustments captcha_difficulty_adjustments_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_difficulty_adjustments
    ADD CONSTRAINT captcha_difficulty_adjustments_created_by_fkey FOREIGN KEY (created_by) REFERENCES authenc.users(id);


--
-- Name: captcha_validation_attempts captcha_validation_attempts_behavioral_metrics_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_validation_attempts
    ADD CONSTRAINT captcha_validation_attempts_behavioral_metrics_id_fkey FOREIGN KEY (behavioral_metrics_id) REFERENCES authenc.captcha_behavioral_metrics(id);


--
-- Name: captcha_validation_attempts captcha_validation_attempts_challenge_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.captcha_validation_attempts
    ADD CONSTRAINT captcha_validation_attempts_challenge_id_fkey FOREIGN KEY (challenge_id) REFERENCES authenc.captcha_challenges(id) ON DELETE CASCADE;


--
-- Name: client_default_scopes client_default_scopes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_default_scopes
    ADD CONSTRAINT client_default_scopes_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: client_default_scopes client_default_scopes_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_default_scopes
    ADD CONSTRAINT client_default_scopes_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES authenc.client_scopes(id) ON DELETE CASCADE;


--
-- Name: client_optional_scopes client_optional_scopes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_optional_scopes
    ADD CONSTRAINT client_optional_scopes_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: client_optional_scopes client_optional_scopes_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_optional_scopes
    ADD CONSTRAINT client_optional_scopes_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES authenc.client_scopes(id) ON DELETE CASCADE;


--
-- Name: client_policies client_policies_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policies
    ADD CONSTRAINT client_policies_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: client_policy_assignments client_policy_assignments_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policy_assignments
    ADD CONSTRAINT client_policy_assignments_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: client_policy_assignments client_policy_assignments_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policy_assignments
    ADD CONSTRAINT client_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES authenc.client_policies(id) ON DELETE CASCADE;


--
-- Name: client_policy_assignments client_policy_assignments_profile_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_policy_assignments
    ADD CONSTRAINT client_policy_assignments_profile_id_fkey FOREIGN KEY (profile_id) REFERENCES authenc.client_profiles(id) ON DELETE CASCADE;


--
-- Name: client_profiles client_profiles_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_profiles
    ADD CONSTRAINT client_profiles_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: client_registration_audit_log client_registration_audit_log_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_audit_log
    ADD CONSTRAINT client_registration_audit_log_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE SET NULL;


--
-- Name: client_registration_audit_log client_registration_audit_log_initial_access_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_audit_log
    ADD CONSTRAINT client_registration_audit_log_initial_access_token_id_fkey FOREIGN KEY (initial_access_token_id) REFERENCES authenc.initial_access_tokens(id) ON DELETE SET NULL;


--
-- Name: client_registration_audit_log client_registration_audit_log_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_audit_log
    ADD CONSTRAINT client_registration_audit_log_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE SET NULL;


--
-- Name: client_registration_audit_log client_registration_audit_log_registration_access_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_audit_log
    ADD CONSTRAINT client_registration_audit_log_registration_access_token_id_fkey FOREIGN KEY (registration_access_token_id) REFERENCES authenc.client_registration_tokens(id) ON DELETE SET NULL;


--
-- Name: client_registration_policies client_registration_policies_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_policies
    ADD CONSTRAINT client_registration_policies_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: client_registration_tokens client_registration_tokens_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_tokens
    ADD CONSTRAINT client_registration_tokens_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: client_registration_tokens client_registration_tokens_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_registration_tokens
    ADD CONSTRAINT client_registration_tokens_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: client_scope_mappings client_scope_mappings_protocol_mapper_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scope_mappings
    ADD CONSTRAINT client_scope_mappings_protocol_mapper_id_fkey FOREIGN KEY (protocol_mapper_id) REFERENCES authenc.protocol_mappers(id) ON DELETE CASCADE;


--
-- Name: client_scope_mappings client_scope_mappings_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scope_mappings
    ADD CONSTRAINT client_scope_mappings_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES authenc.client_scopes(id) ON DELETE CASCADE;


--
-- Name: client_scopes client_scopes_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.client_scopes
    ADD CONSTRAINT client_scopes_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: credential_types credential_types_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.credential_types
    ADD CONSTRAINT credential_types_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: custom_themes custom_themes_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.custom_themes
    ADD CONSTRAINT custom_themes_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: device_sessions device_sessions_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_sessions
    ADD CONSTRAINT device_sessions_device_id_fkey FOREIGN KEY (device_id) REFERENCES authenc.devices(id) ON DELETE CASCADE;


--
-- Name: device_sessions device_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_sessions
    ADD CONSTRAINT device_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: device_sessions device_sessions_user_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_sessions
    ADD CONSTRAINT device_sessions_user_session_id_fkey FOREIGN KEY (user_session_id) REFERENCES authenc.user_sessions(id) ON DELETE CASCADE;


--
-- Name: device_trust_history device_trust_history_changed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_trust_history
    ADD CONSTRAINT device_trust_history_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES authenc.users(id);


--
-- Name: device_trust_history device_trust_history_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.device_trust_history
    ADD CONSTRAINT device_trust_history_device_id_fkey FOREIGN KEY (device_id) REFERENCES authenc.devices(id) ON DELETE CASCADE;


--
-- Name: devices devices_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.devices
    ADD CONSTRAINT devices_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: event_listener_executions event_listener_executions_event_log_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listener_executions
    ADD CONSTRAINT event_listener_executions_event_log_id_fkey FOREIGN KEY (event_log_id) REFERENCES authenc.event_log(id) ON DELETE CASCADE;


--
-- Name: event_listener_executions event_listener_executions_listener_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listener_executions
    ADD CONSTRAINT event_listener_executions_listener_id_fkey FOREIGN KEY (listener_id) REFERENCES authenc.event_listeners(id) ON DELETE CASCADE;


--
-- Name: event_listeners event_listeners_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_listeners
    ADD CONSTRAINT event_listeners_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: event_log event_log_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_log
    ADD CONSTRAINT event_log_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: event_log event_log_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_log
    ADD CONSTRAINT event_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: event_webhooks event_webhooks_listener_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_webhooks
    ADD CONSTRAINT event_webhooks_listener_id_fkey FOREIGN KEY (listener_id) REFERENCES authenc.event_listeners(id) ON DELETE CASCADE;


--
-- Name: event_webhooks event_webhooks_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.event_webhooks
    ADD CONSTRAINT event_webhooks_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: federated_auth_log federated_auth_log_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_auth_log
    ADD CONSTRAINT federated_auth_log_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: federated_auth_log federated_auth_log_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_auth_log
    ADD CONSTRAINT federated_auth_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: federated_identities federated_identities_identity_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identities
    ADD CONSTRAINT federated_identities_identity_provider_id_fkey FOREIGN KEY (identity_provider_id) REFERENCES authenc.identity_providers(id) ON DELETE CASCADE;


--
-- Name: federated_identities federated_identities_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identities
    ADD CONSTRAINT federated_identities_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: federated_identity_links federated_identity_links_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identity_links
    ADD CONSTRAINT federated_identity_links_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: federated_identity_links federated_identity_links_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.federated_identity_links
    ADD CONSTRAINT federated_identity_links_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: mfa_admin_actions fk_mfa_admin_actions_admin_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_admin_actions
    ADD CONSTRAINT fk_mfa_admin_actions_admin_user FOREIGN KEY (admin_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: mfa_admin_actions fk_mfa_admin_actions_target_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_admin_actions
    ADD CONSTRAINT fk_mfa_admin_actions_target_user FOREIGN KEY (target_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: satkers fk_parent_satker; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satkers
    ADD CONSTRAINT fk_parent_satker FOREIGN KEY (parent_code) REFERENCES authenc.satkers(code) ON DELETE SET NULL;


--
-- Name: password_history fk_password_history_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.password_history
    ADD CONSTRAINT fk_password_history_user FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: satker_admin_roles fk_satker_admin_assigned_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_admin_roles
    ADD CONSTRAINT fk_satker_admin_assigned_by FOREIGN KEY (assigned_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: satker_admin_roles fk_satker_admin_satker; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_admin_roles
    ADD CONSTRAINT fk_satker_admin_satker FOREIGN KEY (satker_code) REFERENCES authenc.satkers(code) ON DELETE CASCADE;


--
-- Name: satker_admin_roles fk_satker_admin_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_admin_roles
    ADD CONSTRAINT fk_satker_admin_user FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: satker_audit_logs fk_satker_audit_satker; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_audit_logs
    ADD CONSTRAINT fk_satker_audit_satker FOREIGN KEY (satker_code) REFERENCES authenc.satkers(code) ON DELETE CASCADE;


--
-- Name: satker_audit_logs fk_satker_audit_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_audit_logs
    ADD CONSTRAINT fk_satker_audit_user FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: satker_permissions fk_satker_perm_granted_by; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_permissions
    ADD CONSTRAINT fk_satker_perm_granted_by FOREIGN KEY (granted_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: satker_permissions fk_satker_perm_satker; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_permissions
    ADD CONSTRAINT fk_satker_perm_satker FOREIGN KEY (satker_code) REFERENCES authenc.satkers(code) ON DELETE CASCADE;


--
-- Name: satker_permissions fk_satker_perm_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_permissions
    ADD CONSTRAINT fk_satker_perm_user FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: service_account_audit_log fk_service_account_audit_sa; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_audit_log
    ADD CONSTRAINT fk_service_account_audit_sa FOREIGN KEY (service_account_id) REFERENCES authenc.service_accounts(id) ON DELETE CASCADE;


--
-- Name: service_accounts fk_service_account_realm; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_accounts
    ADD CONSTRAINT fk_service_account_realm FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: service_account_roles fk_service_account_role_role; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_roles
    ADD CONSTRAINT fk_service_account_role_role FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: service_account_roles fk_service_account_role_sa; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.service_account_roles
    ADD CONSTRAINT fk_service_account_role_sa FOREIGN KEY (service_account_id) REFERENCES authenc.service_accounts(id) ON DELETE CASCADE;


--
-- Name: group_attributes group_attributes_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_attributes
    ADD CONSTRAINT group_attributes_group_id_fkey FOREIGN KEY (group_id) REFERENCES authenc.groups(id) ON DELETE CASCADE;


--
-- Name: group_roles group_roles_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_roles
    ADD CONSTRAINT group_roles_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: group_roles group_roles_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_roles
    ADD CONSTRAINT group_roles_group_id_fkey FOREIGN KEY (group_id) REFERENCES authenc.groups(id) ON DELETE CASCADE;


--
-- Name: group_roles group_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.group_roles
    ADD CONSTRAINT group_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: groups groups_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.groups
    ADD CONSTRAINT groups_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES authenc.groups(id) ON DELETE CASCADE;


--
-- Name: groups groups_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.groups
    ADD CONSTRAINT groups_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: identity_broker_configs identity_broker_configs_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_broker_configs
    ADD CONSTRAINT identity_broker_configs_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: identity_provider_mappers identity_provider_mappers_identity_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_provider_mappers
    ADD CONSTRAINT identity_provider_mappers_identity_provider_id_fkey FOREIGN KEY (identity_provider_id) REFERENCES authenc.identity_providers(id) ON DELETE CASCADE;


--
-- Name: identity_provider_mappers identity_provider_mappers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_provider_mappers
    ADD CONSTRAINT identity_provider_mappers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: identity_providers identity_providers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.identity_providers
    ADD CONSTRAINT identity_providers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: initial_access_tokens initial_access_tokens_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.initial_access_tokens
    ADD CONSTRAINT initial_access_tokens_created_by_fkey FOREIGN KEY (created_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: initial_access_tokens initial_access_tokens_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.initial_access_tokens
    ADD CONSTRAINT initial_access_tokens_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: mfa_devices mfa_devices_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_devices
    ADD CONSTRAINT mfa_devices_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: mfa_policies mfa_policies_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.mfa_policies
    ADD CONSTRAINT mfa_policies_created_by_fkey FOREIGN KEY (created_by) REFERENCES authenc.users(id);


--
-- Name: oauth2_access_tokens oauth2_access_tokens_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_access_tokens
    ADD CONSTRAINT oauth2_access_tokens_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: oauth2_access_tokens oauth2_access_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_access_tokens
    ADD CONSTRAINT oauth2_access_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: oauth2_authorization_codes oauth2_authorization_codes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_authorization_codes
    ADD CONSTRAINT oauth2_authorization_codes_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: oauth2_authorization_codes oauth2_authorization_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_authorization_codes
    ADD CONSTRAINT oauth2_authorization_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: oauth2_clients oauth2_clients_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_clients
    ADD CONSTRAINT oauth2_clients_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES authenc.users(id);


--
-- Name: oauth2_clients oauth2_clients_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_clients
    ADD CONSTRAINT oauth2_clients_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: oauth2_provider_configs oauth2_provider_configs_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_provider_configs
    ADD CONSTRAINT oauth2_provider_configs_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: oauth2_states oauth2_states_provider_config_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_states
    ADD CONSTRAINT oauth2_states_provider_config_id_fkey FOREIGN KEY (provider_config_id) REFERENCES authenc.oauth2_provider_configs(id) ON DELETE CASCADE;


--
-- Name: oauth2_states oauth2_states_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_states
    ADD CONSTRAINT oauth2_states_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: oauth2_token_exchanges oauth2_token_exchanges_provider_config_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_token_exchanges
    ADD CONSTRAINT oauth2_token_exchanges_provider_config_id_fkey FOREIGN KEY (provider_config_id) REFERENCES authenc.oauth2_provider_configs(id) ON DELETE CASCADE;


--
-- Name: oauth2_token_exchanges oauth2_token_exchanges_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.oauth2_token_exchanges
    ADD CONSTRAINT oauth2_token_exchanges_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: offline_tokens offline_tokens_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.offline_tokens
    ADD CONSTRAINT offline_tokens_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: offline_tokens offline_tokens_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.offline_tokens
    ADD CONSTRAINT offline_tokens_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: offline_tokens offline_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.offline_tokens
    ADD CONSTRAINT offline_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: organization_domains organization_domains_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_domains
    ADD CONSTRAINT organization_domains_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES authenc.organizations(id) ON DELETE CASCADE;


--
-- Name: organization_identity_providers organization_identity_providers_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_identity_providers
    ADD CONSTRAINT organization_identity_providers_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES authenc.organizations(id) ON DELETE CASCADE;


--
-- Name: organization_invitations organization_invitations_accepted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_accepted_by_fkey FOREIGN KEY (accepted_by) REFERENCES authenc.users(id);


--
-- Name: organization_invitations organization_invitations_invited_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES authenc.users(id);


--
-- Name: organization_invitations organization_invitations_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_invitations
    ADD CONSTRAINT organization_invitations_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES authenc.organizations(id) ON DELETE CASCADE;


--
-- Name: organization_members organization_members_invited_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES authenc.users(id);


--
-- Name: organization_members organization_members_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES authenc.organizations(id) ON DELETE CASCADE;


--
-- Name: organization_members organization_members_role_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_role_type_id_fkey FOREIGN KEY (role_type_id) REFERENCES authenc.role_types(id) ON DELETE SET NULL;


--
-- Name: organization_members organization_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organization_members
    ADD CONSTRAINT organization_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: organizations organizations_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organizations
    ADD CONSTRAINT organizations_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES authenc.users(id);


--
-- Name: organizations organizations_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.organizations
    ADD CONSTRAINT organizations_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: permission_tickets permission_tickets_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.permission_tickets
    ADD CONSTRAINT permission_tickets_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: permission_tickets permission_tickets_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.permission_tickets
    ADD CONSTRAINT permission_tickets_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES authenc.resources(id) ON DELETE CASCADE;


--
-- Name: permission_tickets permission_tickets_resource_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.permission_tickets
    ADD CONSTRAINT permission_tickets_resource_server_id_fkey FOREIGN KEY (resource_server_id) REFERENCES authenc.resource_servers(id) ON DELETE CASCADE;


--
-- Name: permission_tickets permission_tickets_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.permission_tickets
    ADD CONSTRAINT permission_tickets_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES authenc.scopes(id) ON DELETE CASCADE;


--
-- Name: protocol_mappers protocol_mappers_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.protocol_mappers
    ADD CONSTRAINT protocol_mappers_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: protocol_mappers protocol_mappers_client_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.protocol_mappers
    ADD CONSTRAINT protocol_mappers_client_scope_id_fkey FOREIGN KEY (client_scope_id) REFERENCES authenc.client_scopes(id) ON DELETE CASCADE;


--
-- Name: protocol_mappers protocol_mappers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.protocol_mappers
    ADD CONSTRAINT protocol_mappers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: realm_theme_settings realm_theme_settings_account_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_account_theme_id_fkey FOREIGN KEY (account_theme_id) REFERENCES authenc.custom_themes(id) ON DELETE SET NULL;


--
-- Name: realm_theme_settings realm_theme_settings_admin_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_admin_theme_id_fkey FOREIGN KEY (admin_theme_id) REFERENCES authenc.custom_themes(id) ON DELETE SET NULL;


--
-- Name: realm_theme_settings realm_theme_settings_email_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_email_theme_id_fkey FOREIGN KEY (email_theme_id) REFERENCES authenc.custom_themes(id) ON DELETE SET NULL;


--
-- Name: realm_theme_settings realm_theme_settings_login_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_login_theme_id_fkey FOREIGN KEY (login_theme_id) REFERENCES authenc.custom_themes(id) ON DELETE SET NULL;


--
-- Name: realm_theme_settings realm_theme_settings_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.realm_theme_settings
    ADD CONSTRAINT realm_theme_settings_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: refresh_token_history refresh_token_history_user_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.refresh_token_history
    ADD CONSTRAINT refresh_token_history_user_session_id_fkey FOREIGN KEY (user_session_id) REFERENCES authenc.user_sessions(id) ON DELETE CASCADE;


--
-- Name: resource_servers resource_servers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.resource_servers
    ADD CONSTRAINT resource_servers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: resources resources_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.resources
    ADD CONSTRAINT resources_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: resources resources_resource_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.resources
    ADD CONSTRAINT resources_resource_server_id_fkey FOREIGN KEY (resource_server_id) REFERENCES authenc.resource_servers(id) ON DELETE CASCADE;


--
-- Name: role_capabilities role_capabilities_capability_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_capabilities
    ADD CONSTRAINT role_capabilities_capability_id_fkey FOREIGN KEY (capability_id) REFERENCES authenc.capabilities(id) ON DELETE CASCADE;


--
-- Name: role_capabilities role_capabilities_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_capabilities
    ADD CONSTRAINT role_capabilities_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: role_capabilities role_capabilities_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_capabilities
    ADD CONSTRAINT role_capabilities_role_id_fkey FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: role_hierarchy role_hierarchy_child_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_hierarchy
    ADD CONSTRAINT role_hierarchy_child_role_id_fkey FOREIGN KEY (child_role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: role_hierarchy role_hierarchy_parent_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_hierarchy
    ADD CONSTRAINT role_hierarchy_parent_role_id_fkey FOREIGN KEY (parent_role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: role_permissions role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_permissions
    ADD CONSTRAINT role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: role_policies role_policies_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_policies
    ADD CONSTRAINT role_policies_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: role_policies role_policies_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_policies
    ADD CONSTRAINT role_policies_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES authenc.authorization_policies(id) ON DELETE CASCADE;


--
-- Name: role_policies role_policies_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_policies
    ADD CONSTRAINT role_policies_role_id_fkey FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: role_types role_types_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.role_types
    ADD CONSTRAINT role_types_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: roles roles_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.roles
    ADD CONSTRAINT roles_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: saml_identity_providers saml_identity_providers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_identity_providers
    ADD CONSTRAINT saml_identity_providers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: saml_service_providers saml_service_providers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_service_providers
    ADD CONSTRAINT saml_service_providers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: saml_sessions saml_sessions_identity_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_sessions
    ADD CONSTRAINT saml_sessions_identity_provider_id_fkey FOREIGN KEY (identity_provider_id) REFERENCES authenc.saml_identity_providers(id) ON DELETE CASCADE;


--
-- Name: saml_sessions saml_sessions_service_provider_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_sessions
    ADD CONSTRAINT saml_sessions_service_provider_id_fkey FOREIGN KEY (service_provider_id) REFERENCES authenc.saml_service_providers(id) ON DELETE CASCADE;


--
-- Name: saml_sessions saml_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.saml_sessions
    ADD CONSTRAINT saml_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: satker_types satker_types_parent_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.satker_types
    ADD CONSTRAINT satker_types_parent_type_id_fkey FOREIGN KEY (parent_type_id) REFERENCES authenc.satker_types(id) ON DELETE SET NULL;


--
-- Name: scope_types scope_types_parent_scope_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scope_types
    ADD CONSTRAINT scope_types_parent_scope_type_id_fkey FOREIGN KEY (parent_scope_type_id) REFERENCES authenc.scope_types(id) ON DELETE SET NULL;


--
-- Name: scope_types scope_types_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scope_types
    ADD CONSTRAINT scope_types_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: scopes scopes_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scopes
    ADD CONSTRAINT scopes_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: scopes scopes_resource_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.scopes
    ADD CONSTRAINT scopes_resource_server_id_fkey FOREIGN KEY (resource_server_id) REFERENCES authenc.resource_servers(id) ON DELETE CASCADE;


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: social_accounts social_accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_accounts
    ADD CONSTRAINT social_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: social_login_configs social_login_configs_oauth2_config_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.social_login_configs
    ADD CONSTRAINT social_login_configs_oauth2_config_id_fkey FOREIGN KEY (oauth2_config_id) REFERENCES authenc.oauth2_provider_configs(id) ON DELETE CASCADE;


--
-- Name: software_statement_issuers software_statement_issuers_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.software_statement_issuers
    ADD CONSTRAINT software_statement_issuers_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: theme_inheritance theme_inheritance_parent_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_inheritance
    ADD CONSTRAINT theme_inheritance_parent_theme_id_fkey FOREIGN KEY (parent_theme_id) REFERENCES authenc.custom_themes(id) ON DELETE SET NULL;


--
-- Name: theme_inheritance theme_inheritance_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_inheritance
    ADD CONSTRAINT theme_inheritance_theme_id_fkey FOREIGN KEY (theme_id) REFERENCES authenc.custom_themes(id) ON DELETE CASCADE;


--
-- Name: theme_resources theme_resources_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_resources
    ADD CONSTRAINT theme_resources_theme_id_fkey FOREIGN KEY (theme_id) REFERENCES authenc.custom_themes(id) ON DELETE CASCADE;


--
-- Name: theme_templates theme_templates_theme_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.theme_templates
    ADD CONSTRAINT theme_templates_theme_id_fkey FOREIGN KEY (theme_id) REFERENCES authenc.custom_themes(id) ON DELETE CASCADE;


--
-- Name: token_exchange_audit token_exchange_audit_issued_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.token_exchange_audit
    ADD CONSTRAINT token_exchange_audit_issued_token_id_fkey FOREIGN KEY (issued_token_id) REFERENCES authenc.oauth2_access_tokens(id) ON DELETE SET NULL;


--
-- Name: token_exchange_audit token_exchange_audit_original_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.token_exchange_audit
    ADD CONSTRAINT token_exchange_audit_original_client_id_fkey FOREIGN KEY (original_client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE SET NULL;


--
-- Name: token_exchange_audit token_exchange_audit_subject_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.token_exchange_audit
    ADD CONSTRAINT token_exchange_audit_subject_user_id_fkey FOREIGN KEY (subject_user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: token_exchange_audit token_exchange_audit_target_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.token_exchange_audit
    ADD CONSTRAINT token_exchange_audit_target_client_id_fkey FOREIGN KEY (target_client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: uma_permission_requests uma_permission_requests_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_permission_requests
    ADD CONSTRAINT uma_permission_requests_resource_id_fkey FOREIGN KEY (resource_id) REFERENCES authenc.resources(id) ON DELETE CASCADE;


--
-- Name: uma_policies uma_policies_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_policies
    ADD CONSTRAINT uma_policies_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: uma_policies uma_policies_resource_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.uma_policies
    ADD CONSTRAINT uma_policies_resource_server_id_fkey FOREIGN KEY (resource_server_id) REFERENCES authenc.resource_servers(id) ON DELETE CASCADE;


--
-- Name: user_attributes user_attributes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_attributes
    ADD CONSTRAINT user_attributes_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_consent_scopes user_consent_scopes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consent_scopes
    ADD CONSTRAINT user_consent_scopes_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE CASCADE;


--
-- Name: user_consent_scopes user_consent_scopes_scope_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consent_scopes
    ADD CONSTRAINT user_consent_scopes_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES authenc.client_scopes(id) ON DELETE CASCADE;


--
-- Name: user_consent_scopes user_consent_scopes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consent_scopes
    ADD CONSTRAINT user_consent_scopes_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_consents user_consents_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_consents
    ADD CONSTRAINT user_consents_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_groups user_groups_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_groups
    ADD CONSTRAINT user_groups_group_id_fkey FOREIGN KEY (group_id) REFERENCES authenc.groups(id) ON DELETE CASCADE;


--
-- Name: user_groups user_groups_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_groups
    ADD CONSTRAINT user_groups_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_policies user_policies_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_policies
    ADD CONSTRAINT user_policies_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES authenc.users(id) ON DELETE SET NULL;


--
-- Name: user_policies user_policies_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_policies
    ADD CONSTRAINT user_policies_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES authenc.authorization_policies(id) ON DELETE CASCADE;


--
-- Name: user_policies user_policies_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_policies
    ADD CONSTRAINT user_policies_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_roles user_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_roles
    ADD CONSTRAINT user_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES authenc.roles(id) ON DELETE CASCADE;


--
-- Name: user_roles user_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_roles
    ADD CONSTRAINT user_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_client_id_fkey FOREIGN KEY (client_id) REFERENCES authenc.oauth2_clients(id) ON DELETE SET NULL;


--
-- Name: user_sessions user_sessions_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_device_id_fkey FOREIGN KEY (device_id) REFERENCES authenc.devices(id) ON DELETE SET NULL;


--
-- Name: user_sessions user_sessions_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.user_sessions
    ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: users users_realm_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.users
    ADD CONSTRAINT users_realm_id_fkey FOREIGN KEY (realm_id) REFERENCES authenc.realms(id) ON DELETE RESTRICT;


--
-- Name: webauthn_audit_log webauthn_audit_log_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_audit_log
    ADD CONSTRAINT webauthn_audit_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: webauthn_challenges webauthn_challenges_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_challenges
    ADD CONSTRAINT webauthn_challenges_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: webauthn_credentials_old webauthn_credentials_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials_old
    ADD CONSTRAINT webauthn_credentials_user_id_fkey FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- Name: webauthn_credentials webauthn_credentials_user_id_fkey1; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY authenc.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_user_id_fkey1 FOREIGN KEY (user_id) REFERENCES authenc.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--



-- F5-B: pg_dump --schema-only creates matviews WITH NO DATA (drops the initial
-- populate). The seed step fires triggers that REFRESH ... CONCURRENTLY, which
-- requires an already-populated matview. Do the initial non-concurrent populate
-- here (txn-safe; base tables are empty at this point).
REFRESH MATERIALIZED VIEW authenc.captcha_analytics;
REFRESH MATERIALIZED VIEW authenc.mfa_statistics;
