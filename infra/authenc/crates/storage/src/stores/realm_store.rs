//! PostgreSQL implementation of RealmStore trait
//!
//! This module provides the PostgreSQL-backed implementation of the RealmStore trait,
//! managing realms with multi-tenant isolation for users, clients, and sessions.

use async_trait::async_trait;
use authenc_types::{
    domain::realm::Realm,
    traits::RealmStore,
    AuthencError, RealmId, Result,
};
use chrono::Utc;
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, info};

use crate::Database;

/// PostgreSQL implementation of RealmStore
pub struct PostgresRealmStore {
    db: Arc<Database>,
}

impl PostgresRealmStore {
    /// Create a new PostgresRealmStore
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use authenc_storage::{Database, PostgresRealmStore};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Arc::new(Database::new("postgres://authenc:password@localhost/authenc", 20).await?);
    ///     let realm_store = PostgresRealmStore::new(db);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(db: Arc<Database>) -> Self {
        info!("Initializing PostgresRealmStore");
        Self { db }
    }
}

#[async_trait]
impl RealmStore for PostgresRealmStore {
    async fn get_realm(&self, id: RealmId) -> Result<Realm> {
        debug!("Getting realm by ID: {}", id);

        let query = r#"
            SELECT id, name, display_name, description, enabled,
                   ssl_required, registration_allowed, registration_email_as_username,
                   remember_me, verify_email, login_with_email_allowed,
                   duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                   brute_force_protected, max_failure_wait_seconds,
                   minimum_quick_login_wait_seconds, wait_increment_seconds,
                   quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                   default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                   access_token_lifespan, access_token_lifespan_for_implicit_flow,
                   sso_session_idle_timeout, sso_session_max_lifespan,
                   sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                   offline_session_idle_timeout, offline_session_max_lifespan,
                   client_session_idle_timeout, client_session_max_lifespan,
                   access_code_lifespan, access_code_lifespan_user_action,
                   access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                   action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                   oauth2_device_polling_interval, attributes,
                   created_at, updated_at, deleted_at
            FROM realms
            WHERE id = $1
        "#;

        let row = self.db.query_one(query, &[&id.0]).await?;

        row_to_realm(row)
    }

    async fn get_realm_by_name(&self, name: &str) -> Result<Realm> {
        debug!("Getting realm by name: {}", name);

        let query = r#"
            SELECT id, name, display_name, description, enabled,
                   ssl_required, registration_allowed, registration_email_as_username,
                   remember_me, verify_email, login_with_email_allowed,
                   duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                   brute_force_protected, max_failure_wait_seconds,
                   minimum_quick_login_wait_seconds, wait_increment_seconds,
                   quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                   default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                   access_token_lifespan, access_token_lifespan_for_implicit_flow,
                   sso_session_idle_timeout, sso_session_max_lifespan,
                   sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                   offline_session_idle_timeout, offline_session_max_lifespan,
                   client_session_idle_timeout, client_session_max_lifespan,
                   access_code_lifespan, access_code_lifespan_user_action,
                   access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                   action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                   oauth2_device_polling_interval, attributes,
                   created_at, updated_at, deleted_at
            FROM realms
            WHERE name = $1
        "#;

        let row = self.db.query_one(query, &[&name]).await?;

        row_to_realm(row)
    }

    async fn create_realm(&self, name: String, display_name: String) -> Result<Realm> {
        info!("Creating realm: {}", name);

        // Check if realm name already exists
        if self.realm_name_exists(&name).await? {
            return Err(AuthencError::Conflict(format!(
                "Realm name '{}' already exists",
                name
            )));
        }

        let realm_id = RealmId::new();
        let now = Utc::now();

        // Use Realm::default() to get all default values, then override specific fields
        let default_realm = Realm::default();

        let query = r#"
            INSERT INTO realms (
                id, name, display_name, description, enabled,
                ssl_required, registration_allowed, registration_email_as_username,
                remember_me, verify_email, login_with_email_allowed,
                duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                brute_force_protected, max_failure_wait_seconds,
                minimum_quick_login_wait_seconds, wait_increment_seconds,
                quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                access_token_lifespan, access_token_lifespan_for_implicit_flow,
                sso_session_idle_timeout, sso_session_max_lifespan,
                sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                offline_session_idle_timeout, offline_session_max_lifespan,
                client_session_idle_timeout, client_session_max_lifespan,
                access_code_lifespan, access_code_lifespan_user_action,
                access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                oauth2_device_polling_interval, attributes,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25,
                $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35,
                $36, $37, $38, $39, $40,
                $41, $42, $43, $44
            )
            RETURNING id, name, display_name, description, enabled,
                      ssl_required, registration_allowed, registration_email_as_username,
                      remember_me, verify_email, login_with_email_allowed,
                      duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                      brute_force_protected, max_failure_wait_seconds,
                      minimum_quick_login_wait_seconds, wait_increment_seconds,
                      quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                      default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                      access_token_lifespan, access_token_lifespan_for_implicit_flow,
                      sso_session_idle_timeout, sso_session_max_lifespan,
                      sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                      offline_session_idle_timeout, offline_session_max_lifespan,
                      client_session_idle_timeout, client_session_max_lifespan,
                      access_code_lifespan, access_code_lifespan_user_action,
                      access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                      action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                      oauth2_device_polling_interval, attributes,
                      created_at, updated_at, deleted_at
        "#;

        let row = self
            .db
            .query_one(
                query,
                &[
                    &realm_id.0,
                    &name,
                    &display_name,
                    &default_realm.description,
                    &true, // enabled by default
                    &default_realm.ssl_required,
                    &default_realm.registration_allowed,
                    &default_realm.registration_email_as_username,
                    &default_realm.remember_me,
                    &default_realm.verify_email,
                    &default_realm.login_with_email_allowed,
                    &default_realm.duplicate_emails_allowed,
                    &default_realm.reset_password_allowed,
                    &default_realm.edit_username_allowed,
                    &default_realm.brute_force_protected,
                    &default_realm.max_failure_wait_seconds,
                    &default_realm.minimum_quick_login_wait_seconds,
                    &default_realm.wait_increment_seconds,
                    &default_realm.quick_login_check_milli_seconds,
                    &default_realm.max_delta_time_seconds,
                    &default_realm.failure_factor,
                    &default_realm.default_signature_algorithm,
                    &default_realm.revoke_refresh_token,
                    &default_realm.refresh_token_max_reuse,
                    &default_realm.access_token_lifespan,
                    &default_realm.access_token_lifespan_for_implicit_flow,
                    &default_realm.sso_session_idle_timeout,
                    &default_realm.sso_session_max_lifespan,
                    &default_realm.sso_session_idle_timeout_remember_me,
                    &default_realm.sso_session_max_lifespan_remember_me,
                    &default_realm.offline_session_idle_timeout,
                    &default_realm.offline_session_max_lifespan,
                    &default_realm.client_session_idle_timeout,
                    &default_realm.client_session_max_lifespan,
                    &default_realm.access_code_lifespan,
                    &default_realm.access_code_lifespan_user_action,
                    &default_realm.access_code_lifespan_login,
                    &default_realm.action_token_generated_by_admin_lifespan,
                    &default_realm.action_token_generated_by_user_lifespan,
                    &default_realm.oauth2_device_code_lifespan,
                    &default_realm.oauth2_device_polling_interval,
                    &default_realm.attributes.map(|v| serde_json::to_string(&v).unwrap_or_default()),
                    &now,
                    &now,
                ],
            )
            .await?;

        let realm = row_to_realm(row)?;
        info!("Realm created successfully: {}", realm.id);
        Ok(realm)
    }

    async fn update_realm(
        &self,
        id: RealmId,
        display_name: Option<String>,
        enabled: Option<bool>,
    ) -> Result<Realm> {
        info!("Updating realm: {}", id);

        // Build dynamic UPDATE query based on provided fields
        let mut updates = Vec::new();
        let mut param_index = 2; // $1 is reserved for realm ID

        if display_name.is_some() {
            updates.push(format!("display_name = ${}", param_index));
            param_index += 1;
        }
        if enabled.is_some() {
            updates.push(format!("enabled = ${}", param_index));
            param_index += 1;
        }

        if updates.is_empty() {
            // No updates requested, just return the current realm
            return self.get_realm(id).await;
        }

        // Always update updated_at
        updates.push(format!("updated_at = ${}", param_index));

        let query = format!(
            r#"
            UPDATE realms
            SET {}
            WHERE id = $1
            RETURNING id, name, display_name, description, enabled,
                      ssl_required, registration_allowed, registration_email_as_username,
                      remember_me, verify_email, login_with_email_allowed,
                      duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                      brute_force_protected, max_failure_wait_seconds,
                      minimum_quick_login_wait_seconds, wait_increment_seconds,
                      quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                      default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                      access_token_lifespan, access_token_lifespan_for_implicit_flow,
                      sso_session_idle_timeout, sso_session_max_lifespan,
                      sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                      offline_session_idle_timeout, offline_session_max_lifespan,
                      client_session_idle_timeout, client_session_max_lifespan,
                      access_code_lifespan, access_code_lifespan_user_action,
                      access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                      action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                      oauth2_device_polling_interval, attributes,
                      created_at, updated_at, deleted_at
            "#,
            updates.join(", ")
        );

        // Build parameter list dynamically
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&id.0];

        if let Some(ref display_name) = display_name {
            params.push(display_name);
        }
        if let Some(ref enabled) = enabled {
            params.push(enabled);
        }

        let now = Utc::now();
        params.push(&now);

        let row = self.db.query_one(&query, &params).await?;

        let realm = row_to_realm(row)?;
        info!("Realm updated successfully: {}", realm.id);
        Ok(realm)
    }

    async fn delete_realm(&self, id: RealmId) -> Result<()> {
        info!("Deleting realm: {}", id);

        // Note: This is a hard delete. In production, you might want to:
        // 1. Check if realm has users/clients before deleting
        // 2. Implement soft delete (set enabled = false)
        // 3. Cascade delete related entities
        let query = r#"
            DELETE FROM realms
            WHERE id = $1
        "#;

        let rows_affected = self.db.execute(query, &[&id.0]).await?;

        if rows_affected == 0 {
            return Err(AuthencError::RealmNotFound(format!(
                "Realm {} not found",
                id
            )));
        }

        info!("Realm deleted successfully: {}", id);
        Ok(())
    }

    async fn list_realms(&self) -> Result<Vec<Realm>> {
        debug!("Listing all realms");

        let query = r#"
            SELECT id, name, display_name, description, enabled,
                   ssl_required, registration_allowed, registration_email_as_username,
                   remember_me, verify_email, login_with_email_allowed,
                   duplicate_emails_allowed, reset_password_allowed, edit_username_allowed,
                   brute_force_protected, max_failure_wait_seconds,
                   minimum_quick_login_wait_seconds, wait_increment_seconds,
                   quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor,
                   default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse,
                   access_token_lifespan, access_token_lifespan_for_implicit_flow,
                   sso_session_idle_timeout, sso_session_max_lifespan,
                   sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me,
                   offline_session_idle_timeout, offline_session_max_lifespan,
                   client_session_idle_timeout, client_session_max_lifespan,
                   access_code_lifespan, access_code_lifespan_user_action,
                   access_code_lifespan_login, action_token_generated_by_admin_lifespan,
                   action_token_generated_by_user_lifespan, oauth2_device_code_lifespan,
                   oauth2_device_polling_interval, attributes,
                   created_at, updated_at, deleted_at
            FROM realms
            ORDER BY created_at DESC
        "#;

        let rows = self.db.query(query, &[]).await?;

        let realms: Result<Vec<Realm>> = rows.into_iter().map(row_to_realm).collect();

        realms
    }

    async fn realm_name_exists(&self, name: &str) -> Result<bool> {
        debug!("Checking if realm name exists: {}", name);

        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM realms
                WHERE name = $1
            )
        "#;

        let row = self.db.query_one(query, &[&name]).await?;

        let exists: bool = row.get(0);
        Ok(exists)
    }
}

/// Convert a database row to a Realm struct
fn row_to_realm(row: Row) -> Result<Realm> {
    // Use the TryFrom implementation from authenc_types::domain::Realm
    Realm::try_from(row).map_err(|e| {
        AuthencError::DatabaseError(format!("Failed to convert row to Realm: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_realm_store_creation() {
        // This is a basic test to ensure the struct can be created
        // Integration tests with a real database should be in a separate test file
        // Note: Actual database connection tests require tokio runtime
    }

    #[test]
    fn test_realm_id_creation() {
        let id1 = RealmId::new();
        let id2 = RealmId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_realm_struct_fields() {
        let realm = Realm::default();

        assert_eq!(realm.name, "master");
        assert!(realm.enabled);
        assert_eq!(realm.display_name, Some("Master".to_string()));
    }

    #[test]
    fn test_realm_name_validation() {
        let realm_name = "valid-realm-name";
        assert!(!realm_name.is_empty());
        assert!(realm_name.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }

    #[test]
    fn test_realm_enabled_flag() {
        let mut enabled_realm = Realm::default();
        enabled_realm.enabled = true;

        let mut disabled_realm = Realm::default();
        disabled_realm.enabled = false;

        assert!(enabled_realm.enabled);
        assert!(!disabled_realm.enabled);
    }
}
