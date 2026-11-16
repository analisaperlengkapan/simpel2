//! The admin "Status Sinkronisasi" endpoint must say what integrasi's scheduler recorded.
//!
//! `integrasi.sync_status` is written by every run of integrasi's scheduler
//! (`layanan/integrasi/src/sync_run.rs`); the table and its seed rows here come from integrasi's
//! own migrations (see `common.rs`). These tests put each state the scheduler can leave in a row,
//! then read it back through the endpoint the admin page calls, and check the judgement
//! (`keadaan`) an administrator acts on, plus who may read it at all.
//!
//! Roles ride in the mock bearer token `mock::<role>::<user_id>::<satker>`.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::Value;
use std::collections::BTreeMap;

const ADMIN: &str = "00000000-0000-0000-0000-000000000009";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const PUSAT: &str = "00000000-0000-0000-0000-000000000004";
const SATKER: &str = "SKR001";
const PATH: &str = "/admin/status-sinkronisasi";

async fn get(server: &TestServer, role: &str, user: &str) -> axum_test::TestResponse {
    server
        .get(PATH)
        .add_header(
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user}::{SATKER}"
            ))
            .unwrap(),
        )
        .await
}

/// Each state the scheduler can leave a row in, written the way `sync_run.rs` writes it.
const STATES: &str = "
    -- A run that completed five minutes ago; the next is due in five hours.
    UPDATE integrasi.sync_status SET last_sync_status = 'completed',
           last_sync_started_at = now() - interval '10 minutes',
           last_sync_completed_at = now() - interval '5 minutes',
           last_success_at = now() - interval '5 minutes',
           last_sync_records_count = 545, last_sync_error = NULL,
           sync_frequency = '0 */6 * * * Asia/Jakarta',
           next_sync_scheduled_at = now() + interval '5 hours',
           total_syncs_completed = 3, total_syncs_failed = 0
     WHERE module = 'MySIMKARI' AND endpoint = 'satker';
    -- A run that failed; the last good one was six hours earlier.
    UPDATE integrasi.sync_status SET last_sync_status = 'failed',
           last_sync_started_at = now() - interval '10 minutes',
           last_sync_completed_at = now() - interval '5 minutes',
           last_success_at = now() - interval '6 hours',
           last_sync_records_count = 21328,
           last_sync_error = 'Sinkronisasi tidak lengkap: satker 9003: HTTP 500',
           sync_frequency = '0 */6 * * * Asia/Jakarta',
           next_sync_scheduled_at = now() + interval '5 hours',
           total_syncs_completed = 3, total_syncs_failed = 1
     WHERE module = 'MySIMKARI' AND endpoint = 'pegawai';
    -- Running now.
    UPDATE integrasi.sync_status SET last_sync_status = 'running',
           last_sync_started_at = now() - interval '2 minutes',
           sync_frequency = '0 4 * * 0 Asia/Jakarta',
           next_sync_scheduled_at = now() + interval '6 days'
     WHERE module = 'SIMAN' AND endpoint = 'aset';
    -- Completed once, but the next scheduled run never started.
    UPDATE integrasi.sync_status SET last_sync_status = 'completed',
           last_sync_started_at = now() - interval '2 days',
           last_sync_completed_at = now() - interval '2 days',
           last_success_at = now() - interval '2 days',
           next_sync_scheduled_at = now() - interval '2 hours'
     WHERE module = 'MonSAKTI_ADM' AND endpoint = 'ref_admin';
    -- Started, then killed (OOM, deadline): still `running` long after it should have ended.
    UPDATE integrasi.sync_status SET last_sync_status = 'running',
           last_sync_started_at = now() - interval '1 day',
           next_sync_scheduled_at = now() - interval '2 hours'
     WHERE module = 'MonSAKTI_ADM' AND endpoint = 'ref_jns_spp';
    -- MonSAKTI_ADM/ref_bank is left as migration 002 seeded it: never ran.
";

#[tokio::test]
async fn the_admin_sees_what_the_scheduler_recorded() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // As seeded by integrasi's migration and before any run: six rows, none ever ran.
    let fresh = get(&server, "admin", ADMIN).await;
    assert_eq!(fresh.status_code(), 200, "{:?}", fresh.text());
    let fresh = fresh.json::<Value>();
    let rows = fresh["data"].as_array().expect("data is a list");
    assert_eq!(rows.len(), 6, "{fresh}");
    assert!(
        rows.iter().all(|r| r["keadaan"] == "BELUM_PERNAH"),
        "nothing ran yet: {fresh}"
    );

    db.pool()
        .get()
        .await
        .unwrap()
        .batch_execute(STATES)
        .await
        .expect("write the states");

    let res = get(&server, "admin", ADMIN).await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    let by_key: BTreeMap<(String, String), &Value> = body["data"]
        .as_array()
        .expect("data is a list")
        .iter()
        .map(|r| {
            (
                (
                    r["sumber"].as_str().unwrap().to_string(),
                    r["bagian"].as_str().unwrap().to_string(),
                ),
                r,
            )
        })
        .collect();
    let row = |s: &str, b: &str| -> &Value {
        by_key
            .get(&(s.to_string(), b.to_string()))
            .unwrap_or_else(|| panic!("no row {s}/{b}: {body}"))
    };
    let keadaan: BTreeMap<String, String> = by_key
        .iter()
        .map(|((s, b), r)| {
            (
                format!("{s}/{b}"),
                r["keadaan"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let expected: BTreeMap<String, String> = [
        ("MySIMKARI/satker", "BERHASIL"),
        ("MySIMKARI/pegawai", "GAGAL"),
        ("SIMAN/aset", "BERJALAN"),
        ("MonSAKTI_ADM/ref_admin", "TERLAMBAT"),
        ("MonSAKTI_ADM/ref_jns_spp", "TERLAMBAT"),
        ("MonSAKTI_ADM/ref_bank", "BELUM_PERNAH"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    assert_eq!(keadaan, expected, "{body}");

    // What the page shows next to the judgement.
    let ok = row("MySIMKARI", "satker");
    assert_eq!(ok["jumlah_data"], 545);
    assert_eq!(ok["jadwal"], "0 */6 * * * Asia/Jakarta");
    assert!(ok["galat"].is_null());
    assert_eq!(
        (ok["total_berhasil"].as_i64(), ok["total_gagal"].as_i64()),
        (Some(3), Some(0))
    );

    let failed = row("MySIMKARI", "pegawai");
    assert_eq!(
        failed["galat"],
        "Sinkronisasi tidak lengkap: satker 9003: HTTP 500"
    );
    // After a failure the page can still say how old the data is: the last success is older
    // than the failed run's end.
    let success = failed["berhasil_terakhir"].as_str().expect("last success");
    let ended = failed["selesai_terakhir"]
        .as_str()
        .expect("end of the failed run");
    assert!(
        chrono::DateTime::parse_from_rfc3339(success).unwrap()
            < chrono::DateTime::parse_from_rfc3339(ended).unwrap(),
        "{failed}"
    );
    assert_eq!(failed["jumlah_data"], 21328);

    let never = row("MonSAKTI_ADM", "ref_bank");
    assert!(never["mulai_terakhir"].is_null() && never["berhasil_terakhir"].is_null());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn only_an_administrator_may_read_it() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    for (role, user) in [
        ("operator", OPERATOR),
        ("validator_pusat", PUSAT),
        ("validator_wilayah", PUSAT),
    ] {
        let res = get(&server, role, user).await;
        assert_eq!(res.status_code(), 403, "{role}: {:?}", res.text());
    }
    let unauthenticated = server.get(PATH).await;
    assert_eq!(unauthenticated.status_code(), 401);
    teardown_test_db(&db_name).await;
}
