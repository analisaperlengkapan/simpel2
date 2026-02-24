# Cache Invalidation Guide

This guide explains how to use the cache invalidation mechanism in Authenc for maintaining cache consistency across the system.

## Overview

The cache invalidation system provides:

1. **Kafka-based event-driven invalidation** - Automatically invalidates cache when events occur
2. **Manual invalidation API** - Direct cache invalidation for specific scenarios
3. **Cache warming on startup** - Preload frequently accessed data
4. **Automatic event listener integration** - Seamless integration with the event system

## Architecture

```
┌─────────────────┐
│  User Action    │
│  (Update/Login) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Event System   │
│  (EventBus)     │
└────────┬────────┘
         │
         ├──────────────────────────────┐
         │                              │
         ▼                              ▼
┌─────────────────┐          ┌──────────────────────┐
│ Kafka Producer  │          │ Cache Invalidation   │
│ (KafkaListener) │          │ Listener (Direct)    │
└────────┬────────┘          └──────────┬───────────┘
         │                              │
         ▼                              │
┌─────────────────┐                    │
│ Kafka Topic     │                    │
│ (user_events)   │                    │
└────────┬────────┘                    │
         │                              │
         ▼                              │
┌─────────────────┐                    │
│ Kafka Consumer  │                    │
│ (Invalidation)  │                    │
└────────┬────────┘                    │
         │                              │
         └──────────────┬───────────────┘
                        │
                        ▼
              ┌──────────────────────┐
              │ Cache Invalidation   │
              │ Service              │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │ Multi-Layer Cache    │
              │ (L1: Memory, L2: Redis)│
              └──────────────────────┘
```

## Setup

### 1. Initialize Cache Invalidation Service

```rust
use authenc::services::cache::{
    CacheInvalidationService, MultiLayerCache, RedisCache,
};
use std::sync::Arc;

// Create Redis cache
let redis_cache = Arc::new(RedisCache::new(&redis_config).await?);

// Create multi-layer cache
let multi_cache = Arc::new(MultiLayerCache::with_defaults(redis_cache));

// Create cache invalidation service with Kafka
let invalidation_service = Arc::new(
    CacheInvalidationService::new(
        Arc::clone(&multi_cache),
        Some("localhost:9092"),           // Kafka brokers
        Some("authenc.user.events"),      // Kafka topic
        Some("authenc-cache-invalidation") // Consumer group
    ).await?
);

// Start the invalidation service
invalidation_service.start().await?;
```

### 2. Register Cache Invalidation Listener

```rust
use authenc::events::EventBus;
use authenc::services::cache_invalidation_listener::CacheInvalidationListener;

// Create event bus
let event_bus = EventBus::new();

// Create and register cache invalidation listener
let cache_listener = Arc::new(
    CacheInvalidationListener::new(Arc::clone(&invalidation_service))
);

event_bus.register(cache_listener).await;
```

### 3. Cache Warming on Startup

```rust
use authenc::services::cache::CacheWarmingService;

// Create cache warming service
let warming_service = CacheWarmingService::new(Arc::clone(&multi_cache));

// Get list of active users (e.g., from database)
let active_users = database.get_active_users(1000).await?;

// Warm cache with user data
let user_data_provider = |user_id: String| async move {
    database.get_user(&user_id).await
        .map(|user| user.map(|u| serde_json::to_value(u).ok()).flatten())
};

let warmed_count = warming_service
    .warm_active_users(active_users, user_data_provider)
    .await?;

info!("Warmed cache with {} active users", warmed_count);
```

## Usage

### Automatic Invalidation via Events

When you publish events through the event system, cache invalidation happens automatically:

```rust
use authenc::events::{Event, EventCategory, EventType};

// User update event - automatically invalidates user cache
let event = Event::new(
    realm_id,
    EventType::UserUpdated,
    EventCategory::User
).with_user(user_id, &username);

event_bus.dispatch(event).await;
// Cache for this user is automatically invalidated
```

### Manual Invalidation

For scenarios where you need direct control:

```rust
// Invalidate user cache
invalidation_service.invalidate_user("user123").await?;

// Invalidate permissions cache
invalidation_service.invalidate_permissions("user123").await?;

// Invalidate roles cache
invalidation_service.invalidate_roles("user123").await?;

// Invalidate session cache
invalidation_service.invalidate_session("session456").await?;

// Invalidate MFA status cache
invalidation_service.invalidate_mfa_status("user123").await?;

// Clear all caches
invalidation_service.clear_all().await?;
```

### Monitoring Invalidation Statistics

```rust
// Get invalidation statistics
let stats = invalidation_service.get_stats().await;

println!("Total invalidation events: {}", stats.total_events);
println!("User updates: {}", stats.user_updates);
println!("Permission changes: {}", stats.permission_changes);
println!("Role changes: {}", stats.role_changes);
println!("Session invalidations: {}", stats.session_invalidations);
println!("MFA changes: {}", stats.mfa_changes);
println!("Failures: {}", stats.failures);
```

## Event Types and Cache Invalidation

| Event Type | Cache Keys Invalidated |
|-----------|------------------------|
| `UpdateProfile` | `user:{user_id}`, `permissions:{user_id}:*`, `session:user:{user_id}` |
| `UpdateEmail` | `user:{user_id}`, `permissions:{user_id}:*`, `session:user:{user_id}` |
| `UpdateCredential` | `user:{user_id}`, `permissions:{user_id}:*`, `session:user:{user_id}` |
| `MfaSetup` | `mfa:status:{user_id}`, `user:{user_id}` |
| `MfaDisabled` | `mfa:status:{user_id}`, `user:{user_id}` |
| `MfaReset` | `mfa:status:{user_id}`, `user:{user_id}` |
| `Logout` | `session:{session_id}` |
| `GrantConsent` | `permissions:{user_id}:*`, `user:{user_id}` |
| `RevokeGrant` | `permissions:{user_id}:*`, `user:{user_id}` |

## Cache Warming Strategies

### 1. Warm Active Users on Startup

```rust
// Get users who logged in within the last 24 hours
let active_users = database
    .get_users_by_last_login(Duration::from_secs(86400))
    .await?;

warming_service
    .warm_active_users(active_users, user_data_provider)
    .await?;
```

### 2. Warm Frequently Accessed Permissions

```rust
// Get most frequently accessed permissions
let frequent_permissions = database
    .get_frequent_permission_checks(1000)
    .await?;

let permission_provider = |user_id: String, resource: String| async move {
    database.get_user_permissions(&user_id, &resource).await
        .map(|perms| perms.map(|p| serde_json::to_value(p).ok()).flatten())
};

warming_service
    .warm_permissions(frequent_permissions, permission_provider)
    .await?;
```

### 3. Scheduled Cache Warming

```rust
use tokio::time::{interval, Duration};

// Warm cache every hour
let mut interval = interval(Duration::from_secs(3600));

tokio::spawn(async move {
    loop {
        interval.tick().await;

        if let Err(e) = warm_cache(&warming_service, &database).await {
            error!("Failed to warm cache: {}", e);
        }
    }
});
```

## Configuration

### Environment Variables

```bash
# Kafka configuration for cache invalidation
KAFKA_BROKERS=localhost:9092
KAFKA_USER_EVENTS_TOPIC=authenc.user.events
KAFKA_CACHE_INVALIDATION_GROUP=authenc-cache-invalidation

# Cache warming configuration
CACHE_WARMING_ENABLED=true
CACHE_WARMING_ACTIVE_USER_LIMIT=1000
CACHE_WARMING_ON_STARTUP=true
```

### Configuration File (TOML)

```toml
[cache.invalidation]
enabled = true
kafka_brokers = "localhost:9092"
kafka_topic = "authenc.user.events"
consumer_group = "authenc-cache-invalidation"

[cache.warming]
enabled = true
on_startup = true
active_user_limit = 1000
permission_limit = 5000
```

## Best Practices

### 1. Use Event-Driven Invalidation

Prefer event-driven invalidation over manual invalidation for consistency:

```rust
// Good: Publish event, let listener handle invalidation
event_bus.dispatch(user_updated_event).await;

// Avoid: Manual invalidation after every operation
invalidation_service.invalidate_user(&user_id).await?;
```

### 2. Batch Invalidations

When updating multiple users, batch the operations:

```rust
// Collect all user IDs that need invalidation
let user_ids: Vec<String> = updated_users.iter()
    .map(|u| u.id.to_string())
    .collect();

// Invalidate in batch
for user_id in user_ids {
    invalidation_service.invalidate_user(&user_id).await?;
}
```

### 3. Monitor Invalidation Metrics

Set up monitoring for cache invalidation:

```rust
// Expose metrics via Prometheus
let stats = invalidation_service.get_stats().await;

metrics::gauge!("cache_invalidation_total_events", stats.total_events as f64);
metrics::gauge!("cache_invalidation_failures", stats.failures as f64);
metrics::gauge!("cache_invalidation_hit_ratio",
    1.0 - (stats.failures as f64 / stats.total_events as f64));
```

### 4. Graceful Degradation

Handle Kafka unavailability gracefully:

```rust
// Service works without Kafka (manual invalidation only)
let invalidation_service = CacheInvalidationService::new(
    multi_cache,
    None, // No Kafka
    None,
    None
).await?;

// Manual invalidation still works
invalidation_service.invalidate_user(&user_id).await?;
```

## Troubleshooting

### Cache Not Invalidating

1. Check if invalidation service is running:
```rust
if !invalidation_service.is_running().await {
    warn!("Cache invalidation service is not running!");
    invalidation_service.start().await?;
}
```

2. Verify Kafka connectivity:
```bash
# Test Kafka connection
kafka-console-consumer --bootstrap-server localhost:9092 \
    --topic authenc.user.events --from-beginning
```

3. Check invalidation statistics:
```rust
let stats = invalidation_service.get_stats().await;
if stats.failures > 0 {
    error!("Cache invalidation failures detected: {}", stats.failures);
}
```

### High Invalidation Failure Rate

1. Check Redis connectivity
2. Verify cache key patterns
3. Review error logs for specific failures
4. Consider increasing retry attempts

### Cache Warming Taking Too Long

1. Reduce the number of users to warm
2. Use parallel warming with tokio::spawn
3. Implement progressive warming (warm most critical data first)
4. Consider warming cache in background after startup

## Performance Considerations

### L1 vs L2 Invalidation

- **L1 (In-Memory)**: Instant invalidation, but only affects local instance
- **L2 (Redis)**: Slower but affects all instances

For pattern-based invalidation (e.g., `permissions:user123:*`), the system clears L1 entirely to ensure consistency.

### Kafka Consumer Lag

Monitor consumer lag to ensure timely invalidation:

```bash
kafka-consumer-groups --bootstrap-server localhost:9092 \
    --group authenc-cache-invalidation --describe
```

### Cache Warming Impact

Cache warming on startup can delay application readiness. Consider:

1. Warming cache asynchronously after startup
2. Using health checks that wait for cache warming
3. Implementing progressive warming (critical data first)

## Integration with Existing Code

### Adding Invalidation to New Features

When adding new features that modify cached data:

1. Publish appropriate events:
```rust
// After updating user
let event = Event::new(realm_id, EventType::UpdateProfile, EventCategory::User)
    .with_user(user_id, &username);
event_bus.dispatch(event).await;
```

2. Or use manual invalidation:
```rust
// After updating permissions
invalidation_service.invalidate_permissions(&user_id).await?;
```

### Migration from Old Cache System

If migrating from an old cache system:

1. Keep both systems running in parallel
2. Gradually migrate cache keys to new format
3. Monitor invalidation statistics
4. Remove old system once validated

## References

- [Cache Implementation](../src/services/cache/mod.rs)
- [Cache Invalidation Service](../src/services/cache/invalidation.rs)
- [Cache Invalidation Listener](../src/services/cache_invalidation_listener.rs)
- [Event System](../src/events/mod.rs)
- [Requirements: 15.3, 5.3](../../.kiro/specs/authenc-comprehensive-optimization/requirements.md)
