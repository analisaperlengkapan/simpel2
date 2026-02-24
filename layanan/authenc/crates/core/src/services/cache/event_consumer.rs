//! Kafka integration (requires `kafka` feature)
#![cfg(feature = "kafka")]

//! Event-driven cache invalidation consumer
//!
//! This module provides a Kafka consumer that subscribes to permission change
//! and user update events, automatically invalidating relevant cache entries.

use super::CacheInvalidationService;
use authenc_types::domain::events::{AdminEvent, Event, EventType, OperationType, ResourceType};
use authenc_types::{AuthencError, Result};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Configuration for the event-driven cache invalidation consumer
#[derive(Debug, Clone)]
pub struct EventConsumerConfig {
    /// Kafka broker addresses (comma-separated)
    pub brokers: String,
    /// User events topic to subscribe to
    pub user_events_topic: String,
    /// Admin events topic to subscribe to
    pub admin_events_topic: String,
    /// Consumer group ID
    pub consumer_group: String,
    /// Enable auto-commit
    pub enable_auto_commit: bool,
    /// Auto-offset reset strategy (earliest, latest)
    pub auto_offset_reset: String,
}

impl Default for EventConsumerConfig {
    fn default() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            user_events_topic: "authenc.user.events".to_string(),
            admin_events_topic: "authenc.admin.events".to_string(),
            consumer_group: "authenc-cache-invalidation".to_string(),
            enable_auto_commit: true,
            auto_offset_reset: "latest".to_string(),
        }
    }
}

/// Statistics for the event consumer
#[derive(Debug, Clone, Default)]
pub struct EventConsumerStats {
    /// Total events processed
    pub total_events: u64,
    /// User events processed
    pub user_events: u64,
    /// Admin events processed
    pub admin_events: u64,
    /// Events that triggered cache invalidation
    pub invalidations_triggered: u64,
    /// Failed event processing
    pub processing_failures: u64,
    /// Deserialization errors
    pub deserialization_errors: u64,
}

/// Event-driven cache invalidation consumer
pub struct EventDrivenCacheInvalidator {
    /// Cache invalidation service
    invalidation_service: Arc<CacheInvalidationService>,
    /// Kafka consumer for user events
    user_consumer: Arc<StreamConsumer>,
    /// Kafka consumer for admin events
    admin_consumer: Arc<StreamConsumer>,
    /// Configuration
    config: EventConsumerConfig,
    /// Whether the consumer is running
    running: Arc<RwLock<bool>>,
    /// Consumer statistics
    stats: Arc<RwLock<EventConsumerStats>>,
}

impl EventDrivenCacheInvalidator {
    /// Create a new event-driven cache invalidator
    ///
    /// # Arguments
    /// * `invalidation_service` - The cache invalidation service to use
    /// * `config` - Consumer configuration
    pub fn new(
        invalidation_service: Arc<CacheInvalidationService>,
        config: EventConsumerConfig,
    ) -> Result<Self> {
        // Create consumer for user events
        let user_consumer = Self::create_consumer(&config, &config.user_events_topic)?;

        // Create consumer for admin events
        let admin_consumer = Self::create_consumer(&config, &config.admin_events_topic)?;

        info!(
            "Event-driven cache invalidator created (user topic: {}, admin topic: {})",
            config.user_events_topic, config.admin_events_topic
        );

        Ok(Self {
            invalidation_service,
            user_consumer: Arc::new(user_consumer),
            admin_consumer: Arc::new(admin_consumer),
            config,
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(EventConsumerStats::default())),
        })
    }

    /// Create a Kafka consumer
    fn create_consumer(config: &EventConsumerConfig, topic: &str) -> Result<StreamConsumer> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.consumer_group)
            .set(
                "enable.auto.commit",
                if config.enable_auto_commit {
                    "true"
                } else {
                    "false"
                },
            )
            .set("auto.offset.reset", &config.auto_offset_reset)
            .set("session.timeout.ms", "30000")
            .set("heartbeat.interval.ms", "3000")
            .create()
            .map_err(|e| {
                AuthencError::internal(format!("Failed to create Kafka consumer: {}", e))
            })?;

        consumer.subscribe(&[topic]).map_err(|e| {
            AuthencError::internal(format!(
                "Failed to subscribe to Kafka topic {}: {}",
                topic, e
            ))
        })?;

        Ok(consumer)
    }

    /// Start the event consumer
    ///
    /// This will start background tasks that listen for Kafka events
    /// and trigger cache invalidation accordingly.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        *running = true;
        drop(running);

        // Start user events consumer task
        self.start_user_events_consumer().await;

        // Start admin events consumer task
        self.start_admin_events_consumer().await;

        info!("Event-driven cache invalidator started");
        Ok(())
    }

    /// Start the user events consumer task
    async fn start_user_events_consumer(&self) {
        let consumer = Arc::clone(&self.user_consumer);
        let invalidation_service = Arc::clone(&self.invalidation_service);
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            info!("User events consumer task started");

            while *running.read().await {
                match consumer.recv().await {
                    Ok(message) => {
                        if let Some(payload) = message.payload() {
                            match serde_json::from_slice::<Event>(payload) {
                                Ok(event) => {
                                    debug!("Received user event: {:?}", event.event_type);

                                    // Update stats
                                    {
                                        let mut stats = stats.write().await;
                                        stats.total_events += 1;
                                        stats.user_events += 1;
                                    }

                                    // Process the event
                                    if let Err(e) = Self::process_user_event(
                                        &invalidation_service,
                                        &event,
                                        &stats,
                                    )
                                    .await
                                    {
                                        error!("Failed to process user event: {}", e);
                                        let mut stats = stats.write().await;
                                        stats.processing_failures += 1;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to deserialize user event: {}", e);
                                    let mut stats = stats.write().await;
                                    stats.deserialization_errors += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Kafka consumer error (user events): {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            info!("User events consumer task stopped");
        });
    }

    /// Start the admin events consumer task
    async fn start_admin_events_consumer(&self) {
        let consumer = Arc::clone(&self.admin_consumer);
        let invalidation_service = Arc::clone(&self.invalidation_service);
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            info!("Admin events consumer task started");

            while *running.read().await {
                match consumer.recv().await {
                    Ok(message) => {
                        if let Some(payload) = message.payload() {
                            match serde_json::from_slice::<AdminEvent>(payload) {
                                Ok(event) => {
                                    debug!(
                                        "Received admin event: {:?} on {:?}",
                                        event.operation_type, event.resource_type
                                    );

                                    // Update stats
                                    {
                                        let mut stats = stats.write().await;
                                        stats.total_events += 1;
                                        stats.admin_events += 1;
                                    }

                                    // Process the event
                                    if let Err(e) = Self::process_admin_event(
                                        &invalidation_service,
                                        &event,
                                        &stats,
                                    )
                                    .await
                                    {
                                        error!("Failed to process admin event: {}", e);
                                        let mut stats = stats.write().await;
                                        stats.processing_failures += 1;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to deserialize admin event: {}", e);
                                    let mut stats = stats.write().await;
                                    stats.deserialization_errors += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Kafka consumer error (admin events): {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            info!("Admin events consumer task stopped");
        });
    }

    /// Process a user event and trigger cache invalidation if needed
    async fn process_user_event(
        invalidation_service: &Arc<CacheInvalidationService>,
        event: &Event,
        stats: &Arc<RwLock<EventConsumerStats>>,
    ) -> Result<()> {
        let should_invalidate = match event.event_type {
            // User profile updates
            EventType::UpdateProfile | EventType::UpdateEmail | EventType::UpdateCredential => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating cache for user update: {}", user_id);
                    invalidation_service.invalidate_user(user_id).await?;
                    true
                } else {
                    false
                }
            }

            // MFA status changes
            EventType::MfaSetup
            | EventType::MfaEnabled
            | EventType::MfaDisabled
            | EventType::MfaReset => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating MFA cache for user: {}", user_id);
                    invalidation_service.invalidate_mfa_status(user_id).await?;
                    true
                } else {
                    false
                }
            }

            // Session management
            EventType::Logout => {
                if let Some(session_id) = &event.session_id {
                    debug!("Invalidating session cache: {}", session_id);
                    invalidation_service.invalidate_session(session_id).await?;
                    true
                } else {
                    false
                }
            }

            // Password changes
            EventType::UpdatePassword | EventType::ResetPassword => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating cache for password change: {}", user_id);
                    invalidation_service.invalidate_user(user_id).await?;
                    // Also invalidate all sessions for this user
                    true
                } else {
                    false
                }
            }

            // Consent and permissions
            EventType::GrantConsent | EventType::RevokeGrant => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating permissions cache for user: {}", user_id);
                    invalidation_service.invalidate_permissions(user_id).await?;
                    true
                } else {
                    false
                }
            }

            _ => {
                // Event doesn't require cache invalidation
                debug!(
                    "Event {:?} does not require cache invalidation",
                    event.event_type
                );
                false
            }
        };

        if should_invalidate {
            let mut stats = stats.write().await;
            stats.invalidations_triggered += 1;
        }

        Ok(())
    }

    /// Process an admin event and trigger cache invalidation if needed
    async fn process_admin_event(
        invalidation_service: &Arc<CacheInvalidationService>,
        event: &AdminEvent,
        stats: &Arc<RwLock<EventConsumerStats>>,
    ) -> Result<()> {
        let should_invalidate = match (&event.resource_type, &event.operation_type) {
            // User resource changes
            (ResourceType::User, OperationType::Update | OperationType::Delete) => {
                // Extract user ID from resource path (e.g., "/realms/test/users/user123")
                if let Some(user_id) = Self::extract_user_id_from_path(&event.resource_path) {
                    debug!("Invalidating cache for admin user update: {}", user_id);
                    invalidation_service.invalidate_user(&user_id).await?;
                    true
                } else {
                    false
                }
            }

            // Permission and role changes
            (ResourceType::Permission, _)
            | (ResourceType::RealmRole, _)
            | (ResourceType::RealmRoleMapping, _) => {
                // Extract user ID if available
                if let Some(user_id) = Self::extract_user_id_from_path(&event.resource_path) {
                    debug!("Invalidating permissions cache for user: {}", user_id);
                    invalidation_service
                        .invalidate_permissions(&user_id)
                        .await?;
                    invalidation_service.invalidate_roles(&user_id).await?;
                    true
                } else {
                    // If no specific user, this might be a global role change
                    // We could implement a more aggressive invalidation strategy here
                    debug!("Global permission/role change detected");
                    false
                }
            }

            // Group membership changes
            (ResourceType::GroupMembership, _) => {
                if let Some(user_id) = Self::extract_user_id_from_path(&event.resource_path) {
                    debug!(
                        "Invalidating cache for group membership change: {}",
                        user_id
                    );
                    invalidation_service
                        .invalidate_permissions(&user_id)
                        .await?;
                    invalidation_service.invalidate_roles(&user_id).await?;
                    true
                } else {
                    false
                }
            }

            // User session management
            (ResourceType::UserSession, OperationType::Delete) => {
                if let Some(session_id) = Self::extract_session_id_from_path(&event.resource_path) {
                    debug!("Invalidating session cache: {}", session_id);
                    invalidation_service.invalidate_session(&session_id).await?;
                    true
                } else {
                    false
                }
            }

            _ => {
                // Event doesn't require cache invalidation
                debug!(
                    "Admin event {:?} on {:?} does not require cache invalidation",
                    event.operation_type, event.resource_type
                );
                false
            }
        };

        if should_invalidate {
            let mut stats = stats.write().await;
            stats.invalidations_triggered += 1;
        }

        Ok(())
    }

    /// Extract user ID from resource path
    ///
    /// Examples:
    /// - "/realms/test/users/user123" -> Some("user123")
    /// - "/users/user456/permissions" -> Some("user456")
    fn extract_user_id_from_path(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();

        // Look for "users" segment followed by user ID
        for (i, part) in parts.iter().enumerate() {
            if *part == "users" && i + 1 < parts.len() {
                return Some(parts[i + 1].to_string());
            }
        }

        None
    }

    /// Extract session ID from resource path
    ///
    /// Examples:
    /// - "/realms/test/sessions/session123" -> Some("session123")
    /// - "/users/user456/sessions/session789" -> Some("session789")
    fn extract_session_id_from_path(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();

        // Look for "sessions" segment followed by session ID
        for (i, part) in parts.iter().enumerate() {
            if *part == "sessions" && i + 1 < parts.len() {
                return Some(parts[i + 1].to_string());
            }
        }

        None
    }

    /// Stop the event consumer
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Event-driven cache invalidator stopping...");
    }

    /// Check if the consumer is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get consumer statistics
    pub async fn get_stats(&self) -> EventConsumerStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = EventConsumerStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id_from_path() {
        assert_eq!(
            EventDrivenCacheInvalidator::extract_user_id_from_path("/realms/test/users/user123"),
            Some("user123".to_string())
        );

        assert_eq!(
            EventDrivenCacheInvalidator::extract_user_id_from_path("/users/user456/permissions"),
            Some("user456".to_string())
        );

        assert_eq!(
            EventDrivenCacheInvalidator::extract_user_id_from_path("/realms/test/roles"),
            None
        );
    }

    #[test]
    fn test_extract_session_id_from_path() {
        assert_eq!(
            EventDrivenCacheInvalidator::extract_session_id_from_path(
                "/realms/test/sessions/session123"
            ),
            Some("session123".to_string())
        );

        assert_eq!(
            EventDrivenCacheInvalidator::extract_session_id_from_path(
                "/users/user456/sessions/session789"
            ),
            Some("session789".to_string())
        );

        assert_eq!(
            EventDrivenCacheInvalidator::extract_session_id_from_path("/realms/test/users"),
            None
        );
    }

    #[test]
    fn test_event_consumer_config_default() {
        let config = EventConsumerConfig::default();
        assert_eq!(config.brokers, "localhost:9092");
        assert_eq!(config.user_events_topic, "authenc.user.events");
        assert_eq!(config.admin_events_topic, "authenc.admin.events");
        assert_eq!(config.consumer_group, "authenc-cache-invalidation");
        assert!(config.enable_auto_commit);
        assert_eq!(config.auto_offset_reset, "latest");
    }
}
