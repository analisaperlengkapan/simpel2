//! Integration test for complete Penghapusan BMN workflow

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

#[tokio::test]
async fn test_complete_penghapusan_bmn_workflow() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let validator_pusat_id = "00000000-0000-0000-0000-000000000003";
    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";
    let satker_id = "SKR001";
    // A validator_wilayah's claim carries their OWN satker code — the Kejati's —
    // and the wilayah tier resolves the region from it through
    // `integrasi.v_satker_wilayah`. "WIL001" is not a satker code at all; it named
    // nothing in the fixture and nothing in MySIMKARI. That went unnoticed while no
    // by-id handler read the claim, and turned into a 404 the moment one did:
    // an unresolvable wilayah correctly matches no rows. KJT01 is the Kejati the
    // fixture puts above SKR001.
    let wilayah_id = "KJT01";

    // 1. Create usulan penghapusan
    let mut req = server.post("/penghapusan-bmn").json(&json!({
        "satker_id": "00000000-0000-0000-0000-000000000001",
        "asset_id": "00000000-0000-0000-0000-000000000001",
        "kode_barang": "3060201003",
        "nama_barang": "Laptop Dell Latitude 5520",
        "nup": "15",
        "tanggal_penghapusan": "2026-01-01",
        "alasan": "Rusak berat dan sudah usang, tidak ekonomis untuk diperbaiki",
        "metode_penghapusan": "Pemusnahan",
        "nilai_perolehan": 0,
        "lampiran_persyaratan": "https://storage.example.com/lampiran-persyaratan.pdf"
    }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let body = res.json::<serde_json::Value>();
    println!(
        "CREATE PENGHAPUSAN RESPONSE (status={}): {:?}",
        res.status_code(),
        body
    );
    assert_eq!(res.status_code(), 201);
    let usulan_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. Submit to Wilayah
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/submit-wilayah", usulan_id))
        .json(&json!({"catatan": "Mohon verifikasi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "SUBMIT WILAYAH RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 3. Wilayah forwards to Pusat
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/forward-pusat", usulan_id))
        .json(&json!({"aksi": "forward", "catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "FORWARD PUSAT RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 4. Validator Pusat verifies the asset (SubmitPusat → VerifikasiPusat)
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/verifikasi-pusat", usulan_id))
        .json(&json!({"catatan": "Aset terverifikasi, lanjut terbitkan SK"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "VERIFIKASI PUSAT RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 5. Generate Konsep SK
    let mut req = server.post(&format!(
        "/penghapusan-bmn/{}/generate-konsep-sk",
        usulan_id
    ));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "GENERATE SK RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // 6. Upload Signed SK
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/upload-signed-sk", usulan_id))
        .json(&json!({"signed_sk_pdf_url": "https://storage.example.com/sk-signed.pdf"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "UPLOAD SK RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
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
    // KJT01 = the Kejati above SKR001 (see the note in the test above).
    let wilayah_id = "KJT01";

    // Create
    let mut req = server.post("/penghapusan-bmn").json(&json!({
        "satker_id": "00000000-0000-0000-0000-000000000001",
        "asset_id": "00000000-0000-0000-0000-000000000002",
        "kode_barang": "3060201004",
        "nama_barang": "Printer HP LaserJet",
        "nup": "16",
        "tanggal_penghapusan": "2026-01-01",
        "alasan": "Rusak berat dan tidak dapat diperbaiki lagi",
        "metode_penghapusan": "Pemusnahan",
        "nilai_perolehan": 0,
        "lampiran_persyaratan": "https://storage.example.com/lampiran.pdf"
    }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let usulan_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Submit to wilayah
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/submit-wilayah", usulan_id))
        .json(&json!({"catatan": "Mohon verifikasi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, satker_id) {
        req = req.add_header(k, v);
    }
    req.await;

    // Forward to pusat
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/forward-pusat", usulan_id))
        .json(&json!({"aksi": "forward", "catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, wilayah_id) {
        req = req.add_header(k, v);
    }
    req.await;

    // Verifikasi Pusat (SubmitPusat → VerifikasiPusat) — reject is only
    // reachable from VerifikasiPusat per the workflow state machine.
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/verifikasi-pusat", usulan_id))
        .json(&json!({"catatan": "Ditelaah"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "VERIFIKASI PUSAT RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    // Reject via generic transition (to_state is the state NAME: "REJECTED")
    let mut req = server.post(&format!("/penghapusan-bmn/{}/transition", usulan_id))
        .json(&json!({"to_state": "REJECTED", "catatan": "Tidak disetujui karena aset masih layak pakai"}));
    for (k, v) in auth_headers("validator_pusat", validator_pusat_id, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    println!(
        "REJECT RESPONSE (status={}): {:?}",
        res.status_code(),
        res.json::<serde_json::Value>()
    );
    assert_eq!(res.status_code(), 200);

    teardown_test_db(&db_name).await;
}

/// The wilayah tier, asserted rather than assumed.
///
/// The two workflow tests above only prove the *positive* half: KJT01's
/// validator can forward SKR001's usulan. That half also passes when the scope
/// is missing entirely, which is exactly how the gap survived — so it proves
/// nothing on its own. This is the half that fails if the scope is dropped:
/// KJT02 supervises SKR003, not SKR001, and its validator must not be able to
/// stamp its own identity onto another region's usulan.
///
/// 404, not 403: a 403 would confirm the id exists under some other Kejati,
/// which is the cross-tenant existence oracle #93 closed.
#[tokio::test]
async fn a_wilayah_validator_from_another_kejati_cannot_forward_it() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let operator_satker_id = "00000000-0000-0000-0000-000000000001";
    let validator_wilayah_id = "00000000-0000-0000-0000-000000000002";

    let mut req = server.post("/penghapusan-bmn").json(&json!({
        "satker_id": "00000000-0000-0000-0000-000000000001",
        "asset_id": "00000000-0000-0000-0000-000000000003",
        // The third asset the fixture seeds into `integrasi.siman_aset`; create
        // validates kode_barang + NUP against it, so an invented pair 400s
        // before the scope is ever reached.
        "kode_barang": "3060201003",
        "nama_barang": "Meja Kerja",
        "nup": "99",
        "tanggal_penghapusan": "2026-01-01",
        "alasan": "Rusak berat, biaya perbaikan melebihi nilai buku",
        "metode_penghapusan": "Pemusnahan",
        "nilai_perolehan": 0,
        "lampiran_persyaratan": "https://storage.example.com/lampiran.pdf"
    }));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, "SKR001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201);
    let usulan_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/submit-wilayah", usulan_id))
        .json(&json!({"catatan": "Mohon verifikasi"}));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, "SKR001") {
        req = req.add_header(k, v);
    }
    assert_eq!(req.await.status_code(), 200);

    // KJT02's validator: right role, wrong region.
    let mut req = server
        .post(&format!("/penghapusan-bmn/{}/forward-pusat", usulan_id))
        .json(&json!({"aksi": "forward", "catatan": "Diteruskan ke Pusat"}));
    for (k, v) in auth_headers("validator_wilayah", validator_wilayah_id, "KJT02") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        404,
        "a validator from another Kejati must not reach this usulan: {:?}",
        res.json::<serde_json::Value>()
    );

    // And the refusal must be a refusal, not a partial write: the validator
    // identity stamp happens BEFORE the transition in this handler, so a scope
    // check placed only at the transition would still have branded the record.
    let mut req = server.get(&format!("/penghapusan-bmn/{}", usulan_id));
    for (k, v) in auth_headers("operator_satker", operator_satker_id, "SKR001") {
        req = req.add_header(k, v);
    }
    let body = req.await.json::<serde_json::Value>();
    assert_eq!(body["data"]["status"], "SUBMIT_WILAYAH");
    assert!(
        body["data"]["validator_wilayah_id"].is_null(),
        "out-of-region forward must not stamp a validator: {:?}",
        body["data"]
    );

    teardown_test_db(&db_name).await;
}
