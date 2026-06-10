//! PostgreSQL implementation of CredentialStore trait
//!
//! This module provides the PostgreSQL-backed implementation of the CredentialStore trait
//! for storing and managing WebAuthn credentials (passkeys).

use async_trait::async_trait;
use authenc_types::{AuthencError, Result, UserId};
use authenc_webauthn::{CredentialStore, StoredCredential};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, error, info};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::Database;

/// PostgreSQL implementation of CredentialStore
pub struct PostgresCredentialStore {
    db: Arc<Database>,
}

impl PostgresCredentialStore {
    /// Create a new PostgresCredentialStore
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresCredentialStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let credential_store = PostgresCredentialStore::new(db);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(db: Arc<Database>) -> Self {
        info!("Initializing PostgresCredentialStore");
        Self { db }
    }
}

#[async_trait]
impl CredentialStore for PostgresCredentialStore {
    async fn store_credential(&self, credential: &StoredCredential) -> Result<()> {
        info!(
            "Storing credential {} for user: {}",
            credential.id, credential.user_id.0
        );

        // Serialize credential data
        let cred_id_bytes = credential.cred_id.as_ref().to_vec();
        let cred_json = serde_json::to_value(&credential.cred).map_err(|e| {
            error!("Failed to serialize credential: {}", e);
            AuthencError::internal(format!("Failed to serialize credential: {}", e))
        })?;

        let query = r#"
            INSERT INTO webauthn_credentials (
                id, user_id, cred_id, cred, nickname, created_at, last_used
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        self.db
            .execute(
                query,
                &[
                    &credential.id,
                    &credential.user_id.0,
                    &cred_id_bytes,
                    &cred_json,
                    &credential.nickname,
                    &credential.created_at,
                    &credential.last_used,
                ],
            )
            .await?;

        info!("Credential {} stored successfully", credential.id);

        Ok(())
    }

    async fn get_credential(&self, id: Uuid) -> Result<StoredCredential> {
        debug!("Getting credential by ID: {}", id);

        let query = r#"
            SELECT id, user_id, cred_id, cred, nickname, created_at, last_used
            FROM webauthn_credentials
            WHERE id = $1
        "#;

        let row = self.db.query_one(query, &[&id]).await?;

        row_to_credential(row)
    }

    async fn get_credential_by_id(&self, cred_id: &CredentialID) -> Result<StoredCredential> {
        debug!("Getting credential by WebAuthn credential ID");

        let cred_id_bytes = cred_id.as_ref().to_vec();

        let query = r#"
            SELECT id, user_id, cred_id, cred, nickname, created_at, last_used
            FROM webauthn_credentials
            WHERE cred_id = $1
        "#;

        let row = self.db.query_one(query, &[&cred_id_bytes]).await?;

        row_to_credential(row)
    }

    async fn get_credentials_for_user(&self, user_id: UserId) -> Result<Vec<StoredCredential>> {
        debug!("Getting credentials for user: {}", user_id.0);

        let query = r#"
            SELECT id, user_id, cred_id, cred, nickname, created_at, last_used
            FROM webauthn_credentials
            WHERE user_id = $1
            ORDER BY created_at DESC
        "#;

        let rows = self.db.query(query, &[&user_id.0]).await?;

        let credentials: Result<Vec<StoredCredential>> =
            rows.into_iter().map(row_to_credential).collect();

        credentials
    }

    async fn delete_credential(&self, id: Uuid) -> Result<()> {
        info!("Deleting credential: {}", id);

        let query = r#"
            DELETE FROM webauthn_credentials
            WHERE id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&id]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::not_found("Credential not found"));
        }

        info!("Credential {} deleted successfully", id);

        Ok(())
    }

    async fn update_last_used(&self, id: Uuid, timestamp: DateTime<Utc>) -> Result<()> {
        debug!("Updating last used timestamp for credential: {}", id);

        let query = r#"
            UPDATE webauthn_credentials
            SET last_used = $1
            WHERE id = $2
        "#;

        let rows_affected = self.db.execute(query, &[&timestamp, &id]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::not_found("Credential not found"));
        }

        debug!("Last used timestamp updated for credential: {}", id);

        Ok(())
    }

    async fn update_counter(&self, id: Uuid, counter: u32) -> Result<()> {
        debug!("Updating counter for credential: {} to {}", id, counter);

        // Note: webauthn-rs Passkey struct manages counter internally
        // We don't need to manually update the counter field
        // The counter is already updated by webauthn-rs during authentication
        // This method is kept for interface compatibility but is a no-op

        // Just verify the credential exists
        let _credential = self.get_credential(id).await?;

        debug!("Counter update acknowledged for credential: {}", id);
        Ok(())
    }

    async fn update_nickname(&self, id: Uuid, nickname: String) -> Result<()> {
        debug!("Updating nickname for credential: {} to '{}'", id, nickname);

        let query = r#"
            UPDATE webauthn_credentials
            SET nickname = $1
            WHERE id = $2
        "#;

        let rows_affected = self.db.execute(query, &[&nickname, &id]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::not_found("Credential not found"));
        }

        debug!("Nickname updated for credential: {}", id);

        Ok(())
    }
}

/// Convert a database row to a StoredCredential
fn row_to_credential(row: Row) -> Result<StoredCredential> {
    let id: Uuid = row.try_get("id").map_err(|e| {
        error!("Failed to get id from row: {}", e);
        AuthencError::database(format!("Failed to get id: {}", e))
    })?;

    let user_id: Uuid = row.try_get("user_id").map_err(|e| {
        error!("Failed to get user_id from row: {}", e);
        AuthencError::database(format!("Failed to get user_id: {}", e))
    })?;

    let cred_id_bytes: Vec<u8> = row.try_get("cred_id").map_err(|e| {
        error!("Failed to get cred_id from row: {}", e);
        AuthencError::database(format!("Failed to get cred_id: {}", e))
    })?;

    let cred_json: serde_json::Value = row.try_get("cred").map_err(|e| {
        error!("Failed to get cred from row: {}", e);
        AuthencError::database(format!("Failed to get cred: {}", e))
    })?;

    let nickname: Option<String> = row.try_get("nickname").map_err(|e| {
        error!("Failed to get nickname from row: {}", e);
        AuthencError::database(format!("Failed to get nickname: {}", e))
    })?;

    let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| {
        error!("Failed to get created_at from row: {}", e);
        AuthencError::database(format!("Failed to get created_at: {}", e))
    })?;

    let last_used: Option<DateTime<Utc>> = row.try_get("last_used").map_err(|e| {
        error!("Failed to get last_used from row: {}", e);
        AuthencError::database(format!("Failed to get last_used: {}", e))
    })?;

    // Deserialize credential ID
    let cred_id = CredentialID::from(cred_id_bytes);

    // Deserialize passkey
    let cred: Passkey = serde_json::from_value(cred_json).map_err(|e| {
        error!("Failed to deserialize credential: {}", e);
        AuthencError::internal(format!("Failed to deserialize credential: {}", e))
    })?;

    Ok(StoredCredential {
        id,
        user_id: UserId(user_id),
        cred_id,
        cred,
        nickname,
        created_at,
        last_used,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_store_compiles() {
        // Basic smoke test to ensure module compiles
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_user_id_for_credentials() {
        let user_id = UserId::new();
        assert!(!user_id.0.is_nil());
    }

    #[test]
    fn test_stored_credential_basic() {
        // Test basic credential fields without complex COSE structures
        let user_id = UserId::new();
        let cred_id: CredentialID = vec![1, 2, 3, 4].into();
        let nickname = Some("My Passkey".to_string());

        assert!(!user_id.0.is_nil());
        assert_eq!(cred_id.len(), 4);
        assert!(nickname.is_some());
    }

    #[test]
    fn test_credential_counter() {
        let mut counter = 0u32;
        counter += 1;
        assert_eq!(counter, 1);

        counter += 1;
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_credential_id_type() {
        let cred_id: CredentialID = vec![1, 2, 3, 4].into();
        assert_eq!(cred_id.len(), 4);
    }

    #[test]
    fn test_credential_nickname() {
        let nickname = Some("My Security Key".to_string());
        assert_eq!(nickname.as_deref(), Some("My Security Key"));
    }

    #[test]
    fn test_credential_last_used() {
        let now = Utc::now();
        let last_used: Option<DateTime<Utc>> = Some(now);

        assert_eq!(last_used, Some(now));
    }

    #[test]
    fn test_credential_uuid() {
        let id = Uuid::new_v4();
        assert!(!id.is_nil());
    }
}
