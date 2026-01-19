//! CAPTCHA Alerting System
//!
//! Automated alerting for CAPTCHA security events, performance issues, and system health

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::dashboard::{Alert, AlertSeverity};
use super::error::CaptchaError;
use super::metrics::*;
use super::types::*;

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique identifier for the alert rule
    pub id: String,
    /// Human-readable name of the alert rule
    pub name: String,
    /// Detailed description of what this rule monitors
    pub description: String,
    /// Type of metric this rule monitors
    pub rule_type: AlertRuleType,
    /// Condition that triggers the alert
    pub condition: AlertCondition,
    /// Severity level of alerts from this rule
    pub severity: AlertSeverity,
    /// Whether this rule is currently enabled
    pub enabled: bool,
    /// Cooldown period in minutes between alerts
    pub cooldown_minutes: u64,
    /// Channels to send notifications through
    pub notification_channels: Vec<NotificationChannel>,
}

/// Types of alert rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRuleType {
    BotDetectionRate,
    /// Alert when success rate falls below threshold
    SuccessRate,
    /// Alert when average latency exceeds threshold
    AverageLatency,
    /// Alert when error rate exceeds threshold
    ErrorRate,
    /// Alert when concurrent challenges exceed threshold
    ConcurrentChallenges,
    /// Alert on security incidents
    SecurityIncident,
    /// Alert on system health issues
    SystemHealth,
    /// Alert on user experience degradation
    UserExperience,
}

/// Alert condition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Alert when bot detection rate exceeds threshold
pub struct AlertCondition {
    /// Name of the metric to monitor
    pub metric: String,
    /// Comparison operator for the threshold
    pub operator: ComparisonOperator,
    /// Threshold value to compare against
    pub threshold: f64,
    /// Time window in minutes for evaluation
    pub time_window_minutes: u64,
    /// Number of consecutive violations required to trigger
    pub consecutive_violations: u32,
}

/// Comparison operators for alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    /// Metric value must be less than threshold
    LessThan,
    /// Metric value must be greater than or equal to threshold
    GreaterThanOrEqual,
    /// Metric value must be less than or equal to threshold
    LessThanOrEqual,
    /// Metric value must equal threshold
    Equal,
    /// Metric value must not equal threshold
    NotEqual,
}

/// Notification channels for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Metric value must be greater than threshold
pub enum NotificationChannel {
    Email {
        /// List of email addresses to send alerts to
        addresses: Vec<String>,
    },
    /// Send alerts via HTTP webhook
    Webhook {
        /// URL of the webhook endpoint
        url: String,
        /// HTTP headers to include in webhook requests
        headers: HashMap<String, String>,
    },
    /// Send alerts to Slack channel
    Slack {
        /// Slack webhook URL for posting messages
        webhook_url: String,
        /// Slack channel name to post alerts to
        channel: String,
    },
    /// Send alerts to PagerDuty
    PagerDuty {
        /// PagerDuty integration key for routing alerts
        integration_key: String,
    },
    /// Log alerts to system logs
    Log {
        /// Log level to use for logging alerts
        level: LogLevel,
    },
}

/// Log levels for logging notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Send alerts via email to specified addresses
pub enum LogLevel {
    Info,
    /// Warning log level
    Warn,
    /// Error log level
    Error,
    /// Critical log level
    Critical,
}

/// Alert state tracking
#[derive(Debug, Clone)]
struct AlertState {
    /// ID of the alert rule this state belongs to
    rule_id: String,
    /// Number of consecutive violations detected
    consecutive_violations: u32,
    /// Last time this alert was triggered
    last_triggered: Option<SystemTime>,
    /// Last time this alert was resolved
    last_resolved: Option<SystemTime>,
    /// ID of the currently active alert
    active_alert_id: Option<String>,
}

/// Alerting engine
    /// Informational log level
pub struct AlertingEngine {
    rules: Arc<RwLock<Vec<AlertRule>>>,
    alert_states: Arc<RwLock<HashMap<String, AlertState>>>,
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    metrics_collector: Arc<MetricsCollector>,
    notification_sender: Arc<NotificationSender>,
}

impl AlertingEngine {
    /// Create a new alerting engine with default rules and configuration
    pub fn new(
        metrics_collector: Arc<MetricsCollector>,
        notification_sender: Arc<NotificationSender>,
    ) -> Self {
        let default_rules = Self::create_default_rules();

        Self {
            rules: Arc::new(RwLock::new(default_rules)),
            alert_states: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            metrics_collector,
            notification_sender,
        }
    }

    fn create_default_rules() -> Vec<AlertRule> {
        vec![
            // High bot detection rate alert
            AlertRule {
                id: "bot_detection_high".to_string(),
                name: "High Bot Detection Rate".to_string(),
                description: "Bot detection rate exceeds threshold".to_string(),
                rule_type: AlertRuleType::BotDetectionRate,
                condition: AlertCondition {
                    metric: "bot_detection_rate".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 0.15, // 15%
                    time_window_minutes: 15,
                    consecutive_violations: 2,
                },
                severity: AlertSeverity::Warning,
                enabled: true,
                cooldown_minutes: 30,
                notification_channels: vec![
                    NotificationChannel::Log {
                        level: LogLevel::Warn,
                    },
                    NotificationChannel::Email {
                        addresses: vec!["security@example.com".to_string()],
                    },
                ],
            },
            // Low success rate alert
            AlertRule {
                id: "success_rate_low".to_string(),
                name: "Low CAPTCHA Success Rate".to_string(),
                description: "CAPTCHA success rate below acceptable threshold".to_string(),
                rule_type: AlertRuleType::SuccessRate,
                condition: AlertCondition {
                    metric: "success_rate".to_string(),
                    operator: ComparisonOperator::LessThan,
                    threshold: 0.80, // 80%
                    time_window_minutes: 30,
                    consecutive_violations: 3,
                },
                severity: AlertSeverity::Critical,
                enabled: true,
                cooldown_minutes: 60,
                notification_channels: vec![
                    NotificationChannel::Log {
                        level: LogLevel::Error,
                    },
                    NotificationChannel::Email {
                        addresses: vec!["ops@example.com".to_string()],
                    },
                ],
            },
            // High latency alert
            AlertRule {
                id: "latency_high".to_string(),
                name: "High CAPTCHA Latency".to_string(),
                description: "CAPTCHA response time exceeds acceptable threshold".to_string(),
                rule_type: AlertRuleType::AverageLatency,
                condition: AlertCondition {
                    metric: "average_latency_ms".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 2000.0, // 2 seconds
                    time_window_minutes: 10,
                    consecutive_violations: 2,
                },
                severity: AlertSeverity::Warning,
                enabled: true,
                cooldown_minutes: 20,
                notification_channels: vec![NotificationChannel::Log {
                    level: LogLevel::Warn,
                }],
            },
            // Security incident alert
            AlertRule {
                id: "security_incident".to_string(),
                name: "Security Incident Detected".to_string(),
                description: "High number of attack attempts detected".to_string(),
                rule_type: AlertRuleType::SecurityIncident,
                condition: AlertCondition {
                    metric: "attack_attempts".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 100.0,
                    time_window_minutes: 5,
                    consecutive_violations: 1,
                },
                severity: AlertSeverity::Critical,
                enabled: true,
                cooldown_minutes: 15,
                notification_channels: vec![
                    NotificationChannel::Log {
                        level: LogLevel::Critical,
                    },
                    NotificationChannel::Email {
                        addresses: vec!["security@example.com".to_string()],
                    },
                ],
            },
        ]
    }

    /// Start the alerting engine
    pub async fn start(&self) -> Result<(), CaptchaError> {
        let engine = Arc::new(self.clone());

        // Start monitoring loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = engine.evaluate_rules().await {
                    eprintln!("Error evaluating alert rules: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Evaluate all alert rules
    async fn evaluate_rules(&self) -> Result<(), CaptchaError> {
        let rules = self.rules.read().await;

        for rule in rules.iter() {
            if rule.enabled {
                if let Err(e) = self.evaluate_rule(rule).await {
                    eprintln!("Error evaluating rule {}: {}", rule.id, e);
                }
            }
        }

        Ok(())
    }

    /// Evaluate a single alert rule
    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<(), CaptchaError> {
        // Get current metrics
        let metrics = self
            .metrics_collector
            .generate_summary(rule.condition.time_window_minutes)
            .await?;

        // Extract metric value based on rule type
        let metric_value = self.extract_metric_value(&metrics, rule)?;

        // Check if condition is met
        let condition_met = self.evaluate_condition(&rule.condition, metric_value);

        // Update alert state
        let mut alert_states = self.alert_states.write().await;
        let state = alert_states
            .entry(rule.id.clone())
            .or_insert_with(|| AlertState {
                rule_id: rule.id.clone(),
                consecutive_violations: 0,
                last_triggered: None,
                last_resolved: None,
                active_alert_id: None,
            });

        if condition_met {
            state.consecutive_violations += 1;

            // Check if we should trigger an alert
            if state.consecutive_violations >= rule.condition.consecutive_violations {
                // Check cooldown period
                if let Some(last_triggered) = state.last_triggered {
                    let cooldown_duration = Duration::from_secs(rule.cooldown_minutes * 60);
                    if SystemTime::now()
                        .duration_since(last_triggered)
                        .unwrap_or(Duration::ZERO)
                        < cooldown_duration
                    {
                        return Ok(()); // Still in cooldown
                    }
                }

                // Trigger alert
                self.trigger_alert(rule, metric_value, state).await?;
            }
        } else {
            // Condition not met, reset violations and resolve active alert if any
            if state.consecutive_violations > 0 {
                state.consecutive_violations = 0;

                if let Some(alert_id) = &state.active_alert_id {
                    self.resolve_alert(alert_id.clone()).await?;
                    state.active_alert_id = None;
                    state.last_resolved = Some(SystemTime::now());
                }
            }
        }

        Ok(())
    }

    /// Extract metric value from metrics summary
    fn extract_metric_value(
        &self,
        metrics: &MetricsSummary,
        rule: &AlertRule,
    ) -> Result<f64, CaptchaError> {
        let value = match rule.rule_type {
            AlertRuleType::BotDetectionRate => metrics.bot_detection.accuracy_rate,
            AlertRuleType::SuccessRate => metrics.performance.success_rate,
            AlertRuleType::AverageLatency => metrics.performance.validation_latency_ms as f64,
            AlertRuleType::ErrorRate => metrics.performance.failure_rate,
            AlertRuleType::ConcurrentChallenges => metrics.performance.concurrent_challenges as f64,
            AlertRuleType::SecurityIncident => metrics.security_events.attack_attempts as f64,
            AlertRuleType::SystemHealth => {
                // For system health, we could use CPU or memory usage
                metrics.performance.cpu_usage_percent
            }
            AlertRuleType::UserExperience => metrics.user_experience.user_satisfaction_score,
        };

        Ok(value)
    }

    /// Evaluate alert condition
    fn evaluate_condition(&self, condition: &AlertCondition, value: f64) -> bool {
        match condition.operator {
            ComparisonOperator::GreaterThan => value > condition.threshold,
            ComparisonOperator::LessThan => value < condition.threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= condition.threshold,
            ComparisonOperator::LessThanOrEqual => value <= condition.threshold,
            ComparisonOperator::Equal => (value - condition.threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - condition.threshold).abs() >= f64::EPSILON,
        }
    }

    /// Trigger an alert
    async fn trigger_alert(
        &self,
        rule: &AlertRule,
        metric_value: f64,
        state: &mut AlertState,
    ) -> Result<(), CaptchaError> {
        let alert_id = Uuid::new_v4().to_string();

        let alert = Alert {
            id: alert_id.clone(),
            severity: rule.severity.clone(),
            title: rule.name.clone(),
            description: format!(
                "{}: {} {} {} (current: {})",
                rule.description,
                rule.condition.metric,
                self.operator_to_string(&rule.condition.operator),
                rule.condition.threshold,
                metric_value
            ),
            timestamp: SystemTime::now(),
            acknowledged: false,
            resolved: false,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("rule_id".to_string(), rule.id.clone());
                metadata.insert("metric_value".to_string(), metric_value.to_string());
                metadata.insert(
                    "threshold".to_string(),
                    rule.condition.threshold.to_string(),
                );
                metadata
            },
        };

        // Add to active alerts
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.push(alert.clone());

        // Update state
        state.last_triggered = Some(SystemTime::now());
        state.active_alert_id = Some(alert_id);

        // Send notifications
        for channel in &rule.notification_channels {
            if let Err(e) = self
                .notification_sender
                .send_notification(channel, &alert)
                .await
            {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        Ok(())
    }

    /// Resolve an alert
    async fn resolve_alert(&self, alert_id: String) -> Result<(), CaptchaError> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(alert) = active_alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;

            // Send resolution notification
            let resolution_alert = Alert {
                id: Uuid::new_v4().to_string(),
                severity: AlertSeverity::Info,
                title: format!("RESOLVED: {}", alert.title),
                description: format!("Alert '{}' has been resolved", alert.title),
                timestamp: SystemTime::now(),
                acknowledged: true,
                resolved: true,
                metadata: alert.metadata.clone(),
            };

            // For now, just log the resolution
            println!("Alert resolved: {}", resolution_alert.title);
        }

        Ok(())
    }

    /// Convert operator to string for display
    fn operator_to_string(&self, operator: &ComparisonOperator) -> &'static str {
        match operator {
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::Equal => "==",
            ComparisonOperator::NotEqual => "!=",
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().await.clone()
    }

    /// Add a new alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<(), CaptchaError> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Update an existing alert rule
    pub async fn update_rule(
        &self,
        rule_id: &str,
        updated_rule: AlertRule,
    ) -> Result<(), CaptchaError> {
        let mut rules = self.rules.write().await;

        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            *rule = updated_rule;
            Ok(())
        } else {
            Err(CaptchaError::ValidationFailed {
                message: format!("Rule not found: {}", rule_id),
                attempts_remaining: 0,
                next_difficulty: 1,
            })
        }
    }

    /// Delete an alert rule
    pub async fn delete_rule(&self, rule_id: &str) -> Result<(), CaptchaError> {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.id != rule_id);

        if rules.len() < initial_len {
            Ok(())
        } else {
            Err(CaptchaError::ValidationFailed {
                message: format!("Rule not found: {}", rule_id),
                attempts_remaining: 0,
                next_difficulty: 1,
            })
        }
    }

    /// Get all alert rules
    pub async fn get_rules(&self) -> Vec<AlertRule> {
        self.rules.read().await.clone()
    }
}

impl Clone for AlertingEngine {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
            alert_states: self.alert_states.clone(),
            active_alerts: self.active_alerts.clone(),
            metrics_collector: self.metrics_collector.clone(),
            notification_sender: self.notification_sender.clone(),
        }
    }
}
/// Notification sender for alerts
    /// Create default alert rules
pub struct NotificationSender {
    // Configuration for different notification channels
}

impl NotificationSender {
    /// Create a new notification sender instance
    pub fn new() -> Self {
        Self {}
    }

    /// Send notification through specified channel
    pub async fn send_notification(
        &self,
        channel: &NotificationChannel,
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        match channel {
            NotificationChannel::Email { addresses } => {
                self.send_email_notification(addresses, alert).await
            }
            NotificationChannel::Webhook { url, headers } => {
                self.send_webhook_notification(url, headers, alert).await
            }
            NotificationChannel::Slack {
                webhook_url,
                channel,
            } => {
                self.send_slack_notification(webhook_url, channel, alert)
                    .await
            }
            NotificationChannel::PagerDuty { integration_key } => {
                self.send_pagerduty_notification(integration_key, alert)
                    .await
            }
            NotificationChannel::Log { level } => self.send_log_notification(level, alert).await,
        }
    }

    async fn send_email_notification(
        &self,
        addresses: &[String],
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        // In a real implementation, this would integrate with an email service
        println!(
            "EMAIL ALERT to {:?}: {} - {}",
            addresses, alert.title, alert.description
        );
        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook_notification(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        // In a real implementation, this would make an HTTP POST request
        println!(
            "WEBHOOK ALERT to {}: {} - {}",
            url, alert.title, alert.description
        );
        Ok(())
    }

    /// Send Slack notification
    async fn send_slack_notification(
        &self,
        webhook_url: &str,
        channel: &str,
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        // In a real implementation, this would send to Slack webhook
        println!(
            "SLACK ALERT to {} ({}): {} - {}",
            channel, webhook_url, alert.title, alert.description
        );
        Ok(())
    }

    /// Send PagerDuty notification
    async fn send_pagerduty_notification(
        &self,
        integration_key: &str,
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        // In a real implementation, this would integrate with PagerDuty API
        println!(
            "PAGERDUTY ALERT ({}): {} - {}",
            integration_key, alert.title, alert.description
        );
        Ok(())
    }

    /// Send log notification
    async fn send_log_notification(
        &self,
        level: &LogLevel,
        alert: &Alert,
    ) -> Result<(), CaptchaError> {
        match level {
            LogLevel::Info => println!("INFO: {} - {}", alert.title, alert.description),
            LogLevel::Warn => println!("WARN: {} - {}", alert.title, alert.description),
            LogLevel::Error => eprintln!("ERROR: {} - {}", alert.title, alert.description),
            LogLevel::Critical => eprintln!("CRITICAL: {} - {}", alert.title, alert.description),
        }
        Ok(())
    }
}

/// Alert manager that coordinates alerting with the CAPTCHA service
    /// Send email notification
pub struct AlertManager {
    alerting_engine: Arc<AlertingEngine>,
    metrics_collector: Arc<MetricsCollector>,
}

impl AlertManager {
    /// Create a new alert manager with the given metrics collector
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        let notification_sender = Arc::new(NotificationSender::new());
        let alerting_engine = Arc::new(AlertingEngine::new(
            metrics_collector.clone(),
            notification_sender,
        ));

        Self {
            alerting_engine,
            metrics_collector,
        }
    }

    /// Start the alert manager
    pub async fn start(&self) -> Result<(), CaptchaError> {
        self.alerting_engine.start().await
    }

    /// Manually trigger an alert for testing
    pub async fn trigger_test_alert(
        &self,
        severity: AlertSeverity,
        title: String,
        description: String,
    ) -> Result<(), CaptchaError> {
        let alert = Alert {
            id: Uuid::new_v4().to_string(),
            severity,
            title,
            description,
            timestamp: SystemTime::now(),
            acknowledged: false,
            resolved: false,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("test_alert".to_string(), "true".to_string());
                metadata
            },
        };

        // Add to active alerts
        let mut active_alerts = self.alerting_engine.active_alerts.write().await;
        active_alerts.push(alert.clone());

        // Send test notification
        let notification_sender = Arc::new(NotificationSender::new());
        let log_channel = NotificationChannel::Log {
            level: LogLevel::Info,
        };
        notification_sender
            .send_notification(&log_channel, &alert)
            .await?;

        Ok(())
    }

    /// Get alerting engine for direct access
    pub fn get_alerting_engine(&self) -> Arc<AlertingEngine> {
        self.alerting_engine.clone()
    }

    /// Record a security event that might trigger alerts
    pub async fn record_security_event(
        &self,
        event_type: SecurityEventType,
        ip_address: &str,
        risk_level: &RiskLevel,
        country_code: Option<&str>,
    ) -> Result<(), CaptchaError> {
        // Record the event in metrics
        self.metrics_collector
            .record_security_event(event_type, ip_address, risk_level, country_code)
            .await?;

        // Check if this should trigger immediate alerts
        match risk_level {
            RiskLevel::Critical => {
                self.trigger_test_alert(
                    AlertSeverity::Critical,
                    "Critical Security Event".to_string(),
                    format!("Critical security event detected from IP: {}", ip_address),
                )
                .await?;
            }
            RiskLevel::High => {
                self.trigger_test_alert(
                    AlertSeverity::Warning,
                    "High Risk Security Event".to_string(),
                    format!("High risk security event detected from IP: {}", ip_address),
                )
                .await?;
            }
            _ => {} // Lower risk levels handled by regular rule evaluation
        }

        Ok(())
    }

    /// Check system health and trigger alerts if needed
    pub async fn check_system_health(&self) -> Result<(), CaptchaError> {
        let metrics = self.metrics_collector.get_real_time_metrics().await?;

        // Check for performance issues
        if metrics.performance.validation_latency_ms > 5000 {
            self.trigger_test_alert(
                AlertSeverity::Critical,
                "System Performance Critical".to_string(),
                format!(
                    "CAPTCHA validation latency is {}ms, exceeding critical threshold",
                    metrics.performance.validation_latency_ms
                ),
            )
            .await?;
        }

        // Check for high error rates
        if metrics.performance.failure_rate > 0.5 {
            self.trigger_test_alert(
                AlertSeverity::Warning,
                "High Error Rate".to_string(),
                format!(
                    "CAPTCHA failure rate is {:.1}%, exceeding warning threshold",
                    metrics.performance.failure_rate * 100.0
                ),
            )
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_condition_evaluation() {
        let condition = AlertCondition {
            metric: "test_metric".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 10.0,
            time_window_minutes: 5,
            consecutive_violations: 1,
        };

        let metrics_collector = Arc::new(MetricsCollector::new(Arc::new(
            crate::database::CaptchaOperations::new(crate::database::Database::mock().await),
        )));
        let notification_sender = Arc::new(NotificationSender::new());
        let engine = AlertingEngine::new(metrics_collector, notification_sender);

        assert!(engine.evaluate_condition(&condition, 15.0));
        assert!(!engine.evaluate_condition(&condition, 5.0));
        assert!(!engine.evaluate_condition(&condition, 10.0));
    }

    #[tokio::test]
    async fn test_operator_to_string() {
        let metrics_collector = Arc::new(MetricsCollector::new(Arc::new(
            crate::database::CaptchaOperations::new(crate::database::Database::mock().await),
        )));
        let notification_sender = Arc::new(NotificationSender::new());
        let engine = AlertingEngine::new(metrics_collector, notification_sender);

        assert_eq!(
            engine.operator_to_string(&ComparisonOperator::GreaterThan),
            ">"
        );
        assert_eq!(
            engine.operator_to_string(&ComparisonOperator::LessThan),
            "<"
        );
        assert_eq!(engine.operator_to_string(&ComparisonOperator::Equal), "==");
    }

    #[tokio::test]
    async fn test_alert_rule_creation() {
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            rule_type: AlertRuleType::BotDetectionRate,
            condition: AlertCondition {
                metric: "bot_detection_rate".to_string(),
                operator: ComparisonOperator::GreaterThan,
                threshold: 0.1,
                time_window_minutes: 10,
                consecutive_violations: 2,
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 30,
            notification_channels: vec![NotificationChannel::Log {
                level: LogLevel::Warn,
            }],
        };

        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.condition.threshold, 0.1);
        assert!(rule.enabled);
    }
}
