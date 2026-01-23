//! Data Classification Service
//!
//! Implements Indonesian government data classification levels and access control.
//! Classification levels follow Indonesian government standards for data protection.
//!
//! # Classification Levels
//! - **BIASA**: Public information
//! - **TERBATAS**: Internal use only
//! - **RAHASIA**: Confidential - requires MFA
//! - **SANGAT_RAHASIA**: Top Secret - requires MFA and highest clearance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

use crate::error::{CoreError, Result};
use crate::services::mfa::MfaService;

/// Indonesian government classification levels
/// Based on Indonesian government data protection regulations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClassificationLevel {
    /// Public information - no restrictions
    #[serde(rename = "BIASA")]
    Biasa,

    /// Internal use only - restricted to organization
    #[serde(rename = "TERBATAS")]
    Terbatas,

    /// Confidential - requires MFA verification
    #[serde(rename = "RAHASIA")]
    Rahasia,

    /// Top Secret - requires MFA and highest clearance
    #[serde(rename = "SANGAT_RAHASIA")]
    SangatRahasia,
}

impl ClassificationLevel {
    /// Check if this classification level requires MFA
    pub fn requires_mfa(&self) -> bool {
        matches!(self, Self::Rahasia | Self::SangatRahasia)
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Biasa => "BIASA",
            Self::Terbatas => "TERBATAS",
            Self::Rahasia => "RAHASIA",
            Self::SangatRahasia => "SANGAT_RAHASIA",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Biasa => "Public information",
            Self::Terbatas => "Internal use only",
            Self::Rahasia => "Confidential - requires MFA",
            Self::SangatRahasia => "Top Secret - requires MFA and highest clearance",
        }
    }
}

impl std::fmt::Display for ClassificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Classification metadata for a secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetadata {
    /// Classification level
    pub level: ClassificationLevel,

    /// Whether MFA is required for access
    pub requires_mfa: bool,

    /// Minimum clearance level required
    pub clearance_required: ClassificationLevel,

    /// User who classified the secret
    pub created_by: String,

    /// When the classification was set
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub last_accessed_at: Option<DateTime<Utc>>,

    /// Access count
    pub access_count: u64,
}

impl ClassificationMetadata {
    /// Create new classification metadata
    pub fn new(level: ClassificationLevel, created_by: String) -> Self {
        Self {
            level,
            requires_mfa: level.requires_mfa(),
            clearance_required: level,
            created_by,
            created_at: Utc::now(),
            last_accessed_at: None,
            access_count: 0,
        }
    }

    /// Record an access
    pub fn record_access(&mut self) {
        self.last_accessed_at = Some(Utc::now());
        self.access_count += 1;
    }
}

/// Classification statistics for a level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationStats {
    /// Classification level
    pub level: ClassificationLevel,

    /// Number of secrets at this level
    pub secret_count: usize,

    /// Total access count
    pub total_accesses: u64,

    /// Last access timestamp
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Classification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Statistics by classification level
    pub stats_by_level: HashMap<ClassificationLevel, ClassificationStats>,

    /// Total secrets
    pub total_secrets: usize,

    /// Total accesses
    pub total_accesses: u64,
}

/// Classification policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Violation ID
    pub id: String,

    /// Secret path
    pub path: String,

    /// Violation type
    pub violation_type: ViolationType,

    /// User who attempted the operation
    pub user_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional details
    pub details: String,
}

/// Types of policy violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Insufficient clearance level
    InsufficientClearance,

    /// MFA not verified
    MfaNotVerified,

    /// Attempted to store secret without proper classification
    MissingClassification,

    /// Attempted to downgrade classification
    InvalidDowngrade,

    /// Cross-classification access violation
    CrossClassificationViolation,
}

/// Classification service trait
#[async_trait::async_trait]
pub trait ClassificationService: Send + Sync {
    /// Set classification for a secret
    async fn classify(&self, path: &str, level: ClassificationLevel, user_id: &str) -> Result<()>;

    /// Get classification for a secret
    async fn get_classification(&self, path: &str) -> Result<Option<ClassificationMetadata>>;

    /// Check if user has access based on clearance
    async fn check_access(
        &self,
        path: &str,
        user_clearance: ClassificationLevel,
        mfa_verified: bool,
    ) -> Result<bool>;

    /// Enforce MFA for high-classification secrets (RAHASIA, SANGAT_RAHASIA)
    async fn enforce_mfa(&self, path: &str, user_id: &str) -> Result<()>;

    /// Check if user clearance is sufficient for secret
    async fn check_clearance(
        &self,
        path: &str,
        user_clearance: ClassificationLevel,
    ) -> Result<bool>;

    /// Generate classification report
    async fn generate_report(&self) -> Result<ClassificationReport>;

    /// Check if MFA is required for a secret
    async fn require_mfa(&self, path: &str) -> Result<bool>;

    /// Record a policy violation
    async fn record_violation(&self, violation: PolicyViolation) -> Result<()>;

    /// Get policy violations
    async fn get_violations(&self, limit: usize) -> Result<Vec<PolicyViolation>>;

    /// Validate operation against classification policy
    async fn validate_operation(
        &self,
        path: &str,
        user_id: &str,
        user_clearance: ClassificationLevel,
        mfa_verified: bool,
    ) -> Result<()>;
}

/// In-memory classification service implementation
pub struct InMemoryClassificationService {
    /// Classifications by path
    classifications: Arc<RwLock<HashMap<String, ClassificationMetadata>>>,

    /// Policy violations
    violations: Arc<RwLock<Vec<PolicyViolation>>>,

    /// MFA service reference
    mfa_service: Option<Arc<MfaService>>,
}

impl InMemoryClassificationService {
    /// Create new classification service
    pub fn new() -> Self {
        Self {
            classifications: Arc::new(RwLock::new(HashMap::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            mfa_service: None,
        }
    }

    /// Create classification service with MFA integration
    pub fn with_mfa(mfa_service: Arc<MfaService>) -> Self {
        Self {
            classifications: Arc::new(RwLock::new(HashMap::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            mfa_service: Some(mfa_service),
        }
    }
}

impl Default for InMemoryClassificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ClassificationService for InMemoryClassificationService {
    #[instrument(skip(self), fields(
        path = %path,
        level = %level,
        user_id = %user_id,
        operation = "classify"
    ))]
    async fn classify(&self, path: &str, level: ClassificationLevel, user_id: &str) -> Result<()> {
        let mut classifications = self.classifications.write().await;

        // Check if downgrading classification
        if let Some(existing) = classifications.get(path) {
            if level < existing.level {
                warn!(
                    path = %path,
                    old_level = %existing.level,
                    new_level = %level,
                    "Attempted to downgrade classification"
                );

                // Record violation
                let violation = PolicyViolation {
                    id: uuid::Uuid::new_v4().to_string(),
                    path: path.to_string(),
                    violation_type: ViolationType::InvalidDowngrade,
                    user_id: user_id.to_string(),
                    timestamp: Utc::now(),
                    details: format!(
                        "Attempted to downgrade from {} to {}",
                        existing.level, level
                    ),
                };

                let mut violations = self.violations.write().await;
                violations.push(violation);

                return Err(CoreError::invalid_operation(
                    "Cannot downgrade classification level",
                ));
            }
        }

        let metadata = ClassificationMetadata::new(level, user_id.to_string());
        classifications.insert(path.to_string(), metadata);

        info!(
            path = %path,
            level = %level,
            user_id = %user_id,
            "Secret classified"
        );

        Ok(())
    }

    async fn get_classification(&self, path: &str) -> Result<Option<ClassificationMetadata>> {
        let classifications = self.classifications.read().await;
        Ok(classifications.get(path).cloned())
    }

    #[instrument(skip(self), fields(
        path = %path,
        user_clearance = %user_clearance,
        mfa_verified = %mfa_verified,
        operation = "check_access"
    ))]
    async fn check_access(
        &self,
        path: &str,
        user_clearance: ClassificationLevel,
        mfa_verified: bool,
    ) -> Result<bool> {
        let mut classifications = self.classifications.write().await;

        let metadata = match classifications.get_mut(path) {
            Some(m) => m,
            None => {
                // No classification means BIASA (public)
                return Ok(true);
            }
        };

        // Check clearance level
        if user_clearance < metadata.clearance_required {
            warn!(
                path = %path,
                user_clearance = %user_clearance,
                required_clearance = %metadata.clearance_required,
                "Insufficient clearance level"
            );

            // Record violation
            let violation = PolicyViolation {
                id: uuid::Uuid::new_v4().to_string(),
                path: path.to_string(),
                violation_type: ViolationType::InsufficientClearance,
                user_id: "unknown".to_string(), // Would need to pass user_id to check_access
                timestamp: Utc::now(),
                details: format!(
                    "User clearance {} insufficient for {} secret",
                    user_clearance, metadata.clearance_required
                ),
            };

            let mut violations = self.violations.write().await;
            violations.push(violation);

            return Ok(false);
        }

        // Check MFA requirement
        if metadata.requires_mfa && !mfa_verified {
            warn!(
                path = %path,
                level = %metadata.level,
                "MFA verification required but not provided"
            );
            return Ok(false);
        }

        // Record successful access
        metadata.record_access();

        info!(
            path = %path,
            user_clearance = %user_clearance,
            level = %metadata.level,
            "Access granted"
        );

        Ok(true)
    }

    #[instrument(skip(self), fields(operation = "generate_report"))]
    async fn generate_report(&self) -> Result<ClassificationReport> {
        let classifications = self.classifications.read().await;

        let mut stats_by_level: HashMap<ClassificationLevel, ClassificationStats> = HashMap::new();
        let mut total_secrets = 0;
        let mut total_accesses = 0;

        for (_, metadata) in classifications.iter() {
            total_secrets += 1;
            total_accesses += metadata.access_count;

            let stats =
                stats_by_level
                    .entry(metadata.level)
                    .or_insert_with(|| ClassificationStats {
                        level: metadata.level,
                        secret_count: 0,
                        total_accesses: 0,
                        last_accessed: None,
                    });

            stats.secret_count += 1;
            stats.total_accesses += metadata.access_count;

            if let Some(last_accessed) = metadata.last_accessed_at {
                stats.last_accessed = Some(match stats.last_accessed {
                    Some(existing) => existing.max(last_accessed),
                    None => last_accessed,
                });
            }
        }

        Ok(ClassificationReport {
            generated_at: Utc::now(),
            stats_by_level,
            total_secrets,
            total_accesses,
        })
    }

    async fn require_mfa(&self, path: &str) -> Result<bool> {
        let classifications = self.classifications.read().await;

        match classifications.get(path) {
            Some(metadata) => Ok(metadata.requires_mfa),
            None => Ok(false), // No classification means BIASA (no MFA required)
        }
    }

    #[instrument(skip(self, violation), fields(
        path = %violation.path,
        violation_type = ?violation.violation_type,
        user_id = %violation.user_id,
        operation = "record_violation"
    ))]
    async fn record_violation(&self, violation: PolicyViolation) -> Result<()> {
        let mut violations = self.violations.write().await;
        violations.push(violation);
        Ok(())
    }

    #[instrument(skip(self), fields(
        path = %path,
        user_clearance = %user_clearance,
        operation = "check_clearance"
    ))]
    async fn check_clearance(
        &self,
        path: &str,
        user_clearance: ClassificationLevel,
    ) -> Result<bool> {
        let classifications = self.classifications.read().await;

        let metadata = match classifications.get(path) {
            Some(m) => m,
            None => {
                // No classification means BIASA (public) - always accessible
                return Ok(true);
            }
        };

        // Check if user clearance is sufficient
        if user_clearance < metadata.clearance_required {
            warn!(
                path = %path,
                user_clearance = %user_clearance,
                required_clearance = %metadata.clearance_required,
                "Insufficient clearance level"
            );

            // Record violation
            let violation = PolicyViolation {
                id: uuid::Uuid::new_v4().to_string(),
                path: path.to_string(),
                violation_type: ViolationType::InsufficientClearance,
                user_id: "unknown".to_string(),
                timestamp: Utc::now(),
                details: format!(
                    "User clearance {} insufficient for {} secret",
                    user_clearance, metadata.clearance_required
                ),
            };

            let mut violations = self.violations.write().await;
            violations.push(violation);

            return Ok(false);
        }

        info!(
            path = %path,
            user_clearance = %user_clearance,
            required_clearance = %metadata.clearance_required,
            "Clearance check passed"
        );

        Ok(true)
    }

    #[instrument(skip(self), fields(
        path = %path,
        user_id = %user_id,
        operation = "enforce_mfa"
    ))]
    async fn enforce_mfa(&self, path: &str, user_id: &str) -> Result<()> {
        let classifications = self.classifications.read().await;

        let metadata = match classifications.get(path) {
            Some(m) => m,
            None => {
                // No classification means BIASA (no MFA required)
                return Ok(());
            }
        };

        // Check if MFA is required for this classification level
        if !metadata.requires_mfa {
            return Ok(());
        }

        // Check if MFA service is available
        let mfa_service = match &self.mfa_service {
            Some(service) => service,
            None => {
                warn!(
                    path = %path,
                    level = %metadata.level,
                    "MFA service not configured but required for classification level"
                );
                return Err(CoreError::invalid_operation("MFA service not configured"));
            }
        };

        // Check if user has MFA configured
        if !mfa_service.is_configured(user_id).await {
            warn!(
                path = %path,
                user_id = %user_id,
                level = %metadata.level,
                "MFA not configured for user accessing high-classification secret"
            );

            // Record violation
            let violation = PolicyViolation {
                id: uuid::Uuid::new_v4().to_string(),
                path: path.to_string(),
                violation_type: ViolationType::MfaNotVerified,
                user_id: user_id.to_string(),
                timestamp: Utc::now(),
                details: format!(
                    "MFA required for {} but user has no MFA configured",
                    metadata.level
                ),
            };

            let mut violations = self.violations.write().await;
            violations.push(violation);

            return Err(CoreError::authorization(
                "MFA verification required for this classification level",
            ));
        }

        info!(
            path = %path,
            user_id = %user_id,
            level = %metadata.level,
            "MFA enforcement check passed"
        );

        Ok(())
    }

    async fn get_violations(&self, limit: usize) -> Result<Vec<PolicyViolation>> {
        let violations = self.violations.read().await;
        let len = violations.len();
        let start = if len > limit { len - limit } else { 0 };
        Ok(violations[start..].to_vec())
    }

    #[instrument(skip(self), fields(
        path = %path,
        user_id = %user_id,
        user_clearance = %user_clearance,
        mfa_verified = %mfa_verified,
        operation = "validate_operation"
    ))]
    async fn validate_operation(
        &self,
        path: &str,
        user_id: &str,
        user_clearance: ClassificationLevel,
        mfa_verified: bool,
    ) -> Result<()> {
        let classifications = self.classifications.read().await;

        let metadata = match classifications.get(path) {
            Some(m) => m,
            None => {
                // No classification means BIASA (public) - operation allowed
                return Ok(());
            }
        };

        // Policy 1: Check clearance level
        if user_clearance < metadata.clearance_required {
            warn!(
                path = %path,
                user_id = %user_id,
                user_clearance = %user_clearance,
                required_clearance = %metadata.clearance_required,
                "Policy violation: Insufficient clearance"
            );

            let violation = PolicyViolation {
                id: uuid::Uuid::new_v4().to_string(),
                path: path.to_string(),
                violation_type: ViolationType::InsufficientClearance,
                user_id: user_id.to_string(),
                timestamp: Utc::now(),
                details: format!(
                    "User clearance {} insufficient for {} secret",
                    user_clearance, metadata.clearance_required
                ),
            };

            let mut violations = self.violations.write().await;
            violations.push(violation);

            return Err(CoreError::authorization(format!(
                "Insufficient clearance: required {}, have {}",
                metadata.clearance_required, user_clearance
            )));
        }

        // Policy 2: Check MFA requirement
        if metadata.requires_mfa && !mfa_verified {
            warn!(
                path = %path,
                user_id = %user_id,
                level = %metadata.level,
                "Policy violation: MFA not verified"
            );

            let violation = PolicyViolation {
                id: uuid::Uuid::new_v4().to_string(),
                path: path.to_string(),
                violation_type: ViolationType::MfaNotVerified,
                user_id: user_id.to_string(),
                timestamp: Utc::now(),
                details: format!("MFA required for {} but not verified", metadata.level),
            };

            let mut violations = self.violations.write().await;
            violations.push(violation);

            return Err(CoreError::authorization(
                "MFA verification required for this classification level",
            ));
        }

        info!(
            path = %path,
            user_id = %user_id,
            level = %metadata.level,
            "Policy validation passed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classification_levels() {
        assert!(ClassificationLevel::Rahasia.requires_mfa());
        assert!(ClassificationLevel::SangatRahasia.requires_mfa());
        assert!(!ClassificationLevel::Biasa.requires_mfa());
        assert!(!ClassificationLevel::Terbatas.requires_mfa());
    }

    #[tokio::test]
    async fn test_classification_ordering() {
        assert!(ClassificationLevel::Biasa < ClassificationLevel::Terbatas);
        assert!(ClassificationLevel::Terbatas < ClassificationLevel::Rahasia);
        assert!(ClassificationLevel::Rahasia < ClassificationLevel::SangatRahasia);
    }

    #[tokio::test]
    async fn test_classify_secret() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        let metadata = service
            .get_classification("secret/test")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(metadata.level, ClassificationLevel::Rahasia);
        assert!(metadata.requires_mfa);
        assert_eq!(metadata.created_by, "user1");
    }

    #[tokio::test]
    async fn test_prevent_downgrade() {
        let service = InMemoryClassificationService::new();

        // Set to RAHASIA
        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // Try to downgrade to TERBATAS
        let result = service
            .classify("secret/test", ClassificationLevel::Terbatas, "user1")
            .await;

        assert!(result.is_err());

        // Verify violation was recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].violation_type,
            ViolationType::InvalidDowngrade
        ));
    }

    #[tokio::test]
    async fn test_check_access_clearance() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // Sufficient clearance with MFA
        let has_access = service
            .check_access("secret/test", ClassificationLevel::Rahasia, true)
            .await
            .unwrap();
        assert!(has_access);

        // Insufficient clearance
        let has_access = service
            .check_access("secret/test", ClassificationLevel::Terbatas, true)
            .await
            .unwrap();
        assert!(!has_access);
    }

    #[tokio::test]
    async fn test_check_access_mfa_required() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // Sufficient clearance but no MFA
        let has_access = service
            .check_access("secret/test", ClassificationLevel::Rahasia, false)
            .await
            .unwrap();
        assert!(!has_access);

        // Sufficient clearance with MFA
        let has_access = service
            .check_access("secret/test", ClassificationLevel::Rahasia, true)
            .await
            .unwrap();
        assert!(has_access);
    }

    #[tokio::test]
    async fn test_generate_report() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/1", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();
        service
            .classify("secret/2", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();
        service
            .classify("secret/3", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // Access some secrets
        service
            .check_access("secret/2", ClassificationLevel::Rahasia, true)
            .await
            .unwrap();
        service
            .check_access("secret/2", ClassificationLevel::Rahasia, true)
            .await
            .unwrap();

        let report = service.generate_report().await.unwrap();

        assert_eq!(report.total_secrets, 3);
        assert_eq!(report.total_accesses, 2);

        let rahasia_stats = report
            .stats_by_level
            .get(&ClassificationLevel::Rahasia)
            .unwrap();
        assert_eq!(rahasia_stats.secret_count, 2);
        assert_eq!(rahasia_stats.total_accesses, 2);
    }

    #[tokio::test]
    async fn test_require_mfa() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/public", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();
        service
            .classify("secret/confidential", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        assert!(!service.require_mfa("secret/public").await.unwrap());
        assert!(service.require_mfa("secret/confidential").await.unwrap());
        assert!(!service.require_mfa("secret/nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_access_tracking() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Terbatas, "user1")
            .await
            .unwrap();

        // Access multiple times
        for _ in 0..5 {
            service
                .check_access("secret/test", ClassificationLevel::Terbatas, false)
                .await
                .unwrap();
        }

        let metadata = service
            .get_classification("secret/test")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(metadata.access_count, 5);
        assert!(metadata.last_accessed_at.is_some());
    }

    #[tokio::test]
    async fn test_enforce_mfa_not_required() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();

        // MFA enforcement should pass for BIASA level
        let result = service.enforce_mfa("secret/test", "user1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enforce_mfa_required_no_service() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // MFA enforcement should fail when MFA service not configured
        let result = service.enforce_mfa("secret/test", "user1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enforce_mfa_required_with_service() {
        let mfa_service = Arc::new(MfaService::new());
        let service = InMemoryClassificationService::with_mfa(mfa_service.clone());

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // User without MFA configured should fail
        let result = service.enforce_mfa("secret/test", "user1").await;
        assert!(result.is_err());

        // Verify violation was recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].violation_type,
            ViolationType::MfaNotVerified
        ));

        // Enable MFA for user
        mfa_service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@test.com".to_string(),
            )
            .await
            .unwrap();

        // Now enforcement should pass
        let result = service.enforce_mfa("secret/test", "user1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enforce_mfa_sangat_rahasia() {
        let mfa_service = Arc::new(MfaService::new());
        let service = InMemoryClassificationService::with_mfa(mfa_service.clone());

        service
            .classify(
                "secret/topsecret",
                ClassificationLevel::SangatRahasia,
                "user1",
            )
            .await
            .unwrap();

        // User without MFA should fail
        let result = service.enforce_mfa("secret/topsecret", "user1").await;
        assert!(result.is_err());

        // Enable MFA
        mfa_service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@test.com".to_string(),
            )
            .await
            .unwrap();

        // Now should pass
        let result = service.enforce_mfa("secret/topsecret", "user1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_clearance_sufficient() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // User with sufficient clearance
        let has_clearance = service
            .check_clearance("secret/test", ClassificationLevel::Rahasia)
            .await
            .unwrap();
        assert!(has_clearance);

        // User with higher clearance
        let has_clearance = service
            .check_clearance("secret/test", ClassificationLevel::SangatRahasia)
            .await
            .unwrap();
        assert!(has_clearance);
    }

    #[tokio::test]
    async fn test_check_clearance_insufficient() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // User with insufficient clearance
        let has_clearance = service
            .check_clearance("secret/test", ClassificationLevel::Terbatas)
            .await
            .unwrap();
        assert!(!has_clearance);

        // Verify violation was recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].violation_type,
            ViolationType::InsufficientClearance
        ));
    }

    #[tokio::test]
    async fn test_check_clearance_nonexistent_secret() {
        let service = InMemoryClassificationService::new();

        // Non-existent secret should allow access (defaults to BIASA)
        let has_clearance = service
            .check_clearance("secret/nonexistent", ClassificationLevel::Biasa)
            .await
            .unwrap();
        assert!(has_clearance);
    }

    #[tokio::test]
    async fn test_check_clearance_all_levels() {
        let service = InMemoryClassificationService::new();

        // Create secrets at all levels
        service
            .classify("secret/biasa", ClassificationLevel::Biasa, "user1")
            .await
            .unwrap();
        service
            .classify("secret/terbatas", ClassificationLevel::Terbatas, "user1")
            .await
            .unwrap();
        service
            .classify("secret/rahasia", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();
        service
            .classify(
                "secret/sangat_rahasia",
                ClassificationLevel::SangatRahasia,
                "user1",
            )
            .await
            .unwrap();

        // User with TERBATAS clearance
        assert!(
            service
                .check_clearance("secret/biasa", ClassificationLevel::Terbatas)
                .await
                .unwrap()
        );
        assert!(
            service
                .check_clearance("secret/terbatas", ClassificationLevel::Terbatas)
                .await
                .unwrap()
        );
        assert!(
            !service
                .check_clearance("secret/rahasia", ClassificationLevel::Terbatas)
                .await
                .unwrap()
        );
        assert!(
            !service
                .check_clearance("secret/sangat_rahasia", ClassificationLevel::Terbatas)
                .await
                .unwrap()
        );

        // User with SANGAT_RAHASIA clearance (highest)
        assert!(
            service
                .check_clearance("secret/biasa", ClassificationLevel::SangatRahasia)
                .await
                .unwrap()
        );
        assert!(
            service
                .check_clearance("secret/terbatas", ClassificationLevel::SangatRahasia)
                .await
                .unwrap()
        );
        assert!(
            service
                .check_clearance("secret/rahasia", ClassificationLevel::SangatRahasia)
                .await
                .unwrap()
        );
        assert!(
            service
                .check_clearance("secret/sangat_rahasia", ClassificationLevel::SangatRahasia)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_validate_operation_success() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Terbatas, "user1")
            .await
            .unwrap();

        // Valid operation
        let result = service
            .validate_operation("secret/test", "user1", ClassificationLevel::Terbatas, false)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_operation_insufficient_clearance() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // Insufficient clearance
        let result = service
            .validate_operation("secret/test", "user2", ClassificationLevel::Terbatas, true)
            .await;
        assert!(result.is_err());

        // Verify violation was recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].violation_type,
            ViolationType::InsufficientClearance
        ));
    }

    #[tokio::test]
    async fn test_validate_operation_mfa_not_verified() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // MFA not verified
        let result = service
            .validate_operation("secret/test", "user1", ClassificationLevel::Rahasia, false)
            .await;
        assert!(result.is_err());

        // Verify violation was recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].violation_type,
            ViolationType::MfaNotVerified
        ));
    }

    #[tokio::test]
    async fn test_validate_operation_all_checks_pass() {
        let service = InMemoryClassificationService::new();

        service
            .classify("secret/test", ClassificationLevel::Rahasia, "user1")
            .await
            .unwrap();

        // All checks pass
        let result = service
            .validate_operation("secret/test", "user1", ClassificationLevel::Rahasia, true)
            .await;
        assert!(result.is_ok());

        // No violations should be recorded
        let violations = service.get_violations(10).await.unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[tokio::test]
    async fn test_validate_operation_nonexistent_secret() {
        let service = InMemoryClassificationService::new();

        // Non-existent secret should allow operation
        let result = service
            .validate_operation(
                "secret/nonexistent",
                "user1",
                ClassificationLevel::Biasa,
                false,
            )
            .await;
        assert!(result.is_ok());
    }
}
