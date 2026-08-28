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

// The sibling `/kebutuhan-bmn/search` is NOT covered here, and deliberately:
// it is broken in its own right and in more than one way, which this suite
// found the moment it first returned a row —
// `error retrieving column status_nama: invalid column status_nama`. It
// selects `k.*` from the base table and maps the result into the shape of
// `vw_kebutuhan_bmn_summary`, so the first matching row panics; with
// `panic = "abort"` in the release profile that takes the process, not just
// the request. Three of its filters (`satker_id`, `kode_barang`, `is_sbsk`)
// also name columns the base table does not have. That is its own change,
// with its own tests, rather than a fix smuggled in beside this one.
