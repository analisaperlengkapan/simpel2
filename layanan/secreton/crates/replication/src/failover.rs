//! Automatic failover detection and promotion
//!
//! This module implements health monitoring for the primary node and automatic
//! promotion of secondary to primary when the primary fails.
//!
//! **Validates: Requirements 2.2.6**

use crate::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Health status of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Node is healthy and responding
    Healthy,
    /// Node is degraded but still operational
    Degraded,
    /// Node is unhealthy and not responding
    Unhealthy,
    /// Node status is unknown (no recent heartbeat)
    Unknown,
}

/// Heartbeat from primary node
#[derive(Debug, Clone)]
pub struct Heartbeat {
    /// Sequence number at time of heartbeat
    pub sequence: u64,
    /// Timestamp of heartbeat
    pub timestamp: DateTime<Utc>,
    /// Health status reported by primary
    pub status: HealthStatus,
}

/// Failover detector monitors primary health and triggers promotion
pub struct FailoverDetector {
    /// Last heartbeat received from primary
    last_heartbeat: Arc<RwLock<Option<Heartbeat>>>,
    /// Heartbeat timeout threshold
    heartbeat_timeout: Duration,
    /// Number of consecutive failures before triggering failover
    failure_threshold: u32,
    /// Current consecutive failure count
    failure_count: Arc<RwLock<u32>>,
    /// Whether failover has been triggered
    failover_triggered: Arc<RwLock<bool>>,
    /// Callback to invoke when failover is triggered
    on_failover: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl FailoverDetector {
    /// Create a new failover detector
    ///
    /// # Arguments
    ///
    /// * `heartbeat_timeout` - Duration after which a missing heartbeat is considered a failure
    /// * `failure_threshold` - Number of consecutive failures before triggering failover
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use secreton_replication::failover::FailoverDetector;
    ///
    /// let detector = FailoverDetector::new(
    ///     Duration::from_millis(100),  // Short timeout for tests
    ///     3,  // Trigger after 3 consecutive failures
    /// );
    /// ```
    pub fn new(heartbeat_timeout: Duration, failure_threshold: u32) -> Self {
        Self {
            last_heartbeat: Arc::new(RwLock::new(None)),
            heartbeat_timeout,
            failure_threshold,
            failure_count: Arc::new(RwLock::new(0)),
            failover_triggered: Arc::new(RwLock::new(false)),
            on_failover: None,
        }
    }

    /// Set the failover callback
    ///
    /// This callback will be invoked when failover is triggered
    pub fn set_failover_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_failover = Some(Arc::new(callback));
    }

    /// Record a heartbeat from the primary
    ///
    /// Resets the failure count if the heartbeat is healthy
    pub async fn record_heartbeat(&self, heartbeat: Heartbeat) {
        debug!(
            "Received heartbeat: seq={}, status={:?}",
            heartbeat.sequence, heartbeat.status
        );

        *self.last_heartbeat.write().await = Some(heartbeat.clone());

        // Reset failure count if heartbeat is healthy
        if heartbeat.status == HealthStatus::Healthy {
            let prev_count = *self.failure_count.read().await;
            if prev_count > 0 {
                info!(
                    "Primary recovered, resetting failure count from {}",
                    prev_count
                );
                *self.failure_count.write().await = 0;
            }
        }
    }

    /// Check primary health and detect failures
    ///
    /// Returns true if failover should be triggered
    pub async fn check_health(&self) -> Result<bool> {
        let last_heartbeat = self.last_heartbeat.read().await.clone();

        let is_failed = match last_heartbeat {
            None => {
                // No heartbeat received yet
                warn!("No heartbeat received from primary");
                true
            }
            Some(heartbeat) => {
                let elapsed = Utc::now() - heartbeat.timestamp;
                let elapsed_duration = elapsed.to_std().unwrap_or(Duration::from_secs(0));

                if elapsed_duration > self.heartbeat_timeout {
                    warn!(
                        "Heartbeat timeout: last heartbeat was {:?} ago (threshold: {:?})",
                        elapsed_duration, self.heartbeat_timeout
                    );
                    true
                } else if heartbeat.status == HealthStatus::Unhealthy {
                    warn!("Primary reported unhealthy status");
                    true
                } else {
                    // Primary is healthy
                    false
                }
            }
        };

        if is_failed {
            let mut failure_count = self.failure_count.write().await;
            *failure_count += 1;

            info!(
                "Primary failure detected ({}/{})",
                *failure_count, self.failure_threshold
            );

            if *failure_count >= self.failure_threshold {
                return self.trigger_failover().await;
            }
        }

        Ok(false)
    }

    /// Trigger failover promotion
    ///
    /// Returns true if failover was triggered, false if already triggered
    async fn trigger_failover(&self) -> Result<bool> {
        let mut triggered = self.failover_triggered.write().await;

        if *triggered {
            debug!("Failover already triggered");
            return Ok(false);
        }

        error!(
            "Primary failure threshold reached ({} consecutive failures), triggering failover",
            self.failure_threshold
        );

        *triggered = true;

        // Invoke callback if set
        if let Some(callback) = &self.on_failover {
            callback();
        }

        Ok(true)
    }

    /// Check if failover has been triggered
    pub async fn is_failover_triggered(&self) -> bool {
        *self.failover_triggered.read().await
    }

    /// Get the current failure count
    pub async fn failure_count(&self) -> u32 {
        *self.failure_count.read().await
    }

    /// Get the last heartbeat
    pub async fn last_heartbeat(&self) -> Option<Heartbeat> {
        self.last_heartbeat.read().await.clone()
    }

    /// Reset the failover detector
    ///
    /// This is useful for testing or after manual intervention
    pub async fn reset(&self) {
        *self.last_heartbeat.write().await = None;
        *self.failure_count.write().await = 0;
        *self.failover_triggered.write().await = false;
    }

    /// Start the health monitoring loop
    ///
    /// This spawns a background task that periodically checks primary health
    pub fn start_monitoring(&self, check_interval: Duration) {
        let detector = Self {
            last_heartbeat: self.last_heartbeat.clone(),
            heartbeat_timeout: self.heartbeat_timeout,
            failure_threshold: self.failure_threshold,
            failure_count: self.failure_count.clone(),
            failover_triggered: self.failover_triggered.clone(),
            on_failover: self.on_failover.clone(),
        };

        tokio::spawn(async move {
            info!(
                "Starting failover monitoring (check_interval: {:?}, heartbeat_timeout: {:?}, failure_threshold: {})",
                check_interval, detector.heartbeat_timeout, detector.failure_threshold
            );

            loop {
                tokio::time::sleep(check_interval).await;

                match detector.check_health().await {
                    Ok(failover_triggered) => {
                        if failover_triggered {
                            error!("Failover triggered, stopping health monitoring");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Health check failed: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_failover_detector_creation() {
        let detector = FailoverDetector::new(Duration::from_secs(5), 3);

        assert_eq!(detector.failure_count().await, 0);
        assert!(!detector.is_failover_triggered().await);
        assert!(detector.last_heartbeat().await.is_none());
    }

    #[tokio::test]
    async fn test_record_heartbeat() {
        let detector = FailoverDetector::new(Duration::from_secs(5), 3);

        let heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now(),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(heartbeat.clone()).await;

        let last = detector.last_heartbeat().await.unwrap();
        assert_eq!(last.sequence, 100);
        assert_eq!(last.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_heartbeat_timeout_detection() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        // Record an old heartbeat
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(old_heartbeat).await;

        // Check health - should detect timeout
        let result = detector.check_health().await.unwrap();
        assert!(!result); // First failure, not triggered yet
        assert_eq!(detector.failure_count().await, 1);

        // Check again - should increment failure count
        let result = detector.check_health().await.unwrap();
        assert!(result); // Second failure, should trigger
        assert_eq!(detector.failure_count().await, 2);
        assert!(detector.is_failover_triggered().await);
    }

    #[tokio::test]
    async fn test_unhealthy_status_detection() {
        let detector = FailoverDetector::new(Duration::from_secs(10), 2);

        // Record unhealthy heartbeat
        let heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now(),
            status: HealthStatus::Unhealthy,
        };

        detector.record_heartbeat(heartbeat).await;

        // Check health - should detect unhealthy status
        let result = detector.check_health().await.unwrap();
        assert!(!result); // First failure
        assert_eq!(detector.failure_count().await, 1);

        // Check again
        let result = detector.check_health().await.unwrap();
        assert!(result); // Second failure, should trigger
        assert!(detector.is_failover_triggered().await);
    }

    #[tokio::test]
    async fn test_failure_count_reset_on_recovery() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 3);

        // Record old heartbeat to trigger failure
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(old_heartbeat).await;

        // Check health - should increment failure count
        detector.check_health().await.unwrap();
        assert_eq!(detector.failure_count().await, 1);

        // Record new healthy heartbeat
        let new_heartbeat = Heartbeat {
            sequence: 101,
            timestamp: Utc::now(),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(new_heartbeat).await;

        // Failure count should be reset
        assert_eq!(detector.failure_count().await, 0);
    }

    #[tokio::test]
    async fn test_failover_callback() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        let callback_invoked = Arc::new(AtomicBool::new(false));
        let callback_invoked_clone = callback_invoked.clone();

        let mut detector_mut = detector;
        detector_mut.set_failover_callback(move || {
            callback_invoked_clone.store(true, Ordering::SeqCst);
        });

        // Record old heartbeat
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector_mut.record_heartbeat(old_heartbeat).await;

        // Trigger failover
        detector_mut.check_health().await.unwrap();
        detector_mut.check_health().await.unwrap();

        // Callback should have been invoked
        assert!(callback_invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_failover_only_triggered_once() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        // Record old heartbeat
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(old_heartbeat).await;

        // Trigger failover
        detector.check_health().await.unwrap();
        let first_trigger = detector.check_health().await.unwrap();
        assert!(first_trigger);

        // Try to trigger again
        let second_trigger = detector.check_health().await.unwrap();
        assert!(!second_trigger); // Should not trigger again
    }

    #[tokio::test]
    async fn test_reset() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        // Record heartbeat and trigger failure
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector.record_heartbeat(old_heartbeat).await;
        detector.check_health().await.unwrap();
        detector.check_health().await.unwrap();

        assert!(detector.is_failover_triggered().await);
        assert_eq!(detector.failure_count().await, 2);

        // Reset
        detector.reset().await;

        assert!(!detector.is_failover_triggered().await);
        assert_eq!(detector.failure_count().await, 0);
        assert!(detector.last_heartbeat().await.is_none());
    }

    #[tokio::test]
    async fn test_no_heartbeat_detection() {
        let detector = FailoverDetector::new(Duration::from_secs(5), 2);

        // Check health without any heartbeat
        let result = detector.check_health().await.unwrap();
        assert!(!result); // First failure
        assert_eq!(detector.failure_count().await, 1);

        // Check again
        let result = detector.check_health().await.unwrap();
        assert!(result); // Second failure, should trigger
        assert!(detector.is_failover_triggered().await);
    }

    #[tokio::test]
    async fn test_monitoring_loop() {
        let detector = FailoverDetector::new(Duration::from_millis(50), 2);

        let callback_invoked = Arc::new(AtomicBool::new(false));
        let callback_invoked_clone = callback_invoked.clone();

        let mut detector_mut = detector;
        detector_mut.set_failover_callback(move || {
            callback_invoked_clone.store(true, Ordering::SeqCst);
        });

        // Record old heartbeat
        let old_heartbeat = Heartbeat {
            sequence: 100,
            timestamp: Utc::now() - chrono::Duration::milliseconds(100),
            status: HealthStatus::Healthy,
        };

        detector_mut.record_heartbeat(old_heartbeat).await;

        // Start monitoring with short check interval
        detector_mut.start_monitoring(Duration::from_millis(30));

        // Wait for monitoring to detect failure and trigger failover
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Callback should have been invoked
        assert!(callback_invoked.load(Ordering::SeqCst));
        assert!(detector_mut.is_failover_triggered().await);
    }
}
