-- Migration 041: Fix CAPTCHA numeric columns for Rust f64 compatibility
-- tokio-postgres can serialize f64 to DOUBLE PRECISION but not to NUMERIC

-- Fix captcha_validation_attempts
ALTER TABLE captcha_validation_attempts
    ALTER COLUMN confidence_score TYPE DOUBLE PRECISION USING confidence_score::DOUBLE PRECISION;

-- Fix captcha_behavioral_metrics
ALTER TABLE captcha_behavioral_metrics
    ALTER COLUMN risk_score TYPE DOUBLE PRECISION USING risk_score::DOUBLE PRECISION;

-- Fix captcha_performance_metrics
ALTER TABLE captcha_performance_metrics
    ALTER COLUMN success_rate TYPE DOUBLE PRECISION USING success_rate::DOUBLE PRECISION,
    ALTER COLUMN failure_rate TYPE DOUBLE PRECISION USING failure_rate::DOUBLE PRECISION,
    ALTER COLUMN average_difficulty TYPE DOUBLE PRECISION USING average_difficulty::DOUBLE PRECISION,
    ALTER COLUMN memory_usage_mb TYPE DOUBLE PRECISION USING memory_usage_mb::DOUBLE PRECISION,
    ALTER COLUMN cpu_usage_percent TYPE DOUBLE PRECISION USING cpu_usage_percent::DOUBLE PRECISION;

-- Fix captcha_bot_detection_metrics
ALTER TABLE captcha_bot_detection_metrics
    ALTER COLUMN accuracy_rate TYPE DOUBLE PRECISION USING accuracy_rate::DOUBLE PRECISION,
    ALTER COLUMN precision_rate TYPE DOUBLE PRECISION USING precision_rate::DOUBLE PRECISION,
    ALTER COLUMN recall_rate TYPE DOUBLE PRECISION USING recall_rate::DOUBLE PRECISION,
    ALTER COLUMN f1_score TYPE DOUBLE PRECISION USING f1_score::DOUBLE PRECISION;

-- Fix captcha_user_experience_metrics
ALTER TABLE captcha_user_experience_metrics
    ALTER COLUMN abandonment_rate TYPE DOUBLE PRECISION USING abandonment_rate::DOUBLE PRECISION,
    ALTER COLUMN retry_rate TYPE DOUBLE PRECISION USING retry_rate::DOUBLE PRECISION,
    ALTER COLUMN accessibility_usage_rate TYPE DOUBLE PRECISION USING accessibility_usage_rate::DOUBLE PRECISION,
    ALTER COLUMN user_satisfaction_score TYPE DOUBLE PRECISION USING user_satisfaction_score::DOUBLE PRECISION;
