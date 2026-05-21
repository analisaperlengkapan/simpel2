//! Configuration Management Service
//!
//! Provides centralized configuration management with:
//! - Database-backed configuration storage
//! - Secreton integration for secrets
//! - Hot reload capability
//! - Complete audit trail
//! - Caching for performance

use authenc_storage::Database;
use authenc_types::{AuthencError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Configuration cache entry
#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub value: Value,
    pub cached_at: std::time::Instant,
}

/// Configuration Manager
///
/// Manages application configuration with support for:
/// - Loading from database
/// - Caching for performance
/// - Hot reload without restart
/// - Audit trail for all changes
#[derive(Clone)]
pub struct ConfigManager {
    database: Arc<Database>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: u64,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new(database: Arc<Database>, cache_ttl: u64) -> Self {
        Self {
            database,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Get configuration value
    pub async fn get(&self, key: &str) -> Result<Value> {
        // Check cache first
        if let Some(entry) = self.get_from_cache(key).await {
            debug!("Config cache hit: {}", key);
            return Ok(entry.value);
        }

        // Load from database
        let value = self.load_from_database(key).await?;

        // Cache the value
        self.set_cache(key, value.clone()).await;

        Ok(value)
    }

    /// Get all configuration for a category
    pub async fn get_category(&self, category: &str) -> Result<HashMap<String, Value>> {
        let query = r#"
            SELECT key, value FROM authenc.configuration
            WHERE category = $1 AND is_secret = false
            ORDER BY key
        "#;

        let rows = self.database.query(query, &[&category]).await?;

        let mut result = HashMap::new();
        for row in rows {
            let key: String = row.get("key");
            let value: Value = row.get("value");
            result.insert(key, value);
        }

        Ok(result)
    }

    /// Get all non-secret configuration
    pub async fn get_all(&self) -> Result<HashMap<String, Value>> {
        let query = r#"
            SELECT key, value FROM authenc.configuration
            WHERE is_secret = false
            ORDER BY key
        "#;

        let rows = self.database.query(query, &[]).await?;

        let mut result = HashMap::new();
        for row in rows {
            let key: String = row.get("key");
            let value: Value = row.get("value");
            result.insert(key, value);
        }

        Ok(result)
    }

    /// Set configuration value
    pub async fn set(
        &self,
        key: &str,
        value: Value,
        user_id: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        // Get old value for audit trail
        let old_value = self.load_from_database(key).await.ok();

        // Update in database
        let query = r#"
            INSERT INTO authenc.configuration (key, value, updated_at, updated_by)
            VALUES ($1, $2, CURRENT_TIMESTAMP, $3)
            ON CONFLICT (key) DO UPDATE SET
                value = $2,
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $3,
                version = version + 1
        "#;

        let user_uuid = user_id.and_then(|id| uuid::Uuid::parse_str(id).ok());

        self.database
            .execute(query, &[&key, &value.to_string(), &user_uuid])
            .await?;

        // Record in audit trail
        self.record_config_change(key, old_value, value.clone(), user_id, reason)
            .await?;

        // Invalidate cache
        self.invalidate_cache(key).await;

        info!("Configuration updated: {} (reason: {:?})", key, reason);

        Ok(())
    }

    /// Delete configuration value
    pub async fn delete(
        &self,
        key: &str,
        user_id: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        // Get old value for audit trail
        let old_value = self.load_from_database(key).await.ok();

        // Delete from database
        let query = "DELETE FROM authenc.configuration WHERE key = $1";
        self.database.execute(query, &[&key]).await?;

        // Record in audit trail
        if let Some(old_val) = old_value {
            self.record_config_change(key, Some(old_val), Value::Null, user_id, reason)
                .await?;
        }

        // Invalidate cache
        self.invalidate_cache(key).await;

        info!("Configuration deleted: {}", key);

        Ok(())
    }

    /// Reload all configuration (hot reload)
    pub async fn reload_all(&self) -> Result<()> {
        info!("Reloading all configuration from database");

        // Clear cache
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        // Verify database connection
        let query = "SELECT COUNT(*) FROM authenc.configuration";
        let row = self.database.query_one(query, &[]).await.map_err(|e| {
            error!("Failed to reload configuration: {}", e);
            AuthencError::database("Failed to reload configuration")
        })?;
        let _count: i64 = row.get(0);

        info!("Configuration reloaded successfully");

        Ok(())
    }

    /// Get configuration change history
    pub async fn get_history(&self, key: &str, limit: i64) -> Result<Vec<ConfigChange>> {
        let query = r#"
            SELECT
                ch.key,
                ch.old_value,
                ch.new_value,
                ch.change_reason,
                ch.created_at,
                u.username
            FROM authenc.configuration_history ch
            LEFT JOIN authenc.users u ON ch.changed_by = u.id
            WHERE ch.key = $1
            ORDER BY ch.created_at DESC
            LIMIT $2
        "#;

        let rows = self.database.query(query, &[&key, &limit]).await?;

        let changes = rows
            .into_iter()
            .map(|row| {
                let key: String = row.get("key");
                let old_value: Option<Value> = row.get("old_value");
                let new_value: Value = row.get("new_value");
                let reason: Option<String> = row.get("change_reason");
                let created_at: String = row.get("created_at");
                let changed_by: Option<String> = row.get("username");
                ConfigChange {
                    key,
                    old_value,
                    new_value,
                    reason,
                    created_at,
                    changed_by,
                }
            })
            .collect();

        Ok(changes)
    }

    // Private helper methods

    async fn get_from_cache(&self, key: &str) -> Option<CacheEntry> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|entry| {
            if entry.cached_at.elapsed().as_secs() < self.cache_ttl {
                Some(entry.clone())
            } else {
                None
            }
        })
    }

    async fn set_cache(&self, key: &str, value: Value) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry {
                value,
                cached_at: std::time::Instant::now(),
            },
        );
    }

    async fn invalidate_cache(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }

    async fn load_from_database(&self, key: &str) -> Result<Value> {
        let query = "SELECT value FROM authenc.configuration WHERE key = $1";

        let row = self.database.query_one(query, &[&key]).await.map_err(|_| {
            AuthencError::validation(format!("Configuration key not found: {}", key))
        })?;
        let raw: &str = row.get(0);
        let value: Value =
            serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()));

        Ok(value)
    }

    async fn record_config_change(
        &self,
        key: &str,
        old_value: Option<Value>,
        new_value: Value,
        user_id: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        let query = r#"
            INSERT INTO authenc.configuration_history
            (config_id, key, old_value, new_value, changed_by, change_reason, created_at)
            SELECT id, $1, $2, $3, $4, $5, CURRENT_TIMESTAMP
            FROM authenc.configuration
            WHERE key = $1
        "#;

        let user_uuid = user_id.and_then(|id| uuid::Uuid::parse_str(id).ok());

        self.database
            .execute(
                query,
                &[
                    &key,
                    &old_value.map(|v| v.to_string()),
                    &new_value.to_string(),
                    &user_uuid,
                    &reason,
                ],
            )
            .await?;

        Ok(())
    }
}

/// Configuration change record
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
    pub reason: Option<String>,
    pub created_at: String,
    pub changed_by: Option<String>,
}
