//! Secondary node management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Represents a secondary node in the replication cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryNode {
    /// Unique identifier for this secondary node
    pub id: String,

    /// gRPC endpoint for this secondary node
    pub endpoint: String,

    /// Last successful synchronization timestamp
    pub last_sync: DateTime<Utc>,

    /// Current replication lag
    pub lag: Duration,

    /// Current status of this secondary node
    pub status: ReplicationStatus,

    /// Node metadata
    pub metadata: NodeMetadata,
}

/// Metadata about a secondary node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    /// Node name (human-readable)
    pub name: Option<String>,

    /// Geographic region
    pub region: Option<String>,

    /// Data center or availability zone
    pub zone: Option<String>,

    /// Node version
    pub version: Option<String>,

    /// Custom tags
    #[serde(default)]
    pub tags: std::collections::HashMap<String, String>,
}

/// Status of a secondary node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationStatus {
    /// Node is healthy and replicating normally
    Healthy,

    /// Node is lagging behind primary
    Lagging,

    /// Node is disconnected from primary
    Disconnected,

    /// Node has failed and is not operational
    Failed,

    /// Node is initializing (bootstrapping)
    Initializing,

    /// Node is being promoted to primary
    Promoting,
}

impl SecondaryNode {
    /// Create a new secondary node
    pub fn new(endpoint: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            endpoint,
            last_sync: Utc::now(),
            lag: Duration::from_secs(0),
            status: ReplicationStatus::Initializing,
            metadata: NodeMetadata::default(),
        }
    }

    /// Update the last sync timestamp
    pub fn update_sync(&mut self) {
        self.last_sync = Utc::now();
    }

    /// Update the replication lag
    pub fn update_lag(&mut self, lag: Duration) {
        self.lag = lag;
    }

    /// Update the node status
    pub fn update_status(&mut self, status: ReplicationStatus) {
        self.status = status;
    }

    /// Check if the node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ReplicationStatus::Healthy)
    }

    /// Check if the node is operational (can serve requests)
    pub fn is_operational(&self) -> bool {
        matches!(
            self.status,
            ReplicationStatus::Healthy | ReplicationStatus::Lagging
        )
    }

    /// Get the time since last successful sync
    pub fn time_since_last_sync(&self) -> Duration {
        let now = Utc::now();
        (now - self.last_sync)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }
}

impl ReplicationStatus {
    /// Check if this status indicates the node is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Lagging)
    }

    /// Check if this status indicates a problem
    pub fn is_problematic(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Failed)
    }
}

impl std::fmt::Display for ReplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Lagging => write!(f, "lagging"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Failed => write!(f, "failed"),
            Self::Initializing => write!(f, "initializing"),
            Self::Promoting => write!(f, "promoting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_secondary_node() {
        let node = SecondaryNode::new("https://secondary.example.com:50051".to_string());
        assert!(!node.id.is_empty());
        assert_eq!(node.endpoint, "https://secondary.example.com:50051");
        assert_eq!(node.status, ReplicationStatus::Initializing);
        assert_eq!(node.lag, Duration::from_secs(0));
    }

    #[test]
    fn test_node_status_checks() {
        let mut node = SecondaryNode::new("https://test:50051".to_string());

        node.update_status(ReplicationStatus::Healthy);
        assert!(node.is_healthy());
        assert!(node.is_operational());

        node.update_status(ReplicationStatus::Lagging);
        assert!(!node.is_healthy());
        assert!(node.is_operational());

        node.update_status(ReplicationStatus::Failed);
        assert!(!node.is_healthy());
        assert!(!node.is_operational());
    }

    #[test]
    fn test_replication_status_properties() {
        assert!(ReplicationStatus::Healthy.is_operational());
        assert!(ReplicationStatus::Lagging.is_operational());
        assert!(!ReplicationStatus::Disconnected.is_operational());
        assert!(!ReplicationStatus::Failed.is_operational());

        assert!(!ReplicationStatus::Healthy.is_problematic());
        assert!(ReplicationStatus::Disconnected.is_problematic());
        assert!(ReplicationStatus::Failed.is_problematic());
    }
}
