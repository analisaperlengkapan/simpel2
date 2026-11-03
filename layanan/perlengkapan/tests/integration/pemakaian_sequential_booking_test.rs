//! Pemakaian BERURUTAN: izin baru atas aset yang sama boleh diajukan selama
//! periodenya tidak bertumpuk dengan izin `ACTIVE` yang ada.
//!
//! `cek-bmn` sudah lama menjanjikannya — "existing berakhir sebelum usulan
//! mulai → diizinkan" — tetapi jalur TULIS tak pernah mengetahuinya: ia
//! menolak setiap permohonan atas aset yang punya izin `ACTIVE`, apa pun
//! tanggalnya. Layar bilang boleh, lalu permohonannya ditolak. Terukur di
//! staging: izin `E2E-IZIN-001` atas aset X berlaku sampai 2026-12-31;
//! permohonan 2031-03-01..10 atas aset yang sama dijawab 400 "sedang digunakan
//! oleh … hingga 2026-12-31".
//!
//! Tanggal di sini TETAP, bukan relatif terhadap jam. Fixture lain berkas ini
//! menanam izin `CURRENT_DATE ± n` dan mengajukan periode 2026; selama jalur
//! tulis buta tanggal itu tak berarti, tetapi begitu periode menentukan, tes
//! semacam itu membusuk begitu kalender melewati 2026.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const SATKER: &str = "SKR001";
/// Pegawai milik SATKER di fixture `tests/common.rs`.
const NIP: &str = "19800101000000001";
const NUP: &str = "1";
const KODE_BARANG: &str = "3060201003";
const PEMEGANG: &str = "Budi Pemegang Izin Aktif";

/// Izin aktif: 2031-01-01 s.d. 2031-06-30 (inklusif).
const AKTIF_MULAI: &str = "2031-01-01";
const AKTIF_SELESAI: &str = "2031-06-30";

fn auth_headers() -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_static(OPERATOR),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_static("operator_satker"),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_static(SATKER),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::operator_satker::{OPERATOR}::{SATKER}"
            ))
            .unwrap(),
        ),
    ]
}

/// Satu izin ACTIVE dengan periode TETAP, langsung ke DB (`create` lewat HTTP
/// hanya menghasilkan DRAFT, sedangkan cek tabrakan mencari yang ACTIVE).
async fn seed_active_permit(db: &layanan_perlengkapan::shared::db::Database) -> Uuid {
    let client = db.pool().get().await.unwrap();
    let id = Uuid::new_v4();
    client
        .execute(
            "WITH izin AS (
                INSERT INTO perlengkapan.izin_pemakaian_bmn
                    (id, nomor_izin, pegawai_nip, pegawai_nama, pegawai_jabatan,
                     pegawai_satker_id, pegawai_satker_nama, satker_code,
                     tanggal_mulai, tanggal_selesai, status, approved_at)
                VALUES ($1, $2, $3, $4, 'Staf', gen_random_uuid(),
                        'KEJAKSAAN NEGERI UJI A', $5,
                        $6::text::date, $7::text::date, 'ACTIVE', NOW())
                RETURNING id
             )
             INSERT INTO perlengkapan.pemakaian_bmn_items
                (izin_pemakaian_id, urutan, jenis_bmn, bmn_nup, bmn_kode_barang,
                 bmn_nama_barang, tanggal_mulai, tanggal_selesai)
             SELECT id, 1, 'LAINNYA', $8, $9, 'Barang Uji', $6::text::date, $7::text::date
             FROM izin",
            &[
                &id,
                &format!("IZIN/BERURUTAN/{}", &id.simple().to_string()[..8]),
                &NIP,
                &PEMEGANG,
                &SATKER,
                &AKTIF_MULAI,
                &AKTIF_SELESAI,
                &NUP,
                &KODE_BARANG,
            ],
        )
        .await
        .unwrap();
    id
}

fn item(mulai: Option<&str>, selesai: Option<&str>) -> Value {
    let mut it = json!({
        "jenis_bmn": "LAINNYA",
        "bmn_nup": NUP,
        "bmn_kode_barang": KODE_BARANG,
        "bmn_nama_barang": "Barang Uji",
    });
    if let Some(m) = mulai {
        it["tanggal_mulai"] = json!(m);
    }
    if let Some(s) = selesai {
        it["tanggal_selesai"] = json!(s);
    }
    it
}

async fn create(
    server: &TestServer,
    mulai: &str,
    selesai: &str,
    item: Value,
) -> axum_test::TestResponse {
    let mut req = server.post("/pemakaian-bmn").json(&json!({
        "pegawai_nip": NIP,
        "tanggal_mulai": mulai,
        "tanggal_selesai": selesai,
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": [item],
    }));
    for (k, v) in auth_headers() {
        req = req.add_header(k, v);
    }
    req.await
}

async fn availability(server: &TestServer, query: &str) -> axum_test::TestResponse {
    let mut req = server.get(&format!(
        "/pemakaian-bmn/bmn/{NUP}/availability?kode_barang={KODE_BARANG}{query}"
    ));
    for (k, v) in auth_headers() {
        req = req.add_header(k, v);
    }
    req.await
}

// ─── jalur TULIS ────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_permit_that_starts_after_the_active_one_ends_is_accepted() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    // Hari pertama yang bebas: sehari setelah izin aktif berakhir.
    let res = create(&server, "2031-07-01", "2031-07-31", item(None, None)).await;
    assert_eq!(
        res.status_code(),
        201,
        "berurutan harus diterima, sebelumnya ditolak: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_permit_that_ends_before_the_active_one_starts_is_accepted() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    // Sehari sebelum izin aktif mulai: juga tak bertumpuk.
    let res = create(&server, "2030-01-01", "2030-12-31", item(None, None)).await;
    assert_eq!(res.status_code(), 201, "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_overlapping_permit_is_refused_and_says_who_holds_it_until_when() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    // Tiga bentuk tumpang tindih: ujung depan, ujung belakang, dan menelan
    // seluruh izin aktif. Hari terakhir izin aktif (2031-06-30) INKLUSIF —
    // kontrol dua arah dengan 2031-07-01 di tes pertama.
    for (mulai, selesai) in [
        ("2030-12-01", "2031-01-01"),
        ("2031-06-30", "2031-07-15"),
        ("2030-01-01", "2032-12-31"),
        ("2031-03-01", "2031-03-10"),
    ] {
        let res = create(&server, mulai, selesai, item(None, None)).await;
        assert_eq!(
            res.status_code(),
            400,
            "{mulai}..{selesai} bertumpuk dengan izin aktif dan harus ditolak: {}",
            res.text()
        );
        let pesan = res.text();
        assert!(
            pesan.contains(PEMEGANG),
            "pesan menyebut pemegangnya: {pesan}"
        );
        assert!(
            pesan.contains(AKTIF_SELESAI),
            "pesan menyebut sampai kapan: {pesan}"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_item_with_its_own_period_is_judged_on_that_period_not_the_permits() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    // Periode permohonan bebas, tetapi jangka khusus aset ini bertumpuk.
    let res = create(
        &server,
        "2031-07-01",
        "2031-07-31",
        item(Some("2031-06-01"), Some("2031-06-15")),
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "yang dinilai jangka milik aset itu sendiri: {}",
        res.text()
    );

    // Sebaliknya: periode permohonan bertumpuk, tetapi jangka khusus aset ini
    // bebas — yang menentukan adalah jangka aset itu.
    let res = create(
        &server,
        "2031-03-01",
        "2031-03-10",
        item(Some("2031-08-01"), Some("2031-08-10")),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());

    teardown_test_db(&db_name).await;
}

// ─── jalur BACA (yang dipakai form) ─────────────────────────────────────────

#[tokio::test]
async fn availability_without_a_period_still_reports_any_active_permit_as_in_use() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    let res = availability(&server, "").await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let d = res.json::<Value>()["data"].clone();
    // Perilaku lama, untuk klien yang belum mengirim periode.
    assert_eq!(d["is_available"], false);
    assert_eq!(d["status"], "OVERLAP");
    assert_eq!(d["active_permit_holder"], PEMEGANG);
    assert_eq!(d["active_permit_expires"], AKTIF_SELESAI);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn availability_for_a_period_tells_free_from_sequential_from_overlap() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    let ask = |q: &'static str| {
        let server = &server;
        async move {
            let res = availability(server, q).await;
            assert_eq!(res.status_code(), 200, "{q}: {}", res.text());
            res.json::<Value>()["data"].clone()
        }
    };

    // Sesudah izin aktif: boleh, dan pemegangnya disebut sebagai konteks.
    let d = ask("&tgl_mulai=2031-07-01&tgl_selesai=2031-07-31").await;
    assert_eq!(d["is_available"], true);
    assert_eq!(d["status"], "PEMAKAIAN_BERURUTAN");
    assert_eq!(d["active_permit_holder"], PEMEGANG);
    assert_eq!(d["active_permit_expires"], AKTIF_SELESAI);

    // Bertumpuk: ditolak.
    let d = ask("&tgl_mulai=2031-06-30&tgl_selesai=2031-07-15").await;
    assert_eq!(d["is_available"], false);
    assert_eq!(d["status"], "OVERLAP");

    // SEBELUM izin aktif: bebas, dan TIDAK disebut "berurutan" — izin itu tidak
    // mendahuluinya, jadi "mengikuti X sampai 2031-06-30" akan salah.
    let d = ask("&tgl_mulai=2030-01-01&tgl_selesai=2030-12-31").await;
    assert_eq!(d["is_available"], true);
    assert_eq!(d["status"], "AVAILABLE");
    assert!(d["active_permit_holder"].is_null(), "{d}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn availability_refuses_half_a_period_and_a_reversed_one() {
    let (app, db, db_name) = setup_test_app().await;
    seed_active_permit(&db).await;
    let server = TestServer::new(app);

    assert_eq!(
        availability(&server, "&tgl_mulai=2031-07-01")
            .await
            .status_code(),
        400,
        "satu tanggal saja tak berarti apa-apa"
    );
    assert_eq!(
        availability(&server, "&tgl_selesai=2031-07-01")
            .await
            .status_code(),
        400
    );
    assert_eq!(
        availability(&server, "&tgl_mulai=2031-08-01&tgl_selesai=2031-07-01")
            .await
            .status_code(),
        400,
        "selesai sebelum mulai"
    );

    teardown_test_db(&db_name).await;
}
