use super::{NodeId, RaftClusterConfig};
use std::collections::HashMap;

/// Builder for RaftClusterConfig
///
/// # Example
///
/// ```rust
/// use secreton_storage::raft::RaftClusterConfig;
/// use std::collections::HashMap;
///
/// let config = RaftClusterConfig::builder(1)
///     .election_timeout_ms(1500)
///     .heartbeat_interval_ms(450)
///     .add_peer(2, "127.0.0.1:8002".to_string())
///     .add_peer(3, "127.0.0.1:8003".to_string())
///     .build()
///     .expect("valid configuration");
/// ```
pub struct RaftClusterConfigBuilder {
    node_id: NodeId,
    peers: HashMap<NodeId, String>,
    election_timeout_ms: u64,
    heartbeat_interval_ms: u64,
    max_payload_entries: u64,
    enable_tick: bool,
}

impl RaftClusterConfigBuilder {
    /// Create a new builder with default values
    pub fn new(node_id: u64) -> Self {
        let defaults = RaftClusterConfig::default();
        Self {
            node_id,
            peers: HashMap::new(),
            election_timeout_ms: defaults.election_timeout_ms,
            heartbeat_interval_ms: defaults.heartbeat_interval_ms,
            max_payload_entries: defaults.max_payload_entries,
            enable_tick: defaults.enable_tick,
        }
    }

    /// Set election timeout in milliseconds
    pub fn election_timeout_ms(mut self, timeout: u64) -> Self {
        self.election_timeout_ms = timeout;
        self
    }

    /// Set heartbeat interval in milliseconds
    pub fn heartbeat_interval_ms(mut self, interval: u64) -> Self {
        self.heartbeat_interval_ms = interval;
        self
    }

    /// Set maximum payload entries
    pub fn max_payload_entries(mut self, max: u64) -> Self {
        self.max_payload_entries = max;
        self
    }

    /// Enable or disable tick
    pub fn enable_tick(mut self, enable: bool) -> Self {
        self.enable_tick = enable;
        self
    }

    /// Add a peer to the cluster
    pub fn add_peer(mut self, node_id: NodeId, address: String) -> Self {
        self.peers.insert(node_id, address);
        self
    }

    /// Set all peers at once
    pub fn peers(mut self, peers: HashMap<NodeId, String>) -> Self {
        self.peers = peers;
        self
    }

    /// Build and validate the configuration
    pub fn build(self) -> Result<RaftClusterConfig, String> {
        let config = RaftClusterConfig {
            node_id: self.node_id,
            peers: self.peers,
            election_timeout_ms: self.election_timeout_ms,
            heartbeat_interval_ms: self.heartbeat_interval_ms,
            max_payload_entries: self.max_payload_entries,
            enable_tick: self.enable_tick,
        };

        // Validate before returning
        config.validate()?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default_values() {
        let config = RaftClusterConfigBuilder::new(1)
            .build()
            .expect("should build with defaults");

        assert_eq!(config.node_id, 1);
        assert_eq!(config.election_timeout_ms, 1000);
        assert_eq!(config.heartbeat_interval_ms, 300);
    }

    #[test]
    fn test_builder_custom_values() {
        let config = RaftClusterConfigBuilder::new(2)
            .election_timeout_ms(2000)
            .heartbeat_interval_ms(600)
            .max_payload_entries(500)
            .add_peer(1, "127.0.0.1:8001".to_string())
            .build()
            .expect("should build with custom values");

        assert_eq!(config.node_id, 2);
        assert_eq!(config.election_timeout_ms, 2000);
        assert_eq!(config.heartbeat_interval_ms, 600);
        assert_eq!(config.max_payload_entries, 500);
        assert_eq!(config.peers.len(), 1);
    }

    #[test]
    fn test_builder_validation_fails() {
        // Heartbeat >= election timeout should fail
        let result = RaftClusterConfigBuilder::new(1)
            .election_timeout_ms(1000)
            .heartbeat_interval_ms(1000)
            .build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("heartbeat_interval_ms must be less than")
        );
    }
}
