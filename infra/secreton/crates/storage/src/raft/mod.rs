use crate::{StorageError, StorageResult};
use std::collections::HashMap;
use std::sync::Arc;

pub mod state_machine;
pub mod storage;
pub mod types;

pub use state_machine::{SecretonStateMachine, StateMachineCommand, StateMachineResponse};
pub use storage::SecretonStorage;
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
    storage: Arc<SecretonStorage>,
}

impl RaftCluster {
    pub async fn new(config: RaftClusterConfig) -> StorageResult<Self> {
        let storage = Arc::new(SecretonStorage::new());
        let raft_config = Arc::new(config.to_openraft_config());
        // Network layer would be implemented separately for actual cluster communication
        // For now, this is a placeholder for the Raft network interface
        let raft = openraft::Raft::new(config.node_id, raft_config, network, storage.clone())
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "raft".to_string(),
                message: format!("Failed to create Raft node: {}", e),
            })?;

        Ok(Self {
            config,
            raft: Arc::new(raft),
            storage,
        })
    }

    pub async fn status(&self) -> StorageResult<RaftStatus> {
        let metrics = self.raft.metrics().borrow().clone();

        Ok(RaftStatus {
            node_id: self.config.node_id,
            current_term: metrics.current_term,
            leader_id: metrics.current_leader,
            is_leader: metrics.current_leader == Some(self.config.node_id),
            membership: metrics
                .membership_config
                .membership()
                .voter_ids()
                .cloned()
                .collect(),
            last_applied: metrics.last_applied.map(|l| l.index),
            last_log_index: metrics.last_log_index.map(|l| l.index),
        })
    }

    pub async fn is_leader(&self) -> bool {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader == Some(self.config.node_id)
    }

    pub async fn leader_id(&self) -> Option<NodeId> {
        let metrics = self.raft.metrics().borrow().clone();
        metrics.current_leader
    }

    pub async fn propose(&self, cmd: StateMachineCommand) -> StorageResult<StateMachineResponse> {
        let data = bincode::encode_to_vec(&cmd, bincode::config::standard()).map_err(|e| {
            StorageError::SerializationError {
                message: format!("Failed to encode command: {}", e),
            }
        })?;
        let _response = self
            .raft
            .client_write(openraft::raft::ClientWriteRequest::new(
                openraft::EntryPayload::Normal(data.into()),
            ))
            .await
            .map_err(|e| {
                StorageError::ReplicationError(format!("Failed to propose command: {}", e))
            })?;
        Ok(StateMachineResponse::Success)
    }

    pub async fn add_node(&self, node_id: NodeId, address: String) -> StorageResult<()> {
        let mut members = self.get_current_members().await?;
        members.insert(node_id, openraft::BasicNode { addr: address });
        self.raft
            .change_membership(members, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to add node: {}", e)))?;
        Ok(())
    }

    pub async fn remove_node(&self, node_id: NodeId) -> StorageResult<()> {
        let mut members = self.get_current_members().await?;
        members.remove(&node_id);
        self.raft
            .change_membership(members, false)
            .await
            .map_err(|e| StorageError::ReplicationError(format!("Failed to remove node: {}", e)))?;
        Ok(())
    }

    async fn get_current_members(&self) -> StorageResult<HashMap<NodeId, openraft::BasicNode>> {
        let metrics = self.raft.metrics().borrow().clone();
        let mut members = HashMap::new();

        for node_id in metrics.membership_config.membership().voter_ids() {
            if let Some(addr) = self.config.peers.get(node_id) {
                members.insert(*node_id, openraft::BasicNode { addr: addr.clone() });
            }
        }

        Ok(members)
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
