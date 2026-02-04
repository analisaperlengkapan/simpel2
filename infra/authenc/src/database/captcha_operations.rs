//! CAPTCHA Database Operations
//!
//! Database operations for CAPTCHA challenge storage, validation, and analytics

use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::services::captcha::metrics::{
    BotDetectionMetrics, PerformanceMetrics, SecurityEventMetrics, UserExperienceMetrics,
};
use crate::services::captcha::{
    BehaviorClassification, BehavioralMetrics, Challenge, ChallengeType, RiskLevel,
};
use serde_json;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use tokio_postgres::Row;
use uuid::Uuid;

/// CAPTCHA database operations
pub struct CaptchaOperations {
    db: Database,
}

impl CaptchaOperations {
    /// Create new CAPTCHA operations instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Store a new CAPTCHA challenge
    pub async fn store_challenge(&self, challenge: &Challenge) -> Result<()> {
        let created_at: chrono::DateTime<chrono::Utc> = challenge.created_at.into();
        let expires_at: chrono::DateTime<chrono::Utc> = challenge.expires_at.into();

        let challenge_type_str = match challenge.challenge_type {
            ChallengeType::Visual => "Visual",
            ChallengeType::Audio => "Audio",
            ChallengeType::Behavioral => "Behavioral",
            ChallengeType::Logical => "Logical",
            ChallengeType::Hybrid => "Hybrid",
        };

        // Parse UUID from string ID
        let challenge_uuid = Uuid::parse_str(&challenge.id)
            .map_err(|_| AuthencError::validation("Invalid challenge ID format"))?;

        // Parse IP address
        let ip_addr: IpAddr = challenge
            .ip_address
            .parse()
            .map_err(|_| AuthencError::validation("Invalid IP address format"))?;

        // Difficulty as i16 for PostgreSQL
        let difficulty: i16 = challenge.difficulty_level as i16;

        let query = r#"
            INSERT INTO captcha_challenges (
                id, challenge_type, difficulty_level, encrypted_data,
                expected_answer_hash, created_at, expires_at, session_id, ip_address
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        tracing::info!(
            "Storing challenge: id={}, type={}, difficulty={}, ip={}",
            challenge_uuid,
            challenge_type_str,
            difficulty,
            ip_addr
        );

        self.db
            .execute(
                query,
                &[
                    &challenge_uuid,
                    &challenge_type_str,
                    &difficulty,
                    &challenge.encrypted_data,
                    &challenge.expected_answer_hash,
                    &created_at,
                    &expires_at,
                    &challenge.session_id,
                    &ip_addr,
                ],
            )
            .await?;

        Ok(())
    }

    /// Retrieve a CAPTCHA challenge by ID
    pub async fn get_challenge(&self, challenge_id: &str) -> Result<Option<Challenge>> {
        let challenge_uuid = Uuid::parse_str(challenge_id)
            .map_err(|_| AuthencError::validation("Invalid challenge ID format"))?;

        let query = r#"
            SELECT id, challenge_type, difficulty_level, encrypted_data,
                   expected_answer_hash, created_at, expires_at, session_id,
                   ip_address, solved, attempts
            FROM captcha_challenges
            WHERE id = $1
        "#;

        if let Some(row) = self.db.query_opt(query, &[&challenge_uuid]).await? {
            Ok(Some(row_to_challenge(row)?))
        } else {
            Ok(None)
        }
    }

    /// Update challenge attempts and solved status
    pub async fn update_challenge_status(&self, challenge_id: &str, solved: bool) -> Result<()> {
        let challenge_uuid = Uuid::parse_str(challenge_id)
            .map_err(|_| AuthencError::validation("Invalid challenge ID format"))?;

        let query = if solved {
            r#"
                UPDATE captcha_challenges
                SET attempts = attempts + 1, solved = true, solved_at = NOW()
                WHERE id = $1
            "#
        } else {
            r#"
                UPDATE captcha_challenges
                SET attempts = attempts + 1
                WHERE id = $1
            "#
        };

        self.db.execute(query, &[&challenge_uuid]).await?;
        Ok(())
    }

    /// Store behavioral metrics
    pub async fn store_behavioral_metrics(
        &self,
        challenge_id: &str,
        metrics: &BehavioralMetrics,
    ) -> Result<Uuid> {
        let challenge_uuid = Uuid::parse_str(challenge_id)
            .map_err(|_| AuthencError::validation("Invalid challenge ID format"))?;

        let metrics_id = Uuid::new_v4();

        let mouse_movements_json = serde_json::to_value(&metrics.mouse_movements).map_err(|e| {
            AuthencError::internal(&format!("Failed to serialize mouse movements: {}", e))
        })?;

        let keystroke_dynamics_json =
            serde_json::to_value(&metrics.keystroke_dynamics).map_err(|e| {
                AuthencError::internal(&format!("Failed to serialize keystroke dynamics: {}", e))
            })?;

        let timing_patterns_json = serde_json::to_value(&metrics.timing_patterns).map_err(|e| {
            AuthencError::internal(&format!("Failed to serialize timing patterns: {}", e))
        })?;

        let browser_fingerprint_json =
            serde_json::to_value(&metrics.browser_fingerprint).map_err(|e| {
                AuthencError::internal(&format!("Failed to serialize browser fingerprint: {}", e))
            })?;

        let classification_str = match metrics.classification {
            BehaviorClassification::Human => "Human",
            BehaviorClassification::Suspicious => "Suspicious",
            BehaviorClassification::Bot => "Bot",
            BehaviorClassification::Unknown => "Unknown",
        };

        // Convert f64 to String for NUMERIC column
        let risk_score_str = metrics.risk_score.to_string();

        let query = r#"
            INSERT INTO captcha_behavioral_metrics (
                id, challenge_id, session_id, mouse_movements, keystroke_dynamics,
                timing_patterns, browser_fingerprint, risk_score, classification
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9)
        "#;

        self.db
            .execute(
                query,
                &[
                    &metrics_id,
                    &challenge_uuid,
                    &metrics.session_id,
                    &mouse_movements_json,
                    &keystroke_dynamics_json,
                    &timing_patterns_json,
                    &browser_fingerprint_json,
                    &risk_score_str,
                    &classification_str,
                ],
            )
            .await?;

        Ok(metrics_id)
    }

    /// Record a validation attempt
    pub async fn record_validation_attempt(
        &self,
        challenge_id: &str,
        ip_address: &str,
        user_agent: Option<&str>,
        answer_provided: &str,
        success: bool,
        confidence_score: Option<f64>,
        risk_assessment: &RiskLevel,
        behavioral_metrics_id: Option<Uuid>,
    ) -> Result<Uuid> {
        let challenge_uuid = Uuid::parse_str(challenge_id)
            .map_err(|_| AuthencError::validation("Invalid challenge ID format"))?;

        let attempt_id = Uuid::new_v4();

        let ip_addr: IpAddr = ip_address
            .parse()
            .map_err(|_| AuthencError::validation("Invalid IP address format"))?;

        let risk_assessment_str = match risk_assessment {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        };

        // Convert f64 to Option<String> for NUMERIC column
        let confidence_score_str = confidence_score.map(|v| v.to_string());

        let query = r#"
            INSERT INTO captcha_validation_attempts (
                id, challenge_id, ip_address, user_agent, answer_provided,
                success, confidence_score, risk_assessment, behavioral_metrics_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9)
        "#;

        tracing::info!(
            "Recording validation attempt: challenge_id={}, ip={}, success={}, risk={}",
            challenge_id,
            ip_address,
            success,
            risk_assessment_str
        );

        self.db
            .execute(
                query,
                &[
                    &attempt_id,
                    &challenge_uuid,
                    &ip_addr,
                    &user_agent,
                    &answer_provided,
                    &success,
                    &confidence_score_str,
                    &risk_assessment_str,
                    &behavioral_metrics_id,
                ],
            )
            .await?;

        Ok(attempt_id)
    }

    /// Get CAPTCHA difficulty for IP address
    pub async fn get_difficulty_for_ip(
        &self,
        ip_address: &str,
        session_id: Option<&str>,
    ) -> Result<u8> {
        let ip_addr: IpAddr = ip_address
            .parse()
            .map_err(|_| AuthencError::validation("Invalid IP address format"))?;

        let query = "SELECT get_captcha_difficulty($1, $2)";

        let row: tokio_postgres::Row = self.db.query_one(query, &[&ip_addr, &session_id]).await?;

        let difficulty: i16 = row.get(0);
        Ok(difficulty as u8)
    }

    /// Set difficulty adjustment for IP pattern or session
    pub async fn set_difficulty_adjustment(
        &self,
        ip_pattern: Option<&str>,
        session_pattern: Option<&str>,
        difficulty: u8,
        reason: Option<&str>,
        created_by: Option<Uuid>,
        expires_at: Option<SystemTime>,
    ) -> Result<Uuid> {
        let adjustment_id = Uuid::new_v4();

        let expires_at_secs = expires_at
            .map(|expires_at| {
                expires_at
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| AuthencError::internal("Invalid expires_at timestamp"))
                    .map(|duration| duration.as_secs() as i64)
            })
            .transpose()?;

        let difficulty_level: i16 = difficulty as i16;

        let query = r#"
            INSERT INTO captcha_difficulty_adjustments (
                id, ip_pattern, session_pattern, difficulty_level, reason, created_by, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        self.db
            .execute(
                query,
                &[
                    &adjustment_id,
                    &ip_pattern,
                    &session_pattern,
                    &difficulty_level,
                    &reason,
                    &created_by,
                    &expires_at_secs.map(|s| chrono::DateTime::from_timestamp(s, 0)),
                ],
            )
            .await?;

        Ok(adjustment_id)
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&self) -> Result<u64> {
        let query = "SELECT cleanup_expired_captcha_challenges()";
        let row: tokio_postgres::Row = self.db.query_one(query, &[]).await?;
        let deleted_count: i64 = row.get(0);
        Ok(deleted_count as u64)
    }

    /// Get CAPTCHA analytics summary
    pub async fn get_analytics_summary(&self, days: i32) -> Result<CaptchaAnalyticsSummary> {
        let query = "SELECT * FROM get_captcha_analytics_summary($1)";
        let row: tokio_postgres::Row = self.db.query_one(query, &[&days]).await?;

        Ok(CaptchaAnalyticsSummary {
            total_challenges: row.get::<_, i64>(0) as u64,
            solved_challenges: row.get::<_, i64>(1) as u64,
            success_rate: row.get::<_, Option<f64>>(2).unwrap_or(0.0),
            avg_difficulty: row.get::<_, Option<f64>>(3).unwrap_or(3.0),
            bot_detection_rate: row.get::<_, Option<f64>>(4).unwrap_or(0.0),
            unique_ips: row.get::<_, i64>(5) as u64,
            high_risk_attempts: row.get::<_, i64>(6) as u64,
        })
    }

    /// Get recent validation attempts for IP
    pub async fn get_recent_attempts_for_ip(
        &self,
        ip_address: &str,
        hours: i32,
    ) -> Result<Vec<ValidationAttempt>> {
        let ip_addr: IpAddr = ip_address
            .parse()
            .map_err(|_| AuthencError::validation("Invalid IP address format"))?;

        let query = r#"
            SELECT va.id, va.challenge_id, va.success, va.confidence_score,
                   va.risk_assessment, va.created_at
            FROM captcha_validation_attempts va
            WHERE va.ip_address = $1
              AND va.created_at > NOW() - ($2 || ' hours')::INTERVAL
            ORDER BY va.created_at DESC
        "#;

        let rows = self.db.query_raw(query, &[&ip_addr, &hours]).await?;
        let mut attempts = Vec::new();

        for row in rows {
            attempts.push(ValidationAttempt {
                id: row.get::<_, Uuid>(0),
                challenge_id: row.get::<_, Uuid>(1),
                success: row.get(2),
                confidence_score: row.get(3),
                risk_assessment: parse_risk_level(row.get::<_, String>(4))?,
                created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(5),
            });
        }

        Ok(attempts)
    }

    /// Refresh analytics materialized view
    pub async fn refresh_analytics(&self) -> Result<()> {
        let query = "SELECT refresh_captcha_analytics()";
        self.db.execute(query, &[]).await?;
        Ok(())
    }

    /// Get count of active challenges
    pub async fn get_active_challenges_count(&self) -> Result<u64> {
        let query = r#"
            SELECT COUNT(*) FROM captcha_challenges
            WHERE expires_at > NOW() AND solved = false
        "#;

        let row: tokio_postgres::Row = self.db.query_one(query, &[]).await?;
        let count: i64 = row.get(0);
        Ok(count as u64)
    }

    /// Store performance metrics
    pub async fn store_performance_metrics(&self, metrics: &PerformanceMetrics) -> Result<()> {
        // Convert SystemTime to OffsetDateTime for tokio-postgres
        let timestamp = OffsetDateTime::from(metrics.timestamp);

        // Parse UUID
        let metric_uuid = Uuid::parse_str(&metrics.metric_id)
            .map_err(|_| AuthencError::validation("Invalid metric ID"))?;

        // Convert f64 to String for NUMERIC columns (tokio-postgres doesn't support f64→NUMERIC directly)
        let success_rate_str = metrics.success_rate.to_string();
        let failure_rate_str = metrics.failure_rate.to_string();
        let avg_difficulty_str = metrics.average_difficulty.to_string();
        let memory_usage_str = metrics.memory_usage_mb.to_string();
        let cpu_usage_str = metrics.cpu_usage_percent.to_string();

        // Use explicit casts from text to numeric for f64 values
        let query = r#"
            INSERT INTO captcha_performance_metrics (
                id, timestamp, challenge_generation_latency_ms, validation_latency_ms,
                success_rate, failure_rate, average_difficulty, concurrent_challenges,
                memory_usage_mb, cpu_usage_percent
            ) VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8, $9::numeric, $10::numeric)
        "#;

        tracing::info!(
            "Storing performance metrics: id={}, timestamp={}, latency={}, success_rate={}",
            metrics.metric_id,
            timestamp,
            metrics.challenge_generation_latency_ms,
            metrics.success_rate
        );

        // Store values with proper types
        let gen_latency: i64 = metrics.challenge_generation_latency_ms as i64;
        let val_latency: i64 = metrics.validation_latency_ms as i64;
        let concurrent: i64 = metrics.concurrent_challenges as i64;

        // Use &str references for String values
        self.db
            .execute(
                query,
                &[
                    &metric_uuid,
                    &timestamp,
                    &gen_latency,
                    &val_latency,
                    &success_rate_str.as_str(),
                    &failure_rate_str.as_str(),
                    &avg_difficulty_str.as_str(),
                    &concurrent,
                    &memory_usage_str.as_str(),
                    &cpu_usage_str.as_str(),
                ],
            )
            .await?;

        Ok(())
    }

    /// Store bot detection metrics
    pub async fn store_bot_detection_metrics(&self, metrics: &BotDetectionMetrics) -> Result<()> {
        // Convert SystemTime to OffsetDateTime for tokio-postgres
        let timestamp = OffsetDateTime::from(metrics.timestamp);

        let risk_distribution_json =
            serde_json::to_value(&metrics.risk_distribution).map_err(|e| {
                AuthencError::internal(&format!("Failed to serialize risk distribution: {}", e))
            })?;

        // Parse UUID
        let metric_uuid = Uuid::parse_str(&metrics.metric_id)
            .map_err(|_| AuthencError::validation("Invalid metric ID"))?;

        // Convert f64 to String for NUMERIC columns
        let accuracy_rate_str = metrics.accuracy_rate.to_string();
        let precision_str = metrics.precision.to_string();
        let recall_str = metrics.recall.to_string();
        let f1_score_str = metrics.f1_score.to_string();

        let query = r#"
            INSERT INTO captcha_bot_detection_metrics (
                id, timestamp, total_detections, true_positives, false_positives,
                true_negatives, false_negatives, accuracy_rate, precision_rate,
                recall_rate, f1_score, risk_distribution
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric, $10::numeric, $11::numeric, $12)
        "#;

        self.db
            .execute(
                query,
                &[
                    &metric_uuid,
                    &timestamp,
                    &(metrics.total_detections as i64),
                    &(metrics.true_positives as i64),
                    &(metrics.false_positives as i64),
                    &(metrics.true_negatives as i64),
                    &(metrics.false_negatives as i64),
                    &accuracy_rate_str,
                    &precision_str,
                    &recall_str,
                    &f1_score_str,
                    &risk_distribution_json,
                ],
            )
            .await?;

        Ok(())
    }

    /// Store user experience metrics
    pub async fn store_user_experience_metrics(
        &self,
        metrics: &UserExperienceMetrics,
    ) -> Result<()> {
        // Convert SystemTime to OffsetDateTime for tokio-postgres
        let timestamp = OffsetDateTime::from(metrics.timestamp);

        let challenge_type_preferences_json =
            serde_json::to_value(&metrics.challenge_type_preferences).map_err(|e| {
                AuthencError::internal(&format!(
                    "Failed to serialize challenge type preferences: {}",
                    e
                ))
            })?;

        let difficulty_distribution_json = serde_json::to_value(&metrics.difficulty_distribution)
            .map_err(|e| {
            AuthencError::internal(&format!(
                "Failed to serialize difficulty distribution: {}",
                e
            ))
        })?;

        // Parse UUID
        let metric_uuid = Uuid::parse_str(&metrics.metric_id)
            .map_err(|_| AuthencError::validation("Invalid metric ID"))?;

        // Convert f64 to String for NUMERIC columns
        let abandonment_rate_str = metrics.abandonment_rate.to_string();
        let retry_rate_str = metrics.retry_rate.to_string();
        let accessibility_rate_str = metrics.accessibility_usage_rate.to_string();
        let satisfaction_score_str = metrics.user_satisfaction_score.to_string();

        let query = r#"
            INSERT INTO captcha_user_experience_metrics (
                id, timestamp, average_completion_time_ms, abandonment_rate, retry_rate,
                accessibility_usage_rate, user_satisfaction_score, challenge_type_preferences,
                difficulty_distribution
            ) VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::numeric, $8, $9)
        "#;

        self.db
            .execute(
                query,
                &[
                    &metric_uuid,
                    &timestamp,
                    &(metrics.average_completion_time_ms as i64),
                    &abandonment_rate_str,
                    &retry_rate_str,
                    &accessibility_rate_str,
                    &satisfaction_score_str,
                    &challenge_type_preferences_json,
                    &difficulty_distribution_json,
                ],
            )
            .await?;

        Ok(())
    }

    /// Store security event metrics
    pub async fn store_security_event_metrics(&self, metrics: &SecurityEventMetrics) -> Result<()> {
        // Convert SystemTime to OffsetDateTime for tokio-postgres
        let timestamp = OffsetDateTime::from(metrics.timestamp);

        let threat_level_distribution_json =
            serde_json::to_value(&metrics.threat_level_distribution).map_err(|e| {
                AuthencError::internal(&format!(
                    "Failed to serialize threat level distribution: {}",
                    e
                ))
            })?;

        let geographic_distribution_json = serde_json::to_value(&metrics.geographic_distribution)
            .map_err(|e| {
            AuthencError::internal(&format!(
                "Failed to serialize geographic distribution: {}",
                e
            ))
        })?;

        // Parse UUID
        let metric_uuid = Uuid::parse_str(&metrics.metric_id)
            .map_err(|_| AuthencError::validation("Invalid metric ID"))?;

        let query = r#"
            INSERT INTO captcha_security_event_metrics (
                id, timestamp, attack_attempts, blocked_ips, rate_limit_triggers,
                lockout_events, suspicious_behavior_count, threat_level_distribution,
                geographic_distribution
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        self.db
            .execute(
                query,
                &[
                    &metric_uuid,
                    &timestamp,
                    &(metrics.attack_attempts as i64),
                    &(metrics.blocked_ips as i64),
                    &(metrics.rate_limit_triggers as i64),
                    &(metrics.lockout_events as i64),
                    &(metrics.suspicious_behavior_count as i64),
                    &threat_level_distribution_json,
                    &geographic_distribution_json,
                ],
            )
            .await?;

        Ok(())
    }
}

/// CAPTCHA analytics summary
#[derive(Debug, Clone)]
pub struct CaptchaAnalyticsSummary {
    /// Total number of CAPTCHA challenges issued
    pub total_challenges: u64,
    /// Number of challenges successfully solved
    pub solved_challenges: u64,
    /// Success rate as a percentage (0.0 to 1.0)
    pub success_rate: f64,
    /// Average difficulty score of challenges
    pub avg_difficulty: f64,
    /// Rate of detected bot attempts
    pub bot_detection_rate: f64,
    /// Number of unique IP addresses
    pub unique_ips: u64,
    /// Number of high-risk validation attempts
    pub high_risk_attempts: u64,
}

/// Validation attempt record
#[derive(Debug, Clone)]
pub struct ValidationAttempt {
    /// Unique identifier for the validation attempt
    pub id: Uuid,
    /// ID of the CAPTCHA challenge being validated
    pub challenge_id: Uuid,
    /// Whether the validation was successful
    pub success: bool,
    /// Confidence score of the validation (0.0 to 1.0)
    pub confidence_score: Option<f64>,
    /// Risk assessment level
    pub risk_assessment: RiskLevel,
    /// Timestamp when the attempt was made
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Convert database row to Challenge
fn row_to_challenge(row: Row) -> Result<Challenge> {
    let id: Uuid = row.get(0);
    let challenge_type_str: String = row.get(1);
    let difficulty_level: i16 = row.get(2);
    let encrypted_data: String = row.get(3);
    let expected_answer_hash: String = row.get(4);
    let created_at: chrono::DateTime<chrono::Utc> = row.get(5);
    let expires_at: chrono::DateTime<chrono::Utc> = row.get(6);
    let session_id: Option<String> = row.get(7);
    let ip_address: IpAddr = row.get(8);

    let challenge_type = match challenge_type_str.as_str() {
        "Visual" => ChallengeType::Visual,
        "Audio" => ChallengeType::Audio,
        "Behavioral" => ChallengeType::Behavioral,
        "Logical" => ChallengeType::Logical,
        "Hybrid" => ChallengeType::Hybrid,
        _ => return Err(AuthencError::internal("Invalid challenge type in database")),
    };

    let created_at_system =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(created_at.timestamp() as u64);
    let expires_at_system =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(expires_at.timestamp() as u64);

    Ok(Challenge {
        id: id.to_string(),
        challenge_type,
        difficulty_level: difficulty_level as u8,
        encrypted_data,
        expected_answer_hash,
        created_at: created_at_system,
        expires_at: expires_at_system,
        session_id,
        ip_address: ip_address.to_string(),
        encrypted_challenge_data: None, // Legacy records don't have encrypted metadata
        is_encrypted: false,            // Legacy records use unencrypted storage
        plaintext_data: None,
    })
}

/// Parse risk level from string
fn parse_risk_level(risk_str: String) -> Result<RiskLevel> {
    match risk_str.as_str() {
        "Low" => Ok(RiskLevel::Low),
        "Medium" => Ok(RiskLevel::Medium),
        "High" => Ok(RiskLevel::High),
        "Critical" => Ok(RiskLevel::Critical),
        _ => Err(AuthencError::internal("Invalid risk level in database")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::captcha::{BrowserFingerprint, TimingAnalysis};

    #[tokio::test]
    async fn test_challenge_storage_and_retrieval() {
        let db = Database::mock().await;
        let _ops = CaptchaOperations::new(db);

        let challenge = Challenge::new(
            ChallengeType::Visual,
            5,
            "encrypted_test_data".to_string(),
            "test_hash".to_string(),
            Some("test_session".to_string()),
            "192.168.1.1".to_string(),
        );

        // This test would require a real database connection
        // For now, we just test the structure
        assert_eq!(challenge.difficulty_level, 5);
        assert_eq!(challenge.challenge_type, ChallengeType::Visual);
    }

    #[test]
    fn test_behavioral_metrics_serialization() {
        let metrics = BehavioralMetrics {
            session_id: "test_session".to_string(),
            mouse_movements: vec![],
            keystroke_dynamics: vec![],
            timing_patterns: TimingAnalysis::default(),
            browser_fingerprint: BrowserFingerprint::default(),
            risk_score: 0.5,
            classification: BehaviorClassification::Human,
        };

        // Test that serialization works
        let json = serde_json::to_value(&metrics).unwrap();
        assert!(json.is_object());
    }
}
