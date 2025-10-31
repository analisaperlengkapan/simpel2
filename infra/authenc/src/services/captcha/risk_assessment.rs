//! Risk Assessment Service for CAPTCHA
//!
//! Determines when CAPTCHA should be required based on risk factors

use crate::services::captcha::{CaptchaError, RiskLevel};
use std::net::IpAddr;
use std::str::FromStr;

/// Risk assessment configuration
#[derive(Debug, Clone)]
pub struct RiskAssessmentConfig {
    /// Threshold for requiring CAPTCHA (0.0 - 1.0)
    pub captcha_threshold: f64,
    /// Weight for failed login attempts
    pub failed_attempts_weight: f64,
    /// Weight for suspicious IP patterns
    pub ip_reputation_weight: f64,
    /// Weight for unusual timing patterns
    pub timing_pattern_weight: f64,
    /// Weight for geographic anomalies
    pub geographic_weight: f64,
}

impl Default for RiskAssessmentConfig {
    fn default() -> Self {
        Self {
            captcha_threshold: 0.5, // Require CAPTCHA if risk > 50%
            failed_attempts_weight: 0.4,
            ip_reputation_weight: 0.3,
            timing_pattern_weight: 0.2,
            geographic_weight: 0.1,
        }
    }
}

/// Risk assessment factors
#[derive(Debug, Clone, Default)]
pub struct RiskFactors {
    /// Number of recent failed login attempts
    pub failed_attempts: u32,
    /// IP reputation score (0.0 = good, 1.0 = bad)
    pub ip_reputation: f64,
    /// Timing pattern anomaly score (0.0 = normal, 1.0 = suspicious)
    pub timing_anomaly: f64,
    /// Geographic anomaly score (0.0 = normal, 1.0 = suspicious)
    pub geographic_anomaly: f64,
    /// User agent anomaly score (0.0 = normal, 1.0 = suspicious)
    pub user_agent_anomaly: f64,
}

/// Risk assessment service
pub struct RiskAssessmentService {
    config: RiskAssessmentConfig,
}

impl RiskAssessmentService {
    /// Create new risk assessment service
    pub fn new(config: RiskAssessmentConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(RiskAssessmentConfig::default())
    }

    /// Calculate overall risk score
    pub fn calculate_risk_score(&self, factors: &RiskFactors) -> f64 {
        let mut score = 0.0;

        // Failed attempts contribution (exponential growth)
        let failed_attempts_score = if factors.failed_attempts == 0 {
            0.0
        } else {
            (factors.failed_attempts as f64 / 10.0).min(1.0)
        };
        score += failed_attempts_score * self.config.failed_attempts_weight;

        // IP reputation contribution
        score += factors.ip_reputation * self.config.ip_reputation_weight;

        // Timing pattern contribution
        score += factors.timing_anomaly * self.config.timing_pattern_weight;

        // Geographic anomaly contribution
        score += factors.geographic_anomaly * self.config.geographic_weight;

        // Normalize to 0.0 - 1.0 range
        score.min(1.0).max(0.0)
    }

    /// Determine if CAPTCHA should be required
    pub fn should_require_captcha(&self, factors: &RiskFactors) -> bool {
        let risk_score = self.calculate_risk_score(factors);
        risk_score >= self.config.captcha_threshold
    }

    /// Convert risk score to risk level
    pub fn risk_score_to_level(&self, score: f64) -> RiskLevel {
        if score < 0.3 {
            RiskLevel::Low
        } else if score < 0.6 {
            RiskLevel::Medium
        } else if score < 0.9 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    /// Assess risk for login attempt
    pub async fn assess_login_risk(
        &self,
        username: &str,
        ip_address: &str,
        user_agent: Option<&str>,
    ) -> Result<(f64, bool), CaptchaError> {
        // Collect risk factors
        let factors = self.collect_risk_factors(username, ip_address, user_agent).await?;

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&factors);

        // Determine if CAPTCHA is required
        let requires_captcha = self.should_require_captcha(&factors);

        Ok((risk_score, requires_captcha))
    }

    /// Assess risk for MFA setup
    pub async fn assess_mfa_setup_risk(
        &self,
        user_id: &str,
        ip_address: &str,
    ) -> Result<f64, CaptchaError> {
        // For MFA setup, we're more cautious
        let mut factors = RiskFactors::default();

        // Check IP reputation
        factors.ip_reputation = self.check_ip_reputation(ip_address).await;

        // Check for geographic anomalies
        factors.geographic_anomaly = self.check_geographic_anomaly(user_id, ip_address).await;

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&factors);

        Ok(risk_score)
    }

    /// Assess risk for password reset
    pub async fn assess_password_reset_risk(
        &self,
        email: &str,
        ip_address: &str,
    ) -> Result<f64, CaptchaError> {
        // Password reset is always high risk - require CAPTCHA
        // But still calculate actual risk for logging
        let mut factors = RiskFactors::default();

        // Check IP reputation
        factors.ip_reputation = self.check_ip_reputation(ip_address).await;

        // Check for timing anomalies (multiple reset requests)
        factors.timing_anomaly = self.check_reset_timing_anomaly(email).await;

        // Calculate risk score (but always return high enough to require CAPTCHA)
        let risk_score = self.calculate_risk_score(&factors);

        // Ensure minimum risk score of 0.6 for password reset
        Ok(risk_score.max(0.6))
    }

    /// Collect risk factors for assessment
    async fn collect_risk_factors(
        &self,
        username: &str,
        ip_address: &str,
        user_agent: Option<&str>,
    ) -> Result<RiskFactors, CaptchaError> {
        let mut factors = RiskFactors::default();

        // Check failed login attempts (would query database in production)
        factors.failed_attempts = self.get_failed_attempts(username).await;

        // Check IP reputation
        factors.ip_reputation = self.check_ip_reputation(ip_address).await;

        // Check timing patterns
        factors.timing_anomaly = self.check_timing_anomaly(username).await;

        // Check geographic anomalies
        factors.geographic_anomaly = self.check_geographic_anomaly(username, ip_address).await;

        // Check user agent anomalies
        if let Some(ua) = user_agent {
            factors.user_agent_anomaly = self.check_user_agent_anomaly(ua);
        }

        Ok(factors)
    }

    /// Get number of recent failed login attempts
    async fn get_failed_attempts(&self, _username: &str) -> u32 {
        // TODO: Query database for failed attempts in last hour
        // For now, return 0 (low risk)
        0
    }

    /// Check IP reputation
    async fn check_ip_reputation(&self, ip_address: &str) -> f64 {
        // Parse IP address
        let ip = match IpAddr::from_str(ip_address) {
            Ok(ip) => ip,
            Err(_) => return 0.5, // Unknown IP format = medium risk
        };

        // Check if IP is in known bad ranges
        match ip {
            IpAddr::V4(ipv4) => {
                // Check for localhost/private IPs (low risk in development)
                if ipv4.is_loopback() || ipv4.is_private() {
                    return 0.0;
                }

                // TODO: Check against IP reputation database
                // For now, return low risk for all public IPs
                0.1
            }
            IpAddr::V6(ipv6) => {
                // Check for localhost
                if ipv6.is_loopback() {
                    return 0.0;
                }

                // TODO: Check against IP reputation database
                0.1
            }
        }
    }

    /// Check for timing anomalies
    async fn check_timing_anomaly(&self, _username: &str) -> f64 {
        // TODO: Analyze login attempt timing patterns
        // - Multiple rapid attempts = suspicious
        // - Unusual time of day = slightly suspicious
        // For now, return low risk
        0.0
    }

    /// Check for geographic anomalies
    async fn check_geographic_anomaly(&self, _user_id: &str, _ip_address: &str) -> f64 {
        // TODO: Check if IP location differs significantly from user's usual location
        // - Same country = low risk
        // - Different country = medium risk
        // - Impossible travel = high risk
        // For now, return low risk
        0.0
    }

    /// Check for password reset timing anomalies
    async fn check_reset_timing_anomaly(&self, _email: &str) -> f64 {
        // TODO: Check for multiple reset requests in short time
        // For now, return medium risk
        0.3
    }

    /// Check user agent for anomalies
    fn check_user_agent_anomaly(&self, user_agent: &str) -> f64 {
        // Check for suspicious patterns
        let suspicious_patterns = [
            "bot",
            "crawler",
            "spider",
            "scraper",
            "curl",
            "wget",
            "python",
            "java",
        ];

        let ua_lower = user_agent.to_lowercase();
        for pattern in &suspicious_patterns {
            if ua_lower.contains(pattern) {
                return 0.8; // High risk for bot-like user agents
            }
        }

        // Check for missing or very short user agent
        if user_agent.len() < 20 {
            return 0.5; // Medium risk
        }

        0.0 // Low risk for normal user agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_score_calculation() {
        let service = RiskAssessmentService::default();

        // Low risk scenario
        let low_risk = RiskFactors {
            failed_attempts: 0,
            ip_reputation: 0.0,
            timing_anomaly: 0.0,
            geographic_anomaly: 0.0,
            user_agent_anomaly: 0.0,
        };
        assert!(service.calculate_risk_score(&low_risk) < 0.1);

        // High risk scenario
        let high_risk = RiskFactors {
            failed_attempts: 5,
            ip_reputation: 0.8,
            timing_anomaly: 0.7,
            geographic_anomaly: 0.6,
            user_agent_anomaly: 0.9,
        };
        assert!(service.calculate_risk_score(&high_risk) > 0.5);
    }

    #[test]
    fn test_captcha_requirement() {
        let service = RiskAssessmentService::default();

        // Should not require CAPTCHA for low risk
        let low_risk = RiskFactors {
            failed_attempts: 0,
            ip_reputation: 0.1,
            timing_anomaly: 0.0,
            geographic_anomaly: 0.0,
            user_agent_anomaly: 0.0,
        };
        assert!(!service.should_require_captcha(&low_risk));

        // Should require CAPTCHA for high risk
        let high_risk = RiskFactors {
            failed_attempts: 5,
            ip_reputation: 0.8,
            timing_anomaly: 0.7,
            geographic_anomaly: 0.6,
            user_agent_anomaly: 0.9,
        };
        assert!(service.should_require_captcha(&high_risk));
    }

    #[test]
    fn test_risk_level_conversion() {
        let service = RiskAssessmentService::default();

        assert_eq!(service.risk_score_to_level(0.1), RiskLevel::Low);
        assert_eq!(service.risk_score_to_level(0.4), RiskLevel::Medium);
        assert_eq!(service.risk_score_to_level(0.7), RiskLevel::High);
        assert_eq!(service.risk_score_to_level(0.95), RiskLevel::Critical);
    }

    #[test]
    fn test_user_agent_anomaly_detection() {
        let service = RiskAssessmentService::default();

        // Normal user agents
        assert!(service.check_user_agent_anomaly("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36") < 0.3);

        // Suspicious user agents
        assert!(service.check_user_agent_anomaly("python-requests/2.28.0") > 0.5);
        assert!(service.check_user_agent_anomaly("curl/7.68.0") > 0.5);
        assert!(service.check_user_agent_anomaly("bot") > 0.5);

        // Short/missing user agent
        assert!(service.check_user_agent_anomaly("short") > 0.3);
    }
}
