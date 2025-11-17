//! Example: Event-Driven Cache Invalidation
//!
//! This example demonstrates how to set up and use the event-driven
//! cache invalidation system in Authenc.
//!
//! Run with: cargo run --example event_driven_cache_example

use authenc::config::RedisConfig;
use authenc::models::events::{Event, EventType};
use authenc::services::cache::{
    Cache, CacheInvalidationService, EventConsumerConfig, EventDrivenCacheInvalidator,
    MultiLayerCache, RedisCache,
};
use authenc::services::event_publisher::{EventPublisher, EventPublisherConfig, PublishableEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Event-Driven Cache Invalidation Example ===\n");

    // Step 1: Create Redis cache
    println!("1. Creating Redis cache...");
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/0".to_string(),
        ..Default::default()
    };

    let redis_cache = match RedisCache::new(&redis_config).await {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            eprintln!("Failed to create Redis cache: {}", e);
            eprintln!("Make sure Redis is running on localhost:6379");
            return Ok(());
        }
    };

    // Step 2: Create multi-layer cache
    println!("2. Creating multi-layer cache...");
    let cache = Arc::new(MultiLayerCache::with_defaults(redis_cache));

    // Step 3: Create cache invalidation service
    println!("3. Creating cache invalidation service...");
    let invalidation_service =
        Arc::new(CacheInvalidationService::new(Arc::clone(&cache), None, None, None).await?);

    // Step 4: Configure event consumer
    println!("4. Configuring event consumer...");
    let consumer_config = EventConsumerConfig {
        brokers: "localhost:9092".to_string(),
        user_events_topic: "authenc.user.events".to_string(),
        admin_events_topic: "authenc.admin.events".to_string(),
        consumer_group: "authenc-cache-invalidation-example".to_string(),
        enable_auto_commit: true,
        auto_offset_reset: "latest".to_string(),
    };

    // Step 5: Create event-driven cache invalidator
    println!("5. Creating event-driven cache invalidator...");
    let invalidator = match EventDrivenCacheInvalidator::new(
        Arc::clone(&invalidation_service),
        consumer_config,
    ) {
        Ok(inv) => Arc::new(inv),
        Err(e) => {
            eprintln!("Failed to create event-driven cache invalidator: {}", e);
            eprintln!("Make sure Kafka is running on localhost:9092");
            return Ok(());
        }
    };

    // Step 6: Start the invalidator
    println!("6. Starting event-driven cache invalidator...");
    invalidator.start().await?;
    println!("   ✓ Invalidator started and listening for events\n");

    // Step 7: Demonstrate cache invalidation
    println!("7. Demonstrating cache invalidation...\n");

    // Populate cache with user data
    let user_id = "demo-user-123";
    let user_data = serde_json::json!({
        "id": user_id,
        "name": "Demo User",
        "email": "demo@example.com",
        "roles": ["user", "editor"]
    });

    println!("   a) Populating cache with user data...");
    cache
        .set(
            &format!("user:{}", user_id),
            &user_data,
            Duration::from_secs(300),
        )
        .await?;

    // Verify data is cached
    let cached_data = cache.get(&format!("user:{}", user_id)).await?;
    println!("   b) Verifying data is cached: {}", cached_data.is_some());

    // Create event publisher
    println!("   c) Creating event publisher...");
    let publisher_config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        max_retries: 3,
        batch_size: 10,
        flush_interval_ms: 100,
        ..Default::default()
    };

    let event_publisher = match EventPublisher::new(publisher_config) {
        Ok(pub_) => pub_,
        Err(e) => {
            eprintln!("Failed to create event publisher: {}", e);
            invalidator.stop().await;
            return Ok(());
        }
    };

    // Publish user update event
    println!("   d) Publishing user update event...");
    let event = Event::new(EventType::UpdateProfile, "demo-realm".to_string())
        .user_id(user_id.to_string())
        .detail("field".to_string(), "email".to_string());

    let event_json = serde_json::to_string(&event)?;
    let publishable_event = PublishableEvent::new(
        "authenc.user.events".to_string(),
        user_id.to_string(),
        event_json,
    );

    event_publisher.publish(publishable_event).await?;
    event_publisher.flush_batch().await?;
    println!("   e) Event published successfully");

    // Wait for event to be processed
    println!("   f) Waiting for event to be processed (2 seconds)...");
    sleep(Duration::from_secs(2)).await;

    // Verify cache was invalidated
    let cached_data_after = cache.get(&format!("user:{}", user_id)).await?;
    println!(
        "   g) Verifying cache was invalidated: {}",
        cached_data_after.is_none()
    );

    // Step 8: Display statistics
    println!("\n8. Event consumer statistics:");
    let stats = invalidator.get_stats().await;
    println!("   - Total events processed: {}", stats.total_events);
    println!("   - User events: {}", stats.user_events);
    println!("   - Admin events: {}", stats.admin_events);
    println!(
        "   - Invalidations triggered: {}",
        stats.invalidations_triggered
    );
    println!("   - Processing failures: {}", stats.processing_failures);
    println!(
        "   - Deserialization errors: {}",
        stats.deserialization_errors
    );

    // Step 9: Demonstrate MFA cache invalidation
    println!("\n9. Demonstrating MFA cache invalidation...\n");

    let mfa_user_id = "demo-mfa-user";
    let mfa_status = serde_json::json!({
        "user_id": mfa_user_id,
        "mfa_enabled": true,
        "methods": ["totp"]
    });

    println!("   a) Populating MFA status cache...");
    cache
        .set(
            &format!("mfa:status:{}", mfa_user_id),
            &mfa_status,
            Duration::from_secs(300),
        )
        .await?;

    // Publish MFA enabled event
    println!("   b) Publishing MFA enabled event...");
    let mfa_event = Event::new(EventType::MfaEnabled, "demo-realm".to_string())
        .user_id(mfa_user_id.to_string());

    let mfa_event_json = serde_json::to_string(&mfa_event)?;
    let mfa_publishable_event = PublishableEvent::new(
        "authenc.user.events".to_string(),
        mfa_user_id.to_string(),
        mfa_event_json,
    );

    event_publisher.publish(mfa_publishable_event).await?;
    event_publisher.flush_batch().await?;

    // Wait for processing
    println!("   c) Waiting for event to be processed...");
    sleep(Duration::from_secs(2)).await;

    // Verify MFA cache was invalidated
    let mfa_cached_after = cache.get(&format!("mfa:status:{}", mfa_user_id)).await?;
    println!(
        "   d) Verifying MFA cache was invalidated: {}",
        mfa_cached_after.is_none()
    );

    // Step 10: Final statistics
    println!("\n10. Final statistics:");
    let final_stats = invalidator.get_stats().await;
    println!("   - Total events processed: {}", final_stats.total_events);
    println!(
        "   - Invalidations triggered: {}",
        final_stats.invalidations_triggered
    );

    // Step 11: Graceful shutdown
    println!("\n11. Shutting down...");
    event_publisher.shutdown().await?;
    invalidator.stop().await;
    println!("   ✓ Shutdown complete\n");

    println!("=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("1. Event-driven cache invalidation runs in the background");
    println!("2. Cache is automatically invalidated when events are published");
    println!("3. Supports both user events and admin events");
    println!("4. Comprehensive statistics for monitoring");
    println!("5. Graceful shutdown ensures no data loss");

    Ok(())
}
