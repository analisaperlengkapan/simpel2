//! Raft Cluster Implementation
//!
//! Provides the main RaftCluster struct for distributed consensus-based storage.

use super::combined_storage::SecretonRaftStorage;
use super::config_builder::RaftClusterConfigBuilder;
use super::network::{NetworkConfig, SecretonNetwork, SecretonNetworkFactory};
use super::state_machine::{SecretonStateMachine, StateMachineCommand, StateMachineResponse};
use super::types::{NodeId, Raft, SecretonTypeConfig};
use crate::{QueryParams, StorageError, StorageResult, VaultEntry};
use openraft::storage::Adaptor;
use openraft::{BasicNode, Config};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for the Raft cluster
#[derive(Debug, Clone)]
pub struct RaftClusterConfig {
    /// This node's ID in the cluster
    pub node_id: NodeId,
    /// Peer addresses (node_id -> address)
    pub peers: HashMap<NodeId, String>,
    /// Election timeout in milliseconds
    pub election_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Maximum entries per append request
    pub max_payload_entries: u64,
    /// Enable automatic tick for leader election
    pub enable_tick: bool,
}

impl Default for RaftClusterConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            peers: HashMap::new(),
            election_timeout_ms: 1000,
            heartbeat_interval_ms: 300,
            max_payload_entries: 300,
            enable_tick: true,
        }
    }
}

impl RaftClusterConfig {
    /// Create a new builder for RaftClusterConfig
    pub fn builder(node_id: NodeId) -> RaftClusterConfigBuilder {
        RaftClusterConfigBuilder::new(node_id)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.heartbeat_interval_ms >= self.election_timeout_ms {
            return Err(format!(
                "heartbeat_interval_ms ({}) must be less than election_timeout_ms ({})",
                self.heartbeat_interval_ms, self.election_timeout_ms
            ));
        }

        if self.max_payload_entries == 0 {
            return Err("max_payload_entries must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Raft-based distributed storage cluster
///
/// Provides linearizable distributed storage using the Raft consensus protocol.
/// This implements the StorageBackend trait and can be used as a drop-in
/// replacement for other storage backends.
pub struct RaftCluster {
    /// The underlying Raft instance
    pub raft: Raft,
    /// Shared state machine for read operations
    pub state_machine: Arc<RwLock<SecretonStateMachine>>,
    /// Combined storage instance
    storage: Arc<SecretonRaftStorage>,
    /// Cluster configuration
    #[allow(dead_code)]
    config: RaftClusterConfig,
    /// Timestamp of last snapshot
    pub last_snapshot_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl RaftCluster {
    /// Create a new Raft cluster node
    pub async fn new(cluster_config: RaftClusterConfig) -> StorageResult<Self> {
        cluster_config
            .validate()
            .map_err(|e| StorageError::ConfigError { message: e })?;

        let raft_config = Config {
            election_timeout_min: cluster_config.election_timeout_ms,
            election_timeout_max: cluster_config.election_timeout_ms * 2,
            heartbeat_interval: cluster_config.heartbeat_interval_ms,
            max_payload_entries: cluster_config.max_payload_entries,
            enable_tick: cluster_config.enable_tick,
            ..Default::default()
        };

        let raft_config = Arc::new(raft_config);
        let storage = Arc::new(SecretonRaftStorage::new());
        let state_machine = Arc::new(RwLock::new(SecretonStateMachine::new()));

        // Create network factory
        let network_factory =
            SecretonNetworkFactory::new(cluster_config.peers.clone(), NetworkConfig::default());

        // Create the Raft instance using the Adaptor pattern
        let log_store = Adaptor::new(storage.clone());
        let sm_store = Adaptor::new(storage.clone());

        let raft = Raft::new(
            cluster_config.node_id,
            raft_config,
            network_factory,
            log_store,
            sm_store,
        )
        .await
        .map_err(|e| StorageError::BackendError {
            backend: "raft".to_string(),
            message: format!("Failed to create Raft instance: {}", e),
        })?;

        Ok(Self {
            raft,
            state_machine,
            storage,
            config: cluster_config,
            last_snapshot_time: None,
        })
    }

    /// Propose a command to the Raft cluster
    pub async fn propose_command(
        &self,
        command: StateMachineCommand,
    ) -> StorageResult<StateMachineResponse> {
        let response =
            self.raft
                .client_write(command)
                .await
                .map_err(|e| StorageError::BackendError {
                    backend: "raft".to_string(),
                    message: format!("Raft write failed: {}", e),
                })?;

        Ok(response.data)
    }

    /// Initialize this node as the leader of a single-node cluster
    pub async fn initialize_single_node(&self) -> StorageResult<()> {
        let mut members = std::collections::BTreeMap::new();
        members.insert(self.config.node_id, BasicNode::default());

        self.raft
            .initialize(members)
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to initialize cluster: {}", e),
            })?;

        Ok(())
    }

    /// Check if this node is the current leader
    pub fn is_leader(&self) -> bool {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader == Some(self.config.node_id)
    }

    /// Get the current leader's node ID, if known
    pub fn current_leader(&self) -> Option<NodeId> {
        self.raft.metrics().borrow().current_leader
    }

    /// Get cluster metrics for monitoring
    pub fn get_metrics(&self) -> openraft::RaftMetrics<NodeId, BasicNode> {
        self.raft.metrics().borrow().clone()
    }
}
