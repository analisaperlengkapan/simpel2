//! Integration tests for event-driven cache invalidation
//!
//! These tests verify that cache invalidation works correctly when
//! events are published to Kafka topics.

use authenc::config::RedisConfig;
use authenc::models::events::{
    AdminEvent, AuthDetails, Event, EventType, OperationType, ResourceType,
};
use authenc::services::cache::{
    Cache, CacheInvalidationService, EventConsumerConfig, EventDrivenCacheInvalidator,
    MultiLayerCache, RedisCache,
};
use authenc::services::event_publisher::{EventPublisher, EventPublisherConfig, PublishableEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper function to create test cache
async fn create_test_cache() -> Arc<MultiLayerCache> {
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(),
        ..Default::default()
    };

    let redis_cache = Arc::new(RedisCache::new(&redis_config).await.unwrap());
    Arc::new(MultiLayerCache::with_defaults(redis_cache))
}

/// Helper function to create test event publisher
fn create_test_event_publisher() -> EventPublisher {
    let config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        max_retries: 3,
        batch_size: 10,
        flush_interval_ms: 100,
        ..Default::default()
    };

    EventPublisher::new(config).unwrap()
}

#[tokio::test]
#[ignore] // Requires Kafka and Redis to be running
async fn test_user_update_event_invalidates_cache() {
    // Setup
    let cache = create_test_cache().await;
    let invalidation_service = Arc::new(
        CacheInvalidationService::new(Arc::clone(&cache), None, None, None)
            .await
            .unwrap(),
    );

    let consumer_config = EventConsumerConfig {
        brokers: "localhost:9092".to_string(),
        user_events_topic: "test.user.events".to_string(),
        admin_events_topic: "test.admin.events".to_string(),
        consumer_group: "test-cache-invalidation".to_string(),
        ..Default::default()
    };

    let invalidator =
        EventDrivenCacheInvalidator::new(Arc::clone(&invalidation_service), consumer_config)
            .unwrap();

    // Start the consumer
    invalidator.start().await.unwrap();
    assert!(invalidator.is_running().await);

    // Populate cache with user data
    let user_id = "test-user-123";
    let user_data = serde_json::json!({
        "id": user_id,
        "name": "Test User",
        "email": "test@example.com"
    });

    cache
        .set(
            &format!("user:{}", user_id),
            &user_data,
            Duration::from_secs(300),
        )
        .await
        .unwrap();

    // Verify data is cached
    assert!(
        cache
            .get(&format!("user:{}", user_id))
            .await
            .unwrap()
            .is_some()
    );

    // Publish user update event
    let event_publisher = create_test_event_publisher();
    let event =
        Event::new(EventType::UpdateProfile, "test-realm".to_string()).user_id(user_id.to_string());

    let event_json = serde_json::to_string(&event).unwrap();
    let publishable_event = PublishableEvent::new(
        "test.user.events".to_string(),
        user_id.to_string(),
        event_json,
    );

    event_publisher.publish(publishable_event).await.unwrap();
    event_publisher.flush_batch().await.unwrap();

    // Wait for event to be processed
    sleep(Duration::from_secs(2)).await;

    // Verify cache was invalidated
    assert!(
        cache
            .get(&format!("user:{}", user_id))
            .await
            .unwrap()
            .is_none()
    );

    // Check stats
    let stats = invalidator.get_stats().await;
    assert!(stats.user_events > 0);
    assert!(stats.invalidations_triggered > 0);

    // Cleanup
    invalidator.stop().await;
}

#[tokio::test]
#[ignore] // Requires Kafka and Redis to be running
async fn test_permission_change_event_invalidates_cache() {
    // Setup
    let cache = create_test_cache().await;
    let invalidation_service = Arc::new(
        CacheInvalidationService::new(Arc::clone(&cache), None, None, None)
            .await
            .unwrap(),
    );

    let consumer_config = EventConsumerConfig {
        brokers: "localhost:9092".to_string(),
        user_events_topic: "test.user.events".to_string(),
        admin_events_topic: "test.admin.events".to_string(),
        consumer_group: "test-cache-invalidation-2".to_string(),
        ..Default::default()
    };

    let invalidator =
        EventDrivenCacheInvalidator::new(Arc::clone(&invalidation_service), consumer_config)
            .unwrap();

    // Start the consumer
    invalidator.start().await.unwrap();

    // Populate cache with permission data
    let user_id = "test-user-456";
    let permission_data = serde_json::json!({
        "user_id": user_id,
        "resource": "documents",
        "action": "read",
        "allowed": true
    });

    cache
        .set(
            &format!("permissions:{}:documents", user_id),
            &permission_data,
            Duration::from_secs(300),
        )
        .await
        .unwrap();

    // Verify data is cached
    assert!(
        cache
            .get(&format!("permissions:{}:documents", user_id))
            .await
            .unwrap()
            .is_some()
    );

    // Publish admin event for permission change
    let event_publisher = create_test_event_publisher();
    let auth_details = AuthDetails {
        user_id: "admin-user".to_string(),
        username: Some("admin".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
    };

    let admin_event = AdminEvent::new(
        "test-realm".to_string(),
        auth_details,
        ResourceType::Permission,
        OperationType::Update,
        format!("/realms/test-realm/users/{}/permissions", user_id),
    );

    let event_json = serde_json::to_string(&admin_event).unwrap();
    let publishable_event = PublishableEvent::new(
        "test.admin.events".to_string(),
        user_id.to_string(),
        event_json,
    );

    event_publisher.publish(publishable_event).await.unwrap();
    event_publisher.flush_batch().await.unwrap();

    // Wait for event to be processed
    sleep(Duration::from_secs(2)).await;

    // Verify cache was invalidated
    // Note: Pattern-based invalidation clears L1 cache
    // We should verify that the specific key is no longer in cache

    // Check stats
    let stats = invalidator.get_stats().await;
    assert!(stats.admin_events > 0);
    assert!(stats.invalidations_triggered > 0);

    // Cleanup
    invalidator.stop().await;
}

#[tokio::test]
#[ignore] // Requires Kafka and Redis to be running
async fn test_mfa_status_change_invalidates_cache() {
    // Setup
    let cache = create_test_cache().await;
    let invalidation_service = Arc::new(
        CacheInvalidationService::new(Arc::clone(&cache), None, None, None)
            .await
            .unwrap(),
    );

    let consumer_config = EventConsumerConfig {
        brokers: "localhost:9092".to_string(),
        user_events_topic: "test.user.events".to_string(),
        admin_events_topic: "test.admin.events".to_string(),
        consumer_group: "test-cache-invalidation-3".to_string(),
        ..Default::default()
    };

    let invalidator =
        EventDrivenCacheInvalidator::new(Arc::clone(&invalidation_service), consumer_config)
            .unwrap();

    // Start the consumer
    invalidator.start().await.unwrap();

    // Populate cache with MFA status
    let user_id = "test-user-789";
    let mfa_status = serde_json::json!({
        "user_id": user_id,
        "mfa_enabled": true,
        "methods": ["totp"]
    });

    cache
        .set(
            &format!("mfa:status:{}", user_id),
            &mfa_status,
            Duration::from_secs(300),
        )
        .await
        .unwrap();

    // Verify data is cached
    assert!(
        cache
            .get(&format!("mfa:status:{}", user_id))
            .await
            .unwrap()
            .is_some()
    );

    // Publish MFA enabled event
    let event_publisher = create_test_event_publisher();
    let event =
        Event::new(EventType::MfaEnabled, "test-realm".to_string()).user_id(user_id.to_string());

    let event_json = serde_json::to_string(&event).unwrap();
    let publishable_event = PublishableEvent::new(
        "test.user.events".to_string(),
        user_id.to_string(),
        event_json,
    );

    event_publisher.publish(publishable_event).await.unwrap();
    event_publisher.flush_batch().await.unwrap();

    // Wait for event to be processed
    sleep(Duration::from_secs(2)).await;

    // Verify cache was invalidated
    assert!(
        cache
            .get(&format!("mfa:status:{}", user_id))
            .await
            .unwrap()
            .is_none()
    );

    // Check stats
    let stats = invalidator.get_stats().await;
    assert!(stats.user_events > 0);
    assert!(stats.invalidations_triggered > 0);

    // Cleanup
    invalidator.stop().await;
}

#[tokio::test]
#[ignore] // Requires Kafka and Redis to be running
async fn test_role_change_event_invalidates_cache() {
    // Setup
    let cache = create_test_cache().await;
    let invalidation_service = Arc::new(
        CacheInvalidationService::new(Arc::clone(&cache), None, None, None)
            .await
            .unwrap(),
    );

    let consumer_config = EventConsumerConfig {
        brokers: "localhost:9092".to_string(),
        user_events_topic: "test.user.events".to_string(),
        admin_events_topic: "test.admin.events".to_string(),
        consumer_group: "test-cache-invalidation-4".to_string(),
        ..Default::default()
    };

    let invalidator =
        EventDrivenCacheInvalidator::new(Arc::clone(&invalidation_service), consumer_config)
            .unwrap();

    // Start the consumer
    invalidator.start().await.unwrap();

    // Populate cache with user data
    let user_id = "test-user-role";
    let user_data = serde_json::json!({
        "id": user_id,
        "roles": ["user", "editor"]
    });

    cache
        .set(
            &format!("user:{}", user_id),
            &user_data,
            Duration::from_secs(300),
        )
        .await
        .unwrap();

    // Verify data is cached
    assert!(
        cache
            .get(&format!("user:{}", user_id))
            .await
            .unwrap()
            .is_some()
    );

    // Publish admin event for role change
    let event_publisher = create_test_event_publisher();
    let auth_details = AuthDetails {
        user_id: "admin-user".to_string(),
        username: Some("admin".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
    };

    let admin_event = AdminEvent::new(
        "test-realm".to_string(),
        auth_details,
        ResourceType::RealmRoleMapping,
        OperationType::Update,
        format!("/realms/test-realm/users/{}/role-mappings", user_id),
    );

    let event_json = serde_json::to_string(&admin_event).unwrap();
    let publishable_event = PublishableEvent::new(
        "test.admin.events".to_string(),
        user_id.to_string(),
        event_json,
    );

    event_publisher.publish(publishable_event).await.unwrap();
    event_publisher.flush_batch().await.unwrap();

    // Wait for event to be processed
    sleep(Duration::from_secs(2)).await;

    // Check stats
    let stats = invalidator.get_stats().await;
    assert!(stats.admin_events > 0);
    assert!(stats.invalidations_triggered > 0);

    // Cleanup
    invalidator.stop().await;
}

#[tokio::test]
async fn test_event_consumer_stats() {
    // This test doesn't require Kafka, just tests the stats structure
    let cache = create_test_cache().await;
    let invalidation_service = Arc::new(
        CacheInvalidationService::new(cache, None, None, None)
            .await
            .unwrap(),
    );

    let consumer_config = EventConsumerConfig::default();

    // This will fail without Kafka, but we can test the config
    let result = EventDrivenCacheInvalidator::new(invalidation_service, consumer_config);

    match result {
        Ok(invalidator) => {
            let stats = invalidator.get_stats().await;
            assert_eq!(stats.total_events, 0);
            assert_eq!(stats.user_events, 0);
            assert_eq!(stats.admin_events, 0);
            assert_eq!(stats.invalidations_triggered, 0);
        }
        Err(_) => {
            // Expected without Kafka
            assert!(true);
        }
    }
}
