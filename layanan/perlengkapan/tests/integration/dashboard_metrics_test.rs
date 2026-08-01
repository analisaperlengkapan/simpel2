//! Layer-2 integration tests for the dashboard metrics + export endpoints (#116).
//!
//! These exist because #116 was found the expensive way. The metrics queries
//! compile fine, read plausibly, and are wrong — they targeted relations that
//! exist in no environment, and once that was fixed they still killed the
//! service on a NUMERIC-into-f64 read. Nothing in the suite executed them, so
//! the only thing that could catch either fault was a full Playwright e2e run
//! against a built stack: ~30 minutes per attempt, and the failure presented as
//! twelve unrelated tests dying of ENOTFOUND.
//!
//! Every assertion here is about the query actually RUNNING against a real
//! PostgreSQL and its columns actually decoding into the types the Rust readers
//! declare. That is the part `cargo check` cannot see and a psql eyeball cannot
//! either — psql renders a NUMERIC perfectly; it is tokio_postgres that has no
//! `FromSql<f64>` for it.
//!
//! Note the failure MODE these guard against is process death, not a 500: the
//! release profile sets `panic = "abort"`, so a bad column read takes the whole
//! backend down. Under `cargo test` the panic surfaces as a failed test instead,
//! which is exactly why running them here is cheap and running them in e2e is not.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;

/// Authenticated GET. The dashboard endpoints reject anonymous callers, so the
/// mock bearer token (`mock::<role>::<user_id>::<satker>`, decoded by
/// `AuthencClient::dummy()`) is required — asserting the guard itself is
/// `rbac_403_test`'s job, not this file's.
async fn get(server: &TestServer, path: &str) -> axum_test::TestResponse {
    server
        .get(path)
        .add_header(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_static(
                "Bearer mock::validator_pusat::00000000-0000-0000-0000-000000000003::SKR001",
            ),
        )
        .await
}

/// Both metrics and export must answer on a database that has been migrated but
/// never populated — the state of every environment before its first SIMAN sync
/// and before any campaign exists.
///
/// This is not a trivial case: `SUM()` over zero rows is NULL, and reading NULL
/// into `i64` panics (aborting the process in release). It was a real bug.
#[tokio::test]
async fn dashboard_metrics_survive_an_empty_database() {
    let (app, db, db_name) = setup_test_app().await;

    // Own the precondition rather than assuming the harness leaves SIMAN empty:
    // it seeds assets for the penghapusan tests, and "empty" is the whole point
    // of this case.
    db.pool()
        .get()
        .await
        .unwrap()
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();

    let server = TestServer::new(app);

    let res = get(&server, "/dashboard/perlengkapan?tahun_anggaran=2026").await;
    assert_eq!(
        res.status_code(),
        200,
        "metrics on an empty DB: {}",
        res.text()
    );

    // Shape check: every sub-query must have produced its key, not just "the
    // request did not 500". A partially-populated payload is how a silently
    // swallowed error would look.
    // NOTE the envelope asymmetry, pinned here so a future change to either side
    // breaks loudly: `/dashboard/perlengkapan` returns the metrics object BARE,
    // while `/dashboard/stats` wraps its payload in `{success, data}`. Both are
    // consumed by the same FE module.
    let data: serde_json::Value = res.json();
    for key in [
        "kebutuhan_metrics",
        "gap_analysis",
        "pakaian_dinas_metrics",
        "workflow_metrics",
        "asset_utilization",
        "pemakaian_metrics",
        "penghapusan_metrics",
    ] {
        assert!(!data[key].is_null(), "missing `{key}` in {data}");
    }

    // The specific numbers that used to arrive as NULL and panic.
    assert_eq!(data["asset_utilization"]["total_assets"], 0);
    assert_eq!(data["asset_utilization"]["assets_in_good_condition"], 0);
    assert_eq!(data["asset_utilization"]["utilization_percentage"], 0.0);
    // avg_hours: the NUMERIC-vs-f64 abort site. COALESCE(...,0) makes it 0.0.
    assert_eq!(
        data["workflow_metrics"]["average_processing_time_hours"],
        0.0
    );

    teardown_test_db(&db_name).await;
}

/// `/dashboard/stats` read two views (`integrasi.v_siman_summary_total` and
/// `..._per_kategori`) that NOTHING in this repo creates, so it was a guaranteed
/// 500 in every environment. Its e2e passed regardless because it asserted the
/// static card labels, which render whether or not the fetch succeeds.
#[tokio::test]
async fn dashboard_stats_aggregates_the_siman_sot() {
    let (app, db, db_name) = setup_test_app().await;

    // Rows chosen to pin the exact faults this endpoint had:
    //  - `kondisi` NULL / `ur_kondisi` set, the shape the e2e seed and much real
    //    data actually carry (the old filter tested `kondisi` alone and counted
    //    zero good assets);
    //  - a non-numeric `rph_aset`, which a bare ::FLOAT8 cast would throw on;
    //  - two distinct kdsatker_keu, so total_satker is not just row count.
    let client = db.pool().get().await.unwrap();
    // Exact counts are asserted below, so this test owns the table outright
    // instead of adding to whatever the harness seeded for other suites.
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, kategori_aset, nama, ur_kondisi, kdsatker_keu, kode_barang, rph_aset)
             VALUES
                ('Peralatan dan Mesin', 'Alat Angkutan', 'Avanza',  'BAIK',         'KEU-A', 'KB-1', '250000000'),
                ('Peralatan dan Mesin', 'Alat Kantor',   'Laptop',  'BAIK',         'KEU-A', 'KB-2', '15000000'),
                ('Peralatan dan Mesin', 'Alat Kantor',   'Printer', 'RUSAK RINGAN', 'KEU-B', 'KB-3', '4000000'),
                ('Peralatan dan Mesin', 'Alat Kantor',   'Scanner', 'BAIK',         'KEU-B', 'KB-4', 'n/a')",
            &[],
        )
        .await
        .unwrap();
    drop(client);

    let server = TestServer::new(app);
    let res = get(&server, "/dashboard/stats").await;
    assert_eq!(res.status_code(), 200, "stats: {}", res.text());

    let body: serde_json::Value = res.json();
    let data = &body["data"];
    assert_eq!(data["total_aset"], 4);
    assert_eq!(data["total_satker"], 2, "distinct kdsatker_keu");
    // 3 rows say BAIK via ur_kondisi; the old `kondisi = 'BAIK'` filter saw none.
    assert_eq!(data["aset_baik"], 3);
    assert_eq!(data["aset_rusak"], 1);
    // The 'n/a' row contributes 0 rather than throwing.
    assert_eq!(data["total_nilai_aset"], 269_000_000.0);

    let categories = data["categories"].as_array().expect("categories array");
    assert_eq!(categories.len(), 2, "Alat Kantor + Alat Angkutan");
    // Ordered by count DESC: Alat Kantor has 3 rows, Alat Angkutan 1.
    assert_eq!(categories[0]["category"], "Alat Kantor");
    assert_eq!(categories[0]["count"], 3);
    assert_eq!(categories[0]["value"], 19_000_000.0);

    teardown_test_db(&db_name).await;
}

/// The two export handlers call `get_perlengkapan_dashboard_metrics` FIRST, so
/// they inherit every metrics fault — which is why #116 reported BOTH formats
/// failing even though pdf never touches the spreadsheet writer.
///
/// Asserting a non-empty body matters: a handler that returns 200 with zero
/// bytes would satisfy a status-only check while producing an unopenable file.
#[tokio::test]
async fn dashboard_exports_produce_a_non_empty_document() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for fmt in ["excel", "pdf"] {
        let res = get(
            &server,
            &format!("/dashboard/perlengkapan/export/{fmt}?tahun_anggaran=2026"),
        )
        .await;
        assert_eq!(res.status_code(), 200, "export/{fmt}: {}", res.text());
        assert!(
            !res.as_bytes().is_empty(),
            "export/{fmt} returned an empty body"
        );
    }

    teardown_test_db(&db_name).await;
}
