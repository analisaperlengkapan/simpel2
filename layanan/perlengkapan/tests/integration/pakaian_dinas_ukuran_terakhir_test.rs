//! Operator tidak mengetik ulang ukuran yang sudah ada di database.
//!
//! Ukuran baju, celana, dan sepatu tiap pegawai sudah tercatat di profilnya
//! (halaman Ukuran). Dialog "Isi Ukuran" tetap terbuka kosong, sehingga operator
//! satker mengetik ulang data yang sudah dimiliki sistem. Kini roster menyertakan
//! saran dari profil untuk pegawai yang BELUM diisi pada pengajuan itu; yang sudah
//! diisi memakai isinya sendiri.

use super::pakaian_dinas_pengisian_test::{NIP_A, OPERATOR, SATKER_A, buat_kampanye, get, put};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

async fn pegawai_a(server: &TestServer, pengajuan_id: &str) -> Value {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    res.json::<Value>()["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_A)
        .expect("pegawai A ada di roster")
        .clone()
}

#[tokio::test]
async fn ukuran_di_profil_menjadi_saran_untuk_yang_belum_diisi_dan_tidak_menimpa_yang_sudah() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (id, pakaian) = buat_kampanye(&server).await;

    // Tanpa profil: tak ada saran.
    let kosong = pegawai_a(&server, &id).await;
    assert_eq!(kosong["sudah_diisi"], false);
    assert!(kosong["saran"].as_array().unwrap().is_empty(), "{kosong}");

    // Profil pegawai mencatat baju L, celana 32, sepatu 41, berhijab.
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas \
             (nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu, with_hijab) \
             VALUES ($1, 'Pegawai A', 'L', '32', '41', true)",
            &[&NIP_A],
        )
        .await
        .unwrap();

    let p = pegawai_a(&server, &id).await;
    let saran = p["saran"].as_array().unwrap();
    // Kampanye uji hanya punya satu kolom, BAJU: yang disarankan baju, bukan celana atau sepatu.
    assert_eq!(saran.len(), 1, "{p}");
    assert_eq!(saran[0]["pakaian_id"], pakaian.as_str());
    assert_eq!(saran[0]["ukuran"], "L");
    assert_eq!(p["with_hijab"], true, "bendera hijab ikut dari profil");

    // Operator memeriksa dan memilih ukuran lain: yang tersimpan adalah pilihannya.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": true, "ukuran": [{"pakaian_id": pakaian, "ukuran": "M"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    // Yang sudah diisi memakai isinya sendiri: tanpa saran, dan ukuran M tidak tertimpa L.
    let sesudah = pegawai_a(&server, &id).await;
    assert_eq!(sesudah["sudah_diisi"], true);
    assert!(sesudah["saran"].as_array().unwrap().is_empty(), "{sesudah}");
    assert_eq!(sesudah["ukuran"][0]["ukuran"], "M");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_kosong_di_profil_tidak_menjadi_saran() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (id, _pakaian) = buat_kampanye(&server).await;
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas \
             (nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu) VALUES ($1, 'Pegawai A', '  ', '32', '41')",
            &[&NIP_A],
        )
        .await
        .unwrap();
    let p = pegawai_a(&server, &id).await;
    assert!(
        p["saran"].as_array().unwrap().is_empty(),
        "baju kosong: {p}"
    );
    teardown_test_db(&db_name).await;
}
