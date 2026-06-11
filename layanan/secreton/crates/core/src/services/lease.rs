//! Enhanced Lease Management System
//!
//! Comprehensive lease lifecycle management with automatic renewal,
//! revocation cascading, and background cleanup with PostgreSQL persistence.

use crate::models::lease::Lease;
use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::Row;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Lease error types
#[derive(Debug, thiserror::Error)]
pub enum LeaseError {
    #[error("Lease not found: {0}")]
    LeaseNotFound(String),

    #[error("Lease expired")]
    LeaseExpired,

    #[error("Lease revoked")]
    LeaseRevoked,

    #[error("Renewal not allowed")]
    RenewalNotAllowed,

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Enhanced lease with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLease {
    /// Lease ID
    pub id: String,

    /// User/entity ID
    pub user: String,

    /// Resource path
    pub resource: String,

    /// Resource type (secret, database, etc.)
    pub resource_type: String,

    /// Issued timestamp
    pub issued_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expired_at: DateTime<Utc>,

    /// Status (active, revoked, expired)
    pub status: String,

    /// Namespace
    pub namespace: String,

    /// Parent lease ID
    pub parent_id: Option<String>,

    /// Child lease IDs
    pub child_ids: Vec<String>,

    /// Renewable flag
    pub renewable: bool,

    /// Maximum TTL
    pub max_ttl: i64,

    /// Renew count
    pub renew_count: u32,

    /// Maximum renewals allowed
    pub max_renewals: Option<u32>,

    /// Last renewed at
    pub last_renewed_at: Option<DateTime<Utc>>,

    /// Revocation callback
    pub revoke_callback: Option<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl From<Lease> for EnhancedLease {
    fn from(lease: Lease) -> Self {
        Self {
            id: lease.id,
            user: lease.user,
            resource: lease.resource,
            resource_type: lease.resource_type,
            issued_at: lease.issued_at,
            expired_at: lease.expired_at,
            status: lease.status,
            namespace: lease.namespace,
            parent_id: None,
            child_ids: Vec::new(),
            renewable: true,
            max_ttl: 86400,
            renew_count: 0,
            max_renewals: None,
            last_renewed_at: None,
            revoke_callback: None,
            metadata: HashMap::new(),
        }
    }
}

impl From<EnhancedLease> for Lease {
    fn from(val: EnhancedLease) -> Self {
        Lease {
            id: val.id,
            user: val.user,
            resource: val.resource,
            resource_type: val.resource_type,
            issued_at: val.issued_at,
            expired_at: val.expired_at,
            status: val.status,
            namespace: val.namespace,
        }
    }
}

/// Lease scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseSchedulerConfig {
    /// Check interval in seconds (default: 60)
    pub check_interval_secs: u64,

    /// Notification threshold in seconds (default: 300 = 5 minutes)
    pub notification_threshold_secs: i64,

    /// Enable notifications before expiration
    pub enable_notifications: bool,
}

impl Default for LeaseSchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            notification_threshold_secs: 300,
            enable_notifications: false,
        }
    }
}

/// Enhanced lease manager with PostgreSQL persistence
pub struct LeaseManager {
    pool: Pool,
    // In-memory cache for performance
    cache: Arc<RwLock<HashMap<String, EnhancedLease>>>,
    // Scheduler configuration
    scheduler_config: LeaseSchedulerConfig,
    // Metrics registry (optional)
    metrics_registry: Option<Arc<crate::services::metrics::MetricsRegistry>>,
}

/// Parameters for [`LeaseManager::create_lease`].
#[derive(Default)]
pub struct CreateLeaseRequest<'a> {
    pub user: &'a str,
    pub resource: &'a str,
    pub resource_type: &'a str,
    pub namespace: &'a str,
    pub ttl_secs: i64,
    pub max_ttl: i64,
    pub renewable: bool,
    pub parent_id: Option<String>,
    pub max_renewals: Option<u32>,
    pub revoke_callback: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl LeaseManager {
    /// Create new lease manager with database connection pool
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
            scheduler_config: LeaseSchedulerConfig::default(),
            metrics_registry: None,
        }
    }

    /// Create new lease manager with custom scheduler configuration
    pub fn with_config(pool: Pool, scheduler_config: LeaseSchedulerConfig) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
            scheduler_config,
            metrics_registry: None,
        }
    }

    /// Set metrics registry for monitoring
    pub fn with_metrics(
        mut self,
        registry: Arc<crate::services::metrics::MetricsRegistry>,
    ) -> Self {
        self.metrics_registry = Some(registry);
        self
    }

    /// Helper to convert database row to EnhancedLease
    fn row_to_lease(&self, row: &Row) -> Result<EnhancedLease, LeaseError> {
        let metadata_json: serde_json::Value = row
            .try_get("metadata")
            .map_err(|e| LeaseError::StorageError(format!("Failed to get metadata: {}", e)))?;

        let metadata: HashMap<String, String> =
            serde_json::from_value(metadata_json).unwrap_or_default();

        Ok(EnhancedLease {
            id: row
                .try_get("id")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get id: {}", e)))?,
            user: row
                .try_get("user_id")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get user_id: {}", e)))?,
            resource: row
                .try_get("resource")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get resource: {}", e)))?,
            resource_type: row.try_get("resource_type").map_err(|e| {
                LeaseError::StorageError(format!("Failed to get resource_type: {}", e))
            })?,
            issued_at: row
                .try_get("issued_at")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get issued_at: {}", e)))?,
            expired_at: row.try_get("expired_at").map_err(|e| {
                LeaseError::StorageError(format!("Failed to get expired_at: {}", e))
            })?,
            status: row
                .try_get("status")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get status: {}", e)))?,
            namespace: row
                .try_get("namespace")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get namespace: {}", e)))?,
            parent_id: row.try_get("parent_id").ok(),
            child_ids: Vec::new(), // Will be populated separately if needed
            renewable: row
                .try_get("renewable")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get renewable: {}", e)))?,
            max_ttl: row
                .try_get("max_ttl")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get max_ttl: {}", e)))?,
            renew_count: row.try_get::<_, i32>("renew_count").map_err(|e| {
                LeaseError::StorageError(format!("Failed to get renew_count: {}", e))
            })? as u32,
            max_renewals: row
                .try_get::<_, Option<i32>>("max_renewals")
                .ok()
                .flatten()
                .map(|v| v as u32),
            last_renewed_at: row.try_get("last_renewed_at").ok(),
            revoke_callback: row.try_get("revoke_callback").ok(),
            metadata,
        })
    }

    /// Helper to build filter query and parameters
    fn build_filter_query(
        &self,
        user_id: Option<String>,
        namespace: Option<String>,
        resource_type: Option<String>,
        status: Option<String>,
    ) -> (
        String,
        Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
    ) {
        let mut query = String::from(" WHERE 1=1");
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_count = 1;

        if let Some(user) = user_id {
            query.push_str(&format!(" AND user_id = ${}", param_count));
            params.push(Box::new(user));
            param_count += 1;
        }

        if let Some(ns) = namespace {
            query.push_str(&format!(" AND namespace = ${}", param_count));
            params.push(Box::new(ns));
            param_count += 1;
        }

        if let Some(rt) = resource_type {
            query.push_str(&format!(" AND resource_type = ${}", param_count));
            params.push(Box::new(rt));
            param_count += 1;
        }

        if let Some(st) = status {
            query.push_str(&format!(" AND status = ${}", param_count));
            params.push(Box::new(st));
        }

        (query, params)
    }

    /// Create lease with validation and storage persistence
    #[instrument(skip(self, req), fields(
        user = %req.user,
        resource = %req.resource,
        resource_type = %req.resource_type,
        namespace = %req.namespace,
        ttl_secs = %req.ttl_secs,
        operation = "create_lease"
    ))]
    pub async fn create_lease(
        &self,
        req: CreateLeaseRequest<'_>,
    ) -> Result<EnhancedLease, LeaseError> {
        let CreateLeaseRequest {
            user,
            resource,
            resource_type,
            namespace,
            ttl_secs,
            max_ttl,
            renewable,
            parent_id,
            max_renewals,
            revoke_callback,
            metadata,
        } = req;
        // Validation
        if ttl_secs <= 0 || ttl_secs > max_ttl {
            return Err(LeaseError::InvalidTtl(format!(
                "TTL {} must be between 1 and {}",
                ttl_secs, max_ttl
            )));
        }

        if user.is_empty() {
            return Err(LeaseError::InvalidTtl("User cannot be empty".to_string()));
        }

        if resource.is_empty() {
            return Err(LeaseError::InvalidTtl(
                "Resource cannot be empty".to_string(),
            ));
        }

        // Verify parent exists if specified
        if let Some(ref parent_id) = parent_id {
            let parent = self.lookup_lease(parent_id).await?;
            if parent.status != "active" {
                return Err(LeaseError::StorageError(format!(
                    "Parent lease {} is not active",
                    parent_id
                )));
            }
        }

        let now = Utc::now();
        let lease = EnhancedLease {
            id: Uuid::new_v4().to_string(),
            user: user.to_string(),
            resource: resource.to_string(),
            resource_type: resource_type.to_string(),
            issued_at: now,
            expired_at: now + Duration::seconds(ttl_secs),
            status: "active".to_string(),
            namespace: namespace.to_string(),
            parent_id: parent_id.clone(),
            child_ids: Vec::new(),
            renewable,
            max_ttl,
            renew_count: 0,
            max_renewals,
            last_renewed_at: None,
            revoke_callback,
            metadata: metadata.clone(),
        };

        // Store in database
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let metadata_json = serde_json::to_value(&metadata).map_err(|e| {
            LeaseError::StorageError(format!("Failed to serialize metadata: {}", e))
        })?;

        let query = r#"
            INSERT INTO leases
            (id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
             renewable, max_ttl, renew_count, max_renewals, parent_id, revoke_callback, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#;

        client
            .execute(
                query,
                &[
                    &lease.id,
                    &lease.user,
                    &lease.resource,
                    &lease.resource_type,
                    &lease.namespace,
                    &lease.status,
                    &lease.issued_at,
                    &lease.expired_at,
                    &lease.renewable,
                    &lease.max_ttl,
                    &(lease.renew_count as i32),
                    &lease.max_renewals.map(|v| v as i32),
                    &lease.parent_id,
                    &lease.revoke_callback,
                    &metadata_json,
                ],
            )
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to insert lease: {}", e)))?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(lease.id.clone(), lease.clone());

        Ok(lease)
    }

    /// Renew lease with max_ttl check and renew_count increment
    #[instrument(skip(self), fields(
        lease_id = %lease_id,
        increment = %increment,
        operation = "renew_lease"
    ))]
    pub async fn renew_lease(
        &self,
        lease_id: &str,
        increment: i64,
    ) -> Result<EnhancedLease, LeaseError> {
        // Get current lease from database
        let mut lease = self.lookup_lease(lease_id).await?;

        if lease.status != "active" {
            return Err(LeaseError::LeaseRevoked);
        }

        if !lease.renewable {
            return Err(LeaseError::RenewalNotAllowed);
        }

        let now = Utc::now();
        if now > lease.expired_at {
            return Err(LeaseError::LeaseExpired);
        }

        // Check max renewals
        if let Some(max_renewals) = lease.max_renewals
            && lease.renew_count >= max_renewals
        {
            return Err(LeaseError::RenewalNotAllowed);
        }

        // Validate increment
        if increment <= 0 {
            return Err(LeaseError::InvalidTtl(
                "Increment must be positive".to_string(),
            ));
        }

        // Calculate new expiration (capped at max_ttl)
        let new_ttl = increment.min(lease.max_ttl);
        lease.expired_at = now + Duration::seconds(new_ttl);
        lease.renew_count += 1;
        lease.last_renewed_at = Some(now);

        // Update in database
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = r#"
            UPDATE leases
            SET expired_at = $1, renew_count = $2, last_renewed_at = $3, updated_at = NOW()
            WHERE id = $4 AND status = 'active'
        "#;

        let rows_affected = client
            .execute(
                query,
                &[
                    &lease.expired_at,
                    &(lease.renew_count as i32),
                    &lease.last_renewed_at,
                    &lease_id,
                ],
            )
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to update lease: {}", e)))?;

        if rows_affected == 0 {
            return Err(LeaseError::LeaseNotFound(lease_id.to_string()));
        }

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(lease.id.clone(), lease.clone());

        Ok(lease)
    }

    /// Revoke lease with cleanup and cascade to children
    #[instrument(skip(self), fields(
        lease_id = %lease_id,
        operation = "revoke_lease"
    ))]
    pub async fn revoke_lease(&self, lease_id: &str) -> Result<Vec<String>, LeaseError> {
        self.revoke_lease_recursive(lease_id).await
    }

    /// Internal recursive revocation implementation
    fn revoke_lease_recursive<'a>(
        &'a self,
        lease_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<String>, LeaseError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut revoked_ids = Vec::new();

            // Get lease to check if already revoked
            let lease = self.lookup_lease(lease_id).await?;

            if lease.status == "revoked" {
                return Ok(revoked_ids);
            }

            // Get all child leases
            let children = self.get_child_leases(lease_id).await?;

            // Revoke all children first (depth-first)
            for child in children {
                if let Ok(mut child_revoked) = self.revoke_lease_recursive(&child.id).await {
                    revoked_ids.append(&mut child_revoked);
                }
            }

            // Revoke this lease
            let client = self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

            let query = r#"
                UPDATE leases
                SET status = 'revoked', updated_at = NOW()
                WHERE id = $1 AND status = 'active'
            "#;

            let rows_affected = client
                .execute(query, &[&lease_id])
                .await
                .map_err(|e| LeaseError::StorageError(format!("Failed to revoke lease: {}", e)))?;

            if rows_affected > 0 {
                revoked_ids.push(lease_id.to_string());

                // Execute revoke callback if specified
                if let Some(callback_url) = &lease.revoke_callback {
                    tracing::info!("Executing revoke callback: {}", callback_url);
                    self.execute_revoke_callback(callback_url, &lease).await;
                }

                // Remove from cache
                let mut cache = self.cache.write().await;
                cache.remove(lease_id);
            }

            Ok(revoked_ids)
        })
    }

    /// Get child leases for a parent lease
    async fn get_child_leases(&self, parent_id: &str) -> Result<Vec<EnhancedLease>, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = r#"
            SELECT id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
                   last_renewed_at, renewable, max_ttl, renew_count, max_renewals, parent_id,
                   revoke_callback, metadata
            FROM leases
            WHERE parent_id = $1
        "#;

        let rows = client.query(query, &[&parent_id]).await.map_err(|e| {
            LeaseError::StorageError(format!("Failed to query child leases: {}", e))
        })?;

        rows.iter().map(|row| self.row_to_lease(row)).collect()
    }

    /// Lookup lease by ID with database query
    #[instrument(skip(self), fields(
        lease_id = %lease_id,
        operation = "lookup_lease"
    ))]
    pub async fn lookup_lease(&self, lease_id: &str) -> Result<EnhancedLease, LeaseError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(lease) = cache.get(lease_id) {
                return Ok(lease.clone());
            }
        }

        // Query database
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = r#"
            SELECT id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
                   last_renewed_at, renewable, max_ttl, renew_count, max_renewals, parent_id,
                   revoke_callback, metadata
            FROM leases
            WHERE id = $1
        "#;

        let rows = client
            .query(query, &[&lease_id])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to query lease: {}", e)))?;

        let row = rows
            .first()
            .ok_or_else(|| LeaseError::LeaseNotFound(lease_id.to_string()))?;

        let lease = self.row_to_lease(row)?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(lease.id.clone(), lease.clone());

        Ok(lease)
    }

    /// List leases with filtering and pagination
    #[instrument(skip(self), fields(
        user_id = ?user_id,
        namespace = ?namespace,
        resource_type = ?resource_type,
        status = ?status,
        operation = "list_leases"
    ))]
    pub async fn list_leases(
        &self,
        user_id: Option<String>,
        namespace: Option<String>,
        resource_type: Option<String>,
        status: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<EnhancedLease>, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let (filter_clause, mut params) =
            self.build_filter_query(user_id, namespace, resource_type, status);

        let mut query = String::from(
            r#"
            SELECT id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
                   last_renewed_at, renewable, max_ttl, renew_count, max_renewals, parent_id,
                   revoke_callback, metadata
            FROM leases
        "#,
        );
        query.push_str(&filter_clause);

        // param_count needs to continue from where build_filter_query left off
        let mut param_count = params.len() + 1;

        query.push_str(" ORDER BY created_at DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(Box::new(lim));
            param_count += 1;
        }

        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(Box::new(off));
        }

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&query, &param_refs[..])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to list leases: {}", e)))?;

        rows.iter().map(|row| self.row_to_lease(row)).collect()
    }

    /// Count leases with filtering
    #[instrument(skip(self), fields(
        user_id = ?user_id,
        namespace = ?namespace,
        resource_type = ?resource_type,
        status = ?status,
        operation = "count_leases"
    ))]
    pub async fn count_leases(
        &self,
        user_id: Option<String>,
        namespace: Option<String>,
        resource_type: Option<String>,
        status: Option<String>,
    ) -> Result<u64, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let (filter_clause, params) =
            self.build_filter_query(user_id, namespace, resource_type, status);

        let mut query = String::from("SELECT COUNT(*) FROM leases");
        query.push_str(&filter_clause);

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let row = client
            .query_one(&query, &param_refs[..])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to count leases: {}", e)))?;

        let count: i64 = row
            .try_get(0)
            .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;

        Ok(count as u64)
    }

    /// Get expired leases
    pub async fn get_expired_leases(&self) -> Result<Vec<EnhancedLease>, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = r#"
            SELECT id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
                   last_renewed_at, renewable, max_ttl, renew_count, max_renewals, parent_id,
                   revoke_callback, metadata
            FROM leases
            WHERE status = 'active' AND expired_at < NOW()
            ORDER BY expired_at ASC
        "#;

        let rows = client.query(query, &[]).await.map_err(|e| {
            LeaseError::StorageError(format!("Failed to query expired leases: {}", e))
        })?;

        rows.iter().map(|row| self.row_to_lease(row)).collect()
    }

    /// Cleanup expired leases by marking them as expired
    #[instrument(skip(self), fields(operation = "cleanup_expired"))]
    pub async fn cleanup_expired(&self) -> Result<usize, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        // Use the database function to expire leases
        let query = "SELECT expire_old_leases()";

        let row = client
            .query_one(query, &[])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to expire leases: {}", e)))?;

        let count: i32 = row
            .try_get(0)
            .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;

        // Clear cache for expired leases
        let mut cache = self.cache.write().await;
        cache.retain(|_, lease| lease.status == "active" && lease.expired_at > Utc::now());

        Ok(count as usize)
    }

    /// Count active leases
    pub async fn count_active(&self) -> Result<usize, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = "SELECT COUNT(*) FROM leases WHERE status = 'active'";

        let row = client
            .query_one(query, &[])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to count leases: {}", e)))?;

        let count: i64 = row
            .try_get(0)
            .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;

        Ok(count as usize)
    }

    /// Count leases by namespace
    pub async fn count_by_namespace(&self, namespace: &str) -> Result<usize, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = "SELECT COUNT(*) FROM leases WHERE namespace = $1 AND status = 'active'";

        let row = client
            .query_one(query, &[&namespace])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to count leases: {}", e)))?;

        let count: i64 = row
            .try_get(0)
            .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;

        Ok(count as usize)
    }

    /// Start the lease expiration scheduler
    ///
    /// This spawns a background task that periodically checks for expired leases
    /// and automatically revokes them. The task runs until the returned handle is dropped.
    ///
    /// # Returns
    ///
    /// A `tokio::task::JoinHandle` that can be used to wait for or cancel the scheduler.
    pub fn start_expiration_scheduler(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let check_interval =
            std::time::Duration::from_secs(self.scheduler_config.check_interval_secs);

        info!(
            "Starting lease expiration scheduler with check interval: {:?}",
            check_interval
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                debug!("Running lease expiration check");

                match self.run_expiration_check().await {
                    Ok(stats) => {
                        if stats.expired_count > 0 || stats.notified_count > 0 {
                            info!(
                                "Lease expiration check completed: {} expired, {} revoked, {} notified",
                                stats.expired_count, stats.revoked_count, stats.notified_count
                            );
                        } else {
                            debug!("Lease expiration check completed: no leases to process");
                        }

                        // Update metrics if registry is available
                        if let Some(ref registry) = self.metrics_registry {
                            registry
                                .increment_counter(
                                    "secreton_leases_expired_total",
                                    stats.expired_count as u64,
                                )
                                .await;
                            registry
                                .increment_counter(
                                    "secreton_leases_revoked_total",
                                    stats.revoked_count as u64,
                                )
                                .await;

                            if stats.notified_count > 0 {
                                registry
                                    .increment_counter(
                                        "secreton_leases_expiration_notifications_total",
                                        stats.notified_count as u64,
                                    )
                                    .await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Lease expiration check failed: {}", e);

                        // Update error metrics if registry is available
                        if let Some(ref registry) = self.metrics_registry {
                            registry
                                .increment_counter("secreton_leases_expiration_errors_total", 1)
                                .await;
                        }
                    }
                }
            }
        })
    }

    /// Run a single expiration check cycle
    ///
    /// This method:
    /// 1. Finds all expired leases
    /// 2. Revokes them automatically
    /// 3. Optionally sends notifications for leases expiring soon
    /// 4. Returns statistics about the operation
    async fn run_expiration_check(&self) -> Result<ExpirationCheckStats, LeaseError> {
        let mut stats = ExpirationCheckStats::default();

        // Get all expired leases
        let expired_leases = self.get_expired_leases().await?;
        stats.expired_count = expired_leases.len();

        // Revoke each expired lease
        for lease in expired_leases {
            match self.revoke_lease(&lease.id).await {
                Ok(revoked_ids) => {
                    stats.revoked_count += revoked_ids.len();
                    debug!(
                        "Revoked expired lease: {} (and {} children)",
                        lease.id,
                        revoked_ids.len() - 1
                    );
                }
                Err(e) => {
                    warn!("Failed to revoke expired lease {}: {}", lease.id, e);
                    stats.error_count += 1;
                }
            }
        }

        // Send notifications for leases expiring soon (if enabled)
        if self.scheduler_config.enable_notifications {
            match self.notify_expiring_soon().await {
                Ok(count) => {
                    stats.notified_count = count;
                    if count > 0 {
                        debug!("Sent {} expiration notifications", count);
                    }
                }
                Err(e) => {
                    warn!("Failed to send expiration notifications: {}", e);
                }
            }
        }

        Ok(stats)
    }

    /// Get leases that are expiring soon (within notification threshold)
    async fn get_expiring_soon_leases(&self) -> Result<Vec<EnhancedLease>, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let threshold_secs = self.scheduler_config.notification_threshold_secs;

        let query = r#"
            SELECT id, user_id, resource, resource_type, namespace, status, issued_at, expired_at,
                   last_renewed_at, renewable, max_ttl, renew_count, max_renewals, parent_id,
                   revoke_callback, metadata
            FROM leases
            WHERE status = 'active'
              AND expired_at > NOW()
              AND expired_at < NOW() + INTERVAL '1 second' * $1
            ORDER BY expired_at ASC
        "#;

        let rows = client.query(query, &[&threshold_secs]).await.map_err(|e| {
            LeaseError::StorageError(format!("Failed to query expiring leases: {}", e))
        })?;

        rows.iter().map(|row| self.row_to_lease(row)).collect()
    }

    /// Send notifications for leases expiring soon
    ///
    /// Returns the number of notifications sent
    async fn notify_expiring_soon(&self) -> Result<usize, LeaseError> {
        let expiring_leases = self.get_expiring_soon_leases().await?;
        let mut notified_count = 0;

        for lease in expiring_leases {
            let time_until_expiry = lease.expired_at - Utc::now();
            let minutes_remaining = time_until_expiry.num_minutes();

            info!(
                "Lease {} for user {} will expire in {} minutes (resource: {})",
                lease.id, lease.user, minutes_remaining, lease.resource
            );

            // Send notification through configured channels
            self.send_expiry_notification(&lease).await;

            notified_count += 1;
        }

        Ok(notified_count)
    }

    /// Get lease statistics
    pub async fn get_stats(&self) -> Result<LeaseStats, LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        let query = r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COUNT(*) FILTER (WHERE status = 'revoked') as revoked_count,
                COUNT(*) FILTER (WHERE status = 'expired') as expired_count,
                COUNT(*) FILTER (WHERE status = 'active' AND expired_at < NOW() + INTERVAL '5 minutes') as expiring_soon_count,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(DISTINCT namespace) as unique_namespaces
            FROM leases
        "#;

        let row = client
            .query_one(query, &[])
            .await
            .map_err(|e| LeaseError::StorageError(format!("Failed to get stats: {}", e)))?;

        Ok(LeaseStats {
            active_count: row.try_get::<_, i64>(0).unwrap_or(0) as usize,
            revoked_count: row.try_get::<_, i64>(1).unwrap_or(0) as usize,
            expired_count: row.try_get::<_, i64>(2).unwrap_or(0) as usize,
            expiring_soon_count: row.try_get::<_, i64>(3).unwrap_or(0) as usize,
            unique_users: row.try_get::<_, i64>(4).unwrap_or(0) as usize,
            unique_namespaces: row.try_get::<_, i64>(5).unwrap_or(0) as usize,
        })
    }

    /// Get active lease breakdown by resource type and namespace
    pub async fn get_active_lease_breakdown(
        &self,
    ) -> Result<(HashMap<String, usize>, HashMap<String, usize>), LeaseError> {
        let client =
            self.pool.get().await.map_err(|e| {
                LeaseError::StorageError(format!("Failed to get DB connection: {}", e))
            })?;

        // Query by resource type
        let resource_query = r#"
            SELECT resource_type, COUNT(*) as count
            FROM leases
            WHERE status = 'active'
            GROUP BY resource_type
        "#;

        let resource_rows = client.query(resource_query, &[]).await.map_err(|e| {
            LeaseError::StorageError(format!("Failed to query resource stats: {}", e))
        })?;

        let mut by_resource_type = HashMap::new();
        for row in resource_rows {
            let resource_type: String = row.try_get("resource_type").map_err(|e| {
                LeaseError::StorageError(format!("Failed to get resource_type: {}", e))
            })?;
            let count: i64 = row
                .try_get("count")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;
            by_resource_type.insert(resource_type, count as usize);
        }

        // Query by namespace
        let namespace_query = r#"
            SELECT namespace, COUNT(*) as count
            FROM leases
            WHERE status = 'active'
            GROUP BY namespace
        "#;

        let namespace_rows = client.query(namespace_query, &[]).await.map_err(|e| {
            LeaseError::StorageError(format!("Failed to query namespace stats: {}", e))
        })?;

        let mut by_namespace = HashMap::new();
        for row in namespace_rows {
            let namespace: String = row
                .try_get("namespace")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get namespace: {}", e)))?;
            let count: i64 = row
                .try_get("count")
                .map_err(|e| LeaseError::StorageError(format!("Failed to get count: {}", e)))?;
            by_namespace.insert(namespace, count as usize);
        }

        Ok((by_resource_type, by_namespace))
    }

    /// Execute revoke callback via HTTP webhook
    async fn execute_revoke_callback(&self, callback_url: &str, lease: &EnhancedLease) {
        let payload = serde_json::json!({
            "event": "lease_revoked",
            "lease_id": lease.id,
            "user_id": lease.user,
            "resource": lease.resource,
            "resource_type": lease.resource_type,
            "namespace": lease.namespace,
            "revoked_at": Utc::now().to_rfc3339(),
        });

        match reqwest::Client::new()
            .post(callback_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!(
                        "Revoke callback executed successfully for lease {}",
                        lease.id
                    );
                } else {
                    warn!(
                        "Revoke callback returned non-success status {} for lease {}",
                        response.status(),
                        lease.id
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to execute revoke callback for lease {}: {}",
                    lease.id, e
                );
            }
        }
    }

    /// Send expiry notification for a lease
    async fn send_expiry_notification(&self, lease: &EnhancedLease) {
        let time_until_expiry = lease.expired_at - Utc::now();
        let minutes_remaining = time_until_expiry.num_minutes();

        // If there's a callback URL in metadata, use it for notifications
        if let Some(notification_url) = lease.metadata.get("notification_url") {
            let payload = serde_json::json!({
                "event": "lease_expiring_soon",
                "lease_id": lease.id,
                "user_id": lease.user,
                "resource": lease.resource,
                "resource_type": lease.resource_type,
                "namespace": lease.namespace,
                "expires_at": lease.expired_at.to_rfc3339(),
                "minutes_remaining": minutes_remaining,
            });

            match reqwest::Client::new()
                .post(notification_url)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        info!(
                            "Expiry notification sent successfully for lease {}",
                            lease.id
                        );
                    } else {
                        warn!(
                            "Expiry notification returned non-success status {} for lease {}",
                            response.status(),
                            lease.id
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to send expiry notification for lease {}: {}",
                        lease.id, e
                    );
                }
            }
        } else {
            // Log-based notification if no webhook configured
            debug!(
                "Lease {} expiring in {} minutes - no notification webhook configured",
                lease.id, minutes_remaining
            );
        }
    }
}

/// Statistics from an expiration check cycle
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpirationCheckStats {
    /// Number of expired leases found
    pub expired_count: usize,

    /// Number of leases successfully revoked
    pub revoked_count: usize,

    /// Number of expiration notifications sent
    pub notified_count: usize,

    /// Number of errors encountered
    pub error_count: usize,
}

/// Lease statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseStats {
    pub active_count: usize,
    pub revoked_count: usize,
    pub expired_count: usize,
    pub expiring_soon_count: usize,
    pub unique_users: usize,
    pub unique_namespaces: usize,
}

// Legacy functions for compatibility - deprecated, use LeaseManager instead
#[deprecated(note = "Use LeaseManager::create_lease instead")]
pub async fn create_lease(user: &str, resource: &str, resource_type: &str, ttl_secs: i64) -> Lease {
    let now = Utc::now();
    Lease {
        id: Uuid::new_v4().to_string(),
        user: user.to_string(),
        resource: resource.to_string(),
        resource_type: resource_type.to_string(),
        issued_at: now,
        expired_at: now + Duration::seconds(ttl_secs),
        status: "active".to_string(),
        namespace: "default".to_string(),
    }
}

#[deprecated(note = "Use LeaseManager::renew_lease instead")]
pub async fn renew_lease(lease_id: &str, ttl_secs: i64) -> Lease {
    let now = Utc::now();
    Lease {
        id: lease_id.to_string(),
        user: "unknown".to_string(),
        resource: "unknown".to_string(),
        resource_type: "unknown".to_string(),
        issued_at: now,
        expired_at: now + Duration::seconds(ttl_secs),
        status: "active".to_string(),
        namespace: "default".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    async fn setup_test_pool() -> Pool {
        // Use in-memory or test database
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost/secreton_test".to_string()
        });

        let mut cfg = Config::new();
        cfg.url = Some(database_url);
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_create_lease() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(lease.user, "user1");
        assert_eq!(lease.status, "active");
        assert!(lease.renewable);
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_renew_lease() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 1800,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let renewed = manager.renew_lease(&lease.id, 3600).await.unwrap();
        assert_eq!(renewed.renew_count, 1);
        assert!(renewed.last_renewed_at.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_revoke_lease() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let revoked = manager.revoke_lease(&lease.id).await.unwrap();
        assert_eq!(revoked.len(), 1);

        let lease = manager.lookup_lease(&lease.id).await.unwrap();
        assert_eq!(lease.status, "revoked");
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_parent_child_revocation() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        let parent = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/parent",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let _child = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/child",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                parent_id: Some(parent.id.clone()),
                ..Default::default()
            })
            .await
            .unwrap();

        // Revoke parent should revoke child too
        let revoked = manager.revoke_lease(&parent.id).await.unwrap();
        assert_eq!(revoked.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_list_leases_with_filters() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        // Create multiple leases
        manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test1",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        manager
            .create_lease(CreateLeaseRequest {
                user: "user2",
                resource: "/secret/data/test2",
                resource_type: "database",
                namespace: "default",
                ttl_secs: 3600,
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        // List all leases
        let all_leases = manager
            .list_leases(None, None, None, None, None, None)
            .await
            .unwrap();
        assert!(all_leases.len() >= 2);

        // List leases for user1
        let user1_leases = manager
            .list_leases(Some("user1".to_string()), None, None, None, None, None)
            .await
            .unwrap();
        assert!(user1_leases.iter().all(|l| l.user == "user1"));

        // List database leases
        let db_leases = manager
            .list_leases(None, None, Some("database".to_string()), None, None, None)
            .await
            .unwrap();
        assert!(db_leases.iter().all(|l| l.resource_type == "database"));
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_max_renewals() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 1800,
                max_ttl: 86400,
                renewable: true,
                parent_id: None,
                max_renewals: Some(2), // Max 2 renewals
                ..Default::default()
            })
            .await
            .unwrap();

        // First renewal should succeed
        manager.renew_lease(&lease.id, 1800).await.unwrap();

        // Second renewal should succeed
        manager.renew_lease(&lease.id, 1800).await.unwrap();

        // Third renewal should fail
        let result = manager.renew_lease(&lease.id, 1800).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LeaseError::RenewalNotAllowed));
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_cleanup_expired() {
        let pool = setup_test_pool().await;
        let manager = LeaseManager::new(pool);

        // Create a lease that expires immediately
        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 1, // 1 second TTL
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Cleanup expired leases
        let count = manager.cleanup_expired().await.unwrap();
        assert!(count >= 1);

        // Verify lease is expired
        let expired_lease = manager.lookup_lease(&lease.id).await.unwrap();
        assert_eq!(expired_lease.status, "expired");
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_expiration_scheduler() {
        let pool = setup_test_pool().await;

        // Create manager with custom config for faster testing
        let config = LeaseSchedulerConfig {
            check_interval_secs: 2,          // Check every 2 seconds
            notification_threshold_secs: 10, // Notify 10 seconds before expiry
            enable_notifications: true,
        };

        let manager = Arc::new(LeaseManager::with_config(pool, config));

        // Create a lease that expires in 3 seconds
        let lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 3, // 3 second TTL
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        // Start the scheduler
        let scheduler_handle = manager.clone().start_expiration_scheduler();

        // Wait for scheduler to run and expire the lease
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        // Verify lease was expired and revoked
        let expired_lease = manager.lookup_lease(&lease.id).await.unwrap();
        assert_eq!(expired_lease.status, "revoked");

        // Cancel the scheduler
        scheduler_handle.abort();
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_expiring_soon_notification() {
        let pool = setup_test_pool().await;

        let config = LeaseSchedulerConfig {
            check_interval_secs: 60,
            notification_threshold_secs: 300, // 5 minutes
            enable_notifications: true,
        };

        let manager = LeaseManager::with_config(pool, config);

        // Create a lease that expires in 4 minutes (within notification threshold)
        let _lease = manager
            .create_lease(CreateLeaseRequest {
                user: "user1",
                resource: "/secret/data/test",
                resource_type: "kv",
                namespace: "default",
                ttl_secs: 240, // 4 minutes TTL
                max_ttl: 86400,
                renewable: true,
                ..Default::default()
            })
            .await
            .unwrap();

        // Get leases expiring soon
        let expiring = manager.get_expiring_soon_leases().await.unwrap();
        assert_eq!(expiring.len(), 1);

        // Send notifications
        let notified = manager.notify_expiring_soon().await.unwrap();
        assert_eq!(notified, 1);
    }
}
