//! Menghapus jenis pakaian dinas yang masih dipakai kampanye harus ditolak dengan
//! alasan, bukan dengan teks galat basis data.
//!
//! Pengajuan menunjuk jenis lewat foreign key tanpa cascade (spesifikasi di
//! bawahnya cascade, kampanye tidak). Penghapusan jenis yang dipakai gagal di
//! basis data, dan pesan yang sampai ke operator adalah "db error" mentah —
//! tidak ada yang bisa dilakukan dengan kalimat itu. Dua arah: yang dipakai
//! DITOLAK dengan sebab dan tetap ada, yang bebas TERHAPUS.

use super::pakaian_dinas_pengisian_test::{SATKER_A, buat_kampanye, get, post};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const ADMIN: &str = "00000000-0000-0000-0000-000000000009";

async fn hapus_jenis(server: &TestServer, id: &str) -> axum_test::TestResponse {
    let mut req = server.delete(&format!("/pakaian-dinas/jenis/{id}"));
    for (k, v) in super::pakaian_dinas_pengisian_test::headers("admin", ADMIN, SATKER_A) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn id_jenis(server: &TestServer, nama: &str) -> String {
    let res = get(
        server,
        "/pakaian-dinas/jenis?page=1&per_page=100",
        "admin",
        ADMIN,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["nama"] == nama)
        .unwrap_or_else(|| panic!("jenis {nama} tidak ada"))["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn jenis_yang_dipakai_kampanye_ditolak_dengan_sebab() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    // Membuat jenis "PDH" dan satu kampanye yang memakainya.
    let _ = buat_kampanye(&server).await;
    let pdh = id_jenis(&server, "PDH").await;

    let res = hapus_jenis(&server, &pdh).await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());
    let body = res.text();
    assert!(
        body.contains("masih dipakai oleh pengajuan"),
        "pesan menyebut sebabnya: {body}"
    );
    assert!(
        !body.contains("db error") && !body.contains("violates"),
        "bukan teks galat basis data: {body}"
    );
    // Tetap ada.
    assert_eq!(id_jenis(&server, "PDH").await, pdh);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn jenis_yang_tidak_dipakai_terhapus() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let res = post(
        &server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        SATKER_A,
        json!({"nama": "Jenis Sementara", "deskripsi": "tanpa kampanye", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = hapus_jenis(&server, &id).await;
    assert!(res.status_code().is_success(), "{:?}", res.text());
    let res = get(
        &server,
        "/pakaian-dinas/jenis?page=1&per_page=100",
        "admin",
        ADMIN,
        SATKER_A,
    )
    .await;
    assert!(
        !res.text().contains("Jenis Sementara"),
        "jenis bebas benar-benar terhapus"
    );

    teardown_test_db(&db_name).await;
}
