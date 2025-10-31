//! Cache metrics collection for monitoring and observability
//!
//! This module provides comprehensive metrics tracking for cache operations including:
//! - Hit/miss ratio tracking
//! - Operation latency measurements
//! - Cache size and eviction metrics
//! - Prometheus-compatible metric exposure

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache metrics collector for tracking cache performance
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    /// Total number of cache hits
    hits: Arc<AtomicU64>,
    /// Total number of cache misses
    misses: Arc<AtomicU64>,
    /// Total number of cache get operations
    get_operations: Arc<AtomicU64>,
    /// Total number of cache set operations
    set_operations: Arc<AtomicU64>,
    /// Total number of cache delete operations
    delete_operations: Arc<AtomicU64>,
    /// Total number of cache errors
    errors: Arc<AtomicU64>,
    /// Total latency for get operations (in microseconds)
    get_latency_total: Arc<AtomicU64>,
    /// Total latency for set operations (in microseconds)
    set_latency_total: Arc<AtomicU64>,
    /// Total latency for delete operations (in microseconds)
    delete_latency_total: Arc<AtomicU64>,
    /// Number of evictions (if tracked)
    evictions: Arc<AtomicU64>,
    /// Current estimated cache size (number of keys)
    cache_size: Arc<AtomicU64>,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheMetrics {
    /// Create a new cache metrics collector
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            get_operations: Arc::new(AtomicU64::new(0)),
            set_operations: Arc::new(AtomicU64::new(0)),
            delete_operations: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            get_latency_total: Arc::new(AtomicU64::new(0)),
            set_latency_total: Arc::new(AtomicU64::new(0)),
            delete_latency_total: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            cache_size: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a get operation with latency
    pub fn record_get(&self, duration: Duration) {
        self.get_operations.fetch_add(1, Ordering::Relaxed);
        self.get_latency_total
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    /// Record a set operation with latency
    pub fn record_set(&self, duration: Duration) {
        self.set_operations.fetch_add(1, Ordering::Relaxed);
        self.set_latency_total
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    /// Record a delete operation with latency
    pub fn record_delete(&self, duration: Duration) {
        self.delete_operations.fetch_add(1, Ordering::Relaxed);
        self.delete_latency_total
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    /// Record a cache error
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Update cache size estimate
    pub fn update_cache_size(&self, size: u64) {
        self.cache_size.store(size, Ordering::Relaxed);
    }

    /// Increment cache size
    pub fn increment_cache_size(&self) {
        self.cache_size.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement cache size
    pub fn decrement_cache_size(&self) {
        self.cache_size.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get total number of hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get total number of misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get total number of get operations
    pub fn get_operations(&self) -> u64 {
        self.get_operations.load(Ordering::Relaxed)
    }

    /// Get total number of set operations
    pub fn set_operations(&self) -> u64 {
        self.set_operations.load(Ordering::Relaxed)
    }

    /// Get total number of delete operations
    pub fn delete_operations(&self) -> u64 {
        self.delete_operations.load(Ordering::Relaxed)
    }

    /// Get total number of errors
    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    /// Get total number of evictions
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get current cache size
    pub fn cache_size(&self) -> u64 {
        self.cache_size.load(Ordering::Relaxed)
    }

    /// Calculate hit ratio (0.0 to 1.0)
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits();
        let total = hits + self.misses();
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Calculate miss ratio (0.0 to 1.0)
    pub fn miss_ratio(&self) -> f64 {
        1.0 - self.hit_ratio()
    }

    /// Calculate average get latency in microseconds
    pub fn avg_get_latency_micros(&self) -> f64 {
        let total_latency = self.get_latency_total.load(Ordering::Relaxed);
        let operations = self.get_operations();
        if operations == 0 {
            0.0
        } else {
            total_latency as f64 / operations as f64
        }
    }

    /// Calculate average set latency in microseconds
    pub fn avg_set_latency_micros(&self) -> f64 {
        let total_latency = self.set_latency_total.load(Ordering::Relaxed);
        let operations = self.set_operations();
        if operations == 0 {
            0.0
        } else {
            total_latency as f64 / operations as f64
        }
    }

    /// Calculate average delete latency in microseconds
    pub fn avg_delete_latency_micros(&self) -> f64 {
        let total_latency = self.delete_latency_total.load(Ordering::Relaxed);
        let operations = self.delete_operations();
        if operations == 0 {
            0.0
        } else {
            total_latency as f64 / operations as f64
        }
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            hit_ratio: self.hit_ratio(),
            miss_ratio: self.miss_ratio(),
            get_operations: self.get_operations(),
            set_operations: self.set_operations(),
            delete_operations: self.delete_operations(),
            errors: self.errors(),
            evictions: self.evictions(),
            cache_size: self.cache_size(),
            avg_get_latency_micros: self.avg_get_latency_micros(),
            avg_set_latency_micros: self.avg_set_latency_micros(),
            avg_delete_latency_micros: self.avg_delete_latency_micros(),
        }
    }

    /// Reset all metrics (useful for testing)
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.get_operations.store(0, Ordering::Relaxed);
        self.set_operations.store(0, Ordering::Relaxed);
        self.delete_operations.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
        self.get_latency_total.store(0, Ordering::Relaxed);
        self.set_latency_total.store(0, Ordering::Relaxed);
        self.delete_latency_total.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.cache_size.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of cache metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetricsSnapshot {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Miss ratio (0.0 to 1.0)
    pub miss_ratio: f64,
    /// Total get operations
    pub get_operations: u64,
    /// Total set operations
    pub set_operations: u64,
    /// Total delete operations
    pub delete_operations: u64,
    /// Total errors
    pub errors: u64,
    /// Total evictions
    pub evictions: u64,
    /// Current cache size
    pub cache_size: u64,
    /// Average get latency in microseconds
    pub avg_get_latency_micros: f64,
    /// Average set latency in microseconds
    pub avg_set_latency_micros: f64,
    /// Average delete latency in microseconds
    pub avg_delete_latency_micros: f64,
}

impl CacheMetricsSnapshot {
    /// Format metrics for Prometheus exposition format
    pub fn to_prometheus_format(&self, prefix: &str) -> String {
        let mut output = String::new();

        // Counter metrics
        output.push_str(&format!(
            "# HELP {}_hits_total Total number of cache hits\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_hits_total counter\n", prefix));
        output.push_str(&format!("{}_hits_total {}\n\n", prefix, self.hits));

        output.push_str(&format!(
            "# HELP {}_misses_total Total number of cache misses\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_misses_total counter\n", prefix));
        output.push_str(&format!("{}_misses_total {}\n\n", prefix, self.misses));

        output.push_str(&format!(
            "# HELP {}_operations_total Total number of cache operations by type\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_operations_total counter\n", prefix));
        output.push_str(&format!(
            "{}{{operation=\"get\"}} {}\n",
            format!("{}_operations_total", prefix),
            self.get_operations
        ));
        output.push_str(&format!(
            "{}{{operation=\"set\"}} {}\n",
            format!("{}_operations_total", prefix),
            self.set_operations
        ));
        output.push_str(&format!(
            "{}{{operation=\"delete\"}} {}\n\n",
            format!("{}_operations_total", prefix),
            self.delete_operations
        ));

        output.push_str(&format!(
            "# HELP {}_errors_total Total number of cache errors\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_errors_total counter\n", prefix));
        output.push_str(&format!("{}_errors_total {}\n\n", prefix, self.errors));

        output.push_str(&format!(
            "# HELP {}_evictions_total Total number of cache evictions\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_evictions_total counter\n", prefix));
        output.push_str(&format!(
            "{}_evictions_total {}\n\n",
            prefix, self.evictions
        ));

        // Gauge metrics
        output.push_str(&format!(
            "# HELP {}_hit_ratio Current cache hit ratio\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_hit_ratio gauge\n", prefix));
        output.push_str(&format!("{}_hit_ratio {:.4}\n\n", prefix, self.hit_ratio));

        output.push_str(&format!(
            "# HELP {}_size Current number of items in cache\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_size gauge\n", prefix));
        output.push_str(&format!("{}_size {}\n\n", prefix, self.cache_size));

        // Latency metrics (in milliseconds for better readability)
        output.push_str(&format!(
            "# HELP {}_latency_milliseconds Average operation latency in milliseconds\n",
            prefix
        ));
        output.push_str(&format!("# TYPE {}_latency_milliseconds gauge\n", prefix));
        output.push_str(&format!(
            "{}{{operation=\"get\"}} {:.3}\n",
            format!("{}_latency_milliseconds", prefix),
            self.avg_get_latency_micros / 1000.0
        ));
        output.push_str(&format!(
            "{}{{operation=\"set\"}} {:.3}\n",
            format!("{}_latency_milliseconds", prefix),
            self.avg_set_latency_micros / 1000.0
        ));
        output.push_str(&format!(
            "{}{{operation=\"delete\"}} {:.3}\n\n",
            format!("{}_latency_milliseconds", prefix),
            self.avg_delete_latency_micros / 1000.0
        ));

        output
    }
}

/// Helper struct to measure operation duration
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    /// Start a new operation timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_metrics_new() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
        assert_eq!(metrics.hit_ratio(), 0.0);
    }

    #[test]
    fn test_record_hit_miss() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();

        assert_eq!(metrics.hits(), 2);
        assert_eq!(metrics.misses(), 1);
        assert!((metrics.hit_ratio() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_record_operations() {
        let metrics = CacheMetrics::new();

        metrics.record_get(Duration::from_micros(100));
        metrics.record_set(Duration::from_micros(200));
        metrics.record_delete(Duration::from_micros(150));

        assert_eq!(metrics.get_operations(), 1);
        assert_eq!(metrics.set_operations(), 1);
        assert_eq!(metrics.delete_operations(), 1);
        assert_eq!(metrics.avg_get_latency_micros(), 100.0);
        assert_eq!(metrics.avg_set_latency_micros(), 200.0);
        assert_eq!(metrics.avg_delete_latency_micros(), 150.0);
    }

    #[test]
    fn test_cache_size_tracking() {
        let metrics = CacheMetrics::new();

        metrics.increment_cache_size();
        metrics.increment_cache_size();
        assert_eq!(metrics.cache_size(), 2);

        metrics.decrement_cache_size();
        assert_eq!(metrics.cache_size(), 1);

        metrics.update_cache_size(100);
        assert_eq!(metrics.cache_size(), 100);
    }

    #[test]
    fn test_error_tracking() {
        let metrics = CacheMetrics::new();

        metrics.record_error();
        metrics.record_error();

        assert_eq!(metrics.errors(), 2);
    }

    #[test]
    fn test_eviction_tracking() {
        let metrics = CacheMetrics::new();

        metrics.record_eviction();
        assert_eq!(metrics.evictions(), 1);
    }

    #[test]
    fn test_snapshot() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_miss();
        metrics.record_get(Duration::from_micros(100));
        metrics.increment_cache_size();

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.hits, 1);
        assert_eq!(snapshot.misses, 1);
        assert_eq!(snapshot.hit_ratio, 0.5);
        assert_eq!(snapshot.get_operations, 1);
        assert_eq!(snapshot.cache_size, 1);
    }

    #[test]
    fn test_reset() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_miss();
        metrics.increment_cache_size();

        metrics.reset();

        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
        assert_eq!(metrics.cache_size(), 0);
    }

    #[test]
    fn test_prometheus_format() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();
        metrics.record_get(Duration::from_micros(1000));
        metrics.increment_cache_size();

        let snapshot = metrics.snapshot();
        let prometheus_output = snapshot.to_prometheus_format("redis_cache");

        assert!(prometheus_output.contains("redis_cache_hits_total 2"));
        assert!(prometheus_output.contains("redis_cache_misses_total 1"));
        assert!(prometheus_output.contains("redis_cache_hit_ratio"));
        assert!(prometheus_output.contains("redis_cache_size 1"));
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start();
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(50)); // Allow some margin
    }

    #[test]
    fn test_concurrent_metrics() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(CacheMetrics::new());
        let mut handles = vec![];

        // Spawn multiple threads to record metrics concurrently
        for _ in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.record_hit();
                    metrics_clone.record_miss();
                    metrics_clone.increment_cache_size();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify that all operations were recorded
        assert_eq!(metrics.hits(), 1000);
        assert_eq!(metrics.misses(), 1000);
        assert_eq!(metrics.cache_size(), 1000);
    }
}
