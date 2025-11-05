//! Multi-layer cache demonstration
//!
//! This example demonstrates the usage of the MultiLayerCache with L1 (in-memory)
//! and L2 (Redis) caching layers.
//!
//! Run with: cargo run --example multi_layer_cache_demo --features=default

use authenc::config::RedisConfig;
use authenc::services::cache::{Cache, MultiLayerCache, MultiLayerCacheConfig, RedisCache};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Multi-Layer Cache Demo ===\n");

    // Create Redis cache (L2)
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/0".to_string(),
        default_ttl: 3600,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 90,
    };

    println!("Connecting to Redis...");
    let redis_cache = match RedisCache::new(&redis_config).await {
        Ok(cache) => {
            println!("✓ Connected to Redis successfully\n");
            Arc::new(cache)
        }
        Err(e) => {
            eprintln!("✗ Failed to connect to Redis: {}", e);
            eprintln!("  Make sure Redis is running on localhost:6379");
            return Err(e.into());
        }
    };

    // Create multi-layer cache with custom configuration
    let config = MultiLayerCacheConfig {
        l1_max_size: 10_000,
        l1_ttl: Duration::from_secs(60),
        l1_enabled: true,
        l2_enabled: true,
    };

    let cache = MultiLayerCache::new(redis_cache, config);

    println!("Multi-layer cache created:");
    println!("  L1: In-memory (max 10,000 entries, TTL 60s)");
    println!("  L2: Redis (distributed cache)\n");

    // Example 1: Basic set and get
    println!("--- Example 1: Basic Operations ---");
    let key = "user:123";
    let value = serde_json::json!({
        "id": 123,
        "username": "john_doe",
        "email": "john@example.com"
    });

    println!("Setting key '{}' with value: {}", key, value);
    cache.set(key, &value, Duration::from_secs(300)).await?;

    println!("Getting key '{}'...", key);
    if let Some(retrieved) = cache.get(key).await? {
        println!("✓ Retrieved from cache: {}", retrieved);
    }

    println!("L1 cache size: {}", cache.l1_size());
    println!();

    // Example 2: Cache-aside pattern (L1 miss, L2 hit)
    println!("--- Example 2: Cache-Aside Pattern ---");
    println!("Clearing L1 cache to demonstrate L2 fallback...");
    cache.clear_l1();
    println!("L1 cache size after clear: {}", cache.l1_size());

    println!(
        "Getting key '{}' again (should hit L2 and populate L1)...",
        key
    );
    if let Some(retrieved) = cache.get(key).await? {
        println!("✓ Retrieved from L2: {}", retrieved);
    }

    println!("L1 cache size after L2 hit: {}", cache.l1_size());
    println!();

    // Example 3: Multiple keys
    println!("--- Example 3: Multiple Keys ---");
    for i in 1..=5 {
        let key = format!("session:{}", i);
        let value = serde_json::json!({
            "session_id": i,
            "user_id": i * 100,
            "expires_at": chrono::Utc::now().timestamp() + 3600
        });

        cache.set(&key, &value, Duration::from_secs(300)).await?;
        println!("Set {}", key);
    }

    println!("\nL1 cache size: {}", cache.l1_size());
    println!();

    // Example 4: Cache metrics
    println!("--- Example 4: Cache Metrics ---");

    // Generate some cache activity
    for i in 1..=10 {
        let key = format!("session:{}", i);
        let _ = cache.get(&key).await;
    }

    let l1_metrics = cache.l1_metrics();
    let l2_metrics = cache.l2_metrics();

    println!("L1 Cache Metrics:");
    println!("  Hits: {}", l1_metrics.hits());
    println!("  Misses: {}", l1_metrics.misses());
    println!("  Hit Ratio: {:.2}%", cache.l1_hit_ratio() * 100.0);
    println!("  Cache Size: {}", cache.l1_size());

    println!("\nL2 Cache Metrics:");
    println!("  Hits: {}", l2_metrics.hits());
    println!("  Misses: {}", l2_metrics.misses());
    println!("  Hit Ratio: {:.2}%", cache.l2_hit_ratio() * 100.0);

    println!("\nCombined Metrics:");
    println!("  Overall Hit Ratio: {:.2}%", cache.hit_ratio() * 100.0);
    println!();

    // Example 5: Atomic operations
    println!("--- Example 5: Atomic Operations ---");
    let counter_key = "counter:requests";

    println!("Incrementing counter...");
    for i in 1..=5 {
        let value = cache.increment(counter_key, 1).await?;
        println!("  Increment {}: counter = {}", i, value);
    }

    println!();

    // Example 6: Set if not exists (set_nx)
    println!("--- Example 6: Set If Not Exists (set_nx) ---");
    let lock_key = "lock:resource:123";
    let lock_value = serde_json::json!({
        "locked_by": "worker-1",
        "locked_at": chrono::Utc::now().to_rfc3339()
    });

    println!("Attempting to acquire lock...");
    if cache
        .set_nx(lock_key, &lock_value, Duration::from_secs(30))
        .await?
    {
        println!("✓ Lock acquired successfully");

        println!("Attempting to acquire lock again...");
        if !cache
            .set_nx(lock_key, &lock_value, Duration::from_secs(30))
            .await?
        {
            println!("✗ Lock already held (as expected)");
        }
    }

    println!();

    // Example 7: Cleanup
    println!("--- Example 7: Cleanup ---");
    println!("Deleting test keys...");
    cache.delete("user:123").await?;
    cache.delete(counter_key).await?;
    cache.delete(lock_key).await?;

    for i in 1..=5 {
        let key = format!("session:{}", i);
        cache.delete(&key).await?;
    }

    println!("✓ Cleanup complete");
    println!("\nFinal L1 cache size: {}", cache.l1_size());

    println!("\n=== Demo Complete ===");

    Ok(())
}
