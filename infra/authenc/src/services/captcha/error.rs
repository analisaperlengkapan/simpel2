//! CAPTCHA Error Handling Framework
//!
//! Comprehensive error handling with recovery mechanisms and user-friendly messages

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Comprehensive CAPTCHA error types with recovery information
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum CaptchaError {
    // Challenge-related errors
    #[error("Challenge generation failed: {message}")]
    GenerationFailed {
        message: String,
        recoverable: bool,
        retry_after: Option<Duration>,
    },

    #[error("Challenge validation failed: {message}")]
    ValidationFailed {
        message: String,
        attempts_remaining: u32,
        next_difficulty: u8,
    },

    #[error("Challenge not found: {challenge_id}")]
    ChallengeNotFound {
        challenge_id: String,
        expired: bool,
    },

    #[error("Challenge expired: {challenge_id}")]
    ChallengeExpired {
        challenge_id: String,
        expired_at: String,
    },

    // Service integration errors
    #[error("Secreton service unavailable: {message}")]
    SecreonUnavailable {
        message: String,
        fallback_available: bool,
        retry_after: Option<Duration>,
    },

    #[error("Authenc monitoring unavailable: {message}")]
    MonitoringUnavailable {
        message: String,
        degraded_mode: bool,
    },

    #[error("Database connection error: {message}")]
    DatabaseError {
        message: String,
        transient: bool,
        retry_after: Option<Duration>,
    },

    // Rate limiting and security errors
    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        reset_time: Duration,
        max_attempts: u32,
    },

    #[error("User temporarily locked out")]
    UserLockedOut {
        lockout_duration: Duration,
        reason: String,
    },

    #[error("Suspicious activity detected")]
    SuspiciousActivity {
        risk_level: String,
        additional_verification_required: bool,
    },

    // Accessibility and user experience errors
    #[error("Accessibility feature unavailable: {feature}")]
    AccessibilityUnavailable {
        feature: String,
        alternatives: Vec<String>,
    },

    #[error("Audio challenge generation failed")]
    AudioGenerationFailed {
        fallback_to_visual: bool,
    },

    // Configuration and system errors
    #[error("Invalid configuration: {parameter}")]
    ConfigurationError {
        parameter: String,
        expected: String,
        actual: String,
    },

    #[error("System overloaded")]
    SystemOverloaded {
        retry_after: Duration,
        queue_position: Option<u32>,
    },

    // Network and external service errors
    #[error("Network timeout: {service}")]
    NetworkTimeout {
        service: String,
        timeout_duration: Duration,
        retry_count: u32,
    },

    #[error("External service error: {service}")]
    ExternalServiceError {
        service: String,
        error_code: Option<String>,
        recoverable: bool,
    },
}

/// Error recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry the operation after a delay
    Retry {
        max_attempts: u32,
        delay: Duration,
        exponential_backoff: bool,
    },
    /// Fallback to alternative implementation
    Fallback {
        fallback_type: FallbackType,
        degraded_functionality: bool,
    },
    /// Manual intervention required
    ManualIntervention {
        contact_info: String,
        ticket_id: Option<String>,
    },
    /// Graceful degradation
    GracefulDegradation {
        reduced_security: bool,
        alternative_flow: String,
    },
}

/// Types of fallback mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackType {
    LocalEncryption,
    SimplifiedChallenge,
    ManualVerification,
    AlternativeProvider,
    CachedResponse,
}

/// User-friendly error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserErrorInfo {
    pub title: String,
    pub message: String,
    pub suggested_actions: Vec<String>,
    pub help_link: Option<String>,
    pub contact_support: bool,
    pub error_code: String,
}

/// Error context for debugging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: String,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: String,
    pub component: String,
    pub operation: String,
    pub additional_data: serde_json::Value,
}

/// Error recovery result
#[derive(Debug, Clone)]
pub enum RecoveryResult<T> {
    /// Operation succeeded after recovery
    Recovered(T),
    /// Recovery failed, but fallback succeeded
    FallbackSucceeded(T),
    /// All recovery attempts failed
    Failed(CaptchaError),
    /// Manual intervention required
    RequiresIntervention(CaptchaError),
}

impl CaptchaError {
    /// Get recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            CaptchaError::GenerationFailed { recoverable, retry_after, .. } => {
                if *recoverable {
                    RecoveryStrategy::Retry {
                        max_attempts: 3,
                        delay: retry_after.unwrap_or(Duration::from_secs(1)),
                        exponential_backoff: true,
                    }
                } else {
                    RecoveryStrategy::Fallback {
                        fallback_type: FallbackType::SimplifiedChallenge,
                        degraded_functionality: true,
                    }
                }
            }
            CaptchaError::SecreonUnavailable { fallback_available, .. } => {
                if *fallback_available {
                    RecoveryStrategy::Fallback {
                        fallback_type: FallbackType::LocalEncryption,
                        degraded_functionality: false,
                    }
                } else {
                    RecoveryStrategy::GracefulDegradation {
                        reduced_security: true,
                        alternative_flow: "manual_verification".to_string(),
                    }
                }
            }
            CaptchaError::MonitoringUnavailable { .. } => {
                RecoveryStrategy::Fallback {
                    fallback_type: FallbackType::CachedResponse,
                    degraded_functionality: true,
                }
            }
            CaptchaError::DatabaseError { transient, retry_after, .. } => {
                if *transient {
                    RecoveryStrategy::Retry {
                        max_attempts: 5,
                        delay: retry_after.unwrap_or(Duration::from_secs(2)),
                        exponential_backoff: true,
                    }
                } else {
                    RecoveryStrategy::ManualIntervention {
                        contact_info: "support@example.com".to_string(),
                        ticket_id: None,
                    }
                }
            }
            CaptchaError::RateLimitExceeded { reset_time, .. } => {
                RecoveryStrategy::Retry {
                    max_attempts: 1,
                    delay: *reset_time,
                    exponential_backoff: false,
                }
            }
            CaptchaError::AccessibilityUnavailable { alternatives, .. } => {
                if !alternatives.is_empty() {
                    RecoveryStrategy::Fallback {
                        fallback_type: FallbackType::AlternativeProvider,
                        degraded_functionality: false,
                    }
                } else {
                    RecoveryStrategy::ManualIntervention {
                        contact_info: "accessibility@example.com".to_string(),
                        ticket_id: None,
                    }
                }
            }
            CaptchaError::AudioGenerationFailed { fallback_to_visual } => {
                if *fallback_to_visual {
                    RecoveryStrategy::Fallback {
                        fallback_type: FallbackType::AlternativeProvider,
                        degraded_functionality: false,
                    }
                } else {
                    RecoveryStrategy::ManualIntervention {
                        contact_info: "accessibility@example.com".to_string(),
                        ticket_id: None,
                    }
                }
            }
            CaptchaError::SystemOverloaded { retry_after, .. } => {
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    delay: *retry_after,
                    exponential_backoff: false,
                }
            }
            _ => RecoveryStrategy::ManualIntervention {
                contact_info: "support@example.com".to_string(),
                ticket_id: None,
            },
        }
    }

    /// Get user-friendly error information
    pub fn user_info(&self) -> UserErrorInfo {
        match self {
            CaptchaError::GenerationFailed { .. } => UserErrorInfo {
                title: "Challenge Generation Error".to_string(),
                message: "We're having trouble creating your security challenge. Please try again in a moment.".to_string(),
                suggested_actions: vec![
                    "Refresh the page and try again".to_string(),
                    "Check your internet connection".to_string(),
                    "Try a different browser if the problem persists".to_string(),
                ],
                help_link: Some("/help/captcha-issues".to_string()),
                contact_support: false,
                error_code: "CAPTCHA_GEN_001".to_string(),
            },
            CaptchaError::ValidationFailed { attempts_remaining, .. } => UserErrorInfo {
                title: "Incorrect Answer".to_string(),
                message: format!("The answer you provided is incorrect. You have {} attempts remaining.", attempts_remaining),
                suggested_actions: vec![
                    "Look carefully at the challenge and try again".to_string(),
                    "Use the audio option if you're having trouble with the visual challenge".to_string(),
                    "Request a new challenge if this one is unclear".to_string(),
                ],
                help_link: Some("/help/captcha-tips".to_string()),
                contact_support: *attempts_remaining == 0,
                error_code: "CAPTCHA_VAL_001".to_string(),
            },
            CaptchaError::RateLimitExceeded { reset_time, .. } => UserErrorInfo {
                title: "Too Many Attempts".to_string(),
                message: format!("You've made too many attempts. Please wait {} seconds before trying again.", reset_time.as_secs()),
                suggested_actions: vec![
                    format!("Wait {} seconds and try again", reset_time.as_secs()),
                    "Make sure you're entering the correct answer".to_string(),
                    "Contact support if you continue having issues".to_string(),
                ],
                help_link: Some("/help/rate-limits".to_string()),
                contact_support: false,
                error_code: "CAPTCHA_RATE_001".to_string(),
            },
            CaptchaError::UserLockedOut { lockout_duration, reason } => UserErrorInfo {
                title: "Account Temporarily Locked".to_string(),
                message: format!("Your account has been temporarily locked for {} minutes due to {}.", lockout_duration.as_secs() / 60, reason),
                suggested_actions: vec![
                    format!("Wait {} minutes before trying again", lockout_duration.as_secs() / 60),
                    "Contact support if you believe this is an error".to_string(),
                ],
                help_link: Some("/help/account-locked".to_string()),
                contact_support: true,
                error_code: "CAPTCHA_LOCK_001".to_string(),
            },
            CaptchaError::AccessibilityUnavailable { feature, alternatives } => UserErrorInfo {
                title: "Accessibility Feature Unavailable".to_string(),
                message: format!("The {} accessibility feature is currently unavailable.", feature),
                suggested_actions: if !alternatives.is_empty() {
                    let mut actions = vec![format!("Try these alternatives: {}", alternatives.join(", "))];
                    actions.push("Contact support for additional assistance".to_string());
                    actions
                } else {
                    vec!["Contact support for alternative verification methods".to_string()]
                },
                help_link: Some("/help/accessibility".to_string()),
                contact_support: true,
                error_code: "CAPTCHA_ACC_001".to_string(),
            },
            CaptchaError::SystemOverloaded { retry_after, queue_position } => UserErrorInfo {
                title: "System Busy".to_string(),
                message: if let Some(position) = queue_position {
                    format!("The system is currently busy. You are number {} in the queue.", position)
                } else {
                    "The system is currently experiencing high load. Please try again shortly.".to_string()
                },
                suggested_actions: vec![
                    format!("Please wait {} seconds and try again", retry_after.as_secs()),
                    "Try again during off-peak hours".to_string(),
                ],
                help_link: Some("/help/system-status".to_string()),
                contact_support: false,
                error_code: "CAPTCHA_SYS_001".to_string(),
            },
            _ => UserErrorInfo {
                title: "Unexpected Error".to_string(),
                message: "An unexpected error occurred. Please try again or contact support.".to_string(),
                suggested_actions: vec![
                    "Refresh the page and try again".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                help_link: Some("/help/general".to_string()),
                contact_support: true,
                error_code: "CAPTCHA_UNK_001".to_string(),
            },
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CaptchaError::GenerationFailed { recoverable: true, .. } |
            CaptchaError::SecreonUnavailable { fallback_available: true, .. } |
            CaptchaError::DatabaseError { transient: true, .. } |
            CaptchaError::NetworkTimeout { .. } |
            CaptchaError::SystemOverloaded { .. }
        )
    }

    /// Get retry delay if applicable
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            CaptchaError::GenerationFailed { retry_after, .. } => *retry_after,
            CaptchaError::SecreonUnavailable { retry_after, .. } => *retry_after,
            CaptchaError::DatabaseError { retry_after, .. } => *retry_after,
            CaptchaError::RateLimitExceeded { reset_time, .. } => Some(*reset_time),
            CaptchaError::SystemOverloaded { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }
}

/// Error recovery executor
pub struct ErrorRecovery {
    max_retry_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self {
            max_retry_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl ErrorRecovery {
    pub fn new(max_retry_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retry_attempts,
            base_delay,
            max_delay,
        }
    }

    /// Execute operation with automatic retry and recovery
    pub async fn execute_with_recovery<T, F, Fut>(
        &self,
        mut operation: F,
        context: ErrorContext,
    ) -> RecoveryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, CaptchaError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retry_attempts {
            match operation().await {
                Ok(result) => {
                    if attempts > 0 {
                        return RecoveryResult::Recovered(result);
                    } else {
                        return RecoveryResult::Recovered(result);
                    }
                }
                Err(error) => {
                    attempts += 1;
                    last_error = Some(error.clone());

                    if !error.is_recoverable() {
                        break;
                    }

                    if let Some(delay) = error.retry_delay() {
                        tokio::time::sleep(delay).await;
                    } else {
                        let delay = self.calculate_backoff_delay(attempts);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // If we get here, all retries failed
        if let Some(error) = last_error {
            match error.recovery_strategy() {
                RecoveryStrategy::Fallback { .. } => {
                    // Attempt fallback mechanism
                    RecoveryResult::RequiresIntervention(error)
                }
                RecoveryStrategy::ManualIntervention { .. } => {
                    RecoveryResult::RequiresIntervention(error)
                }
                _ => RecoveryResult::Failed(error),
            }
        } else {
            RecoveryResult::Failed(CaptchaError::GenerationFailed {
                message: "Unknown error during recovery".to_string(),
                recoverable: false,
                retry_after: None,
            })
        }
    }

    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.base_delay.as_millis() * (2_u128.pow(attempt - 1));
        let delay = Duration::from_millis(delay_ms.min(self.max_delay.as_millis()) as u64);
        delay
    }
}

/// Helper trait for converting standard errors to CAPTCHA errors
pub trait IntoCaptchaError {
    fn into_captcha_error(self, context: &str) -> CaptchaError;
}

impl IntoCaptchaError for tokio_postgres::Error {
    fn into_captcha_error(self, context: &str) -> CaptchaError {
        let transient = self.is_closed() || format!("{:?}", self).contains("timeout") || format!("{:?}", self).contains("connection");

        CaptchaError::DatabaseError {
            message: format!("{}: {}", context, self),
            transient,
            retry_after: if transient {
                Some(Duration::from_secs(2))
            } else {
                None
            },
        }
    }
}

impl IntoCaptchaError for reqwest::Error {
    fn into_captcha_error(self, context: &str) -> CaptchaError {
        if self.is_timeout() {
            CaptchaError::NetworkTimeout {
                service: context.to_string(),
                timeout_duration: Duration::from_secs(30), // Default timeout
                retry_count: 0,
            }
        } else {
            CaptchaError::ExternalServiceError {
                service: context.to_string(),
                error_code: self.status().map(|s| s.to_string()),
                recoverable: self.is_connect() || self.is_timeout(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_strategy() {
        let error = CaptchaError::GenerationFailed {
            message: "Test error".to_string(),
            recoverable: true,
            retry_after: Some(Duration::from_secs(1)),
        };

        match error.recovery_strategy() {
            RecoveryStrategy::Retry { max_attempts, delay, .. } => {
                assert_eq!(max_attempts, 3);
                assert_eq!(delay, Duration::from_secs(1));
            }
            _ => panic!("Expected retry strategy"),
        }
    }

    #[test]
    fn test_user_error_info() {
        let error = CaptchaError::ValidationFailed {
            message: "Wrong answer".to_string(),
            attempts_remaining: 2,
            next_difficulty: 3,
        };

        let info = error.user_info();
        assert_eq!(info.title, "Incorrect Answer");
        assert!(info.message.contains("2 attempts remaining"));
        assert_eq!(info.error_code, "CAPTCHA_VAL_001");
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable_error = CaptchaError::GenerationFailed {
            message: "Test".to_string(),
            recoverable: true,
            retry_after: None,
        };

        let non_recoverable_error = CaptchaError::UserLockedOut {
            lockout_duration: Duration::from_secs(300),
            reason: "Too many failures".to_string(),
        };

        assert!(recoverable_error.is_recoverable());
        assert!(!non_recoverable_error.is_recoverable());
    }
}
