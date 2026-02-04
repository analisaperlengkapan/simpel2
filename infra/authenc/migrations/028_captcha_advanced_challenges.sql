SET search_path = authenc, public;
-- Migration: Advanced CAPTCHA Challenge Types and Personalization
-- Description: Adds tables for challenge type preferences, user history, and effectiveness analytics
-- Version: 028

-- Create table for user challenge history and preferences
CREATE TABLE IF NOT EXISTS captcha_user_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    session_id VARCHAR(255),
    total_attempts INTEGER NOT NULL DEFAULT 0,
    successful_completions INTEGER NOT NULL DEFAULT 0,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    current_difficulty SMALLINT NOT NULL DEFAULT 3 CHECK (current_difficulty >= 1 AND current_difficulty <= 10),
    preferred_challenge_types TEXT[], -- Array of preferred challenge type names
    avoid_challenge_types TEXT[], -- Array of challenge types to avoid
    last_challenge_time TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_history UNIQUE (user_id)
);

-- Create indexes for user history
CREATE INDEX IF NOT EXISTS idx_user_history_user_id ON captcha_user_history(user_id);
CREATE INDEX IF NOT EXISTS idx_user_history_session_id ON captcha_user_history(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_history_updated ON captcha_user_history(updated_at DESC);

-- Create table for challenge type success rates per user
CREATE TABLE IF NOT EXISTS captcha_user_type_performance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    challenge_type VARCHAR(100) NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    successes INTEGER NOT NULL DEFAULT 0,
    success_rate NUMERIC(5,4) NOT NULL DEFAULT 0.0 CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    avg_completion_time_secs NUMERIC(8,2) NOT NULL DEFAULT 30.0,
    last_attempt_time TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_type_performance UNIQUE (user_id, challenge_type)
);

-- Create indexes for user type performance
CREATE INDEX IF NOT EXISTS idx_user_type_perf_user ON captcha_user_type_performance(user_id, success_rate DESC);
CREATE INDEX IF NOT EXISTS idx_user_type_perf_type ON captcha_user_type_performance(challenge_type, success_rate DESC);
CREATE INDEX IF NOT EXISTS idx_user_type_perf_updated ON captcha_user_type_performance(updated_at DESC);

-- Create table for global challenge type effectiveness metrics
CREATE TABLE IF NOT EXISTS captcha_type_effectiveness (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_type VARCHAR(100) NOT NULL UNIQUE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    success_rate NUMERIC(5,4) NOT NULL DEFAULT 0.5 CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    avg_completion_time_secs NUMERIC(8,2) NOT NULL DEFAULT 30.0,
    bot_detection_rate NUMERIC(5,4) NOT NULL DEFAULT 0.5 CHECK (bot_detection_rate >= 0.0 AND bot_detection_rate <= 1.0),
    user_satisfaction NUMERIC(5,4) NOT NULL DEFAULT 0.5 CHECK (user_satisfaction >= 0.0 AND user_satisfaction <= 1.0),
    effectiveness_score NUMERIC(5,4) NOT NULL DEFAULT 0.5 CHECK (effectiveness_score >= 0.0 AND effectiveness_score <= 1.0),
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for type effectiveness
CREATE INDEX IF NOT EXISTS idx_type_effectiveness_score ON captcha_type_effectiveness(effectiveness_score DESC);
CREATE INDEX IF NOT EXISTS idx_type_effectiveness_type ON captcha_type_effectiveness(challenge_type);
CREATE INDEX IF NOT EXISTS idx_type_effectiveness_updated ON captcha_type_effectiveness(last_updated DESC);

-- Initialize effectiveness metrics for all challenge types
INSERT INTO captcha_type_effectiveness (challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score)
VALUES
    ('Visual', 0, 0.5, 30.0, 0.5, 0.5, 0.5),
    ('Audio', 0, 0.5, 45.0, 0.5, 0.5, 0.5),
    ('Logical', 0, 0.5, 35.0, 0.5, 0.5, 0.5),
    ('Behavioral', 0, 0.5, 40.0, 0.6, 0.5, 0.53),
    ('Hybrid', 0, 0.5, 60.0, 0.7, 0.4, 0.53),
    ('ImageJigsaw', 0, 0.5, 45.0, 0.6, 0.5, 0.53),
    ('ImageRotation', 0, 0.5, 20.0, 0.4, 0.6, 0.47),
    ('ImageObjectSelection', 0, 0.5, 35.0, 0.6, 0.5, 0.53),
    ('ImageSequence', 0, 0.5, 50.0, 0.7, 0.5, 0.57),
    ('AudioToneSequence', 0, 0.5, 40.0, 0.6, 0.5, 0.53),
    ('AudioSpokenDigits', 0, 0.5, 35.0, 0.5, 0.6, 0.53),
    ('AudioSpokenWords', 0, 0.5, 45.0, 0.6, 0.5, 0.53),
    ('AudioPatternRecognition', 0, 0.5, 50.0, 0.7, 0.4, 0.53),
    ('AudioSoundIdentification', 0, 0.5, 40.0, 0.6, 0.5, 0.53)
ON CONFLICT (challenge_type) DO NOTHING;

-- Function to update user history after challenge completion
CREATE OR REPLACE FUNCTION update_captcha_user_history(
    p_user_id VARCHAR(255),
    p_challenge_type VARCHAR(100),
    p_success BOOLEAN,
    p_completion_time_secs NUMERIC(8,2)
) RETURNS VOID AS $$
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
$$ LANGUAGE plpgsql;

-- Function to update global challenge type effectiveness
CREATE OR REPLACE FUNCTION update_captcha_type_effectiveness(
    p_challenge_type VARCHAR(100),
    p_success BOOLEAN,
    p_completion_time_secs NUMERIC(8,2),
    p_bot_detected BOOLEAN
) RETURNS VOID AS $$
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
$$ LANGUAGE plpgsql;

-- Function to get recommended challenge type for user
CREATE OR REPLACE FUNCTION get_recommended_challenge_type(
    p_user_id VARCHAR(255),
    p_risk_score NUMERIC(3,2) DEFAULT 0.5
) RETURNS VARCHAR(100) AS $$
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
$$ LANGUAGE plpgsql;

-- Function to get user's recommended difficulty
CREATE OR REPLACE FUNCTION get_recommended_difficulty(
    p_user_id VARCHAR(255)
) RETURNS SMALLINT AS $$
DECLARE
    v_difficulty SMALLINT;
BEGIN
    SELECT current_difficulty INTO v_difficulty
    FROM captcha_user_history
    WHERE user_id = p_user_id;

    RETURN COALESCE(v_difficulty, 3);
END;
$$ LANGUAGE plpgsql;

-- Create view for challenge type analytics
CREATE OR REPLACE VIEW captcha_type_analytics AS
SELECT
    cte.challenge_type,
    cte.usage_count,
    cte.success_rate,
    cte.avg_completion_time_secs,
    cte.bot_detection_rate,
    cte.user_satisfaction,
    cte.effectiveness_score,
    COUNT(DISTINCT cutp.user_id) as unique_users,
    AVG(cutp.success_rate) as avg_user_success_rate,
    cte.last_updated
FROM captcha_type_effectiveness cte
LEFT JOIN captcha_user_type_performance cutp ON cte.challenge_type = cutp.challenge_type
GROUP BY cte.challenge_type, cte.usage_count, cte.success_rate, cte.avg_completion_time_secs,
         cte.bot_detection_rate, cte.user_satisfaction, cte.effectiveness_score, cte.last_updated
ORDER BY cte.effectiveness_score DESC;

-- Add comment to tables
COMMENT ON TABLE captcha_user_history IS 'Stores user challenge history and preferences for personalization';
COMMENT ON TABLE captcha_user_type_performance IS 'Tracks user performance by challenge type';
COMMENT ON TABLE captcha_type_effectiveness IS 'Global effectiveness metrics for each challenge type';
COMMENT ON FUNCTION update_captcha_user_history IS 'Updates user history and adjusts difficulty after challenge completion';
COMMENT ON FUNCTION update_captcha_type_effectiveness IS 'Updates global effectiveness metrics for challenge types';
COMMENT ON FUNCTION get_recommended_challenge_type IS 'Returns recommended challenge type based on user history and risk score';
COMMENT ON FUNCTION get_recommended_difficulty IS 'Returns recommended difficulty level for user';
