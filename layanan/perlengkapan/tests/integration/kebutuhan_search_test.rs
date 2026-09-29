//! The kebutuhan search surfaces, exercised through the real router.
//!
//! `/kebutuhan-bmn/search/suggestions` answered 500 to every caller in every
//! environment from the day it was written: `SELECT DISTINCT nama … ORDER BY
//! similarity(nama, $2)` is rejected at PARSE time, so no input reached it and
//! no data was needed to trigger it. Nothing caught that, because nothing ever
//! called it — the route-coverage gate walks frontend routes, and a backend
//! endpoint with no caller has no route to walk.
//!
//! So these tests are deliberately shallow and deliberately exhaustive about
//! one thing: every search endpoint answers, with a shape, for a real role.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

fn auth_headers(
    role: &str,
    satker: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_static("00000000-0000-0000-0000-000000000001"),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_str(satker).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::00000000-0000-0000-0000-000000000001::{}",
                role, satker
            ))
            .unwrap(),
        ),
    ]
}

async fn get(server: &TestServer, path: &str, role: &str, satker: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// Seed one campaign so the suggestion query has something to match. It is not
/// what makes the test meaningful — the old query failed on an empty table too
/// — but a suggestion list that can never be non-empty proves less.
async fn seed_campaign(server: &TestServer) {
    let mut req = server.post("/kebutuhan-bmn/pengajuan").json(&json!({
        "nama": "Kebutuhan Meja Kerja 2026",
        "tahun": 2026,
        "tgl_mulai": "2026-01-01",
        "tgl_selesai": "2026-12-31"
    }));
    for (k, v) in auth_headers("validator_pusat", "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "seed campaign: {:?}", res.text());
}

#[tokio::test]
async fn suggestions_answers_instead_of_failing_to_parse() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_campaign(&server).await;

    let res = get(
        &server,
        "/kebutuhan-bmn/search/suggestions?q=Kebutuhan",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_eq!(
        res.status_code(),
        200,
        "suggestions must not 500: {:?}",
        res.text()
    );
    let body = res.json::<serde_json::Value>();
    let items = body["data"].as_array().expect("a list of suggestions");
    assert!(
        items
            .iter()
            .any(|s| s.as_str() == Some("Kebutuhan Meja Kerja 2026")),
        "the seeded campaign should be suggested: {body:?}"
    );

    teardown_test_db(&db_name).await;
}

/// The distinct set is what the endpoint promises, and it is the reason the
/// original used `SELECT DISTINCT` — which is what made the ordering illegal.
/// Two campaigns sharing a name must yield ONE suggestion.
#[tokio::test]
async fn suggestions_are_distinct() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_campaign(&server).await;
    seed_campaign(&server).await;

    let res = get(
        &server,
        "/kebutuhan-bmn/search/suggestions?q=Meja",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<serde_json::Value>();
    let items = body["data"].as_array().unwrap();
    let hits = items
        .iter()
        .filter(|s| s.as_str() == Some("Kebutuhan Meja Kerja 2026"))
        .count();
    assert_eq!(hits, 1, "duplicates must collapse: {body:?}");

    teardown_test_db(&db_name).await;
}

/// An empty result is the other half: the query must survive matching nothing
/// rather than erroring, which is how a caller's first keystroke behaves.
#[tokio::test]
async fn suggestions_survive_matching_nothing() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        "/kebutuhan-bmn/search/suggestions?q=zzzzzz",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    assert!(
        res.json::<serde_json::Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    teardown_test_db(&db_name).await;
}

// ── /kebutuhan-bmn/search ────────────────────────────────────────────────
//
// Nothing had ever called it either. A probe against staging sees 200 — but
// only because the search happens to match nothing: the query selected `k.*`
// from the base table and handed the row to a mapper that reads
// `vw_kebutuhan_bmn_summary`, so the FIRST matching row panicked on
// `invalid column status_nama`. With `panic = "abort"` in the release profile
// that ends the process, not the request.
//
// So every test below insists the search MATCHES something. An empty result is
// precisely the state in which this endpoint looked healthy.

async fn post(server: &TestServer, path: &str, body: serde_json::Value) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers("validator_pusat", "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

/// POST as the satker's own operator — the only role that enters barang.
async fn post_as_operator(
    server: &TestServer,
    path: &str,
    body: serde_json::Value,
    satker: &str,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers("operator_satker", satker) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn pusat_get(server: &TestServer, path: &str) -> axum_test::TestResponse {
    get(server, path, "validator_pusat", "PUSAT001").await
}

/// A campaign whose text matches, with one participating satker and one barang
/// under it — the two levels the `satker_id` and `kode_barang` filters have to
/// reach through.
async fn seed_full(server: &TestServer) {
    let r = post(
        server,
        "/kebutuhan-bmn/pengajuan",
        json!({"nama": "Kebutuhan Meja Kerja 2026",
               "deskripsi": "Pengadaan meja untuk ruang pelayanan",
               "tahun": 2026, "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "seed campaign: {:?}", r.text());
    let body = r.json::<serde_json::Value>();
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .or_else(|| body["data"]["id"].as_str())
        .unwrap_or_else(|| panic!("no pengajuan id in {body:?}"))
        .to_string();

    let r = post(
        server,
        &format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"),
        json!({"satker_id": "SKR001"}),
    )
    .await;
    assert!(
        r.status_code() == 200 || r.status_code() == 201,
        "seed satker: {:?}",
        r.text()
    );
    let satker_row = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let r = post_as_operator(
        server,
        &format!("/kebutuhan-bmn/satker/{satker_row}/barang"),
        json!({"nama": "Meja Kerja Eselon IV", "jumlah": 3, "kode_barang": "3060201003"}),
        "SKR001",
    )
    .await;
    assert_eq!(r.status_code(), 201, "seed barang: {:?}", r.text());
}

fn rows(res: &axum_test::TestResponse) -> Vec<serde_json::Value> {
    res.json::<serde_json::Value>()["data"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[tokio::test]
async fn a_matching_search_returns_the_summary_shape() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let res = pusat_get(&server, "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let items = rows(&res);
    assert_eq!(items.len(), 1, "the seeded campaign must match: {items:?}");

    // The columns the mapper reads, and the ones the base table never had.
    let row = &items[0];
    assert_eq!(row["nama"], "Kebutuhan Meja Kerja 2026");
    assert!(row["status_nama"].is_string(), "{row:?}");
    assert_eq!(row["total_satker"], 1, "{row:?}");
    assert_eq!(row["total_barang"], 1, "{row:?}");
    assert_eq!(row["total_jumlah_diminta"], 3, "{row:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_total_counts_the_rows_the_page_came_from() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let res = pusat_get(&server, "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20").await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    assert_eq!(
        body["total"],
        body["data"].as_array().unwrap().len(),
        "count and page must agree: {body:?}"
    );

    teardown_test_db(&db_name).await;
}

/// A campaign has no satker column: the filter has to reach the participation
/// rows. It was also typed `Uuid` against a TEXT `kode_satker`.
#[tokio::test]
async fn the_satker_filter_reaches_the_participation_rows() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let hit = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&satker_id=SKR001",
    )
    .await;
    assert_eq!(hit.status_code(), 200, "{:?}", hit.text());
    assert_eq!(rows(&hit).len(), 1);

    let miss = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&satker_id=SKR002",
    )
    .await;
    assert_eq!(miss.status_code(), 200, "{:?}", miss.text());
    assert!(rows(&miss).is_empty(), "SKR002 does not take part");

    teardown_test_db(&db_name).await;
}

/// `kode_barang` lives two levels down, on the barang rows.
#[tokio::test]
async fn the_kode_barang_filter_reaches_two_levels_down() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let hit = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&kode_barang=3060201003",
    )
    .await;
    assert_eq!(hit.status_code(), 200, "{:?}", hit.text());
    assert_eq!(rows(&hit).len(), 1);

    let miss = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&kode_barang=9999999999",
    )
    .await;
    assert_eq!(miss.status_code(), 200, "{:?}", miss.text());
    assert!(rows(&miss).is_empty());

    teardown_test_db(&db_name).await;
}

/// `status_kode` is an integer and the filter arrives as text, so the old
/// `status_kode = ANY($n)` could not even bind. Both spellings must work.
#[tokio::test]
async fn the_status_filter_accepts_a_name_or_a_code() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let by_code = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&status=2000",
    )
    .await;
    assert_eq!(by_code.status_code(), 200, "{:?}", by_code.text());
    assert_eq!(rows(&by_code).len(), 1, "2000 is the seeded state");

    // The label now comes from `ms_workflow_status` ("Draft"), not from
    // `ms_aktivitas_bmn` ("DRAFT"). Both spellings must keep working, so the
    // comparison is case-insensitive — a caller who wrote the old token for a
    // state whose two masters agreed is not broken by the switch.
    for spelling in ["Draft", "DRAFT", "draft"] {
        let by_name = pusat_get(
            &server,
            &format!("/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&status={spelling}"),
        )
        .await;
        assert_eq!(by_name.status_code(), 200, "{:?}", by_name.text());
        assert_eq!(rows(&by_name).len(), 1, "2000 is Draft, spelled {spelling}");
    }

    // A real label that no seeded row carries.
    let neither = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&status=Ditolak",
    )
    .await;
    assert_eq!(neither.status_code(), 200);
    assert!(rows(&neither).is_empty());

    // And the machine tokens that named the WRONG step are deliberately gone:
    // 2004 was `ANALISIS_KELAYAKAN` in the old master and is the queue for
    // Pusat in the enum, so keeping that spelling reachable would keep the
    // mistake reachable.
    let old_token = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&status=ANALISIS_KELAYAKAN",
    )
    .await;
    assert_eq!(old_token.status_code(), 200);
    assert!(
        rows(&old_token).is_empty(),
        "the old token must not resolve to a state at all"
    );

    teardown_test_db(&db_name).await;
}

/// The dates arrive as strings; without forcing the parameter to text first,
/// Postgres resolves the placeholder to timestamptz and the bind fails before
/// any row is read.
#[tokio::test]
async fn the_date_range_filter_binds() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    let res = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20\
         &date_from=2020-01-01T00:00:00Z&date_to=2100-01-01T00:00:00Z",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    assert_eq!(rows(&res).len(), 1);

    let outside = pusat_get(
        &server,
        "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&date_to=2020-01-01T00:00:00Z",
    )
    .await;
    assert_eq!(outside.status_code(), 200, "{:?}", outside.text());
    assert!(rows(&outside).is_empty());

    teardown_test_db(&db_name).await;
}

/// Three of the six sort options named columns a campaign row does not have.
/// Each was a "column does not exist" waiting for a caller to pick that sort,
/// so the assertion is simply: every option a caller can spell must answer.
#[tokio::test]
async fn every_sort_option_names_a_column_that_exists() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_full(&server).await;

    for sort_by in [
        "relevance",
        "created_at",
        "created",
        "updated_at",
        "updated",
        "priority",
        "gap",
        "tahun_anggaran",
        "tahun",
    ] {
        for dir in ["asc", "desc"] {
            let res = pusat_get(
                &server,
                &format!(
                    "/kebutuhan-bmn/search?q=Meja&page=1&per_page=20&sort_by={sort_by}&sort_dir={dir}"
                ),
            )
            .await;
            assert_eq!(
                res.status_code(),
                200,
                "sort_by={sort_by} sort_dir={dir}: {:?}",
                res.text()
            );
            assert_eq!(rows(&res).len(), 1, "sort_by={sort_by} must still match");
        }
    }

    teardown_test_db(&db_name).await;
}
