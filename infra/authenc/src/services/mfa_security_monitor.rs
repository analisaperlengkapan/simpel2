//! MFA-specific security monitoring and anomaly detection service

use crate::error::AuthencError;
use crate::models::events::EventType;
use crate::services::events::EventManager;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// MFA security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSecurityMonitorConfig {
    /// Maximum failed attempts per IP in time window before alerting
    pub max_failed_attempts_per_ip: u32,
    /// Time window for failed attempt tracking (seconds)
    pub failed_attempts_window_seconds: u64,
    /// Maximum MFA setups per IP per hour before alerting
    pub max_setups_per_ip_per_hour: u32,
    /// Threshold for suspicious geographic patterns
    pub geographic_anomaly_threshold: f64,
    /// Enable real-time alerting
    pub enable_real_time_alerts: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Minimum time between alerts for same IP (seconds)
    pub alert_cooldown_seconds: u64,
}

impl Default for MfaSecurityMonitorConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts_per_ip: 20,
            failed_attempts_window_seconds: 3600, // 1 hour
            max_setups_per_ip_per_hour: 5,
            geographic_anomaly_threshold: 0.8,
            enable_real_time_alerts: true,
            enable_anomaly_detection: true,
            alert_cooldown_seconds: 300, // 5 minutes
        }
    }
}

/// MFA security event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaSecurityEventType {
    /// Excessive failed MFA attempts from single IP
    ExcessiveFailedAttempts {
        /// IP address showing excessive failed attempts
        ip: String,
        /// Number of failed attempts
        attempts: u32,
        /// Time window for the attempts
        time_window: Duration,
    },
    /// Suspicious MFA setup pattern
    SuspiciousMfaSetupPattern {
        /// IP address showing suspicious setup pattern
        ip: String,
        /// Number of MFA setups in time window
        setups: u32,
        /// Time window for the setups
        time_window: Duration,
    },
    /// Geographic anomaly detected
    GeographicAnomaly {
        /// ID of the user showing geographic anomaly
        user_id: Uuid,
        /// Previous location of the user
        previous_location: String,
        /// Current location of the user
        current_location: String,
        /// Distance between locations in kilometers
        distance_km: f64,
        /// Time difference in minutes between logins
        time_diff_minutes: u64,
    },
    /// Account lockout triggered
    AccountLockoutTriggered {
        /// ID of the user whose account was locked
        user_id: Uuid,
        /// IP address that triggered the lockout
        ip: String,
        /// Number of failed attempts that triggered lockout
        failed_attempts: u32,
    },
    /// Brute force attack detected
    BruteForceAttackDetected {
        /// IP address performing the brute force attack
        ip: String,
        /// List of user IDs targeted by the attack
        target_users: Vec<Uuid>,
        /// Number of attack attempts
        attempts: u32,
    },
    /// Time-based anomaly (unusual login times)
    TimeBasedAnomaly {
        /// ID of the user showing time-based anomaly
        user_id: Uuid,
        /// Usual login hours for this user
        usual_hours: Vec<u8>,
        /// Current login hour
        current_hour: u8,
        /// Anomaly score for this event
        anomaly_score: f64,
    },
}

/// Security alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Low severity alert
    Low,
    /// Medium severity alert
    Medium,
    /// High severity alert
    High,
    /// Critical severity alert requiring immediate attention
    Critical,
}

/// Security alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    /// Unique identifier of the security alert
    pub id: Uuid,
    /// Type of security event that triggered the alert
    pub event_type: MfaSecurityEventType,
    /// Severity level of the alert
    pub severity: AlertSeverity,
    /// Timestamp when the alert was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Human-readable description of the alert
    pub description: String,
    /// Recommended actions to resolve the alert
    pub recommended_actions: Vec<String>,
    /// List of users affected by this alert
    pub affected_users: Vec<Uuid>,
    /// List of IP addresses involved in the alert
    pub source_ips: Vec<String>,
}

/// IP-based tracking information
#[derive(Debug)]
struct IpTrackingInfo {
    failed_attempts: u32,
    mfa_setups: u32,
    first_seen: Instant,
    last_activity: Instant,
    user_agents: Vec<String>,
    targeted_users: Vec<Uuid>,
}

/// User behavior pattern
#[derive(Debug)]
struct UserBehaviorPattern {
    usual_login_hours: HashMap<u8, u32>,   // Hour -> count
    usual_locations: HashMap<String, u32>, // Location -> count
    last_successful_mfa: Option<Instant>,
    average_mfa_time: Duration,
}

/// MFA security monitoring service
pub struct MfaSecurityMonitor {
    config: MfaSecurityMonitorConfig,
    event_manager: Arc<RwLock<EventManager>>,

    // Tracking data
    ip_tracking: Arc<RwLock<HashMap<String, IpTrackingInfo>>>,
    user_patterns: Arc<RwLock<HashMap<Uuid, UserBehaviorPattern>>>,
    recent_alerts: Arc<RwLock<HashMap<String, Instant>>>, // Alert key -> last alert time

    // Alert handlers
    alert_handlers: Vec<Box<dyn AlertHandler + Send + Sync>>,
}

/// Trait for handling security alerts
pub trait AlertHandler: Send + Sync {
    /// Handle a security alert by processing it according to implementation logic
    fn handle_alert(&self, alert: &SecurityAlert) -> Result<(), AuthencError>;
}

/// Default alert handler that logs alerts
pub struct LoggingAlertHandler;

impl AlertHandler for LoggingAlertHandler {
    fn handle_alert(&self, alert: &SecurityAlert) -> Result<(), AuthencError> {
        match alert.severity {
            AlertSeverity::Critical => {
                error!(
                    alert_id = %alert.id,
                    event_type = ?alert.event_type,
                    description = %alert.description,
                    "CRITICAL MFA Security Alert"
                );
            }
            AlertSeverity::High => {
                warn!(
                    alert_id = %alert.id,
                    event_type = ?alert.event_type,
                    description = %alert.description,
                    "HIGH MFA Security Alert"
                );
            }
            AlertSeverity::Medium => {
                warn!(
                    alert_id = %alert.id,
                    event_type = ?alert.event_type,
                    description = %alert.description,
                    "MEDIUM MFA Security Alert"
                );
            }
            AlertSeverity::Low => {
                info!(
                    alert_id = %alert.id,
                    event_type = ?alert.event_type,
                    description = %alert.description,
                    "LOW MFA Security Alert"
                );
            }
        }
        Ok(())
    }
}

impl MfaSecurityMonitor {
    /// Create a new MFA security monitor
    pub fn new(config: MfaSecurityMonitorConfig, event_manager: Arc<RwLock<EventManager>>) -> Self {
        let monitor = Self {
            config,
            event_manager,
            ip_tracking: Arc::new(RwLock::new(HashMap::new())),
            user_patterns: Arc::new(RwLock::new(HashMap::new())),
            recent_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_handlers: vec![Box::new(LoggingAlertHandler)],
        };

        // Start background cleanup task
        monitor.start_cleanup_task();

        monitor
    }

    /// Add a custom alert handler
    pub fn add_alert_handler(&mut self, handler: Box<dyn AlertHandler + Send + Sync>) {
        self.alert_handlers.push(handler);
    }

    /// Record a failed MFA attempt for monitoring
    pub async fn record_failed_mfa_attempt(
        &self,
        ip: &str,
        user_id: Option<Uuid>,
        user_agent: Option<String>,
    ) -> Result<(), AuthencError> {
        let mut ip_tracking = self.ip_tracking.write().await;
        let now = Instant::now();

        let tracking_info = ip_tracking
            .entry(ip.to_string())
            .or_insert_with(|| IpTrackingInfo {
                failed_attempts: 0,
                mfa_setups: 0,
                first_seen: now,
                last_activity: now,
                user_agents: Vec::new(),
                targeted_users: Vec::new(),
            });

        tracking_info.failed_attempts += 1;
        tracking_info.last_activity = now;

        if let Some(ua) = user_agent {
            if !tracking_info.user_agents.contains(&ua) {
                tracking_info.user_agents.push(ua);
            }
        }

        if let Some(uid) = user_id {
            if !tracking_info.targeted_users.contains(&uid) {
                tracking_info.targeted_users.push(uid);
            }
        }

        // Check for excessive failed attempts
        let should_alert_excessive = {
            let window_start =
                now - Duration::from_secs(self.config.failed_attempts_window_seconds);
            tracking_info.first_seen > window_start
                && tracking_info.failed_attempts >= self.config.max_failed_attempts_per_ip
        };

        if should_alert_excessive {
            let attempts = tracking_info.failed_attempts;
            let time_window = Duration::from_secs(self.config.failed_attempts_window_seconds);
            drop(ip_tracking); // Release lock before generating alert

            self.generate_alert(MfaSecurityEventType::ExcessiveFailedAttempts {
                ip: ip.to_string(),
                attempts,
                time_window,
            })
            .await?;
            return Ok(());
        }

        // Check for brute force patterns (multiple users targeted)
        let should_alert_brute_force =
            tracking_info.targeted_users.len() > 3 && tracking_info.failed_attempts > 10;

        if should_alert_brute_force {
            let target_users = tracking_info.targeted_users.clone();
            let attempts = tracking_info.failed_attempts;
            drop(ip_tracking);

            self.generate_alert(MfaSecurityEventType::BruteForceAttackDetected {
                ip: ip.to_string(),
                target_users,
                attempts,
            })
            .await?;
        }

        Ok(())
    }

    /// Record a successful MFA verification for pattern learning
    pub async fn record_successful_mfa(
        &self,
        user_id: Uuid,
        ip: &str,
        location: Option<String>,
    ) -> Result<(), AuthencError> {
        let mut user_patterns = self.user_patterns.write().await;
        let now = Instant::now();
        let current_hour = chrono::Utc::now().hour() as u8;

        let pattern = user_patterns
            .entry(user_id)
            .or_insert_with(|| UserBehaviorPattern {
                usual_login_hours: HashMap::new(),
                usual_locations: HashMap::new(),
                last_successful_mfa: None,
                average_mfa_time: Duration::from_secs(30),
            });

        // Update login hour pattern
        *pattern.usual_login_hours.entry(current_hour).or_insert(0) += 1;

        // Update location pattern
        if let Some(loc) = location {
            *pattern.usual_locations.entry(loc).or_insert(0) += 1;
        }

        pattern.last_successful_mfa = Some(now);

        // Check for time-based anomalies
        if self.config.enable_anomaly_detection {
            let total_logins: u32 = pattern.usual_login_hours.values().sum();
            let current_hour_count = pattern.usual_login_hours.get(&current_hour).unwrap_or(&0);

            if total_logins > 10 {
                // Only check after sufficient data
                let hour_probability = *current_hour_count as f64 / total_logins as f64;

                if hour_probability < 0.1 {
                    // Less than 10% of usual activity
                    let usual_hours: Vec<u8> = pattern
                        .usual_login_hours
                        .iter()
                        .filter(|&(_, &count)| count > 0)
                        .map(|(&hour, _)| hour)
                        .collect();

                    drop(user_patterns);

                    self.generate_alert(MfaSecurityEventType::TimeBasedAnomaly {
                        user_id,
                        usual_hours,
                        current_hour,
                        anomaly_score: 1.0 - hour_probability,
                    })
                    .await?;
                }
            }
        }

        Ok(())
    }

    /// Record an MFA setup for monitoring
    pub async fn record_mfa_setup(&self, ip: &str, user_id: Uuid) -> Result<(), AuthencError> {
        let mut ip_tracking = self.ip_tracking.write().await;
        let now = Instant::now();

        let tracking_info = ip_tracking
            .entry(ip.to_string())
            .or_insert_with(|| IpTrackingInfo {
                failed_attempts: 0,
                mfa_setups: 0,
                first_seen: now,
                last_activity: now,
                user_agents: Vec::new(),
                targeted_users: Vec::new(),
            });

        tracking_info.mfa_setups += 1;
        tracking_info.last_activity = now;

        // Check for excessive MFA setups
        let should_alert_setup = {
            let one_hour_ago = now - Duration::from_secs(3600);
            tracking_info.first_seen > one_hour_ago
                && tracking_info.mfa_setups >= self.config.max_setups_per_ip_per_hour
        };

        if should_alert_setup {
            let setups = tracking_info.mfa_setups;
            drop(ip_tracking);

            self.generate_alert(MfaSecurityEventType::SuspiciousMfaSetupPattern {
                ip: ip.to_string(),
                setups,
                time_window: Duration::from_secs(3600),
            })
            .await?;
        }

        Ok(())
    }

    /// Record an account lockout event
    pub async fn record_account_lockout(
        &self,
        user_id: Uuid,
        ip: &str,
        failed_attempts: u32,
    ) -> Result<(), AuthencError> {
        self.generate_alert(MfaSecurityEventType::AccountLockoutTriggered {
            user_id,
            ip: ip.to_string(),
            failed_attempts,
        })
        .await
    }

    /// Generate and handle a security alert
    async fn generate_alert(&self, event_type: MfaSecurityEventType) -> Result<(), AuthencError> {
        let alert_key = self.get_alert_key(&event_type);

        // Check alert cooldown
        {
            let recent_alerts = self.recent_alerts.read().await;
            if let Some(last_alert_time) = recent_alerts.get(&alert_key) {
                let cooldown = Duration::from_secs(self.config.alert_cooldown_seconds);
                if last_alert_time.elapsed() < cooldown {
                    return Ok(()); // Skip alert due to cooldown
                }
            }
        }

        // Update alert time
        {
            let mut recent_alerts = self.recent_alerts.write().await;
            recent_alerts.insert(alert_key, Instant::now());
        }

        let (severity, description, recommended_actions, affected_users, source_ips) =
            self.analyze_event(&event_type);

        let alert = SecurityAlert {
            id: Uuid::new_v4(),
            event_type: event_type.clone(),
            severity,
            timestamp: chrono::Utc::now(),
            description,
            recommended_actions,
            affected_users,
            source_ips,
        };

        // Handle the alert
        for handler in &self.alert_handlers {
            if let Err(e) = handler.handle_alert(&alert) {
                error!("Alert handler failed: {}", e);
            }
        }

        // Fire security event
        let security_event = crate::services::events::EventBuilder::new(
            EventType::SecurityAlert,
            "master".to_string(),
        )
        .detail("alert_id", alert.id.to_string())
        .detail("severity", format!("{:?}", alert.severity))
        .detail("event_type", format!("{:?}", event_type))
        .detail("description", alert.description.clone())
        .build();

        if let Err(e) = self
            .event_manager
            .write()
            .await
            .fire_event(security_event)
            .await
        {
            error!("Failed to fire security event: {}", e);
        }

        Ok(())
    }

    /// Analyze security event and determine response
    fn analyze_event(
        &self,
        event_type: &MfaSecurityEventType,
    ) -> (AlertSeverity, String, Vec<String>, Vec<Uuid>, Vec<String>) {
        match event_type {
            MfaSecurityEventType::ExcessiveFailedAttempts { ip, attempts, .. } => (
                AlertSeverity::High,
                format!(
                    "Excessive MFA failed attempts from IP {}: {} attempts",
                    ip, attempts
                ),
                vec![
                    "Consider blocking IP address".to_string(),
                    "Review authentication logs".to_string(),
                    "Check for compromised accounts".to_string(),
                ],
                vec![],
                vec![ip.clone()],
            ),
            MfaSecurityEventType::BruteForceAttackDetected {
                ip,
                target_users,
                attempts,
            } => (
                AlertSeverity::Critical,
                format!(
                    "Brute force attack detected from IP {}: {} attempts against {} users",
                    ip,
                    attempts,
                    target_users.len()
                ),
                vec![
                    "IMMEDIATELY block IP address".to_string(),
                    "Lock affected user accounts".to_string(),
                    "Notify security team".to_string(),
                    "Review network security".to_string(),
                ],
                target_users.clone(),
                vec![ip.clone()],
            ),
            MfaSecurityEventType::AccountLockoutTriggered {
                user_id,
                ip,
                failed_attempts,
            } => (
                AlertSeverity::Medium,
                format!(
                    "Account {} locked due to {} failed MFA attempts from {}",
                    user_id, failed_attempts, ip
                ),
                vec![
                    "Verify user identity before unlocking".to_string(),
                    "Check for account compromise".to_string(),
                    "Review user's recent activity".to_string(),
                ],
                vec![*user_id],
                vec![ip.clone()],
            ),
            MfaSecurityEventType::SuspiciousMfaSetupPattern { ip, setups, .. } => (
                AlertSeverity::Medium,
                format!(
                    "Suspicious MFA setup pattern from IP {}: {} setups",
                    ip, setups
                ),
                vec![
                    "Review MFA setup requests".to_string(),
                    "Verify user identities".to_string(),
                    "Consider rate limiting MFA setups".to_string(),
                ],
                vec![],
                vec![ip.clone()],
            ),
            MfaSecurityEventType::TimeBasedAnomaly {
                user_id,
                current_hour,
                anomaly_score,
                ..
            } => (
                AlertSeverity::Low,
                format!(
                    "Unusual login time for user {}: hour {} (anomaly score: {:.2})",
                    user_id, current_hour, anomaly_score
                ),
                vec![
                    "Verify user identity".to_string(),
                    "Check for account sharing".to_string(),
                    "Review user's location".to_string(),
                ],
                vec![*user_id],
                vec![],
            ),
            MfaSecurityEventType::GeographicAnomaly {
                user_id,
                distance_km,
                ..
            } => (
                AlertSeverity::High,
                format!(
                    "Geographic anomaly for user {}: {:.0} km travel",
                    user_id, distance_km
                ),
                vec![
                    "Verify user identity immediately".to_string(),
                    "Check for account compromise".to_string(),
                    "Consider additional authentication".to_string(),
                ],
                vec![*user_id],
                vec![],
            ),
        }
    }

    /// Generate a unique key for alert cooldown tracking
    fn get_alert_key(&self, event_type: &MfaSecurityEventType) -> String {
        match event_type {
            MfaSecurityEventType::ExcessiveFailedAttempts { ip, .. } => {
                format!("excessive_failed:{}", ip)
            }
            MfaSecurityEventType::BruteForceAttackDetected { ip, .. } => {
                format!("brute_force:{}", ip)
            }
            MfaSecurityEventType::AccountLockoutTriggered { user_id, .. } => {
                format!("lockout:{}", user_id)
            }
            MfaSecurityEventType::SuspiciousMfaSetupPattern { ip, .. } => {
                format!("suspicious_setup:{}", ip)
            }
            MfaSecurityEventType::TimeBasedAnomaly { user_id, .. } => {
                format!("time_anomaly:{}", user_id)
            }
            MfaSecurityEventType::GeographicAnomaly { user_id, .. } => {
                format!("geo_anomaly:{}", user_id)
            }
        }
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let ip_tracking = self.ip_tracking.clone();
        let user_patterns = self.user_patterns.clone();
        let recent_alerts = self.recent_alerts.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour

            loop {
                interval.tick().await;

                let now = Instant::now();
                let cleanup_threshold = now - Duration::from_secs(86400); // 24 hours

                // Clean up old IP tracking data
                {
                    let mut tracking = ip_tracking.write().await;
                    tracking.retain(|_, info| info.last_activity > cleanup_threshold);
                }

                // Clean up old alert cooldowns
                {
                    let mut alerts = recent_alerts.write().await;
                    let cooldown_threshold = now - Duration::from_secs(3600); // 1 hour
                    alerts.retain(|_, &mut last_time| last_time > cooldown_threshold);
                }

                info!("Cleaned up old MFA security monitoring data");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_failed_attempt_tracking() {
        let config = MfaSecurityMonitorConfig {
            max_failed_attempts_per_ip: 3,
            failed_attempts_window_seconds: 60,
            ..Default::default()
        };

        let event_manager = Arc::new(RwLock::new(EventManager::new()));
        let monitor = MfaSecurityMonitor::new(config, event_manager);

        let ip = "192.168.1.100";
        let user_id = Uuid::new_v4();

        // Record failed attempts
        for _ in 0..4 {
            monitor
                .record_failed_mfa_attempt(ip, Some(user_id), None)
                .await
                .unwrap();
        }

        // Should have generated an alert
        let ip_tracking = monitor.ip_tracking.read().await;
        let tracking_info = ip_tracking.get(ip).unwrap();
        assert_eq!(tracking_info.failed_attempts, 4);
    }

    #[tokio::test]
    async fn test_alert_cooldown() {
        let config = MfaSecurityMonitorConfig {
            alert_cooldown_seconds: 1,
            ..Default::default()
        };

        let event_manager = Arc::new(RwLock::new(EventManager::new()));
        let monitor = MfaSecurityMonitor::new(config, event_manager);

        let event_type = MfaSecurityEventType::ExcessiveFailedAttempts {
            ip: "192.168.1.1".to_string(),
            attempts: 10,
            time_window: Duration::from_secs(60),
        };

        // First alert should go through
        monitor.generate_alert(event_type.clone()).await.unwrap();

        // Second alert should be blocked by cooldown
        monitor.generate_alert(event_type.clone()).await.unwrap();

        // Wait for cooldown to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Third alert should go through
        monitor.generate_alert(event_type).await.unwrap();
    }
}
