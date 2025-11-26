/// Raft cluster metrics for monitoring and observability
///
/// This module provides metrics collection and reporting for Raft cluster operations.
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Metrics snapshot for a Raft node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftMetrics {
    /// Node ID
    pub node_id: u64,

    /// Current Raft term
    pub current_term: u64,

    /// Current leader (if known)
    pub leader_id: Option<u64>,

    /// Whether this node is the leader
    pub is_leader: bool,

    /// Number of leader elections since startup
    pub leader_elections: u64,

    /// Last applied log index
    pub last_applied_index: Option<u64>,

    /// Last log index
    pub last_log_index: Option<u64>,

    /// Number of log entries
    pub log_size: u64,

    /// Number of cluster members
    pub cluster_size: usize,

    /// Number of committed entries
    pub committed_entries: u64,

    /// Average commit latency (milliseconds)
    pub avg_commit_latency_ms: Option<f64>,

    /// Number of snapshot operations
    pub snapshots_created: u64,

    /// Last snapshot index
    pub last_snapshot_index: Option<u64>,

    /// Health status
    pub health: HealthStatus,
}

/// Health status of a Raft node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Node is healthy and operational
    Healthy,

    /// Node is operational but degraded (e.g., lost quorum temporarily)
    Degraded,

    /// Node is unhealthy (e.g., cannot reach quorum)
    Unhealthy,

    /// Node is starting up
    Starting,

    /// Node is shutting down
    Shutting,
}

impl Default for RaftMetrics {
    fn default() -> Self {
        Self {
            node_id: 0,
            current_term: 0,
            leader_id: None,
            is_leader: false,
            leader_elections: 0,
            last_applied_index: None,
            last_log_index: None,
            log_size: 0,
            cluster_size: 0,
            committed_entries: 0,
            avg_commit_latency_ms: None,
            snapshots_created: 0,
            last_snapshot_index: None,
            health: HealthStatus::Starting,
        }
    }
}

impl RaftMetrics {
    /// Create a new metrics snapshot
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            ..Default::default()
        }
    }

    /// Check if the node has quorum (can process requests)
    pub fn has_quorum(&self) -> bool {
        // In a Raft cluster, we need majority
        // If we're the leader, we have quorum
        // If we're a follower and leader is known, cluster likely has quorum
        self.is_leader || self.leader_id.is_some()
    }

    /// Get replication lag (if follower)
    pub fn replication_lag(&self) -> Option<u64> {
        if let (Some(last_log), Some(last_applied)) = (self.last_log_index, self.last_applied_index)
        {
            Some(last_log.saturating_sub(last_applied))
        } else {
            None
        }
    }

    /// Format metrics as human-readable string
    pub fn to_string(&self) -> String {
        format!(
            "RaftMetrics {{ node={}, term={}, leader={:?}, is_leader={}, log={}/{}, health={:?} }}",
            self.node_id,
            self.current_term,
            self.leader_id,
            self.is_leader,
            self.last_applied_index.unwrap_or(0),
            self.last_log_index.unwrap_or(0),
            self.health
        )
    }
}

/// Metrics collector for Raft operations
///
/// Tracks timing and counts for various Raft operations
pub struct MetricsCollector {
    /// Commit latencies (for averaging)
    commit_latencies: Vec<Duration>,

    /// Max samples to keep
    max_samples: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            commit_latencies: Vec::new(),
            max_samples: 1000,
        }
    }

    /// Record a commit latency
    pub fn record_commit_latency(&mut self, latency: Duration) {
        self.commit_latencies.push(latency);

        // Keep only recent samples
        if self.commit_latencies.len() > self.max_samples {
            self.commit_latencies.drain(0..self.max_samples / 2);
        }
    }

    /// Get average commit latency
    pub fn avg_commit_latency(&self) -> Option<f64> {
        if self.commit_latencies.is_empty() {
            return None;
        }

        let sum: Duration = self.commit_latencies.iter().sum();
        let avg = sum / self.commit_latencies.len() as u32;
        Some(avg.as_secs_f64() * 1000.0) // Convert to ms
    }

    /// Get p99 commit latency
    pub fn p99_commit_latency(&self) -> Option<Duration> {
        if self.commit_latencies.is_empty() {
            return None;
        }

        let mut sorted = self.commit_latencies.clone();
        sorted.sort();

        let p99_index = (sorted.len() as f64 * 0.99) as usize;
        sorted.get(p99_index).copied()
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        self.commit_latencies.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = RaftMetrics::new(1);
        assert_eq!(metrics.node_id, 1);
        assert_eq!(metrics.current_term, 0);
        assert_eq!(metrics.health, HealthStatus::Starting);
    }

    #[test]
    fn test_has_quorum() {
        let mut metrics = RaftMetrics::new(1);
        metrics.is_leader = true;
        assert!(metrics.has_quorum());

        let mut follower = RaftMetrics::new(2);
        follower.leader_id = Some(1);
        assert!(follower.has_quorum());

        let orphan = RaftMetrics::new(3);
        assert!(!orphan.has_quorum());
    }

    #[test]
    fn test_replication_lag() {
        let mut metrics = RaftMetrics::new(1);
        metrics.last_log_index = Some(100);
        metrics.last_applied_index = Some(90);

        assert_eq!(metrics.replication_lag(), Some(10));
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.record_commit_latency(Duration::from_millis(10));
        collector.record_commit_latency(Duration::from_millis(20));
        collector.record_commit_latency(Duration::from_millis(30));

        let avg = collector.avg_commit_latency().unwrap();
        assert!(avg >= 19.0 && avg <= 21.0); // ~20ms average
    }
}
