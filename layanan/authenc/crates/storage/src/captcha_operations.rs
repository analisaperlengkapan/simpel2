use crate::database::Database;
use authenc_types::{
    Result,
    domain::captcha::{Challenge, ChallengeType},
};
use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

impl Database {
    /// Store a new CAPTCHA challenge in the database
    pub async fn store_captcha_challenge(
        &self,
        challenge: &Challenge,
        ip_address: IpAddr,
        raw_data: String,
    ) -> Result<()> {
        let client = self.get_connection().await?;

        let challenge_type_str = match challenge.challenge_type {
            ChallengeType::Visual => "Visual",
            ChallengeType::Logical => "Logical",
        };

        let difficulty = challenge.difficulty as i16;

        let query = r#"
            INSERT INTO captcha_challenges (
                id, challenge_type, difficulty_level, encrypted_data,
                expected_answer_hash, created_at, expires_at, session_id, ip_address,
                solved, solved_at, attempts, max_attempts
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#;

        client
            .execute(
                query,
                &[
                    &challenge.id,
                    &challenge_type_str,
                    &difficulty,
                    &raw_data,
                    &challenge.answer_hash,
                    &challenge.created_at,
                    &challenge.expires_at,
                    &challenge.session_id,
                    &ip_address,
                    &challenge.verified,    // solved
                    &None::<DateTime<Utc>>, // solved_at
                    &0_i32,                 // attempts
                    &3_i32,                 // max_attempts
                ],
            )
            .await?;

        Ok(())
    }

    /// Retrieve a CAPTCHA challenge by ID
    pub async fn get_captcha_challenge(&self, id: Uuid) -> Result<Option<Challenge>> {
        let client = self.get_connection().await?;

        let query = r#"
            SELECT id, challenge_type, expected_answer_hash, difficulty_level,
                   expires_at, solved, session_id, created_at
            FROM captcha_challenges
            WHERE id = $1
        "#;

        let row_opt = client.query_opt(query, &[&id]).await?;

        if let Some(row) = row_opt {
            let type_str: String = row.get("challenge_type");
            let challenge_type = match type_str.as_str() {
                "Logical" => ChallengeType::Logical,
                _ => ChallengeType::Visual,
            };

            let difficulty: i16 = row.get("difficulty_level");

            Ok(Some(Challenge {
                id: row.get("id"),
                challenge_type,
                answer_hash: row.get("expected_answer_hash"),
                difficulty: difficulty as u32,
                expires_at: row.get("expires_at"),
                verified: row.get("solved"),
                session_id: row.get("session_id"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Mark a CAPTCHA challenge as solved
    pub async fn mark_captcha_challenge_verified(&self, id: Uuid) -> Result<()> {
        let client = self.get_connection().await?;

        let query = r#"
            UPDATE captcha_challenges
            SET solved = true, solved_at = NOW()
            WHERE id = $1
        "#;

        client.execute(query, &[&id]).await?;

        Ok(())
    }

    /// Atomically consume a SOLVED, unexpired CAPTCHA challenge for login (#49).
    ///
    /// Single-use: the `DELETE ... RETURNING` lets exactly one caller win, so a
    /// solved challenge cannot be replayed for a second login. Returns `true`
    /// iff a fresh (unexpired), already-solved challenge was consumed. Freshness
    /// is bounded by the challenge's own `expires_at` (≤5 min after creation).
    pub async fn consume_solved_captcha(&self, id: Uuid) -> Result<bool> {
        let client = self.get_connection().await?;

        let query = r#"
            DELETE FROM captcha_challenges
            WHERE id = $1 AND solved = true AND expires_at > NOW()
            RETURNING id
        "#;

        let row = client.query_opt(query, &[&id]).await?;
        Ok(row.is_some())
    }

    /// Record a CAPTCHA validation attempt using the stored procedure
    pub async fn record_captcha_validation_attempt(
        &self,
        challenge_id: Uuid,
        ip_address: IpAddr,
        user_agent: Option<String>,
        answer_provided: String,
        success: bool,
        risk_score: Option<f32>,
    ) -> Result<Uuid> {
        let client = self.get_connection().await?;

        let confidence_score = risk_score.map(|s| 1.0 - s as f64);
        let risk_assessment = match risk_score {
            Some(s) if s > 0.8 => "Critical",
            Some(s) if s > 0.5 => "High",
            Some(s) if s > 0.25 => "Medium",
            _ => "Low",
        };

        let query = r#"
            SELECT record_captcha_validation($1, $2, $3, $4, $5, $6, $7, $8) as attempt_id
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &challenge_id,
                    &ip_address,
                    &user_agent,
                    &answer_provided,
                    &success,
                    &confidence_score,
                    &risk_assessment,
                    &None::<Uuid>, // behavioral_metrics_id
                ],
            )
            .await?;

        let attempt_id: Uuid = row.get("attempt_id");
        Ok(attempt_id)
    }
}
