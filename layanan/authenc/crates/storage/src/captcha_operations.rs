//! CAPTCHA Database Operations
//!
//! Database operations for CAPTCHA challenge storage, validation, and analytics
//!
//! **Status**: MIGRATED BUT DISABLED
//! **Reason**: Depends on models and services not yet migrated:
//! - `services::captcha::metrics::{BotDetectionMetrics, PerformanceMetrics, SecurityEventMetrics, UserExperienceMetrics}`
//! - `services::captcha::{BehaviorClassification, BehavioralMetrics, Challenge, ChallengeType, RiskLevel}`
//! - `services::captcha::{BrowserFingerprint, TimingAnalysis}`
//!
//! **TODO**: Enable after Phase 3 (models migration) and Phase 4 (services migration)

#![allow(dead_code, unused_imports)]

use crate::database::Database;
use authenc_types::{AuthencError, Result};
// use crate::services::captcha::metrics::{...}; // TODO: Migrate services
// use crate::services::captcha::{...}; // TODO: Migrate services
use serde_json;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use tokio_postgres::Row;
use uuid::Uuid;

/*
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

    // ... rest of implementation commented out ...
    // See original file for full implementation
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

// Full implementation is commented out - see original file
// This file contains ~800 lines of CAPTCHA operations
*/
