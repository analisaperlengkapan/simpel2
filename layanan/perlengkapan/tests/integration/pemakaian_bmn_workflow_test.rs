//! Integration test for complete Pemakaian BMN workflow.
//!
//! Exercises the REAL internal-satker lifecycle against a live Postgres:
//!   operator: create (DRAFT) → submit (SUBMITTED)
//!   validator_satker: forward (SUBMITTED → SUBMITTED_APPROVER_SATKER)
//!   approver_satker: approve (→ APPROVED → auto ACTIVE)
//!   then: renew (operator) / revoke (approver_satker)
//! with per-step RBAC roles enforced server-side.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

/// Helper to generate auth headers for a given role/user/satker.
fn auth_headers(
    role: &str,
    user_id: &str,
    satker_id: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
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
const VALIDATOR_SATKER: &str = "00000000-0000-0000-0000-000000000002";
const APPROVER_SATKER: &str = "00000000-0000-0000-0000-000000000003";
const SATKER: &str = "SKR001";

fn create_permit_body(nup: &str) -> serde_json::Value {
    json!({
        "pegawai_nip": "19800101000000001",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": [{
            "jenis_bmn": "LAPTOP",
            "bmn_nup": nup,
            "bmn_kode_barang": "3060201003",
            "bmn_nama_barang": "Laptop Dell Latitude 5520",
            "bmn_merk": "Dell",
            "serial_number": "SN-12345678"
        }]
    })
}

/// Drive a fresh permit through create → submit → forward → approve and
/// return its id. On return the permit is ACTIVE.
async fn drive_to_active(server: &TestServer, nup: &str) -> String {
    // 1. operator creates → DRAFT (version 1)
    let mut req = server.post("/pemakaian-bmn").json(&create_permit_body(nup));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "create: {:?}", res.text());
    let permit_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. operator submits DRAFT → SUBMITTED (generic transition)
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/transition", permit_id))
        .json(&json!({"target_status": "SUBMITTED", "catatan": "Mohon validasi"}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "submit: {:?}", res.text());

    // 3. validator_satker forwards SUBMITTED → SUBMITTED_APPROVER_SATKER
    //    (version still 1 at this point — submit does not bump version).
    let mut req = server
        .post(&format!(
            "/pemakaian-bmn/{}/validator-satker-action",
            permit_id
        ))
        .json(&json!({"action": "forward", "expected_version": 1, "catatan": "Diteruskan"}));
    for (k, v) in auth_headers("validator_satker", VALIDATOR_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        200,
        "validator forward: {:?}",
        res.text()
    );

    // 4. approver_satker approves (→ APPROVED → auto ACTIVE). Forward bumped
    //    version to 2.
    let mut req = server
        .post(&format!(
            "/pemakaian-bmn/{}/approver-satker-action",
            permit_id
        ))
        .json(&json!({"action": "approve", "expected_version": 2, "catatan": "Disetujui"}));
    for (k, v) in auth_headers("approver_satker", APPROVER_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "approver approve: {:?}", res.text());

    permit_id
}

#[tokio::test]
async fn test_complete_pemakaian_bmn_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let permit_id = drive_to_active(&server, "15").await;

    // Permit should now be ACTIVE.
    let mut req = server.get(&format!("/pemakaian-bmn/{}", permit_id));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let body = res.json::<serde_json::Value>();
    println!("FINAL PERMIT (status={}): {:?}", res.status_code(), body);
    assert_eq!(res.status_code(), 200);
    assert_eq!(body["data"]["status"].as_str(), Some("ACTIVE"));
    assert!(
        body["data"]["nomor_izin"].as_str().is_some(),
        "active permit must have a nomor_izin"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_permit_renewal_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let permit_id = drive_to_active(&server, "16").await;

    // Renew the active permit (operator).
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/renew", permit_id))
        .json(&json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan pemakaian untuk tahun berikutnya"
        }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "RENEW RESPONSE (status={}): {:?}",
        res.status_code(),
        res.text()
    );
    assert_eq!(res.status_code(), 201);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_permit_revocation_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let permit_id = drive_to_active(&server, "017").await;

    // Revoke the active permit — only Approver Satker is allowed.
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/revoke", permit_id))
        .json(&json!({"alasan": "BMN rusak berat dan tidak layak pakai lagi"}));
    for (k, v) in auth_headers("approver_satker", APPROVER_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "REVOKE RESPONSE (status={}): {:?}",
        res.status_code(),
        res.text()
    );
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}

/// Two tabs on a permit sent back for revision: one cancels, one re-submits.
///
/// The engine commits the new status text before the service writes the matching `status_kode`, so
/// for a moment the text says CANCELLED while the code still says REVISI_OPERATOR. The re-submit
/// checked only the version and the code, so it slipped into that moment, turned the cancelled
/// permit back into SUBMITTED, and then the cancel's own write stamped its code over it:
/// SUBMITTED with a CANCELLED code, which no validator action (they gate on the code) could ever
/// accept. The re-submit must now see the text too.
#[tokio::test]
async fn a_resubmit_cannot_revive_a_permit_the_engine_just_cancelled() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // DRAFT -> SUBMITTED -> (validator returns it) REVISI_OPERATOR.
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_permit_body("NUP-RACE-1"));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "create: {:?}", res.text());
    let id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/transition"))
        .json(&json!({"target_status": "SUBMITTED"}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    assert_eq!(req.await.status_code(), 200);

    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/validator-satker-action"))
        .json(&json!({
            "action": "return",
            "expected_version": 1,
            "catatan": "Lengkapi nomor seri dan lokasi pemakaian."
        }));
    for (k, v) in auth_headers("validator_satker", VALIDATOR_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "return: {:?}", res.text());
    let version = res.json::<serde_json::Value>()["data"]["version"]
        .as_i64()
        .unwrap();

    // The moment between the engine's commit and the status_kode write.
    let client = db.pool().get().await.unwrap();
    let n = client
        .execute(
            "UPDATE perlengkapan.izin_pemakaian_bmn SET status = 'CANCELLED' WHERE id = $1::text::uuid",
            &[&id],
        )
        .await
        .unwrap();
    assert_eq!(n, 1);

    // Through the API the policy reads the status text first and already refuses: nothing revives.
    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/resubmit"))
        .json(&json!({"expected_version": version}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert!(
        res.status_code() == 403 || res.status_code() == 409,
        "a re-submit must not revive what was just cancelled: {} {:?}",
        res.status_code(),
        res.text()
    );

    // The window the review described is the one the API cannot show from outside: the policy
    // read happened BEFORE the cancel committed, the write after. That write is the repository's.
    let repo = layanan_perlengkapan::pemakaian_bmn::PemakaianBmnRepository::new(db.pool().clone());
    let err = repo
        .operator_resubmit(
            uuid::Uuid::parse_str(&id).unwrap(),
            uuid::Uuid::parse_str(OPERATOR).unwrap(),
            "Operator",
            version as i32,
        )
        .await
        .expect_err("the write must see the status text, not only the code");
    assert!(
        matches!(
            err,
            layanan_perlengkapan::shared::error::AppError::Conflict(_)
        ),
        "expected a Conflict, got {err:?}"
    );

    let row = client
        .query_one(
            "SELECT status FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1::text::uuid",
            &[&id],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>(0), "CANCELLED");

    teardown_test_db(&db_name).await;
}
