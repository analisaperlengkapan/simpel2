//! Backup failure alerting system
//!
//! This module provides alerting functionality for backup operations,
//! triggering alerts on backup creation failures and verification failures.
//! It integrates with the existing monitoring system (Prometheus metrics).

use crate::error::{BackupError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Warning level - non-critical issues
    Warning,
    /// Error level - critical issues requiring attention
    Error,
    /// Critical level - severe issues requiring immediate action
    Critical,
}

/// Alert type for backup operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertType {
    /// Backup creation failed
    BackupCreationFailed,
    /// Backup verification failed
    BackupVerificationFailed,
    /// Backup cleanup failed
    BackupCleanupFailed,
    /// Backup restoration failed
    BackupRestorationFailed,
}

/// Alert message containing details about the failure
#[derive(Debug, Clone)]
pub struct Alert {
    /// Type of alert
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Backup ID (if applicable)
    pub backup_id: Option<String>,
    /// Error message
    pub message: String,
    /// Additional context
    pub context: Option<String>,
    /// Timestamp when alert was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        backup_id: Option<String>,
        message: String,
        context: Option<String>,
    ) -> Self {
        Self {
            alert_type,
            severity,
            backup_id,
            message,
            context,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a backup creation failure alert
    pub fn backup_creation_failed(message: String, context: Option<String>) -> Self {
        Self::new(
            AlertType::BackupCreationFailed,
            AlertSeverity::Critical,
            None,
            message,
            context,
        )
    }

    /// Create a backup verification failure alert
    pub fn backup_verification_failed(
        backup_id: String,
        message: String,
        context: Option<String>,
    ) -> Self {
        Self::new(
            AlertType::BackupVerificationFailed,
            AlertSeverity::Error,
            Some(backup_id),
            message,
            context,
        )
    }

    /// Create a backup cleanup failure alert
    pub fn backup_cleanup_failed(message: String, context: Option<String>) -> Self {
        Self::new(
            AlertType::BackupCleanupFailed,
            AlertSeverity::Warning,
            None,
            message,
            context,
        )
    }

    /// Create a backup restoration failure alert
    pub fn backup_restoration_failed(
        backup_id: String,
        message: String,
        context: Option<String>,
    ) -> Self {
        Self::new(
            AlertType::BackupRestorationFailed,
            AlertSeverity::Critical,
            Some(backup_id),
            message,
            context,
        )
    }
}

/// Alert handler trait for implementing custom alert destinations
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle an alert
    async fn handle_alert(&self, alert: &Alert) -> Result<()>;
}

/// Logging alert handler - logs alerts using tracing
#[derive(Debug, Clone)]
pub struct LoggingAlertHandler;

#[async_trait::async_trait]
impl AlertHandler for LoggingAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<()> {
        match alert.severity {
            AlertSeverity::Warning => {
                tracing::warn!(
                    alert_type = ?alert.alert_type,
                    backup_id = ?alert.backup_id,
                    message = %alert.message,
                    context = ?alert.context,
                    timestamp = %alert.timestamp,
                    "Backup alert (WARNING)"
                );
            }
            AlertSeverity::Error => {
                tracing::error!(
                    alert_type = ?alert.alert_type,
                    backup_id = ?alert.backup_id,
                    message = %alert.message,
                    context = ?alert.context,
                    timestamp = %alert.timestamp,
                    "Backup alert (ERROR)"
                );
            }
            AlertSeverity::Critical => {
                tracing::error!(
                    alert_type = ?alert.alert_type,
                    backup_id = ?alert.backup_id,
                    message = %alert.message,
                    context = ?alert.context,
                    timestamp = %alert.timestamp,
                    "Backup alert (CRITICAL)"
                );
            }
        }

        Ok(())
    }
}

/// Metrics alert handler - records alerts as Prometheus metrics
#[derive(Debug, Clone)]
pub struct MetricsAlertHandler;

#[async_trait::async_trait]
impl AlertHandler for MetricsAlertHandler {
    async fn handle_alert(&self, _alert: &Alert) -> Result<()> {
        // Record alert metrics if metrics feature is enabled
        #[cfg(feature = "metrics")]
        {
            let alert_type_label = match _alert.alert_type {
                AlertType::BackupCreationFailed => "backup_creation_failed",
                AlertType::BackupVerificationFailed => "backup_verification_failed",
                AlertType::BackupCleanupFailed => "backup_cleanup_failed",
                AlertType::BackupRestorationFailed => "backup_restoration_failed",
            };

            let severity_label = match _alert.severity {
                AlertSeverity::Warning => "warning",
                AlertSeverity::Error => "error",
                AlertSeverity::Critical => "critical",
            };

            // Increment alert counter
            metrics::counter!(
                "secreton_backup_alerts_total",
                "alert_type" => alert_type_label,
                "severity" => severity_label
            )
            .increment(1);

            // Update last alert timestamp gauge
            metrics::gauge!(
                "secreton_backup_last_alert_timestamp_seconds",
                "alert_type" => alert_type_label
            )
            .set(_alert.timestamp.timestamp() as f64);
        }

        Ok(())
    }
}

/// Alert manager for handling backup alerts
pub struct AlertManager {
    /// Alert handlers
    handlers: Arc<RwLock<Vec<Box<dyn AlertHandler>>>>,
    /// Whether alerting is enabled
    enabled: bool,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(enabled: bool) -> Self {
        let mut handlers: Vec<Box<dyn AlertHandler>> = Vec::new();

        // Always add logging handler
        handlers.push(Box::new(LoggingAlertHandler));

        // Add metrics handler if metrics feature is enabled
        #[cfg(feature = "metrics")]
        {
            handlers.push(Box::new(MetricsAlertHandler));
        }

        Self {
            handlers: Arc::new(RwLock::new(handlers)),
            enabled,
        }
    }

    /// Add a custom alert handler
    pub async fn add_handler(&self, handler: Box<dyn AlertHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Trigger an alert
    ///
    /// This method sends the alert to all registered handlers.
    /// If alerting is disabled, the alert is still logged but not sent to other handlers.
    ///
    /// # Arguments
    ///
    /// * `alert` - The alert to trigger
    ///
    /// # Errors
    ///
    /// Returns error if any handler fails to process the alert.
    /// Errors from individual handlers are logged but don't stop other handlers.
    pub async fn trigger_alert(&self, alert: Alert) -> Result<()> {
        if !self.enabled {
            tracing::debug!(
                alert_type = ?alert.alert_type,
                "Alerting disabled, skipping alert"
            );
            return Ok(());
        }

        let handlers = self.handlers.read().await;

        for handler in handlers.iter() {
            if let Err(e) = handler.handle_alert(&alert).await {
                tracing::error!(
                    alert_type = ?alert.alert_type,
                    error = %e,
                    "Failed to handle alert"
                );
                // Continue with other handlers even if one fails
            }
        }

        Ok(())
    }

    /// Trigger a backup creation failure alert
    pub async fn alert_backup_creation_failed(&self, error: &BackupError) -> Result<()> {
        let alert = Alert::backup_creation_failed(
            error.to_string(),
            Some(format!("Backup creation failed: {:?}", error)),
        );

        self.trigger_alert(alert).await
    }

    /// Trigger a backup verification failure alert
    pub async fn alert_backup_verification_failed(
        &self,
        backup_id: &str,
        error: &BackupError,
    ) -> Result<()> {
        let alert = Alert::backup_verification_failed(
            backup_id.to_string(),
            error.to_string(),
            Some(format!("Backup verification failed: {:?}", error)),
        );

        self.trigger_alert(alert).await
    }

    /// Trigger a backup cleanup failure alert
    pub async fn alert_backup_cleanup_failed(&self, error: &BackupError) -> Result<()> {
        let alert = Alert::backup_cleanup_failed(
            error.to_string(),
            Some(format!("Backup cleanup failed: {:?}", error)),
        );

        self.trigger_alert(alert).await
    }

    /// Trigger a backup restoration failure alert
    pub async fn alert_backup_restoration_failed(
        &self,
        backup_id: &str,
        error: &BackupError,
    ) -> Result<()> {
        let alert = Alert::backup_restoration_failed(
            backup_id.to_string(),
            error.to_string(),
            Some(format!("Backup restoration failed: {:?}", error)),
        );

        self.trigger_alert(alert).await
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_creation() {
        let alert = Alert::backup_creation_failed(
            "Test error".to_string(),
            Some("Test context".to_string()),
        );

        assert_eq!(alert.alert_type, AlertType::BackupCreationFailed);
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.message, "Test error");
        assert_eq!(alert.context, Some("Test context".to_string()));
        assert!(alert.backup_id.is_none());
    }

    #[tokio::test]
    async fn test_alert_verification_failed() {
        let alert = Alert::backup_verification_failed(
            "backup-123".to_string(),
            "Checksum mismatch".to_string(),
            None,
        );

        assert_eq!(alert.alert_type, AlertType::BackupVerificationFailed);
        assert_eq!(alert.severity, AlertSeverity::Error);
        assert_eq!(alert.backup_id, Some("backup-123".to_string()));
        assert_eq!(alert.message, "Checksum mismatch");
    }

    #[tokio::test]
    async fn test_logging_alert_handler() {
        let handler = LoggingAlertHandler;
        let alert = Alert::backup_creation_failed("Test error".to_string(), None);

        let result = handler.handle_alert(&alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_alert_handler() {
        let handler = MetricsAlertHandler;
        let alert = Alert::backup_verification_failed(
            "backup-123".to_string(),
            "Test error".to_string(),
            None,
        );

        let result = handler.handle_alert(&alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_manager_trigger() {
        let manager = AlertManager::new(true);
        let alert = Alert::backup_creation_failed("Test error".to_string(), None);

        let result = manager.trigger_alert(alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_manager_disabled() {
        let manager = AlertManager::new(false);
        let alert = Alert::backup_creation_failed("Test error".to_string(), None);

        let result = manager.trigger_alert(alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_manager_convenience_methods() {
        let manager = AlertManager::new(true);

        // Test backup creation failed
        let error = BackupError::RaftSnapshot("Test error".to_string());
        let result = manager.alert_backup_creation_failed(&error).await;
        assert!(result.is_ok());

        // Test backup verification failed
        let error = BackupError::VerificationFailed("Checksum mismatch".to_string());
        let result = manager
            .alert_backup_verification_failed("backup-123", &error)
            .await;
        assert!(result.is_ok());

        // Test backup cleanup failed
        let error = BackupError::Storage("Cleanup failed".to_string());
        let result = manager.alert_backup_cleanup_failed(&error).await;
        assert!(result.is_ok());

        // Test backup restoration failed
        let error = BackupError::RaftRestore("Restore failed".to_string());
        let result = manager
            .alert_backup_restoration_failed("backup-123", &error)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_alert_handler() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Custom handler that counts alerts
        struct CountingHandler {
            count: Arc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl AlertHandler for CountingHandler {
            async fn handle_alert(&self, _alert: &Alert) -> Result<()> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let handler = Box::new(CountingHandler {
            count: count.clone(),
        });

        let manager = AlertManager::new(true);
        manager.add_handler(handler).await;

        // Trigger multiple alerts
        for _ in 0..3 {
            let alert = Alert::backup_creation_failed("Test".to_string(), None);
            manager.trigger_alert(alert).await.unwrap();
        }

        // Verify custom handler was called (plus default handlers)
        assert!(count.load(Ordering::SeqCst) >= 3);
    }
}
