//! Layer-2 RBAC-403 integration tests (#33 / F5-C).
//!
//! Asserts that role-gated endpoints are enforced **server-side**: a caller with
//! the wrong role gets 403 FORBIDDEN (not 200, and not merely a hidden FE
//! button), and a caller with no token gets 401. Role is carried in the mock
//! bearer token `mock::<role>::<user_id>::<satker>` decoded by
//! `AuthencClient::dummy()`.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker_id: &str) -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user_id).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_str(satker_id).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::{}::{}",
                role, user_id, satker_id
            ))
            .unwrap(),
        ),
    ]
}

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const VALIDATOR_WILAYAH: &str = "00000000-0000-0000-0000-000000000002";
const VALIDATOR_PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const SATKER: &str = "SKR001";

/// POST helper that attaches headers + an optional JSON body.
async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    body: Option<serde_json::Value>,
) -> axum_test::TestResponse {
    let mut req = match body {
        Some(b) => server.post(path).json(&b),
        None => server.post(path),
    };
    for (k, v) in auth_headers(role, user, SATKER) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn create_penghapusan(server: &TestServer) -> String {
    let res = post(
        server,
        "/penghapusan-bmn",
        "operator_satker",
        OPERATOR,
        Some(json!({
            "satker_id": "00000000-0000-0000-0000-000000000001",
            "asset_id": "00000000-0000-0000-0000-000000000001",
            "kode_barang": "3060201003",
            "nama_barang": "Laptop Dell",
            "nup": "99",
            "tanggal_penghapusan": "2026-01-01",
            "alasan": "Rusak berat dan sudah usang, tidak ekonomis diperbaiki",
            "metode_penghapusan": "Pemusnahan",
            "nilai_perolehan": 0,
            "lampiran_persyaratan": "https://storage.example.com/l.pdf"
        })),
    )
    .await;
    assert_eq!(res.status_code(), 201, "setup create: {:?}", res.text());
    res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_penghapusan_rbac_403() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_penghapusan(&server).await;

    // submit-wilayah is operator-only → validator_pusat denied.
    let r = post(
        &server,
        &format!("/penghapusan-bmn/{}/submit-wilayah", id),
        "validator_pusat",
        VALIDATOR_PUSAT,
        Some(json!({})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "submit-wilayah as pusat: {:?}",
        r.text()
    );

    // forward-pusat is validator_wilayah-only → operator denied.
    let r = post(
        &server,
        &format!("/penghapusan-bmn/{}/forward-pusat", id),
        "operator_satker",
        OPERATOR,
        Some(json!({"aksi": "forward"})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "forward-pusat as operator: {:?}",
        r.text()
    );

    // verifikasi-pusat is validator_pusat-only → operator denied.
    let r = post(
        &server,
        &format!("/penghapusan-bmn/{}/verifikasi-pusat", id),
        "operator_satker",
        OPERATOR,
        Some(json!({})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "verifikasi-pusat as operator: {:?}",
        r.text()
    );

    // generate-konsep-sk is validator-only → operator denied.
    let r = post(
        &server,
        &format!("/penghapusan-bmn/{}/generate-konsep-sk", id),
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "generate-sk as operator: {:?}",
        r.text()
    );

    // delete is operator-only → validator_wilayah denied.
    let mut req = server.delete(&format!("/penghapusan-bmn/{}", id));
    for (k, v) in auth_headers("validator_wilayah", VALIDATOR_WILAYAH, SATKER) {
        req = req.add_header(k, v);
    }
    let r = req.await;
    assert_eq!(r.status_code(), 403, "delete as wilayah: {:?}", r.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_pemakaian_rbac_403() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Create a permit (DRAFT) so the policy-gated handlers reach their RBAC
    // check (they load the entity before authorizing).
    let res = post(
        &server,
        "/pemakaian-bmn",
        "operator_satker",
        OPERATOR,
        Some(json!({
            "pegawai_nip": "198501012010011001",
            "pegawai_nama": "Budi",
            "pegawai_satker_id": "00000000-0000-0000-0000-000000000001",
            "pegawai_satker_nama": "Kejari Jaksel",
            "jenis_bmn": "LAPTOP",
            "bmn_nup": "98",
            "bmn_kode_barang": "3060201003",
            "bmn_nama_barang": "Laptop Dell",
            "bmn_merk": "Dell",
            "serial_number": "SN-1",
            "tanggal_mulai": "2026-01-01",
            "tanggal_selesai": "2026-12-31",
            "keperluan": "Tugas kedinasan sehari-hari operator"
        })),
    )
    .await;
    assert_eq!(res.status_code(), 201, "setup create: {:?}", res.text());
    let id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // validator-satker-action is validator_satker-only → operator denied.
    let r = post(
        &server,
        &format!("/pemakaian-bmn/{}/validator-satker-action", id),
        "operator_satker",
        OPERATOR,
        Some(json!({"action": "forward", "expected_version": 1})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "validator-action as operator: {:?}",
        r.text()
    );

    // approver-satker-action is approver_satker-only → operator denied.
    let r = post(
        &server,
        &format!("/pemakaian-bmn/{}/approver-satker-action", id),
        "operator_satker",
        OPERATOR,
        Some(json!({"action": "approve", "expected_version": 1})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "approver-action as operator: {:?}",
        r.text()
    );

    // revoke is approver_satker-only → operator denied …
    let r = post(
        &server,
        &format!("/pemakaian-bmn/{}/revoke", id),
        "operator_satker",
        OPERATOR,
        Some(json!({"alasan": "x"})),
    )
    .await;
    assert_eq!(r.status_code(), 403, "revoke as operator: {:?}", r.text());

    // … and admin is explicitly blocked from revoke (stakeholder mandate).
    let r = post(
        &server,
        &format!("/pemakaian-bmn/{}/revoke", id),
        "admin",
        OPERATOR,
        Some(json!({"alasan": "x"})),
    )
    .await;
    assert_eq!(r.status_code(), 403, "revoke as admin: {:?}", r.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_kebutuhan_rbac_403() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // These handlers enforce require_role before any DB lookup, so a random
    // satker id still yields a clean role denial.
    let sid = uuid::Uuid::new_v4();

    // submit-wilayah is operator-only → validator_pusat denied.
    let r = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/submit-wilayah", sid),
        "validator_pusat",
        VALIDATOR_PUSAT,
        Some(json!({"lampiran_surat_permohonan": "https://x/l.pdf", "lampiran_pendukung": []})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "kebutuhan submit as pusat: {:?}",
        r.text()
    );

    // validator-wilayah is validator_wilayah-only → operator denied.
    let r = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/validator-wilayah", sid),
        "operator_satker",
        OPERATOR,
        Some(json!({"aksi": "forward"})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "kebutuhan validator-wilayah as operator: {:?}",
        r.text()
    );

    // keputusan-pusat is validator_pusat-only → operator denied.
    let r = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/keputusan-pusat", sid),
        "operator_satker",
        OPERATOR,
        Some(json!({"is_approved": true})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "kebutuhan keputusan as operator: {:?}",
        r.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_missing_token_401() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // No Authorization header at all → 401 (auth enforced before RBAC).
    let r = server
        .post("/penghapusan-bmn/00000000-0000-0000-0000-000000000001/verifikasi-pusat")
        .json(&json!({}))
        .await;
    assert_eq!(r.status_code(), 401, "missing token: {:?}", r.text());

    teardown_test_db(&db_name).await;
}
