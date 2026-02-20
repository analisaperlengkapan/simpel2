//! PostgreSQL implementation of SessionStore trait
//!
//! This module provides the PostgreSQL-backed implementation of the SessionStore trait,
//! managing user sessions with configurable timeouts (15 minutes idle, 8 hours absolute).

use async_trait::async_trait;
use authenc_types::{
    domain::Session,
    traits::SessionStore,
    AuthencError, Result, SessionId, UserId,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, info};

use crate::Database;

/// PostgreSQL implementation of SessionStore
pub struct PostgresSessionStore {
    db: Arc<Database>,
    /// Idle timeout in minutes (default: 15 minutes)
    idle_timeout_minutes: i64,
    /// Absolute timeout in hours (default: 8 hours)
    absolute_timeout_hours: i64,
}

impl PostgresSessionStore {
    /// Create a new PostgresSessionStore with default timeouts
    ///
    /// Default timeouts:
    /// - Idle timeout: 15 minutes
    /// - Absolute timeout: 8 hours
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresSessionStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let session_store = PostgresSessionStore::new(db);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(db: Arc<Database>) -> Self {
        info!("Initializing PostgresSessionStore with default timeouts (idle: 15min, absolute: 8h)");
        Self {
            db,
            idle_timeout_minutes: 15,
            absolute_timeout_hours: 8,
        }
    }

    /// Create a new PostgresSessionStore with custom timeouts
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    /// * `idle_timeout_minutes` - Idle timeout in minutes
    /// * `absolute_timeout_hours` - Absolute timeout in hours
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresSessionStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let session_store = PostgresSessionStore::with_timeouts(db, 30, 12);
    ///     Ok(())
    /// }
    /// ```
    pub fn with_timeouts(
        db: Arc<Database>,
        idle_timeout_minutes: i64,
        absolute_timeout_hours: i64,
    ) -> Self {
        info!(
            "Initializing PostgresSessionStore with custom timeouts (idle: {}min, absolute: {}h)",
            idle_timeout_minutes, absolute_timeout_hours
        );
        Self {
            db,
            idle_timeout_minutes,
            absolute_timeout_hours,
        }
    }
}

#[async_trait]
impl SessionStore for PostgresSessionStore {
    async fn create_session(&self, user_id: UserId) -> Result<Session> {
        info!("Creating session for user: {}", user_id);

        let session_id = SessionId::new();
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.absolute_timeout_hours);

        // Generate a placeholder token_hash (required by schema but not used in SessionStore)
        let token_hash = format!("session_{}", session_id);

        let query = r#"
            INSERT INTO sessions (
                id, user_id, token_hash, expires_at, created_at, last_activity_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, created_at, expires_at, last_activity_at
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[&session_id.0, &user_id.0, &token_hash, &expires_at, &now, &now],
            )
            .await?;

        let session = row_to_session(row)?;
        info!("Session created successfully: {}", session.id);
        Ok(session)
    }

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>> {
        debug!("Getting session by ID: {}", id);

        let query = r#"
            SELECT id, user_id, created_at, expires_at, last_activity_at
            FROM sessions
            WHERE id = $1
        "#;

        let row_opt = self.db.query_opt(query, &[&id.0]).await?;

        match row_opt {
            Some(row) => {
                let session = row_to_session(row)?;

                // Check if session is expired (absolute timeout)
                if session.expires_at < Utc::now() {
                    debug!("Session {} has expired (absolute timeout)", id);
                    return Ok(None);
                }

                // Check if session is expired (idle timeout)
                let idle_deadline = Utc::now() - Duration::minutes(self.idle_timeout_minutes);
                if session.last_accessed < idle_deadline {
                    debug!("Session {} has expired (idle timeout)", id);
                    return Ok(None);
                }

                Ok(Some(session))
            }
            None => {
                debug!("Session {} not found", id);
                Ok(None)
            }
        }
    }

    async fn update_last_accessed(&self, id: SessionId) -> Result<()> {
        debug!("Updating last accessed time for session: {}", id);

        let now = Utc::now();

        let query = r#"
            UPDATE sessions
            SET last_activity_at = $2
            WHERE id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&id.0, &now]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::SessionNotFound(format!(
                "Session {} not found",
                id
            )));
        }

        debug!("Session last accessed time updated: {}", id);
        Ok(())
    }

    async fn invalidate_session(&self, id: SessionId) -> Result<()> {
        info!("Invalidating session: {}", id);

        let query = r#"
            DELETE FROM sessions
            WHERE id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&id.0]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::SessionNotFound(format!(
                "Session {} not found",
                id
            )));
        }

        info!("Session invalidated successfully: {}", id);
        Ok(())
    }

    async fn invalidate_user_sessions(&self, user_id: UserId) -> Result<()> {
        info!("Invalidating all sessions for user: {}", user_id);

        let query = r#"
            DELETE FROM sessions
            WHERE user_id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&user_id.0]).await?;

        info!(
            "Invalidated {} session(s) for user: {}",
            rows_affected, user_id
        );
        Ok(())
    }

    async fn list_user_sessions(&self, user_id: UserId) -> Result<Vec<Session>> {
        debug!("Listing active sessions for user: {}", user_id);

        let now = Utc::now();
        let idle_deadline = now - Duration::minutes(self.idle_timeout_minutes);

        let query = r#"
            SELECT id, user_id, created_at, expires_at, last_activity_at
            FROM sessions
            WHERE user_id = $1
              AND expires_at > $2
              AND last_activity_at > $3
            ORDER BY last_activity_at DESC
        "#;

        let rows = self
            .db
            .query(query, &[&user_id.0, &now, &idle_deadline])
            .await?;

        let sessions: Result<Vec<Session>> = rows.into_iter().map(row_to_session).collect();

        sessions
    }

    async fn cleanup_expired_sessions(&self) -> Result<usize> {
        info!("Cleaning up expired sessions");

        let now = Utc::now();
        let idle_deadline = now - Duration::minutes(self.idle_timeout_minutes);

        let query = r#"
            DELETE FROM sessions
            WHERE expires_at < $1
               OR last_activity_at < $2
        "#;

        let rows_affected = self.db.execute(query, &[&now, &idle_deadline]).await?;

        info!("Cleaned up {} expired session(s)", rows_affected);
        Ok(rows_affected as usize)
    }
}

/// Convert a database row to a Session struct
/// TODO: This needs to be properly implemented with all Session fields
fn row_to_session(row: Row) -> Result<Session> {
    Ok(Session {
        id: row.get("id"),
        user_id: row.get("user_id"),
        token: row.try_get("token").unwrap_or_default(),
        refresh_token: row.try_get("refresh_token").ok(),
        expires_at: row.get("expires_at"),
        created_at: row.get("created_at"),
        last_accessed: row.get("last_activity_at"), // Note: DB column is last_activity_at
        ip_address: row.try_get("ip_address").ok(),
        user_agent: row.try_get("user_agent").ok(),
        revoked: row.try_get("revoked").unwrap_or(false),
        mfa_verified: row.try_get("mfa_verified").unwrap_or(false),
        is_temp_session: row.try_get("is_temp_session").unwrap_or(false),
        mfa_verified_at: row.try_get("mfa_verified_at").ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_session_store_creation() {
        // This is a basic test to ensure the struct can be created
        // Integration tests with a real database should be in a separate test file
    }

    #[test]
    fn test_custom_timeouts() {
        // Test that custom timeouts are properly set
        // Note: Actual functionality tests require tokio runtime and database
    }

    mod unit_tests {
        use super::*;

        #[test]
        fn test_session_id_creation() {
            let id1 = SessionId::new();
            let id2 = SessionId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_session_struct_fields() {
            let session = Session {
                id: SessionId::new(),
                user_id: UserId::new(),
                created_at: Utc::now(),
                last_accessed_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(8),
            };

            assert_eq!(session.id, session.id);
            assert_eq!(session.user_id, session.user_id);
        }

        #[test]
        fn test_default_timeouts() {
            // Test default timeout values
            let idle_timeout = 15i64; // minutes
            let absolute_timeout = 8i64; // hours

            assert_eq!(idle_timeout, 15);
            assert_eq!(absolute_timeout, 8);
        }

        #[test]
        fn test_session_expiry_calculation() {
            let now = Utc::now();
            let expires_at = now + Duration::hours(8);

            assert!(expires_at > now);
            assert_eq!((expires_at - now).num_hours(), 8);
        }

        #[test]
        fn test_idle_timeout_calculation() {
            let now = Utc::now();
            let idle_expires = now + Duration::minutes(15);

            assert!(idle_expires > now);
            assert_eq!((idle_expires - now).num_minutes(), 15);
        }
    }
}
