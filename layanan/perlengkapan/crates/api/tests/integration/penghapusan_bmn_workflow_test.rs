//! Integration test for complete Penghapusan BMN workflow

use axum_test::TestServer;
use serde_json::json;
use crate::common::{setup_test_app, teardown_test_db};

/// Helper to generate auth headers
fn auth_headers(role: &str, user_id: &str, satker_id: &str) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    vec![
        (reqwest::header::HeaderName::from_static("x-user-id"), reqwest::header::HeaderValue::from_str(user_id).unwrap()),
        (reqwest::header::HeaderName::from_static("x-user-role"), reqwest::header::HeaderValue::from_str(role).unwrap()),
        (reqwest::header::HeaderName::from_static("x-satker-id"), reqwest::header::HeaderValue::from_str(satker_id).unwrap()),
        (reqwest::header::HeaderName::from_static("authorization"), reqwest::header::HeaderValue::from_str(&format!("Bearer mock::{}::{}::{}", role, user_id, satker_id)).unwrap()),
    ]
}

#[tokio::test]
async fn test_complete_penghapusan_bmn_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let validator_pusat_id = "00000000-0000-0000-0000-000000000003";
    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";
    let satker_id = "SKR001";
    let wilayah_id = "WIL001";

    // 1. Create usulan penghapusan
    let mut req = server.post("/penghapusan-bmn")
        .json(&json!({
            "satker_id": "00000000-0000-0000-0000-000000000001",
            "asset_id": "00000000-0000-0000-0000-000000000001",
            "kode_barang": "3.06.02.01.003",
            "nama_barang": "Laptop Dell Latitude 5520",
            "nup": "015",
            "tanggal_penghapusan": "2026-01-01",
            "alasan": "Rusak berat dan sudah usang, tidak ekonomis untuk diperbaiki",
            "metode_penghapusan": "Pemusnahan",
            "nilai_residu": 0,
            "lampiran_persyaratan": "https://storage.example.com/lampiran-persyaratan.pdf"
        }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) { req = req.add_header(k, v); }
    let res = req.await;
    let body = res.json::<serde_json::Value>();
    println!("CREATE PENGHAPUSAN RESPONSE (status={}): {:?}", res.status_code(), body);
    assert_eq!(res.status_code(), 201);
    let usulan_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. Submit to Wilayah
    let mut req = server.post(&format!("/penghapusan-bmn/{}/submit-wilayah", usulan_id))
        .json(&json!({"catatan": "Mohon verifikasi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) { req = req.add_header(k, v); }
    let res = req.await;
    println!("SUBMIT WILAYAH RESPONSE (status={}): {:?}", res.status_code(), res.json::<serde_json::Value>());
    assert_eq!(res.status_code(), 200);

    // 3. Wilayah forwards to Pusat
    let mut req = server.post(&format!("/penghapusan-bmn/{}/forward-pusat", usulan_id))
        .json(&json!({"catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) { req = req.add_header(k, v); }
    let res = req.await;
    println!("FORWARD PUSAT RESPONSE (status={}): {:?}", res.status_code(), res.json::<serde_json::Value>());
    assert_eq!(res.status_code(), 200);

    // 4. Generate Konsep SK
    let mut req = server.post(&format!("/penghapusan-bmn/{}/generate-konsep-sk", usulan_id));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") { req = req.add_header(k, v); }
    let res = req.await;
    println!("GENERATE SK RESPONSE (status={}): {:?}", res.status_code(), res.json::<serde_json::Value>());
    assert_eq!(res.status_code(), 200);

    // 5. Upload Signed SK
    let mut req = server.post(&format!("/penghapusan-bmn/{}/upload-signed-sk", usulan_id))
        .json(&json!({"signed_sk_pdf_url": "https://storage.example.com/sk-signed.pdf"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") { req = req.add_header(k, v); }
    let res = req.await;
    println!("UPLOAD SK RESPONSE (status={}): {:?}", res.status_code(), res.json::<serde_json::Value>());
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_penghapusan_bmn_rejection_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let validator_pusat_id = "00000000-0000-0000-0000-000000000003";
    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";
    let satker_id = "SKR001";
    let wilayah_id = "WIL001";

    // Create
    let mut req = server.post("/penghapusan-bmn")
        .json(&json!({
            "satker_id": "00000000-0000-0000-0000-000000000001",
            "asset_id": "00000000-0000-0000-0000-000000000002",
            "kode_barang": "3.06.02.01.004",
            "nama_barang": "Printer HP LaserJet",
            "nup": "016",
            "tanggal_penghapusan": "2026-01-01",
            "alasan": "Rusak berat dan tidak dapat diperbaiki lagi",
            "metode_penghapusan": "Pemusnahan",
            "nilai_residu": 0,
            "lampiran_persyaratan": "https://storage.example.com/lampiran.pdf"
        }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) { req = req.add_header(k, v); }
    let res = req.await;
    let usulan_id = res.json::<serde_json::Value>()["data"]["id"].as_str().unwrap().to_string();

    // Submit to wilayah
    let mut req = server.post(&format!("/penghapusan-bmn/{}/submit-wilayah", usulan_id))
        .json(&json!({"catatan": "Mohon verifikasi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) { req = req.add_header(k, v); }
    req.await;

    // Forward to pusat
    let mut req = server.post(&format!("/penghapusan-bmn/{}/forward-pusat", usulan_id))
        .json(&json!({"catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) { req = req.add_header(k, v); }
    req.await;

    // Reject via transition (target_status is i32: 4008 = Rejected)
    let mut req = server.post(&format!("/penghapusan-bmn/{}/transition", usulan_id))
        .json(&json!({"target_status": 4008, "catatan": "Tidak disetujui karena aset masih layak pakai"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") { req = req.add_header(k, v); }
    let res = req.await;
    println!("REJECT RESPONSE (status={}): {:?}", res.status_code(), res.json::<serde_json::Value>());
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}
