//! Cache metrics demonstration
//!
//! This example demonstrates how to use the cache metrics collection
//! functionality to monitor Redis cache performance.

use authenc::services::cache::metrics::{CacheMetrics, OperationTimer};
use std::time::Duration;

fn main() {
    println!("=== Cache Metrics Demo ===\n");

    // Create a new metrics collector
    let metrics = CacheMetrics::new();

    println!("1. Initial state:");
    print_metrics(&metrics);

    // Simulate cache operations
    println!("\n2. Simulating cache operations...");

    // Simulate 10 get operations with hits and misses
    for i in 0..10 {
        let timer = OperationTimer::start();
        std::thread::sleep(Duration::from_micros(100 + i * 10));
        let elapsed = timer.elapsed();

        metrics.record_get(elapsed);

        if i % 3 == 0 {
            metrics.record_miss();
        } else {
            metrics.record_hit();
        }
    }

    // Simulate 5 set operations
    for i in 0..5 {
        let timer = OperationTimer::start();
        std::thread::sleep(Duration::from_micros(200 + i * 20));
        let elapsed = timer.elapsed();

        metrics.record_set(elapsed);
        metrics.increment_cache_size();
    }

    // Simulate 2 delete operations
    for i in 0..2 {
        let timer = OperationTimer::start();
        std::thread::sleep(Duration::from_micros(150 + i * 15));
        let elapsed = timer.elapsed();

        metrics.record_delete(elapsed);
        metrics.decrement_cache_size();
    }

    // Simulate 1 error
    metrics.record_error();

    // Simulate 1 eviction
    metrics.record_eviction();

    println!("\n3. After operations:");
    print_metrics(&metrics);

    // Get a snapshot
    println!("\n4. Metrics snapshot:");
    let snapshot = metrics.snapshot();
    println!("   Snapshot: {:#?}", snapshot);

    // Export Prometheus format
    println!("\n5. Prometheus format:");
    println!("{}", snapshot.to_prometheus_format("redis_cache"));

    // Reset metrics
    println!("\n6. After reset:");
    metrics.reset();
    print_metrics(&metrics);
}

fn print_metrics(metrics: &CacheMetrics) {
    println!("   Hits: {}", metrics.hits());
    println!("   Misses: {}", metrics.misses());
    println!("   Hit Ratio: {:.2}%", metrics.hit_ratio() * 100.0);
    println!("   Miss Ratio: {:.2}%", metrics.miss_ratio() * 100.0);
    println!("   Get Operations: {}", metrics.get_operations());
    println!("   Set Operations: {}", metrics.set_operations());
    println!("   Delete Operations: {}", metrics.delete_operations());
    println!("   Errors: {}", metrics.errors());
    println!("   Evictions: {}", metrics.evictions());
    println!("   Cache Size: {}", metrics.cache_size());
    println!(
        "   Avg Get Latency: {:.2} μs",
        metrics.avg_get_latency_micros()
    );
    println!(
        "   Avg Set Latency: {:.2} μs",
        metrics.avg_set_latency_micros()
    );
    println!(
        "   Avg Delete Latency: {:.2} μs",
        metrics.avg_delete_latency_micros()
    );
}
