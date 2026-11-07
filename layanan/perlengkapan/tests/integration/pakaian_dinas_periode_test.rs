//! Tanggal kegiatan ditegakkan.
//!
//! Tanggal mulai dan selesai pada kegiatan hanya dipakai untuk menampilkan lencana
//! "Dibuka" di kartu; tak ada yang menolak apa pun di luar jendela itu. Terukur di
//! staging: kegiatan berperiode Januari 2026 bertanda tidak dibuka, namun operator
//! tetap bisa menyimpan ukuran dan mengajukan (keduanya dijawab 200).
//!
//! Dua arah: di luar jendela DITOLAK dengan sebab (belum dibuka / sudah ditutup,
//! lengkap dengan tanggalnya), di dalam jendela berjalan, dan validator pusat bisa
//! memperpanjang kegiatan yang tertutup (kalau tidak, tanggal yang ditegakkan
//! membuat kegiatan itu mati).

use super::pakaian_dinas_pengisian_test::{
    NIP_A, OPERATOR, PUSAT, SATKER_A, buat_kampanye_periode, get, post, put,
};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

fn hari(selisih: i64) -> String {
    (chrono::Utc::now().date_naive() + chrono::Duration::days(selisih)).to_string()
}

async fn isi(server: &TestServer, pengajuan_id: &str, pakaian_id: &str) -> axum_test::TestResponse {
    put(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await
}

async fn baris_satker(server: &TestServer, pengajuan_id: &str) -> String {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    res.json::<Value>()["data"][0]["id"]
        .as_str()
        .expect("baris satker")
        .to_string()
}

async fn aksi(server: &TestServer, row: &str, role: &str, aksi: &str) -> axum_test::TestResponse {
    let user = if role == "operator_satker" {
        OPERATOR
    } else {
        PUSAT
    };
    post(
        server,
        "/pakaian-dinas/validator-action",
        role,
        user,
        SATKER_A,
        json!({"pengajuan_satker_id": row, "aksi": aksi, "komentar": "uji periode"}),
    )
    .await
}

#[tokio::test]
async fn kegiatan_yang_sudah_ditutup_menolak_isi_ajukan_dan_keputusan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (id, pakaian) = buat_kampanye_periode(&server, &hari(-60), &hari(-1)).await;
    let row = baris_satker(&server, &id).await;

    let res = isi(&server, &id, &pakaian).await;
    assert_eq!(
        res.status_code(),
        400,
        "isi di luar periode: {:?}",
        res.text()
    );
    assert!(res.text().contains("sudah ditutup"), "{}", res.text());

    let res = aksi(&server, &row, "operator_satker", "submit").await;
    assert_eq!(
        res.status_code(),
        400,
        "ajukan di luar periode: {:?}",
        res.text()
    );
    assert!(res.text().contains("sudah ditutup"), "{}", res.text());

    // Satker yang sudah menunggu wilayah juga tak bisa diputuskan: setelah tanggal
    // selesai tidak ada yang berubah, bagi siapa pun.
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "UPDATE perlengkapan.pengajuan_pakaian_dinas_satker SET aktivitas_id = 1001 WHERE id = $1",
            &[&uuid::Uuid::parse_str(&row).unwrap()],
        )
        .await
        .unwrap();
    let res = aksi(&server, &row, "validator_wilayah", "approve").await;
    assert_eq!(
        res.status_code(),
        400,
        "putuskan di luar periode: {:?}",
        res.text()
    );
    assert!(res.text().contains("sudah ditutup"), "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn kegiatan_yang_belum_dibuka_menolak_isi_dan_ajukan() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (id, pakaian) = buat_kampanye_periode(&server, &hari(5), &hari(60)).await;
    let row = baris_satker(&server, &id).await;

    let res = isi(&server, &id, &pakaian).await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());
    assert!(res.text().contains("belum dibuka"), "{}", res.text());

    let res = aksi(&server, &row, "operator_satker", "submit").await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());
    assert!(res.text().contains("belum dibuka"), "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn hari_pertama_dan_hari_terakhir_masih_termasuk() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    // Mulai hari ini dan selesai hari ini: jendela satu hari.
    let (id, pakaian) = buat_kampanye_periode(&server, &hari(0), &hari(0)).await;
    let res = isi(&server, &id, &pakaian).await;
    assert_eq!(res.status_code(), 200, "satu hari penuh: {:?}", res.text());
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pusat_dapat_memperpanjang_kegiatan_yang_ditutup() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (id, pakaian) = buat_kampanye_periode(&server, &hari(-60), &hari(-1)).await;
    assert_eq!(isi(&server, &id, &pakaian).await.status_code(), 400);

    // Hanya validator pusat yang boleh mengubah kegiatan.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"tgl_selesai": hari(30)}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        403,
        "operator mengubah kegiatan: {:?}",
        res.text()
    );

    // Tanggal terbalik ditolak.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({"tgl_mulai": hari(10), "tgl_selesai": hari(5)}),
    )
    .await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());

    // Pusat memperpanjang: yang lain tak berubah, dan pengisian jalan lagi.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({"tgl_selesai": hari(30), "deskripsi": "Diperpanjang"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    assert_eq!(body["data"]["tgl_selesai"], hari(30));
    assert_eq!(
        body["data"]["tgl_mulai"],
        hari(-60),
        "yang tak dikirim tak berubah"
    );
    assert_eq!(body["data"]["deskripsi"], "Diperpanjang");
    assert_eq!(body["data"]["is_open"], true);

    assert_eq!(isi(&server, &id, &pakaian).await.status_code(), 200);
    teardown_test_db(&db_name).await;
}
