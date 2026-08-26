//! Sebuah aset dikenali oleh **kode satker + kode barang + NUP**, dan berkas
//! ini yang menjaganya tetap begitu di jalur pemakaian BMN.
//!
//! Aturan domainnya dari stakeholder, apa adanya: "untuk mengidentifikasi
//! suatu aset itu dibedakan tiga hal yaitu nama satkernya, nama barangnya, dan
//! nup nya … misal ada satker kejaksaan negeri mamuju nama barang kendaraan
//! unit tahanan nup 2 itu pasti unik tidak ada duplikasinya". NUP adalah nomor
//! urut pendaftaran — nomor urut DI DALAM satu satker untuk satu kode barang.
//!
//! Cek tabrakan pemesanan dulu ber-key `bmn_nup` saja. Diukur pada snapshot
//! SIMAN staging (624.533 baris, 2026-06-17):
//!
//! * 14.142 nilai NUP berbeda menampung 624.533 aset — rata-rata 44,2 aset
//!   per NUP;
//! * NUP `1` sendirian dipakai **44.017 aset di 553 satker**;
//! * bahkan di DALAM satu satker, 41,6% pasangan (satker, NUP) menunjuk lebih
//!   dari satu kode barang, sampai 425 kode barang untuk satu NUP.
//!
//! Jadi satu izin aktif atas NUP `1` di satker mana pun memblokir 552 satker
//! lain memakai aset mereka SENDIRI yang kebetulan bernomor 1 — sambil
//! menyebutkan nama pegawai satker lain sebagai pemegangnya di pesan error.
//!
//! Tak ada yang bisa memerahkannya: tabel izin di staging hanya berisi 2 baris
//! seed dengan NUP berbeda, jadi tabrakan itu tak pernah terjadi di sana.
//! Fixture di bawah sengaja memakai NUP yang SAMA di dua satker — ruang nilai
//! yang sebenarnya, bukan yang enak dibaca.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker_code: &str) -> Headers {
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
            reqwest::header::HeaderValue::from_str(satker_code).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user_id}::{satker_code}"
            ))
            .unwrap(),
        ),
    ]
}

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";

/// SKR001 dan SKR002 bernaung di Kejati KJT01; SKR003 di KJT02.
const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
const SATKER_C: &str = "SKR003";

/// NUP yang dipakai kedua satker — inti perkaranya.
const NUP: &str = "1";
const KODE_BARANG: &str = "3060201003";
/// Kode barang lain di satker yang sama dengan NUP yang sama. Ini bukan
/// karangan: 41,6% pasangan (satker, NUP) nyata memang begini.
const KODE_BARANG_LAIN: &str = "3050104002";

/// Nama pemegang di satker B. Kalau nama ini muncul di jawaban untuk satker A,
/// batas satker sudah bocor.
const PEMEGANG_B: &str = "Budi Pemegang Satker B";

/// Sisipkan satu izin ACTIVE langsung ke DB.
///
/// `create` lewat HTTP hanya menghasilkan DRAFT, sedangkan cek tabrakan
/// mencari yang ACTIVE — jadi prasyaratnya diseed, bukan digiring lewat
/// seluruh rantai persetujuan.
async fn seed_active_permit(
    db: &layanan_perlengkapan::shared::db::Database,
    satker: &str,
    nup: &str,
    kode_barang: &str,
    pegawai: &str,
) -> Uuid {
    let client = db.pool().get().await.unwrap();
    let id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO perlengkapan.izin_pemakaian_bmn
                (id, nomor_izin, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang,
                 pegawai_nip, pegawai_nama, pegawai_jabatan,
                 pegawai_satker_id, pegawai_satker_nama, satker_code,
                 tanggal_mulai, tanggal_selesai, status, approved_at)
             VALUES ($1, $2, 'LAINNYA', $3, $4, 'Barang Uji',
                     $5, $6, 'Staf',
                     gen_random_uuid(), $7, $8,
                     CURRENT_DATE - 10, CURRENT_DATE + 20, 'ACTIVE', NOW())",
            &[
                &id,
                // `nomor_izin` unik: dua izin bisa sengaja menunjuk aset yang
                // sama (lihat tes perpanjangan), jadi nomornya tak boleh
                // diturunkan dari identitas asetnya.
                &format!(
                    "IZIN/{satker}/{kode_barang}/{nup}/{}",
                    &id.simple().to_string()[..8]
                ),
                &nup,
                &kode_barang,
                &format!("1990010100{}", &satker[5..]),
                &pegawai,
                &format!("KEJAKSAAN NEGERI UJI {}", &satker[5..]),
                &satker,
            ],
        )
        .await
        .unwrap();
    id
}

fn create_body(nup: &str, kode_barang: &str) -> serde_json::Value {
    json!({
        "pegawai_nip": "198501012010011001",
        "pegawai_nama": "Ani Pemohon",
        "pegawai_satker_id": "00000000-0000-0000-0000-000000000009",
        "pegawai_satker_nama": "Kejaksaan Negeri Uji",
        "jenis_bmn": "LAINNYA",
        "bmn_nup": nup,
        "bmn_kode_barang": kode_barang,
        "bmn_nama_barang": "Barang Uji",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari"
    })
}

async fn post_create(
    server: &TestServer,
    satker: &str,
    nup: &str,
    kode_barang: &str,
) -> axum_test::TestResponse {
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_body(nup, kode_barang));
    for (k, v) in auth_headers("operator_satker", OPERATOR, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn get_as(server: &TestServer, path: &str, satker: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers("operator_satker", OPERATOR, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// Dua satker boleh sama-sama memakai aset MEREKA SENDIRI yang ber-NUP sama.
///
/// Ini regresi utamanya. Dengan kunci lama, izin aktif milik satker B atas NUP
/// `1` menolak permohonan satker A atas aset yang sama sekali berbeda — dan
/// menyebut nama pegawai satker B sebagai alasannya.
#[tokio::test]
async fn a_second_satker_may_book_its_own_asset_with_the_same_nup() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    seed_active_permit(&db, SATKER_B, NUP, KODE_BARANG, PEMEGANG_B).await;

    let res = post_create(&server, SATKER_A, NUP, KODE_BARANG).await;
    let body = res.text();
    assert_eq!(
        res.status_code(),
        201,
        "satker A ditolak atas aset satker B yang kebetulan ber-NUP sama: {body}"
    );
    assert!(
        !body.contains(PEMEGANG_B),
        "jawaban untuk satker A memuat nama pemegang dari satker B:\n{body}"
    );

    teardown_test_db(&db_name).await;
}

/// Aset yang SAMA persis tetap tak boleh dipesan dua kali — kalau ini lolos,
/// tes di atas cuma membuktikan cek tabrakannya sudah mati total.
#[tokio::test]
async fn the_very_same_asset_still_cannot_be_double_booked() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    seed_active_permit(&db, SATKER_B, NUP, KODE_BARANG, PEMEGANG_B).await;

    let res = post_create(&server, SATKER_B, NUP, KODE_BARANG).await;
    let body = res.text();
    assert_eq!(
        res.status_code(),
        400,
        "aset yang sama berhasil dipesan dua kali: {body}"
    );
    // Di dalam satker sendiri, nama pemegang memang harus disebut — operator
    // perlu tahu harus menghubungi siapa.
    assert!(
        body.contains(PEMEGANG_B),
        "penolakan di satker sendiri tidak menyebutkan pemegangnya:\n{body}"
    );
    assert!(
        body.contains(KODE_BARANG),
        "pesan penolakan tidak menyebut kode barang, padahal NUP saja ambigu:\n{body}"
    );

    teardown_test_db(&db_name).await;
}

/// NUP yang sama di satker yang sama tapi kode barang berbeda = aset berbeda.
#[tokio::test]
async fn the_same_nup_under_a_different_kode_barang_is_a_different_asset() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    seed_active_permit(&db, SATKER_B, NUP, KODE_BARANG, PEMEGANG_B).await;

    let res = post_create(&server, SATKER_B, NUP, KODE_BARANG_LAIN).await;
    assert_eq!(
        res.status_code(),
        201,
        "kode barang berbeda ditolak seolah aset yang sama: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Perpanjangan tak boleh bertabrakan dengan izinnya sendiri.
///
/// Pengecualian-diri dulu dilakukan dengan membandingkan `active_permit_id`
/// hasil query yang bisa teredaksi; sekarang di SQL.
#[tokio::test]
async fn renewing_an_active_permit_is_not_blocked_by_itself() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let id = seed_active_permit(&db, SATKER_A, NUP, KODE_BARANG, "Ani Pemegang").await;

    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/renew"))
        .json(&json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan pemakaian untuk tahun berikutnya"
        }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER_A) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        201,
        "izin menolak perpanjangannya sendiri: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// …tapi izin LAIN atas aset yang sama tetap memblokir perpanjangan.
#[tokio::test]
async fn renewal_is_blocked_by_another_active_permit_on_the_same_asset() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let id = seed_active_permit(&db, SATKER_A, NUP, KODE_BARANG, "Ani Pemegang").await;
    seed_active_permit(&db, SATKER_A, NUP, KODE_BARANG, "Cecep Pemegang Kedua").await;

    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/renew"))
        .json(&json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan pemakaian untuk tahun berikutnya"
        }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER_A) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        400,
        "perpanjangan lolos padahal aset yang sama sedang dipegang orang lain: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Endpoint ketersediaan menolak pertanyaan yang tak punya jawaban.
#[tokio::test]
async fn availability_refuses_a_question_keyed_on_nup_alone() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = get_as(
        &server,
        &format!("/pemakaian-bmn/bmn/{NUP}/availability"),
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "ketersediaan dijawab tanpa kode barang, padahal NUP saja bukan identitas: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Ketersediaan dijawab tentang aset MILIK PEMANGGIL, dan nama pemegang dari
/// satker lain tak pernah ikut.
#[tokio::test]
async fn availability_answers_about_the_callers_own_asset() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    seed_active_permit(&db, SATKER_B, NUP, KODE_BARANG, PEMEGANG_B).await;
    let path = format!("/pemakaian-bmn/bmn/{NUP}/availability?kode_barang={KODE_BARANG}");

    let res_a = get_as(&server, &path, SATKER_A).await;
    let body_a = res_a.text();
    assert_eq!(res_a.status_code(), 200, "satker A: {body_a}");
    assert!(
        body_a.contains("\"is_available\":true"),
        "aset satker A dilaporkan terpakai karena izin satker B:\n{body_a}"
    );
    assert!(
        !body_a.contains(PEMEGANG_B),
        "nama pemegang satker B bocor ke satker A:\n{body_a}"
    );

    let res_b = get_as(&server, &path, SATKER_B).await;
    let body_b = res_b.text();
    assert_eq!(res_b.status_code(), 200, "satker B: {body_b}");
    assert!(
        body_b.contains("\"is_available\":false") && body_b.contains(PEMEGANG_B),
        "satker B tidak melihat izin miliknya sendiri:\n{body_b}"
    );

    teardown_test_db(&db_name).await;
}

/// Menanyakan satker di luar scope dijawab 404, bukan 403.
///
/// 403 akan mengonfirmasi bahwa satker itu ada dan punya aset — oracle
/// keberadaan yang sama yang ditutup #93.
#[tokio::test]
async fn asking_about_a_satker_outside_the_scope_is_not_found() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    seed_active_permit(&db, SATKER_C, NUP, KODE_BARANG, "Cici Pemegang Satker C").await;

    let res = get_as(
        &server,
        &format!(
            "/pemakaian-bmn/bmn/{NUP}/availability?kode_barang={KODE_BARANG}&satker_code={SATKER_C}"
        ),
        SATKER_A,
    )
    .await;
    let body = res.text();
    assert_eq!(
        res.status_code(),
        404,
        "satker di luar scope dijawab selain 404: {body}"
    );
    assert!(
        !body.contains("Cici Pemegang Satker C"),
        "penolakan membocorkan nama pemegang:\n{body}"
    );

    teardown_test_db(&db_name).await;
}

/// Izin harus melekat pada satu satker; tanpa identitas satker, permohonan
/// ditolak alih-alih menulis baris yatim yang tak terlihat siapa pun.
#[tokio::test]
async fn a_caller_without_a_satker_identity_cannot_create_a_permit() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Peran yang MEMANG boleh membuat izin, tapi klaimnya tanpa kode satker —
    // supaya yang diuji penjaga satker, bukan penjaga peran.
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_body(NUP, KODE_BARANG));
    for (k, v) in auth_headers("operator_satker", OPERATOR, "") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        400,
        "izin tanpa identitas satker berhasil dibuat: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}
