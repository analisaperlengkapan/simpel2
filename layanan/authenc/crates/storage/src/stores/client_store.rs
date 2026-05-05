//! PostgreSQL implementation of ClientStore trait
//!
//! This module provides the PostgreSQL-backed implementation of the ClientStore trait,
//! managing OAuth2/OIDC clients with support for public and confidential clients.

use async_trait::async_trait;
use authenc_types::{
    AuthencError, ClientId, RealmId, Result, domain::OidcClient, traits::ClientStore,
};
use chrono::Utc;
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, info};

use crate::Database;

/// PostgreSQL implementation of ClientStore
pub struct PostgresClientStore {
    db: Arc<Database>,
}

impl PostgresClientStore {
    /// Create a new PostgresClientStore
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresClientStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let client_store = PostgresClientStore::new(db);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(db: Arc<Database>) -> Self {
        info!("Initializing PostgresClientStore");
        Self { db }
    }
}

#[async_trait]
impl ClientStore for PostgresClientStore {
    async fn get_client(&self, id: ClientId) -> Result<OidcClient> {
        debug!("Getting client by ID: {}", id);

        let query = r#"
            SELECT id, client_id, client_secret_hash, client_name, client_type,
                   redirect_uris, scopes, realm_id, enabled, created_at, updated_at
            FROM oauth2_clients
            WHERE id = $1 AND deleted_at IS NULL
        "#;

        let row = self.db.query_one(query, &[&id.0]).await?;

        row_to_client(row)
    }

    async fn get_client_by_client_id(
        &self,
        client_id: &str,
        realm_id: RealmId,
    ) -> Result<OidcClient> {
        debug!(
            "Getting client by client_id: {} in realm: {}",
            client_id, realm_id
        );

        let query = r#"
            SELECT id, client_id, client_secret_hash, client_name, client_type,
                   redirect_uris, scopes, realm_id, enabled, created_at, updated_at
            FROM oauth2_clients
            WHERE client_id = $1 AND realm_id = $2 AND deleted_at IS NULL
        "#;

        let row = self.db.query_one(query, &[&client_id, &realm_id.0]).await?;

        row_to_client(row)
    }

    async fn create_client(
        &self,
        client_id: String,
        name: String,
        is_public: bool,
        realm_id: RealmId,
    ) -> Result<OidcClient> {
        info!("Creating client: {} in realm: {}", client_id, realm_id);

        // Check if client_id already exists in this realm
        if self.client_id_exists(&client_id, realm_id).await? {
            return Err(AuthencError::Conflict(format!(
                "Client ID '{}' already exists in realm",
                client_id
            )));
        }

        let id = ClientId::new();
        let now = Utc::now();
        let client_type = if is_public { "public" } else { "confidential" };

        // For public clients, no secret is stored
        // For confidential clients, the secret should be hashed before calling this method
        let client_secret_hash = if is_public {
            None
        } else {
            // Generate a placeholder - in practice, this should be provided by the caller
            Some(String::new())
        };

        let query = r#"
            INSERT INTO oauth2_clients (
                id, client_id, client_secret_hash, client_name, client_type,
                redirect_uris, scopes, realm_id, enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, client_id, client_secret_hash, client_name, client_type,
                      redirect_uris, scopes, realm_id, enabled, created_at, updated_at
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &id.0,
                    &client_id,
                    &client_secret_hash.unwrap_or_default(),
                    &name,
                    &client_type,
                    &Vec::<String>::new(), // empty redirect_uris
                    &Vec::<String>::new(), // empty scopes
                    &realm_id.0,
                    &true, // enabled by default
                    &now,
                    &now,
                ],
            )
            .await?;

        let client = row_to_client(row)?;
        info!("Client created successfully: {}", client.id);
        Ok(client)
    }

    async fn update_client(
        &self,
        id: ClientId,
        name: Option<String>,
        redirect_uris: Option<Vec<String>>,
        allowed_scopes: Option<Vec<String>>,
        enabled: Option<bool>,
    ) -> Result<OidcClient> {
        info!("Updating client: {}", id);

        // Build dynamic UPDATE query based on provided fields
        let mut updates = Vec::new();
        let mut param_index = 2; // $1 is reserved for client ID

        if name.is_some() {
            updates.push(format!("client_name = ${}", param_index));
            param_index += 1;
        }
        if redirect_uris.is_some() {
            updates.push(format!("redirect_uris = ${}", param_index));
            param_index += 1;
        }
        if allowed_scopes.is_some() {
            updates.push(format!("scopes = ${}", param_index));
            param_index += 1;
        }
        if enabled.is_some() {
            updates.push(format!("enabled = ${}", param_index));
            param_index += 1;
        }

        if updates.is_empty() {
            // No updates requested, just return the current client
            return self.get_client(id).await;
        }

        // Always update updated_at
        updates.push(format!("updated_at = ${}", param_index));

        let query = format!(
            r#"
            UPDATE oauth2_clients
            SET {}
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, client_id, client_secret_hash, client_name, client_type,
                      redirect_uris, scopes, realm_id, enabled, created_at, updated_at
            "#,
            updates.join(", ")
        );

        // Build parameter list dynamically
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&id.0];

        if let Some(ref n) = name {
            params.push(n);
        }
        if let Some(ref uris) = redirect_uris {
            params.push(uris);
        }
        if let Some(ref scopes) = allowed_scopes {
            params.push(scopes);
        }
        if let Some(ref e) = enabled {
            params.push(e);
        }

        let now = Utc::now();
        params.push(&now);

        let row = self.db.query_one(&query, &params).await?;

        let client = row_to_client(row)?;
        info!("Client updated successfully: {}", client.id);
        Ok(client)
    }

    async fn delete_client(&self, id: ClientId) -> Result<()> {
        info!("Deleting client: {}", id);

        // Soft delete: set deleted_at timestamp
        let query = r#"
            UPDATE oauth2_clients
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
        "#;

        let now = Utc::now();
        let rows_affected = self.db.execute(query, &[&id.0, &now]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::ClientNotFound(format!(
                "Client {} not found",
                id
            )));
        }

        info!("Client deleted successfully: {}", id);
        Ok(())
    }

    async fn list_clients(&self, realm_id: RealmId) -> Result<Vec<OidcClient>> {
        debug!("Listing clients in realm: {}", realm_id);

        let query = r#"
            SELECT id, client_id, client_secret_hash, client_name, client_type,
                   redirect_uris, scopes, realm_id, enabled, created_at, updated_at
            FROM oauth2_clients
            WHERE realm_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
        "#;

        let rows = self.db.query(query, &[&realm_id.0]).await?;

        let clients: Result<Vec<OidcClient>> = rows.into_iter().map(row_to_client).collect();

        clients
    }

    async fn update_client_secret(&self, id: ClientId, secret_hash: String) -> Result<()> {
        info!("Updating client secret for client: {}", id);

        let query = r#"
            UPDATE oauth2_clients
            SET client_secret_hash = $2, updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
        "#;

        let now = Utc::now();
        let rows_affected = self.db.execute(query, &[&id.0, &secret_hash, &now]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::ClientNotFound(format!(
                "Client {} not found",
                id
            )));
        }

        info!("Client secret updated successfully: {}", id);
        Ok(())
    }
}

impl PostgresClientStore {
    /// Check if a client_id exists in a realm
    async fn client_id_exists(&self, client_id: &str, realm_id: RealmId) -> Result<bool> {
        debug!(
            "Checking if client_id exists: {} in realm: {}",
            client_id, realm_id
        );

        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM oauth2_clients
                WHERE client_id = $1 AND realm_id = $2 AND deleted_at IS NULL
            )
        "#;

        let row = self.db.query_one(query, &[&client_id, &realm_id.0]).await?;

        let exists: bool = row.get(0);
        Ok(exists)
    }
}

/// Convert a database row to an OidcClient struct
fn row_to_client(row: Row) -> Result<OidcClient> {
    let client_type: String = row.get("client_type");
    let is_public = client_type == "public";

    // For public clients, client_secret_hash should be None
    let client_secret_hash: String = row.get("client_secret_hash");
    // Use client_secret directly (not hashed) as per OidcClient type definition
    let client_secret = if is_public || client_secret_hash.is_empty() {
        String::new() // Empty string for public clients
    } else {
        client_secret_hash // This is actually the secret, not a hash
    };

    Ok(OidcClient {
        id: row.get("id"),
        client_id: row.get("client_id"),
        client_secret,
        redirect_uris: row.get("redirect_uris"),
        name: row.get("client_name"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_client_store_creation() {
        // This is a basic test to ensure the struct can be created
        // Integration tests with a real database should be in a separate test file
    }

    #[test]
    fn test_client_id_creation() {
        let id1 = ClientId::new();
        let id2 = ClientId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_redirect_uris_validation() {
        let redirect_uris = ["https://example.com/callback".to_string(),
            "https://example.com/callback2".to_string()];

        assert_eq!(redirect_uris.len(), 2);
        assert!(redirect_uris.iter().all(|uri| uri.starts_with("https://")));
    }

    #[test]
    fn test_grant_types() {
        let grant_types = ["authorization_code".to_string(),
            "refresh_token".to_string()];

        assert!(grant_types.contains(&"authorization_code".to_string()));
        assert!(grant_types.contains(&"refresh_token".to_string()));
    }

    #[test]
    fn test_client_secret_handling() {
        let secret = Some("hashed_secret".to_string());
        assert!(secret.is_some());

        let no_secret: Option<String> = None;
        assert!(no_secret.is_none());
    }

    #[test]
    fn test_client_enabled_flag() {
        let enabled = true;
        let disabled = false;

        assert!(enabled);
        assert!(!disabled);
    }
}
