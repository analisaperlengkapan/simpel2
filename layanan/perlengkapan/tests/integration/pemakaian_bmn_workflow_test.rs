//! Integration test for complete Pemakaian BMN workflow

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

/// Helper to generate auth headers
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

fn create_permit_body(nup: &str) -> serde_json::Value {
    json!({
        "pegawai_nip": "198501012010011001",
        "pegawai_nama": "Budi Santoso",
        "pegawai_satker_id": "00000000-0000-0000-0000-000000000001",
        "pegawai_satker_nama": "Kejaksaan Negeri Jakarta Selatan",
        "jenis_bmn": "LAPTOP",
        "bmn_nup": nup,
        "bmn_kode_barang": "3.06.02.01.003",
        "bmn_nama_barang": "Laptop Dell Latitude 5520",
        "bmn_merk": "Dell",
        "serial_number": "SN-12345678",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari"
    })
}

#[tokio::test]
async fn test_complete_pemakaian_bmn_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let satker_id = "SKR001";

    // 1. Create permit
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_permit_body("015"));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let body = res.json::<serde_json::Value>();
    println!(
        "CREATE PERMIT RESPONSE (status={}): {:?}",
        res.status_code(),
        body
    );
    assert_eq!(res.status_code(), 201);
    let permit_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. Generate konsep surat
    let mut req = server.post(&format!(
        "/pemakaian-bmn/{}/generate-konsep-surat",
        permit_id
    ));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "GENERATE KONSEP RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 3. Upload signed document
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/upload-signed-pdf", permit_id))
        .json(&json!({"signed_pdf_url": "https://storage.example.com/signed.pdf"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "UPLOAD SIGNED RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 4. Activate permit
    let mut req = server.post(&format!("/pemakaian-bmn/{}/activate", permit_id));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "ACTIVATE RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_permit_renewal_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let satker_id = "SKR001";

    // Create + activate original permit
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_permit_body("016"));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let permit_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server.post(&format!(
        "/pemakaian-bmn/{}/generate-konsep-surat",
        permit_id
    ));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/upload-signed-pdf", permit_id))
        .json(&json!({"signed_pdf_url": "https://storage.example.com/signed.pdf"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server.post(&format!("/pemakaian-bmn/{}/activate", permit_id));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    // Renew permit
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/renew", permit_id))
        .json(&json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan pemakaian untuk tahun berikutnya"
        }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "RENEW RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 201);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_permit_revocation_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let satker_id = "SKR001";

    // Create + activate
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_permit_body("017"));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let permit_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server.post(&format!(
        "/pemakaian-bmn/{}/generate-konsep-surat",
        permit_id
    ));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/upload-signed-pdf", permit_id))
        .json(&json!({"signed_pdf_url": "https://storage.example.com/signed.pdf"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server.post(&format!("/pemakaian-bmn/{}/activate", permit_id));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    // Revoke
    let mut req = server
        .post(&format!("/pemakaian-bmn/{}/revoke", permit_id))
        .json(&json!({"alasan": "BMN rusak berat dan tidak layak pakai lagi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "REVOKE RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}
