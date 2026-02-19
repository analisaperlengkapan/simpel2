//! Raft cluster health check
//!
//! This health check verifies the health of the Raft cluster by checking:
//! - Whether a leader is elected
//! - Peer connectivity status
//! - Replication lag
//!
//! **Validates: Requirements 2.6.3** - Health check system must report Raft cluster status

use crate::{HealthCheck, HealthCheckResult, HealthStatus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Trait for accessing Raft cluster status
///
/// This trait abstracts the Raft cluster status check to allow for testing
/// and different implementations.
#[async_trait]
pub trait RaftClusterProvider: Send + Sync {
    /// Returns the current leader ID, if any
    async fn get_leader_id(&self) -> Option<u64>;

    /// Returns the list of peer node IDs
    async fn get_peer_ids(&self) -> Vec<u64>;

    /// Returns the replication lag for each peer (in log entries)
    async fn get_peer_lags(&self) -> HashMap<u64, u64>;

    /// Returns whether the Raft cluster is available
    async fn is_available(&self) -> bool;
}

/// Raft cluster health check
///
/// This health check monitors the Raft cluster status and reports:
/// - `Unhealthy` if Raft is completely unavailable
/// - `Degraded` if no leader is elected (cluster cannot serve writes)
/// - `Healthy` if cluster is operational with a leader
///
/// This is a critical health check that affects Kubernetes readiness probes.
/// When degraded or unhealthy, the pod should not receive write traffic.
///
/// # Example
///
/// ```rust
/// use secreton_health::{HealthCheck, HealthCheckRegistry};
/// use secreton_health::checks::RaftClusterHealthCheck;
/// use secreton_storage::raft::RaftCluster;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let raft_cluster = Arc::new(RaftCluster::new(Default::default()).await.unwrap());
///     let health_check = RaftClusterHealthCheck::new(raft_cluster);
///
///     let mut registry = HealthCheckRegistry::new();
///     registry.register(Box::new(health_check)).await.unwrap();
///
///     let results = registry.check_all().await;
///     println!("Raft cluster status: {:?}", results.overall_status());
/// }
/// ```
pub struct RaftClusterHealthCheck<P: RaftClusterProvider> {
    provider: Arc<P>,
    name: String,
    /// Maximum acceptable replication lag in log entries
    max_acceptable_lag: u64,
}

impl<P: RaftClusterProvider> RaftClusterHealthCheck<P> {
    /// Creates a new Raft cluster health check
    ///
    /// # Arguments
    ///
    /// * `provider` - The Raft cluster provider (typically a RaftCluster)
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            name: "raft_cluster".to_string(),
            max_acceptable_lag: 1000, // Default: 1000 entries
        }
    }

    /// Creates a new Raft cluster health check with a custom name
    pub fn with_name(provider: Arc<P>, name: String) -> Self {
        Self {
            provider,
            name,
            max_acceptable_lag: 1000,
        }
    }

    /// Sets the maximum acceptable replication lag
    pub fn with_max_lag(mut self, max_lag: u64) -> Self {
        self.max_acceptable_lag = max_lag;
        self
    }
}

#[async_trait]
impl<P: RaftClusterProvider + 'static> HealthCheck for RaftClusterHealthCheck<P> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Check if Raft is available
        let is_available = self.provider.is_available().await;
        if !is_available {
            let response_time_ms = start.elapsed().as_millis() as u64;
            return HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: Some("Raft cluster is unavailable".to_string()),
                response_time_ms,
                details: Some(
                    vec![
                        ("available".to_string(), serde_json::json!(false)),
                        ("has_leader".to_string(), serde_json::json!(false)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            };
        }

        // Check for leader
        let leader_id = self.provider.get_leader_id().await;
        if leader_id.is_none() {
            let response_time_ms = start.elapsed().as_millis() as u64;
            return HealthCheckResult {
                status: HealthStatus::Degraded,
                message: Some(
                    "Raft cluster has no leader. Leader election in progress.".to_string(),
                ),
                response_time_ms,
                details: Some(
                    vec![
                        ("available".to_string(), serde_json::json!(true)),
                        ("has_leader".to_string(), serde_json::json!(false)),
                        ("leader_id".to_string(), serde_json::json!(null)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            };
        }

        // Check peer connectivity and replication lag
        let peer_ids = self.provider.get_peer_ids().await;
        let peer_lags = self.provider.get_peer_lags().await;

        let mut high_lag_peers = Vec::new();
        for peer_id in &peer_ids {
            if let Some(&lag) = peer_lags.get(peer_id) {
                if lag > self.max_acceptable_lag {
                    high_lag_peers.push((*peer_id, lag));
                }
            }
        }

        let response_time_ms = start.elapsed().as_millis() as u64;

        // If there are peers with high lag, report as degraded
        if !high_lag_peers.is_empty() {
            let mut details = HashMap::new();
            details.insert("available".to_string(), serde_json::json!(true));
            details.insert("has_leader".to_string(), serde_json::json!(true));
            details.insert("leader_id".to_string(), serde_json::json!(leader_id));
            details.insert("peer_count".to_string(), serde_json::json!(peer_ids.len()));
            details.insert(
                "high_lag_peers".to_string(),
                serde_json::json!(high_lag_peers
                    .iter()
                    .map(|(id, lag)| format!("node_{}: {} entries", id, lag))
                    .collect::<Vec<_>>()),
            );

            return HealthCheckResult {
                status: HealthStatus::Degraded,
                message: Some(format!(
                    "Raft cluster operational but {} peer(s) have high replication lag",
                    high_lag_peers.len()
                )),
                response_time_ms,
                details: Some(details),
            };
        }

        // Cluster is healthy
        let mut details = HashMap::new();
        details.insert("available".to_string(), serde_json::json!(true));
        details.insert("has_leader".to_string(), serde_json::json!(true));
        details.insert("leader_id".to_string(), serde_json::json!(leader_id));
        details.insert("peer_count".to_string(), serde_json::json!(peer_ids.len()));
        details.insert(
            "max_peer_lag".to_string(),
            serde_json::json!(peer_lags.values().max().unwrap_or(&0)),
        );

        HealthCheckResult {
            status: HealthStatus::Healthy,
            message: Some(format!(
                "Raft cluster operational with leader node {}",
                leader_id.unwrap()
            )),
            response_time_ms,
            details: Some(details),
        }
    }

    fn is_critical(&self) -> bool {
        // Raft cluster status is critical - if unhealthy, the system cannot serve requests
        true
    }

    fn tags(&self) -> Vec<String> {
        vec![
            "raft".to_string(),
            "cluster".to_string(),
            "replication".to_string(),
            "kubernetes".to_string(),
            "readiness".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    /// Mock Raft cluster provider for testing
    struct MockRaftClusterProvider {
        available: AtomicBool,
        leader_id: Mutex<Option<u64>>,
        peer_ids: Mutex<Vec<u64>>,
        peer_lags: Mutex<HashMap<u64, u64>>,
    }

    impl MockRaftClusterProvider {
        fn new() -> Self {
            Self {
                available: AtomicBool::new(true),
                leader_id: Mutex::new(Some(1)),
                peer_ids: Mutex::new(vec![2, 3]),
                peer_lags: Mutex::new(HashMap::new()),
            }
        }

        fn set_available(&self, available: bool) {
            self.available.store(available, Ordering::SeqCst);
        }

        fn set_leader_id(&self, leader_id: Option<u64>) {
            *self.leader_id.lock().unwrap() = leader_id;
        }

        fn set_peer_ids(&self, peer_ids: Vec<u64>) {
            *self.peer_ids.lock().unwrap() = peer_ids;
        }

        fn set_peer_lags(&self, peer_lags: HashMap<u64, u64>) {
            *self.peer_lags.lock().unwrap() = peer_lags;
        }
    }

    #[async_trait]
    impl RaftClusterProvider for MockRaftClusterProvider {
        async fn get_leader_id(&self) -> Option<u64> {
            *self.leader_id.lock().unwrap()
        }

        async fn get_peer_ids(&self) -> Vec<u64> {
            self.peer_ids.lock().unwrap().clone()
        }

        async fn get_peer_lags(&self) -> HashMap<u64, u64> {
            self.peer_lags.lock().unwrap().clone()
        }

        async fn is_available(&self) -> bool {
            self.available.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn test_raft_cluster_check_healthy() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check = RaftClusterHealthCheck::new(provider.clone());

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.unwrap().contains("operational"));
        let details = result.details.unwrap();
        assert_eq!(details.get("has_leader").unwrap(), &serde_json::json!(true));
        assert_eq!(details.get("leader_id").unwrap(), &serde_json::json!(1));
    }

    #[tokio::test]
    async fn test_raft_cluster_check_no_leader() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        provider.set_leader_id(None);
        let check = RaftClusterHealthCheck::new(provider);

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Degraded);
        assert!(result.message.unwrap().contains("no leader"));
        let details = result.details.unwrap();
        assert_eq!(
            details.get("has_leader").unwrap(),
            &serde_json::json!(false)
        );
    }

    #[tokio::test]
    async fn test_raft_cluster_check_unavailable() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        provider.set_available(false);
        let check = RaftClusterHealthCheck::new(provider);

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.message.unwrap().contains("unavailable"));
        let details = result.details.unwrap();
        assert_eq!(
            details.get("available").unwrap(),
            &serde_json::json!(false)
        );
    }

    #[tokio::test]
    async fn test_raft_cluster_check_high_lag() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let mut lags = HashMap::new();
        lags.insert(2, 50); // Low lag
        lags.insert(3, 2000); // High lag (> 1000 default threshold)
        provider.set_peer_lags(lags);

        let check = RaftClusterHealthCheck::new(provider);

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Degraded);
        assert!(result.message.unwrap().contains("high replication lag"));
        let details = result.details.unwrap();
        assert_eq!(details.get("has_leader").unwrap(), &serde_json::json!(true));
    }

    #[tokio::test]
    async fn test_raft_cluster_check_custom_lag_threshold() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let mut lags = HashMap::new();
        lags.insert(2, 500); // Would be OK with default threshold
        provider.set_peer_lags(lags);

        let check = RaftClusterHealthCheck::new(provider).with_max_lag(100);

        let result = check.check().await;

        // Should be degraded because lag (500) > custom threshold (100)
        assert_eq!(result.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_raft_cluster_check_is_critical() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check = RaftClusterHealthCheck::new(provider);

        assert!(check.is_critical());
    }

    #[tokio::test]
    async fn test_raft_cluster_check_has_correct_tags() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check = RaftClusterHealthCheck::new(provider);

        let tags = check.tags();
        assert!(tags.contains(&"raft".to_string()));
        assert!(tags.contains(&"cluster".to_string()));
        assert!(tags.contains(&"replication".to_string()));
        assert!(tags.contains(&"kubernetes".to_string()));
        assert!(tags.contains(&"readiness".to_string()));
    }

    #[tokio::test]
    async fn test_raft_cluster_check_custom_name() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check =
            RaftClusterHealthCheck::with_name(provider, "custom_raft_check".to_string());

        assert_eq!(check.name(), "custom_raft_check");
    }

    #[tokio::test]
    async fn test_raft_cluster_check_response_time() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check = RaftClusterHealthCheck::new(provider);

        let result = check.check().await;

        // Response time should be very fast (< 100ms for in-memory check)
        assert!(result.response_time_ms < 100);
    }

    #[tokio::test]
    async fn test_raft_cluster_transitions() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        let check = RaftClusterHealthCheck::new(provider.clone());

        // Initially healthy
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);

        // Lose leader
        provider.set_leader_id(None);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Degraded);

        // Become unavailable
        provider.set_available(false);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Unhealthy);

        // Recover
        provider.set_available(true);
        provider.set_leader_id(Some(1));
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_raft_cluster_no_peers() {
        let provider = Arc::new(MockRaftClusterProvider::new());
        provider.set_peer_ids(vec![]); // Single-node cluster
        let check = RaftClusterHealthCheck::new(provider);

        let result = check.check().await;

        // Should still be healthy with no peers (single-node cluster)
        assert_eq!(result.status, HealthStatus::Healthy);
        let details = result.details.unwrap();
        assert_eq!(details.get("peer_count").unwrap(), &serde_json::json!(0));
    }
}
