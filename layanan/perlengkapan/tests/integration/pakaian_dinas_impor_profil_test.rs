//! Impor profil ukuran awal (PR 4): `POST /pakaian-dinas/impor-profil`.
//!
//! Data awal datang dari berkas satker yang dibersihkan di luar aplikasi; endpoint ini tetap
//! memvalidasi ulang dengan aturan yang sama dengan setiap jalur penyimpan ukuran, menolak baris
//! yang buruk SENDIRI-SENDIRI, tak pernah menimpa isian orangnya sendiri, dan tak membawa NIP utuh
//! pada galat.

use super::pakaian_dinas_pengisian_test::{OPERATOR, SATKER_A, post};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";

fn ukuran(jenis: &str, spek: &str, nilai: &str) -> Value {
    json!({"jenis": jenis, "spesifikasi": spek, "ukuran": nilai})
}

fn baris(rujukan: &str, nip: &str, nama: &str, ukuran: Vec<Value>) -> Value {
    json!({"rujukan": rujukan, "nip": nip, "nama": nama, "jenis_kelamin": "L", "with_hijab": false, "ukuran": ukuran})
}

async fn impor(server: &TestServer, dry_run: bool, baris: Vec<Value>) -> axum_test::TestResponse {
    post(
        server,
        "/pakaian-dinas/impor-profil",
        "admin",
        ADMIN,
        SATKER_A,
        json!({"dry_run": dry_run, "baris": baris}),
    )
    .await
}

async fn nilai(
    db: &layanan_perlengkapan::shared::db::Database,
    nip: &str,
    jenis: &str,
    spek: &str,
) -> Option<(String, String)> {
    let client = db.pool().get().await.unwrap();
    client
        .query_opt(
            "SELECT u.ukuran, u.sumber FROM perlengkapan.pegawai_pakaian_dinas_ukuran u
             JOIN perlengkapan.ms_spesifikasi_pakaian_dinas s ON s.id = u.spesifikasi_id
             JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
             WHERE u.nip = $1 AND upper(j.nama) = $2 AND upper(s.nama) = upper($3)",
            &[&nip, &jenis, &spek],
        )
        .await
        .unwrap()
        .map(|r| (r.get("ukuran"), r.get("sumber")))
}

async fn jumlah_profil(db: &layanan_perlengkapan::shared::db::Database, nip: &str) -> i64 {
    let client = db.pool().get().await.unwrap();
    client
        .query_one(
            "SELECT count(*) AS n FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = $1",
            &[&nip],
        )
        .await
        .unwrap()
        .get("n")
}

#[tokio::test]
async fn dry_run_menghitung_apa_yang_akan_terjadi_tanpa_menulis_apa_pun() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let nip = "198001010000000011";

    let res = impor(
        &server,
        true,
        vec![baris(
            "uji:1",
            nip,
            "Budi Santoso",
            vec![
                ukuran("PDH", "Pakaian Dinas", "xl"),
                ukuran("PDL", "Nama Dada", "BUDI"),
            ],
        )],
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let h = &res.json::<Value>()["data"];
    assert_eq!(h["dry_run"], true);
    assert_eq!(h["baris_dimuat"], 1);
    assert_eq!(h["profil_baru"], 1);
    assert_eq!(h["ukuran_ditulis"], 2);
    assert_eq!(
        jumlah_profil(&db, nip).await,
        0,
        "dry_run tak menulis profil"
    );
    assert!(
        nilai(&db, nip, "PDH", "Pakaian Dinas").await.is_none(),
        "dry_run tak menulis ukuran"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn impor_menulis_profil_dan_ukuran_per_spesifikasi_dengan_ejaan_daftar_dan_sumber_impor() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let nip = "198001010000000012";

    let res = impor(
        &server,
        false,
        vec![json!({"rujukan": "uji:1", "nip": nip, "nama": "  Siti   Aminah ", "jenis_kelamin": "p", "with_hijab": true,
            "ukuran": [
              ukuran("PDH", "Pakaian Dinas", "xl"),
              ukuran("PDH", "Sepatu Dinas", "42"),
              ukuran("PDL", "Sepatu", "43"),
              ukuran("PDL", "Topi", "57"),
              ukuran("PDL", "Nama Dada", "S. AMINAH"),
            ]})],
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let h = &res.json::<Value>()["data"];
    assert_eq!(
        (
            h["baris_dimuat"].as_i64(),
            h["profil_baru"].as_i64(),
            h["ukuran_ditulis"].as_i64()
        ),
        (Some(1), Some(1), Some(5))
    );

    // Ejaan daftar ("xl" -> "XL"), PDH dan PDL sepatu terpisah, teks bebas utuh, sumber IMPOR.
    assert_eq!(
        nilai(&db, nip, "PDH", "Pakaian Dinas").await,
        Some(("XL".into(), "IMPOR".into()))
    );
    assert_eq!(
        nilai(&db, nip, "PDH", "Sepatu Dinas").await,
        Some(("42".into(), "IMPOR".into()))
    );
    assert_eq!(
        nilai(&db, nip, "PDL", "Sepatu").await,
        Some(("43".into(), "IMPOR".into()))
    );
    assert_eq!(
        nilai(&db, nip, "PDL", "Topi").await,
        Some(("57".into(), "IMPOR".into()))
    );
    assert_eq!(
        nilai(&db, nip, "PDL", "Nama Dada").await,
        Some(("S. AMINAH".into(), "IMPOR".into()))
    );
    let client = db.pool().get().await.unwrap();
    let p = client
        .query_one("SELECT nama, jenis_kelamin, with_hijab, ukuran_baju FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = $1", &[&nip])
        .await
        .unwrap();
    assert_eq!(p.get::<_, String>("nama"), "Siti Aminah", "nama dirapikan");
    assert_eq!(p.get::<_, String>("jenis_kelamin"), "P");
    assert!(p.get::<_, bool>("with_hijab"));
    assert!(
        p.get::<_, Option<String>>("ukuran_baju").is_none(),
        "kolom lama tak disentuh"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn baris_yang_buruk_ditolak_sendiri_sendiri_dan_galat_tak_memuat_nip_utuh() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let bagus = "198001010000000013";
    let nip_pendek = "19800101000000001";
    let nip_salah_ukuran = "198001010000000014";
    let nip_spek = "198001010000000015";

    let res = impor(
        &server,
        false,
        vec![
            baris(
                "uji:1",
                bagus,
                "Baik",
                vec![ukuran("PDH", "Pakaian Dinas", "L")],
            ),
            baris(
                "uji:2",
                nip_pendek,
                "NIP Pendek",
                vec![ukuran("PDH", "Pakaian Dinas", "L")],
            ),
            baris(
                "uji:3",
                nip_salah_ukuran,
                "Ukuran Salah",
                vec![ukuran("PDH", "Pakaian Dinas", "99XL")],
            ),
            baris(
                "uji:4",
                nip_spek,
                "Kolom Tak Ada",
                vec![ukuran("PDH", "Jubah Ajaib", "L")],
            ),
            baris(
                "uji:5",
                bagus,
                "NIP Ganda",
                vec![ukuran("PDH", "Pakaian Dinas", "M")],
            ),
            baris("uji:6", "198001010000000016", "Tanpa Ukuran", vec![]),
        ],
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.json::<Value>();
    let h = &body["data"];
    assert_eq!(
        (h["diterima"].as_i64(), h["baris_dimuat"].as_i64()),
        (Some(6), Some(1))
    );
    let ditolak = h["ditolak"].as_array().unwrap();
    assert_eq!(ditolak.len(), 5, "{body}");
    let alasan = |rujukan: &str| -> String {
        ditolak
            .iter()
            .find(|d| d["rujukan"] == rujukan)
            .unwrap_or_else(|| panic!("{rujukan} tak ditolak: {body}"))["alasan"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(alasan("uji:2").contains("18 digit"));
    assert!(
        alasan("uji:3").contains("tidak ada di daftar"),
        "{}",
        alasan("uji:3")
    );
    assert!(alasan("uji:4").contains("tidak dikenal"));
    assert!(alasan("uji:5").contains("ganda"));
    assert!(alasan("uji:6").contains("Tidak ada ukuran"));

    // Galat dan respons tak memuat NIP utuh siapa pun (hanya yang disamarkan).
    let teks = res.text();
    for nip in [bagus, nip_pendek, nip_salah_ukuran, nip_spek] {
        assert!(!teks.contains(nip), "respons memuat NIP utuh {nip}: {teks}");
    }
    assert!(
        teks.contains("198000…13") || teks.contains("198001…13"),
        "NIP tersamar ada: {teks}"
    );

    // Yang baik tetap termuat, yang buruk tidak.
    assert_eq!(jumlah_profil(&db, bagus).await, 1);
    assert_eq!(jumlah_profil(&db, nip_salah_ukuran).await, 0);
    assert_eq!(
        nilai(&db, bagus, "PDH", "Pakaian Dinas").await.unwrap().0,
        "L",
        "baris ganda tak menimpa yang pertama"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn impor_ulang_menimpa_hasil_impor_tetapi_tidak_isian_orangnya_atau_profilnya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let nip = "199203142014031003";

    let hasil = |res: &axum_test::TestResponse| res.json::<Value>()["data"].clone();
    let res = impor(
        &server,
        false,
        vec![baris(
            "uji:1",
            nip,
            "Budi Santoso",
            vec![ukuran("PDL", "Kaos", "M")],
        )],
    )
    .await;
    assert_eq!(hasil(&res)["profil_baru"], 1, "{}", res.text());

    // Orangnya sendiri mengubah kaosnya lewat aplikasi (sumber APLIKASI).
    let kaos: String = {
        let client = db.pool().get().await.unwrap();
        let id: uuid::Uuid = client
            .query_one(
                "SELECT s.id FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                 JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
                 WHERE upper(j.nama) = 'PDL' AND upper(s.nama) = 'KAOS'",
                &[],
            )
            .await
            .unwrap()
            .get("id");
        id.to_string()
    };
    let auth = reqwest::header::HeaderValue::from_str(&format!(
        "Bearer mock::operator_satker::{OPERATOR}::{SATKER_A}::{nip}"
    ))
    .unwrap();
    let res = server
        .put("/pakaian-dinas/ukuran-profil")
        .add_header(reqwest::header::AUTHORIZATION, auth)
        .json(&json!({"ukuran": [{"spesifikasi_id": kaos, "ukuran": "L"}]}))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert_eq!(
        nilai(&db, nip, "PDL", "Kaos").await,
        Some(("L".into(), "APLIKASI".into()))
    );

    // Impor ulang dengan nama lain, kaos lain, dan topi baru: isian orangnya dan profilnya tetap;
    // yang baru (topi) masuk.
    let res = impor(
        &server,
        false,
        vec![baris(
            "uji:2",
            nip,
            "Nama Lain",
            vec![ukuran("PDL", "Kaos", "S"), ukuran("PDL", "Topi", "58")],
        )],
    )
    .await;
    let h = hasil(&res);
    assert_eq!(
        (h["profil_sudah_ada"].as_i64(), h["profil_baru"].as_i64()),
        (Some(1), Some(0)),
        "{h}"
    );
    assert_eq!(
        (
            h["ukuran_ditulis"].as_i64(),
            h["ukuran_dilewati_sudah_diisi"].as_i64()
        ),
        (Some(1), Some(1)),
        "{h}"
    );
    assert_eq!(
        nilai(&db, nip, "PDL", "Kaos").await.unwrap().0,
        "L",
        "isian orangnya sendiri tak ditimpa"
    );
    assert_eq!(
        nilai(&db, nip, "PDL", "Topi").await,
        Some(("58".into(), "IMPOR".into()))
    );
    let client = db.pool().get().await.unwrap();
    let nama: Option<String> = client
        .query_one(
            "SELECT nama FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = $1",
            &[&nip],
        )
        .await
        .unwrap()
        .get("nama");
    assert_ne!(
        nama.as_deref(),
        Some("Nama Lain"),
        "profil yang sudah ada tak diubah"
    );

    // Hasil impor sebelumnya boleh diganti oleh impor berikutnya.
    let res = impor(
        &server,
        false,
        vec![baris(
            "uji:3",
            nip,
            "Nama Lain",
            vec![ukuran("PDL", "Topi", "59")],
        )],
    )
    .await;
    assert_eq!(hasil(&res)["ukuran_ditulis"], 1, "{}", res.text());
    assert_eq!(
        nilai(&db, nip, "PDL", "Topi").await,
        Some(("59".into(), "IMPOR".into()))
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn hanya_admin_dan_satu_lot_paling_banyak_2000_baris() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = post(
        &server,
        "/pakaian-dinas/impor-profil",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"baris": [baris("uji:1", "198001010000000017", "X", vec![ukuran("PDH", "Pakaian Dinas", "L")])]}),
    )
    .await;
    assert_eq!(res.status_code(), 403, "{}", res.text());

    let res = impor(&server, false, vec![]).await;
    assert_eq!(res.status_code(), 400, "lot kosong: {}", res.text());
    let banyak: Vec<Value> = (0..2001)
        .map(|i| baris(&format!("uji:{i}"), "198001010000000018", "X", vec![]))
        .collect();
    let res = impor(&server, true, banyak).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert!(res.text().contains("2000"), "{}", res.text());

    teardown_test_db(&db_name).await;
}
