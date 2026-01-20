-- Migration for CAPTCHA System
-- This migration adds tables and functions needed for AI-resistant CAPTCHA functionality

-- Create CAPTCHA challenges table
CREATE TABLE IF NOT EXISTS captcha_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_type VARCHAR(50) NOT NULL CHECK (challenge_type IN ('Visual', 'Audio', 'Behavioral', 'Logical', 'Hybrid')),
    difficulty_level SMALLINT NOT NULL CHECK (difficulty_level >= 1 AND difficulty_level <= 10),
    encrypted_data TEXT NOT NULL,
    expected_answer_hash VARCHAR(256) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    session_id VARCHAR(255),
    ip_address INET NOT NULL,
    solved BOOLEAN NOT NULL DEFAULT false,
    solved_at TIMESTAMP WITH TIME ZONE,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3
);

-- Create indexes for CAPTCHA challenges
CREATE INDEX IF NOT EXISTS idx_captcha_challenges_expires_at ON captcha_challenges(expires_at);
CREATE INDEX IF NOT EXISTS idx_captcha_challenges_ip_created ON captcha_challenges(ip_address, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_captcha_challenges_session ON captcha_challenges(session_id, created_at DESC) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_captcha_challenges_unsolved ON captcha_challenges(created_at DESC) WHERE solved = false;

-- Create behavioral metrics table
CREATE TABLE IF NOT EXISTS captcha_behavioral_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_id UUID NOT NULL REFERENCES captcha_challenges(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    mouse_movements JSONB,
    keystroke_dynamics JSONB,
    timing_patterns JSONB NOT NULL,
    browser_fingerprint JSONB NOT NULL,
    risk_score NUMERIC(3,2) NOT NULL CHECK (risk_score >= 0.0 AND risk_score <= 1.0),
    classification VARCHAR(20) NOT NULL CHECK (classification IN ('Human', 'Suspicious', 'Bot', 'Unknown')),
    created_at TIMESTAMPE ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for behavioral metrics
CREATE INDEX IF NOT EXISTS idx_behavioral_metrics_challenge ON captcha_behavioral_metrics(challenge_id);
CREATE INDEX IF NOT EXISTS idx_behavioral_metrics_session ON captcha_behavioral_metrics(session_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_behavioral_metrics_risk_score ON captcha_behavioral_metrics(risk_score DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_behavioral_metrics_classification ON captcha_behavioral_metrics(classification, created_at DESC);

-- Create CAPTCHA validation attempts table
CREATE TABLE IF NOT EXISTS captcha_validation_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_id UUID NOT NULL REFERENCES captcha_challenges(id) ON DELETE CASCADE,
    ip_address INET NOT NULL,
    user_agent TEXT,
    answer_provided TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    confidence_score NUMERIC(3,2) CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    risk_assessment VARCHAR(20) NOT NULL CHECK (risk_assessment IN ('Low', 'Medium', 'High', 'Critical')),
    behavioral_metrics_id UUID REFERENCES captcha_behavioral_metrics(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for validation attempts
CREATE INDEX IF NOT EXISTS idx_validation_attempts_challenge ON captcha_validation_attempts(challenge_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_validation_attempts_ip ON captcha_validation_attempts(ip_address, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_validation_attempts_success ON captcha_validation_attempts(success, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_validation_attempts_risk ON captcha_validation_attempts(risk_assessment, created_at DESC);

-- Create CAPTCHA difficulty adjustments table
CREATE TABLE IF NOT EXISTS captcha_difficulty_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_pattern CIDR,
    session_pattern VARCHAR(255),
    difficulty_level SMALLINT NOT NULL CHECK (difficulty_level >= 1 AND difficulty_level <= 10),
    reason TEXT,
    created_by UUID REFERENCES users(id),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for difficulty adjustments
CREATE INDEX IF NOT EXISTS idx_difficulty_adjustments_ip ON captcha_difficulty_adjustments(ip_pattern) WHERE ip_pattern IS NOT NULL AND active = true;
CREATE INDEX IF NOT EXISTS idx_difficulty_adjustments_session ON captcha_difficulty_adjustments(session_pattern) WHERE session_pattern IS NOT NULL AND active = true;
CREATE INDEX IF NOT EXISTS idx_difficulty_adjustments_active ON captcha_difficulty_adjustments(active, created_at DESC);

-- Create CAPTCHA analytics summary table (materialized view)
CREATE MATERIALIZED VIEW IF NOT EXISTS captcha_analytics AS
SELECT
    DATE(c.created_at) as date,
    c.challenge_type,
    c.difficulty_level,
    COUNT(*) as total_challenges,
    COUNT(*) FILTER (WHERE c.solved = true) as solved_challenges,
    COUNT(*) FILTER (WHERE c.solved = false AND c.expires_at < NOW()) as expired_challenges,
    AVG(c.attempts) as avg_attempts,
    COUNT(DISTINCT c.ip_address) as unique_ips,
    COUNT(DISTINCT c.session_id) as unique_sessions,
    -- Behavioral analysis aggregates
    AVG(bm.risk_score) as avg_risk_score,
    COUNT(*) FILTER (WHERE bm.classification = 'Bot') as bot_detections,
    COUNT(*) FILTER (WHERE bm.classification = 'Suspicious') as suspicious_detections,
    COUNT(*) FILTER (WHERE bm.classification = 'Human') as human_detections,
    -- Validation attempts aggregates
    COUNT(va.id) as total_validation_attempts,
    COUNT(*) FILTER (WHERE va.success = true) as successful_validations,
    AVG(va.confidence_score) as avg_confidence_score,
    COUNT(*) FILTER (WHERE va.risk_assessment = 'High') as high_risk_attempts,
    COUNT(*) FILTER (WHERE va.risk_assessment = 'Critical') as critical_risk_attempts,
    MAX(c.created_at) as last_updated
FROM captcha_challenges c
LEFT JOIN captcha_behavioral_metrics bm ON c.id = bm.challenge_id
LEFT JOIN captcha_validation_attempts va ON c.id = va.challenge_id
WHERE c.created_at >= NOW() - INTERVAL '90 days'
GROUP BY DATE(c.created_at), c.challenge_type, c.difficulty_level;

-- Create unique index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_captcha_analytics_unique ON captcha_analytics(date, challenge_type, difficulty_level);

-- Create CAPTCHA performance metrics table
CREATE TABLE IF NOT EXISTS captcha_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    challenge_generation_latency_ms BIGINT NOT NULL,
    validation_latency_ms BIGINT NOT NULL,
    success_rate NUMERIC(5,4) NOT NULL CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    failure_rate NUMERIC(5,4) NOT NULL CHECK (failure_rate >= 0.0 AND failure_rate <= 1.0),
    average_difficulty NUMERIC(4,2) NOT NULL,
    concurrent_challenges BIGINT NOT NULL,
    memory_usage_mb NUMERIC(10,2) NOT NULL,
    cpu_usage_percent NUMERIC(5,2) NOT NULL CHECK (cpu_usage_percent >= 0.0 AND cpu_usage_percent <= 100.0)
);

-- Create indexes for performance metrics
CREATE INDEX IF NOT EXISTS idx_performance_metrics_timestamp ON captcha_performance_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_latency ON captcha_performance_metrics(challenge_generation_latency_ms, validation_latency_ms);

-- Create CAPTCHA bot detection metrics table
CREATE TABLE IF NOT EXISTS captcha_bot_detection_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    total_detections BIGINT NOT NULL,
    true_positives BIGINT NOT NULL,
    false_positives BIGINT NOT NULL,
    true_negatives BIGINT NOT NULL,
    false_negatives BIGINT NOT NULL,
    accuracy_rate NUMERIC(5,4) NOT NULL CHECK (accuracy_rate >= 0.0 AND accuracy_rate <= 1.0),
    precision_rate NUMERIC(5,4) NOT NULL CHECK (precision_rate >= 0.0 AND precision_rate <= 1.0),
    recall_rate NUMERIC(5,4) NOT NULL CHECK (recall_rate >= 0.0 AND recall_rate <= 1.0),
    f1_score NUMERIC(5,4) NOT NULL CHECK (f1_score >= 0.0 AND f1_score <= 1.0),
    risk_distribution JSONB NOT NULL
);

-- Create indexes for bot detection metrics
CREATE INDEX IF NOT EXISTS idx_bot_detection_metrics_timestamp ON captcha_bot_detection_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_bot_detection_metrics_accuracy ON captcha_bot_detection_metrics(accuracy_rate DESC, timestamp DESC);

-- Create CAPTCHA user experience metrics table
CREATE TABLE IF NOT EXISTS captcha_user_experience_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    average_completion_time_ms BIGINT NOT NULL,
    abandonment_rate NUMERIC(5,4) NOT NULL CHECK (abandonment_rate >= 0.0 AND abandonment_rate <= 1.0),
    retry_rate NUMERIC(5,4) NOT NULL CHECK (retry_rate >= 0.0 AND retry_rate <= 1.0),
    accessibility_usage_rate NUMERIC(5,4) NOT NULL CHECK (accessibility_usage_rate >= 0.0 AND accessibility_usage_rate <= 1.0),
    user_satisfaction_score NUMERIC(3,2) NOT NULL CHECK (user_satisfaction_score >= 0.0 AND user_satisfaction_score <= 1.0),
    challenge_type_preferences JSONB NOT NULL,
    difficulty_distribution JSONB NOT NULL
);

-- Create indexes for user experience metrics
CREATE INDEX IF NOT EXISTS idx_user_experience_metrics_timestamp ON captcha_user_experience_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_user_experience_metrics_satisfaction ON captcha_user_experience_metrics(user_satisfaction_score DESC, timestamp DESC);

-- Create CAPTCHA security event metrics table
CREATE TABLE IF NOT EXISTS captcha_security_event_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    attack_attempts BIGINT NOT NULL,
    blocked_ips BIGINT NOT NULL,
    rate_limit_triggers BIGINT NOT NULL,
    lockout_events BIGINT NOT NULL,
    suspicious_behavior_count BIGINT NOT NULL,
    threat_level_distribution JSONB NOT NULL,
    geographic_distribution JSONB NOT NULL
);

-- Create indexes for security event metrics
CREATE INDEX IF NOT EXISTS idx_security_event_metrics_timestamp ON captcha_security_event_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_security_event_metrics_attacks ON captcha_security_event_metrics(attack_attempts DESC, timestamp DESC);

-- Function to clean up expired challenges
CREATE OR REPLACE FUNCTION cleanup_expired_captcha_challenges()
RETURNS BIGINT
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

-- Function to get challenge difficulty for IP/session
CREATE OR REPLACE FUNCTION get_captcha_difficulty(
    p_ip_address INET,
    p_session_id VARCHAR(255) DEFAULT NULL
)
RETURNS SMALLINT
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

-- Function to record CAPTCHA validation attempt
CREATE OR REPLACE FUNCTION record_captcha_validation(
    p_challenge_id UUID,
    p_ip_address INET,
    p_user_agent TEXT,
    p_answer_provided TEXT,
    p_success BOOLEAN,
    p_confidence_score NUMERIC DEFAULT NULL,
    p_risk_assessment VARCHAR(20) DEFAULT 'Medium',
    p_behavioral_metrics_id UUID DEFAULT NULL
)
RETURNS UUID
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

-- Function to get CAPTCHA analytics summary
CREATE OR REPLACE FUNCTION get_captcha_analytics_summary(
    p_days INTEGER DEFAULT 7
)
RETURNS TABLE(
    total_challenges BIGINT,
    solved_challenges BIGINT,
    success_rate NUMERIC(5,2),
    avg_difficulty NUMERIC(3,1),
    bot_detection_rate NUMERIC(5,2),
    unique_ips BIGINT,
    high_risk_attempts BIGINT
)
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

-- Function to refresh CAPTCHA analytics
CREATE OR REPLACE FUNCTION refresh_captcha_analytics()
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY captcha_analytics;
END;
$$;

-- Create trigger to automatically refresh analytics periodically
CREATE OR REPLACE FUNCTION trigger_refresh_captcha_analytics()
RETURNS TRIGGER
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

-- Create triggers on CAPTCHA tables to refresh analytics
DROP TRIGGER IF EXISTS trigger_captcha_analytics_refresh ON captcha_challenges;
CREATE TRIGGER trigger_captcha_analytics_refresh
    AFTER INSERT OR UPDATE OR DELETE
    ON captcha_challenges
    FOR EACH STATEMENT
    EXECUTE FUNCTION trigger_refresh_captcha_analytics();

-- Create scheduled job to clean up expired challenges (requires pg_cron extension)
-- This is optional and will only work if pg_cron is installed
-- SELECT cron.schedule('cleanup-expired-captcha', '*/15 * * * *', 'SELECT cleanup_expired_captcha_challenges();');

-- Add comments for documentation
COMMENT ON TABLE captcha_challenges IS 'Stores CAPTCHA challenges with encrypted data and metadata';
COMMENT ON TABLE captcha_behavioral_metrics IS 'Stores behavioral analysis data for bot detection';
COMMENT ON TABLE captcha_validation_attempts IS 'Audit log of all CAPTCHA validation attempts';
COMMENT ON TABLE captcha_difficulty_adjustments IS 'Configuration for adaptive difficulty adjustments';
COMMENT ON MATERIALIZED VIEW captcha_analytics IS 'Aggregated CAPTCHA analytics for monitoring and reporting';

COMMENT ON FUNCTION cleanup_expired_captcha_challenges() IS 'Cleans up expired CAPTCHA challenges and related data';
COMMENT ON FUNCTION get_captcha_difficulty(INET, VARCHAR) IS 'Determines appropriate CAPTCHA difficulty for IP/session';
COMMENT ON FUNCTION record_captcha_validation(UUID, INET, TEXT, TEXT, BOOLEAN, NUMERIC, VARCHAR, UUID) IS 'Records a CAPTCHA validation attempt with all metadata';
COMMENT ON FUNCTION get_captcha_analytics_summary(INTEGER) IS 'Returns summary analytics for CAPTCHA system performance';
COMMENT ON FUNCTION refresh_captcha_analytics() IS 'Refreshes the CAPTCHA analytics materialized view';
