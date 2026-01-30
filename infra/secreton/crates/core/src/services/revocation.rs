//! Secret Revocation Service
//!
//! Comprehensive secret revocation with cascade support, emergency revocation,
//! audit history, and orphaned secret detection.

use crate::error::CoreError;
use crate::services::lease::{LeaseError, LeaseManager};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Revocation error types
#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Revocation failed: {0}")]
    RevocationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Lease error: {0}")]
    LeaseError(#[from] LeaseError),

    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

impl From<RevocationError> for CoreError {
    fn from(err: RevocationError) -> Self {
        match err {
            RevocationError::SecretNotFound(msg) => CoreError::not_found(msg),
            RevocationError::RevocationFailed(msg) => CoreError::invalid_operation(msg),
            RevocationError::StorageError(msg) => CoreError::database(msg),
            RevocationError::LeaseError(e) => CoreError::internal(e.to_string()),
            RevocationError::Core(e) => e,
        }
    }
}

/// Revocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRequest {
    /// Path to secret to revoke
    pub path: String,

    /// Reason for revocation
    pub reason: String,

    /// Whether to cascade to dependent secrets
    pub cascade: bool,

    /// Emergency revocation flag (bypass normal checks)
    pub emergency: bool,

    /// Actor performing the revocation
    pub actor: String,

    /// Namespace
    pub namespace: String,
}

/// Revocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    /// Unique revocation ID
    pub id: Uuid,

    /// Path of revoked secret
    pub path: String,

    /// Timestamp of revocation
    pub revoked_at: DateTime<Utc>,

    /// Actor who performed revocation
    pub revoked_by: String,

    /// Reason for revocation
    pub reason: String,

    /// Number of cascaded revocations
    pub cascade_count: usize,

    /// Namespace
    pub namespace: String,

    /// Emergency revocation flag
    pub emergency: bool,

    /// List of cascaded paths
    pub cascaded_paths: Vec<String>,
}

/// Orphaned secret information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedSecret {
    /// Secret path
    pub path: String,

    /// Namespace
    pub namespace: String,

    /// Last accessed timestamp
    pub last_accessed: Option<DateTime<Utc>>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Days since last access
    pub days_since_access: i64,

    /// Whether secret has any active leases
    pub has_active_leases: bool,
}

/// Revocation service trait
#[async_trait::async_trait]
pub trait RevocationService: Send + Sync {
    /// Revoke secret and associated leases
    async fn revoke(
        &self,
        request: RevocationRequest,
    ) -> std::result::Result<RevocationRecord, RevocationError>;

    /// Emergency revocation by pattern
    async fn emergency_revoke(
        &self,
        pattern: &str,
        actor: &str,
        namespace: &str,
    ) -> std::result::Result<Vec<RevocationRecord>, RevocationError>;

    /// Get revocation history for a path
    async fn get_history(
        &self,
        path: &str,
    ) -> std::result::Result<Vec<RevocationRecord>, RevocationError>;

    /// Detect orphaned secrets
    async fn detect_orphans(
        &self,
        threshold_days: u32,
    ) -> std::result::Result<Vec<OrphanedSecret>, RevocationError>;

    /// Get revocation statistics
    async fn get_stats(&self) -> std::result::Result<RevocationStats, RevocationError>;
}

/// Revocation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationStats {
    /// Total revocations
    pub total_revocations: usize,

    /// Emergency revocations
    pub emergency_revocations: usize,

    /// Cascade revocations
    pub cascade_revocations: usize,

    /// Orphaned secrets detected
    pub orphaned_secrets: usize,
}

/// Revocation manager implementation
pub struct RevocationManager {
    pool: Pool,
    pub lease_manager: Arc<LeaseManager>,
    // Cache for dependency graph
    dependency_cache: Arc<RwLock<DependencyGraph>>,
}

/// Dependency graph for tracking secret relationships
#[derive(Debug, Clone, Default)]
struct DependencyGraph {
    /// Map of secret path to its dependencies
    dependencies: std::collections::HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Add a dependency relationship
    fn add_dependency(&mut self, parent: String, child: String) {
        self.dependencies.entry(parent).or_default().insert(child);
    }

    /// Get all dependencies for a path (recursive)
    fn get_all_dependencies(&self, path: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut to_process = vec![path.to_string()];

        while let Some(current) = to_process.pop() {
            if let Some(deps) = self.dependencies.get(&current) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        to_process.push(dep.clone());
                    }
                }
            }
        }

        result
    }
}

impl RevocationManager {
    /// Create new revocation manager
    pub fn new(pool: Pool, lease_manager: Arc<LeaseManager>) -> Self {
        Self {
            pool,
            lease_manager,
            dependency_cache: Arc::new(RwLock::new(DependencyGraph::default())),
        }
    }

    /// Initialize revocation tables if they don't exist
    pub async fn initialize(&self) -> std::result::Result<(), RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        // Create revocations table
        let create_table = r#"
            CREATE TABLE IF NOT EXISTS revocations (
                id UUID PRIMARY KEY,
                path TEXT NOT NULL,
                namespace TEXT NOT NULL,
                revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                revoked_by TEXT NOT NULL,
                reason TEXT NOT NULL,
                cascade_count INTEGER NOT NULL DEFAULT 0,
                emergency BOOLEAN NOT NULL DEFAULT FALSE,
                cascaded_paths JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;

        client.execute(create_table, &[]).await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to create revocations table: {}", e))
        })?;

        // Create index on path for faster lookups
        let create_index = r#"
            CREATE INDEX IF NOT EXISTS idx_revocations_path
            ON revocations(path, namespace)
        "#;

        client
            .execute(create_index, &[])
            .await
            .map_err(|e| RevocationError::StorageError(format!("Failed to create index: {}", e)))?;

        // Create index on revoked_at for history queries
        let create_time_index = r#"
            CREATE INDEX IF NOT EXISTS idx_revocations_time
            ON revocations(revoked_at DESC)
        "#;

        client.execute(create_time_index, &[]).await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to create time index: {}", e))
        })?;

        // Create secret_dependencies table for tracking relationships
        let create_deps_table = r#"
            CREATE TABLE IF NOT EXISTS secret_dependencies (
                parent_path TEXT NOT NULL,
                child_path TEXT NOT NULL,
                namespace TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (parent_path, child_path, namespace)
            )
        "#;

        client.execute(create_deps_table, &[]).await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to create dependencies table: {}", e))
        })?;

        info!("Revocation service initialized successfully");
        Ok(())
    }

    /// Load dependency graph from database
    async fn load_dependencies(&self) -> std::result::Result<(), RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        let query = "SELECT parent_path, child_path FROM secret_dependencies";

        let rows = client.query(query, &[]).await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to load dependencies: {}", e))
        })?;

        let mut graph = self.dependency_cache.write().await;
        for row in rows {
            let parent: String = row.get(0);
            let child: String = row.get(1);
            graph.add_dependency(parent, child);
        }

        debug!(
            "Loaded {} dependency relationships",
            graph.dependencies.len()
        );
        Ok(())
    }

    /// Add a dependency relationship
    pub async fn add_dependency(
        &self,
        parent_path: &str,
        child_path: &str,
        namespace: &str,
    ) -> std::result::Result<(), RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        let query = r#"
            INSERT INTO secret_dependencies (parent_path, child_path, namespace)
            VALUES ($1, $2, $3)
            ON CONFLICT (parent_path, child_path, namespace) DO NOTHING
        "#;

        client
            .execute(query, &[&parent_path, &child_path, &namespace])
            .await
            .map_err(|e| {
                RevocationError::StorageError(format!("Failed to add dependency: {}", e))
            })?;

        // Update cache
        let mut graph = self.dependency_cache.write().await;
        graph.add_dependency(parent_path.to_string(), child_path.to_string());

        Ok(())
    }

    /// Revoke a secret and optionally cascade to dependencies
    #[instrument(skip(self), fields(
        path = %request.path,
        cascade = %request.cascade,
        emergency = %request.emergency,
        operation = "revoke_secret"
    ))]
    async fn revoke_secret_internal(
        &self,
        request: &RevocationRequest,
    ) -> std::result::Result<RevocationRecord, RevocationError> {
        let start_time = std::time::Instant::now();
        let mut cascaded_paths = Vec::new();

        // Get all paths to revoke (including dependencies if cascade is enabled)
        let paths_to_revoke = if request.cascade {
            let graph = self.dependency_cache.read().await;
            let mut paths = graph.get_all_dependencies(&request.path);
            paths.insert(request.path.clone());
            paths.into_iter().collect::<Vec<_>>()
        } else {
            vec![request.path.clone()]
        };

        // Revoke all associated leases
        for path in &paths_to_revoke {
            // Find all leases for this path
            match self
                .lease_manager
                .list_leases(
                    None,
                    Some(request.namespace.clone()),
                    None,
                    Some("active".to_string()),
                    None,
                    None,
                )
                .await
            {
                Ok(leases) => {
                    for lease in leases.iter().filter(|l| l.resource == *path) {
                        match self.lease_manager.revoke_lease(&lease.id).await {
                            Ok(revoked_ids) => {
                                debug!("Revoked {} leases for secret {}", revoked_ids.len(), path);
                            }
                            Err(e) => {
                                warn!("Failed to revoke lease {}: {}", lease.id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to list leases for {}: {}", path, e);
                }
            }

            if path != &request.path {
                cascaded_paths.push(path.clone());
            }
        }

        // Create revocation record
        let record = RevocationRecord {
            id: Uuid::new_v4(),
            path: request.path.clone(),
            revoked_at: Utc::now(),
            revoked_by: request.actor.clone(),
            reason: request.reason.clone(),
            cascade_count: cascaded_paths.len(),
            namespace: request.namespace.clone(),
            emergency: request.emergency,
            cascaded_paths: cascaded_paths.clone(),
        };

        // Store revocation record in database
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        let cascaded_json = serde_json::to_value(&cascaded_paths).map_err(|e| {
            RevocationError::StorageError(format!("Failed to serialize cascaded paths: {}", e))
        })?;

        let query = r#"
            INSERT INTO revocations
            (id, path, namespace, revoked_at, revoked_by, reason, cascade_count, emergency, cascaded_paths)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        client
            .execute(
                query,
                &[
                    &record.id,
                    &record.path,
                    &record.namespace,
                    &record.revoked_at,
                    &record.revoked_by,
                    &record.reason,
                    &(record.cascade_count as i32),
                    &record.emergency,
                    &cascaded_json,
                ],
            )
            .await
            .map_err(|e| {
                RevocationError::StorageError(format!("Failed to store revocation record: {}", e))
            })?;

        let duration = start_time.elapsed();
        info!(
            "Revoked secret {} with {} cascaded secrets in {:?}",
            request.path,
            cascaded_paths.len(),
            duration
        );

        // For emergency revocations, ensure we complete within 1 second
        if request.emergency && duration.as_secs() >= 1 {
            warn!(
                "Emergency revocation took {:?}, exceeding 1 second target",
                duration
            );
        }

        Ok(record)
    }
}

#[async_trait::async_trait]
impl RevocationService for RevocationManager {
    #[instrument(skip(self), fields(
        path = %request.path,
        actor = %request.actor,
        operation = "revoke"
    ))]
    async fn revoke(
        &self,
        request: RevocationRequest,
    ) -> std::result::Result<RevocationRecord, RevocationError> {
        // Validate request
        if request.path.is_empty() {
            return Err(RevocationError::RevocationFailed(
                "Path cannot be empty".to_string(),
            ));
        }

        if request.actor.is_empty() {
            return Err(RevocationError::RevocationFailed(
                "Actor cannot be empty".to_string(),
            ));
        }

        self.revoke_secret_internal(&request).await
    }

    #[instrument(skip(self), fields(
        pattern = %pattern,
        actor = %actor,
        operation = "emergency_revoke"
    ))]
    async fn emergency_revoke(
        &self,
        pattern: &str,
        actor: &str,
        namespace: &str,
    ) -> std::result::Result<Vec<RevocationRecord>, RevocationError> {
        let start_time = std::time::Instant::now();

        // Find all secrets matching the pattern
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        // Query secrets table (assuming it exists)
        // This is a simplified version - actual implementation would query the secrets storage
        let query = r#"
            SELECT DISTINCT resource
            FROM leases
            WHERE resource LIKE $1 AND namespace = $2 AND status = 'active'
        "#;

        let pattern_sql = format!("{}%", pattern);
        let rows = client
            .query(query, &[&pattern_sql, &namespace])
            .await
            .map_err(|e| {
                RevocationError::StorageError(format!("Failed to query secrets: {}", e))
            })?;

        let mut records = Vec::new();

        for row in rows {
            let path: String = row.get(0);

            let request = RevocationRequest {
                path,
                reason: format!("Emergency revocation by pattern: {}", pattern),
                cascade: true,
                emergency: true,
                actor: actor.to_string(),
                namespace: namespace.to_string(),
            };

            match self.revoke_secret_internal(&request).await {
                Ok(record) => {
                    records.push(record);
                }
                Err(e) => {
                    error!("Failed to revoke secret {}: {}", request.path, e);
                }
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Emergency revocation completed: {} secrets revoked in {:?}",
            records.len(),
            duration
        );

        if duration.as_secs() >= 1 {
            warn!(
                "Emergency revocation took {:?}, exceeding 1 second target",
                duration
            );
        }

        Ok(records)
    }

    #[instrument(skip(self), fields(
        path = %path,
        operation = "get_history"
    ))]
    async fn get_history(
        &self,
        path: &str,
    ) -> std::result::Result<Vec<RevocationRecord>, RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        let query = r#"
            SELECT id, path, namespace, revoked_at, revoked_by, reason, cascade_count, emergency, cascaded_paths
            FROM revocations
            WHERE path = $1
            ORDER BY revoked_at DESC
        "#;

        let rows = client.query(query, &[&path]).await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to query revocation history: {}", e))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let cascaded_json: serde_json::Value = row.get(8);
            let cascaded_paths: Vec<String> =
                serde_json::from_value(cascaded_json).unwrap_or_default();

            records.push(RevocationRecord {
                id: row.get(0),
                path: row.get(1),
                namespace: row.get(2),
                revoked_at: row.get(3),
                revoked_by: row.get(4),
                reason: row.get(5),
                cascade_count: row.get::<_, i32>(6) as usize,
                emergency: row.get(7),
                cascaded_paths,
            });
        }

        Ok(records)
    }

    #[instrument(skip(self), fields(
        threshold_days = %threshold_days,
        operation = "detect_orphans"
    ))]
    async fn detect_orphans(
        &self,
        threshold_days: u32,
    ) -> std::result::Result<Vec<OrphanedSecret>, RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        // Find secrets with no active leases and no recent access
        let query = r#"
            SELECT DISTINCT l.resource, l.namespace, MAX(l.issued_at) as last_accessed,
                   MIN(l.issued_at) as created_at,
                   COUNT(CASE WHEN l.status = 'active' THEN 1 END) as active_count
            FROM leases l
            WHERE l.status != 'active'
            GROUP BY l.resource, l.namespace
            HAVING MAX(l.issued_at) < NOW() - INTERVAL '1 day' * $1
            ORDER BY MAX(l.issued_at) ASC
        "#;

        let rows = client
            .query(query, &[&(threshold_days as i32)])
            .await
            .map_err(|e| {
                RevocationError::StorageError(format!("Failed to query orphaned secrets: {}", e))
            })?;

        let mut orphans = Vec::new();
        for row in rows {
            let last_accessed: Option<DateTime<Utc>> = row.get(2);
            let created_at: DateTime<Utc> = row.get(3);
            let active_count: i64 = row.get(4);

            let days_since_access = if let Some(last) = last_accessed {
                (Utc::now() - last).num_days()
            } else {
                (Utc::now() - created_at).num_days()
            };

            orphans.push(OrphanedSecret {
                path: row.get(0),
                namespace: row.get(1),
                last_accessed,
                created_at,
                days_since_access,
                has_active_leases: active_count > 0,
            });
        }

        info!("Detected {} orphaned secrets", orphans.len());
        Ok(orphans)
    }

    async fn get_stats(&self) -> std::result::Result<RevocationStats, RevocationError> {
        let client = self.pool.get().await.map_err(|e| {
            RevocationError::StorageError(format!("Failed to get DB connection: {}", e))
        })?;

        let query = r#"
            SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN emergency = true THEN 1 END) as emergency,
                SUM(cascade_count) as cascaded
            FROM revocations
        "#;

        let row = client
            .query_one(query, &[])
            .await
            .map_err(|e| RevocationError::StorageError(format!("Failed to get stats: {}", e)))?;

        // Get orphaned secrets count
        let orphans = self.detect_orphans(30).await?;

        let cascade_sum: Option<i64> = row.get(2);

        Ok(RevocationStats {
            total_revocations: row.get::<_, i64>(0) as usize,
            emergency_revocations: row.get::<_, i64>(1) as usize,
            cascade_revocations: cascade_sum.unwrap_or(0) as usize,
            orphaned_secrets: orphans.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    async fn setup_test_pool() -> Pool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost/secreton_test".to_string()
        });

        let mut cfg = Config::new();
        cfg.url = Some(database_url);
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_revocation_manager_initialization() {
        let pool = setup_test_pool().await;
        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
        let revocation_manager = RevocationManager::new(pool, lease_manager);

        let result = revocation_manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_add_dependency() {
        let pool = setup_test_pool().await;
        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
        let revocation_manager = RevocationManager::new(pool, lease_manager);

        revocation_manager.initialize().await.unwrap();

        let result = revocation_manager
            .add_dependency("/parent/secret", "/child/secret", "default")
            .await;

        assert!(result.is_ok());
    }
}
