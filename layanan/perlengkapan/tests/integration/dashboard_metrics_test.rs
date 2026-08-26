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
            // Production shape: `kategori_aset` and `kode_barang` stay NULL
            // (SIMAN writes neither), the taxonomy lives in `jenis_aset` and the
            // barang code in `kd_brg`. This fixture used to fill the dead pair
            // and assert the breakdown off `kategori_aset` — so the test PASSED
            // while the endpoint bucketed all 624 533 production assets under a
            // single '(tanpa kategori)' label. `jenis_aset` values are real
            // SIMAN taxonomy entries.
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nama, ur_kondisi, kdsatker_keu, kd_brg, rph_aset)
             VALUES
                ('Alat Angkutan Bermotor',   'Avanza',  'Baik',         'KEU-A', '3020104001', '250000000'),
                ('Peralatan Mesin Non TIK',  'Laptop',  'Baik',         'KEU-A', '3050105039', '15000000'),
                ('Peralatan Mesin Non TIK',  'Printer', 'Rusak Ringan', 'KEU-B', '3050105040', '4000000'),
                ('Peralatan Mesin Non TIK',  'Scanner', 'Baik',         'KEU-B', '3050105041', 'n/a')",
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

    // Breakdown driven by `jenis_aset`. Had it still read `kategori_aset` —
    // NULL in every real row — all four would collapse into one bucket, which
    // is precisely what production served.
    let categories = data["categories"].as_array().expect("categories array");
    assert_eq!(
        categories.len(),
        2,
        "Peralatan Mesin Non TIK + Alat Angkutan Bermotor, got {categories:?}"
    );
    // Ordered by count DESC: Non TIK has 3 rows, Angkutan Bermotor 1.
    assert_eq!(categories[0]["category"], "Peralatan Mesin Non TIK");
    assert_eq!(categories[0]["count"], 3);
    assert_eq!(categories[0]["value"], 19_000_000.0);

    teardown_test_db(&db_name).await;
}

/// Gap analysis matched ZERO SIMAN assets, so every gap was reported as the
/// full requested quantity — including for asset types the organisation already
/// owns tens of thousands of.
///
/// Two independent reasons, either alone fatal, and the second is the one a
/// column-level fix misses:
///
///   1. the join read `sa.kode_barang`, populated in 0 of 624 533 rows;
///   2. FORMAT — `kd_brg`, the column SIMAN does fill, holds ten digits with NO
///      dots (`3050201002`), while the request side stores the dotted
///      presentation form (`3.05.02.01.002`). Correcting the column alone still
///      matches nothing.
///
/// The fixture reproduces exactly that asymmetry: dotted on the request side,
/// undotted in SIMAN. Reverting either half of the fix makes this fail.
/// Measured on staging after the fix: 0 -> 36 787 existing good assets for one
/// requested code.
#[tokio::test]
async fn gap_analysis_matches_siman_across_the_dotted_kode_barang_divide() {
    let (app, db, db_name) = setup_test_app().await;
    let client = db.pool().get().await.unwrap();

    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    // Four assets under one barang code: three serviceable, one not. The
    // condition column is the live one, in the Title Case production uses.
    client
        .execute(
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nama, ur_kondisi, kdsatker_keu, kd_brg, rph_aset)
             VALUES
                ('Peralatan Mesin Non TIK', 'Kursi A', 'Baik',       'KEU-A', '3050201002', '1000'),
                ('Peralatan Mesin Non TIK', 'Kursi B', 'Baik',       'KEU-A', '3050201002', '1000'),
                ('Peralatan Mesin Non TIK', 'Kursi C', 'Baik',       'KEU-A', '3050201002', '1000'),
                ('Peralatan Mesin Non TIK', 'Kursi D', 'Rusak Berat','KEU-A', '3050201002', '1000')",
            &[],
        )
        .await
        .unwrap();

    // A campaign in the CURRENT year — the query filters on
    // `EXTRACT(YEAR FROM CURRENT_DATE)`, so a hard-coded year would make this
    // test start passing vacuously next January.
    let campaign = uuid::Uuid::new_v4();
    let satker_row = uuid::Uuid::new_v4();
    client
        .execute(
            "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn
                (id, nama, tahun, tgl_mulai, tgl_selesai, status_kode, scope_satker)
             VALUES ($1, 'Uji Gap', EXTRACT(YEAR FROM CURRENT_DATE)::int,
                     CURRENT_DATE, CURRENT_DATE, 2000, 'semua')",
            &[&campaign],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker
                (id, pengajuan_id, satker_id, satker_nama, status_kode)
             VALUES ($1, $2, '005001', 'KEJAKSAAN NEGERI UJI', 2001)",
            &[&satker_row, &campaign],
        )
        .await
        .unwrap();
    client
        .execute(
            // DOTTED on the request side — the format the app actually stores.
            "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
                (pengajuan_satker_id, nama, kode_barang, jumlah)
             VALUES ($1, 'Kursi Kerja', '3.05.02.01.002', 10)",
            &[&satker_row],
        )
        .await
        .unwrap();
    drop(client);

    let server = TestServer::new(app);
    // `tahun_anggaran` is required by the extractor. The gap query itself keys
    // on CURRENT_DATE, so the year is derived rather than hard-coded — a literal
    // would make this test pass vacuously the moment the calendar rolls over.
    let year = chrono::Utc::now().format("%Y").to_string();
    let res = get(
        &server,
        &format!("/dashboard/perlengkapan?tahun_anggaran={year}"),
    )
    .await;
    assert_eq!(res.status_code(), 200, "metrics: {}", res.text());

    let data: serde_json::Value = res.json();
    let gaps = data["gap_analysis"].as_array().expect("gap_analysis array");
    let row = gaps
        .iter()
        .find(|g| g["kode_barang"] == "3.05.02.01.002")
        .unwrap_or_else(|| panic!("requested code absent from gap analysis: {gaps:?}"));

    // Three of the four are serviceable. Had the join read the dead column, or
    // compared the dotted form against the undotted one, this would be 0 and the
    // gap would be the full 10.
    assert_eq!(
        row["existing_good_quantity"], 3,
        "SIMAN assets must be counted through kd_brg with dots normalised: {row}"
    );
    assert_eq!(row["gap"], 7, "gap = requested 10 - existing 3: {row}");

    teardown_test_db(&db_name).await;
}

/// The two export handlers call `get_perlengkapan_dashboard_metrics` FIRST, so
/// they inherit every metrics fault — which is why #116 reported BOTH formats
/// failing even though pdf never touches the spreadsheet writer.
///
/// Asserting a non-empty body was NOT enough, and this test proves why it now
/// checks the file SIGNATURE instead.
///
/// `export/pdf` returned 200 with a non-empty body for months while serving an
/// HTML document under `Content-Type: application/pdf` and
/// `filename="dashboard_perlengkapan_<year>.pdf"` — `html_to_simple_pdf` was a
/// placeholder that returned `html.as_bytes()`. Users clicking "Export PDF" got
/// a .pdf no reader can open, with no error anywhere in the stack. Status +
/// length could never see it; the first four bytes can.
#[tokio::test]
async fn dashboard_exports_produce_a_document_of_the_declared_format() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // (format, magic bytes). XLSX is a ZIP container, hence "PK".
    for (fmt, magic) in [("excel", b"PK".as_slice()), ("pdf", b"%PDF-".as_slice())] {
        let res = get(
            &server,
            &format!("/dashboard/perlengkapan/export/{fmt}?tahun_anggaran=2026"),
        )
        .await;
        assert_eq!(res.status_code(), 200, "export/{fmt}: {}", res.text());
        let body = res.as_bytes();
        assert!(!body.is_empty(), "export/{fmt} returned an empty body");
        assert!(
            body.starts_with(magic),
            "export/{fmt} is not really {fmt}: expected it to start with {:?}, got {:?}",
            String::from_utf8_lossy(magic),
            String::from_utf8_lossy(&body[..body.len().min(32)])
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

/// The `nup` column is the second column in `integrasi.siman_aset` that the
/// ingest never writes, found after the `kategori_aset` fix above.
///
/// The ingest builds its INSERT column list from the SIMAN payload's own JSON
/// keys (`layanan/integrasi/src/db.rs`), so a schema column only ever fills if
/// SIMAN sends a field of that name. SIMAN sends `no_aset`; it never sends
/// `nup`. On staging `nup` was non-empty in 5 of 624 533 rows — exactly the five
/// the e2e seed inserts — so the list rendered "-" in its NUP column for every
/// real asset and `GET /bank-aset/lookup?nup=` 404'd for all of them.
///
/// The fixture reproduces that shape deliberately: `nup` NULL everywhere, real
/// values only in `no_aset`. Reverting `ASSET_NUP_SQL` makes both halves fail.
#[tokio::test]
async fn bank_aset_reads_nup_from_siman_no_aset() {
    let (app, db, db_name) = setup_test_app().await;

    let client = db.pool().get().await.unwrap();
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            // `nup` deliberately NULL — the production shape. NUP lives in no_aset.
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nup, no_aset, ur_sskel, nama, kd_brg, ur_kondisi, kdsatker_keu, rph_aset)
             VALUES
                ('Peralatan Mesin Non TIK', NULL, '43', 'Display',  'Display',  '3050105039', 'BAIK', 'KEU-A', '250000000'),
                ('Peralatan Mesin Non TIK', NULL, '44', 'Display',  'Display',  '3050105039', 'BAIK', 'KEU-A', '15000000'),
                ('Peralatan Mesin Non TIK', NULL, '',   'Tanpa NUP','Tanpa NUP','3050105039', 'BAIK', 'KEU-A', '1000')",
            &[],
        )
        .await
        .unwrap();
    drop(client);

    let server = TestServer::new(app);

    // 1) The list projection must surface no_aset as the NUP.
    let res = get(&server, "/bank-aset?per_page=50").await;
    assert_eq!(res.status_code(), 200, "list failed: {}", res.text());
    let body: serde_json::Value = res.json();
    let rows = body["data"].as_array().expect("data array");
    let nups: Vec<Option<&str>> = rows.iter().map(|r| r["nup"].as_str()).collect();
    assert!(
        nups.contains(&Some("43")) && nups.contains(&Some("44")),
        "NUP must come from SIMAN no_aset; had this still read the `nup` column \
         every row would be null. got {nups:?}"
    );
    // A blank no_aset must render as absent, not as an invented value.
    assert!(
        nups.contains(&None),
        "a row with a blank no_aset must yield a null NUP, got {nups:?}"
    );

    // 2) The lookup endpoint must FIND a real asset by its NUP. This is the half
    //    that returned 404 for all 624 533 staging rows.
    let res = get(&server, "/bank-aset/lookup?nup=43").await;
    assert_eq!(
        res.status_code(),
        200,
        "lookup by a real NUP must resolve, not 404: {}",
        res.text()
    );
    let body: serde_json::Value = res.json();
    assert_eq!(
        body["data"]["nup"], "43",
        "lookup must echo the NUP it matched on, got {body}"
    );

    // 3) Search must reach NUP, which is what the FE placeholder
    //    "Cari nama/kode/NUP/merk..." promises.
    let res = get(&server, "/bank-aset?per_page=50&search=44").await;
    assert_eq!(res.status_code(), 200, "search failed: {}", res.text());
    let body: serde_json::Value = res.json();
    let found: Vec<Option<&str>> = body["data"]
        .as_array()
        .expect("data array")
        .iter()
        .map(|r| r["nup"].as_str())
        .collect();
    assert!(
        found.contains(&Some("44")),
        "searching a NUP must match it, got {found:?}"
    );

    teardown_test_db(&db_name).await;
}
