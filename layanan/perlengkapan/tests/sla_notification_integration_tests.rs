// ============================================================================
// SLA Monitoring with Notification Integration Tests
// Description: SLA breach detection + escalation against a real Postgres.
// Requirements: REQ-W003, REQ-N008, NFR-M004
//
// Uses the shared ephemeral-DB harness (tests/common.rs): each test gets a
// fresh migrated `test_db_<uuid>`. The workflow status lives on the per-satker
// row `pengajuan_kebutuhan_bmn_satker` (status_kode) and the SLA config is keyed
// on the state NAME from `ms_aktivitas_bmn`. ANALISIS_KELAYAKAN (kode 2004) has
// a 3-day (4320 min) SLA in WorkflowConfig::default_kebutuhan_bmn.
//
// These need a real Postgres, so they live under `mod integration` — the CI
// unit-test job runs `cargo test … -- --skip integration::` (ci.yml), which
// excludes them; they run locally and against the e2e stack where a DB exists.
// ============================================================================

mod common;

#[cfg(test)]
mod integration {
    use super::common::{setup_test_db, teardown_test_db};
    use chrono::{DateTime, Duration, Utc};
    use deadpool_postgres::Pool;
    use layanan_perlengkapan::workflow::{config::WorkflowConfig, sla::SlaMonitor};
    use uuid::Uuid;

    /// State kode 2004 = ANALISIS_KELAYAKAN — has a 3-day SLA in the default config.
    const KODE_ANALISIS_KELAYAKAN: i32 = 2004;
    const SEEDED_USER: &str = "00000000-0000-0000-0000-000000000001";

    /// Seed a per-satker workflow row in `status_kode`, whose latest activity (and
    /// thus state-entry time) is `entered_at`. Returns the satker-row id, which is
    /// the entity the SLA monitor checks.
    async fn seed_satker_row(pool: &Pool, status_kode: i32, entered_at: DateTime<Utc>) -> Uuid {
        let client = pool.get().await.expect("db client");
        let pengajuan_id = Uuid::new_v4();
        let satker_row_id = Uuid::new_v4();
        let user_id = Uuid::parse_str(SEEDED_USER).unwrap();

        // Parent pengajuan (NOT NULL: nama, tahun, tgl_mulai, tgl_selesai).
        client
            .execute(
                r#"INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
                    (id, nama, tahun, tgl_mulai, tgl_selesai, created_by, status_kode)
                   VALUES ($1, 'Test Pengajuan', 2026, '2026-01-01', '2026-12-31', $2, $3)"#,
                &[&pengajuan_id, &user_id, &status_kode],
            )
            .await
            .expect("insert pengajuan");

        // Per-satker workflow row carrying the status_kode.
        client
            .execute(
                r#"INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
                    (id, pengajuan_id, satker_id, status_kode, created_by, created_at)
                   VALUES ($1, $2, 'SKR001', $3, $4, $5)"#,
                &[
                    &satker_row_id,
                    &pengajuan_id,
                    &status_kode,
                    &user_id,
                    &entered_at,
                ],
            )
            .await
            .expect("insert satker row");

        // Activity marking entry into the current state (drives state_entered_at).
        client
            .execute(
                r#"INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                    (pengajuan_satker_id, to_status_kode, user_id, aksi, komentar, created_at)
                   VALUES ($1, $2, $3, 'TRANSITION', 'enter state', $4)"#,
                &[&satker_row_id, &status_kode, &user_id, &entered_at],
            )
            .await
            .expect("insert aktivitas");

        satker_row_id
    }

    #[tokio::test]
    async fn test_sla_breach_detection() {
        let (db, db_name) = setup_test_db().await;
        let pool = db.pool().clone();
        let monitor = SlaMonitor::new(WorkflowConfig::default_kebutuhan_bmn(), pool.clone());

        // Entered ANALISIS_KELAYAKAN 4 days ago; SLA is 3 days → breach.
        let entered = Utc::now() - Duration::days(4);
        let id = seed_satker_row(&pool, KODE_ANALISIS_KELAYAKAN, entered).await;

        let breach = monitor.check_sla(id).await.expect("check_sla ok");
        assert!(breach.is_some(), "SLA breach should be detected");
        let info = breach.unwrap();
        assert_eq!(info.entity_id, id);
        assert_eq!(info.current_state, "ANALISIS_KELAYAKAN");
        assert!(info.breach_duration_minutes > 0);

        teardown_test_db(&db_name).await;
    }

    #[tokio::test]
    async fn test_sla_no_breach() {
        let (db, db_name) = setup_test_db().await;
        let pool = db.pool().clone();
        let monitor = SlaMonitor::new(WorkflowConfig::default_kebutuhan_bmn(), pool.clone());

        // Entered 1 day ago; SLA is 3 days → no breach.
        let entered = Utc::now() - Duration::days(1);
        let id = seed_satker_row(&pool, KODE_ANALISIS_KELAYAKAN, entered).await;

        let breach = monitor.check_sla(id).await.expect("check_sla ok");
        assert!(breach.is_none(), "no SLA breach expected within window");

        teardown_test_db(&db_name).await;
    }

    #[tokio::test]
    async fn test_sla_check_all() {
        let (db, db_name) = setup_test_db().await;
        let pool = db.pool().clone();
        let monitor = SlaMonitor::new(WorkflowConfig::default_kebutuhan_bmn(), pool.clone());

        let breaching = seed_satker_row(
            &pool,
            KODE_ANALISIS_KELAYAKAN,
            Utc::now() - Duration::days(4),
        )
        .await;
        let healthy = seed_satker_row(
            &pool,
            KODE_ANALISIS_KELAYAKAN,
            Utc::now() - Duration::days(1),
        )
        .await;

        let breaches = monitor.check_all_sla().await.expect("check_all_sla ok");
        assert!(
            breaches.iter().any(|b| b.entity_id == breaching),
            "breaching row should be reported"
        );
        assert!(
            !breaches.iter().any(|b| b.entity_id == healthy),
            "healthy row should not be reported"
        );

        teardown_test_db(&db_name).await;
    }

    #[tokio::test]
    async fn test_sla_escalation_with_notification() {
        let (db, db_name) = setup_test_db().await;
        let pool = db.pool().clone();

        // In-process NotifikasiService (no gRPC server) wired as the notifier port.
        let notifier: std::sync::Arc<dyn layanan_perlengkapan::contracts::NotificationSender> =
            std::sync::Arc::new(
                layanan_perlengkapan::notifikasi::service::NotifikasiService::new(pool.clone()),
            );
        let monitor = SlaMonitor::with_notifier(
            WorkflowConfig::default_kebutuhan_bmn(),
            pool.clone(),
            notifier,
        );

        let id = seed_satker_row(
            &pool,
            KODE_ANALISIS_KELAYAKAN,
            Utc::now() - Duration::days(4),
        )
        .await;

        // monitor_and_escalate detects the breach, resolves approver/requester,
        // sends notifications, and logs an SLA_ESCALATION activity row.
        let escalated = monitor
            .monitor_and_escalate()
            .await
            .expect("monitor_and_escalate ok");
        assert!(escalated >= 1, "at least one breach should be escalated");

        // The escalation activity row was recorded.
        let client = pool.get().await.unwrap();
        let n: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas \
                 WHERE pengajuan_satker_id = $1 AND aksi = 'SLA_ESCALATION'",
                &[&id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(n, 1, "one SLA_ESCALATION activity row expected");

        teardown_test_db(&db_name).await;
    }
}
