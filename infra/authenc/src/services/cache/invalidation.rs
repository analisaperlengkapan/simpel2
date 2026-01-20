//! Cache invalidation mechanism
//!
//! This module provides cache invalidation strategies including:
//! - Kafka-based event-driven invalidation
//! - Permission/role change invalidation
//! - User update invalidation
//! - Cache warming on startup

use super::{Cache, CacheKeys, MultiLayerCache};
use crate::error::{AuthencError, Result};
use crate::models::events::{Event, EventType};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Cache invalidation event types
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidationEvent {
    /// User data was updated
    UserUpdated { user_id: String },
    /// User permissions changed
    PermissionChanged { user_id: String },
    /// User role changed
    RoleChanged { user_id: String },
    /// User session invalidated
    SessionInvalidated { session_id: String },
    /// MFA status changed
    MfaStatusChanged { user_id: String },
    /// Global cache clear
    ClearAll,
}

impl InvalidationEvent {
    /// Get cache keys that should be invalidated for this event
    pub fn get_cache_keys(&self) -> Vec<String> {
        match self {
            InvalidationEvent::UserUpdated { user_id } => {
                vec![
                    format!("user:{}", user_id),
                    format!("permissions:{}:*", user_id),
                    format!("session:user:{}", user_id),
                ]
            }
            InvalidationEvent::PermissionChanged { user_id } => {
                vec![
                    format!("permissions:{}:*", user_id),
                    format!("user:{}", user_id),
                ]
            }
            InvalidationEvent::RoleChanged { user_id } => {
                vec![
                    format!("permissions:{}:*", user_id),
                    format!("user:{}", user_id),
                ]
            }
            InvalidationEvent::SessionInvalidated { session_id } => {
                vec![format!("session:{}", session_id)]
            }
            InvalidationEvent::MfaStatusChanged { user_id } => {
                vec![CacheKeys::mfa_status(user_id), format!("user:{}", user_id)]
            }
            InvalidationEvent::ClearAll => vec!["*".to_string()],
        }
    }

    /// Convert from Kafka event
    pub fn from_kafka_event(event: &Event) -> Option<Self> {
        match event.event_type {
            EventType::UpdateProfile | EventType::UpdateEmail | EventType::UpdateCredential => {
                event
                    .user_id
                    .as_ref()
                    .map(|user_id| InvalidationEvent::UserUpdated {
                        user_id: user_id.clone(),
                    })
            }
            EventType::MfaSetup | EventType::MfaDisabled | EventType::MfaReset => event
                .user_id
                .as_ref()
                .map(|user_id| InvalidationEvent::MfaStatusChanged {
                    user_id: user_id.clone(),
                }),
            EventType::Logout => {
                event
                    .session_id
                    .as_ref()
                    .map(|session_id| InvalidationEvent::SessionInvalidated {
                        session_id: session_id.clone(),
                    })
            }
            _ => None,
        }
    }
}

/// Cache invalidation service
pub struct CacheInvalidationService {
    /// Multi-layer cache to invalidate
    cache: Arc<MultiLayerCache>,
    /// Kafka consumer for cache invalidation events
    consumer: Option<Arc<StreamConsumer>>,
    /// Whether the service is running
    running: Arc<RwLock<bool>>,
    /// Invalidation statistics
    stats: Arc<RwLock<InvalidationStats>>,
}

/// Statistics for cache invalidation
#[derive(Debug, Clone, Default)]
pub struct InvalidationStats {
    /// Total invalidation events processed
    pub total_events: u64,
    /// User update invalidations
    pub user_updates: u64,
    /// Permission change invalidations
    pub permission_changes: u64,
    /// Role change invalidations
    pub role_changes: u64,
    /// Session invalidations
    pub session_invalidations: u64,
    /// MFA status change invalidations
    pub mfa_changes: u64,
    /// Failed invalidations
    pub failures: u64,
}

impl CacheInvalidationService {
    /// Create a new cache invalidation service
    ///
    /// # Arguments
    /// * `cache` - The multi-layer cache to invalidate
    /// * `kafka_brokers` - Kafka broker addresses (optional, for eviven invalidation)
    /// * `kafka_topic` - Kafka topic to subscribe to for invalidation events
    /// * `consumer_group` - Kafka consumer group ID
    pub async fn new(
        cache: Arc<MultiLayerCache>,
        kafka_brokers: Option<&str>,
        kafka_topic: Option<&str>,
        consumer_group: Option<&str>,
    ) -> Result<Self> {
        let consumer = if let (Some(brokers), Some(topic), Some(group)) =
            (kafka_brokers, kafka_topic, consumer_group)
        {
            match Self::create_kafka_consumer(brokers, topic, group) {
                Ok(consumer) => {
                    info!(
                        "Cache invalidation service initialized with Kafka consumer (topic: {})",
                        topic
                    );
                    Some(Arc::new(consumer))
                }
                Err(e) => {
                    warn!(
                        "Failed to create Kafka consumer for cache invalidation: {}. \
                         Cache invalidation will work via direct API calls only.",
                        e
                    );
                    None
                }
            }
        } else {
            debug!("Cache invalidation service initialized without Kafka consumer");
            None
        };

        Ok(Self {
            cache,
            consumer,
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(InvalidationStats::default())),
        })
    }

    /// Create Kafka consumer for cache invalidation events
    fn create_kafka_consumer(brokers: &str, topic: &str, group_id: &str) -> Result<StreamConsumer> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest") // Only process new events
            .create()
            .map_err(|e| {
                AuthencError::internal(format!("Failed to create Kafka consumer: {}", e))
            })?;

        consumer.subscribe(&[topic]).map_err(|e| {
            AuthencError::internal(format!("Failed to subscribe to Kafka topic: {}", e))
        })?;

        Ok(consumer)
    }

    /// Start the cache invalidation service
    ///
    /// This will start a background task that listens for Kafka events
    /// and invalidates cache entries accordingly.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        *running = true;
        drop(running);

        if let Some(consumer) = &self.consumer {
            let consumer = Arc::clone(consumer);
            let cache = Arc::clone(&self.cache);
            let running = Arc::clone(&self.running);
            let stats = Arc::clone(&self.stats);

            tokio::spawn(async move {
                info!("Cache invalidation service started");

                while *running.read().await {
                    match consumer.recv().await {
                        Ok(message) => {
                            if let Some(payload) = message.payload() {
                                match serde_json::from_slice::<Event>(payload) {
                                    Ok(event) => {
                                        if let Some(invalidation_event) =
                                            InvalidationEvent::from_kafka_event(&event)
                                        {
                                            debug!(
                                                "Received cache invalidation event: {:?}",
                                                invalidation_event
                                            );

                                            if let Err(e) = Self::process_invalidation_event(
                                                &cache,
                                                &invalidation_event,
                                                &stats,
                                            )
                                            .await
                                            {
                                                error!(
                                                    "Failed to process invalidation event: {}",
                                                    e
                                                );
                                                let mut stats = stats.write().await;
                                                stats.failures += 1;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to deserialize Kafka event: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Kafka consumer error: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }

                info!("Cache invalidation service stopped");
            });
        }

        Ok(())
    }

    /// Stop the cache invalidation service
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Cache invalidation service stopping...");
    }

    /// Process a cache invalidation event
    async fn process_invalidation_event(
        cache: &Arc<MultiLayerCache>,
        event: &InvalidationEvent,
        stats: &Arc<RwLock<InvalidationStats>>,
    ) -> Result<()> {
        let cache_keys = event.get_cache_keys();

        for key_pattern in cache_keys {
            if key_pattern.ends_with("*") {
                // Pattern-based invalidation (e.g., "permissions:user123:*")
                // For now, we'll clear L1 cache for pattern matches
                // L2 (Redis) would need SCAN command for pattern matching
                debug!("Pattern-based cache invalidation: {}", key_pattern);
                cache.clear_l1();
            } else {
                // Exact key invalidation
                if let Err(e) = cache.delete(&key_pattern).await {
                    warn!("Failed to invalidate cache key {}: {}", key_pattern, e);
                }
            }
        }

        // Update statistics
        let mut stats = stats.write().await;
        stats.total_events += 1;

        match event {
            InvalidationEvent::UserUpdated { .. } => stats.user_updates += 1,
            InvalidationEvent::PermissionChanged { .. } => stats.permission_changes += 1,
            InvalidationEvent::RoleChanged { .. } => stats.role_changes += 1,
            InvalidationEvent::SessionInvalidated { .. } => stats.session_invalidations += 1,
            InvalidationEvent::MfaStatusChanged { .. } => stats.mfa_changes += 1,
            InvalidationEvent::ClearAll => {
                cache.clear_l1();
            }
        }

        Ok(())
    }

    /// Manually invalidate cache for a user
    pub async fn invalidate_user(&self, user_id: &str) -> Result<()> {
        let event = InvalidationEvent::UserUpdated {
            user_id: user_id.to_string(),
        };
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Manually invalidate cache for user permissions
    pub async fn invalidate_permissions(&self, user_id: &str) -> Result<()> {
        let event = InvalidationEvent::PermissionChanged {
            user_id: user_id.to_string(),
        };
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Manually invalidate cache for user roles
    pub async fn invalidate_roles(&self, user_id: &str) -> Result<()> {
        let event = InvalidationEvent::RoleChanged {
            user_id: user_id.to_string(),
        };
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Manually invalidate cache for a session
    pub async fn invalidate_session(&self, session_id: &str) -> Result<()> {
        let event = InvalidationEvent::SessionInvalidated {
            session_id: session_id.to_string(),
        };
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Manually invalidate cache for MFA status
    pub async fn invalidate_mfa_status(&self, user_id: &str) -> Result<()> {
        let event = InvalidationEvent::MfaStatusChanged {
            user_id: user_id.to_string(),
        };
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> Result<()> {
        let event = InvalidationEvent::ClearAll;
        Self::process_invalidation_event(&self.cache, &event, &self.stats).await
    }

    /// Get invalidation statistics
    pub async fn get_stats(&self) -> InvalidationStats {
        self.stats.read().await.clone()
    }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// Cache warming service for preloading active user data
pub struct CacheWarmingService {
    /// Multi-layer cache to warm
    cache: Arc<MultiLayerCache>,
}

impl CacheWarmingService {
    /// Create a new cache warming service
    pub fn new(cache: Arc<MultiLayerCache>) -> Self {
        Self { cache }
    }

    /// Warm cache with active user data
    ///
    /// This should be called on startup to preload frequently accessed data
    ///
    /// # Arguments
    /// * `active_users` - List of active user IDs to warm cache for
    /// * `user_data_provider` - Async function to fetch user data
    pub async fn warm_active_users<F, Fut>(
        &self,
        active_users: Vec<String>,
        user_data_provider: F,
    ) -> Result<usize>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<Option<serde_json::Value>>>,
    {
        let mut warmed_count = 0;

        info!(
            "Starting cache warming for {} active users",
            active_users.len()
        );

        for user_id in active_users {
            match user_data_provider(user_id.clone()).await {
                Ok(Some(user_data)) => {
                    let cache_key = format!("user:{}", user_id);
                    if let Err(e) = self
                        .cache
                        .set(&cache_key, &user_data, Duration::from_secs(300))
                        .await
                    {
                        warn!("Failed to warm cache for user {}: {}", user_id, e);
                    } else {
                        warmed_count += 1;
                    }
                }
                Ok(None) => {
                    debug!("No data found for user {} during cache warming", user_id);
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch data for user {} during cache warming: {}",
                        user_id, e
                    );
                }
            }
        }

        info!("Cache warming completed: {} users warmed", warmed_count);
        Ok(warmed_count)
    }

    /// Warm cache with permission data
    pub async fn warm_permissions<F, Fut>(
        &self,
        user_permissions: Vec<(String, String)>, // (user_id, resource)
        permission_data_provider: F,
    ) -> Result<usize>
    where
        F: Fn(String, String) -> Fut,
        Fut: std::future::Future<Output = Result<Option<serde_json::Value>>>,
    {
        let mut warmed_count = 0;

        info!(
            "Starting cache warming for {} permission entries",
            user_permissions.len()
        );

        for (user_id, resource) in user_permissions {
            match permission_data_provider(user_id.clone(), resource.clone()).await {
                Ok(Some(permission_data)) => {
                    let cache_key = format!("permissions:{}:{}", user_id, resource);
                    if let Err(e) = self
                        .cache
                        .set(&cache_key, &permission_data, Duration::from_secs(300))
                        .await
                    {
                        warn!(
                            "Failed to warm cache for permission {}:{}: {}",
                            user_id, resource, e
                        );
                    } else {
                        warmed_count += 1;
                    }
                }
                Ok(None) => {
                    debug!(
                        "No permission data found for {}:{} during cache warming",
                        user_id, resource
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch permission data for {}:{} during cache warming: {}",
                        user_id, resource, e
                    );
                }
            }
        }

        info!(
            "Permission cache warming completed: {} entries warmed",
            warmed_count
        );
        Ok(warmed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;
    use crate::services::cache::{MultiLayerCache, RedisCache};

    /// Get Redis URL from environment or use default with password for docker
    fn get_test_redis_url() -> String {
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:redis_password@localhost:6379/15".to_string())
    }

    async fn create_test_cache() -> std::result::Result<Arc<MultiLayerCache>, AuthencError> {
        let redis_config = RedisConfig {
            enabled: true,
            url: get_test_redis_url(),
            ..Default::default()
        };

        let redis_cache = Arc::new(RedisCache::new(&redis_config).await?);
        Ok(Arc::new(MultiLayerCache::with_defaults(redis_cache)))
    }

    #[tokio::test]
    async fn test_invalidation_event_cache_keys() {
        let event = InvalidationEvent::UserUpdated {
            user_id: "user123".to_string(),
        };

        let keys = event.get_cache_keys();
        assert!(keys.contains(&"user:user123".to_string()));
        assert!(keys.iter().any(|k| k.starts_with("permissions:user123:")));
    }

    #[tokio::test]
    async fn test_invalidation_event_from_kafka() {
        let mut event = Event::new(EventType::UpdateProfile, "realm1".to_string());
        event.user_id = Some("user123".to_string());

        let invalidation_event = InvalidationEvent::from_kafka_event(&event);
        assert!(invalidation_event.is_some());

        if let Some(InvalidationEvent::UserUpdated { user_id }) = invalidation_event {
            assert_eq!(user_id, "user123");
        } else {
            panic!("Expected UserUpdated event");
        }
    }

    #[tokio::test]
    async fn test_cache_invalidation_service_creation() {
        let cache = match create_test_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let service = CacheInvalidationService::new(cache, None, None, None)
            .await
            .unwrap();

        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_manual_invalidation() {
        let cache = match create_test_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Set some test data
        let user_data = serde_json::json!({"id": "user123", "name": "Test User"});
        cache
            .set("user:user123", &user_data, Duration::from_secs(60))
            .await
            .unwrap();

        // Verify data is cached
        assert!(cache.get("user:user123").await.unwrap().is_some());

        // Create invalidation service and invalidate
        let service = CacheInvalidationService::new(Arc::clone(&cache), None, None, None)
            .await
            .unwrap();

        service.invalidate_user("user123").await.unwrap();

        // Verify data is invalidated
        assert!(cache.get("user:user123").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_warming_service() {
        let cache = match create_test_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let warming_service = CacheWarmingService::new(Arc::clone(&cache));

        let active_users = vec!["user1".to_string(), "user2".to_string()];

        let user_data_provider = |user_id: String| async move {
            Ok(Some(serde_json::json!({
                "id": user_id,
                "name": format!("User {}", user_id)
            })))
        };

        let warmed_count = warming_service
            .warm_active_users(active_users, user_data_provider)
            .await
            .unwrap();

        assert_eq!(warmed_count, 2);

        // Verify data is cached
        assert!(cache.get("user:user1").await.unwrap().is_some());
        assert!(cache.get("user:user2").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_invalidation_stats() {
        let cache = match create_test_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let service = CacheInvalidationService::new(cache, None, None, None)
            .await
            .unwrap();

        service.invalidate_user("user1").await.unwrap();
        service.invalidate_permissions("user2").await.unwrap();
        service.invalidate_roles("user3").await.unwrap();

        let stats = service.get_stats().await;
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.user_updates, 1);
        assert_eq!(stats.permission_changes, 1);
        assert_eq!(stats.role_changes, 1);
    }
}
