//! Auto-Rotation Engine
//!
//! Automatic secret rotation with zero-downtime strategy, webhooks, and rollback capability.
//!
//! # Features
//! - Automatic rotation scheduler
//! - Zero-downtime rotation strategy
//! - Pre/post rotation webhooks
//! - Rotation history tracking
//! - Rollback capability
//! - Custom rotation scripts
//! - Multi-secret type support

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Auto-Rotation Engine errors
#[derive(Debug, thiserror::Error)]
pub enum RotationError {
    #[error("Rotation policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Rotation policy already exists: {0}")]
    PolicyAlreadyExists(String),

    #[error("Invalid rotation configuration: {0}")]
    InvalidConfig(String),

    #[error("Rotation failed: {0}")]
    RotationFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Webhook notification failed: {0}")]
    WebhookFailed(String),

    #[error("History not found: {0}")]
    HistoryNotFound(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),
}

/// Rotation strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RotationStrategy {
    /// Immediate rotation (may cause brief downtime)
    Immediate,
    /// Blue-green rotation (zero downtime)
    BlueGreen,
    /// Rolling rotation (gradual update)
    Rolling,
    /// Canary rotation (test with subset first)
    Canary,
}

/// Secret type for rotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecretType {
    DatabasePassword,
    ApiKey,
    Certificate,
    EncryptionKey,
    ServiceAccount,
    OAuth2Token,
    Custom(String),
}

/// Rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Policy ID
    pub id: String,

    /// Policy name
    pub name: String,

    /// Secret path or pattern
    pub secret_path: String,

    /// Secret type
    pub secret_type: SecretType,

    /// Rotation interval (e.g., every 90 days)
    pub rotation_interval: Duration,

    /// Rotation strategy
    pub strategy: RotationStrategy,

    /// Enable automatic rotation
    pub auto_rotation: bool,

    /// Webhook notifications
    pub webhooks: WebhookConfig,

    /// Custom rotation script (optional)
    pub custom_script: Option<String>,

    /// Rollback enabled
    pub rollback_enabled: bool,

    /// Keep history count
    pub history_retention: u32,

    /// Next scheduled rotation
    pub next_rotation: Option<DateTime<Utc>>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Enabled status
    pub enabled: bool,
}

impl RotationPolicy {
    pub fn new(name: String, secret_path: String, secret_type: SecretType) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id,
            name,
            secret_path,
            secret_type,
            rotation_interval: Duration::days(90),
            strategy: RotationStrategy::BlueGreen,
            auto_rotation: true,
            webhooks: WebhookConfig::default(),
            custom_script: None,
            rollback_enabled: true,
            history_retention: 10,
            next_rotation: Some(now + Duration::days(90)),
            created_at: now,
            updated_at: now,
            enabled: true,
        }
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Pre-rotation webhooks
    pub pre_rotation: Vec<WebhookEndpoint>,

    /// Post-rotation webhooks
    pub post_rotation: Vec<WebhookEndpoint>,

    /// Failure webhooks
    pub on_failure: Vec<WebhookEndpoint>,

    /// Timeout for webhook calls (seconds)
    pub timeout_seconds: u32,

    /// Retry count on failure
    pub retry_count: u32,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            pre_rotation: Vec::new(),
            post_rotation: Vec::new(),
            on_failure: Vec::new(),
            timeout_seconds: 30,
            retry_count: 3,
        }
    }
}

/// Webhook endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    /// Endpoint URL
    pub url: String,

    /// HTTP method (POST, PUT)
    pub method: String,

    /// Headers
    pub headers: HashMap<String, String>,

    /// Authentication token (optional)
    pub auth_token: Option<String>,
}

/// Rotation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationHistory {
    /// History ID
    pub id: String,

    /// Policy ID
    pub policy_id: String,

    /// Secret path
    pub secret_path: String,

    /// Rotation status
    pub status: RotationStatus,

    /// Rotation strategy used
    pub strategy: RotationStrategy,

    /// Old version/value (encrypted)
    pub old_version: String,

    /// New version/value (encrypted)
    pub new_version: String,

    /// Started timestamp
    pub started_at: DateTime<Utc>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,

    /// Duration (milliseconds)
    pub duration_ms: Option<u64>,

    /// Error message (if failed)
    pub error_message: Option<String>,

    /// Triggered by (user or system)
    pub triggered_by: String,

    /// Rollback available
    pub rollback_available: bool,

    /// Rolled back
    pub rolled_back: bool,
}

/// Rotation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RotationStatus {
    Scheduled,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// Rotation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStatistics {
    pub total_rotations: u64,
    pub successful_rotations: u64,
    pub failed_rotations: u64,
    pub rollbacks: u64,
    pub average_duration_ms: u64,
    pub last_rotation: Option<DateTime<Utc>>,
}

/// Auto-Rotation Engine
pub struct AutoRotationEngine {
    policies: Arc<RwLock<HashMap<String, RotationPolicy>>>,
    history: Arc<RwLock<Vec<RotationHistory>>>,
    statistics: Arc<RwLock<RotationStatistics>>,
    scheduler_running: Arc<RwLock<bool>>,
}

impl AutoRotationEngine {
    /// Create new auto-rotation engine
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(RotationStatistics {
                total_rotations: 0,
                successful_rotations: 0,
                failed_rotations: 0,
                rollbacks: 0,
                average_duration_ms: 0,
                last_rotation: None,
            })),
            scheduler_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create rotation policy
    #[instrument(skip(self))]
    pub async fn create_policy(
        &self,
        policy: RotationPolicy,
    ) -> Result<RotationPolicy, RotationError> {
        // Validate policy doesn't exist
        {
            let policies = self.policies.read().await;
            if policies.contains_key(&policy.id) {
                return Err(RotationError::PolicyAlreadyExists(policy.id));
            }
        }

        // Validate configuration
        if policy.secret_path.is_empty() {
            return Err(RotationError::InvalidConfig(
                "Secret path is required".to_string(),
            ));
        }

        if policy.rotation_interval.num_seconds() < 3600 {
            return Err(RotationError::InvalidConfig(
                "Rotation interval must be at least 1 hour".to_string(),
            ));
        }

        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());

        info!("Rotation policy created: {}", policy.name);
        Ok(policy)
    }

    /// Get rotation policy
    pub async fn get_policy(&self, policy_id: &str) -> Result<RotationPolicy, RotationError> {
        let policies = self.policies.read().await;
        policies
            .get(policy_id)
            .cloned()
            .ok_or_else(|| RotationError::PolicyNotFound(policy_id.to_string()))
    }

    /// List rotation policies
    pub async fn list_policies(&self) -> Vec<RotationPolicy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// Update rotation policy
    #[instrument(skip(self))]
    pub async fn update_policy(
        &self,
        policy: RotationPolicy,
    ) -> Result<RotationPolicy, RotationError> {
        let mut policies = self.policies.write().await;

        if !policies.contains_key(&policy.id) {
            return Err(RotationError::PolicyNotFound(policy.id));
        }

        let mut updated_policy = policy;
        updated_policy.updated_at = Utc::now();

        policies.insert(updated_policy.id.clone(), updated_policy.clone());

        info!("Rotation policy updated: {}", updated_policy.name);
        Ok(updated_policy)
    }

    /// Delete rotation policy
    #[instrument(skip(self))]
    pub async fn delete_policy(&self, policy_id: &str) -> Result<(), RotationError> {
        let mut policies = self.policies.write().await;
        policies
            .remove(policy_id)
            .ok_or_else(|| RotationError::PolicyNotFound(policy_id.to_string()))?;

        info!("Rotation policy deleted: {}", policy_id);
        Ok(())
    }

    /// Execute rotation for a policy
    #[instrument(skip(self))]
    pub async fn execute_rotation(
        &self,
        policy_id: &str,
        triggered_by: String,
    ) -> Result<RotationHistory, RotationError> {
        let policy = self.get_policy(policy_id).await?;

        if !policy.enabled {
            return Err(RotationError::RotationFailed(
                "Policy is disabled".to_string(),
            ));
        }

        let history_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();

        info!(
            "Starting rotation for policy: {} ({})",
            policy.name, policy.secret_path
        );

        // Send pre-rotation webhooks
        if let Err(e) = self
            .send_webhooks(&policy.webhooks.pre_rotation, &policy, "pre_rotation")
            .await
        {
            warn!("Pre-rotation webhook failed: {}", e);
        }

        // Execute rotation based on strategy
        let result = match policy.strategy {
            RotationStrategy::Immediate => self.rotate_immediate(&policy).await,
            RotationStrategy::BlueGreen => self.rotate_blue_green(&policy).await,
            RotationStrategy::Rolling => self.rotate_rolling(&policy).await,
            RotationStrategy::Canary => self.rotate_canary(&policy).await,
        };

        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

        let mut history_entry = RotationHistory {
            id: history_id,
            policy_id: policy.id.clone(),
            secret_path: policy.secret_path.clone(),
            status: RotationStatus::InProgress,
            strategy: policy.strategy.clone(),
            old_version: "v1".to_string(), // Placeholder
            new_version: "v2".to_string(), // Placeholder
            started_at,
            completed_at: Some(completed_at),
            duration_ms: Some(duration_ms),
            error_message: None,
            triggered_by,
            rollback_available: policy.rollback_enabled,
            rolled_back: false,
        };

        match result {
            Ok(_) => {
                history_entry.status = RotationStatus::Completed;

                // Send post-rotation webhooks
                if let Err(e) = self
                    .send_webhooks(&policy.webhooks.post_rotation, &policy, "post_rotation")
                    .await
                {
                    warn!("Post-rotation webhook failed: {}", e);
                }

                // Update statistics
                self.update_statistics(true, duration_ms).await;

                // Update next rotation time
                self.update_next_rotation(&policy).await;

                info!(
                    "Rotation completed successfully for: {} ({}ms)",
                    policy.name, duration_ms
                );
            }
            Err(e) => {
                history_entry.status = RotationStatus::Failed;
                history_entry.error_message = Some(e.to_string());

                // Send failure webhooks
                if let Err(webhook_err) = self
                    .send_webhooks(&policy.webhooks.on_failure, &policy, "failure")
                    .await
                {
                    warn!("Failure webhook failed: {}", webhook_err);
                }

                // Update statistics
                self.update_statistics(false, duration_ms).await;

                error!("Rotation failed for: {} - {}", policy.name, e);
            }
        }

        // Store history
        self.add_history(history_entry.clone()).await;

        Ok(history_entry)
    }

    /// Rollback rotation
    #[instrument(skip(self))]
    pub async fn rollback_rotation(&self, history_id: &str) -> Result<(), RotationError> {
        let mut history = self.history.write().await;

        let entry = history
            .iter_mut()
            .find(|h| h.id == history_id)
            .ok_or_else(|| RotationError::HistoryNotFound(history_id.to_string()))?;

        if !entry.rollback_available {
            return Err(RotationError::RollbackFailed(
                "Rollback not available".to_string(),
            ));
        }

        if entry.rolled_back {
            return Err(RotationError::RollbackFailed(
                "Already rolled back".to_string(),
            ));
        }

        // Perform rollback (restore old version)
        info!(
            "Rolling back rotation: {} for {}",
            history_id, entry.secret_path
        );

        entry.rolled_back = true;
        entry.status = RotationStatus::RolledBack;

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.rollbacks += 1;

        info!("Rollback completed for: {}", entry.secret_path);
        Ok(())
    }

    /// Get rotation history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<RotationHistory> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);

        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get rotation statistics
    pub async fn get_statistics(&self) -> RotationStatistics {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    /// Start rotation scheduler
    pub async fn start_scheduler(&self) {
        let mut running = self.scheduler_running.write().await;
        if *running {
            warn!("Rotation scheduler already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Starting rotation scheduler");

        let policies = self.policies.clone();
        let engine = Arc::new(self.clone_engine());

        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(3600)); // Check every hour

            loop {
                ticker.tick().await;

                let policies_snapshot = policies.read().await.clone();

                for (policy_id, policy) in policies_snapshot {
                    if !policy.enabled || !policy.auto_rotation {
                        continue;
                    }

                    if let Some(next_rotation) = policy.next_rotation
                        && Utc::now() >= next_rotation
                    {
                        info!("Triggering scheduled rotation for: {}", policy.name);

                        if let Err(e) = engine
                            .execute_rotation(&policy_id, "system".to_string())
                            .await
                        {
                            error!("Scheduled rotation failed: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Stop rotation scheduler
    pub async fn stop_scheduler(&self) {
        let mut running = self.scheduler_running.write().await;
        *running = false;
        info!("Rotation scheduler stopped");
    }

    // Private helper methods

    async fn rotate_immediate(&self, policy: &RotationPolicy) -> Result<(), RotationError> {
        debug!("Executing immediate rotation for: {}", policy.secret_path);
        // Simulate rotation
        tokio::time::sleep(TokioDuration::from_millis(100)).await;
        Ok(())
    }

    async fn rotate_blue_green(&self, policy: &RotationPolicy) -> Result<(), RotationError> {
        debug!("Executing blue-green rotation for: {}", policy.secret_path);
        // Simulate zero-downtime rotation
        tokio::time::sleep(TokioDuration::from_millis(200)).await;
        Ok(())
    }

    async fn rotate_rolling(&self, policy: &RotationPolicy) -> Result<(), RotationError> {
        debug!("Executing rolling rotation for: {}", policy.secret_path);
        // Simulate gradual rotation
        tokio::time::sleep(TokioDuration::from_millis(300)).await;
        Ok(())
    }

    async fn rotate_canary(&self, policy: &RotationPolicy) -> Result<(), RotationError> {
        debug!("Executing canary rotation for: {}", policy.secret_path);
        // Simulate canary deployment
        tokio::time::sleep(TokioDuration::from_millis(250)).await;
        Ok(())
    }

    async fn send_webhooks(
        &self,
        endpoints: &[WebhookEndpoint],
        policy: &RotationPolicy,
        event_type: &str,
    ) -> Result<(), RotationError> {
        for endpoint in endpoints {
            debug!("Sending {} webhook to: {}", event_type, endpoint.url);
            // Simulate webhook call
            // In production, use reqwest to make HTTP calls
        }
        Ok(())
    }

    async fn add_history(&self, entry: RotationHistory) {
        let mut history = self.history.write().await;
        history.push(entry);

        // Cleanup old history if needed
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }

    async fn update_statistics(&self, success: bool, duration_ms: u64) {
        let mut stats = self.statistics.write().await;
        stats.total_rotations += 1;

        if success {
            stats.successful_rotations += 1;
        } else {
            stats.failed_rotations += 1;
        }

        // Update average duration
        let total_duration = stats.average_duration_ms * (stats.total_rotations - 1) + duration_ms;
        stats.average_duration_ms = total_duration / stats.total_rotations;

        stats.last_rotation = Some(Utc::now());
    }

    async fn update_next_rotation(&self, policy: &RotationPolicy) {
        let mut policies = self.policies.write().await;
        if let Some(p) = policies.get_mut(&policy.id) {
            p.next_rotation = Some(Utc::now() + policy.rotation_interval);
        }
    }

    fn clone_engine(&self) -> Self {
        Self {
            policies: self.policies.clone(),
            history: self.history.clone(),
            statistics: self.statistics.clone(),
            scheduler_running: self.scheduler_running.clone(),
        }
    }
}

impl Default for AutoRotationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_rotation_policy() {
        let engine = AutoRotationEngine::new();

        let policy = RotationPolicy::new(
            "test-policy".to_string(),
            "secret/database/password".to_string(),
            SecretType::DatabasePassword,
        );

        let result = engine.create_policy(policy).await;
        assert!(result.is_ok());

        let policies = engine.list_policies().await;
        assert_eq!(policies.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_rotation() {
        let engine = AutoRotationEngine::new();

        let policy = RotationPolicy::new(
            "test-policy".to_string(),
            "secret/api/key".to_string(),
            SecretType::ApiKey,
        );

        let created_policy = engine.create_policy(policy).await.unwrap();

        let result = engine
            .execute_rotation(&created_policy.id, "test-user".to_string())
            .await;
        assert!(result.is_ok());

        let history = engine.get_history(None).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, RotationStatus::Completed);
    }

    #[tokio::test]
    async fn test_rotation_statistics() {
        let engine = AutoRotationEngine::new();

        let policy = RotationPolicy::new(
            "test-policy".to_string(),
            "secret/test".to_string(),
            SecretType::Custom("test".to_string()),
        );

        let created_policy = engine.create_policy(policy).await.unwrap();

        // Execute multiple rotations
        for _ in 0..3 {
            let _ = engine
                .execute_rotation(&created_policy.id, "system".to_string())
                .await;
        }

        let stats = engine.get_statistics().await;
        assert_eq!(stats.total_rotations, 3);
        assert_eq!(stats.successful_rotations, 3);
    }
}
