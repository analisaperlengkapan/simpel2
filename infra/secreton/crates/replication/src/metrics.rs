//! Replication metrics for Prometheus
//!
//! This module provides Prometheus metrics for monitoring replication lag,
//! throughput, and health status.

use prometheus::{
    register_gauge_vec, register_histogram_vec, register_int_counter_vec, register_int_gauge_vec,
    GaugeVec, HistogramVec, IntCounterVec, IntGaugeVec,
};
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    /// Replication lag in bytes (sequence number difference)
    pub static ref REPLICATION_LAG_BYTES: IntGaugeVec = register_int_gauge_vec!(
        "secreton_replication_lag_bytes",
        "Replication lag in bytes (sequence number difference)",
        &["node_id", "mode"]
    ).unwrap();

    /// Replication lag in milliseconds (time difference)
    pub static ref REPLICATION_LAG_MS: GaugeVec = register_gauge_vec!(
        "secreton_replication_lag_ms",
        "Replication lag in milliseconds",
        &["node_id", "mode"]
    ).unwrap();

    /// Total number of operations replicated
    pub static ref REPLICATION_OPERATIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "secreton_replication_operations_total",
        "Total number of operations replicated",
        &["node_id", "operation_type", "status"]
    ).unwrap();

    /// Current replication status (0=disconnected, 1=lagging, 2=healthy)
    pub static ref REPLICATION_STATUS: IntGaugeVec = register_int_gauge_vec!(
        "secreton_replication_status",
        "Current replication status (0=disconnected, 1=lagging, 2=healthy)",
        &["node_id", "mode"]
    ).unwrap();

    /// Replication throughput (operations per second)
    pub static ref REPLICATION_THROUGHPUT: GaugeVec = register_gauge_vec!(
        "secreton_replication_throughput_ops",
        "Replication throughput in operations per second",
        &["node_id"]
    ).unwrap();

    /// Replication operation latency histogram
    pub static ref REPLICATION_OPERATION_LATENCY: HistogramVec = register_histogram_vec!(
        "secreton_replication_operation_latency_seconds",
        "Replication operation latency in seconds",
        &["node_id", "operation_type"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    /// Number of active secondary nodes
    pub static ref REPLICATION_SECONDARIES: IntGaugeVec = register_int_gauge_vec!(
        "secreton_replication_secondaries",
        "Number of active secondary nodes",
        &["status"]
    ).unwrap();

    /// Primary sequence number (WAL position)
    pub static ref REPLICATION_PRIMARY_SEQUENCE: IntGaugeVec = register_int_gauge_vec!(
        "secreton_replication_primary_sequence",
        "Primary node's current sequence number",
        &[]
    ).unwrap();

    /// Secondary sequence number (WAL position)
    pub static ref REPLICATION_SECONDARY_SEQUENCE: IntGaugeVec = register_int_gauge_vec!(
        "secreton_replication_secondary_sequence",
        "Secondary node's current sequence number",
        &["node_id"]
    ).unwrap();
}

/// Replication metrics collector
pub struct ReplicationMetrics {
    /// Node ID for this instance
    node_id: String,

    /// Replication mode
    mode: String,

    /// Last recorded lag in bytes
    lag_bytes: Arc<RwLock<u64>>,

    /// Last recorded lag in milliseconds
    lag_ms: Arc<RwLock<u64>>,

    /// Last recorded primary sequence
    primary_sequence: Arc<RwLock<u64>>,

    /// Last recorded secondary sequence
    secondary_sequence: Arc<RwLock<u64>>,
}

impl ReplicationMetrics {
    /// Create a new metrics collector
    pub fn new(node_id: String, mode: String) -> Self {
        Self {
            node_id,
            mode,
            lag_bytes: Arc::new(RwLock::new(0)),
            lag_ms: Arc::new(RwLock::new(0)),
            primary_sequence: Arc::new(RwLock::new(0)),
            secondary_sequence: Arc::new(RwLock::new(0)),
        }
    }

    /// Update replication lag metrics
    pub async fn update_lag(&self, lag_bytes: u64, lag_ms: u64) {
        *self.lag_bytes.write().await = lag_bytes;
        *self.lag_ms.write().await = lag_ms;

        REPLICATION_LAG_BYTES
            .with_label_values(&[&self.node_id, &self.mode])
            .set(lag_bytes as i64);

        REPLICATION_LAG_MS
            .with_label_values(&[&self.node_id, &self.mode])
            .set(lag_ms as f64);
    }

    /// Update WAL positions
    pub async fn update_wal_positions(&self, primary_seq: u64, secondary_seq: u64) {
        *self.primary_sequence.write().await = primary_seq;
        *self.secondary_sequence.write().await = secondary_seq;

        REPLICATION_PRIMARY_SEQUENCE
            .with_label_values(&[])
            .set(primary_seq as i64);

        REPLICATION_SECONDARY_SEQUENCE
            .with_label_values(&[&self.node_id])
            .set(secondary_seq as i64);

        // Calculate and update lag in bytes
        let lag_bytes = primary_seq.saturating_sub(secondary_seq);
        let current_lag_ms = *self.lag_ms.read().await;
        self.update_lag(lag_bytes, current_lag_ms).await;
    }

    /// Update replication status
    pub async fn update_status(&self, status: ReplicationStatusValue) {
        let status_value = match status {
            ReplicationStatusValue::Disconnected => 0,
            ReplicationStatusValue::Lagging => 1,
            ReplicationStatusValue::Healthy => 2,
        };

        REPLICATION_STATUS
            .with_label_values(&[&self.node_id, &self.mode])
            .set(status_value);
    }

    /// Record a replicated operation
    pub fn record_operation(&self, operation_type: &str, success: bool) {
        let status = if success { "success" } else { "failure" };

        REPLICATION_OPERATIONS_TOTAL
            .with_label_values(&[&self.node_id, operation_type, status])
            .inc();
    }

    /// Update replication throughput
    pub fn update_throughput(&self, ops_per_second: f64) {
        REPLICATION_THROUGHPUT
            .with_label_values(&[&self.node_id])
            .set(ops_per_second);
    }

    /// Record operation latency
    pub fn record_latency(&self, operation_type: &str, latency_seconds: f64) {
        REPLICATION_OPERATION_LATENCY
            .with_label_values(&[&self.node_id, operation_type])
            .observe(latency_seconds);
    }

    /// Update secondary node count
    pub fn update_secondary_count(&self, healthy: usize, lagging: usize, disconnected: usize) {
        REPLICATION_SECONDARIES
            .with_label_values(&["healthy"])
            .set(healthy as i64);

        REPLICATION_SECONDARIES
            .with_label_values(&["lagging"])
            .set(lagging as i64);

        REPLICATION_SECONDARIES
            .with_label_values(&["disconnected"])
            .set(disconnected as i64);
    }

    /// Get current lag in bytes
    pub async fn get_lag_bytes(&self) -> u64 {
        *self.lag_bytes.read().await
    }

    /// Get current lag in milliseconds
    pub async fn get_lag_ms(&self) -> u64 {
        *self.lag_ms.read().await
    }

    /// Get current primary sequence
    pub async fn get_primary_sequence(&self) -> u64 {
        *self.primary_sequence.read().await
    }

    /// Get current secondary sequence
    pub async fn get_secondary_sequence(&self) -> u64 {
        *self.secondary_sequence.read().await
    }

    /// Record a promotion event
    ///
    /// This increments a counter tracking secondary-to-primary promotions
    pub async fn record_promotion(&self) {
        // Record promotion as a special operation
        self.record_operation("promotion", true);

        // Reset lag metrics since we're now primary
        self.update_lag(0, 0).await;

        // Update status to healthy
        self.update_status(ReplicationStatusValue::Healthy).await;
    }
}

/// Replication status values for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationStatusValue {
    /// Node is disconnected
    Disconnected,

    /// Node is lagging behind threshold
    Lagging,

    /// Node is healthy and up-to-date
    Healthy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_update_lag() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.update_lag(1000, 50).await;

        assert_eq!(metrics.get_lag_bytes().await, 1000);
        assert_eq!(metrics.get_lag_ms().await, 50);
    }

    #[tokio::test]
    async fn test_metrics_update_wal_positions() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.update_wal_positions(1000, 950).await;

        assert_eq!(metrics.get_primary_sequence().await, 1000);
        assert_eq!(metrics.get_secondary_sequence().await, 950);
        assert_eq!(metrics.get_lag_bytes().await, 50);
    }

    #[tokio::test]
    async fn test_metrics_record_operation() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.record_operation("secret_write", true);
        metrics.record_operation("secret_write", false);

        // Metrics are recorded, but we can't easily verify counter values in tests
        // This test mainly ensures no panics occur
    }

    #[tokio::test]
    async fn test_metrics_update_status() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.update_status(ReplicationStatusValue::Healthy).await;
        metrics.update_status(ReplicationStatusValue::Lagging).await;
        metrics.update_status(ReplicationStatusValue::Disconnected).await;

        // Metrics are recorded, but we can't easily verify gauge values in tests
        // This test mainly ensures no panics occur
    }

    #[tokio::test]
    async fn test_metrics_throughput_and_latency() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.update_throughput(100.5);
        metrics.record_latency("secret_write", 0.025);

        // Metrics are recorded, but we can't easily verify values in tests
        // This test mainly ensures no panics occur
    }

    #[tokio::test]
    async fn test_metrics_secondary_count() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "performance".to_string());

        metrics.update_secondary_count(2, 1, 0);

        // Metrics are recorded, but we can't easily verify gauge values in tests
        // This test mainly ensures no panics occur
    }

    #[tokio::test]
    async fn test_metrics_record_promotion() {
        let metrics = ReplicationMetrics::new("test-node".to_string(), "dr".to_string());

        // Set some lag before promotion
        metrics.update_lag(1000, 50).await;
        assert_eq!(metrics.get_lag_bytes().await, 1000);
        assert_eq!(metrics.get_lag_ms().await, 50);

        // Record promotion
        metrics.record_promotion().await;

        // Lag should be reset to 0
        assert_eq!(metrics.get_lag_bytes().await, 0);
        assert_eq!(metrics.get_lag_ms().await, 0);
    }
}
