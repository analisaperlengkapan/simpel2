//! Cache invalidation event listener
//!
//! This module provides an event listener that automatically invalidates
//! cache entries when relevant events occur in the system.

use crate::error::Result;
use crate::events::{Event as SystemEvent, EventError, EventListener, EventType as SystemEventType};
use crate::models::events::EventType;
use crate::services::cache::CacheInvalidationService;
use async_trait::async_trait;
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
    async fn handle_invalidation(&self, event: &SystemEvent) -> Result<()> {
        // Convert system event type to model event type for processing
        let event_type = self.map_event_type(&event.event_type);

        match event_type {
            Some(EventType::UpdateProfile)
            | Some(EventType::UpdateEmail)
            | Some(EventType::UpdateCredential) => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating cache for user update: {}", user_id);
                    self.invalidation_service
                        .invalidate_user(&user_id.to_string())
                        .await?;
                }
            }
            Some(EventType::MfaSetup)
            | Some(EventType::MfaDisabled)
            | Some(EventType::MfaReset) => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating MFA cache for user: {}", user_id);
                    self.invalidation_service
                        .invalidate_mfa_status(&user_id.to_string())
                        .await?;
                }
            }
            Some(EventType::Logout) => {
                if let Some(session_id) = &event.session_id {
                    debug!("Invalidating session cache: {}", session_id);
                    self.invalidation_service
                        .invalidate_session(&session_id.to_string())
                        .await?;
                }
            }
            Some(EventType::GrantConsent) | Some(EventType::RevokeGrant) => {
                if let Some(user_id) = &event.user_id {
                    debug!("Invalidating permissions cache for user: {}", user_id);
                    self.invalidation_service
                        .invalidate_permissions(&user_id.to_string())
                        .await?;
                }
            }
            _ => {
                // Event doesn't require cache invalidation
                debug!("Event {:?} does not require cache invalidation", event.event_type);
            }
        }

        Ok(())
    }

    /// Map system event  model event type
    fn map_event_type(&self, event_type: &SystemEventType) -> Option<EventType> {
        match event_type {
            SystemEventType::UserUpdated => Some(EventType::UpdateProfile),
            SystemEventType::UserLogin => Some(EventType::Login),
            SystemEventType::UserLogout => Some(EventType::Logout),
            SystemEventType::MfaSuccess => Some(EventType::MfaSetup),
            SystemEventType::SessionTerminated => Some(EventType::Logout),
            SystemEventType::Custom(s) if s == "UPDATE_EMAIL" => Some(EventType::UpdateEmail),
            SystemEventType::Custom(s) if s == "UPDATE_CREDENTIAL" => {
                Some(EventType::UpdateCredential)
            }
            SystemEventType::Custom(s) if s == "MFA_DISABLED" => Some(EventType::MfaDisabled),
            SystemEventType::Custom(s) if s == "MFA_RESET" => Some(EventType::MfaReset),
            SystemEventType::Custom(s) if s == "GRANT_CONSENT" => Some(EventType::GrantConsent),
            SystemEventType::Custom(s) if s == "REVOKE_GRANT" => Some(EventType::RevokeGrant),
            _ => None,
        }
    }
}

#[async_trait]
impl EventListener for CacheInvalidationListener {
    fn name(&self) -> &str {
        &self.name
    }

    fn listener_type(&self) -> &str {
        "cache-invalidation"
    }

    fn accepts(&self, event_type: &SystemEventType) -> bool {
        // Accept events that might require cache invalidation
        matches!(
            event_type,
            SystemEventType::UserUpdated
                | SystemEventType::UserLogin
                | SystemEventType::UserLogout
                | SystemEventType::MfaSuccess
                | SystemEventType::MfaFailure
                | SystemEventType::SessionTerminated
                | SystemEventType::Custom(_)
        )
    }

    async fn handle(&self, event: &SystemEvent) -> std::result::Result<(), EventError> {
        self.handle_invalidation(event)
            .await
            .map_err(|e| EventError::ListenerFailed(e.to_string()))
    }

    fn is_async(&self) -> bool {
        true // Cache invalidation should be async
    }

    fn priority(&self) -> i32 {
        30 // Medium-high priority (after logging but before webhooks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;
    use crate::events::{EventCategory, EventType as SystemEventType};
    use crate::services::cache::{MultiLayerCache, RedisCache};
    use uuid::Uuid;

    async fn create_test_listener() -> CacheInvalidationListener {
        let redis_config = RedisConfig {
            enabled: true,
            url: "redis://localhost:6379/15".to_string(),
            ..Default::default()
        };

        let redis_cache = Arc::new(RedisCache::new(&redis_config).await.unwrap());
        let multi_cache = Arc::new(MultiLayerCache::with_defaults(redis_cache));

        let invalidation_service =
            Arc::new(CacheInvalidationService::new(multi_cache, None, None, None).await.unwrap());

        CacheInvalidationListener::new(invalidation_service)
    }

    #[tokio::test]
    async fn test_listener_accepts_relevant_events() {
        let listener = create_test_listener().await;

        assert!(listener.accepts(&SystemEventType::UserUpdated));
        assert!(listener.accepts(&SystemEventType::UserLogout));
        assert!(listener.accepts(&SystemEventType::MfaSuccess));
        assert!(!listener.accepts(&SystemEventType::SystemStartup));
    }

    #[tokio::test]
    async fn test_listener_handles_user_update() {
        let listener = create_test_listener().await;

        let event = SystemEvent::new(
            Uuid::new_v4(),
            SystemEventType::UserUpdated,
            EventCategory::User,
        )
        .with_user(Uuid::new_v4(), "testuser");

        let result = listener.handle(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_listener_priority() {
        let listener = create_test_listener().await;
        assert_eq!(listener.priority(), 30);
    }

    #[tokio::test]
    async fn test_listener_is_async() {
        let listener = create_test_listener().await;
        assert!(listener.is_async());
    }
}

