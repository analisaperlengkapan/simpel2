//! User Synchronization Service
//!
//! Provides periodic synchronization of users from external identity providers:
//! - LDAP/Active Directory batch import
//! - Incremental user updates
//! - Scheduled sync jobs
//! - Manual sync triggers

use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::User;
use crate::services::federation_manager::{FederationManager, FederationProviderType};
use crate::spi::ldap_federation::LdapFederationProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Sync result statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Provider alias
    pub provider_alias: String,
    /// Number of users added
    pub users_added: usize,
    /// Number of users updated
    pub users_updated: usize,
    /// Number of users removed
    pub users_removed: usize,
    /// Number of failed operations
    pub failed: usize,
    /// Total processing time in milliseconds
    pub duration_ms: u64,
    /// Sync timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Error message if sync failed
    pub error: Option<String>,
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    /// Sync is running
    Running,
    /// Sync completed successfully
    Completed,
    /// Sync failed
    Failed,
    /// Sync is scheduled but not started
    Scheduled,
}

/// Sync job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: Uuid,
    pub provider_alias: String,
    pub realm_id: Uuid,
    pub status: SyncStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<SyncResult>,
}

/// User Sync Service
pub struct UserSyncService {
    db: Arc<Database>,
    federation_manager: Arc<FederationManager>,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl UserSyncService {
    /// Create a new UserSyncService
    pub fn new(db: Arc<Database>, federation_manager: Arc<FederationManager>) -> Self {
        Self {
            db,
            federation_manager,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Start periodic sync scheduler
    pub async fn start_scheduler(&self, interval_minutes: u64) {
        info!("Starting user sync scheduler with interval: {} minutes", interval_minutes);

        let db = self.db.clone();
        let federation_manager = self.federation_manager.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));

            loop {
                interval.tick().await;

                // Check if sync is already running
                {
                    let running = is_running.read().await;
                    if *running {
                        warn!("Sync is already running, skipping this interval");
                        continue;
                    }
                }

                info!("Starting scheduled user sync...");

                // Set running flag
                {
                    let mut running = is_running.write().await;
                    *running = true;
                }

                // Run sync for all configured providers
                let service = UserSyncService {
                    db: db.clone(),
                    federation_manager: federation_manager.clone(),
                    is_running: is_running.clone(),
                };

                match service.sync_all_providers().await {
                    Ok(results) => {
                        info!("Scheduled sync completed: {:?}", results);
                    }
                    Err(e) => {
                        error!("Scheduled sync failed: {}", e);
                    }
                }

                // Clear running flag
                {
                    let mut running = is_running.write().await;
                    *running = false;
                }
            }
        });

        info!("User sync scheduler started");
    }

    /// Sync all configured providers
    pub async fn sync_all_providers(&self) -> Result<Vec<SyncResult>> {
        info!("Starting sync for all providers");

        let mut results = Vec::new();

        // Get all realms (simplified - in production you'd query realms table)
        let realms = self.get_all_realms().await?;

        for realm_id in realms {
            // Get all identity providers for this realm
            let providers = self.federation_manager.list_identity_providers(realm_id).await?;

            for provider_config in providers {
                // Only sync LDAP/AD providers
                if !matches!(
                    provider_config.provider_type,
                    FederationProviderType::Ldap | FederationProviderType::ActiveDirectory
                ) {
                    continue;
                }

                if !provider_config.enabled {
                    continue;
                }

                // Check if sync is enabled in config
                let sync_enabled = provider_config
                    .config
                    .get("sync_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if !sync_enabled {
                    info!("Sync disabled for provider: {}", provider_config.alias);
                    continue;
                }

                info!("Syncing provider: {} (realm: {})", provider_config.alias, realm_id);

                match self.sync_provider(&provider_config.alias, realm_id).await {
                    Ok(result) => {
                        info!("Sync completed for {}: {:?}", provider_config.alias, result);
                        results.push(result);
                    }
                    Err(e) => {
                        error!("Sync failed for {}: {}", provider_config.alias, e);
                        results.push(SyncResult {
                            provider_alias: provider_config.alias.clone(),
                            users_added: 0,
                            users_updated: 0,
                            users_removed: 0,
                            failed: 0,
                            duration_ms: 0,
                            timestamp: chrono::Utc::now(),
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        info!("Sync for all providers completed. Total results: {}", results.len());
        Ok(results)
    }

    /// Sync users from a specific provider
    pub async fn sync_provider(&self, provider_alias: &str, realm_id: Uuid) -> Result<SyncResult> {
        let start_time = std::time::Instant::now();
        info!("Starting sync for provider: {} in realm: {}", provider_alias, realm_id);

        let mut users_added = 0;
        let mut users_updated = 0;
        let mut failed = 0;

        // Get LDAP provider from federation manager
        let provider = self
            .federation_manager
            .get_ldap_provider(provider_alias)
            .await
            .ok_or_else(|| {
                AuthencError::not_found(format!("Provider '{}' not found", provider_alias))
            })?;

        // Get provider configuration to determine batch size
        let config = self.get_provider_config(provider_alias, realm_id).await?;
        let batch_size = config
            .get("batch_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        // Search for all users in LDAP (using wildcard)
        let ldap_users = provider.search_users("*", batch_size * 10).await?;

        info!("Found {} users in LDAP", ldap_users.len());

        // Process users in batches
        for chunk in ldap_users.chunks(batch_size) {
            for ldap_user in chunk {
                match self.sync_user(ldap_user, provider_alias, realm_id).await {
                    Ok(is_new) => {
                        if is_new {
                            users_added += 1;
                        } else {
                            users_updated += 1;
                        }
                    }
                    Err(e) => {
                        error!("Failed to sync user {}: {}", ldap_user.username, e);
                        failed += 1;
                    }
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Sync completed for {}: added={}, updated={}, failed={}, duration={}ms",
            provider_alias, users_added, users_updated, failed, duration_ms
        );

        Ok(SyncResult {
            provider_alias: provider_alias.to_string(),
            users_added,
            users_updated,
            users_removed: 0, // Not implemented yet
            failed,
            duration_ms,
            timestamp: chrono::Utc::now(),
            error: None,
        })
    }

    /// Sync a single user from LDAP
    async fn sync_user(&self, ldap_user: &User, provider_alias: &str, realm_id: Uuid) -> Result<bool> {
        let client = self.db.get_connection().await?;

        // Check if user already exists via identity link
        let existing_link = client
            .query_opt(
                "SELECT user_id FROM federated_identity_links
                 WHERE identity_provider_alias = $1 AND federated_user_id = $2 AND realm_id = $3",
                &[&provider_alias, &ldap_user.username, &realm_id],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to check existing link: {}", e)))?;

        if let Some(row) = existing_link {
            // User exists - update attributes
            let user_id: Uuid = row.get(0);

            client
                .execute(
                    "UPDATE users
                     SET email = $2,
                         first_name = $3,
                         last_name = $4,
                         attributes = $5,
                         updated_at = NOW()
                     WHERE id = $1",
                    &[
                        &user_id,
                        &ldap_user.email,
                        &ldap_user.first_name,
                        &ldap_user.last_name,
                        &ldap_user.attributes,
                    ],
                )
                .await
                .map_err(|e| AuthencError::database(format!("Failed to update user: {}", e)))?;

            // Update identity link
            client
                .execute(
                    "UPDATE federated_identity_links
                     SET federated_attributes = $2,
                         updated_at = NOW()
                     WHERE user_id = $1 AND identity_provider_alias = $3",
                    &[
                        &user_id,
                        &serde_json::to_value(ldap_user).ok(),
                        &provider_alias,
                    ],
                )
                .await
                .map_err(|e| AuthencError::database(format!("Failed to update identity link: {}", e)))?;

            Ok(false) // Not new
        } else {
            // User doesn't exist - create new user and link
            let new_user_id = Uuid::new_v4();

            client
                .execute(
                    "INSERT INTO users (id, username, email, satker_code, first_name, last_name,
                                       email_verified, federated, attributes, realm_id, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, true, true, $7, $8, NOW(), NOW())",
                    &[
                        &new_user_id,
                        &ldap_user.username,
                        &ldap_user.email,
                        &ldap_user.satker_code,
                        &ldap_user.first_name,
                        &ldap_user.last_name,
                        &ldap_user.attributes,
                        &realm_id,
                    ],
                )
                .await
                .map_err(|e| AuthencError::database(format!("Failed to create user: {}", e)))?;

            // Create identity link
            let link_id = Uuid::new_v4();
            client
                .execute(
                    "INSERT INTO federated_identity_links
                     (id, user_id, realm_id, identity_provider_alias, federated_user_id,
                      federated_username, federated_attributes, linked_at, authentication_count)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), 0)",
                    &[
                        &link_id,
                        &new_user_id,
                        &realm_id,
                        &provider_alias,
                        &ldap_user.username,
                        &Some(&ldap_user.username),
                        &serde_json::to_value(ldap_user).ok(),
                    ],
                )
                .await
                .map_err(|e| AuthencError::database(format!("Failed to create identity link: {}", e)))?;

            Ok(true) // New user
        }
    }

    /// Get provider configuration
    async fn get_provider_config(&self, provider_alias: &str, realm_id: Uuid) -> Result<serde_json::Value> {
        let client = self.db.get_connection().await?;

        let row = client
            .query_one(
                "SELECT config FROM identity_broker_configs
                 WHERE alias = $1 AND realm_id = $2",
                &[&provider_alias, &realm_id],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get provider config: {}", e)))?;

        Ok(row.get(0))
    }

    /// Get all realm IDs (simplified version)
    async fn get_all_realms(&self) -> Result<Vec<Uuid>> {
        let client = self.db.get_connection().await?;

        let rows = client
            .query("SELECT id FROM realms WHERE enabled = true", &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get realms: {}", e)))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    /// Get sync history for a provider
    pub async fn get_sync_history(
        &self,
        provider_alias: &str,
        realm_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SyncResult>> {
        // In production, you'd store sync results in a separate table
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Check if sync is currently running
    pub async fn is_sync_running(&self) -> bool {
        *self.is_running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status() {
        assert_eq!(SyncStatus::Running, SyncStatus::Running);
        assert_eq!(SyncStatus::Completed, SyncStatus::Completed);
        assert_ne!(SyncStatus::Running, SyncStatus::Completed);
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult {
            provider_alias: "test-ldap".to_string(),
            users_added: 10,
            users_updated: 5,
            users_removed: 2,
            failed: 1,
            duration_ms: 1500,
            timestamp: chrono::Utc::now(),
            error: None,
        };

        assert_eq!(result.users_added, 10);
        assert_eq!(result.users_updated, 5);
        assert_eq!(result.failed, 1);
        assert!(result.error.is_none());
    }
}
