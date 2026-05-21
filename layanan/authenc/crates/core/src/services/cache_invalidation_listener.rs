//! Cache invalidation event listener
//!
//! This module provides an event listener that automatically invalidates
//! cache entries when relevant events occur in the system.

use crate::services::cache::CacheInvalidationService;
use crate::services::events::EventListenerProvider;
use async_trait::async_trait;
use authenc_types::Result;
use authenc_types::domain::events::{AdminEvent, Event, EventType};
use std::sync::Arc;
use tracing::debug;

/// Event listener that invalidates cache based on system events
pub struct CacheInvalidationListener {
    /// Cache invalidation service
    invalidation_service: Arc<CacheInvalidationService>,
    /// Listener name
    name: String,
}

impl CacheInvalidationListener {
    /// Create a new cache invalidation listener
    pub fn new(invalidation_service: Arc<CacheInvalidationService>) -> Self {
        Self {
            invalidation_service,
            name: "cache-invalidation-listener".to_string(),
        }
    }

    /// Handle cache invalidation for an event
    async fn handle_invalidation(&self, event: &Event) -> Result<()> {
        match event.event_type {
            EventType::UpdateProfile | EventType::UpdateEmail | EventType::UpdateCredential => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating cache for user update: {}", user_id);
                    self.invalidation_service.invalidate_user(user_id).await?;
                }
            }
            EventType::MfaSetup | EventType::MfaDisabled | EventType::MfaReset => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating MFA cache for user: {}", user_id);
                    self.invalidation_service
                        .invalidate_mfa_status(user_id)
                        .await?;
                }
            }
            EventType::Logout => {
                if let Some(session_id) = &event.session_id {
                    debug!("Invalidating session cache: {}", session_id);
                    self.invalidation_service
                        .invalidate_session(session_id)
                        .await?;
                }
            }
            EventType::GrantConsent | EventType::RevokeGrant => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating permissions cache for user: {}", user_id);
                    self.invalidation_service
                        .invalidate_permissions(user_id)
                        .await?;
                }
            }
            _ => {
                // Event doesn't require cache invalidation
                debug!(
                    "Event {:?} does not require cache invalidation",
                    event.event_type
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventListenerProvider for CacheInvalidationListener {
    async fn on_event(&self, event: &Event) -> Result<()> {
        self.handle_invalidation(event).await
    }

    async fn on_admin_event(
        &self,
        _event: &AdminEvent,
        _include_representation: bool,
    ) -> Result<()> {
        // Admin events currently don't trigger cache invalidation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;
    use crate::services::cache::{MultiLayerCache, RedisCache};
    use std::time::Duration;
    use uuid::Uuid;

    /// Get Redis URL from environment or use default with password for docker
    fn get_test_redis_url() -> String {
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:redis_password@localhost:6379/15".to_string())
    }

    async fn create_test_listener()
    -> std::result::Result<CacheInvalidationListener, crate::error::AuthencError> {
        let redis_config = RedisConfig {
            enabled: true,
            url: get_test_redis_url(),
            ..Default::default()
        };

        let redis_cache = match tokio::time::timeout(
            Duration::from_secs(3),
            RedisCache::new(&redis_config),
        )
        .await
        {
            Ok(Ok(cache)) => Arc::new(cache),
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(crate::error::AuthencError::internal(
                    "Redis connection timed out - Redis not available",
                ));
            }
        };
        let multi_cache = Arc::new(MultiLayerCache::with_defaults(redis_cache));

        let invalidation_service = Arc::new(
            CacheInvalidationService::new(multi_cache, None, None, None)
                .await
                .unwrap(),
        );

        Ok(CacheInvalidationListener::new(invalidation_service))
    }

    #[tokio::test]
    async fn test_listener_handles_user_update() {
        let listener = match create_test_listener().await {
            Ok(l) => l,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let event = Event::new(EventType::UpdateProfile, "test-realm".to_string())
            .user_id(Uuid::new_v4().to_string());

        let result = listener.on_event(&event).await;
        assert!(result.is_ok());
    }
}
