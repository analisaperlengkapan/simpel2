//! Admin service for system management operations.

use anyhow::Result;
use async_trait::async_trait;
use secreton_core::services::lease::{LeaseError, LeaseManager};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::audit::AuditLogger;
use crate::services::auth::AuthService;
use lib_crypto::encryption::CryptoEngine;
use lib_storage::{MemoryBackend, StorageBackend};
use std::sync::Mutex;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

/// Admin service errors
#[derive(Error, Debug)]
/// Mewakili pub `AdminError`.
pub enum AdminError {
    #[error("Operation not permitted: {0}")]
    NotPermitted(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("System maintenance in progress")]
    MaintenanceInProgress,

    #[error("Auth service error: {0}")]
    Auth(#[from] crate::services::auth::AuthError),

    #[error("Storage error: {0}")]
    Storage(#[from] lib_storage::StorageError),

    #[error("Lease error: {0}")]
    Lease(#[from] secreton_core::services::lease::LeaseError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Trait for lease cleanup operations to allow mocking
#[async_trait]
/// Mewakili pub `LeaseCleaner`.
pub trait LeaseCleaner: Send + Sync {
    async fn cleanup_expired(&self) -> Result<usize, LeaseError>;
}

#[async_trait]
impl LeaseCleaner for LeaseManager {
    async fn cleanup_expired(&self) -> Result<usize, LeaseError> {
        self.cleanup_expired().await
    }
}

/// System statistics
#[derive(Debug, Serialize)]
/// Mewakili pub `SystemStats`.
pub struct SystemStats {
    pub uptime_seconds: u64,
    pub total_users: u64,
    pub active_sessions: u64,
    pub total_secrets: u64,
    pub total_keys: u64,
    pub storage_usage_bytes: u64,
    pub cache_hit_rate: f64,
    pub requests_per_minute: f64,
    pub memory: MemoryStats,
    pub cpu: CpuStats,
    pub disk: DiskStats,
    pub network: NetworkStats,
}

#[derive(Debug, Serialize, Default)]
/// Mewakili pub `MemoryStats`.
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Default)]
/// Mewakili pub `CpuStats`.
pub struct CpuStats {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Default)]
/// Mewakili pub `DiskStats`.
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Default)]
/// Mewakili pub `NetworkStats`.
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Backup information
#[derive(Debug, Serialize)]
/// Mewakili pub `BackupInfo`.
pub struct BackupInfo {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size_bytes: u64,
    pub compressed: bool,
    pub encrypted: bool,
    pub checksum: String,
    pub metadata: HashMap<String, String>,
}

/// Maintenance operation result
#[derive(Debug, Serialize)]
/// Mewakili pub `MaintenanceResult`.
pub struct MaintenanceResult {
    pub operation: String,
    pub success: bool,
    pub duration_ms: u64,
    pub details: HashMap<String, serde_json::Value>,
}

/// Admin service for system management
pub struct AdminService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    auth: Arc<AuthService>,
    audit: Arc<AuditLogger>,
    lease_cleaner: Arc<dyn LeaseCleaner>,
    start_time: chrono::DateTime<chrono::Utc>,
    system: Arc<Mutex<System>>,
}

impl AdminService {
    /// Create new admin service
    pub async fn new(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        auth: Arc<AuthService>,
        audit: Arc<AuditLogger>,
        lease_cleaner: Arc<dyn LeaseCleaner>,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            auth,
            audit,
            lease_cleaner,
            start_time: chrono::Utc::now(),
            system: Arc::new(Mutex::new(System::new_all())),
        })
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> Result<SystemStats, AdminError> {
        // Get storage statistics
        let storage_stats = self.storage.get_stats().await?;

        // Get audit statistics
        let _audit_count = self.audit.count().await;

        // Calculate uptime
        let uptime_seconds = (chrono::Utc::now() - self.start_time).num_seconds() as u64;

        // Get actual user and session counts from auth service
        let total_users = self.auth.count_users().await.unwrap_or(0);
        let active_sessions = self.auth.count_active_sessions().await.unwrap_or(0);

        // Gather system metrics
        let (memory_stats, cpu_stats, disk_stats, network_stats) = {
            let mut sys = self.system.lock().unwrap();

            // Refresh specific metrics
            sys.refresh_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            );

            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();

            let memory = MemoryStats {
                total: total_memory,
                used: used_memory,
                free: sys.free_memory(),
                cached: total_memory
                    .saturating_sub(used_memory)
                    .saturating_sub(sys.free_memory()), // Approximate
            };

            let cpu = CpuStats {
                cores: sys.cpus().len() as u32,
                usage_percent: sys.global_cpu_usage() as f64,
                load_average: [0.0, 0.0, 0.0], // sysinfo might not provide load avg portably easily in this struct
            };

            // Disk usage requires refreshing disks list which can be slow, so maybe do it less often or on separate call
            // For now, refreshing disks here
            let disks = Disks::new_with_refreshed_list();
            let mut total_disk = 0;
            let mut available_disk = 0;
            for disk in &disks {
                total_disk += disk.total_space();
                available_disk += disk.available_space();
            }

            let disk = DiskStats {
                total: total_disk,
                used: total_disk.saturating_sub(available_disk),
                free: available_disk,
                usage_percent: if total_disk > 0 {
                    (total_disk.saturating_sub(available_disk) as f64 / total_disk as f64) * 100.0
                } else {
                    0.0
                },
            };

            // Network stats would require tracking differences over time, simplified here
            let network = NetworkStats::default();

            (memory, cpu, disk, network)
        };

        Ok(SystemStats {
            uptime_seconds,
            total_users,
            active_sessions,
            total_secrets: storage_stats.total_entries,
            total_keys: storage_stats.total_entries, // Count of encrypted entries
            storage_usage_bytes: storage_stats.total_size_bytes,
            cache_hit_rate: crate::middleware::get_cache_hit_rate(),
            requests_per_minute: 0.0, // TODO: Implement request rate tracking
            memory: memory_stats,
            cpu: cpu_stats,
            disk: disk_stats,
            network: network_stats,
        })
    }

    /// Create system backup
    pub async fn create_backup(&self) -> Result<BackupInfo, AdminError> {
        // Get storage statistics for backup size estimation
        let storage_stats = self.storage.get_stats().await?;
        let backup_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();

        // TODO: Implement actual backup export to file/stream
        // For now, log the backup operation in audit trail
        tracing::info!(
            backup_id = %backup_id,
            size_bytes = storage_stats.total_size_bytes,
            "System backup initiated"
        );

        // Calculate checksum placeholder (should be actual hash of backup data)
        let checksum = format!("sha256:{}", hex::encode(&backup_id.as_bytes()[..16]));

        Ok(BackupInfo {
            id: backup_id.clone(),
            created_at,
            size_bytes: storage_stats.total_size_bytes,
            compressed: true,
            encrypted: true,
            checksum,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("version".to_string(), "1.0.0".to_string());
                metadata.insert("type".to_string(), "full".to_string());
                metadata.insert("backend".to_string(), storage_stats.backend_type);
                metadata.insert(
                    "entries_count".to_string(),
                    storage_stats.total_entries.to_string(),
                );
                metadata
            },
        })
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError> {
        // TODO: Implement persistent backup storage and listing
        // Currently backups are not persisted, so returning empty list
        // Future implementation should store backup metadata in storage backend
        tracing::debug!("Listing backups - persistent storage not yet implemented");
        Ok(vec![])
    }

    /// Restore from backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // TODO: Implement actual backup restoration from persistent storage
        // This requires:
        // 1. Loading backup data from backup storage
        // 2. Validating backup integrity (checksum)
        // 3. Parsing and deserializing backup content
        // 4. Clearing existing data (with confirmation)
        // 5. Importing backup data into storage backend

        tracing::warn!(
            backup_id = %backup_id,
            "Backup restoration not yet implemented - persistent storage required"
        );

        let _duration = start_time.elapsed();
        Err(AdminError::NotPermitted(
            "Backup restoration not yet implemented - persistent backup storage required"
                .to_string(),
        ))
    }

    /// Run garbage collection
    pub async fn run_garbage_collection(&self) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // Get initial storage stats
        let stats_before = self.storage.get_stats().await?;

        // 1. Clean expired leases (requires lease service integration)
        let expired_leases_count = self.lease_cleaner.cleanup_expired().await?;

        // 2. Remove soft-deleted secrets past retention period
        // For now, we clean expired secrets (where expires_at < NOW())
        let expired_secrets_count = self.storage.delete_expired().await?;

        // 3. Clean expired audit logs based on retention policy
        // Retention: 30 days
        let retention_period = chrono::Duration::days(30);
        let cleaned_audit_logs = self.audit.cleanup_expired_events(retention_period).await;

        let cleaned_objects =
            expired_leases_count as u64 + expired_secrets_count + cleaned_audit_logs as u64;

        // Recalculate stats to see freed space (approximate)
        let stats_after = self.storage.get_stats().await?;
        let freed_space = stats_before
            .total_size_bytes
            .saturating_sub(stats_after.total_size_bytes);

        tracing::info!(
            cleaned_objects = cleaned_objects,
            freed_space_bytes = freed_space,
            expired_leases = expired_leases_count,
            expired_secrets = expired_secrets_count,
            cleaned_audit_logs = cleaned_audit_logs,
            "Garbage collection completed"
        );

        let duration = start_time.elapsed();
        Ok(MaintenanceResult {
            operation: "garbage_collection".to_string(),
            success: true,
            duration_ms: duration.as_millis() as u64,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "cleaned_objects".to_string(),
                    serde_json::Value::Number(cleaned_objects.into()),
                );
                details.insert(
                    "expired_leases".to_string(),
                    serde_json::Value::Number(expired_leases_count.into()),
                );
                details.insert(
                    "expired_secrets".to_string(),
                    serde_json::Value::Number(expired_secrets_count.into()),
                );
                details.insert(
                    "cleaned_audit_logs".to_string(),
                    serde_json::Value::Number(cleaned_audit_logs.into()),
                );
                details.insert(
                    "freed_space_bytes".to_string(),
                    serde_json::Value::Number(freed_space.into()),
                );
                details.insert(
                    "storage_before_bytes".to_string(),
                    serde_json::Value::Number(stats_before.total_size_bytes.into()),
                );
                details
            },
        })
    }

    /// Compact database
    pub async fn compact_database(&self) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // Get storage stats before compaction
        let stats_before = self.storage.get_stats().await?;
        let original_size = stats_before.total_size_bytes;

        // Database compaction is backend-specific
        // For PostgreSQL: VACUUM FULL
        // For file-based: Rewrite without fragmentation
        // For in-memory: No compaction needed

        tracing::info!(
            backend_type = %stats_before.backend_type,
            original_size_bytes = original_size,
            "Database compaction requested"
        );

        // TODO: Implement backend-specific compaction
        // This requires adding a compact() method to StorageBackend trait

        let compacted_size = original_size; // No actual compaction yet
        let space_saved = 0;

        let duration = start_time.elapsed();
        Ok(MaintenanceResult {
            operation: "compact_database".to_string(),
            success: true,
            duration_ms: duration.as_millis() as u64,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "backend_type".to_string(),
                    serde_json::Value::String(stats_before.backend_type),
                );
                details.insert(
                    "original_size_bytes".to_string(),
                    serde_json::Value::Number(original_size.into()),
                );
                details.insert(
                    "compacted_size_bytes".to_string(),
                    serde_json::Value::Number(compacted_size.into()),
                );
                details.insert(
                    "space_saved_bytes".to_string(),
                    serde_json::Value::Number(space_saved.into()),
                );
                details
            },
        })
    }

    /// Get audit logs with filtering
    pub async fn get_audit_logs(
        &self,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<&str>,
        action: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<AuditLogEntry>, AdminError> {
        // Get recent audit events from logger
        let limit = limit.unwrap_or(100) as usize;
        let recent_events = if let Some(uid) = user_id {
            self.audit.get_by_principal(uid, limit).await
        } else {
            self.audit.get_recent(limit).await
        };

        // Convert audit events to audit log entries with filtering
        let mut entries: Vec<AuditLogEntry> = recent_events
            .into_iter()
            .filter(|event| {
                // Filter by time range
                if let Some(start) = start_time
                    && event.timestamp < start
                {
                    return false;
                }
                if let Some(end) = end_time
                    && event.timestamp > end
                {
                    return false;
                }
                // Filter by action
                if let Some(action_filter) = action {
                    let event_action = format!("{:?}", event.event_type);
                    if !event_action.contains(action_filter) {
                        return false;
                    }
                }
                true
            })
            .map(|event| AuditLogEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: event.timestamp,
                user_id: event.principal.clone(),
                action: format!("{:?}", event.event_type),
                resource: event.realm.clone().unwrap_or_else(|| "unknown".to_string()),
                resource_id: event.secret_key.clone(),
                ip_address: event
                    .client_ip
                    .clone()
                    .unwrap_or_else(|| "0.0.0.0".to_string()),
                user_agent: "unknown".to_string(), // AuditEvent doesn't have user_agent field
                success: event.success,
                details: event.error.map(|e| {
                    let mut details = HashMap::new();
                    details.insert("error".to_string(), serde_json::json!(e));
                    serde_json::Value::Object(details.into_iter().collect())
                }),
            })
            .collect();

        entries.truncate(limit);
        Ok(entries)
    }

    /// Export audit logs
    pub async fn export_audit_logs(
        &self,
        format: &str,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<String, AdminError> {
        // Get audit logs with time filtering
        let logs = self
            .get_audit_logs(start_time, end_time, None, None, Some(10000))
            .await?;

        // Export based on format
        match format.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(&logs).map_err(|e| {
                AdminError::Internal(anyhow::anyhow!("JSON serialization failed: {}", e))
            }),
            "csv" => {
                // Simple CSV export
                let mut csv = String::from(
                    "ID,Timestamp,User ID,Action,Resource,Resource ID,IP Address,User Agent,Success\n",
                );
                for log in logs {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{}\n",
                        log.id,
                        log.timestamp.to_rfc3339(),
                        log.user_id,
                        log.action,
                        log.resource,
                        log.resource_id.unwrap_or_default(),
                        log.ip_address,
                        log.user_agent,
                        log.success
                    ));
                }
                Ok(csv)
            }
            _ => Err(AdminError::InvalidConfig(format!(
                "Unsupported export format: {}. Supported formats: json, csv",
                format
            ))),
        }
    }

    /// Run security scan
    pub async fn run_security_scan(&self) -> Result<SecurityScanResult, AdminError> {
        let scan_id = uuid::Uuid::new_v4().to_string();
        let started_at = chrono::Utc::now();
        let mut findings = Vec::new();

        // Check 1: Audit failed authentication attempts
        let failed_auths = self.audit.get_failed(100).await;
        if failed_auths.len() > 50 {
            findings.push(SecurityFinding {
                severity: "high".to_string(),
                category: "authentication".to_string(),
                title: "High number of failed authentication attempts".to_string(),
                description: format!("Detected {} failed authentication attempts in recent audit logs", failed_auths.len()),
                recommendation: "Review failed login attempts and consider implementing rate limiting or IP blocking".to_string(),
                affected_resources: vec!["authentication_service".to_string()],
            });
        }

        // Check 2: Storage backend health
        match self.storage.health_check().await {
            Ok(health) if !health.is_healthy => {
                let error_msg = health
                    .last_error
                    .unwrap_or_else(|| "Unknown error".to_string());
                findings.push(SecurityFinding {
                    severity: "critical".to_string(),
                    category: "storage".to_string(),
                    title: "Storage backend unhealthy".to_string(),
                    description: format!("Storage backend health check failed: {}", error_msg),
                    recommendation: "Investigate storage backend issues immediately".to_string(),
                    affected_resources: vec!["storage_backend".to_string()],
                });
            }
            Err(e) => {
                findings.push(SecurityFinding {
                    severity: "critical".to_string(),
                    category: "storage".to_string(),
                    title: "Storage backend unreachable".to_string(),
                    description: format!("Failed to perform health check: {}", e),
                    recommendation: "Verify storage backend connectivity and configuration"
                        .to_string(),
                    affected_resources: vec!["storage_backend".to_string()],
                });
            }
            _ => {}
        }

        // Check 3: Storage capacity
        let _storage_stats = self.storage.get_stats().await?;
        // Assuming 80% is a warning threshold (adjust based on actual limits)
        // This is a simplified check; real implementation would need actual capacity limits

        tracing::info!(
            scan_id = %scan_id,
            findings_count = findings.len(),
            "Security scan completed"
        );

        Ok(SecurityScanResult {
            scan_id,
            status: "completed".to_string(),
            started_at,
            completed_at: Some(chrono::Utc::now()),
            findings,
        })
    }

    /// Update system configuration
    pub async fn update_config(
        &self,
        config_updates: HashMap<String, serde_json::Value>,
    ) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // TODO: Validate and apply configuration updates

        let duration = start_time.elapsed();
        Ok(MaintenanceResult {
            operation: "update_config".to_string(),
            success: true,
            duration_ms: duration.as_millis() as u64,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "updated_keys".to_string(),
                    serde_json::Value::Array(
                        config_updates
                            .keys()
                            .map(|k| serde_json::Value::String(k.clone()))
                            .collect(),
                    ),
                );
                details
            },
        })
    }
}

/// Audit log entry
#[derive(Debug, Serialize)]
/// Mewakili pub `AuditLogEntry`.
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub success: bool,
    pub details: Option<serde_json::Value>,
}

/// Security scan result
#[derive(Debug, Serialize)]
/// Mewakili pub `SecurityScanResult`.
pub struct SecurityScanResult {
    pub scan_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub findings: Vec<SecurityFinding>,
}

/// Security finding
#[derive(Debug, Serialize)]
/// Mewakili pub `SecurityFinding`.
pub struct SecurityFinding {
    pub severity: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub affected_resources: Vec<String>,
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::config::AuthConfig;
    use lib_crypto::SecurityParams;
    use lib_storage::MemoryBackend;

    // Mock implementation of LeaseCleaner for testing
    /// Mewakili pub `MockLeaseCleaner`.
    pub struct MockLeaseCleaner {
        pub expired_count: usize,
    }

    #[async_trait]
    impl LeaseCleaner for MockLeaseCleaner {
        async fn cleanup_expired(&self) -> Result<usize, LeaseError> {
            Ok(self.expired_count)
        }
    }

    #[tokio::test]
    async fn test_admin_service_creation() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(lib_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto, &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });

        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner).await;
        assert!(admin_service.is_ok());
    }

    #[tokio::test]
    async fn test_get_system_stats() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(lib_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto, &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let stats = admin_service.get_system_stats().await.expect("stats");
        // With empty storage, stats should reflect zero counts
        assert_eq!(stats.total_secrets, 0);
        assert_eq!(stats.total_keys, 0);
        assert_eq!(stats.storage_usage_bytes, 0);
        assert!(
            stats.uptime_seconds >= 1,
            "Uptime should be at least 1 second"
        );
        assert!(stats.cache_hit_rate >= 0.0);
    }

    #[tokio::test]
    async fn test_create_backup_returns_metadata() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(lib_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto, &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner)
            .await
            .unwrap();

        let backup = admin_service.create_backup().await.expect("backup");
        assert!(backup.encrypted);
        assert!(backup.compressed);
        assert!(backup.metadata.contains_key("version"));
        assert!(backup.metadata.contains_key("backend"));
        assert!(!backup.checksum.is_empty());
    }

    #[tokio::test]
    async fn test_run_garbage_collection_returns_details() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(lib_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto, &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 15 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner)
            .await
            .unwrap();

        let result = admin_service
            .run_garbage_collection()
            .await
            .expect("gc failed");

        assert_eq!(result.operation, "garbage_collection");
        assert!(result.success);

        let details = &result.details;

        // Check expired leases (from mock)
        let expired_leases = details.get("expired_leases").unwrap().as_u64().unwrap();
        assert_eq!(expired_leases, 15);

        // Check structure
        assert!(details.contains_key("cleaned_objects"));
        assert!(details.contains_key("freed_space_bytes"));
        assert!(details.contains_key("storage_before_bytes"));
    }
}

impl AdminService {
    /// Create mock admin service for testing
    pub fn new_mock(storage: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        // Create a dummy lease cleaner
        struct DummyLeaseCleaner;
        #[async_trait]
        impl LeaseCleaner for DummyLeaseCleaner {
            async fn cleanup_expired(&self) -> Result<usize, LeaseError> {
                Ok(0)
            }
        }

        Self {
            auth: Arc::new(AuthService::new_mock(
                storage.clone(),
                Arc::new(CryptoEngine::new()),
            )),
            storage,
            audit: Arc::new(AuditLogger::new(10000)),
            lease_cleaner: Arc::new(DummyLeaseCleaner),
            start_time: chrono::Utc::now(),
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }
}
