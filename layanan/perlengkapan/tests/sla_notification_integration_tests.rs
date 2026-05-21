// ============================================================================
// SLA Monitoring with Notification Integration Tests
// Description: Tests for SLA breach detection and notification delivery
// Requirements: REQ-W003, REQ-N008, NFR-M004
// ============================================================================

use chrono::{Duration, Utc};
use deadpool_postgres::{Config, Pool, Runtime};
use std::env;
use tokio_postgres::NoTls;
use uuid::Uuid;

// Import workflow modules
use layanan_perlengkapan::workflow::{
    config::WorkflowConfig, notifikasi_client::NotifikasiClient, sla::SlaMonitor,
};

/// Helper function to create a test database pool
async fn create_test_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some(env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()));
    cfg.port = Some(
        env::var("DATABASE_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .unwrap(),
    );
    cfg.dbname =
        Some(env::var("DATABASE_NAME").unwrap_or_else(|_| "perlengkapan_test".to_string()));
    cfg.user = Some(env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string()));
    cfg.password = Some(env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string()));

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create test database pool")
}

/// Helper function to create a test kebutuhan BMN entity
async fn create_test_kebutuhan(
    pool: &Pool,
    status: &str,
    created_at: chrono::DateTime<Utc>,
) -> Uuid {
    let client = pool.get().await.expect("Failed to get database client");

    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Insert kebutuhan BMN
    let query = r#"
        INSERT INTO perlengkapan.kebutuhan_bmn
        (id, satker_id, kode_barang, nama_barang, jumlah_kebutuhan, tahun_anggaran, status, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
    "#;

    client
        .execute(
            query,
            &[
                &entity_id,
                &Uuid::new_v4(), // satker_id
                &"1.01.01.01.001",
                &"Test Barang",
                &10i32,
                &2026i32,
                &status,
                &user_id,
                &created_at,
            ],
        )
        .await
        .expect("Failed to insert test kebutuhan");

    // Insert workflow activity
    let activity_query = r#"
        INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
        (pengajuan_id, aktivitas_id, user_id, catatan, created_at)
        VALUES (
            $1,
            (SELECT id FROM perlengkapan.ms_aktivitas_bmn WHERE kode = $2 LIMIT 1),
            $3,
            'Test activity',
            $4
        )
    "#;

    client
        .execute(
            activity_query,
            &[&entity_id, &status, &user_id, &created_at],
        )
        .await
        .expect("Failed to insert test activity");

    entity_id
}

/// Helper function to cleanup test data
async fn cleanup_test_data(pool: &Pool, entity_id: Uuid) {
    let client = pool.get().await.expect("Failed to get database client");

    // Delete workflow activities
    client
        .execute(
            "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas WHERE pengajuan_id = $1",
            &[&entity_id],
        )
        .await
        .ok();

    // Delete kebutuhan
    client
        .execute(
            "DELETE FROM perlengkapan.kebutuhan_bmn WHERE id = $1",
            &[&entity_id],
        )
        .await
        .ok();
}

#[tokio::test]
#[ignore] // Requires database and notification service
async fn test_sla_breach_detection() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create entity in SUBMITTED state, 3 days ago (SLA is 2 days)
    let created_at = Utc::now() - Duration::days(3);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Test: Check SLA for entity
    let result: Result<Option<layanan_perlengkapan::workflow::sla::SlaBreachInfo>, _> =
        monitor.check_sla(entity_id).await;
    assert!(result.is_ok());

    let breach = result.unwrap();
    assert!(breach.is_some(), "SLA breach should be detected");

    let breach_info = breach.unwrap();
    assert_eq!(breach_info.entity_id, entity_id);
    assert_eq!(breach_info.current_state, "SUBMITTED");
    assert!(
        breach_info.breach_duration_minutes > 0,
        "Breach duration should be positive"
    );

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}

#[tokio::test]
#[ignore] // Requires database and notification service
async fn test_sla_no_breach() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create entity in SUBMITTED state, 1 day ago (SLA is 2 days)
    let created_at = Utc::now() - Duration::days(1);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Test: Check SLA for entity
    let result: Result<Option<layanan_perlengkapan::workflow::sla::SlaBreachInfo>, _> =
        monitor.check_sla(entity_id).await;
    assert!(result.is_ok());

    let breach = result.unwrap();
    assert!(breach.is_none(), "SLA breach should not be detected");

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}

#[tokio::test]
#[ignore] // Requires database and notification service
async fn test_sla_check_all() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create multiple entities with different SLA statuses
    let entity1 = create_test_kebutuhan(&pool, "SUBMITTED", Utc::now() - Duration::days(3)).await;
    let entity2 = create_test_kebutuhan(&pool, "SUBMITTED", Utc::now() - Duration::days(1)).await;
    let entity3 = create_test_kebutuhan(&pool, "REVIEWED", Utc::now() - Duration::days(2)).await;

    // Test: Check all SLAs
    let result: Result<Vec<layanan_perlengkapan::workflow::sla::SlaBreachInfo>, _> =
        monitor.check_all_sla().await;
    assert!(result.is_ok());

    let breaches = result.unwrap();
    assert!(
        breaches.len() >= 2,
        "At least 2 SLA breaches should be detected"
    );

    // Verify entity1 is in breaches
    assert!(
        breaches.iter().any(|b| b.entity_id == entity1),
        "Entity1 should have SLA breach"
    );

    // Verify entity2 is not in breaches
    assert!(
        !breaches.iter().any(|b| b.entity_id == entity2),
        "Entity2 should not have SLA breach"
    );

    // Cleanup
    cleanup_test_data(&pool, entity1).await;
    cleanup_test_data(&pool, entity2).await;
    cleanup_test_data(&pool, entity3).await;
}

#[tokio::test]
#[ignore] // Requires database and notification service running
async fn test_sla_escalation_with_notification() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();

    // Create notification client
    let notifikasi_endpoint =
        env::var("NOTIFIKASI_GRPC_URL").unwrap_or_else(|_| "http://localhost:50053".to_string());

    let notifikasi_client = NotifikasiClient::new(&notifikasi_endpoint)
        .await
        .expect("Failed to create notification client");

    let monitor = SlaMonitor::with_notifikasi(config, pool.clone(), notifikasi_client);

    // Create entity with SLA breach
    let created_at = Utc::now() - Duration::days(3);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Test: Check SLA and escalate
    let breach_result: Result<Option<layanan_perlengkapan::workflow::sla::SlaBreachInfo>, _> =
        monitor.check_sla(entity_id).await;
    assert!(breach_result.is_ok());

    let breach = breach_result.unwrap();
    assert!(breach.is_some(), "SLA breach should be detected");

    let breach_info = breach.unwrap();

    // Test: Escalate SLA breach
    let escalate_result: Result<(), _> = monitor.escalate_sla_breach(&breach_info).await;
    assert!(escalate_result.is_ok(), "Escalation should succeed");

    // Verify escalation was logged in workflow activity
    let client = pool.get().await.expect("Failed to get database client");
    let query = r#"
        SELECT COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
        WHERE pengajuan_id = $1
        AND catatan LIKE '%SLA breach detected%'
    "#;

    let row = client
        .query_one(query, &[&entity_id])
        .await
        .expect("Failed to query");
    let count: i64 = row.get("count");
    assert!(
        count > 0,
        "SLA breach should be logged in workflow activity"
    );

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_sla_monitor_and_escalate() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create multiple entities with SLA breaches
    let entity1 = create_test_kebutuhan(&pool, "SUBMITTED", Utc::now() - Duration::days(3)).await;
    let entity2 = create_test_kebutuhan(&pool, "REVIEWED", Utc::now() - Duration::days(2)).await;

    // Test: Monitor and escalate all breaches
    let result: Result<usize, _> = monitor.monitor_and_escalate().await;
    assert!(result.is_ok());

    let breach_count = result.unwrap();
    assert!(breach_count >= 2, "At least 2 breaches should be escalated");

    // Cleanup
    cleanup_test_data(&pool, entity1).await;
    cleanup_test_data(&pool, entity2).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_sla_status_normal() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create entity in SUBMITTED state, 1 hour ago (SLA is 2 days)
    let created_at = Utc::now() - Duration::hours(1);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Test: Get SLA status
    let result: Result<layanan_perlengkapan::workflow::sla::SlaStatus, _> =
        monitor.get_sla_status(entity_id).await;
    assert!(result.is_ok());

    let status = result.unwrap();
    match status {
        layanan_perlengkapan::workflow::sla::SlaStatus::Normal { remaining_minutes } => {
            assert!(
                remaining_minutes > 0,
                "Remaining minutes should be positive"
            );
        }
        _ => panic!("Expected Normal status"),
    }

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_sla_status_breached() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create entity in SUBMITTED state, 3 days ago (SLA is 2 days)
    let created_at = Utc::now() - Duration::days(3);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Test: Get SLA status
    let result: Result<layanan_perlengkapan::workflow::sla::SlaStatus, _> =
        monitor.get_sla_status(entity_id).await;
    assert!(result.is_ok());

    let status = result.unwrap();
    match status {
        layanan_perlengkapan::workflow::sla::SlaStatus::Breached {
            breach_duration_minutes,
        } => {
            assert!(
                breach_duration_minutes > 0,
                "Breach duration should be positive"
            );
        }
        _ => panic!("Expected Breached status"),
    }

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_sla_metrics_recorded() {
    // Setup
    let pool = create_test_pool().await;
    let config = WorkflowConfig::default_kebutuhan_bmn();
    let monitor = SlaMonitor::new(config, pool.clone());

    // Create entity with SLA breach
    let created_at = Utc::now() - Duration::days(3);
    let entity_id = create_test_kebutuhan(&pool, "SUBMITTED", created_at).await;

    // Get initial metric values
    let initial_breaches = layanan_perlengkapan::metrics::workflow_sla_breaches_total()
        .with_label_values(&["kebutuhan_bmn", "SUBMITTED"])
        .get();

    let initial_escalations = layanan_perlengkapan::metrics::workflow_escalations_total()
        .with_label_values(&["kebutuhan_bmn", "SUBMITTED", "success"])
        .get();

    // Test: Check SLA and escalate
    let breach_result: Result<Option<layanan_perlengkapan::workflow::sla::SlaBreachInfo>, _> =
        monitor.check_sla(entity_id).await;
    assert!(breach_result.is_ok());

    let breach = breach_result.unwrap();
    assert!(breach.is_some());

    let breach_info = breach.unwrap();
    let escalate_result: Result<(), _> = monitor.escalate_sla_breach(&breach_info).await;
    assert!(escalate_result.is_ok());

    // Verify metrics were incremented
    let final_breaches = layanan_perlengkapan::metrics::workflow_sla_breaches_total()
        .with_label_values(&["kebutuhan_bmn", "SUBMITTED"])
        .get();

    let final_escalations = layanan_perlengkapan::metrics::workflow_escalations_total()
        .with_label_values(&["kebutuhan_bmn", "SUBMITTED", "success"])
        .get();

    assert!(
        final_breaches > initial_breaches,
        "SLA breach metric should be incremented"
    );
    assert!(
        final_escalations > initial_escalations,
        "Escalation metric should be incremented"
    );

    // Cleanup
    cleanup_test_data(&pool, entity_id).await;
}
