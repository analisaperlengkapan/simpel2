//! Raft Node Implementation
//!
//! High-level Raft node with network integration.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use super::{RaftCluster, RaftClusterConfig, RaftMessage, RaftStatus};
use crate::{StorageBackend, VaultEntry};

/// Raft node configuration
#[derive(Debug, Clone)]
pub struct RaftNodeConfig {
    /// Node ID
    pub node_id: u64,
    
    /// Listen address for Raft protocol
    pub listen_addr: String,
    
    /// Cluster peers
    pub peers: HashMap<u64, String>,
    
    /// Data directory
    pub data_dir: String,
    
    /// Enable auto-join
    pub auto_join: bool,
}

impl Default for RaftNodeConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            listen_addr: "127.0.0.1:7000".to_string(),
            peers: HashMap::new(),
            data_dir: "/tmp/secreton-raft".to_string(),
            auto_join: false,
        }
    }
}

/// High-level Raft node with storage integration
pub struct RaftNode {
    config: RaftNodeConfig,
    cluster: Arc<RaftCluster>,
    proposals: Arc<RwLock<HashMap<Uuid, mpsc::Sender<Result<()>>>>>,
}

impl RaftNode {
    /// Create a new Raft node
    pub async fn new(config: RaftNodeConfig) -> Result<Self> {
        // Create Raft cluster config
        let cluster_config = RaftClusterConfig {
            node_id: config.node_id,
            peers: config.peers.clone(),
            ..Default::default()
        };
        
        // Create cluster
        let cluster = RaftCluster::new(cluster_config)?;
        
        Ok(Self {
            config,
            cluster: Arc::new(cluster),
            proposals: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start the Raft node
    pub async fn start(&self) -> Result<()> {
        // Bootstrap if this is the first node
        if self.config.peers.is_empty() {
            let mut cluster = Arc::clone(&self.cluster);
            // Bootstrap with single node
            // cluster.bootstrap(vec![self.config.node_id])?;
        }
        
        // Start the Raft event loop
        let cluster = Arc::clone(&self.cluster);
        tokio::spawn(async move {
            if let Err(e) = cluster.run().await {
                tracing::error!("Raft cluster error: {}", e);
            }
        });
        
        tracing::info!("Raft node {} started on {}", self.config.node_id, self.config.listen_addr);
        
        Ok(())
    }
    
    /// Store a vault entry (replicated)
    pub async fn store(&self, entry: &VaultEntry) -> Result<()> {
        // Serialize the entry
        let data = bincode::serialize(&super::state_machine::StateMachineCommand::Store(entry.clone()))
            .map_err(|e| anyhow!("Failed to serialize entry: {}", e))?;
        
        // Propose to cluster
        let proposal_id = self.cluster.propose(data).await?;
        
        // Wait for proposal to be committed
        let (tx, mut rx) = mpsc::channel(1);
        {
            let mut proposals = self.proposals.write().await;
            proposals.insert(proposal_id, tx);
        }
        
        // Wait for result with timeout
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Some(Ok(())) => Ok(()),
                    Some(Err(e)) => Err(e),
                    None => Err(anyhow!("Proposal channel closed")),
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                Err(anyhow!("Proposal timeout"))
            }
        }
    }
    
    /// Update a vault entry (replicated)
    pub async fn update(&self, entry: &VaultEntry) -> Result<()> {
        let data = bincode::serialize(&super::state_machine::StateMachineCommand::Update(entry.clone()))
            .map_err(|e| anyhow!("Failed to serialize entry: {}", e))?;
        
        let proposal_id = self.cluster.propose(data).await?;
        
        let (tx, mut rx) = mpsc::channel(1);
        {
            let mut proposals = self.proposals.write().await;
            proposals.insert(proposal_id, tx);
        }
        
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Some(Ok(())) => Ok(()),
                    Some(Err(e)) => Err(e),
                    None => Err(anyhow!("Proposal channel closed")),
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                Err(anyhow!("Proposal timeout"))
            }
        }
    }
    
    /// Delete a vault entry (replicated)
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let data = bincode::serialize(&super::state_machine::StateMachineCommand::Delete(id))
            .map_err(|e| anyhow!("Failed to serialize command: {}", e))?;
        
        let proposal_id = self.cluster.propose(data).await?;
        
        let (tx, mut rx) = mpsc::channel(1);
        {
            let mut proposals = self.proposals.write().await;
            proposals.insert(proposal_id, tx);
        }
        
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Some(Ok(())) => Ok(()),
                    Some(Err(e)) => Err(e),
                    None => Err(anyhow!("Proposal channel closed")),
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                Err(anyhow!("Proposal timeout"))
            }
        }
    }
    
    /// Add a node to the cluster
    pub async fn add_peer(&self, node_id: u64, address: String) -> Result<()> {
        self.cluster.add_node(node_id, address).await
    }
    
    /// Remove a node from the cluster
    pub async fn remove_peer(&self, node_id: u64) -> Result<()> {
        self.cluster.remove_node(node_id).await
    }
    
    /// Check if this node is the leader
    pub fn is_leader(&self) -> bool {
        self.cluster.is_leader()
    }
    
    /// Get current leader ID
    pub fn leader_id(&self) -> u64 {
        self.cluster.leader_id()
    }
    
    /// Get cluster status
    pub fn status(&self) -> RaftStatus {
        self.cluster.status()
    }
    
    /// Get node configuration
    pub fn config(&self) -> &RaftNodeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SecurityLevel;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_raft_node_creation() {
        let config = RaftNodeConfig::default();
        let node = RaftNode::new(config).await;
        assert!(node.is_ok());
    }
    
    #[tokio::test]
    async fn test_raft_node_start() {
        let config = RaftNodeConfig::default();
        let node = RaftNode::new(config).await.unwrap();
        
        let result = node.start().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_raft_node_status() {
        let config = RaftNodeConfig::default();
        let node = RaftNode::new(config).await.unwrap();
        
        let status = node.status();
        assert_eq!(status.node_id, 1);
    }
    
    fn create_test_entry() -> VaultEntry {
        VaultEntry {
            id: Uuid::new_v4(),
            path: "test/path".to_string(),
            encrypted_data: vec![1, 2, 3],
            encryption_metadata: serde_json::json!({}),
            security_level: SecurityLevel::Confidential,
            metadata: serde_json::json!({}),
            tags: vec![],
            version: 1,
            owner_id: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        }
    }
}
