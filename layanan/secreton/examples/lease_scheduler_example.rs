//! Lease Expiration Scheduler Example
//!
//! This example demonstrates how to use the lease expiration scheduler
//! to automatically revoke expired leases.
//!
//! Run with: cargo run --example lease_scheduler_example

use deadpool_postgres::{Config, Runtime};
use secreton_core::services::lease::{LeaseManager, LeaseSchedulerConfig};
use secreton_core::services::metrics::MetricsRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::NoTls;
use tracing::{Level, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting lease expiration scheduler example");

    // Create database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    info!("Database connection pool created");

    // Create metrics registry
    let metrics_registry = Arc::new(MetricsRegistry::new());

    // Create custom scheduler configuration
    let scheduler_config = LeaseSchedulerConfig {
        check_interval_secs: 10,         // Check every 10 seconds for demo
        notification_threshold_secs: 30, // Notify 30 seconds before expiry
        enable_notifications: true,      // Enable notifications
    };

    // Create lease manager with custom config and metrics
    let manager = Arc::new(
        LeaseManager::with_config(pool, scheduler_config).with_metrics(metrics_registry.clone()),
    );

    info!("Lease manager created with custom configuration");

    // Create some test leases
    info!("Creating test leases...");

    // Lease 1: Expires in 15 seconds
    let lease1 = manager
        .create_lease(
            "user1",
            "/secret/data/test1",
            "kv",
            "default",
            15, // 15 seconds TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await?;
    info!("Created lease 1: {} (expires in 15 seconds)", lease1.id);

    // Lease 2: Expires in 25 seconds
    let lease2 = manager
        .create_lease(
            "user2",
            "/secret/data/test2",
            "kv",
            "default",
            25, // 25 seconds TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await?;
    info!("Created lease 2: {} (expires in 25 seconds)", lease2.id);

    // Lease 3: Expires in 60 seconds (won't expire during demo)
    let lease3 = manager
        .create_lease(
            "user3",
            "/secret/data/test3",
            "kv",
            "default",
            60, // 60 seconds TTL
            86400,
            true,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await?;
    info!("Created lease 3: {} (expires in 60 seconds)", lease3.id);

    // Start the expiration scheduler
    info!("Starting lease expiration scheduler...");
    let scheduler_handle = manager.clone().start_expiration_scheduler();

    // Wait and observe the scheduler in action
    info!("Waiting 40 seconds to observe scheduler behavior...");
    info!("Watch for:");
    info!("  - Expiration notifications (around 15s and 25s)");
    info!("  - Lease revocations (around 15s and 25s)");
    info!("  - Metrics updates");

    for i in 1..=8 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Get current stats
        let stats = manager.get_stats().await?;
        info!(
            "After {}s - Active: {}, Expired: {}, Revoked: {}, Expiring soon: {}",
            i * 5,
            stats.active_count,
            stats.expired_count,
            stats.revoked_count,
            stats.expiring_soon_count
        );

        // Export metrics
        if i % 2 == 0 {
            let metrics = metrics_registry.export_prometheus().await;
            info!("Metrics snapshot:\n{}", metrics);
        }
    }

    // Verify final state
    info!("Verifying final lease states...");

    match manager.lookup_lease(&lease1.id).await {
        Ok(l) => info!("Lease 1 status: {}", l.status),
        Err(e) => info!("Lease 1 lookup failed: {}", e),
    }

    match manager.lookup_lease(&lease2.id).await {
        Ok(l) => info!("Lease 2 status: {}", l.status),
        Err(e) => info!("Lease 2 lookup failed: {}", e),
    }

    match manager.lookup_lease(&lease3.id).await {
        Ok(l) => info!("Lease 3 status: {}", l.status),
        Err(e) => info!("Lease 3 lookup failed: {}", e),
    }

    // Stop the scheduler
    info!("Stopping scheduler...");
    scheduler_handle.abort();

    // Final metrics
    let final_metrics = metrics_registry.export_prometheus().await;
    info!("Final metrics:\n{}", final_metrics);

    info!("Example completed successfully!");

    Ok(())
}
