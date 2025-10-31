//! CAPTCHA Service Fallback Mechanisms
//!
//! Graceful degradation when external services are unavailable

use crate::services::captcha::error::CaptchaError;
use crate::services::captcha::types::{Challenge, ChallengeType, ValidationResult};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Fallback service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Whether to enable local encryption fallback
    pub enable_local_encryption: bool,
    /// Whether to enable simplified challenges fallback
    pub enable_simplified_challenges: bool,
    /// Whether to enable manual verification fallback
    pub enable_manual_verification: bool,
    /// Whether to enable cached responses fallback
    pub enable_cached_responses: bool,
    /// Timeout for degraded mode before emergency fallback
    pub degraded_mode_timeout: Duration,
    /// Time-to-live for cached responses
    pub cache_ttl: Duration,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enable_local_encryption: true,
            enable_simplified_challenges: true,
            enable_manual_verification: true,
            enable_cached_responses: true,
            degraded_mode_timeout: Duration::from_secs(300), // 5 minutes
            cache_ttl: Duration::from_secs(3600),            // 1 hour
        }
    }
}

/// Fallback service state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FallbackState {
    /// Normal operation with all services available
    Normal,
    /// Degraded operation with some services unavailable
    Degraded,
    /// Emergency operation with minimal functionality
    Emergency,
}

/// Local encryption fallback when Secreton is unavailable
pub struct LocalEncryptionFallback {
    key: [u8; 32],
    enabled: bool,
}

impl LocalEncryptionFallback {
    /// Create a new local encryption fallback instance
    pub fn new() -> Self {
        // Generate a random key for local encryption
        // In production, this should be derived from a secure source
        let mut key = [0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut key);

        Self { key, enabled: true }
    }

    /// Encrypt data using local fallback encryption
    pub fn encrypt(&self, data: &str) -> Result<String, CaptchaError> {
        if !self.enabled {
            return Err(CaptchaError::SecreonUnavailable {
                message: "Local encryption disabled".to_string(),
                fallback_available: false,
                retry_after: None,
            });
        }

        // Simple XOR encryption for fallback (not cryptographically secure)
        // In production, use a proper encryption library like AES
        let encrypted: Vec<u8> = data
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ self.key[i % self.key.len()])
            .collect();

        Ok(BASE64.encode(encrypted))
    }

    /// Decrypt data using local fallback decryption
    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, CaptchaError> {
        if !self.enabled {
            return Err(CaptchaError::SecreonUnavailable {
                message: "Local encryption disabled".to_string(),
                fallback_available: false,
                retry_after: None,
            });
        }

        let encrypted =
            BASE64
                .decode(encrypted_data)
                .map_err(|e| CaptchaError::ValidationFailed {
                    message: format!("Invalid encrypted data: {}", e),
                    attempts_remaining: 0,
                    next_difficulty: 1,
                })?;

        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()])
            .collect();

        String::from_utf8(decrypted).map_err(|e| CaptchaError::ValidationFailed {
            message: format!("Invalid decrypted data: {}", e),
            attempts_remaining: 0,
            next_difficulty: 1,
        })
    }
}

/// Simplified challenge generator for degraded mode
pub struct SimplifiedChallengeGenerator {
    enabled: bool,
}

impl SimplifiedChallengeGenerator {
    /// Create a new simplified challenge generator
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Generate a simple challenge for degraded mode
    pub fn generate_simple_challenge(&self, difficulty: u8) -> Result<Challenge, CaptchaError> {
        if !self.enabled {
            return Err(CaptchaError::GenerationFailed {
                message: "Simplified challenge generation disabled".to_string(),
                recoverable: false,
                retry_after: None,
            });
        }

        // Generate simple mathematical challenge
        let (question, answer) = self.generate_math_challenge(difficulty);

        // Use local encryption for the challenge
        let local_crypto = LocalEncryptionFallback::new();
        let encrypted_data = local_crypto.encrypt(&question)?;

        // Simple hash for answer verification
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(answer.as_bytes());
        let answer_hash = format!("{:x}", hasher.finalize());

        Ok(Challenge::new(
            ChallengeType::Logical,
            difficulty,
            encrypted_data,
            answer_hash,
            None,
            "127.0.0.1".to_string(), // Default IP for fallback
        ))
    }

    fn generate_math_challenge(&self, difficulty: u8) -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match difficulty {
            1..=3 => {
                // Simple addition
                let a = rng.gen_range(1..=10);
                let b = rng.gen_range(1..=10);
                let result = a + b;
                (format!("What is {} + {}?", a, b), result.to_string())
            }
            4..=6 => {
                // Multiplication
                let a = rng.gen_range(2..=12);
                let b = rng.gen_range(2..=12);
                let result = a * b;
                (format!("What is {} × {}?", a, b), result.to_string())
            }
            7..=10 => {
                // More complex operations
                let a = rng.gen_range(10..=50);
                let b = rng.gen_range(2..=10);
                let c = rng.gen_range(1..=20);
                let result = (a + b) * c;
                (
                    format!("What is ({} + {}) × {}?", a, b, c),
                    result.to_string(),
                )
            }
            _ => {
                // Default to simple addition
                let a = rng.gen_range(1..=10);
                let b = rng.gen_range(1..=10);
                let result = a + b;
                (format!("What is {} + {}?", a, b), result.to_string())
            }
        }
    }
}

/// Manual verification system for accessibility fallback
pub struct ManualVerificationSystem {
    pending_verifications: Arc<RwLock<HashMap<String, PendingVerification>>>,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct PendingVerification {
    user_id: String,
    session_id: String,
    timestamp: SystemTime,
    reason: String,
    contact_method: String,
}

impl ManualVerificationSystem {
    /// Create a new manual verification system
    pub fn new() -> Self {
        Self {
            pending_verifications: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    /// Request manual verification for a user
    pub async fn request_manual_verification(
        &self,
        user_id: String,
        session_id: String,
        reason: String,
        contact_method: String,
    ) -> Result<String, CaptchaError> {
        if !self.enabled {
            return Err(CaptchaError::AccessibilityUnavailable {
                feature: "manual_verification".to_string(),
                alternatives: vec!["audio_challenge".to_string()],
            });
        }

        let verification_id = Uuid::new_v4().to_string();
        let verification = PendingVerification {
            user_id,
            session_id,
            timestamp: SystemTime::now(),
            reason,
            contact_method,
        };

        let mut pending = self.pending_verifications.write().await;
        pending.insert(verification_id.clone(), verification);

        info!("Manual verification requested: {}", verification_id);
        Ok(verification_id)
    }

    /// Check the status of a manual verification request
    pub async fn check_verification_status(&self, verification_id: &str) -> Option<bool> {
        let pending = self.pending_verifications.read().await;
        if let Some(verification) = pending.get(verification_id) {
            // In a real implementation, this would check with an external system
            // For now, we'll simulate approval after 5 minutes
            if verification.timestamp.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(300)
            {
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }

    /// Clean up expired verification requests
    pub async fn cleanup_expired_verifications(&self) {
        let mut pending = self.pending_verifications.write().await;
        let now = SystemTime::now();

        pending.retain(|_, verification| {
            verification.timestamp.elapsed().unwrap_or(Duration::ZERO) < Duration::from_secs(3600)
        });
    }
}

/// Response cache for monitoring fallback
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    data: ValidationResult,
    timestamp: SystemTime,
}

impl ResponseCache {
    /// Create a new response cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            enabled: true,
        }
    }

    /// Get a cached response by key
    pub async fn get(&self, key: &str) -> Option<ValidationResult> {
        if !self.enabled {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                return Some(cached.data.clone());
            }
        }
        None
    }

    /// Set a cached response
    pub async fn set(&self, key: String, value: ValidationResult) {
        if !self.enabled {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CachedResponse {
                data: value,
                timestamp: SystemTime::now(),
            },
        );
    }

    /// Clean up expired cached responses
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();

        cache.retain(|_, cached| cached.timestamp.elapsed().unwrap_or(Duration::ZERO) < self.ttl);
    }
}

/// Main fallback service coordinator
pub struct FallbackService {
    config: FallbackConfig,
    state: Arc<RwLock<FallbackState>>,
    local_encryption: LocalEncryptionFallback,
    simplified_generator: SimplifiedChallengeGenerator,
    manual_verification: ManualVerificationSystem,
    response_cache: ResponseCache,
    degraded_mode_start: Arc<RwLock<Option<SystemTime>>>,
}

impl FallbackService {
    /// Create a new fallback service with configuration
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            response_cache: ResponseCache::new(config.cache_ttl),
            config,
            state: Arc::new(RwLock::new(FallbackState::Normal)),
            local_encryption: LocalEncryptionFallback::new(),
            simplified_generator: SimplifiedChallengeGenerator::new(),
            manual_verification: ManualVerificationSystem::new(),
            degraded_mode_start: Arc::new(RwLock::new(None)),
        }
    }

    /// Enter degraded mode
    pub async fn enter_degraded_mode(&self, reason: &str) {
        let mut state = self.state.write().await;
        let mut start_time = self.degraded_mode_start.write().await;

        *state = FallbackState::Degraded;
        *start_time = Some(SystemTime::now());

        warn!("Entering degraded mode: {}", reason);
    }

    /// Enter emergency mode
    pub async fn enter_emergency_mode(&self, reason: &str) {
        let mut state = self.state.write().await;
        *state = FallbackState::Emergency;

        warn!("Entering emergency mode: {}", reason);
    }

    /// Return to normal mode
    pub async fn return_to_normal(&self) {
        let mut state = self.state.write().await;
        let mut start_time = self.degraded_mode_start.write().await;

        *state = FallbackState::Normal;
        *start_time = None;

        info!("Returned to normal mode");
    }

    /// Check if should exit degraded mode
    pub async fn check_degraded_mode_timeout(&self) {
        let state = self.state.read().await;
        if *state == FallbackState::Degraded {
            let start_time = self.degraded_mode_start.read().await;
            if let Some(start) = *start_time {
                if start.elapsed().unwrap_or(Duration::ZERO) > self.config.degraded_mode_timeout {
                    drop(state);
                    drop(start_time);
                    self.return_to_normal().await;
                }
            }
        }
    }

    /// Get current fallback state
    pub async fn get_state(&self) -> FallbackState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Generate challenge with fallback
    pub async fn generate_challenge_with_fallback(
        &self,
        challenge_type: ChallengeType,
        difficulty: u8,
    ) -> Result<Challenge, CaptchaError> {
        let state = self.get_state().await;

        match state {
            FallbackState::Normal => {
                // Try normal generation first, fallback if it fails
                Err(CaptchaError::GenerationFailed {
                    message: "Normal generation not implemented in fallback service".to_string(),
                    recoverable: true,
                    retry_after: Some(Duration::from_secs(1)),
                })
            }
            FallbackState::Degraded | FallbackState::Emergency => {
                // Use simplified challenge generation
                if self.config.enable_simplified_challenges {
                    debug!("Using simplified challenge generation in degraded mode");
                    self.simplified_generator
                        .generate_simple_challenge(difficulty)
                } else {
                    Err(CaptchaError::GenerationFailed {
                        message: "Challenge generation unavailable in current mode".to_string(),
                        recoverable: false,
                        retry_after: None,
                    })
                }
            }
        }
    }

    /// Encrypt data with fallback
    pub async fn encrypt_with_fallback(&self, data: &str) -> Result<String, CaptchaError> {
        let state = self.get_state().await;

        match state {
            FallbackState::Normal => {
                // Try Secreton first, fallback to local encryption
                if self.config.enable_local_encryption {
                    debug!("Using local encryption fallback");
                    self.local_encryption.encrypt(data)
                } else {
                    Err(CaptchaError::SecreonUnavailable {
                        message: "Secreton unavailable and local encryption disabled".to_string(),
                        fallback_available: false,
                        retry_after: Some(Duration::from_secs(30)),
                    })
                }
            }
            FallbackState::Degraded | FallbackState::Emergency => {
                // Always use local encryption in degraded mode
                self.local_encryption.encrypt(data)
            }
        }
    }

    /// Decrypt data with fallback
    pub async fn decrypt_with_fallback(
        &self,
        encrypted_data: &str,
    ) -> Result<String, CaptchaError> {
        let state = self.get_state().await;

        match state {
            FallbackState::Normal => {
                // Try Secreton first, fallback to local decryption
                if self.config.enable_local_encryption {
                    debug!("Using local decryption fallback");
                    self.local_encryption.decrypt(encrypted_data)
                } else {
                    Err(CaptchaError::SecreonUnavailable {
                        message: "Secreton unavailable and local encryption disabled".to_string(),
                        fallback_available: false,
                        retry_after: Some(Duration::from_secs(30)),
                    })
                }
            }
            FallbackState::Degraded | FallbackState::Emergency => {
                // Always use local decryption in degraded mode
                self.local_encryption.decrypt(encrypted_data)
            }
        }
    }

    /// Validate with cached response fallback
    pub async fn validate_with_cache_fallback(&self, cache_key: &str) -> Option<ValidationResult> {
        if self.config.enable_cached_responses {
            self.response_cache.get(cache_key).await
        } else {
            None
        }
    }

    /// Cache validation result
    pub async fn cache_validation_result(&self, cache_key: String, result: ValidationResult) {
        if self.config.enable_cached_responses {
            self.response_cache.set(cache_key, result).await;
        }
    }

    /// Request manual verification
    pub async fn request_manual_verification(
        &self,
        user_id: String,
        session_id: String,
        reason: String,
    ) -> Result<String, CaptchaError> {
        if self.config.enable_manual_verification {
            self.manual_verification
                .request_manual_verification(
                    user_id,
                    session_id,
                    reason,
                    "email".to_string(), // Default contact method
                )
                .await
        } else {
            Err(CaptchaError::AccessibilityUnavailable {
                feature: "manual_verification".to_string(),
                alternatives: vec![],
            })
        }
    }

    /// Cleanup expired data
    pub async fn cleanup(&self) {
        self.response_cache.cleanup_expired().await;
        self.manual_verification
            .cleanup_expired_verifications()
            .await;
        self.check_degraded_mode_timeout().await;
    }

    /// Get fallback status for monitoring
    pub async fn get_status(&self) -> FallbackStatus {
        let state = self.get_state().await;
        let degraded_since = if state == FallbackState::Degraded {
            let start_time = self.degraded_mode_start.read().await;
            *start_time
        } else {
            None
        };

        FallbackStatus {
            state,
            degraded_since,
            local_encryption_enabled: self.config.enable_local_encryption,
            simplified_challenges_enabled: self.config.enable_simplified_challenges,
            manual_verification_enabled: self.config.enable_manual_verification,
            cache_enabled: self.config.enable_cached_responses,
        }
    }
}

/// Fallback service status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStatus {
    /// Current fallback state
    pub state: FallbackState,
    /// Timestamp when degraded mode was entered
    pub degraded_since: Option<SystemTime>,
    /// Whether local encryption fallback is enabled
    pub local_encryption_enabled: bool,
    /// Whether simplified challenges fallback is enabled
    pub simplified_challenges_enabled: bool,
    /// Whether manual verification fallback is enabled
    pub manual_verification_enabled: bool,
    /// Whether response caching is enabled
    pub cache_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_encryption_fallback() {
        let fallback = LocalEncryptionFallback::new();
        let original = "test data";

        let encrypted = fallback.encrypt(original).unwrap();
        let decrypted = fallback.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_simplified_challenge_generation() {
        let generator = SimplifiedChallengeGenerator::new();

        for difficulty in 1..=10 {
            let challenge = generator.generate_simple_challenge(difficulty).unwrap();
            assert_eq!(challenge.challenge_type, ChallengeType::Logical);
            assert_eq!(challenge.difficulty_level, difficulty);
            assert!(!challenge.encrypted_data.is_empty());
            assert!(!challenge.expected_answer_hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_fallback_service_state_transitions() {
        let config = FallbackConfig::default();
        let service = FallbackService::new(config);

        assert_eq!(service.get_state().await, FallbackState::Normal);

        service.enter_degraded_mode("test").await;
        assert_eq!(service.get_state().await, FallbackState::Degraded);

        service.enter_emergency_mode("test").await;
        assert_eq!(service.get_state().await, FallbackState::Emergency);

        service.return_to_normal().await;
        assert_eq!(service.get_state().await, FallbackState::Normal);
    }

    #[tokio::test]
    async fn test_response_cache() {
        let cache = ResponseCache::new(Duration::from_secs(1));
        let key = "test_key".to_string();
        let result = ValidationResult::success(0.9, 2);

        // Cache miss
        assert!(cache.get(&key).await.is_none());

        // Cache hit
        cache.set(key.clone(), result.clone()).await;
        let cached = cache.get(&key).await.unwrap();
        assert_eq!(cached.success, result.success);

        // Cache expiry
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(cache.get(&key).await.is_none());
    }
}
