//! Object-level scoping for the pakaian-dinas employee surfaces.
//!
//! These endpoints carry personal data — NIP, name, phone, gender, rank, and
//! uniform measurements — keyed by a satker the CALLER names in the path or
//! implies in the body. Until this suite existed, every one of them took an
//! unused `_claims` and answered for any satker, which was measured against
//! deployed staging: one satker's operator read another's roster (200 on all
//! three reads) and rewrote another's employee profile (200, row changed).
//!
//! Every test below therefore has both halves. The positive half alone passes
//! just as well with no scope at all, which is exactly how the gap survived.

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

const ROSTER_PATHS: [&str; 3] = [
    "/pakaian-dinas/pegawai-satker/{}",
    "/pakaian-dinas/pegawai-satker/{}/with-sizes",
    "/pakaian-dinas/pegawai-satker/{}/roster",
];

#[tokio::test]
async fn an_operator_reads_its_own_roster() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for tpl in ROSTER_PATHS {
        let path = tpl.replace("{}", "SKR001");
        let res = get(&server, &path, "operator_satker", "SKR001").await;
        assert_eq!(
            res.status_code(),
            200,
            "own roster must stay readable: {path}"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_operator_cannot_read_another_satkers_roster() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for tpl in ROSTER_PATHS {
        let path = tpl.replace("{}", "SKR002");
        let res = get(&server, &path, "operator_satker", "SKR001").await;
        // 404, not 403: 403 confirms the satker exists and has staff, which is
        // the cross-tenant existence oracle #93 closed.
        assert_eq!(
            res.status_code(),
            404,
            "cross-satker roster must not be readable: {path} -> {:?}",
            res.json::<serde_json::Value>()
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_wilayah_validator_reads_its_region_but_not_the_next_one() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // KJT01 supervises SKR001 and SKR002; SKR003 belongs to KJT02.
    for satker in ["SKR001", "SKR002"] {
        let path = format!("/pakaian-dinas/pegawai-satker/{satker}");
        let res = get(&server, &path, "validator_wilayah", "KJT01").await;
        assert_eq!(res.status_code(), 200, "{satker} is inside KJT01");
    }

    let res = get(
        &server,
        "/pakaian-dinas/pegawai-satker/SKR003",
        "validator_wilayah",
        "KJT01",
    )
    .await;
    assert_eq!(res.status_code(), 404, "SKR003 belongs to KJT02");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_cross_satker_role_reads_every_roster() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for satker in ["SKR001", "SKR002", "SKR003"] {
        let path = format!("/pakaian-dinas/pegawai-satker/{satker}");
        let res = get(&server, &path, "validator_pusat", "PUSAT001").await;
        assert_eq!(res.status_code(), 200, "pusat sees {satker}");
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_caller_without_a_satker_identity_reads_nothing() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Authenticated, satker-bound role, no satker code: fail closed.
    let mut req = server.get("/pakaian-dinas/pegawai-satker/SKR001");
    req = req.add_header(
        reqwest::header::HeaderName::from_static("x-user-id"),
        reqwest::header::HeaderValue::from_static("00000000-0000-0000-0000-000000000001"),
    );
    req = req.add_header(
        reqwest::header::HeaderName::from_static("x-user-role"),
        reqwest::header::HeaderValue::from_static("operator_satker"),
    );
    req = req.add_header(
        reqwest::header::HeaderName::from_static("authorization"),
        reqwest::header::HeaderValue::from_static(
            "Bearer mock::operator_satker::00000000-0000-0000-0000-000000000001::",
        ),
    );
    let res = req.await;
    assert_eq!(res.status_code(), 404);

    teardown_test_db(&db_name).await;
}

// ── Writes ────────────────────────────────────────────────────────────────

async fn upsert(
    server: &TestServer,
    role: &str,
    satker: &str,
    body: serde_json::Value,
) -> axum_test::TestResponse {
    let mut req = server.post("/pakaian-dinas/pegawai-profile").json(&body);
    for (k, v) in auth_headers(role, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

#[tokio::test]
async fn an_operator_writes_a_profile_for_its_own_employee() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = upsert(
        &server,
        "operator_satker",
        "SKR001",
        json!({"nip": "19800101000000001", "ukuran_baju": "L", "with_hijab": false}),
    )
    .await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["data"]["ukuran_baju"], "L");
    // Written from the SoT, not from the request — the request did not even
    // carry a satker.
    assert_eq!(body["data"]["kode_satker"], "SKR001");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_operator_cannot_write_another_satkers_employee() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // The exact shape measured on staging: name the other satker's employee,
    // and claim their satker in the body.
    let res = upsert(
        &server,
        "operator_satker",
        "SKR001",
        json!({
            "nip": "19800101000000002",
            "nama": "DITULIS OLEH OPERATOR SATKER LAIN",
            "kode_satker": "SKR002",
            "ukuran_baju": "XXL",
            "with_hijab": false
        }),
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "cross-satker profile write must be refused: {:?}",
        res.json::<serde_json::Value>()
    );

    // And refused means nothing was written: the roster read for SKR002 (as a
    // caller who MAY see it) must still show no sizes.
    let res = get(
        &server,
        "/pakaian-dinas/pegawai-satker/SKR002/with-sizes",
        "validator_pusat",
        "PUSAT001",
    )
    .await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    assert!(
        body["data"][0]["existing_sizes"].is_null(),
        "refused write must leave no trace: {:?}",
        body["data"]
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_body_supplied_satker_cannot_relabel_an_employee() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // In scope for the employee, but lying about which satker they belong to.
    // The SoT wins; the claim is ignored rather than obeyed.
    let res = upsert(
        &server,
        "operator_satker",
        "SKR001",
        json!({
            "nip": "19800101000000001",
            "kode_satker": "SKR003",
            "ukuran_sepatu": "42",
            "with_hijab": false
        }),
    )
    .await;
    assert_eq!(res.status_code(), 200);
    assert_eq!(
        res.json::<serde_json::Value>()["data"]["kode_satker"],
        "SKR001"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_bulk_write_admits_only_the_rows_the_caller_may_touch() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut req = server
        .post("/pakaian-dinas/pegawai-profile/bulk")
        .json(&json!([
            {"nip": "19800101000000001", "ukuran_baju": "M", "with_hijab": false},
            {"nip": "19800101000000002", "ukuran_baju": "M", "with_hijab": false},
            {"nip": "19800101000000003", "ukuran_baju": "M", "with_hijab": false}
        ]));
    for (k, v) in auth_headers("operator_satker", "SKR001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200);
    // One of the three is theirs. A batch check that ran once over the whole
    // list would have written all three or none.
    assert_eq!(res.json::<serde_json::Value>()["data"], 1);

    let res = get(
        &server,
        "/pakaian-dinas/pegawai-satker/SKR002/with-sizes",
        "validator_pusat",
        "PUSAT001",
    )
    .await;
    assert!(
        res.json::<serde_json::Value>()["data"][0]["existing_sizes"].is_null(),
        "the out-of-scope rows of a batch must not be written"
    );

    teardown_test_db(&db_name).await;
}
