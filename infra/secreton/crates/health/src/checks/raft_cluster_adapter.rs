//! Adapter for RaftCluster to implement RaftClusterProvider
//!
//! This module provides an adapter that allows the actual RaftCluster
//! from secreton-storage to be used with the RaftClusterHealthCheck.

use super::raft_cluster::RaftClusterProvider;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait that represents the minimal interface needed from RaftCluster
///
/// This trait is implemented by the actual RaftCluster from secreton-storage.
/// It allows the health check to work with RaftCluster without creating
/// a circular dependency.
#[async_trait]
pub trait RaftClusterLike: Send + Sync {
    /// Returns the current leader ID, if any
    async fn leader_id(&self) -> Option<u64>;

    /// Returns the cluster status
    async fn status(&self) -> Result<RaftStatusLike, String>;

    /// Returns whether the cluster is healthy
    async fn is_cluster_healthy(&self) -> bool;
}

/// Minimal representation of RaftStatus needed for health checks
pub struct RaftStatusLike {
    pub leader_id: Option<u64>,
    pub membership: Vec<u64>,
    pub peer_lags: HashMap<u64, u64>,
}

/// Adapter that implements RaftClusterProvider for any RaftClusterLike
pub struct RaftClusterAdapter<T: RaftClusterLike> {
    cluster: T,
}

impl<T: RaftClusterLike> RaftClusterAdapter<T> {
    pub fn new(cluster: T) -> Self {
        Self { cluster }
    }
}

#[async_trait]
impl<T: RaftClusterLike + 'static> RaftClusterProvider for RaftClusterAdapter<T> {
    async fn get_leader_id(&self) -> Option<u64> {
        self.cluster.leader_id().await
    }

    async fn get_peer_ids(&self) -> Vec<u64> {
        match self.cluster.status().await {
            Ok(status) => status.membership,
            Err(_) => Vec::new(),
        }
    }

    async fn get_peer_lags(&self) -> HashMap<u64, u64> {
        match self.cluster.status().await {
            Ok(status) => status.peer_lags,
            Err(_) => HashMap::new(),
        }
    }

    async fn is_available(&self) -> bool {
        // Try to get status - if it fails, cluster is unavailable
        self.cluster.status().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRaftCluster {
        leader_id: Option<u64>,
        healthy: bool,
    }

    #[async_trait]
    impl RaftClusterLike for MockRaftCluster {
        async fn leader_id(&self) -> Option<u64> {
            self.leader_id
        }

        async fn status(&self) -> Result<RaftStatusLike, String> {
            if !self.healthy {
                return Err("Cluster unavailable".to_string());
            }

            Ok(RaftStatusLike {
                leader_id: self.leader_id,
                membership: vec![1, 2, 3],
                peer_lags: HashMap::from([(2, 10), (3, 20)]),
            })
        }

        async fn is_cluster_healthy(&self) -> bool {
            self.healthy
        }
    }

    #[tokio::test]
    async fn test_adapter_get_leader_id() {
        let cluster = MockRaftCluster {
            leader_id: Some(1),
            healthy: true,
        };
        let adapter = RaftClusterAdapter::new(cluster);

        let leader_id = adapter.get_leader_id().await;
        assert_eq!(leader_id, Some(1));
    }

    #[tokio::test]
    async fn test_adapter_get_peer_ids() {
        let cluster = MockRaftCluster {
            leader_id: Some(1),
            healthy: true,
        };
        let adapter = RaftClusterAdapter::new(cluster);

        let peer_ids = adapter.get_peer_ids().await;
        assert_eq!(peer_ids, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_adapter_get_peer_lags() {
        let cluster = MockRaftCluster {
            leader_id: Some(1),
            healthy: true,
        };
        let adapter = RaftClusterAdapter::new(cluster);

        let peer_lags = adapter.get_peer_lags().await;
        assert_eq!(peer_lags.get(&2), Some(&10));
        assert_eq!(peer_lags.get(&3), Some(&20));
    }

    #[tokio::test]
    async fn test_adapter_is_available() {
        let cluster = MockRaftCluster {
            leader_id: Some(1),
            healthy: true,
        };
        let adapter = RaftClusterAdapter::new(cluster);

        assert!(adapter.is_available().await);
    }

    #[tokio::test]
    async fn test_adapter_unavailable_cluster() {
        let cluster = MockRaftCluster {
            leader_id: None,
            healthy: false,
        };
        let adapter = RaftClusterAdapter::new(cluster);

        assert!(!adapter.is_available().await);
        assert_eq!(adapter.get_peer_ids().await, Vec::<u64>::new());
        assert_eq!(adapter.get_peer_lags().await, HashMap::new());
    }
}
