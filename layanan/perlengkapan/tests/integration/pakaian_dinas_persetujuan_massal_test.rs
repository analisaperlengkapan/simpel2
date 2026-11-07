//! Persetujuan massal: satu permintaan, banyak satker, tiap satker lewat jalur yang sama.
//!
//! Skala nasional berarti ratusan satker; menyetujui satu per satu berarti ratusan
//! klik. Yang ditegakkan di sini adalah bahwa "massal" tidak melonggarkan apa pun:
//! baris yang tidak berhak (tahap salah, di luar cakupan) ditolak SENDIRI dengan
//! alasannya, dan sisanya tetap jalan. Hasil sebagian adalah jawaban normal (200).

use super::pakaian_dinas_pengisian_test::{PUSAT, SATKER_A, buat_kampanye, get, post};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

/// Id of SATKER_A's row in a campaign. Two campaigns for the same satker give two
/// rows that one operator, one wilayah and one pusat can all see, which is what a
/// bulk request needs without bringing a second satker's scope into the test.
async fn baris_satker_a(server: &TestServer, pengajuan_id: &str) -> String {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker?page=1&per_page=50"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"] == SATKER_A)
        .expect("baris satker")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn status(server: &TestServer, pengajuan_id: &str) -> i64 {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker?page=1&per_page=50"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"] == SATKER_A)
        .unwrap()["aktivitas_id"]
        .as_i64()
        .unwrap()
}

/// Two campaigns, SATKER_A's row of each: `[(campaign, row), (campaign, row)]`.
async fn dua_baris(server: &TestServer) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for _ in 0..2 {
        let (id, _pakaian) = buat_kampanye(server).await;
        let row = baris_satker_a(server, &id).await;
        out.push((id, row));
    }
    out
}

async fn massal(
    server: &TestServer,
    role: &str,
    ids: &[String],
    aksi: &str,
) -> axum_test::TestResponse {
    post(
        server,
        "/pakaian-dinas/validator-action/bulk",
        role,
        PUSAT,
        SATKER_A,
        json!({"pengajuan_satker_ids": ids, "aksi": aksi, "komentar": "disetujui massal"}),
    )
    .await
}

async fn satu(server: &TestServer, role: &str, id: &str, aksi: &str) {
    let res = post(
        server,
        "/pakaian-dinas/validator-action",
        role,
        PUSAT,
        SATKER_A,
        json!({"pengajuan_satker_id": id, "aksi": aksi}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{role} {aksi}: {:?}", res.text());
}

#[tokio::test]
async fn wilayah_meneruskan_banyak_satker_sekaligus_dan_yang_salah_tahap_ditolak_sendiri() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let dua = dua_baris(&server).await;
    let (a, b) = (dua[0].clone(), dua[1].clone());

    // Keduanya diajukan operator (1000 -> 1001) lewat jalur biasa.
    satu(&server, "operator_satker", &a.1, "submit").await;
    satu(&server, "operator_satker", &b.1, "submit").await;
    // Satu sudah diteruskan sendiri: tahapnya kini salah untuk "meneruskan" lagi.
    satu(&server, "validator_wilayah", &b.1, "approve").await;

    let res = massal(
        &server,
        "validator_wilayah",
        &[a.1.clone(), b.1.clone()],
        "approve",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    let berhasil = body["data"]["berhasil"].as_array().unwrap();
    let gagal = body["data"]["gagal"].as_array().unwrap();
    assert_eq!(berhasil.len(), 1, "{body:?}");
    assert_eq!(berhasil[0]["pengajuan_satker_id"], a.1.as_str());
    assert_eq!(berhasil[0]["aktivitas_id"], 1004, "diteruskan ke Pusat");
    assert_eq!(gagal.len(), 1, "{body:?}");
    assert_eq!(gagal[0]["pengajuan_satker_id"], b.1.as_str());
    assert!(
        !gagal[0]["alasan"].as_str().unwrap().is_empty(),
        "yang ditolak menyebut sebabnya"
    );

    // Keduanya kini menunggu Pusat; yang salah tahap tidak bergeser dua kali.
    assert_eq!(status(&server, &a.0).await, 1004);
    assert_eq!(status(&server, &b.0).await, 1004);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pusat_menyetujui_massal_sampai_selesai() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let dua = dua_baris(&server).await;
    for (_, row) in &dua {
        satu(&server, "operator_satker", row, "submit").await;
        satu(&server, "validator_wilayah", row, "approve").await;
    }
    let ids: Vec<String> = dua.iter().map(|d| d.1.clone()).collect();

    let res = massal(&server, "validator_pusat", &ids, "approve").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    assert_eq!(
        body["data"]["berhasil"].as_array().unwrap().len(),
        2,
        "{body:?}"
    );
    assert!(
        body["data"]["gagal"].as_array().unwrap().is_empty(),
        "{body:?}"
    );
    for (campaign, _) in &dua {
        assert_eq!(status(&server, campaign).await, 1008, "Selesai");
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn massal_tidak_melonggarkan_peran_dan_hanya_menyetujui() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let dua = dua_baris(&server).await;
    let ids: Vec<String> = dua.iter().map(|d| d.1.clone()).collect();
    for row in &ids {
        satu(&server, "operator_satker", row, "submit").await;
    }

    // Operator tidak boleh memakai jalur massal.
    let res = massal(&server, "operator_satker", &ids, "approve").await;
    assert_eq!(res.status_code(), 403, "{:?}", res.text());

    // Pusat tidak boleh menyetujui baris yang masih menunggu wilayah: ditolak per baris.
    let res = massal(&server, "validator_pusat", &ids, "approve").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    assert!(
        body["data"]["berhasil"].as_array().unwrap().is_empty(),
        "{body:?}"
    );
    assert_eq!(
        body["data"]["gagal"].as_array().unwrap().len(),
        2,
        "{body:?}"
    );
    for (campaign, _) in &dua {
        assert_eq!(status(&server, campaign).await, 1001, "tidak bergeser");
    }

    // Mengembalikan tidak lewat jalur massal (butuh catatan sendiri-sendiri).
    let res = massal(&server, "validator_wilayah", &ids, "reject").await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());
    // Kosong ditolak.
    let res = massal(&server, "validator_wilayah", &[], "approve").await;
    assert_eq!(res.status_code(), 400, "{:?}", res.text());

    teardown_test_db(&db_name).await;
}
