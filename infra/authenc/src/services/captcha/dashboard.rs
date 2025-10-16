//! CAPTCHA Monitoring Dashboard System
//!
//! Real-time analytics dashboard for CAPTCHA performance, security, and user experience

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::error::CaptchaError;
use super::metrics::*;
use super::types::*;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub refresh_interval_seconds: u64,
    pub data_retention_hours: u64,
    pub alert_thresholds: AlertThresholds,
    pub chart_time_windows: Vec<u64>, // in minutes
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub bot_detection_rate_high: f64,
    pub success_rate_low: f64,
    pub average_latency_high_ms: u64,
    pub error_rate_high: f64,
    pub concurrent_challenges_high: u64,
}

/// Real-time dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub timestamp: SystemTime,
    pub overview: OverviewMetrics,
    pub performance: PerformanceChartData,
    pub security: SecurityChartData,
    pub user_experience: UserExperienceChartData,
    pub alerts: Vec<Alert>,
    pub system_health: SystemHealthStatus,
}

/// Overview metrics for dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewMetrics {
    pub total_challenges_today: u64,
    pub success_rate_24h: f64,
    pub bot_detection_rate_24h: f64,
    pub average_response_time_ms: u64,
    pub active_challenges: u64,
    pub unique_users_24h: u64,
    pub security_incidents_24h: u64,
}
/// Performance chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceChartData {
    pub latency_over_time: Vec<TimeSeriesPoint>,
    pub success_rate_over_time: Vec<TimeSeriesPoint>,
    pub throughput_over_time: Vec<TimeSeriesPoint>,
    pub difficulty_distribution: HashMap<u8, u64>,
    pub challenge_type_distribution: HashMap<ChallengeType, u64>,
}

/// Security chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityChartData {
    pub threat_level_distribution: HashMap<RiskLevel, u64>,
    pub bot_detection_over_time: Vec<TimeSeriesPoint>,
    pub attack_attempts_over_time: Vec<TimeSeriesPoint>,
    pub geographic_threat_map: HashMap<String, ThreatData>,
    pub top_attacking_ips: Vec<IpThreatInfo>,
}

/// User experience chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserExperienceChartData {
    pub completion_time_distribution: Vec<TimeSeriesPoint>,
    pub abandonment_rate_over_time: Vec<TimeSeriesPoint>,
    pub accessibility_usage_over_time: Vec<TimeSeriesPoint>,
    pub satisfaction_score_over_time: Vec<TimeSeriesPoint>,
    pub retry_rate_by_difficulty: HashMap<u8, f64>,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: SystemTime,
    pub value: f64,
    pub label: Option<String>,
}

/// Geographic threat data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatData {
    pub country_code: String,
    pub country_name: String,
    pub threat_count: u64,
    pub risk_level: RiskLevel,
    pub coordinates: Option<(f64, f64)>, // lat, lng
}

/// IP threat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpThreatInfo {
    pub ip_address: String,
    pub threat_count: u64,
    pub risk_level: RiskLevel,
    pub country_code: Option<String>,
    pub last_seen: SystemTime,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub uptime_percentage: f64,
    pub last_incident: Option<SystemTime>,
}

/// Health status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Down,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub error_rate: f64,
    pub last_check: SystemTime,
    pub message: Option<String>,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub timestamp: SystemTime,
    pub acknowledged: bool,
    pub resolved: bool,
    pub metadata: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}
/// Dashboard service for generating real-time analytics
pub struct DashboardService {
    metrics_collector: Arc<MetricsCollector>,
    db_ops: Arc<crate::database::CaptchaOperations>,
    config: DashboardConfig,
    alerts: Arc<tokio::sync::RwLock<Vec<Alert>>>,
}

impl DashboardService {
    pub fn new(
        metrics_collector: Arc<MetricsCollector>,
        db_ops: Arc<crate::database::CaptchaOperations>,
        config: DashboardConfig,
    ) -> Self {
        Self {
            metrics_collector,
            db_ops,
            config,
            alerts: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Generate complete dashboard data
    pub async fn generate_dashboard_data(&self) -> Result<DashboardData, CaptchaError> {
        let timestamp = SystemTime::now();

        // Generate all dashboard components concurrently
        let (overview, performance, security, user_experience, system_health) = tokio::try_join!(
            self.generate_overview_metrics(),
            self.generate_performance_chart_data(),
            self.generate_security_chart_data(),
            self.generate_user_experience_chart_data(),
            self.generate_system_health_status()
        )?;

        let alerts = self.alerts.read().await.clone();

        Ok(DashboardData {
            timestamp,
            overview,
            performance,
            security,
            user_experience,
            alerts,
            system_health,
        })
    }

    /// Generate overview metrics
    async fn generate_overview_metrics(&self) -> Result<OverviewMetrics, CaptchaError> {
        let summary_24h = self.metrics_collector.generate_summary(24 * 60).await?;
        let active_challenges = self.db_ops.get_active_challenges_count().await
            .map_err(|e| CaptchaError::DatabaseError {
                message: e.to_string(),
                transient: true,
                retry_after: Some(Duration::from_secs(2)),
            })?;

        Ok(OverviewMetrics {
            total_challenges_today: summary_24h.performance.concurrent_challenges,
            success_rate_24h: summary_24h.performance.success_rate,
            bot_detection_rate_24h: summary_24h.bot_detection.accuracy_rate,
            average_response_time_ms: summary_24h.performance.validation_latency_ms,
            active_challenges,
            unique_users_24h: summary_24h.security_events.blocked_ips, // Approximation
            security_incidents_24h: summary_24h.security_events.attack_attempts,
        })
    }

    /// Generate performance chart data
    async fn generate_performance_chart_data(&self) -> Result<PerformanceChartData, CaptchaError> {
        let mut latency_over_time = Vec::new();
        let mut success_rate_over_time = Vec::new();
        let mut throughput_over_time = Vec::new();

        // Generate time series data for the last 24 hours
        let now = SystemTime::now();
        for i in 0..24 {
            let time_point = now - Duration::from_secs(i * 3600);
            let summary = self.metrics_collector.generate_summary(60).await?; // 1 hour window

            latency_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: summary.performance.validation_latency_ms as f64,
                label: None,
            });

            success_rate_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: summary.performance.success_rate,
                label: None,
            });

            throughput_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: summary.performance.concurrent_challenges as f64,
                label: None,
            });
        }

        // Generate difficulty and challenge type distributions
        let difficulty_distribution = self.generate_difficulty_distribution().await?;
        let challenge_type_distribution = self.generate_challenge_type_distribution().await?;

        Ok(PerformanceChartData {
            latency_over_time,
            success_rate_over_time,
            throughput_over_time,
            difficulty_distribution,
            challenge_type_distribution,
        })
    }

    /// Generate security chart data
    async fn generate_security_chart_data(&self) -> Result<SecurityChartData, CaptchaError> {
        let summary = self.metrics_collector.generate_summary(24 * 60).await?;

        let mut bot_detection_over_time = Vec::new();
        let mut attack_attempts_over_time = Vec::new();

        // Generate time series for security metrics
        let now = SystemTime::now();
        for i in 0..24 {
            let time_point = now - Duration::from_secs(i * 3600);
            let hourly_summary = self.metrics_collector.generate_summary(60).await?;

            bot_detection_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.bot_detection.accuracy_rate,
                label: None,
            });

            attack_attempts_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.security_events.attack_attempts as f64,
                label: None,
            });
        }

        let geographic_threat_map = self.generate_geographic_threat_map().await?;
        let top_attacking_ips = self.generate_top_attacking_ips().await?;

        Ok(SecurityChartData {
            threat_level_distribution: summary.security_events.threat_level_distribution,
            bot_detection_over_time,
            attack_attempts_over_time,
            geographic_threat_map,
            top_attacking_ips,
        })
    }

    /// Generate user experience chart data
    async fn generate_user_experience_chart_data(&self) -> Result<UserExperienceChartData, CaptchaError> {
        let summary = self.metrics_collector.generate_summary(24 * 60).await?;

        let mut completion_time_distribution = Vec::new();
        let mut abandonment_rate_over_time = Vec::new();
        let mut accessibility_usage_over_time = Vec::new();
        let mut satisfaction_score_over_time = Vec::new();

        // Generate time series for UX metrics
        let now = SystemTime::now();
        for i in 0..24 {
            let time_point = now - Duration::from_secs(i * 3600);
            let hourly_summary = self.metrics_collector.generate_summary(60).await?;

            completion_time_distribution.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.user_experience.average_completion_time_ms as f64,
                label: None,
            });

            abandonment_rate_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.user_experience.abandonment_rate,
                label: None,
            });

            accessibility_usage_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.user_experience.accessibility_usage_rate,
                label: None,
            });

            satisfaction_score_over_time.push(TimeSeriesPoint {
                timestamp: time_point,
                value: hourly_summary.user_experience.user_satisfaction_score,
                label: None,
            });
        }

        let retry_rate_by_difficulty = self.generate_retry_rate_by_difficulty().await?;

        Ok(UserExperienceChartData {
            completion_time_distribution,
            abandonment_rate_over_time,
            accessibility_usage_over_time,
            satisfaction_score_over_time,
            retry_rate_by_difficulty
        })
    }

    /// Generate system health status
    async fn generate_system_health_status(&self) -> Result<SystemHealthStatus, CaptchaError> {
        let mut components = HashMap::new();

        // Check database health
        let db_health = self.check_database_health().await;
        components.insert("database".to_string(), db_health);

        // Check metrics collector health
        let metrics_health = self.check_metrics_health().await;
        components.insert("metrics".to_string(), metrics_health);

        // Check CAPTCHA service health
        let captcha_health = self.check_captcha_service_health().await;
        components.insert("captcha_service".to_string(), captcha_health);

        // Determine overall status
        let overall_status = self.determine_overall_health(&components);

        Ok(SystemHealthStatus {
            overall_status,
            components,
            uptime_percentage: 99.9, // Would be calculated from actual uptime data
            last_incident: None, // Would be retrieved from incident log
        })
    }

    // Helper methods for generating specific data
    async fn generate_difficulty_distribution(&self) -> Result<HashMap<u8, u64>, CaptchaError> {
        // This would query the database for actual difficulty distribution
        let mut distribution = HashMap::new();
        for difficulty in 1..=10 {
            distribution.insert(difficulty, (difficulty as u64) * 10); // Mock data
        }
        Ok(distribution)
    }

    async fn generate_challenge_type_distribution(&self) -> Result<HashMap<ChallengeType, u64>, CaptchaError> {
        let mut distribution = HashMap::new();
        distribution.insert(ChallengeType::Visual, 150);
        distribution.insert(ChallengeType::Audio, 30);
        distribution.insert(ChallengeType::Behavioral, 80);
        distribution.insert(ChallengeType::Logical, 45);
        distribution.insert(ChallengeType::Hybrid, 25);
        Ok(distribution)
    }

    async fn generate_geographic_threat_map(&self) -> Result<HashMap<String, ThreatData>, CaptchaError> {
        let mut threat_map = HashMap::new();

        // Mock geographic threat data
        threat_map.insert("US".to_string(), ThreatData {
            country_code: "US".to_string(),
            country_name: "United States".to_string(),
            threat_count: 45,
            risk_level: RiskLevel::Medium,
            coordinates: Some((39.8283, -98.5795)),
        });

        threat_map.insert("CN".to_string(), ThreatData {
            country_code: "CN".to_string(),
            country_name: "China".to_string(),
            threat_count: 120,
            risk_level: RiskLevel::High,
            coordinates: Some((35.8617, 104.1954)),
        });

        Ok(threat_map)
    }

    async fn generate_top_attacking_ips(&self) -> Result<Vec<IpThreatInfo>, CaptchaError> {
        // Mock data - would query actual attack data from database
        Ok(vec![
            IpThreatInfo {
                ip_address: "192.168.1.100".to_string(),
                threat_count: 25,
                risk_level: RiskLevel::High,
                country_code: Some("CN".to_string()),
                last_seen: SystemTime::now(),
            },
            IpThreatInfo {
                ip_address: "10.0.0.50".to_string(),
                threat_count: 18,
                risk_level: RiskLevel::Medium,
                country_code: Some("RU".to_string()),
                last_seen: SystemTime::now() - Duration::from_secs(3600),
            },
        ])
    }

    async fn generate_retry_rate_by_difficulty(&self) -> Result<HashMap<u8, f64>, CaptchaError> {
        let mut retry_rates = HashMap::new();
        for difficulty in 1..=10 {
            retry_rates.insert(difficulty, (difficulty as f64) * 0.05); // Mock increasing retry rate
        }
        Ok(retry_rates)
    }

    // Health check methods
    async fn check_database_health(&self) -> ComponentHealth {
        let start_time = std::time::Instant::now();

        // Try to perform a simple database operation
        match self.db_ops.get_active_challenges_count().await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                ComponentHealth {
                    status: if response_time < 100 { HealthStatus::Healthy } else { HealthStatus::Warning },
                    response_time_ms: Some(response_time),
                    error_rate: 0.0,
                    last_check: SystemTime::now(),
                    message: Some("Database connection healthy".to_string()),
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Critical,
                response_time_ms: None,
                error_rate: 1.0,
                last_check: SystemTime::now(),
                message: Some(format!("Database error: {}", e)),
            }
        }
    }

    async fn check_metrics_health(&self) -> ComponentHealth {
        let start_time = std::time::Instant::now();

        match self.metrics_collector.get_real_time_metrics().await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                ComponentHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: Some(response_time),
                    error_rate: 0.0,
                    last_check: SystemTime::now(),
                    message: Some("Metrics collector healthy".to_string()),
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Warning,
                response_time_ms: None,
                error_rate: 0.5,
                last_check: SystemTime::now(),
                message: Some(format!("Metrics collector warning: {}", e)),
            }
        }
    }

    async fn check_captcha_service_health(&self) -> ComponentHealth {
        // This would perform a health check on the CAPTCHA service
        ComponentHealth {
            status: HealthStatus::Healthy,
            response_time_ms: Some(50),
            error_rate: 0.01,
            last_check: SystemTime::now(),
            message: Some("CAPTCHA service operational".to_string()),
        }
    }

    fn determine_overall_health(&self, components: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let mut has_critical = false;
        let mut has_warning = false;

        for component in components.values() {
            match component.status {
                HealthStatus::Critical | HealthStatus::Down => has_critical = true,
                HealthStatus::Warning => has_warning = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_critical {
            HealthStatus::Critical
        } else if has_warning {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    /// Add an alert to the dashboard
    pub async fn add_alert(&self, alert: Alert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);

        // Keep only the last 100 alerts
        let current_len = alerts.len();
        if current_len > 100 {
            alerts.drain(0..current_len - 100);
        }
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), CaptchaError> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(CaptchaError::ValidationFailed {
                message: format!("Alert not found: {}", alert_id),
                attempts_remaining: 0,
                next_difficulty: 1,
            })
        }
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), CaptchaError> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            Ok(())
        } else {
            Err(CaptchaError::ValidationFailed {
                message: format!("Alert not found: {}", alert_id),
                attempts_remaining: 0,
                next_difficulty: 1,
            })
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            data_retention_hours: 24,
            alert_thresholds: AlertThresholds {
                bot_detection_rate_high: 0.15,
                success_rate_low: 0.80,
                average_latency_high_ms: 2000,
                error_rate_high: 0.05,
                concurrent_challenges_high: 1000,
            },
            chart_time_windows: vec![5, 15, 60, 240, 1440], // 5min, 15min, 1h, 4h, 24h
        }
    }
}
