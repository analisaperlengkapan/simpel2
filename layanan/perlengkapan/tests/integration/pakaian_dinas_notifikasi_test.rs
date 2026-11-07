//! Setiap serah-terima pakaian dinas memberi tahu peran yang mendapat giliran.
//!
//! Operator mengajukan -> wilayah; wilayah meneruskan -> pusat; pengembalian
//! kembali ke yang mengirim; persetujuan akhir ke operator. Sebelumnya tak seorang
//! pun diberi tahu: pencari persetujuan di mesin workflow hanya placeholder yang
//! mengembalikan daftar kosong, sehingga pekerjaan menunggu di daftar sampai peran
//! berikutnya kebetulan membukanya.
//!
//! Penerima dikenali dari siapa yang benar-benar memakai aplikasi (tabel
//! `pengguna_dikenal`): hanya yang membuka layar bisa membaca lonceng.

use super::pakaian_dinas_pengisian_test::{
    NIP_A, OPERATOR, PUSAT, SATKER_A, buat_kampanye, get, post, put,
};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const WILAYAH: &str = "00000000-0000-0000-0000-0000000000a1";
const WILAYAH_LAIN: &str = "00000000-0000-0000-0000-0000000000a2";

/// Membuka daftar kegiatan sebagai `role`: begitulah pengguna menjadi dikenal.
async fn muncul(server: &TestServer, role: &str, user: &str, satker: &str) {
    let res = get(server, "/pakaian-dinas/pengajuan", role, user, satker).await;
    assert_eq!(res.status_code(), 200, "{role}: {:?}", res.text());
}

async fn langkah(
    server: &TestServer,
    role: &str,
    user: &str,
    row: &str,
    aksi: &str,
    catatan: &str,
) {
    let res = post(
        server,
        "/pakaian-dinas/validator-action",
        role,
        user,
        SATKER_A,
        json!({"pengajuan_satker_id": row, "aksi": aksi, "komentar": catatan}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{role} {aksi}: {:?}", res.text());
}

/// (event, pesan) yang diterima `user`, terlama dulu.
async fn kotak_masuk(
    db: &layanan_perlengkapan::shared::db::Database,
    user: &str,
) -> Vec<(String, String)> {
    let client = db.pool().get().await.unwrap();
    client
        .query(
            "SELECT notification_type, message FROM notifikasi.in_app_notifications \
             WHERE user_id = $1::text::uuid ORDER BY created_at, id",
            &[&user],
        )
        .await
        .unwrap()
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect()
}

async fn baris(server: &TestServer, pengajuan_id: &str) -> String {
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
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn setiap_serah_terima_memberi_tahu_peran_berikutnya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    muncul(&server, "operator_satker", OPERATOR, SATKER_A).await;
    muncul(&server, "validator_wilayah", WILAYAH, SATKER_A).await;
    muncul(&server, "validator_pusat", PUSAT, SATKER_A).await;
    let (id, pakaian) = buat_kampanye(&server).await;
    let row = baris(&server, &id).await;

    // Operator mengisi dan mengajukan: HANYA wilayah yang diberi tahu.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    langkah(&server, "operator_satker", OPERATOR, &row, "submit", "").await;
    let w = kotak_masuk(&db, WILAYAH).await;
    assert_eq!(w.len(), 1, "wilayah diberi tahu: {w:?}");
    assert_eq!(w[0].0, "pakaian_dinas.diajukan");
    assert!(kotak_masuk(&db, PUSAT).await.is_empty(), "pusat belum");
    assert!(
        kotak_masuk(&db, OPERATOR).await.is_empty(),
        "pengirim tak diberi tahu soal kirimannya"
    );

    // Wilayah meneruskan: pusat.
    langkah(
        &server,
        "validator_wilayah",
        WILAYAH,
        &row,
        "approve",
        "lanjut",
    )
    .await;
    let p = kotak_masuk(&db, PUSAT).await;
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, "pakaian_dinas.diteruskan");

    // Pusat mengembalikan dengan catatan: wilayah, lengkap dengan catatannya.
    langkah(
        &server,
        "validator_pusat",
        PUSAT,
        &row,
        "reject",
        "Ukuran sepatu perlu dicek",
    )
    .await;
    let w = kotak_masuk(&db, WILAYAH).await;
    assert_eq!(w.len(), 2, "{w:?}");
    assert_eq!(w[1].0, "pakaian_dinas.dikembalikan_pusat");
    assert!(w[1].1.contains("Ukuran sepatu perlu dicek"), "{}", w[1].1);

    // Wilayah menurunkannya ke satker: operator, lengkap dengan catatannya.
    langkah(
        &server,
        "validator_wilayah",
        WILAYAH,
        &row,
        "reject",
        "Konfirmasi ke pegawai",
    )
    .await;
    let o = kotak_masuk(&db, OPERATOR).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].0, "pakaian_dinas.dikembalikan");
    assert!(o[0].1.contains("Konfirmasi ke pegawai"), "{}", o[0].1);

    // Diperbaiki, diteruskan, disetujui: operator diberi tahu bahwa sudah selesai.
    langkah(&server, "operator_satker", OPERATOR, &row, "submit", "").await;
    langkah(&server, "validator_wilayah", WILAYAH, &row, "approve", "").await;
    langkah(&server, "validator_pusat", PUSAT, &row, "approve", "").await;
    let o = kotak_masuk(&db, OPERATOR).await;
    assert_eq!(o.last().unwrap().0, "pakaian_dinas.selesai", "{o:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn validator_di_wilayah_lain_dan_yang_belum_pernah_membuka_aplikasi_tidak_diberi_tahu() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    muncul(&server, "operator_satker", OPERATOR, SATKER_A).await;
    muncul(&server, "validator_wilayah", WILAYAH, SATKER_A).await;
    // Dikenal, tetapi satkernya tak punya wilayah yang sama dengan SATKER_A.
    muncul(
        &server,
        "validator_wilayah",
        WILAYAH_LAIN,
        "WILAYAH-LAIN-99",
    )
    .await;
    let (id, pakaian) = buat_kampanye(&server).await;
    let row = baris(&server, &id).await;
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    langkah(&server, "operator_satker", OPERATOR, &row, "submit", "").await;

    assert_eq!(
        kotak_masuk(&db, WILAYAH).await.len(),
        1,
        "wilayah yang berhak diberi tahu"
    );
    assert!(
        kotak_masuk(&db, WILAYAH_LAIN).await.is_empty(),
        "validator wilayah lain tidak diberi tahu soal satker bukan wilayahnya"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn jumlah_yang_menunggu_dihitung_per_peran() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    muncul(&server, "operator_satker", OPERATOR, SATKER_A).await;
    let (id, pakaian) = buat_kampanye(&server).await;
    let row = baris(&server, &id).await;
    let menunggu = |role: &'static str, user: &'static str| {
        let server = &server;
        async move {
            let res = get(server, "/pakaian-dinas/menunggu", role, user, SATKER_A).await;
            assert_eq!(res.status_code(), 200, "{role}: {:?}", res.text());
            let d = res.json::<Value>()["data"].clone();
            (
                d["jumlah"].as_i64().unwrap(),
                d["peran"].as_str().unwrap().to_string(),
            )
        }
    };
    assert_eq!(
        menunggu("validator_wilayah", WILAYAH).await,
        (0, "validator_wilayah".to_string())
    );

    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200);
    langkah(&server, "operator_satker", OPERATOR, &row, "submit", "").await;
    assert_eq!(
        menunggu("validator_wilayah", WILAYAH).await,
        (1, "validator_wilayah".to_string())
    );
    assert_eq!(
        menunggu("validator_pusat", PUSAT).await,
        (0, "validator_pusat".to_string())
    );

    langkah(&server, "validator_wilayah", WILAYAH, &row, "approve", "").await;
    assert_eq!(menunggu("validator_wilayah", WILAYAH).await.0, 0);
    assert_eq!(menunggu("validator_pusat", PUSAT).await.0, 1);

    langkah(&server, "validator_pusat", PUSAT, &row, "reject", "cek").await;
    langkah(&server, "validator_wilayah", WILAYAH, &row, "reject", "cek").await;
    assert_eq!(
        menunggu("operator_satker", OPERATOR).await,
        (1, "operator_satker".to_string())
    );

    // Peran tanpa giliran dalam alur ini tak punya hitungan.
    let res = get(
        &server,
        "/pakaian-dinas/menunggu",
        "admin",
        WILAYAH,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200);
    assert_eq!(res.json::<Value>()["data"]["peran"], "");

    teardown_test_db(&db_name).await;
}
