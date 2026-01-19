//! MFA Performance Monitoring Service
//!
//! This module provides comprehensive performance monitoring for MFA operations
//! including metrics collection, alerting, and dashboard integration.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MFA performance monitoring service
pub struct MfaPerformanceMonitor {
    metrics: Arc<RwLock<MfaMetrics>>,
    /// Alert thresholds configuration
    alert_config: MfaAlertConfig,
    /// Performance history for trend analysis
    history: Arc<RwLock<Vec<MfaPerformanceSnapshot>>>,
}

/// MFA performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
    /// Performance metrics storage
pub struct MfaMetrics {
    /// Setup operation metrics
    pub setup_metrics: OperationMetrics,
    /// Verification operation metrics
    pub verification_metrics: OperationMetrics,
    /// Status lookup metrics
    pub status_lookup_metrics: OperationMetrics,
    /// Cache operation metrics
    pub cache_metrics: CacheMetrics,
    /// Database operation metrics
    pub database_metrics: DatabaseOperationMetrics,
    /// Error metrics
    pub error_metrics: ErrorMetrics,
    /// Rate limiting metrics
    pub rate_limit_metrics: RateLimitMetrics,
}

/// Operation-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Total number of operations
    pub total_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Maximum response time recorded
    pub max_response_time_ms: f64,
    /// Minimum response time recorded
    pub min_response_time_ms: f64,
    /// Operations per second (current rate)
    pub operations_per_second: f64,
    response_times: Vec<f64>,
}

/// Cache-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
    /// Response time history for percentile calculation
pub struct CacheMetrics {
    /// Cache hit count
    pub cache_hits: u64,
    /// Cache miss count
    pub cache_misses: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Average cache response time
    pub avg_cache_response_time_ms: f64,
    /// Cache evictions
    pub cache_evictions: u64,
    /// Cache errors
    pub cache_errors: u64,
}

/// Database operation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseOperationMetrics {
    /// Query execution metrics by query type
    pub query_metrics: HashMap<String, OperationMetrics>,
    /// Connection pool metrics
    pub pool_metrics: PoolMetrics,
    /// Slow query count (>100ms)
    pub slow_queries: u64,
    /// Database connection errors
    pub connection_errors: u64,
}

/// Connection pool metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolMetrics {
    /// Current active connections
    pub active_connections: u32,
    /// Maximum connections allowed
    pub max_connections: u32,
    /// Average connection wait time
    pub avg_wait_time_ms: f64,
    /// Connection timeouts
    pub connection_timeouts: u64,
}

/// Error metrics by category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Invalid OTP code errors
    pub invalid_otp_errors: u64,
    /// Rate limit exceeded errors
    pub rate_limit_errors: u64,
    /// Database errors
    pub database_errors: u64,
    /// Cache errors
    pub cache_errors: u64,
    /// Secreton integration errors
    pub secreton_errors: u64,
    /// Other errors
    pub other_errors: u64,
}

/// Rate limiting metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitMetrics {
    /// Total rate limit checks
    pub total_checks: u64,
    /// Rate limit violations
    pub violations: u64,
    /// Average rate limit check time
    pub avg_check_time_ms: f64,
}

/// Performance snapshot for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaPerformanceSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// Metrics at this point in time
    pub metrics: MfaMetrics,
    /// System load information
    pub system_load: SystemLoadInfo,
}

/// System load information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLoadInfo {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Active database connections
    pub db_connections: u32,
    /// Cache memory usage in MB
    pub cache_memory_mb: f64,
}

/// Alert configuration for MFA performance monitoring
#[derive(Debug, Clone)]
pub struct MfaAlertConfig {
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms: f64,
    /// Minimum acceptable success rate (0.0 to 1.0)
    pub min_success_rate: f64,
    /// Maximum acceptable error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Minimum acceptable cache hit ratio
    pub min_cache_hit_ratio: f64,
    /// Maximum acceptable database query time
    pub max_db_query_time_ms: f64,
}

impl Default for MfaAlertConfig {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000.0, // 1 second
            min_success_rate: 0.95,       // 95%
            max_error_rate: 0.05,         // 5%
            min_cache_hit_ratio: 0.80,    // 80%
            max_db_query_time_ms: 100.0,  // 100ms
        }
    }
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert severity level
    pub severity: AlertSeverity,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert message
    pub message: String,
    /// Current metric value
    pub current_value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Timestamp when alert was triggered
    pub timestamp: DateTime<Utc>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    /// Warning alert
    Warning,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Informational alert
pub enum AlertType {
    HighResponseTime,
    /// Low success rate alert
    LowSuccessRate,
    /// High error rate alert
    HighErrorRate,
    /// Low cache hit ratio alert
    LowCacheHitRatio,
    /// Slow database query alert
    SlowDatabaseQuery,
    /// High system load alert
    HighSystemLoad,
}

impl MfaPerformanceMonitor {
    /// Create a new MFA performance monitor
    /// High response time alert
    pub fn new(alert_config: Option<MfaAlertConfig>) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MfaMetrics::default())),
            alert_config: alert_config.unwrap_or_default(),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record MFA setup operation metrics
    pub async fn record_setup_operation(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        self.update_operation_metrics(&mut metrics.setup_metrics, duration, success);

        if !success {
            metrics.error_metrics.other_errors += 1;
        }

        debug!(
            operation = "mfa_setup",
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded MFA setup operation"
        );
    }

    /// Record MFA verification operation metrics
    pub async fn record_verification_operation(
        &self,
        duration: Duration,
        success: bool,
        error_type: Option<&str>,
    ) {
        let mut metrics = self.metrics.write().await;
        self.update_operation_metrics(&mut metrics.verification_metrics, duration, success);

        if !success {
            match error_type {
                Some("invalid_otp") => metrics.error_metrics.invalid_otp_errors += 1,
                Some("rate_limit") => metrics.error_metrics.rate_limit_errors += 1,
                Some("database") => metrics.error_metrics.database_errors += 1,
                Some("cache") => metrics.error_metrics.cache_errors += 1,
                Some("secreton") => metrics.error_metrics.secreton_errors += 1,
                _ => metrics.error_metrics.other_errors += 1,
            }
        }

        debug!(
            operation = "mfa_verification",
            duration_ms = duration.as_millis(),
            success = success,
            error_type = error_type,
            "Recorded MFA verification operation"
        );

        // Check for alerts
        self.check_alerts(&metrics).await;
    }

    /// Record MFA status lookup operation metrics
    pub async fn record_status_lookup(&self, duration: Duration, success: bool, cache_hit: bool) {
        let mut metrics = self.metrics.write().await;
        self.update_operation_metrics(&mut metrics.status_lookup_metrics, duration, success);

        // Update cache metrics
        if cache_hit {
            metrics.cache_metrics.cache_hits += 1;
        } else {
            metrics.cache_metrics.cache_misses += 1;
        }

        let total_cache_ops = metrics.cache_metrics.cache_hits + metrics.cache_metrics.cache_misses;
        if total_cache_ops > 0 {
            metrics.cache_metrics.hit_ratio =
                metrics.cache_metrics.cache_hits as f64 / total_cache_ops as f64;
        }

        debug!(
            operation = "mfa_status_lookup",
            duration_ms = duration.as_millis(),
            success = success,
            cache_hit = cache_hit,
            "Recorded MFA status lookup operation"
        );
    }

    /// Record database query metrics
    pub async fn record_database_query(&self, query_type: &str, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        let query_metrics = metrics
            .database_metrics
            .query_metrics
            .entry(query_type.to_string())
            .or_insert_with(OperationMetrics::default);

        self.update_operation_metrics(query_metrics, duration, success);

        // Track slow queries
        if duration.as_millis() > 100 {
            metrics.database_metrics.slow_queries += 1;
            warn!(
                query_type = query_type,
                duration_ms = duration.as_millis(),
                "Slow MFA database query detected"
            );
        }

        debug!(
            query_type = query_type,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded database query metrics"
        );
    }

    /// Record cache operation metrics
    pub async fn record_cache_operation(&self, operation: &str, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        if success {
            // Update average cache response time
            let total_ops = metrics.cache_metrics.cache_hits + metrics.cache_metrics.cache_misses;
            let duration_ms = duration.as_millis() as f64;

            if total_ops == 0 {
                metrics.cache_metrics.avg_cache_response_time_ms = duration_ms;
            } else {
                let alpha = 0.1; // Exponential moving average factor
                metrics.cache_metrics.avg_cache_response_time_ms = alpha * duration_ms
                    + (1.0 - alpha) * metrics.cache_metrics.avg_cache_response_time_ms;
            }
        } else {
            metrics.cache_metrics.cache_errors += 1;
        }

        debug!(
            cache_operation = operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Recorded cache operation metrics"
        );
    }

    /// Record rate limiting check
    pub async fn record_rate_limit_check(&self, duration: Duration, violated: bool) {
        let mut metrics = self.metrics.write().await;

        metrics.rate_limit_metrics.total_checks += 1;
        if violated {
            metrics.rate_limit_metrics.violations += 1;
        }

        // Update average check time
        let duration_ms = duration.as_millis() as f64;
        if metrics.rate_limit_metrics.total_checks == 1 {
            metrics.rate_limit_metrics.avg_check_time_ms = duration_ms;
        } else {
            let alpha = 0.1;
            metrics.rate_limit_metrics.avg_check_time_ms =
                alpha * duration_ms + (1.0 - alpha) * metrics.rate_limit_metrics.avg_check_time_ms;
        }

        debug!(
            duration_ms = duration.as_millis(),
            violated = violated,
            "Recorded rate limit check"
        );
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> MfaMetrics {
        self.metrics.read().await.clone()
    }

    /// Get performance dashboard data
    pub async fn get_dashboard_data(&self) -> MfaDashboardData {
        let metrics = self.metrics.read().await;
        let history = self.history.read().await;

        MfaDashboardData {
            current_metrics: metrics.clone(),
            success_rates: SuccessRates {
                setup_success_rate: self.calculate_success_rate(&metrics.setup_metrics),
                verification_success_rate: self
                    .calculate_success_rate(&metrics.verification_metrics),
                status_lookup_success_rate: self
                    .calculate_success_rate(&metrics.status_lookup_metrics),
            },
            response_times: ResponseTimes {
                avg_setup_time_ms: metrics.setup_metrics.avg_response_time_ms,
                avg_verification_time_ms: metrics.verification_metrics.avg_response_time_ms,
                avg_status_lookup_time_ms: metrics.status_lookup_metrics.avg_response_time_ms,
            },
            error_breakdown: metrics.error_metrics.clone(),
            cache_performance: CachePerformance {
                hit_ratio: metrics.cache_metrics.hit_ratio,
                avg_response_time_ms: metrics.cache_metrics.avg_cache_response_time_ms,
                total_operations: metrics.cache_metrics.cache_hits
                    + metrics.cache_metrics.cache_misses,
            },
            recent_snapshots: history.iter().rev().take(24).cloned().collect(), // Last 24 snapshots
        }
    }

    /// Take a performance snapshot for historical analysis
    pub async fn take_snapshot(&self) -> Result<()> {
        let metrics = self.metrics.read().await.clone();

        // Get system load information (simplified implementation)
        let system_load = SystemLoadInfo {
            cpu_usage: 0.0,       // Would be implemented with system monitoring
            memory_usage: 0.0,    // Would be implemented with system monitoring
            db_connections: 0,    // Would be implemented with pool monitoring
            cache_memory_mb: 0.0, // Would be implemented with cache monitoring
        };

        let snapshot = MfaPerformanceSnapshot {
            timestamp: Utc::now(),
            metrics,
            system_load,
        };

        let mut history = self.history.write().await;
        history.push(snapshot);

        // Keep only last 1000 snapshots to prevent memory growth
        let current_len = history.len();
        if current_len > 1000 {
            history.drain(0..current_len - 1000);
        }

        info!("Performance snapshot taken");
        Ok(())
    }

    async fn check_alerts(&self, metrics: &MfaMetrics) {
        let mut alerts = Vec::new();

        // Check response time alerts
        if metrics.verification_metrics.avg_response_time_ms
            > self.alert_config.max_response_time_ms
        {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Warning,
                alert_type: AlertType::HighResponseTime,
                message: format!(
                    "MFA verification response time ({:.2}ms) exceeds threshold ({:.2}ms)",
                    metrics.verification_metrics.avg_response_time_ms,
                    self.alert_config.max_response_time_ms
                ),
                current_value: metrics.verification_metrics.avg_response_time_ms,
                threshold: self.alert_config.max_response_time_ms,
                timestamp: Utc::now(),
            });
        }

        // Check success rate alerts
        let verification_success_rate = self.calculate_success_rate(&metrics.verification_metrics);
        if verification_success_rate < self.alert_config.min_success_rate {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Critical,
                alert_type: AlertType::LowSuccessRate,
                message: format!(
                    "MFA verification success rate ({:.2}%) below threshold ({:.2}%)",
                    verification_success_rate * 100.0,
                    self.alert_config.min_success_rate * 100.0
                ),
                current_value: verification_success_rate,
                threshold: self.alert_config.min_success_rate,
                timestamp: Utc::now(),
            });
        }

        // Check cache hit ratio alerts
        if metrics.cache_metrics.hit_ratio < self.alert_config.min_cache_hit_ratio {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Warning,
                alert_type: AlertType::LowCacheHitRatio,
                message: format!(
                    "MFA cache hit ratio ({:.2}%) below threshold ({:.2}%)",
                    metrics.cache_metrics.hit_ratio * 100.0,
                    self.alert_config.min_cache_hit_ratio * 100.0
                ),
                current_value: metrics.cache_metrics.hit_ratio,
                threshold: self.alert_config.min_cache_hit_ratio,
                timestamp: Utc::now(),
            });
        }

        // Log alerts
        for alert in alerts {
            match alert.severity {
                AlertSeverity::Info => info!("MFA Performance Alert: {}", alert.message),
                AlertSeverity::Warning => warn!("MFA Performance Alert: {}", alert.message),
                AlertSeverity::Critical => error!("MFA Performance Alert: {}", alert.message),
            }
        }
    }

    /// Update operation metrics with new data point
    fn update_operation_metrics(
        &self,
        metrics: &mut OperationMetrics,
        duration: Duration,
        success: bool,
    ) {
        let duration_ms = duration.as_millis() as f64;

        metrics.total_operations += 1;
        if success {
            metrics.successful_operations += 1;
        } else {
            metrics.failed_operations += 1;
        }

        // Update response time statistics
        metrics.response_times.push(duration_ms);

        // Keep only last 1000 response times for percentile calculation
        if metrics.response_times.len() > 1000 {
            metrics
                .response_times
                .drain(0..metrics.response_times.len() - 1000);
        }

        // Update min/max
        if metrics.total_operations == 1 {
            metrics.min_response_time_ms = duration_ms;
            metrics.max_response_time_ms = duration_ms;
            metrics.avg_response_time_ms = duration_ms;
        } else {
            metrics.min_response_time_ms = metrics.min_response_time_ms.min(duration_ms);
            metrics.max_response_time_ms = metrics.max_response_time_ms.max(duration_ms);

            // Update average using exponential moving average
            let alpha = 0.1;
            metrics.avg_response_time_ms =
                alpha * duration_ms + (1.0 - alpha) * metrics.avg_response_time_ms;
        }

        // Calculate percentiles
        if !metrics.response_times.is_empty() {
            let mut sorted_times = metrics.response_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let len = sorted_times.len();
            metrics.p95_response_time_ms = sorted_times[(len * 95) / 100];
            metrics.p99_response_time_ms = sorted_times[(len * 99) / 100];
        }

        // Calculate operations per second (simplified)
        metrics.operations_per_second = metrics.total_operations as f64 / 60.0; // Rough estimate
    }

    /// Calculate success rate for operation metrics
    fn calculate_success_rate(&self, metrics: &OperationMetrics) -> f64 {
        if metrics.total_operations == 0 {
            return 1.0;
        }
        metrics.successful_operations as f64 / metrics.total_operations as f64
    }
}

/// Dashboard data structure for MFA performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Check for performance alerts
pub struct MfaDashboardData {
    /// Current performance metrics
    pub current_metrics: MfaMetrics,
    /// Success rates by operation type
    pub success_rates: SuccessRates,
    /// Response times by operation type
    pub response_times: ResponseTimes,
    /// Error breakdown
    pub error_breakdown: ErrorMetrics,
    /// Cache performance summary
    pub cache_performance: CachePerformance,
    /// Recent performance snapshots
    pub recent_snapshots: Vec<MfaPerformanceSnapshot>,
}

/// Success rates by operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessRates {
    /// MFA setup success rate
    pub setup_success_rate: f64,
    /// MFA verification success rate
    pub verification_success_rate: f64,
    /// Status lookup success rate
    pub status_lookup_success_rate: f64,
}

/// Response times by operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimes {
    /// Average setup time in milliseconds
    pub avg_setup_time_ms: f64,
    /// Average verification time in milliseconds
    pub avg_verification_time_ms: f64,
    /// Average status lookup time in milliseconds
    pub avg_status_lookup_time_ms: f64,
}

/// Cache performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformance {
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Average cache response time
    pub avg_response_time_ms: f64,
    /// Total cache operations
    pub total_operations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_performance_monitor_basic_operations() {
        let monitor = MfaPerformanceMonitor::new(None);

        // Record some operations
        monitor
            .record_setup_operation(Duration::from_millis(100), true)
            .await;
        monitor
            .record_verification_operation(Duration::from_millis(50), true, None)
            .await;
        monitor
            .record_status_lookup(Duration::from_millis(25), true, true)
            .await;

        let metrics = monitor.get_metrics().await;

        assert_eq!(metrics.setup_metrics.total_operations, 1);
        assert_eq!(metrics.verification_metrics.total_operations, 1);
        assert_eq!(metrics.status_lookup_metrics.total_operations, 1);
        assert_eq!(metrics.cache_metrics.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_success_rate_calculation() {
        let monitor = MfaPerformanceMonitor::new(None);

        // Record mixed success/failure operations
        monitor
            .record_verification_operation(Duration::from_millis(50), true, None)
            .await;
        monitor
            .record_verification_operation(Duration::from_millis(75), true, None)
            .await;
        monitor
            .record_verification_operation(Duration::from_millis(100), false, Some("invalid_otp"))
            .await;

        let metrics = monitor.get_metrics().await;
        let success_rate = monitor.calculate_success_rate(&metrics.verification_metrics);

        assert!((success_rate - 0.6667).abs() < 0.001); // 2/3 ≈ 0.6667
        assert_eq!(metrics.error_metrics.invalid_otp_errors, 1);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let monitor = MfaPerformanceMonitor::new(None);

        // Record cache hits and misses
        monitor
            .record_status_lookup(Duration::from_millis(10), true, true)
            .await; // hit
        monitor
            .record_status_lookup(Duration::from_millis(50), true, false)
            .await; // miss
        monitor
            .record_status_lookup(Duration::from_millis(15), true, true)
            .await; // hit

        let metrics = monitor.get_metrics().await;

        assert_eq!(metrics.cache_metrics.cache_hits, 2);
        assert_eq!(metrics.cache_metrics.cache_misses, 1);
        assert!((metrics.cache_metrics.hit_ratio - 0.6667).abs() < 0.001); // 2/3
    }
}
