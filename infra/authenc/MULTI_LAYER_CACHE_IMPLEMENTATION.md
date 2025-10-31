# Multi-Layer Cache Implementation

## Overview

This document describes the implementation of the multi-layer caching strategy for Authenc, which provides a two-tier caching system with L1 (in-memory) and L2 (Redis) caches.

## Architecture

### Cache Hierarchy

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│                                         │
│    ┌─────────────────────────────┐     │
│    │   MultiLayerCache Wrapper   │     │
│    └─────────────────────────────┘     │
│              │         │                │
│              ▼         ▼                │
│    ┌──────────┐   ┌──────────┐         │
│    │ L1 Cache │   │ L2 Cache │         │
│    │(In-Mem)  │   │ (Redis)  │         │
│    └──────────┘   └──────────┘         │
│         │              │                │
└─────────┼──────────────┼────────────────┘
          │              │
          ▼              ▼
    DashMap (10k)   Redis Server
    TTL: 60s        TTL: Configurable
```

### Components

#### 1. InMemoryCache (L1)

**File**: `src/services/cache/in_memory_cache.rs`

**Features**:
- Fast in-memory storage using `DashMap` for concurrent access
- TTL-based expiration (default: 60 seconds)
- LRU eviction when cache is full (max: 10,000 entries)
- Automatic cleanup of expired entries
- Thread-safe with minimal locking overhead

**Key Characteristics**:
- **Capacity**: 10,000 entries (configurable)
- **TTL**: 60 seconds (configurable)
- **Eviction**: LRU (Least Recently Used)
- **Latency**: < 1ms for get/set operations
- **Concurrency**: Lock-free reads, minimal locking for writes

**Implementation Details**:
```rust
pub struct InMemoryCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    max_size: usize,
    default_ttl: Duration,
    metrics: Arc<CacheMetrics>,
}

struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
    last_accessed: Instant,  // For LRU tracking
}
```

#### 2. RedisCache (L2)

**File**: `src/services/cache/redis_cache.rs`

**Features**:
- Distributed caching with Redis
- Connection pooling with `ConnectionManager`
- Automatic reconnection on failure
- Batch operations support
- Health check monitoring

**Key Characteristics**:
- **Capacity**: Unlimited (managed by Redis)
- **TTL**: Configurable per key type
- **Eviction**: Redis allkeys-lru policy
- **Latency**: 1-5ms for local Redis, 10-50ms for remote
- **Persistence**: Optional (Redis configuration)

#### 3. MultiLayerCache (Wrapper)

**File**: `src/services/cache/multi_layer_cache.rs`

**Features**:
- Transparent two-tier caching
- Cache-aside pattern implementation
- Automatic L1 population from L2
- Graceful fallback on cache failure
ned metrics tracking

**Cache-Aside Pattern**:

**GET Operation**:
```
1. Check L1 (in-memory)
   ├─ HIT → Return value (fast path)
   └─ MISS → Check L2 (Redis)
       ├─ HIT → Populate L1, return value
       └─ MISS → Return None
```

**SET Operation**:
```
1. Write to L2 (Redis) - source of truth
2. Write to L1 (in-memory) with shorter TTL
```

**DELETE Operation**:
```
1. Delete from L1
2. Delete from L2
```

## Configuration

### MultiLayerCacheConfig

```rust
pub struct MultiLayerCacheConfig {
    /// L1 cache maximum size (number of entries)
    pub l1_max_size: usize,        // Default: 10,000

    /// L1 cache default TTL
    pub l1_ttl: Duration,          // Default: 60 seconds

    /// Whether to enable L1 cache
    pub l1_enabled: bool,          // Default: true

    /// Whether to enable L2 cache
    pub l2_enabled: bool,          // Default: true
}
```

### Usage Example

```rust
use authenc::services::cache::{
    Cache, MultiLayerCache, MultiLayerCacheConfig, RedisCache
};
use std::sync::Arc;
use std::time::Duration;

// Create Redis cache (L2)
let redis_config = RedisConfig {
    enabled: true,
    url: "redis://localhost:6379/0".to_string(),
    default_ttl: 3600,
    mfa_cache_ttl: 300,
    otp_verification_ttl: 90,
};

let redis_cache = Arc::new(RedisCache::new(&redis_config).await?);

// Create multi-layer cache with default config
let cache = MultiLayerCache::with_defaults(redis_cache);

// Or with custom config
let config = MultiLayerCacheConfig {
    l1_max_size: 5_000,
    l1_ttl: Duration::from_secs(30),
    l1_enabled: true,
    l2_enabled: true,
};
let cache = MultiLayerCache::new(redis_cache, config);

// Use the cache
let key = "user:123";
let value = serde_json::json!({"id": 123, "name": "John"});

// Set
cache.set(key, &value, Duration::from_secs(300)).await?;

// Get
if let Some(data) = cache.get(key).await? {
    println!("Found: {}", data);
}

// Delete
cache.delete(key).await?;
```

## Performance Characteristics

### Latency Targets

| Operation | L1 (In-Memory) | L2 (Redis Local) | L2 (Redis Remote) |
|-----------|----------------|------------------|-------------------|
| GET       | < 1ms          | 1-5ms            | 10-50ms           |
| SET       | < 1ms          | 1-5ms            | 10-50ms           |
| DELETE    | < 1ms          | 1-5ms            | 10-50ms           |

### Cache Hit Ratios

**Expected Performance**:
- L1 Hit Ratio: 60-80% (hot data)
- L2 Hit Ratio: 15-30% (warm data)
- Combined Hit Ratio: > 80%

**Factors Affecting Hit Ratio**:
- L1 cache size (larger = higher hit ratio)
- L1 TTL (longer = higher hit ratio, but stale data risk)
- Access patterns (temporal locality)
- Data distribution (key popularity)

### Memory Usage

**L1 Cache**:
- Entry overhead: ~100 bytes per entry
- Value size: Variable (JSON serialized)
- Total: ~1-10 MB for 10,000 entries (depends on value size)

**L2 Cache**:
- Managed by Redis
- Typical: 100 MB - 1 GB
- Configurable via Redis maxmemory

## Metrics and Monitoring

### Available Metrics

```rust
// L1 metrics
let l1_metrics = cache.l1_metrics();
println!("L1 Hits: {}", l1_metrics.hits());
println!("L1 Misses: {}", l1_metrics.misses());
println!("L1 Hit Ratio: {:.2}%", cache.l1_hit_ratio() * 100.0);

// L2 metrics
let l2_metrics = cache.l2_metrics();
println!("L2 Hits: {}", l2_metrics.hits());
println!("L2 Misses: {}", l2_metrics.misses());
println!("L2 Hit Ratio: {:.2}%", cache.l2_hit_ratio() * 100.0);

// Combined metrics
println!("Overall Hit Ratio: {:.2}%", cache.hit_ratio() * 100.0);
```

### Prometheus Metrics

The cache exposes Prometheus-compatible metrics:

```
# L1 Cache
authenc_l1_cache_hits_total
authenc_l1_cache_misses_total
authenc_l1_cache_hit_ratio
authenc_l1_cache_size
authenc_l1_cache_evictions_total

# L2 Cache
authenc_l2_cache_hits_total
authenc_l2_cache_misses_total
authenc_l2_cache_hit_ratio
authenc_l2_cache_operations_total{operation="get|set|delete"}
authenc_l2_cache_latency_milliseconds{operation="get|set|delete"}
```

## Cache Key Strategy

### Key Prefixes

```rust
// Session cache
session:{session_id}                    // TTL: 1h

// User profile cache
user:{user_id}                          // TTL: 5m

// Permission cache
permissions:{user_id}:{resource}        // TTL: 5m

// MFA secret cache
mfa:{user_id}:secret                    // TTL: 5m

// Rate limiting
rate_limit:{ip}:{endpoint}              // TTL: 1m

// Token validation
token:{token_hash}                      // TTL: 5m
```

### TTL Strategy

| Data Type | L1 TTL | L2 TTL | Rationale |
|-----------|--------|--------|-----------|
| Session | 60s | 1h | Frequently accessed, short L1 for freshness |
| User Profile | 60s | 5m | Moderate access, balance freshness vs load |
| Permissions | 60s | 5m | Security-critical, short TTL for updates |
| MFA Secret | 60s | 5m | Security-critical, short TTL |
| Rate Limit | 60s | 1m | Very short-lived, high write frequency |
| Token Validation | 60s | 5m | High read frequency, moderate freshness |

## Error Handling

### Graceful Degradation

The multi-layer cache implements graceful degradation:

1. **L1 Failure**: Falls back to L2 (Redis)
2. **L2 Failure**: Continues with L1 only (degraded mode)
3. **Both Failures**: Returns cache miss, application continues

```rust
// Example: L2 failure handling
if self.config.l2_enabled {
    match self.l2.get(key).await {
        Ok(Some(value)) => {
            // Populate L1 on L2 hit
            if let Err(e) = self.l1.set(key, &value, self.config.l1_ttl).await {
                warn!("Failed to populate L1 cache from L2: {}", e);
            }
            return Ok(Some(value));
        }
        Err(e) => {
            warn!("L2 cache error for key {}: {}", key, e);
            self.metrics.record_error();
            // Continue to return None (cache miss)
        }
        _ => {}
    }
}
```

## Testing

### Unit Tests

Run unit tests:
```bash
cargo test --lib services::cache::in_memory_cache
cargo test --lib services::cache::multi_layer_cache
```

### Integration Tests

Integration tests require a running Redis instance:
```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run integration tests
cargo test --lib services::cache::multi_layer_cache -- --ignored
```

### Example Demo

Run the example demo:
```bash
cargo run --example multi_layer_cache_demo --features=default
```

## Best Practices

### 1. Cache Key Design

- Use consistent prefixes for different data types
- Include version numbers for schema changes
- Keep keys short but descriptive
- Use hierarchical structure (e.g., `user:123:profile`)

### 2. TTL Selection

- Shorter TTL for security-critical data
- Longer TTL for rarely-changing data
- Consider L1 TTL < L2 TTL for freshness
- Monitor cache hit ratio to tune TTL

### 3. Cache Invalidation

- Invalidate on data updates (write-through)
- Use event-driven invalidation (Kafka events)
- Implement cache warming for critical data
- Consider cache versioning for schema changes

### 4. Monitoring

- Track cache hit ratios (target > 80%)
- Monitor cache size and eviction rate
- Alert on low hit ratios or high error rates
- Use distributed tracing for cache operations

### 5. Capacity Planning

- L1: Size based on hot data set (typically 10k-100k entries)
- L2: Size based on working set (typically 100 MB - 1 GB)
- Monitor memory usage and adjust limits
- Consider cache warming on startup

## Integration with Authenc

### AppState Integration

```rust
pub struct AppState {
    pub cache: Arc<MultiLayerCache>,
    // ... other fields
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        // Create Redis cache
        let redis_cache = Arc::new(RedisCache::new(&config.redis).await?);

        // Create multi-layer cache
        let cache_config = MultiLayerCacheConfig {
            l1_max_size: config.cache.l1_max_size,
            l1_ttl: Duration::from_secs(config.cache.l1_ttl),
            l1_enabled: config.cache.l1_enabled,
            l2_enabled: config.cache.l2_enabled,
        };
        let cache = Arc::new(MultiLayerCache::new(redis_cache, cache_config));

        Ok(Self {
            cache,
            // ... other fields
        })
    }
}
```

### Usage in Handlers

```rust
async fn validate_token(
    State(state): State<Arc<AppState>>,
    token: String,
) -> Result<Json<TokenValidationResponse>> {
    let cache_key = format!("token:{}", hash(&token));

    // Try cache first
    if let Some(cached) = state.cache.get(&cache_key).await? {
        return Ok(Json(serde_json::from_value(cached)?));
    }

    // Cache miss - validate token
    let result = validate_jwt(&token)?;

    // Cache the result
    let cache_value = serde_json::to_value(&result)?;
    state.cache.set(&cache_key, &cache_value, Duration::from_secs(300)).await?;

    Ok(Json(result))
}
```

## Requirements Fulfilled

This implementation fulfills the following requirements from the spec:

### Requirement 15.1: Multi-Layer Caching
✅ L1 in-memory cache with DashMap (TTL: 60s, size: 10k entries)
✅ L2 Redis cache integration with fallback
✅ Cache-aside pattern with automatic L1 population from L2
✅ LRU eviction for L1 cache

### Requirement 15.2: Cache Performance
✅ Fast L1 cache (< 1ms latency)
✅ Graceful fallback on cache failures
✅ Connection pooling for Redis
✅ Metrics collection for monitoring

## Future Enhancements

1. **Cache Warming**: Pre-populate cache on startup with frequently accessed data
2. **Cache Invalidation**: Event-driven invalidation via Kafka
3. **Cache Compression**: Compress large values in L2
4. **Cache Partitioning**: Shard cache by key prefix for better scalability
5. **Cache Replication**: Multi-region Redis replication for HA
6. **Adaptive TTL**: Dynamically adjust TTL based on access patterns
7. **Cache Analytics**: Detailed analytics on cache usage patterns

## Conclusion

The multi-layer cache implementation provides a robust, high-performance caching solution for Authenc with:

- **Fast access**: < 1ms for L1, 1-5ms for L2
- **High availability**: Graceful degradation on failures
- **Scalability**: Horizontal scaling with Redis
- **Observability**: Comprehensive metrics and monitoring
- **Flexibility**: Configurable TTL, size, and behavior

This implementation significantly improves Authenc's performance by reducing database load and improving response times for frequently accessed data.
