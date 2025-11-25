//! OpenRaft Network Layer Implementation
//!
//! Provides gRPC-based peer-to-peer communication for Raft consensus.

use super::types::{NodeId, SecretonTypeConfig};
use openraft::BasicNode;
use openraft::error::{InstallSnapshotError, RPCError, RaftError};
use openraft::network::{RPCOption, RaftNetwork, RaftNetworkFactory};
use openraft::raft::{AppendEntriesRequest, InstallSnapshotRequest, VoteRequest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

// For now, use in-memory mock network until we implement full gRPC
// This allows the code to compile and we can test cluster formation
// TODO: Implement full gRPC network layer with proto definitions

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 3000,
            request_timeout_ms: 5000,
            max_retries: 3,
        }
    }
}

/// Mock Raft network implementation for testing
///
/// This is a simplified in-memory implementation that allows cluster
/// code to compile and be tested without full gRPC infrastructure.
/// For production, this should be replaced with actual gRPC network layer.
pub struct SecretonNetwork {
    /// Peer addresses (node_id -> address)
    #[allow(dead_code)] // Reserved for future gRPC implementation
    peers: Arc<RwLock<HashMap<NodeId, String>>>,
    /// Network configuration
    #[allow(dead_code)] // Reserved for future gRPC implementation
    config: NetworkConfig,
    /// Target node for this network instance
    #[allow(dead_code)] // Reserved for future gRPC implementation
    target_node: Option<NodeId>,
}

impl SecretonNetwork {
    /// Create a new network instance
    pub fn new(peers: HashMap<NodeId, String>, config: NetworkConfig) -> Self {
        Self {
            peers: Arc::new(RwLock::new(peers)),
            config,
            target_node: None,
        }
    }

    /// Create instance for specific target
    fn for_target(peers: HashMap<NodeId, String>, config: NetworkConfig, target: NodeId) -> Self {
        Self {
            peers: Arc::new(RwLock::new(peers)),
            config,
            target_node: Some(target),
        }
    }
}

impl RaftNetwork<SecretonTypeConfig> for SecretonNetwork {
    async fn append_entries(
        &mut self,
        rpc: AppendEntriesRequest<SecretonTypeConfig>,
        _option: RPCOption,
    ) -> Result<
        openraft::raft::AppendEntriesResponse<NodeId>,
        RPCError<NodeId, BasicNode, RaftError<NodeId>>,
    > {
        debug!("Mock AppendEntries from {:?}", rpc.vote.leader_id);

        // Mock response - in real impl, this would call gRPC
        Ok(openraft::raft::AppendEntriesResponse::Success)
    }

    async fn vote(
        &mut self,
        rpc: VoteRequest<NodeId>,
        _option: RPCOption,
    ) -> Result<openraft::raft::VoteResponse<NodeId>, RPCError<NodeId, BasicNode, RaftError<NodeId>>>
    {
        debug!("Mock Vote request from {:?}", rpc.vote.leader_id);

        // Mock response - grant vote
        Ok(openraft::raft::VoteResponse {
            vote: rpc.vote,
            vote_granted: true,
            last_log_id: rpc.last_log_id,
        })
    }

    async fn install_snapshot(
        &mut self,
        rpc: InstallSnapshotRequest<SecretonTypeConfig>,
        _option: RPCOption,
    ) -> Result<
        openraft::raft::InstallSnapshotResponse<NodeId>,
        RPCError<NodeId, BasicNode, RaftError<NodeId, InstallSnapshotError>>,
    > {
        debug!("Mock InstallSnapshot");

        // Mock response
        Ok(openraft::raft::InstallSnapshotResponse { vote: rpc.vote })
    }
}

/// Factory for creating network instances
pub struct SecretonNetworkFactory {
    peers: HashMap<NodeId, String>,
    config: NetworkConfig,
}

impl SecretonNetworkFactory {
    pub fn new(peers: HashMap<NodeId, String>, config: NetworkConfig) -> Self {
        Self { peers, config }
    }
}

impl RaftNetworkFactory<SecretonTypeConfig> for SecretonNetworkFactory {
    type Network = SecretonNetwork;

    async fn new_client(&mut self, target: NodeId, _node: &openraft::BasicNode) -> Self::Network {
        SecretonNetwork::for_target(self.peers.clone(), self.config.clone(), target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.connect_timeout_ms, 3000);
        assert_eq!(config.request_timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_network_creation() {
        let mut peers = HashMap::new();
        peers.insert(1, "localhost:7001".to_string());
        peers.insert(2, "localhost:7002".to_string());

        let network = SecretonNetwork::new(peers, NetworkConfig::default());
        assert_eq!(network.peers.read().await.len(), 2);
    }

    #[tokio::test]
    async fn test_network_factory() {
        let mut peers = HashMap::new();
        peers.insert(1, "localhost:7001".to_string());

        let mut factory = SecretonNetworkFactory::new(peers, NetworkConfig::default());
        let _network = factory
            .new_client(
                1,
                &openraft::BasicNode {
                    addr: "localhost:7001".to_string(),
                },
            )
            .await;
    }
}
