//! Menghapus kampanye tidak boleh membuang pekerjaan yang sudah masuk.
//!
//! Penjaganya hanya memeriksa status HEADER kampanye (`aktivitas_id != 1000`),
//! sedangkan header tak pernah bergeser dari 1000: alur berjalan di baris satker.
//! Jadi penjaga itu lolos untuk setiap kampanye, dan ikon tempat sampah di kartu
//! kampanye menghapus — dengan cascade — ukuran yang sudah diisi, pengajuan, dan
//! persetujuan seluruh satker. Di staging, kampanye dengan satu satker di 1001
//! terhapus dengan "berhasil dihapus".
//!
//! Dua arah: yang sudah berjalan DITOLAK dan utuh, yang belum disentuh TERHAPUS
//! beserta semua turunannya.

use super::pakaian_dinas_pengisian_test::{
    NIP_A, OPERATOR, PUSAT, SATKER_A, buat_kampanye, get, post, put,
};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};
use uuid::Uuid;

async fn hapus(server: &TestServer, pengajuan_id: &str) -> axum_test::TestResponse {
    let mut req = server.delete(&format!("/pakaian-dinas/pengajuan/{pengajuan_id}"));
    for (k, v) in super::pakaian_dinas_pengisian_test::headers("validator_pusat", PUSAT, SATKER_A) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn jumlah(db: &layanan_perlengkapan::shared::db::Database, sql: &str, id: &Uuid) -> i64 {
    let client = db.pool().get().await.unwrap();
    client.query_one(sql, &[id]).await.unwrap().get(0)
}

#[tokio::test]
async fn kampanye_yang_sudah_berjalan_tidak_dapat_dihapus() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let id = Uuid::parse_str(&pengajuan_id).unwrap();

    // Satu satker mengisi dan mengajukan: 1000 -> 1001. Yang lain tetap 1000.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let row = res.json::<Value>()["data"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": row, "aksi": "submit"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    let sebelum_satker = jumlah(
        &db,
        "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
        &id,
    )
    .await;
    let sebelum_ukuran = jumlah(
        &db,
        "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran u \
         JOIN perlengkapan.pengajuan_pakaian_dinas_satker s ON s.id = u.pengajuan_satker_id WHERE s.pengajuan_id = $1",
        &id,
    )
    .await;
    assert!(
        sebelum_satker >= 2 && sebelum_ukuran >= 1,
        "ada pekerjaan yang harus dilindungi"
    );

    let res = hapus(&server, &pengajuan_id).await;
    assert_eq!(
        res.status_code(),
        400,
        "kampanye yang sudah berjalan: {:?}",
        res.text()
    );
    assert!(
        res.text().contains("sudah mengajukan"),
        "pesannya menyebut sebabnya: {}",
        res.text()
    );

    // Utuh: kampanye, baris satker, ukuran, dan riwayat.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kampanye masih ada");
    assert_eq!(jumlah(&db, "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1", &id).await, sebelum_satker);
    assert_eq!(
        jumlah(
            &db,
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran u \
             JOIN perlengkapan.pengajuan_pakaian_dinas_satker s ON s.id = u.pengajuan_satker_id WHERE s.pengajuan_id = $1",
            &id
        )
        .await,
        sebelum_ukuran
    );
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn kampanye_yang_belum_disentuh_terhapus_bersama_turunannya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let id = Uuid::parse_str(&pengajuan_id).unwrap();

    // Ukuran sudah diisi tetapi BELUM diajukan: ini masih draf, boleh dibuang.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    // Catat id baris satker SEBELUM dihapus: sesudahnya tak ada lagi yang bisa ditanyakan
    // lewat kampanye, jadi turunannya dicari lewat id baris itu.
    let baris: Vec<Uuid> = {
        let client = db.pool().get().await.unwrap();
        client
            .query(
                "SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
                &[&id],
            )
            .await
            .unwrap()
            .iter()
            .map(|r| r.get(0))
            .collect()
    };
    assert!(!baris.is_empty());

    let res = hapus(&server, &pengajuan_id).await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 404);
    for (apa, sql) in [
        (
            "baris satker",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
        ),
        (
            "kolom pakaian",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE pengajuan_id = $1",
        ),
        (
            "satker terpilih",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih WHERE pengajuan_id = $1",
        ),
    ] {
        assert_eq!(jumlah(&db, sql, &id).await, 0, "{apa} tidak boleh tersisa");
    }
    let client = db.pool().get().await.unwrap();
    for (apa, sql) in [
        (
            "pegawai",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ANY($1)",
        ),
        (
            "ukuran pegawai",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran WHERE pengajuan_satker_id = ANY($1)",
        ),
        (
            "riwayat",
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas WHERE pengajuan_satker_id = ANY($1)",
        ),
    ] {
        let n: i64 = client.query_one(sql, &[&baris]).await.unwrap().get(0);
        assert_eq!(n, 0, "{apa} tidak boleh tersisa");
    }
    teardown_test_db(&db_name).await;
}
