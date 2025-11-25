use crate::{
    HealthStatus, QueryParams, StorageBackend, StorageError, StorageResult, StorageStats,
    StorageTransaction, VaultEntry,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub mod combined_storage;
pub mod config_builder;
pub mod metrics;
mod network;
mod state_machine;
mod types;

pub use combined_storage::SecretonRaftStorage;
pub use config_builder::RaftClusterConfigBuilder;
pub use metrics::{HealthStatus as RaftHealthStatus, MetricsCollector, RaftMetrics};

pub use network::{NetworkConfig, SecretonNetwork, SecretonNetworkFactory};
pub use state_machine::{SecretonStateMachine, StateMachineCommand, StateMachineResponse};
pub use types::{Config, Entry, LogId, Membership, NodeId, Raft, SecretonTypeConfig, Vote};

#[derive(Debug, Clone)]
pub struct RaftClusterConfig {
    pub node_id: NodeId,
    pub peers: HashMap<NodeId, String>,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_payload_entries: u64,
    pub enable_tick: bool,
}

impl Default for RaftClusterConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            peers: HashMap::new(),
            election_timeout_ms: 1000,
            heartbeat_interval_ms: 300,
            max_payload_entries: 1000,
            enable_tick: true,
        }
    }
}

impl RaftClusterConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.node_id == 0 {
            return Err("node_id cannot be 0".to_string());
        }

        if self.heartbeat_interval_ms == 0 {
            return Err("heartbeat_interval_ms must be greater than 0".to_string());
        }

        if self.election_timeout_ms == 0 {
            return Err("election_timeout_ms must be greater than 0".to_string());
        }

        if self.heartbeat_interval_ms >= self.election_timeout_ms {
            return Err("heartbeat_interval_ms must be less than election_timeout_ms".to_string());
        }

        if self.max_payload_entries == 0 {
            return Err("max_payload_entries must be greater than 0".to_string());
        }

        // Check for self in peers list
        if self.peers.contains_key(&self.node_id) {
            return Err("peers list should not contain self (node_id)".to_string());
        }

        Ok(())
    }

    /// Create a builder for RaftClusterConfig
    pub fn builder(node_id: u64) -> RaftClusterConfigBuilder {
        RaftClusterConfigBuilder::new(node_id)
    }

    /// Create development configuration preset
    ///
    /// - Shorter timeouts for faster testing
    /// - Smaller payload for quick iteration
    pub fn development(node_id: u64) -> Self {
        Self {
            node_id,
            peers: HashMap::new(),
            election_timeout_ms: 500, // Shorter for faster tests
            heartbeat_interval_ms: 150,
            max_payload_entries: 100,
            enable_tick: true,
        }
    }

    /// Create staging configuration preset
    ///
    /// - Balanced timeouts
    /// - Moderate payload size
    pub fn staging(node_id: u64) -> Self {
        Self {
            node_id,
            peers: HashMap::new(),
            election_timeout_ms: 1500,
            heartbeat_interval_ms: 450,
            max_payload_entries: 500,
            enable_tick: true,
        }
    }

    /// Create production configuration preset
    ///
    /// - Conservative timeouts for stability
    /// - Large payload for efficiency
    pub fn production(node_id: u64) -> Self {
        Self {
            node_id,
            peers: HashMap::new(),
            election_timeout_ms: 3000, // Longer for network stability
            heartbeat_interval_ms: 900,
            max_payload_entries: 2000, // Larger batches
            enable_tick: true,
        }
    }

    pub fn to_openraft_config(&self) -> Config {
        Config {
            cluster_name: "secreton-cluster".to_string(),
            heartbeat_interval: self.heartbeat_interval_ms,
            election_timeout_min: self.election_timeout_ms,
            election_timeout_max: self.election_timeout_ms * 2,
            max_payload_entries: self.max_payload_entries,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct RaftStatus {
    pub node_id: NodeId,
    pub current_term: u64,
    pub leader_id: Option<NodeId>,
    pub is_leader: bool,
    pub membership: Vec<NodeId>,
    pub last_applied: Option<u64>,
    pub last_log_index: Option<u64>,
}

pub struct RaftCluster {
    config: RaftClusterConfig,
    raft: Arc<Raft>,
    storage: Arc<SecretonRaftStorage>,
    /// Snapshot configuration
    snapshot_config: SnapshotConfig,
}

/// Snapshot configuration
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Enable automatic snapshots
    pub enabled: bool,
    /// Snapshot interval in seconds
    pub interval_secs: u64,
    /// Log entries threshold for triggering snapshot
    pub log_entries_threshold: u64,
    /// Maximum number of snapshots to retain
    pub max_snapshots: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600, // 1 hour
            log_entries_threshold: 10000,
            max_snapshots: 5,
        }
    }
}

impl RaftCluster {
    pub async fn new(config: RaftClusterConfig) -> StorageResult<Self> {
        // Create combined storage
        let storage = SecretonRaftStorage::new();
        let raft_config = Arc::new(config.to_openraft_config());

        // Create network factory with peer addresses
        let network_factory = crate::raft::SecretonNetworkFactory::new(
            config.peers.clone(),
            crate::raft::NetworkConfig::default(),
        );

        // Use Adaptor to split storage into log_store and state_machine
        let (log_store, state_machine) = openraft::storage::Adaptor::new(storage.clone());

        // Create Raft instance with split stores
        let raft = openraft::Raft::new(
            config.node_id,
            raft_config,
            network_factory,
            log_store,
            state_machine,
        )
        .await
        .map_err(|e| StorageError::BackendError {
            backend: "raft".to_string(),
            message: format!("Failed to create Raft node: {}", e),
        })?;

        let cluster = Self {
            config,
            raft: Arc::new(raft),
            storage: Arc::new(storage),
            snapshot_config: SnapshotConfig::default(),
        };

        // Start automatic snapshot task if enabled
        if cluster.snapshot_config.enabled {
            cluster.start_snapshot_automation();
        }

        Ok(cluster)
    }

    /// Start automatic snapshot creation task
    fn start_snapshot_automation(&self) {
        let raft = Arc::clone(&self.raft);
        let config = self.snapshot_config.clone();
        let node_id = self.config.node_id;

        tokio::spawn(async move {
            let interval = Duration::from_secs(config.interval_secs);

            loop {
                tokio::time::sleep(interval).await;

                // Only leader creates snapshots
                let metrics = raft.metrics().borrow().clone();
                if metrics.current_leader != Some(node_id) {
                    tracing::debug!("Node {} is not leader, skipping snapshot", node_id);
                    continue;
                }

                // Check if snapshot is needed based on log size
                let log_size = metrics.last_log_index.unwrap_or(0);
                let last_snapshot = metrics.snapshot.as_ref().map(|s| s.index).unwrap_or(0);
                let entries_since_snapshot = log_size.saturating_sub(last_snapshot);

                if entries_since_snapshot >= config.log_entries_threshold {
                    tracing::info!(
                        "Creating automatic snapshot: {} entries since last snapshot (threshold: {})",
                        entries_since_snapshot,
                        config.log_entries_threshold
                    );

                    match raft.trigger().snapshot().await {
                        Ok(_) => {
                            tracing::info!("Snapshot created successfully at index {}", log_size);

                            #[cfg(feature = "metrics")]
                            {
                                metrics::counter!("secreton_raft_snapshots_created").increment(1);
                                metrics::gauge!(
                                    "secreton_raft_last_snapshot_index",
                                    log_size as f64
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to create snapshot: {}", e);

                            #[cfg(feature = "metrics")]
                            {
                                metrics::counter!("secreton_raft_snapshot_errors").increment(1);
                            }
                        }
                    }
                } else {
                    tracing::debug!(
                        "Snapshot not needed: {} entries since last snapshot (threshold: {})",
                        entries_since_snapshot,
                        config.log_entries_threshold
                    );
                }
            }
        });
    }

    pub async fn status(&self) -> StorageResult<RaftStatus> {
        let metrics = self.raft.metrics().borrow().clone();

        Ok(RaftStatus {
            node_id: self.config.node_id,
            current_term: metrics.current_term,
            leader_id: metrics.current_leader,
            is_leader: metrics.current_leader == Some(self.config.node_id),
            membership: metrics.membership_config.membership().voter_ids().collect(),
            last_applied: metrics.last_applied.map(|l| l.index),
            last_log_index: metrics.last_log_index,
        })
    }

    pub async fn is_leader(&self) -> bool {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader == Some(self.config.node_id)
    }

    /// Get access to the underlying storage for metrics/monitoring
    pub fn get_storage(&self) -> Arc<SecretonRaftStorage> {
        Arc::clone(&self.storage)
    }

    /// Get metrics snapshot for monitoring
    pub async fn get_metrics(&self) -> StorageResult<RaftMetrics> {
        let raft_metrics = self.raft.metrics().borrow().clone();

        Ok(RaftMetrics {
            node_id: self.config.node_id,
            current_term: raft_metrics.current_term,
            leader_id: raft_metrics.current_leader,
            is_leader: raft_metrics.current_leader == Some(self.config.node_id),
            leader_elections: raft_metrics.current_term, // Approximate
            last_applied_index: raft_metrics.last_applied.map(|l| l.index),
            last_log_index: raft_metrics.last_log_index,
            log_size: raft_metrics.last_log_index.unwrap_or(0),
            cluster_size: raft_metrics
                .membership_config
                .membership()
                .voter_ids()
                .count(),
            committed_entries: raft_metrics.last_applied.map(|l| l.index).unwrap_or(0),
            avg_commit_latency_ms: None, // TODO: Track with MetricsCollector
            snapshots_created: 0,        // TODO: Track snapshots
            last_snapshot_index: raft_metrics.snapshot.map(|meta| meta.index),
            health: if raft_metrics.current_leader.is_some() {
                metrics::HealthStatus::Healthy
            } else {
                metrics::HealthStatus::Degraded
            },
        })
    }

    pub async fn leader_id(&self) -> Option<NodeId> {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader
    }

    pub async fn propose(&self, data: StateMachineCommand) -> StorageResult<StateMachineResponse> {
        let response = self.raft.client_write(data).await.map_err(|e| {
            StorageError::ReplicationError(format!("Failed to propose command: {}", e))
        })?;
        Ok(response.data)
    }

    pub async fn add_node(&self, node_id: NodeId, address: String) -> StorageResult<()> {
        // First add as learner
        let node = openraft::BasicNode {
            addr: address.clone(),
        };
        self.raft
            .add_learner(node_id, node, true)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to add learner: {}", e)))?;

        // Then update membership
        let mut members = self.get_current_members().await?;
        members.insert(node_id, openraft::BasicNode { addr: address });

        let member_ids: std::collections::BTreeSet<NodeId> = members.keys().cloned().collect();
        self.raft
            .change_membership(member_ids, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to add node: {}", e)))?;
        Ok(())
    }

    pub async fn remove_node(&self, node_id: NodeId) -> StorageResult<()> {
        let mut members = self.get_current_members().await?;
        members.remove(&node_id);

        let member_ids: std::collections::BTreeSet<NodeId> = members.keys().cloned().collect();
        self.raft
            .change_membership(member_ids, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to remove node: {}", e)))?;
        Ok(())
    }

    async fn get_current_members(&self) -> StorageResult<HashMap<NodeId, openraft::BasicNode>> {
        let metrics = self.raft.metrics().borrow().clone();
        let mut members = HashMap::new();

        for node_id in metrics.membership_config.membership().voter_ids() {
            if let Some(addr) = self.config.peers.get(&node_id) {
                members.insert(node_id, openraft::BasicNode { addr: addr.clone() });
            }
        }

        Ok(members)
    }

    /// Manually trigger a snapshot
    pub async fn create_snapshot(&self) -> StorageResult<()> {
        tracing::info!("Manually triggering Raft snapshot");

        self.raft
            .trigger()
            .snapshot()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to create snapshot: {}", e),
            })?;

        tracing::info!("Snapshot created successfully");
        Ok(())
    }

    pub async fn shutdown(&self) -> StorageResult<()> {
        self.raft
            .shutdown()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to shutdown Raft: {}", e),
            })?;
        Ok(())
    }

    /// Get reference to self as Any for downcasting
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl StorageBackend for RaftCluster {
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        if entry.path.trim().is_empty() {
            return Err(StorageError::InvalidQuery {
                message: "Path cannot be empty".to_string(),
            });
        }

        let response = self
            .propose(StateMachineCommand::Store(entry.clone()))
            .await?;

        match response {
            StateMachineResponse::Stored(_) | StateMachineResponse::Success => Ok(()),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft store failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for store: {:?}",
                other
            ))),
        }
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        let entry = self.storage.get_entry_by_id(id).await;
        Ok(entry)
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        let entry = self.storage.get_entry_by_path(path).await;
        Ok(entry)
    }

    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        let response = self
            .propose(StateMachineCommand::Update(entry.clone()))
            .await?;

        match response {
            StateMachineResponse::Updated(_) | StateMachineResponse::Success => Ok(()),
            StateMachineResponse::Error(msg) if msg == "Entry not found" => {
                Err(StorageError::NotFound {
                    resource_type: "VaultEntry".to_string(),
                    id: entry.id.to_string(),
                })
            }
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft update failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for update: {:?}",
                other
            ))),
        }
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let response = self.propose(StateMachineCommand::Delete(id)).await?;

        match response {
            StateMachineResponse::Deleted(existed) => Ok(existed),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft delete_by_id failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for delete_by_id: {:?}",
                other
            ))),
        }
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let response = self
            .propose(StateMachineCommand::DeleteByPath(path.to_string()))
            .await?;

        match response {
            StateMachineResponse::Deleted(existed) => Ok(existed),
            StateMachineResponse::Error(msg) => Err(StorageError::ReplicationError(format!(
                "Raft delete_by_path failed: {}",
                msg
            ))),
            other => Err(StorageError::ReplicationError(format!(
                "Unexpected Raft response for delete_by_path: {:?}",
                other
            ))),
        }
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        let entries = self.storage.list_entries(params).await;
        Ok(entries)
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let count = self.storage.count_entries(params).await;
        Ok(count)
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let exists = self.storage.exists_path(path).await;
        Ok(exists)
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        Err(StorageError::TransactionNotSupported {
            backend: "raft".to_string(),
        })
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        let metrics = self.get_metrics().await?;

        let is_healthy = matches!(metrics.health, RaftHealthStatus::Healthy);

        Ok(HealthStatus {
            is_healthy,
            response_time_ms: 0.0,
            connections_active: 1,
            connections_idle: 0,
            last_error: None,
            uptime_seconds: 0,
        })
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let entries = self.storage.list_entries(&QueryParams::new()).await;
        let total_entries = entries.len() as u64;
        let total_size_bytes: u64 = entries.iter().map(|e| e.encrypted_data.len() as u64).sum();

        let average_entry_size = if total_entries > 0 {
            total_size_bytes as f64 / total_entries as f64
        } else {
            0.0
        };

        let mut entries_by_security_level = HashMap::new();
        let mut expired_entries = 0u64;

        for entry in &entries {
            *entries_by_security_level
                .entry(entry.security_level)
                .or_insert(0) += 1;

            if entry.is_expired() {
                expired_entries += 1;
            }
        }

        Ok(StorageStats {
            backend_type: "raft".to_string(),
            total_entries,
            total_size_bytes,
            average_entry_size,
            entries_by_security_level,
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries,
            last_backup: None,
            metadata: serde_json::json!({
                "node_id": self.config.node_id,
            }),
        })
    }

    async fn migrate(&self) -> StorageResult<()> {
        // Raft-based storage does not require schema migrations
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cluster_creation() {
        let config = RaftClusterConfig::default();
        let cluster = RaftCluster::new(config).await;
        assert!(cluster.is_ok());
    }

    #[tokio::test]
    async fn test_cluster_status() {
        let config = RaftClusterConfig::default();
        let cluster = RaftCluster::new(config).await.unwrap();
        let status = cluster.status().await;
        assert!(status.is_ok());
    }
}
