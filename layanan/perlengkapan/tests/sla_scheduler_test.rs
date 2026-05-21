// Integration test for SLA escalation scheduler
use layanan_perlengkapan::workflow::{SlaEscalationScheduler, SlaSchedulerConfig};
use std::time::Duration;
use tokio_postgres::NoTls;

#[tokio::test]
async fn test_sla_scheduler_creation() {
    // Create a test database pool
    let config = tokio_postgres::Config::new();
    let manager = deadpool_postgres::Manager::new(config, NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(1)
        .build()
        .unwrap();

    // Create scheduler with disabled config
    let config = SlaSchedulerConfig {
        check_interval_cron: "0 */15 * * * *".to_string(),
        enabled: false,
    };

    let scheduler = SlaEscalationScheduler::with_config(config, pool);

    // Start should succeed even with disabled scheduler
    assert!(scheduler.start().is_ok());
}

#[tokio::test]
async fn test_sla_scheduler_config_default() {
    let config = SlaSchedulerConfig::default();
    assert_eq!(config.check_interval_cron, "0 */15 * * * *");
    assert!(config.enabled);
}

#[tokio::test]
async fn test_sla_scheduler_config_from_env() {
    // Set environment variables
    unsafe {
        std::env::set_var("SLA_CHECK_INTERVAL_CRON", "0 */30 * * * *");
        std::env::set_var("SLA_SCHEDULER_ENABLED", "false");
    }

    let config = SlaSchedulerConfig::from_env();
    assert_eq!(config.check_interval_cron, "0 */30 * * * *");
    assert!(!config.enabled);

    // Cleanup
    unsafe {
        std::env::remove_var("SLA_CHECK_INTERVAL_CRON");
        std::env::remove_var("SLA_SCHEDULER_ENABLED");
    }
}

#[tokio::test]
async fn test_sla_scheduler_disabled() {
    // Create a test database pool
    let config = tokio_postgres::Config::new();
    let manager = deadpool_postgres::Manager::new(config, NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(1)
        .build()
        .unwrap();

    // Create scheduler with disabled config
    let config = SlaSchedulerConfig {
        check_interval_cron: "0 */15 * * * *".to_string(),
        enabled: false,
    };

    let scheduler = SlaEscalationScheduler::with_config(config, pool);

    // Start should succeed and return immediately
    let result = scheduler.start();
    assert!(result.is_ok());

    // Give it a moment to ensure it doesn't start
    tokio::time::sleep(Duration::from_millis(100)).await;
}
