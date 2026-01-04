//! PostgreSQL storage backend implementation using tokio-postgres

use crate::{
    HealthStatus, QueryParams, SecurityLevel, StorageBackend, StorageError, StorageResult,
    StorageStats, StorageTransaction, VaultEntry,
};
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
#[cfg(feature = "metrics")]
use metrics::{counter, gauge};
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use std::time::Duration;
use tokio_postgres::{NoTls, Row};
use tokio_postgres_rustls::MakeRustlsConnect;
use uuid::Uuid;

/// PostgreSQL storage backend
pub struct PostgresBackend {
    pool: Pool,
    /// Pool exhaustion threshold (percentage)
    exhaustion_threshold: f64,
}

impl PostgresBackend {
    /// Create a new PostgreSQL backend
    pub async fn new(database_url: &str) -> StorageResult<Self> {
        let mut cfg = Config::new();
        cfg.url = Some(database_url.to_string());

        let tls_mode =
            std::env::var("SECRETON_STORAGE_TLS_MODE").unwrap_or_else(|_| "disable".to_string());

        let pool_result = if tls_mode.eq_ignore_ascii_case("disable") {
            cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        } else {
            let mut root_store = RootCertStore::empty();
            let certs = load_native_certs();
            for cert in certs.certs {
                let _ = root_store.add(cert);
            }

            cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);

            let tls_config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            let tls = MakeRustlsConnect::new(tls_config);

            cfg.create_pool(Some(Runtime::Tokio1), tls)
        };

        let pool = pool_result.map_err(|e| StorageError::ConnectionFailed {
            source: None,
            message: format!("Failed to create PostgreSQL pool: {}", e),
        })?;

        let backend = Self {
            pool,
            exhaustion_threshold: 0.8, // Alert at 80% pool usage
        };

        // Start background pool monitoring
        backend.start_pool_monitoring();

        Ok(backend)
    }

    /// Get the connection pool
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Start background task for pool monitoring
    fn start_pool_monitoring(&self) {
        let pool = self.pool.clone();
        let threshold = self.exhaustion_threshold;

        tokio::spawn(async move {
            let monitor_interval = Duration::from_secs(30); // Check every 30 seconds

            loop {
                tokio::time::sleep(monitor_interval).await;

                let status = pool.status();
                let max_size = status.max_size;
                let available = status.available;
                let size = status.size;

                // Calculate pool utilization
                let utilization = if max_size > 0 {
                    (size as f64) / (max_size as f64)
                } else {
                    0.0
                };

                // Log metrics
                tracing::debug!(
                    "PostgreSQL pool status: size={}/{}, available={}, utilization={:.1}%",
                    size,
                    max_size,
                    available,
                    utilization * 100.0
                );

                // Alert on high utilization
                if utilization >= threshold {
                    tracing::warn!(
                        "PostgreSQL pool exhaustion warning: {:.1}% utilized ({}/{}), {} available",
                        utilization * 100.0,
                        size,
                        max_size,
                        available
                    );

                    // Record metric for monitoring systems
                    #[cfg(feature = "metrics")]
                    {
                        gauge!("secreton_postgres_pool_utilization").set(utilization);
                        gauge!("secreton_postgres_pool_available").set(available as f64);
                        counter!("secreton_postgres_pool_exhaustion_warnings").increment(1);
                    }
                }

                // Alert on zero available connections
                if available == 0 && size > 0 {
                    tracing::error!(
                        "PostgreSQL pool exhausted: 0 connections available out of {} total",
                        size
                    );

                    #[cfg(feature = "metrics")]
                    {
                        counter!("secreton_postgres_pool_exhausted").increment(1);
                    }
                }
            }
        });
    }

    /// Get current pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        let status = self.pool.status();

        PoolStats {
            max_size: status.max_size,
            size: status.size,
            available: status.available,
            utilization: if status.max_size > 0 {
                (status.size as f64) / (status.max_size as f64)
            } else {
                0.0
            },
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_size: usize,
    pub size: usize,
    pub available: usize,
    pub utilization: f64,
}

#[async_trait]
impl StorageBackend for PostgresBackend {
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = r#"
            INSERT INTO vault_entries
            (id, path, encrypted_data, encryption_metadata, security_level, metadata, tags, version, owner_id, created_at, updated_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        let encryption_metadata_json =
            serde_json::to_value(&entry.encryption_metadata).map_err(|e| {
                StorageError::SerializationError {
                    source: None,
                    message: format!("Failed to serialize encryption metadata: {}", e),
                }
            })?;

        let metadata_json = serde_json::to_value(&entry.metadata).map_err(|e| {
            StorageError::SerializationError {
                source: None,
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        client
            .execute(
                query,
                &[
                    &entry.id,
                    &entry.path,
                    &entry.encrypted_data,
                    &encryption_metadata_json,
                    &(entry.security_level as i32),
                    &metadata_json,
                    &entry.tags,
                    &(entry.version as i32),
                    &entry.owner_id,
                    &entry.created_at,
                    &entry.updated_at,
                    &entry.expires_at,
                ],
            )
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to store vault entry: {}", e),
            })?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = r#"
            SELECT id, path, encrypted_data, encryption_metadata, security_level, metadata, tags, version, owner_id, created_at, updated_at, expires_at
            FROM vault_entries
            WHERE id = $1
        "#;

        let rows = client
            .query(query, &[&id])
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to query vault entry: {}", e),
            })?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        let entry = self.row_to_vault_entry(row)?;
        Ok(Some(entry))
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = r#"
            SELECT id, path, encrypted_data, encryption_metadata, security_level, metadata, tags, version, owner_id, created_at, updated_at, expires_at
            FROM vault_entries
            WHERE path = $1
        "#;

        let rows = client
            .query(query, &[&path])
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to query vault entry: {}", e),
            })?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        let entry = self.row_to_vault_entry(row)?;
        Ok(Some(entry))
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let mut query = "SELECT id, path, encrypted_data, encryption_metadata, security_level, metadata, tags, version, owner_id, created_at, updated_at, expires_at FROM vault_entries WHERE 1=1".to_string();
        let mut bind_params: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> = Vec::new();
        let mut param_count = 1;

        if let Some(prefix) = &params.path_prefix {
            query.push_str(&format!(" AND path LIKE ${}", param_count));
            let prefix_pattern = format!("{}%", prefix);
            bind_params.push(Box::new(prefix_pattern));
            param_count += 1;
        }

        if let Some(owner) = &params.owner_id {
            query.push_str(&format!(" AND owner_id = ${}", param_count));
            bind_params.push(Box::new(*owner));
            param_count += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", param_count));
        let limit = params.limit.unwrap_or(100);
        bind_params.push(Box::new(limit as i64));

        let bind_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = bind_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();
        let rows =
            client
                .query(&query, &bind_refs)
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to list vault entries: {}", e),
                })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(self.row_to_vault_entry(&row)?);
        }

        Ok(entries)
    }

    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = r#"
            UPDATE vault_entries
            SET path = $2, encrypted_data = $3, encryption_metadata = $4, security_level = $5,
                metadata = $6, tags = $7, version = $8, updated_at = $9, expires_at = $10
            WHERE id = $1
        "#;

        let encryption_metadata_json =
            serde_json::to_value(&entry.encryption_metadata).map_err(|e| {
                StorageError::SerializationError {
                    source: None,
                    message: format!("Failed to serialize encryption metadata: {}", e),
                }
            })?;

        let metadata_json = serde_json::to_value(&entry.metadata).map_err(|e| {
            StorageError::SerializationError {
                source: None,
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        let rows_affected = client
            .execute(
                query,
                &[
                    &entry.id,
                    &entry.path,
                    &entry.encrypted_data,
                    &encryption_metadata_json,
                    &(entry.security_level as i32),
                    &metadata_json,
                    &entry.tags,
                    &(entry.version as i32),
                    &entry.updated_at,
                    &entry.expires_at,
                ],
            )
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to update vault entry: {}", e),
            })?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound {
                resource_type: "VaultEntry".to_string(),
                id: entry.id.to_string(),
            });
        }

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = "DELETE FROM vault_entries WHERE id = $1";

        let rows_affected =
            client
                .execute(query, &[&id])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to delete vault entry: {}", e),
                })?;

        Ok(rows_affected > 0)
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = "DELETE FROM vault_entries WHERE path = $1";

        let rows_affected =
            client
                .execute(query, &[&path])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to delete vault entry: {}", e),
                })?;

        Ok(rows_affected > 0)
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let mut query = "SELECT COUNT(*) FROM vault_entries WHERE 1=1".to_string();
        let mut bind_params: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> = Vec::new();
        let mut param_count = 1;

        if let Some(prefix) = &params.path_prefix {
            query.push_str(&format!(" AND path LIKE ${}", param_count));
            let prefix_pattern = format!("{}%", prefix);
            bind_params.push(Box::new(prefix_pattern));
            param_count += 1;
        }

        if let Some(owner) = &params.owner_id {
            query.push_str(&format!(" AND owner_id = ${}", param_count));
            bind_params.push(Box::new(*owner));
        }

        let bind_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = bind_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();
        let rows =
            client
                .query(&query, &bind_refs)
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to count vault entries: {}", e),
                })?;

        let count: i64 = rows[0].get(0);
        Ok(count as u64)
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = "SELECT EXISTS(SELECT 1 FROM vault_entries WHERE path = $1)";
        let rows = client
            .query(query, &[&path])
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to check existence: {}", e),
            })?;

        let exists: bool = rows[0].get(0);
        Ok(exists)
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        use std::time::Instant;

        let start = Instant::now();

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query_result = client.query("SELECT 1", &[]).await;

        let response_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Get real pool status
        let pool_status = self.pool.status();

        match query_result {
            Ok(_) => Ok(HealthStatus {
                is_healthy: true,
                response_time_ms,
                connections_active: pool_status.size as u32,
                connections_idle: pool_status.available as u32,
                last_error: None,
                uptime_seconds: 0, // Not tracked at backend level
            }),
            Err(e) => Ok(HealthStatus {
                is_healthy: false,
                response_time_ms,
                connections_active: pool_status.size as u32,
                connections_idle: pool_status.available as u32,
                last_error: Some(format!("Health check query failed: {}", e)),
                uptime_seconds: 0,
            }),
        }
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let count_query = "SELECT COUNT(*) FROM vault_entries";
        let size_query = "SELECT pg_total_relation_size('vault_entries')";

        let count_rows =
            client
                .query(count_query, &[])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to get entry count: {}", e),
                })?;

        let size_rows =
            client
                .query(size_query, &[])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to get storage size: {}", e),
                })?;

        let total_entries: i64 = count_rows[0].get(0);
        let storage_size: i64 = size_rows[0].get(0);

        Ok(StorageStats {
            backend_type: "postgres".to_string(),
            total_entries: total_entries as u64,
            total_size_bytes: storage_size as u64,
            average_entry_size: if total_entries > 0 {
                storage_size as f64 / total_entries as f64
            } else {
                0.0
            },
            entries_by_security_level: std::collections::HashMap::new(),
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries: 0,
            last_backup: None,
            metadata: serde_json::json!({}),
        })
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        // TODO: Fix transaction lifetime management
        // Currently disabled due to incompatible types between deadpool and tokio-postgres
        Err(StorageError::TransactionNotSupported {
            backend: "postgres".to_string(),
        })
    }

    async fn migrate(&self) -> StorageResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS vault_entries (
                id UUID PRIMARY KEY,
                path VARCHAR NOT NULL UNIQUE,
                encrypted_data BYTEA NOT NULL,
                encryption_metadata JSONB NOT NULL,
                security_level INTEGER NOT NULL,
                metadata JSONB NOT NULL,
                tags TEXT[] NOT NULL,
                version INTEGER NOT NULL,
                owner_id UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ
            );
            CREATE INDEX IF NOT EXISTS idx_vault_entries_path ON vault_entries(path);
            CREATE INDEX IF NOT EXISTS idx_vault_entries_owner ON vault_entries(owner_id);
            CREATE INDEX IF NOT EXISTS idx_vault_entries_security_level ON vault_entries(security_level);
        "#;

        client
            .batch_execute(create_table_query)
            .await
            .map_err(|e| StorageError::MigrationError {
                message: format!("Failed to run migrations: {}", e),
            })?;

        Ok(())
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::ConnectionFailed {
                source: None,
                message: format!("Failed to get connection: {}", e),
            })?;

        let query = "DELETE FROM vault_entries WHERE expires_at < NOW()";

        let rows_affected =
            client
                .execute(query, &[])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to delete expired entries: {}", e),
                })?;

        Ok(rows_affected)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PostgresBackend {
    fn row_to_vault_entry(&self, row: &Row) -> StorageResult<VaultEntry> {
        let encryption_metadata_value: serde_json::Value = row.get("encryption_metadata");
        let encryption_metadata =
            serde_json::from_value(encryption_metadata_value).map_err(|e| {
                StorageError::SerializationError {
                    source: None,
                    message: format!("Failed to deserialize encryption metadata: {}", e),
                }
            })?;

        let metadata_value: serde_json::Value = row.get("metadata");
        let metadata = serde_json::from_value(metadata_value).map_err(|e| {
            StorageError::SerializationError {
                source: None,
                message: format!("Failed to deserialize metadata: {}", e),
            }
        })?;

        let security_level_int: i32 = row.get("security_level");
        let security_level = match security_level_int {
            0 => SecurityLevel::Public,
            1 => SecurityLevel::Internal,
            2 => SecurityLevel::Confidential,
            3 => SecurityLevel::Secret,
            4 => SecurityLevel::TopSecret,
            _ => SecurityLevel::Internal,
        };

        Ok(VaultEntry {
            id: row.get("id"),
            path: row.get("path"),
            encrypted_data: row.get("encrypted_data"),
            encryption_metadata,
            security_level,
            metadata,
            tags: row.get("tags"),
            version: row.get::<_, i32>("version") as u32,
            owner_id: row.get::<_, Uuid>("owner_id").to_string(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            expires_at: row.get("expires_at"),
        })
    }
}

/// PostgreSQL transaction implementation
pub struct PostgresTransaction {
    transaction: Option<tokio_postgres::Transaction<'static>>,
}

impl PostgresTransaction {
    fn new(transaction: tokio_postgres::Transaction<'static>) -> Self {
        Self {
            transaction: Some(transaction),
        }
    }
}

#[async_trait]
impl StorageTransaction for PostgresTransaction {
    async fn store(&mut self, entry: &VaultEntry) -> StorageResult<()> {
        let transaction =
            self.transaction
                .as_mut()
                .ok_or_else(|| StorageError::TransactionFailed {
                    message: "Transaction already committed or rolled back".to_string(),
                    source: None,
                })?;

        let query = r#"
            INSERT INTO vault_entries
            (id, path, encrypted_data, encryption_metadata, security_level, metadata, tags, version, owner_id, created_at, updated_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        let encryption_metadata_json =
            serde_json::to_value(&entry.encryption_metadata).map_err(|e| {
                StorageError::SerializationError {
                    source: None,
                    message: format!("Failed to serialize encryption metadata: {}", e),
                }
            })?;

        let metadata_json = serde_json::to_value(&entry.metadata).map_err(|e| {
            StorageError::SerializationError {
                source: None,
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        transaction
            .execute(
                query,
                &[
                    &entry.id,
                    &entry.path,
                    &entry.encrypted_data,
                    &encryption_metadata_json,
                    &(entry.security_level as i32),
                    &metadata_json,
                    &entry.tags,
                    &(entry.version as i32),
                    &entry.owner_id,
                    &entry.created_at,
                    &entry.updated_at,
                    &entry.expires_at,
                ],
            )
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to store vault entry in transaction: {}", e),
            })?;

        Ok(())
    }

    async fn update(&mut self, entry: &VaultEntry) -> StorageResult<()> {
        let transaction =
            self.transaction
                .as_mut()
                .ok_or_else(|| StorageError::TransactionFailed {
                    message: "Transaction already committed or rolled back".to_string(),
                    source: None,
                })?;

        let query = r#"
            UPDATE vault_entries
            SET path = $2, encrypted_data = $3, encryption_metadata = $4, security_level = $5,
                metadata = $6, tags = $7, version = $8, updated_at = $9, expires_at = $10
            WHERE id = $1
        "#;

        let encryption_metadata_json =
            serde_json::to_value(&entry.encryption_metadata).map_err(|e| {
                StorageError::SerializationError {
                    source: None,
                    message: format!("Failed to serialize encryption metadata: {}", e),
                }
            })?;

        let metadata_json = serde_json::to_value(&entry.metadata).map_err(|e| {
            StorageError::SerializationError {
                source: None,
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        let rows_affected = transaction
            .execute(
                query,
                &[
                    &entry.id,
                    &entry.path,
                    &entry.encrypted_data,
                    &encryption_metadata_json,
                    &(entry.security_level as i32),
                    &metadata_json,
                    &entry.tags,
                    &(entry.version as i32),
                    &entry.updated_at,
                    &entry.expires_at,
                ],
            )
            .await
            .map_err(|e| StorageError::QueryFailed {
                source: None,
                message: format!("Failed to update vault entry in transaction: {}", e),
            })?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound {
                resource_type: "VaultEntry".to_string(),
                id: entry.id.to_string(),
            });
        }

        Ok(())
    }

    async fn delete(&mut self, id: Uuid) -> StorageResult<bool> {
        let transaction =
            self.transaction
                .as_mut()
                .ok_or_else(|| StorageError::TransactionFailed {
                    message: "Transaction already committed or rolled back".to_string(),
                    source: None,
                })?;

        let query = "DELETE FROM vault_entries WHERE id = $1";

        let rows_affected =
            transaction
                .execute(query, &[&id])
                .await
                .map_err(|e| StorageError::QueryFailed {
                    source: None,
                    message: format!("Failed to delete vault entry in transaction: {}", e),
                })?;

        Ok(rows_affected > 0)
    }

    async fn commit(mut self: Box<Self>) -> StorageResult<()> {
        let transaction =
            self.transaction
                .take()
                .ok_or_else(|| StorageError::TransactionFailed {
                    message: "Transaction already committed or rolled back".to_string(),
                    source: None,
                })?;

        transaction
            .commit()
            .await
            .map_err(|e| StorageError::TransactionFailed {
                message: format!("Failed to commit transaction: {}", e),
                source: None,
            })?;

        Ok(())
    }

    async fn rollback(mut self: Box<Self>) -> StorageResult<()> {
        let transaction =
            self.transaction
                .take()
                .ok_or_else(|| StorageError::TransactionFailed {
                    message: "Transaction already committed or rolled back".to_string(),
                    source: None,
                })?;

        transaction
            .rollback()
            .await
            .map_err(|e| StorageError::TransactionFailed {
                message: format!("Failed to rollback transaction: {}", e),
                source: None,
            })?;

        Ok(())
    }
}
