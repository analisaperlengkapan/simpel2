//! The feasibility analysis, against the SIMAN source of truth it is supposed
//! to compare with.
//!
//! `get_analisis_kelayakan` exists to answer "how much of what this satker
//! asked for does it already hold". It never asked. The SIMAN client it went
//! through was an `Option` that no code path in the application ever filled —
//! `with_siman` and `set_siman` had zero callers — so every request fell
//! through to the stored `existing_count` column and an empty asset list,
//! while the screen said the comparison had been made.
//!
//! It reads `integrasi.siman_aset` now, the same source `bank_aset` reads.
//! These tests pin the two things that silently return an empty answer rather
//! than an error: the dotted-vs-undotted barang code, and the MySIMKARI ->
//! SIMAN satker code mapping.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

fn auth_headers(
    role: &str,
    satker: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    let user = "00000000-0000-0000-0000-000000000003";
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user).unwrap(),
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
                "Bearer mock::{role}::{user}::{satker}"
            ))
            .unwrap(),
        ),
    ]
}

async fn post(server: &TestServer, path: &str, body: serde_json::Value) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers("validator_pusat", "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

/// POST as the satker's own OPERATOR — the only role that enters barang. The
/// fixture used to add barang as `validator_pusat`, which only worked because
/// the endpoint had no role check at all.
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

async fn get(server: &TestServer, path: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers("validator_pusat", "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

/// One campaign, one participating satker, one barang under it.
///
/// `kode_barang` is passed in the DOTTED presentation form on purpose: the
/// harness seeds `integrasi.siman_aset` with the undotted `3060201003`, and
/// comparing the two raw is an equality that never holds — an empty answer,
/// not an error, which is why it needs a test rather than a smoke check.
async fn seed(server: &TestServer, satker: &str, kode_barang: Option<&str>) -> String {
    let r = post(
        server,
        "/kebutuhan-bmn/pengajuan",
        json!({"nama": "Kebutuhan Meja 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"}),
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
        json!({"satker_id": satker}),
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

    let mut barang = json!({"nama": "Meja Kerja", "jumlah": 5});
    if let Some(kode) = kode_barang {
        barang["kode_barang"] = json!(kode);
    }
    let r = post_as_operator(
        server,
        &format!("/kebutuhan-bmn/satker/{satker_row}/barang"),
        barang,
        satker,
    )
    .await;
    assert_eq!(r.status_code(), 201, "seed barang: {:?}", r.text());

    satker_row
}

fn first_row(res: &axum_test::TestResponse) -> serde_json::Value {
    res.json::<serde_json::Value>()["data"]["barang_list"][0].clone()
}

/// The harness seeds two SIMAN assets under `3060201003` for SKR001. The
/// analysis must find both, and the gap must be computed from that count.
#[tokio::test]
async fn the_analysis_counts_what_the_satker_already_holds() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let satker_row = seed(&server, "SKR001", Some("3.06.02.01.003")).await;

    let res = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{satker_row}/analisis"),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let row = first_row(&res);

    assert_eq!(
        row["existing_count"], 2,
        "SKR001 holds two assets under this barang code: {row:?}"
    );
    assert_eq!(row["gap"], 3, "5 requested - 2 held: {row:?}");
    assert_eq!(
        row["existing_assets"].as_array().unwrap().len(),
        2,
        "the matching assets should be listed, not just counted: {row:?}"
    );
    let summary = res.json::<serde_json::Value>()["data"]["summary"].clone();
    assert_eq!(summary["total_existing"], 2, "{summary:?}");

    teardown_test_db(&db_name).await;
}

/// The same barang code at a satker with no such assets. This is the half that
/// fails if the satker mapping is dropped: SIMAN keys assets by
/// `kdsatker_keu`, and matching on anything else returns another satker's
/// inventory as if it were this one's.
#[tokio::test]
async fn the_analysis_does_not_count_another_satkers_assets() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let satker_row = seed(&server, "SKR002", Some("3.06.02.01.003")).await;

    let res = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{satker_row}/analisis"),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let row = first_row(&res);

    assert_eq!(
        row["existing_count"], 0,
        "SKR002 holds none of these; SKR001's must not be counted: {row:?}"
    );
    assert_eq!(row["gap"], 5, "{row:?}");

    teardown_test_db(&db_name).await;
}

/// The dotted form is what the rest of the system carries; SIMAN stores digits.
/// Both spellings must reach the same assets, or the comparison silently
/// reports nothing held.
#[tokio::test]
async fn a_dotted_and_an_undotted_code_reach_the_same_assets() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let dotted = seed(&server, "SKR001", Some("3.06.02.01.003")).await;
    let undotted = seed(&server, "SKR001", Some("3060201003")).await;

    for satker_row in [dotted, undotted] {
        let res = get(
            &server,
            &format!("/kebutuhan-bmn/satker/{satker_row}/analisis"),
        )
        .await;
        assert_eq!(res.status_code(), 200, "{:?}", res.text());
        assert_eq!(
            first_row(&res)["existing_count"],
            2,
            "both spellings of the barang code must match"
        );
    }

    teardown_test_db(&db_name).await;
}

/// A barang with no code cannot be matched against SIMAN. It must report
/// nothing held rather than matching every asset the satker owns.
#[tokio::test]
async fn a_barang_without_a_code_matches_nothing() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let satker_row = seed(&server, "SKR001", None).await;

    let res = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{satker_row}/analisis"),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let row = first_row(&res);
    assert_eq!(row["existing_count"], 0, "{row:?}");
    assert!(
        row["existing_assets"].as_array().unwrap().is_empty(),
        "{row:?}"
    );

    teardown_test_db(&db_name).await;
}
