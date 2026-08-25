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

/// `/bank-aset/dashboard` must report the SIMAN asset taxonomy, and must not
/// die on a NULL.
///
/// Two faults met here, and the endpoint had NO test of any kind.
///
/// 1. The breakdown read `kategori_aset` with a bare `r.get()` into a `String`.
///    That column is written by no ingest path at all, and in real SIMAN data
///    it is NULL in 624 528 of 624 533 rows — the only populated rows were the
///    five the e2e seed inserts itself. `r.get()` panics on NULL, and
///    `panic = "abort"` makes that process death rather than a 500: opening the
///    Bank Aset dashboard killed the backend for every user and the frontend
///    retried into a crash loop. It surfaced only as unrelated e2e flakiness
///    and intermittent 401s while the process restarted.
///
/// 2. Even guarded, `kategori_aset` is the wrong source. The real taxonomy is
///    SIMAN's `jenis_aset` (Tanah, Gedung dan Bangunan, Alat Angkutan Bermotor,
///    Peralatan Mesin Khusus/Non TIK, ...), which is populated for every row.
///    Keyed on the dead column, the tile counted 0 categories and the dropdown
///    was empty.
///
/// So the fixture below is shaped like production — `kategori_aset` NULL
/// throughout — plus one row with a BLANK `jenis_aset` to pin the fallback.
/// Blank rather than NULL because `jenis_aset` is NOT NULL in the schema (both
/// in the test harness and on staging), so the empty string is the only
/// degenerate value actually reachable; that is exactly why it is the safe
/// column to key on and `kategori_aset` was not.
#[tokio::test]
async fn bank_aset_dashboard_reports_siman_jenis_and_survives_null() {
    let (app, db, db_name) = setup_test_app().await;

    let client = db.pool().get().await.unwrap();
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, kategori_aset, nama, ur_kondisi, kdsatker_keu, kode_barang, rph_aset)
             VALUES
                ('Peralatan Mesin Non TIK', NULL, 'Avanza',  'BAIK', 'KEU-A', 'KB-1', '250000000'),
                ('Peralatan Mesin Non TIK', NULL, 'Laptop',  'BAIK', 'KEU-A', 'KB-2', '15000000'),
                ('Tanah',                   NULL, 'Kavling', 'BAIK', 'KEU-B', 'KB-3', '4000000'),
                ('',                        NULL, 'Misteri', 'BAIK', 'KEU-B', 'KB-4', '1000')",
            &[],
        )
        .await
        .unwrap();
    drop(client);

    let server = TestServer::new(app);
    let res = get(&server, "/bank-aset/dashboard").await;
    assert_eq!(
        res.status_code(),
        200,
        "a NULL asset type must not kill the endpoint: {}",
        res.text()
    );

    let body: serde_json::Value = res.json();
    let data = &body["data"];
    let breakdown = data["kategori_breakdown"]
        .as_array()
        .expect("kategori_breakdown array");
    let by_name: std::collections::HashMap<&str, i64> = breakdown
        .iter()
        .map(|k| {
            (
                k["kategori"].as_str().unwrap_or_default(),
                k["count"].as_i64().unwrap_or_default(),
            )
        })
        .collect();

    // Reading the SIMAN column, not the dead one: had this still read
    // `kategori_aset` every row here would collapse into a single NULL bucket.
    assert_eq!(
        by_name.get("Peralatan Mesin Non TIK"),
        Some(&2),
        "SIMAN jenis_aset must drive the breakdown, got {by_name:?}"
    );
    assert_eq!(by_name.get("Tanah"), Some(&1), "got {by_name:?}");
    assert_eq!(
        by_name.get("TIDAK DIKETAHUI"),
        Some(&1),
        "a row with a blank jenis_aset must fall back to a label, got {by_name:?}"
    );

    // Three distinct types, counted off the same expression the breakdown uses.
    assert_eq!(data["total_kategori"], 3);

    teardown_test_db(&db_name).await;
}
