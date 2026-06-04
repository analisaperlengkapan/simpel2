//! PostgreSQL-backed token revocation list (F2H).
//!
//! Lets authenc invalidate access tokens **before** their `exp` at three
//! granularities — single token (`jti`), whole session (`sid`, used by logout),
//! and all of a user's tokens issued before a cutoff (`user`, used on role
//! change / account deactivation).
//!
//! The validate/verify path calls [`PostgresRevocationStore::is_revoked`] right
//! after JWT signature/expiry verification; a single indexed `EXISTS` query
//! covers all three kinds in one round-trip.

use std::sync::Arc;

use authenc_types::Result;
use chrono::{DateTime, Utc};

use crate::Database;

/// PostgreSQL implementation of the token revocation list.
pub struct PostgresRevocationStore {
    db: Arc<Database>,
}

impl PostgresRevocationStore {
    /// Create a new store over the shared database pool.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Revoke a single token by its JWT ID (`jti`). `expires_at` should be the
    /// token's own `exp` so the row can be purged once the token would expire
    /// anyway.
    pub async fn revoke_jti(
        &self,
        jti: &str,
        expires_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<()> {
        self.insert("jti", jti, expires_at, reason).await
    }

    /// Revoke an entire session by its `sid` (logout). Every token carrying this
    /// `sid` is rejected until `expires_at`.
    pub async fn revoke_session(
        &self,
        sid: &str,
        expires_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<()> {
        self.insert("sid", sid, expires_at, reason).await
    }

    /// Revoke ALL tokens for a user issued before now (role change / disable).
    /// Tokens with `iat < revoked_at` are rejected; re-authenticated tokens
    /// (newer `iat`) remain valid. Re-revoking refreshes the cutoff.
    pub async fn revoke_user(
        &self,
        user_id: &str,
        expires_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<()> {
        // For 'user' the cutoff is revoked_at, so a repeat revoke must move it
        // forward — hence DO UPDATE (jti/sid are immutable → DO NOTHING).
        let sql = r#"
            INSERT INTO authenc.token_revocations (kind, value, revoked_at, expires_at, reason)
            VALUES ('user', $1, NOW(), $2, $3)
            ON CONFLICT (kind, value)
            DO UPDATE SET revoked_at = NOW(),
                          expires_at = EXCLUDED.expires_at,
                          reason     = EXCLUDED.reason
        "#;
        self.db
            .execute(sql, &[&user_id, &expires_at, &reason])
            .await?;
        Ok(())
    }

    async fn insert(
        &self,
        kind: &str,
        value: &str,
        expires_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<()> {
        let sql = r#"
            INSERT INTO authenc.token_revocations (kind, value, expires_at, reason)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (kind, value) DO NOTHING
        "#;
        self.db
            .execute(sql, &[&kind, &value, &expires_at, &reason])
            .await?;
        Ok(())
    }

    /// Returns `true` if the presented token is revoked by any rule. `iat` is the
    /// token's issued-at (Unix seconds), used for the `user` cutoff comparison.
    pub async fn is_revoked(
        &self,
        jti: &str,
        sid: Option<&str>,
        user_id: &str,
        iat: i64,
    ) -> Result<bool> {
        // Empty string never matches a real sid (sids are non-empty).
        let sid = sid.unwrap_or("");
        let sql = r#"
            SELECT EXISTS (
                SELECT 1 FROM authenc.token_revocations r
                WHERE r.expires_at > NOW() AND (
                       (r.kind = 'jti'  AND r.value = $1)
                    OR (r.kind = 'sid'  AND r.value = $2)
                    OR (r.kind = 'user' AND r.value = $3 AND r.revoked_at > to_timestamp($4))
                )
            )
        "#;
        let row = self
            .db
            .query_one(sql, &[&jti, &sid, &user_id, &iat])
            .await?;
        Ok(row.get::<_, bool>(0))
    }

    /// Purge revocation rows that have passed their `expires_at`. Returns the
    /// number of rows removed. Intended to be called periodically.
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let n = self
            .db
            .execute(
                "DELETE FROM authenc.token_revocations WHERE expires_at <= NOW()",
                &[],
            )
            .await?;
        Ok(n)
    }
}
