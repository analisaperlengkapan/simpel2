//! Performance metrics for secreton operations
//! Tracks operation latency, throughput, and error rates

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics for vault operations
#[derive(Debug, Clone)]
pub struct VaultMetrics {
    // Operation counters
    secret_creates: Arc<AtomicU64>,
    secret_reads: Arc<AtomicU64>,
    secret_updates: Arc<AtomicU64>,
    secret_deletes: Arc<AtomicU64>,
    secret_rotations: Arc<AtomicU64>,

    // Error counters
    auth_failures: Arc<AtomicU64>,
    operation_failures: Arc<AtomicU64>,

    // Latency tracking (nanoseconds)
    total_latency_ns: Arc<AtomicU64>,
    operation_count: Arc<AtomicU64>,

    // Health
    health_checks: Arc<AtomicU64>,
    health_failures: Arc<AtomicU64>,
}

impl Default for VaultMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            secret_creates: Arc::new(AtomicU64::new(0)),
            secret_reads: Arc::new(AtomicU64::new(0)),
            secret_updates: Arc::new(AtomicU64::new(0)),
            secret_deletes: Arc::new(AtomicU64::new(0)),
            secret_rotations: Arc::new(AtomicU64::new(0)),
            auth_failures: Arc::new(AtomicU64::new(0)),
            operation_failures: Arc::new(AtomicU64::new(0)),
            total_latency_ns: Arc::new(AtomicU64::new(0)),
            operation_count: Arc::new(AtomicU64::new(0)),
            health_checks: Arc::new(AtomicU64::new(0)),
            health_failures: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record secret creation
    pub fn record_create(&self, duration: Duration) {
        self.secret_creates.fetch_add(1, Ordering::Relaxed);
        self.record_latency(duration);
    }

    /// Record secret read
    pub fn record_read(&self, duration: Duration) {
        self.secret_reads.fetch_add(1, Ordering::Relaxed);
        self.record_latency(duration);
    }

    /// Record secret update
    pub fn record_update(&self, duration: Duration) {
        self.secret_updates.fetch_add(1, Ordering::Relaxed);
        self.record_latency(duration);
    }

    /// Record secret deletion
    pub fn record_delete(&self, duration: Duration) {
        self.secret_deletes.fetch_add(1, Ordering::Relaxed);
        self.record_latency(duration);
    }

    /// Record secret rotation
    pub fn record_rotation(&self, duration: Duration) {
        self.secret_rotations.fetch_add(1, Ordering::Relaxed);
        self.record_latency(duration);
    }

    /// Record authentication failure
    pub fn record_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Record operation failure
    pub fn record_operation_failure(&self) {
        self.operation_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Record health check
    pub fn record_health_check(&self, success: bool) {
        self.health_checks.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.health_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record operation latency
    fn record_latency(&self, duration: Duration) {
        self.total_latency_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);

        let avg_latency_ms = if operation_count > 0 {
            (total_latency_ns / operation_count) as f64 / 1_000_000.0
        } else {
            0.0
        };

        MetricsSnapshot {
            secret_creates: self.secret_creates.load(Ordering::Relaxed),
            secret_reads: self.secret_reads.load(Ordering::Relaxed),
            secret_updates: self.secret_updates.load(Ordering::Relaxed),
            secret_deletes: self.secret_deletes.load(Ordering::Relaxed),
            secret_rotations: self.secret_rotations.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            operation_failures: self.operation_failures.load(Ordering::Relaxed),
            health_checks: self.health_checks.load(Ordering::Relaxed),
            health_failures: self.health_failures.load(Ordering::Relaxed),
            average_latency_ms: avg_latency_ms,
            total_operations: operation_count,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.secret_creates.store(0, Ordering::Relaxed);
        self.secret_reads.store(0, Ordering::Relaxed);
        self.secret_updates.store(0, Ordering::Relaxed);
        self.secret_deletes.store(0, Ordering::Relaxed);
        self.secret_rotations.store(0, Ordering::Relaxed);
        self.auth_failures.store(0, Ordering::Relaxed);
        self.operation_failures.store(0, Ordering::Relaxed);
        self.total_latency_ns.store(0, Ordering::Relaxed);
        self.operation_count.store(0, Ordering::Relaxed);
        self.health_checks.store(0, Ordering::Relaxed);
        self.health_failures.store(0, Ordering::Relaxed);
    }

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        format!(
            r#"# HELP vault_secret_creates_total Total number of secret creations
# TYPE vault_secret_creates_total counter
vault_secret_creates_total {}

# HELP vault_secret_reads_total Total number of secret reads
# TYPE vault_secret_reads_total counter
vault_secret_reads_total {}

# HELP vault_secret_updates_total Total number of secret updates
# TYPE vault_secret_updates_total counter
vault_secret_updates_total {}

# HELP vault_secret_deletes_total Total number of secret deletions
# TYPE vault_secret_deletes_total counter
vault_secret_deletes_total {}

# HELP vault_secret_rotations_total Total number of secret rotations
# TYPE vault_secret_rotations_total counter
vault_secret_rotations_total {}

# HELP vault_auth_failures_total Total number of authentication failures
# TYPE vault_auth_failures_total counter
vault_auth_failures_total {}

# HELP vault_operation_failures_total Total number of operation failures
# TYPE vault_operation_failures_total counter
vault_operation_failures_total {}

# HELP vault_health_checks_total Total number of health checks
# TYPE vault_health_checks_total counter
vault_health_checks_total {}

# HELP vault_health_failures_total Total number of health check failures
# TYPE vault_health_failures_total counter
vault_health_failures_total {}

# HELP vault_operation_latency_ms Average operation latency in milliseconds
# TYPE vault_operation_latency_ms gauge
vault_operation_latency_ms {}

# HELP vault_total_operations Total number of operations
# TYPE vault_total_operations counter
vault_total_operations {}
"#,
            snapshot.secret_creates,
            snapshot.secret_reads,
            snapshot.secret_updates,
            snapshot.secret_deletes,
            snapshot.secret_rotations,
            snapshot.auth_failures,
            snapshot.operation_failures,
            snapshot.health_checks,
            snapshot.health_failures,
            snapshot.average_latency_ms,
            snapshot.total_operations,
        )
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub secret_creates: u64,
    pub secret_reads: u64,
    pub secret_updates: u64,
    pub secret_deletes: u64,
    pub secret_rotations: u64,
    pub auth_failures: u64,
    pub operation_failures: u64,
    pub health_checks: u64,
    pub health_failures: u64,
    pub average_latency_ms: f64,
    pub total_operations: u64,
}

impl MetricsSnapshot {
    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        (self.operation_failures as f64 / self.total_operations as f64) * 100.0
    }

    /// Calculate throughput (operations per second)
    pub fn throughput(&self, elapsed_seconds: f64) -> f64 {
        if elapsed_seconds <= 0.0 {
            return 0.0;
        }
        self.total_operations as f64 / elapsed_seconds
    }
}

/// Helper to measure operation duration
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// gRPC TLS metrics
#[derive(Debug, Clone)]
pub struct GrpcTlsMetrics {
    // Connection counters
    total_connections: Arc<AtomicU64>,
    successful_handshakes: Arc<AtomicU64>,
    failed_handshakes: Arc<AtomicU64>,

    // Client certificate verification
    client_cert_verifications: Arc<AtomicU64>,
    failed_client_cert_verifications: Arc<AtomicU64>,
}

impl Default for GrpcTlsMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl GrpcTlsMetrics {
    /// Create new gRPC TLS metrics tracker
    pub fn new() -> Self {
        Self {
            total_connections: Arc::new(AtomicU64::new(0)),
            successful_handshakes: Arc::new(AtomicU64::new(0)),
            failed_handshakes: Arc::new(AtomicU64::new(0)),
            client_cert_verifications: Arc::new(AtomicU64::new(0)),
            failed_client_cert_verifications: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record TLS connection attempt
    pub fn record_connection(&self, success: bool) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_handshakes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_handshakes.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record client certificate verification
    pub fn record_client_cert_verification(&self, success: bool) {
        self.client_cert_verifications
            .fetch_add(1, Ordering::Relaxed);
        if !success {
            self.failed_client_cert_verifications
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> GrpcTlsMetricsSnapshot {
        GrpcTlsMetricsSnapshot {
            total_connections: self.total_connections.load(Ordering::Relaxed),
            successful_handshakes: self.successful_handshakes.load(Ordering::Relaxed),
            failed_handshakes: self.failed_handshakes.load(Ordering::Relaxed),
            client_cert_verifications: self.client_cert_verifications.load(Ordering::Relaxed),
            failed_client_cert_verifications: self
                .failed_client_cert_verifications
                .load(Ordering::Relaxed),
        }
    }

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        format!(
            r#"# HELP grpc_tls_connections_total Total number of gRPC TLS connections
# TYPE grpc_tls_connections_total counter
grpc_tls_connections_total {}

# HELP grpc_tls_handshakes_successful Successful TLS handshakes
# TYPE grpc_tls_handshakes_successful counter
grpc_tls_handshakes_successful {}

# HELP grpc_tls_handshakes_failed Failed TLS handshakes
# TYPE grpc_tls_handshakes_failed counter
grpc_tls_handshakes_failed {}

# HELP grpc_tls_client_cert_verifications_total Total client certificate verifications
# TYPE grpc_tls_client_cert_verifications_total counter
grpc_tls_client_cert_verifications_total {}

# HELP grpc_tls_client_cert_verifications_failed Failed client certificate verifications
# TYPE grpc_tls_client_cert_verifications_failed counter
grpc_tls_client_cert_verifications_failed {}

# HELP grpc_tls_success_rate TLS handshake success rate percentage
# TYPE grpc_tls_success_rate gauge
grpc_tls_success_rate {}
"#,
            snapshot.total_connections,
            snapshot.successful_handshakes,
            snapshot.failed_handshakes,
            snapshot.client_cert_verifications,
            snapshot.failed_client_cert_verifications,
            snapshot.success_rate(),
        )
    }
}

/// gRPC TLS metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcTlsMetricsSnapshot {
    pub total_connections: u64,
    pub successful_handshakes: u64,
    pub failed_handshakes: u64,
    pub client_cert_verifications: u64,
    pub failed_client_cert_verifications: u64,
}

impl GrpcTlsMetricsSnapshot {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.successful_handshakes as f64 / self.total_connections as f64) * 100.0
        }
    }

    /// Calculate client cert verification rate
    pub fn client_cert_success_rate(&self) -> f64 {
        if self.client_cert_verifications == 0 {
            0.0
        } else {
            let successful = self.client_cert_verifications - self.failed_client_cert_verifications;
            (successful as f64 / self.client_cert_verifications as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = VaultMetrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.total_operations, 0);
        assert_eq!(snapshot.secret_creates, 0);
    }

    #[test]
    fn test_record_operations() {
        let metrics = VaultMetrics::new();

        metrics.record_create(Duration::from_millis(10));
        metrics.record_read(Duration::from_millis(5));
        metrics.record_update(Duration::from_millis(15));

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.secret_creates, 1);
        assert_eq!(snapshot.secret_reads, 1);
        assert_eq!(snapshot.secret_updates, 1);
        assert_eq!(snapshot.total_operations, 3);
        assert!(snapshot.average_latency_ms > 0.0);
    }

    #[test]
    fn test_record_failures() {
        let metrics = VaultMetrics::new();

        metrics.record_auth_failure();
        metrics.record_operation_failure();

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.auth_failures, 1);
        assert_eq!(snapshot.operation_failures, 1);
    }

    #[test]
    fn test_health_check_tracking() {
        let metrics = VaultMetrics::new();

        metrics.record_health_check(true);
        metrics.record_health_check(true);
        metrics.record_health_check(false);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.health_checks, 3);
        assert_eq!(snapshot.health_failures, 1);
    }

    #[test]
    fn test_average_latency() {
        let metrics = VaultMetrics::new();

        metrics.record_create(Duration::from_millis(10));
        metrics.record_read(Duration::from_millis(20));

        let snapshot = metrics.snapshot();

        // Average should be around 15ms
        assert!(snapshot.average_latency_ms >= 14.0);
        assert!(snapshot.average_latency_ms <= 16.0);
    }

    #[test]
    fn test_reset_metrics() {
        let metrics = VaultMetrics::new();

        metrics.record_create(Duration::from_millis(10));
        metrics.record_read(Duration::from_millis(10));

        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_operations, 0);
        assert_eq!(snapshot.secret_creates, 0);
    }

    #[test]
    fn test_error_rate_calculation() {
        let snapshot = MetricsSnapshot {
            secret_creates: 10,
            secret_reads: 80,
            secret_updates: 5,
            secret_deletes: 5,
            secret_rotations: 0,
            auth_failures: 0,
            operation_failures: 10,
            health_checks: 0,
            health_failures: 0,
            average_latency_ms: 10.0,
            total_operations: 100,
        };

        assert_eq!(snapshot.error_rate(), 10.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let snapshot = MetricsSnapshot {
            secret_creates: 10,
            secret_reads: 80,
            secret_updates: 5,
            secret_deletes: 5,
            secret_rotations: 0,
            auth_failures: 0,
            operation_failures: 0,
            health_checks: 0,
            health_failures: 0,
            average_latency_ms: 10.0,
            total_operations: 100,
        };

        // 100 operations in 10 seconds = 10 ops/sec
        assert_eq!(snapshot.throughput(10.0), 10.0);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = VaultMetrics::new();

        metrics.record_create(Duration::from_millis(10));
        metrics.record_read(Duration::from_millis(5));

        let prometheus = metrics.to_prometheus();

        assert!(prometheus.contains("vault_secret_creates_total 1"));
        assert!(prometheus.contains("vault_secret_reads_total 1"));
        assert!(prometheus.contains("vault_total_operations 2"));
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new();

        thread::sleep(Duration::from_millis(10));

        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }
}
