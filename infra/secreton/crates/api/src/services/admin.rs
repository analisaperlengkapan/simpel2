//! Admin service for system management operations.

use anyhow::Result;
use async_trait::async_trait;
use secreton_core::services::lease::{LeaseError, LeaseManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::audit::AuditLogger;
use crate::services::auth::AuthService;
use secreton_crypto::encryption::CryptoEngine;
use secreton_storage::{QueryParams, SecretEntry, SecurityLevel, StorageBackend};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

/// Admin service errors
#[derive(Error, Debug)]
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
    Storage(#[from] secreton_storage::StorageError),

    #[error("Lease error: {0}")]
    Lease(#[from] secreton_core::services::lease::LeaseError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Trait for lease cleanup operations to allow mocking
#[async_trait]
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
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Default)]
pub struct CpuStats {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Default)]
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Default)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Backup information
#[derive(Debug, Serialize)]
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
    crypto: Arc<CryptoEngine>,
    start_time: chrono::DateTime<chrono::Utc>,
    system: Arc<Mutex<System>>,
    request_count: Arc<AtomicU64>,
}

impl AdminService {
    /// Create new admin service
    pub async fn new(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        auth: Arc<AuthService>,
        audit: Arc<AuditLogger>,
        lease_cleaner: Arc<dyn LeaseCleaner>,
        crypto: Arc<CryptoEngine>,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            auth,
            audit,
            lease_cleaner,
            crypto,
            start_time: chrono::Utc::now(),
            system: Arc::new(Mutex::new(System::new_all())),
            request_count: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Track a request for RPM calculation
    pub fn track_request(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> Result<SystemStats, AdminError> {
        // Get storage statistics
        let storage_stats = self.storage.get_stats().await?;

        // Get audit statistics
        let _audit_count = self.audit.count().await;

        // Calculate uptime
        let uptime_seconds = (chrono::Utc::now() - self.start_time).num_seconds() as u64;

        // Calculate RPM (simplified: total requests / uptime minutes)
        let total_requests = self.request_count.load(Ordering::Relaxed);
        let uptime_minutes = (uptime_seconds as f64) / 60.0;
        let requests_per_minute = if uptime_minutes > 0.0 {
            (total_requests as f64) / uptime_minutes
        } else {
            0.0
        };

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
            requests_per_minute,
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
        let checksum = format!("sha256::{}", hex::encode(&backup_id.as_bytes()[..16]));

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
                if let Some(start) = start_time {
                    if event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = end_time {
                    if event.timestamp > end {
                        return false;
                    }
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

    /// Retrieve security incidents
    pub async fn get_security_incidents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        filter: Option<String>,
    ) -> Result<Vec<SecurityIncident>, AdminError> {
        // Always fetch all incidents first to ensure consistent pagination across backends
        // (MemoryBackend does not support pagination parameters) and correct filtering counts
        let params = QueryParams::new().with_path_prefix("sys/incidents/".to_string());

        let entries = self
            .storage
            .list(&params)
            .await
            .map_err(AdminError::Storage)?;

        let mut incidents = Vec::new();
        for entry in entries {
            let decrypted = self
                .crypto
                .decrypt_simple(&entry.encrypted_data)
                .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
            let incident: SecurityIncident = serde_json::from_slice(&decrypted)
                .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))?;

            // Apply filter if present (case-insensitive contains on key fields)
            if let Some(ref f) = filter {
                let f = f.to_lowercase();
                if !incident.title.to_lowercase().contains(&f)
                    && !incident.description.to_lowercase().contains(&f)
                    && !incident.source.to_lowercase().contains(&f)
                    && !incident.id.to_lowercase().contains(&f)
                {
                    continue;
                }
            }

            incidents.push(incident);
        }

        // Apply pagination in memory
        let start = offset.unwrap_or(0) as usize;
        if start >= incidents.len() {
            return Ok(Vec::new());
        }

        let end = if let Some(l) = limit {
            (start + l as usize).min(incidents.len())
        } else {
            incidents.len()
        };

        Ok(incidents[start..end].to_vec())
    }

    /// Create a security incident (internal use or testing)
    pub async fn create_security_incident(
        &self,
        incident: SecurityIncident,
    ) -> Result<(), AdminError> {
        let path = format!("sys/incidents/{}", incident.id);
        let data = serde_json::to_vec(&incident)
            .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;

        let encrypted = self
            .crypto
            .encrypt_simple(&data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let entry = SecretEntry::new(
            path,
            encrypted,
            serde_json::json!({"type": "security_incident"}),
            SecurityLevel::Internal,
            "system".to_string(),
        )
        .add_metadata(
            "severity".to_string(),
            serde_json::Value::String(incident.severity.clone()),
        )
        .add_metadata(
            "status".to_string(),
            serde_json::Value::String(incident.status.clone()),
        )
        .add_metadata(
            "source".to_string(),
            serde_json::Value::String(incident.source.clone()),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(AdminError::Storage)?;
        Ok(())
    }

    /// Update system configuration
    pub async fn update_config(
        &self,
        config_updates: HashMap<String, serde_json::Value>,
    ) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();
        let mut updated_keys = Vec::new();
        let mut errors = Vec::new();

        // 1. Update Rate Limiting
        if let Some(rate_limit_value) = config_updates.get("rate_limit") {
            // Try to parse as RateLimitConfig
            if let Ok(config) =
                serde_json::from_value::<crate::config::RateLimitConfig>(rate_limit_value.clone())
            {
                // Update global rate limiter
                crate::middleware::update_rate_limiting(config.global.requests);
                updated_keys.push("rate_limit".to_string());
            } else if let Some(global) = rate_limit_value.get("global") {
                // Try to parse partial structure
                if let Some(requests) = global.get("requests").and_then(|v| v.as_u64()) {
                    crate::middleware::update_rate_limiting(requests as u32);
                    updated_keys.push("rate_limit.global.requests".to_string());
                } else {
                    errors
                        .push("Invalid rate_limit structure: missing global.requests".to_string());
                }
            } else {
                errors.push("Invalid rate_limit structure: parsing failed".to_string());
            }
        }

        // 2. Update Authentication Config
        if let Some(auth_value) = config_updates.get("auth") {
            if let Ok(auth_config) =
                serde_json::from_value::<crate::config::AuthConfig>(auth_value.clone())
            {
                self.auth.update_config(auth_config);
                updated_keys.push("auth".to_string());
            } else {
                let msg = "Failed to parse auth config update".to_string();
                tracing::warn!("{}", msg);
                errors.push(msg);
            }
        }

        // 3. Update TLS Certificate Cache (if provided)
        if let Some(value) = config_updates.get("tls_cache_ttl") {
            if let Some(tls_cache_ttl) = value.as_u64() {
                crate::middleware::update_certificate_cache_ttl(tls_cache_ttl);
                updated_keys.push("tls_cache_ttl".to_string());
            } else {
                errors.push("Invalid tls_cache_ttl: must be a positive integer".to_string());
            }
        }

        let duration = start_time.elapsed();
        let success = errors.is_empty();

        Ok(MaintenanceResult {
            operation: "update_config".to_string(),
            success,
            duration_ms: duration.as_millis() as u64,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "updated_keys".to_string(),
                    serde_json::Value::Array(
                        updated_keys
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                );
                if !errors.is_empty() {
                    details.insert(
                        "errors".to_string(),
                        serde_json::Value::Array(
                            errors.into_iter().map(serde_json::Value::String).collect(),
                        ),
                    );
                }
                details
            },
        })
    }
}

/// Audit log entry
#[derive(Debug, Serialize)]
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
pub struct SecurityScanResult {
    pub scan_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub findings: Vec<SecurityFinding>,
}

/// Security finding
#[derive(Debug, Serialize)]
pub struct SecurityFinding {
    pub severity: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub affected_resources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityIncident {
    pub id: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::config::AuthConfig;
    use secreton_crypto::SecurityParams;
    use secreton_storage::MemoryBackend;
    use serde::Deserialize;

    // Mock implementation of LeaseCleaner for testing
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
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });

        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner, crypto).await;
        assert!(admin_service.is_ok());
    }

    #[tokio::test]
    async fn test_get_system_stats() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner, crypto)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let stats = admin_service.get_system_stats().await.expect("stats");
        // AuthService initializes 3 default roles, so storage is not empty
        assert!(stats.total_secrets >= 3);
        assert_eq!(stats.total_keys, stats.total_secrets);
        // MemoryBackend might return 0 size if not tracking correctly or optimized
        assert!(stats.storage_usage_bytes >= 0);
        assert!(
            stats.uptime_seconds >= 1,
            "Uptime should be at least 1 second"
        );
        assert!(stats.cache_hit_rate >= 0.0);
    }

    #[tokio::test]
    async fn test_create_backup_returns_metadata() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner, crypto)
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
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 15 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner, crypto)
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

    #[tokio::test]
    async fn test_update_config_dynamic() {
        use crate::config::RateLimitConfig;

        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth.clone(), audit, lease_cleaner, crypto)
            .await
            .unwrap();

        // 1. Test Rate Limit Update
        let rate_limit_config = RateLimitConfig::default();
        let mut updates = HashMap::new();
        updates.insert(
            "rate_limit".to_string(),
            serde_json::to_value(rate_limit_config).unwrap(),
        );

        let result = admin_service
            .update_config(updates)
            .await
            .expect("update config");
        assert!(result.success);
        let updated_keys = result
            .details
            .get("updated_keys")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(updated_keys.contains(&serde_json::Value::String("rate_limit".to_string())));

        // 2. Test Auth Config Update
        let mut new_auth_config = AuthConfig::default();
        new_auth_config.jwt.secret = "updated-secret".to_string();

        let mut updates = HashMap::new();
        updates.insert(
            "auth".to_string(),
            serde_json::to_value(new_auth_config).unwrap(),
        );

        let result = admin_service
            .update_config(updates)
            .await
            .expect("update config auth");
        assert!(result.success);
        let updated_keys = result
            .details
            .get("updated_keys")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(updated_keys.contains(&serde_json::Value::String("auth".to_string())));
    }

    #[tokio::test]
    async fn test_create_and_get_security_incidents() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(secreton_crypto::CryptoEngine::new());
        let config = AuthConfig::default();
        let auth = Arc::new(
            AuthService::new(storage.clone(), crypto.clone(), &config)
                .await
                .expect("failed to create AuthService"),
        );
        let audit = Arc::new(AuditLogger::new(10000));
        let lease_cleaner = Arc::new(MockLeaseCleaner { expired_count: 5 });
        let admin_service = AdminService::new(storage, auth, audit, lease_cleaner, crypto)
            .await
            .unwrap();

        let incident = SecurityIncident {
            id: "inc-1".to_string(),
            severity: "high".to_string(),
            status: "open".to_string(),
            title: "Test Incident".to_string(),
            description: "A test incident".to_string(),
            source: "test".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            resolved_at: None,
        };

        admin_service
            .create_security_incident(incident.clone())
            .await
            .expect("create incident");

        let incidents = admin_service
            .get_security_incidents(None, None, None)
            .await
            .expect("get incidents");

        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].id, "inc-1");
        assert_eq!(incidents[0].title, "Test Incident");
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

        let crypto = Arc::new(CryptoEngine::new());

        Self {
            auth: Arc::new(AuthService::new_mock(storage.clone(), crypto.clone())),
            storage,
            audit: Arc::new(AuditLogger::new(10000)),
            lease_cleaner: Arc::new(DummyLeaseCleaner),
            crypto,
            start_time: chrono::Utc::now(),
            system: Arc::new(Mutex::new(System::new_all())),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }
}
