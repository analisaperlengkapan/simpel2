//! Integration test for complete Kebutuhan BMN workflow

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

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

#[tokio::test]
async fn test_complete_kebutuhan_bmn_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let validator_pusat_id = "00000000-0000-0000-0000-000000000003";
    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";
    let satker_id = "SKR001";
    let wilayah_id = "WIL001";

    // 1. Validator Pusat creates pengajuan
    let mut req = server.post("/kebutuhan-bmn/pengajuan").json(&json!({
        "nama": "Kebutuhan BMN 2026",
        "tahun": 2026,
        "tgl_mulai": "2026-01-01",
        "tgl_selesai": "2026-12-31"
    }));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let body: serde_json::Value = res.json();
    println!("CREATE PENGAJUAN RESPONSE: {:?}", body);
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .unwrap_or(body["data"]["id"].as_str().unwrap_or(""))
        .to_string();
    assert_eq!(res.status_code(), 201);

    // 2. Add Satker to Pengajuan
    let mut req = server
        .post(&format!("/kebutuhan-bmn/pengajuan/{}/satker", pengajuan_id))
        .json(&json!({
            "satker_id": satker_id,
            "satker_name": "Kejaksaan Negeri Jakarta Selatan"
        }));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let satker_res_json = res.json::<serde_json::Value>();
    println!("ADD SATKER RESPONSE: {:?}", satker_res_json);
    assert_eq!(res.status_code(), 201);

    let pengajuan_satker_id = satker_res_json["data"]["id"].as_str().unwrap().to_string();
    println!("PENGAJUAN SATKER ID: {}", pengajuan_satker_id);

    // 3. Operator Satker submits barang
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/barang",
            pengajuan_satker_id
        ))
        .json(&json!({
            "nama": "Laptop",
            "jumlah": 5,
            "satuan": "Unit"
        }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let barang_res_json = res.json::<serde_json::Value>();
    println!(
        "CREATE BARANG RESPONSE (status={}): {:?}",
        res.status_code(),
        barang_res_json
    );
    assert_eq!(res.status_code(), 201);

    // 4. Submit to Validator Wilayah
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/submit-wilayah",
            pengajuan_satker_id
        ))
        .json(&json!({
            "catatan_satker": "Mohon persetujuan",
            "lampiran_surat_permohonan": "https://storage.example.com/surat-permohonan.pdf"
        }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let submit_body = res.json::<serde_json::Value>();
    println!(
        "SUBMIT WILAYAH RESPONSE (status={}): {:?}",
        res.status_code(),
        submit_body
    );
    assert_eq!(res.status_code(), 200);

    // 5. Validator Wilayah forwards to Pusat
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/validator-wilayah",
            pengajuan_satker_id
        ))
        .json(&json!({
            "aksi": "forward",
            "catatan": "Diteruskan ke Pusat"
        }));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let fwd_body = res.json::<serde_json::Value>();
    println!(
        "FORWARD PUSAT RESPONSE (status={}): {:?}",
        res.status_code(),
        fwd_body
    );
    assert_eq!(res.status_code(), 200);

    // 6. Validator Pusat approves
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/keputusan-pusat",
            pengajuan_satker_id
        ))
        .json(&json!({
            "is_approved": true,
            "alasan": "Sesuai standar"
        }));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let approval_body = res.json::<serde_json::Value>();
    println!(
        "APPROVAL RESPONSE (status={}): {:?}",
        res.status_code(),
        approval_body
    );
    assert_eq!(res.status_code(), 200);
    assert_eq!(approval_body["data"]["status_kode"].as_i64().unwrap(), 2006);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn test_kebutuhan_bmn_rejection_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let validator_pusat_id = "00000000-0000-0000-0000-000000000003";
    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";
    let satker_id = "SKR002";
    let wilayah_id = "WIL001";

    let mut req = server.post("/kebutuhan-bmn/pengajuan")
        .json(&json!({"nama": "Kebutuhan BMN 2026", "tahun": 2026, "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let pengajuan_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server
        .post(&format!("/kebutuhan-bmn/pengajuan/{}/satker", pengajuan_id))
        .json(&json!({"satker_id": satker_id, "satker_name": "Kejaksaan Negeri Jakarta Utara"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201);
    let pengajuan_satker_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server.post(&format!("/kebutuhan-bmn/satker/{}/submit-wilayah", pengajuan_satker_id))
        .json(&json!({"catatan_satker": "Mohon persetujuan", "lampiran_surat_permohonan": "https://storage.example.com/surat.pdf"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/validator-wilayah",
            pengajuan_satker_id
        ))
        .json(&json!({"aksi": "forward", "catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) {
        req = req.add_header(k, v);
    }
    req.await;

    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{}/keputusan-pusat",
            pengajuan_satker_id
        ))
        .json(&json!({"is_approved": false, "alasan": "Tidak sesuai standar"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200);
    assert_eq!(
        res.json::<serde_json::Value>()["data"]["status_kode"]
            .as_i64()
            .unwrap(),
        2007
    );

    teardown_test_db(&db_name).await;
}
