//! Pengisian ukuran pakaian dinas per satker, dari nol sampai diajukan.
//!
//! Tiga hal yang tidak pernah ada sebelum perubahan ini, dan yang tes ini jaga:
//!
//!   1. membuat kampanye juga membuat baris workflow per satker. Sebelumnya
//!      hanya `_satker_terpilih` yang terisi, sehingga kampanye baru lahir
//!      dengan daftar satker KOSONG — tidak ada yang bisa dibuka, diisi, atau
//!      diajukan. Komentar di `pakaian_dinas_workflow_test.rs` mencatat celah
//!      itu sebagai alasan ia menyemai barisnya sendiri;
//!   2. daftar pengisian yang menggabungkan kolom kampanye, daftar pegawai
//!      dari kepegawaian, dan ukuran yang sudah tersimpan;
//!   3. jalur tulisnya — tak ada satu pun kode yang pernah menulis
//!      `pengajuan_pakaian_dinas_satker_pegawai` maupun tabel ukurannya.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";
pub(crate) const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
pub(crate) const PUSAT: &str = "00000000-0000-0000-0000-000000000002";
pub(crate) const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
pub(crate) const NIP_A: &str = "19800101000000001";

pub(crate) fn headers(role: &str, user: &str, satker: &str) -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_str(satker).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::{}::{}",
                role, user, satker
            ))
            .unwrap(),
        ),
    ]
}

pub(crate) async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
    body: Value,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

pub(crate) async fn get(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

pub(crate) async fn put(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
    body: Value,
) -> axum_test::TestResponse {
    let mut req = server.put(path).json(&body);
    for (k, v) in headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

pub(crate) async fn delete(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
) -> axum_test::TestResponse {
    let mut req = server.delete(path);
    for (k, v) in headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// Kampanye PDH dengan satu kolom BAJU, menyasar SKR001 dan SKR002.
/// Mengembalikan `(pengajuan_id, pakaian_id)`.
/// A campaign window that contains today, whenever the test runs. The dates are
/// enforced (nothing is filled or decided outside them), so a fixed year would
/// turn every test that drives a campaign red on 1 January.
pub(crate) fn periode_berjalan() -> (String, String) {
    let hari = chrono::Utc::now().date_naive();
    (
        (hari - chrono::Duration::days(30)).to_string(),
        (hari + chrono::Duration::days(300)).to_string(),
    )
}

pub(crate) async fn buat_kampanye(server: &TestServer) -> (String, String) {
    let (mulai, selesai) = periode_berjalan();
    buat_kampanye_periode(server, &mulai, &selesai).await
}

/// [`buat_kampanye`] with an explicit window, for the tests about the window.
pub(crate) async fn buat_kampanye_periode(
    server: &TestServer,
    mulai: &str,
    selesai: &str,
) -> (String, String) {
    let res = post(
        server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        SATKER_A,
        json!({"nama": "PDH", "deskripsi": "Pakaian Dinas Harian", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "buat jenis: {:?}", res.text());
    let jenis_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = post(
        server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        SATKER_A,
        json!({
            "jenis_pakaian_dinas_id": jenis_id,
            "nama": "Pakaian Dinas",
            "gender": "SEMUA",
            "ukuran_group": "BAJU",
            "is_active": true
        }),
    )
    .await;
    assert_eq!(res.status_code(), 201, "buat spesifikasi: {:?}", res.text());
    let spesifikasi_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = post(
        server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({
            "nama": "Kebutuhan Pakaian Dinas PDH 2026",
            "tgl_mulai": mulai,
            "tgl_selesai": selesai,
            "is_reguler": true,
            "tahun": 2026,
            "pilihan_satker": "sebagian",
            "jenis_pakaian_dinas_id": jenis_id,
            "spesifikasi_ids": [spesifikasi_id],
            "satker_ids": [SATKER_A, SATKER_B]
        }),
    )
    .await;
    assert_eq!(res.status_code(), 201, "buat pengajuan: {:?}", res.text());
    let pengajuan_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Kolomnya diambil dari daftar pengisian, bukan ditebak dari spesifikasi:
    // `pengajuan_pakaian_dinas_pakaian.id` adalah kunci yang dipakai saat
    // menyimpan ukuran, dan ia BUKAN `spesifikasi_id`.
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "roster: {:?}", res.text());
    let pakaian_id = res.json::<Value>()["data"]["pakaian"][0]["id"]
        .as_str()
        .expect("kampanye harus membawa minimal satu kolom pakaian")
        .to_string();

    (pengajuan_id, pakaian_id)
}

#[tokio::test]
async fn kampanye_baru_langsung_punya_baris_satker_yang_bisa_dibuka() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _) = buat_kampanye(&server).await;

    // Inilah cacat yang ditutup: sebelumnya daftar ini kosong meski
    // `total_satker` melaporkan 2, jadi tak ada satker yang bisa dibuka.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "daftar satker: {:?}", res.text());
    let body = res.json::<Value>();
    let rows = body["data"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "kedua satker sasaran harus punya barisnya");
    for row in rows {
        assert_eq!(
            row["aktivitas_id"].as_i64(),
            Some(1000),
            "satker mulai pada status Input agar operatornya bisa mengisi"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn daftar_pengisian_memuat_kolom_kampanye_dan_pegawai_kepegawaian() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "roster: {:?}", res.text());
    let data = res.json::<Value>()["data"].clone();

    assert_eq!(data["satker_kode"].as_str(), Some(SATKER_A));
    assert_eq!(
        data["dapat_diubah"].as_bool(),
        Some(true),
        "status Input harus bisa diisi"
    );
    assert_eq!(
        data["pakaian"][0]["spesifikasi_ukuran_group"].as_str(),
        Some("BAJU"),
        "grup ukuran menentukan daftar ukuran yang ditawarkan"
    );
    assert!(!pakaian_id.is_empty());

    // Orangnya datang dari kepegawaian, bukan dari tabel pengajuan: pegawai
    // yang belum pernah diisi tetap harus muncul, kalau tidak tak ada cara
    // menambahkannya.
    let pegawai = data["pegawai"].as_array().unwrap();
    let a = pegawai
        .iter()
        .find(|p| p["nip"].as_str() == Some(NIP_A))
        .expect("pegawai satker harus muncul walau belum diisi");
    assert_eq!(a["sudah_diisi"].as_bool(), Some(false));
    assert!(a["ukuran"].as_array().unwrap().is_empty());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_tersimpan_per_pegawai_dan_terbaca_kembali() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}");

    let res = put(
        &server,
        &path,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "simpan ukuran: {:?}", res.text());

    // Respons menyertakan daftar yang sudah diperbarui supaya layar tidak
    // perlu menebak keadaan barunya.
    let data = res.json::<Value>()["data"].clone();
    let a = data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"].as_str() == Some(NIP_A))
        .unwrap()
        .clone();
    assert_eq!(a["sudah_diisi"].as_bool(), Some(true));
    assert_eq!(a["ukuran"][0]["ukuran"].as_str(), Some("L"));
    assert_eq!(
        a["ukuran"][0]["pakaian_id"].as_str(),
        Some(pakaian_id.as_str())
    );

    // Menyimpan ulang MENGGANTI, tidak menambah — kalau tidak, ukuran lama
    // bertahan di laporan setelah operator mengubahnya.
    let res = put(
        &server,
        &path,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "XL"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200);
    let a = res.json::<Value>()["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"].as_str() == Some(NIP_A))
        .unwrap()
        .clone();
    assert_eq!(
        a["ukuran"].as_array().unwrap().len(),
        1,
        "ukuran diganti, bukan ditumpuk"
    );
    assert_eq!(a["ukuran"][0]["ukuran"].as_str(), Some("XL"));

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn satker_lain_tidak_terbaca_dan_tidak_tertulis() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    // NotFound, bukan Forbidden: 403 akan mengonfirmasi barisnya ada di satker
    // lain — oracle keberadaan lintas-satker yang ditutup #93.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "roster satker lain tidak boleh terbaca: {:?}",
        res.text()
    );

    let res = put(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai/19800101000000002"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "menulis ke satker lain tidak boleh: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_dari_kampanye_lain_ditolak() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _) = buat_kampanye(&server).await;

    // `pakaian_id` acak tidak dimiliki kampanye ini. Tanpa penjagaan, baris
    // ukuran tersimpan dan muncul di laporan kampanye yang keliru.
    let asing = uuid::Uuid::new_v4().to_string();
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": asing, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "kolom di luar kampanye harus ditolak: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn setelah_diajukan_ukuran_tidak_dapat_diubah_lagi() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}");

    let res = put(
        &server,
        &path,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200);

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let satker_row_id = res.json::<Value>()["data"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": satker_row_id, "aksi": "submit"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    // Penolakannya harus datang dari SERVER, bukan sekadar tombol yang mati.
    let res = put(
        &server,
        &path,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "XL"}]}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "satker yang sudah diajukan tidak boleh diubah: {:?}",
        res.text()
    );

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.json::<Value>()["data"]["dapat_diubah"].as_bool(),
        Some(false),
        "layar harus tahu bahwa isinya terkunci"
    );

    teardown_test_db(&db_name).await;
}

/// Kampanye nasional ("semua"): setiap satker MySIMKARI mendapat barisnya. Staging
/// membawa 545 satker, dan layar daftar hanya memuat 100 pertama tanpa cara menjangkau sisanya;
/// laporan menyatakan "Belum ada satker yang ditetapkan" karena `total_satker` menghitung
/// `_satker_terpilih`, yang KOSONG untuk kampanye semacam ini. Tes ini menjaga sisi server
/// keduanya: hitungan satker peserta, halaman yang tak saling tumpang, pencarian, dan
/// penyaring status.
#[tokio::test]
async fn kampanye_nasional_dihitung_utuh_dan_daftarnya_bisa_dihalamani_dan_dicari() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (mulai, selesai) = periode_berjalan();

    let res = post(
        &server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        SATKER_A,
        json!({"nama": "PDH Nasional", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "jenis: {:?}", res.text());
    let jenis_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        SATKER_A,
        json!({"jenis_pakaian_dinas_id": jenis_id, "nama": "Baju Nasional",
               "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "spesifikasi: {:?}", res.text());
    let spesifikasi_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({"nama": "Kampanye nasional", "tgl_mulai": mulai, "tgl_selesai": selesai,
               "is_reguler": true, "tahun": 2026, "pilihan_satker": "semua",
               "jenis_pakaian_dinas_id": jenis_id, "spesifikasi_ids": [spesifikasi_id]}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "pengajuan: {:?}", res.text());
    let pengajuan_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let daftar = |query: &'static str| {
        let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker{query}");
        let server = &server;
        async move { get(server, &path, "validator_pusat", PUSAT, SATKER_A).await }
    };
    let kode = |body: &Value| -> Vec<String> {
        body["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["satker_id"].as_str().unwrap().to_string())
            .collect()
    };

    // 1. Seluruh satker MySIMKARI jadi peserta, dan itulah yang dihitung laporan.
    let res = daftar("?per_page=100").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let semua = res.json::<Value>();
    let jumlah = semua["total"].as_i64().unwrap();
    assert!(
        jumlah >= 5,
        "kampanye nasional harus memuat semua satker: {jumlah}"
    );

    let res = get(
        &server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let kampanye = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_str() == Some(pengajuan_id.as_str()))
        .cloned()
        .expect("kampanye ada di daftarnya");
    assert_eq!(
        kampanye["total_satker"].as_i64(),
        Some(jumlah),
        "total_satker menghitung peserta, bukan `_satker_terpilih` yang kosong untuk 'semua'"
    );

    // 2. Halaman: tidak saling tumpang, urutannya tetap, dan gabungannya utuh.
    let mut dari_halaman = Vec::new();
    for page in 1..=3 {
        let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker?page={page}&per_page=2");
        let res = get(&server, &path, "validator_pusat", PUSAT, SATKER_A).await;
        assert_eq!(res.status_code(), 200, "{:?}", res.text());
        let body = res.json::<Value>();
        assert_eq!(body["total"].as_i64(), Some(jumlah));
        dari_halaman.extend(kode(&body));
    }
    assert_eq!(
        dari_halaman.len(),
        5.min(jumlah as usize),
        "tiga halaman dua-dua"
    );
    let mut unik = dari_halaman.clone();
    unik.sort();
    unik.dedup();
    assert_eq!(
        unik.len(),
        dari_halaman.len(),
        "halaman tidak boleh tumpang: {dari_halaman:?}"
    );
    assert_eq!(
        kode(&semua)[..dari_halaman.len()],
        dari_halaman[..],
        "urutan sama di tiap ukuran halaman"
    );

    // 3. Pencarian: nama dan kode, tanpa peduli huruf besar; `%` yang diketik adalah teks.
    let res = daftar("?q=uji%20dua").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    assert_eq!(kode(&body), vec!["KJT02".to_string()], "{body:?}");
    assert_eq!(body["total"].as_i64(), Some(1), "total mengikuti pencarian");
    let res = daftar("?q=skr003").await;
    assert_eq!(kode(&res.json::<Value>()), vec!["SKR003".to_string()]);
    let res = daftar("?q=%25").await;
    assert_eq!(res.status_code(), 200);
    assert!(
        kode(&res.json::<Value>()).is_empty(),
        "`%` bukan 'apa saja'"
    );

    // 4. Status: semua baru lahir di Input (1000); tak ada yang Selesai (1008).
    let res = daftar("?status=1000&per_page=100").await;
    assert_eq!(res.json::<Value>()["total"].as_i64(), Some(jumlah));
    let res = daftar("?status=1001,1008").await;
    assert_eq!(res.json::<Value>()["total"].as_i64(), Some(0));
    let res = daftar("?status=abc").await;
    assert_eq!(
        res.status_code(),
        400,
        "bukan angka = permintaan salah: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Aksi validator masuk jejak audit DENGAN pelakunya. Sebelumnya kejadiannya dibuat tanpa aktor sama
/// sekali (`actor_id` dan `actor_name` null), sehingga halaman Audit Log yang menjanjikan "siapa, apa,
/// kapan" tak punya jawaban untuk persetujuan pakaian dinas; satu-satunya jejak pelakunya ada di baris
/// aktivitas per satker.
#[tokio::test]
async fn aksi_validator_pakaian_dinas_tercatat_di_jejak_audit_beserta_pelakunya() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}");

    let res = put(
        &server,
        &path,
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
    let satker_row_id = res.json::<Value>()["data"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": satker_row_id, "aksi": "submit"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    let res = get(
        &server,
        "/audit?module=pakaian_dinas&per_page=50",
        "admin",
        ADMIN,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    let entry = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["action_name"].as_str() == Some("pakaian_dinas.validator_action"))
        .unwrap_or_else(|| panic!("aksi validator harus tercatat: {body:?}"));

    assert_eq!(
        entry["actor_id"].as_str(),
        Some(OPERATOR),
        "tanpa pelaku, jejak audit tak bisa menjawab siapa yang bertindak: {entry:?}"
    );
    let nama = entry["actor_name"].as_str().unwrap_or_default();
    assert!(
        !nama.trim().is_empty() && nama != "unknown",
        "nama pelaku harus terbaca manusia, bukan penanda kosong: {entry:?}"
    );

    teardown_test_db(&db_name).await;
}

/// The confirmation dialogs need counts the client cannot compute — it only
/// holds one page of the satker list. This pins the two questions they ask:
/// "how many employees of mine are still unfilled?" and "which satker are still
/// behind me?".
#[tokio::test]
async fn kesiapan_menghitung_yang_belum_mengisi_dan_yang_belum_mengajukan() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _) = buat_kampanye(&server).await;

    // Pusat: both satker are still at Input, so neither has asked the wilayah
    // validator for anything yet.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=pusat"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan pusat: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["tahap"].as_str(), Some("pusat"));
    assert_eq!(k["total"].as_i64(), Some(2), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(2), "{k:?}");
    // Nothing is being moved, so the "rows being moved" group is empty.
    assert_eq!(k["fokus_satker"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // The operator's own gate counts employees, not satker.
    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=satker&satker_code={SATKER_A}"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan satker: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // Submit SKR001 without filling anybody: allowed (the dialog only warns),
    // and from here the row counts as "submitted but incomplete".
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let row_id = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"].as_str() == Some(SATKER_A))
        .and_then(|r| r["id"].as_str())
        .expect("baris SKR001")
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": row_id, "aksi": "submit"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    // A bulk move at pusat names the row it is moving (`terpilih`). That row is
    // reported on its own (it was submitted with nothing filled in), and left out
    // of "who is behind": the remaining satker is still at Input.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=pusat&terpilih={row_id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan pusat: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_satker"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_belum_lengkap"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_pegawai_belum"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_pegawai_total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // `kecuali` is the old name of `terpilih`: a front end cached from before the
    // rename must get the same answer.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=pusat&kecuali={row_id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "alias kecuali: {:?}", res.text());
    assert_eq!(
        res.json::<Value>()["data"]["fokus_satker"].as_i64(),
        Some(1),
        "alias kecuali"
    );

    // Wilayah, nothing being moved: SKR001 was submitted (so it is no longer
    // "behind"), SKR002 has not submitted at all.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah"),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan wilayah: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(2), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_satker"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // A bulk move names EVERY selected row: with both participation rows named,
    // nobody is left behind (`total` 0), but the two rows being moved are both
    // still empty and the answer says so. This is the comma list the
    // mass-action bar sends.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    let rows = res.json::<Value>()["data"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let other_id = rows
        .iter()
        .find(|r| r["satker_id"].as_str() == Some(SATKER_B))
        .and_then(|r| r["id"].as_str())
        .expect("baris SKR002")
        .to_string();
    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&terpilih={row_id},{other_id}"
        ),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan massal: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["fokus_satker"].as_i64(), Some(2), "{k:?}");
    assert_eq!(k["fokus_belum_lengkap"].as_i64(), Some(2), "{k:?}");
    assert_eq!(k["fokus_pegawai_belum"].as_i64(), Some(2), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // A malformed entry is a 400, not a silently dropped row.
    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&terpilih=bukan-uuid"
        ),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "terpilih tidak valid: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

// ============================================================================
// Readiness must count PEOPLE, not rows. The four tests below pin the cases
// where the old "count participation rows" definition lied: an empty form
// saved, an employee the operator removed, an employee who transferred out of
// a legacy (unsnapshotted) campaign, and a satker with nobody to fill in.
// ============================================================================

const NIP_D: &str = "19800101000000004";
const NIP_E: &str = "19800101000000005";

async fn tambah_pegawai(
    db: &layanan_perlengkapan::shared::db::Database,
    nip: &str,
    satker_api_id: &str,
    nama: &str,
    jk: &str,
) {
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, jk)
             VALUES ($1, $2, $3, 'SATKER UJI', 'Staf', $4)
             ON CONFLICT (nip) DO UPDATE SET satker_id = EXCLUDED.satker_id",
            &[&nip, &nama, &satker_api_id, &jk],
        )
        .await
        .unwrap();
}

async fn simpan_ukuran(
    server: &TestServer,
    pengajuan_id: &str,
    nip: &str,
    pakaian_id: &str,
    ukuran: &str,
) {
    let res = put(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{nip}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": ukuran}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "simpan {nip}: {:?}", res.text());
}

async fn simpan_kosong(server: &TestServer, pengajuan_id: &str, nip: &str) {
    let res = put(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{nip}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": []}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        200,
        "simpan kosong {nip}: {:?}",
        res.text()
    );
}

async fn roster(server: &TestServer, pengajuan_id: &str) -> Value {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "roster: {:?}", res.text());
    res.json::<Value>()["data"].clone()
}

async fn kesiapan_satker(server: &TestServer, pengajuan_id: &str) -> Value {
    let res = get(
        server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=satker&satker_code={SATKER_A}"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan satker: {:?}", res.text());
    res.json::<Value>()["data"].clone()
}

#[tokio::test]
async fn ukuran_kosong_tidak_dihitung_sudah_mengisi() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    tambah_pegawai(&db, NIP_D, "api-skr001", "Pegawai Uji D", "P").await;
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    // The operator opens D's form and saves it without choosing anything. The
    // row is created; no size is.
    simpan_kosong(&server, &pengajuan_id, NIP_D).await;

    let data = roster(&server, &pengajuan_id).await;
    let d = data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"].as_str() == Some(NIP_D))
        .expect("D ada di roster");
    assert_eq!(
        d["sudah_diisi"].as_bool(),
        Some(false),
        "formulir kosong bukan isian: {d:?}"
    );
    assert!(d["ukuran"].as_array().unwrap().is_empty(), "{d:?}");

    let k = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(k["total"].as_i64(), Some(2), "{k:?}");
    assert_eq!(
        k["pegawai_belum_diisi"].as_i64(),
        Some(1),
        "D belum mengisi walau barisnya ada: {k:?}"
    );
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pegawai_yang_dikeluarkan_tidak_lagi_dihitung_belum_mengisi() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    tambah_pegawai(&db, NIP_D, "api-skr001", "Pegawai Uji D", "P").await;
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    simpan_ukuran(&server, &pengajuan_id, NIP_D, &pakaian_id, "M").await;
    assert_eq!(
        kesiapan_satker(&server, &pengajuan_id).await["pegawai_belum_diisi"].as_i64(),
        Some(0)
    );

    // D does not need a uniform: the operator removes them.
    let mut req = server.delete(&format!(
        "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}"
    ));
    for (k, v) in headers("operator_satker", OPERATOR, SATKER_A) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "keluarkan: {:?}", res.text());

    // Still on the list, flagged — that is what makes "Masukkan kembali"
    // reachable. Hiding the row was the bug: the way back was a size form the
    // list no longer offered.
    let data = roster(&server, &pengajuan_id).await;
    let d = pegawai_dari_roster(&data, NIP_D);
    assert_eq!(d["dikeluarkan"].as_bool(), Some(true), "{d:?}");
    assert_eq!(d["sudah_diisi"].as_bool(), Some(false), "{d:?}");

    let k = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(
        k["total"].as_i64(),
        Some(1),
        "D keluar dari penyebut: {k:?}"
    );
    assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(true), "{k:?}");

    // Putting them back (or saving a size for them) restores the debt.
    let res = delete(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}/keluarkan"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    simpan_ukuran(&server, &pengajuan_id, NIP_D, &pakaian_id, "M").await;
    let k = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(k["total"].as_i64(), Some(2), "kembali masuk: {k:?}");
    assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(0), "{k:?}");

    teardown_test_db(&db_name).await;
}

/// A legacy campaign (no snapshot) reads the live roster. A size saved by an
/// employee who has since transferred must not stand in for their replacement:
/// the counts are matched by NIP, not "rows vs people".
#[tokio::test]
async fn ukuran_pegawai_yang_pindah_tidak_menutupi_pengganti() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    // Make it a legacy campaign: no frozen roster, so the live branch applies.
    let pid = uuid::Uuid::parse_str(&pengajuan_id).unwrap();
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "UPDATE perlengkapan.pengajuan_pakaian_dinas
             SET roster_snapshot_at = NULL WHERE id = $1",
            &[&pid],
        )
        .await
        .unwrap();
    client
        .execute(
            "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot
             WHERE pengajuan_id = $1",
            &[&pid],
        )
        .await
        .unwrap();

    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let k = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(0), "{k:?}");

    // A transfers out; E transfers in. The satker still has one person.
    client
        .execute(
            "UPDATE integrasi.mysimkari_pegawai SET satker_id = 'api-skr002' WHERE nip = $1",
            &[&NIP_A],
        )
        .await
        .unwrap();
    tambah_pegawai(&db, NIP_E, "api-skr001", "Pegawai Uji E", "L").await;

    let data = roster(&server, &pengajuan_id).await;
    let nips: Vec<&str> = data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["nip"].as_str())
        .collect();
    assert_eq!(nips, vec![NIP_E], "roster hidup: hanya yang ada sekarang");

    let k = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(
        k["pegawai_belum_diisi"].as_i64(),
        Some(1),
        "ukuran A tidak boleh menutupi E: {k:?}"
    );
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    teardown_test_db(&db_name).await;
}

/// A satker with nobody in the roster cannot be submitted (the button needs at
/// least one size), so counting it as "belum mengajukan" made the warning — and
/// `lengkap` — unreachable forever.
#[tokio::test]
async fn satker_tanpa_pegawai_tidak_menahan_peringatan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "DELETE FROM integrasi.mysimkari_pegawai WHERE satker_id = 'api-skr002'",
            &[],
        )
        .await
        .unwrap();
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah"),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(2), "{k:?}");
    assert_eq!(
        k["belum_mengajukan"].as_i64(),
        Some(1),
        "hanya satker yang punya pegawai: {k:?}"
    );
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // SKR001 fills in and submits; the empty satker does not hold the gate open.
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    let row_id = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"].as_str() == Some(SATKER_A))
        .and_then(|r| r["id"].as_str())
        .unwrap()
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": row_id, "aksi": "submit", "tahap": "satker"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah"),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(true), "{k:?}");

    teardown_test_db(&db_name).await;
}

/// The id of the participation row of `satker` in the campaign.
async fn baris_satker(server: &TestServer, pengajuan_id: &str, satker: &str) -> String {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"].as_str() == Some(satker))
        .and_then(|r| r["id"].as_str())
        .unwrap_or_else(|| panic!("baris {satker}"))
        .to_string()
}

/// Moving ONE satker on (the single-satker action of a wilayah or pusat
/// validator) is only about THAT satker. Whether some other satker has
/// submitted, or has not, must not appear in what it is warned about.
#[tokio::test]
async fn kesiapan_satker_tunggal_hanya_menyangkut_satker_itu() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let satker_a = format!(
        "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=satker&satker_code={SATKER_A}"
    );

    // SKR001's one employee has no size; SKR002 has not submitted either. The
    // validator moving SKR001 is told about SKR001 and nothing else.
    for (rol, user) in [("validator_wilayah", PUSAT), ("validator_pusat", PUSAT)] {
        let res = get(&server, &satker_a, rol, user, SATKER_A).await;
        assert_eq!(res.status_code(), 200, "{rol}: {:?}", res.text());
        let k = res.json::<Value>()["data"].clone();
        assert_eq!(k["tahap"].as_str(), Some("satker"), "{k:?}");
        assert_eq!(k["total"].as_i64(), Some(1), "{rol}: {k:?}");
        assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(1), "{rol}: {k:?}");
        assert_eq!(k["lengkap"].as_bool(), Some(false), "{rol}: {k:?}");
        assert_eq!(k["belum_mengajukan"].as_i64(), Some(0), "{rol}: {k:?}");
        assert_eq!(k["belum_diteruskan"].as_i64(), Some(0), "{rol}: {k:?}");
    }

    // Once SKR001 is filled it is complete, although SKR002 is still untouched:
    // that is not SKR001's concern.
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let res = get(&server, &satker_a, "validator_wilayah", PUSAT, SATKER_A).await;
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["pegawai_belum_diisi"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(true), "{k:?}");

    teardown_test_db(&db_name).await;
}

/// A bulk move answers two separate questions: how complete the rows being
/// moved are, and who is left behind. Moving a complete row while another satker
/// has not submitted reports the second, not the first.
#[tokio::test]
async fn kesiapan_massal_memisahkan_yang_dipindah_dari_yang_tertinggal() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    // SKR001 is filled in and submitted; SKR002 is untouched.
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let row_a = baris_satker(&server, &pengajuan_id, SATKER_A).await;
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": row_a, "aksi": "submit", "tahap": "satker"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&terpilih={row_a}"),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    // What is being moved is complete...
    assert_eq!(k["fokus_satker"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["fokus_belum_lengkap"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["fokus_pegawai_belum"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["fokus_pegawai_total"].as_i64(), Some(1), "{k:?}");
    // ...but SKR002 has not submitted, and that alone makes the move "not complete".
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // Moving SKR002 (never submitted, nothing filled) instead: the focus group
    // is the incomplete one, and nobody is left behind below.
    let row_b = baris_satker(&server, &pengajuan_id, SATKER_B).await;
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&terpilih={row_b}"),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["fokus_belum_lengkap"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    teardown_test_db(&db_name).await;
}

// ============================================================================
// Monitoring: progress of the campaign per level.
// ============================================================================

/// The id of the participation row of `satker` in the campaign.
async fn id_baris_progres(server: &TestServer, pengajuan_id: &str, satker: &str) -> String {
    let data = daftar_satker_progres(server, pengajuan_id).await;
    data["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["satker_id"].as_str() == Some(satker))
        .and_then(|r| r["id"].as_str())
        .unwrap_or_else(|| panic!("baris {satker}: {data}"))
        .to_string()
}

async fn daftar_satker_progres(server: &TestServer, pengajuan_id: &str) -> Value {
    get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await
    .json::<Value>()
}

async fn progres(
    server: &TestServer,
    pengajuan_id: &str,
    role: &str,
    user: &str,
    satker: &str,
) -> Value {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/progres"),
        role,
        user,
        satker,
    )
    .await;
    assert_eq!(res.status_code(), 200, "progres: {:?}", res.text());
    res.json::<Value>()["data"].clone()
}

/// The monitoring answer for a whole campaign: the summary by stage, the table per
/// wilayah and the satker furthest behind, all from the same roster and the same
/// "filled" the warnings use.
#[tokio::test]
async fn progres_kegiatan_menyusun_ringkasan_wilayah_dan_yang_tertinggal() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    // Nothing has happened: both satker are at Input, nobody has filled anything.
    let k = progres(&server, &pengajuan_id, "validator_pusat", PUSAT, SATKER_A).await;
    let r = &k["ringkasan"];
    assert_eq!(r["satker_total"].as_i64(), Some(2), "{k}");
    assert_eq!(r["belum_mengajukan"].as_i64(), Some(2), "{k}");
    assert_eq!(r["di_wilayah"].as_i64(), Some(0), "{k}");
    assert_eq!(r["selesai"].as_i64(), Some(0), "{k}");
    assert_eq!(r["pegawai_total"].as_i64(), Some(2), "{k}");
    assert_eq!(r["pegawai_terisi"].as_i64(), Some(0), "{k}");

    // SKR001 fills in and submits; SKR002 is untouched.
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let row_a = id_baris_progres(&server, &pengajuan_id, SATKER_A).await;
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"pengajuan_satker_id": row_a, "aksi": "submit", "tahap": "satker"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "ajukan: {:?}", res.text());

    let k = progres(&server, &pengajuan_id, "validator_pusat", PUSAT, SATKER_A).await;
    let r = &k["ringkasan"];
    assert_eq!(r["belum_mengajukan"].as_i64(), Some(1), "{k}");
    assert_eq!(r["di_wilayah"].as_i64(), Some(1), "{k}");
    assert_eq!(r["pegawai_terisi"].as_i64(), Some(1), "{k}");
    assert_eq!(r["pegawai_total"].as_i64(), Some(2), "{k}");

    // The wilayah table adds up to the summary, whichever way the satker group.
    let per = k["per_wilayah"].as_array().unwrap();
    assert!(!per.is_empty(), "{k}");
    let jumlah = |kunci: &str| {
        per.iter()
            .map(|w| w[kunci].as_i64().unwrap_or(0))
            .sum::<i64>()
    };
    assert_eq!(jumlah("satker_total"), 2, "{k}");
    assert_eq!(jumlah("pegawai_total"), 2, "{k}");
    assert_eq!(jumlah("pegawai_terisi"), 1, "{k}");

    // Furthest behind: the untouched satker leads, the filled one follows.
    let t = k["tertinggal"].as_array().unwrap();
    assert_eq!(t.len(), 2, "{k}");
    assert_eq!(t[0]["satker_id"].as_str(), Some(SATKER_B), "{k}");
    assert_eq!(t[0]["pegawai_terisi"].as_i64(), Some(0), "{k}");
    assert_eq!(t[1]["satker_id"].as_str(), Some(SATKER_A), "{k}");

    teardown_test_db(&db_name).await;
}

/// Monitoring stays inside the caller's scope: an operator sees their own satker,
/// not the campaign.
#[tokio::test]
async fn progres_operator_hanya_melihat_satkernya_sendiri() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _) = buat_kampanye(&server).await;

    let k = progres(
        &server,
        &pengajuan_id,
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(k["ringkasan"]["satker_total"].as_i64(), Some(1), "{k}");
    let tertinggal = k["tertinggal"].as_array().unwrap();
    assert!(
        tertinggal
            .iter()
            .all(|s| s["satker_id"].as_str() == Some(SATKER_A)),
        "{k}"
    );

    teardown_test_db(&db_name).await;
}

/// The satker list carries the denominator behind each card ("12 dari 15"), counted
/// the way the warnings count it: excluded people are out, an empty save is not
/// "filled".
#[tokio::test]
async fn daftar_satker_membawa_roster_dan_yang_sudah_terisi() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    let isian = |data: &Value, satker: &str| -> (Option<i64>, Option<i64>) {
        let r = data["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["satker_id"].as_str() == Some(satker))
            .unwrap_or_else(|| panic!("baris {satker}: {data}"))
            .clone();
        (r["pegawai_roster"].as_i64(), r["pegawai_terisi"].as_i64())
    };
    let sebelum = daftar_satker_progres(&server, &pengajuan_id).await;
    assert_eq!(isian(&sebelum, SATKER_A), (Some(1), Some(0)), "{sebelum}");

    // Saving an empty form is not "filled".
    simpan_kosong(&server, &pengajuan_id, NIP_A).await;
    let kosong = daftar_satker_progres(&server, &pengajuan_id).await;
    assert_eq!(isian(&kosong, SATKER_A), (Some(1), Some(0)), "{kosong}");

    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;
    let sesudah = daftar_satker_progres(&server, &pengajuan_id).await;
    assert_eq!(isian(&sesudah, SATKER_A), (Some(1), Some(1)), "{sesudah}");
    assert_eq!(isian(&sesudah, SATKER_B), (Some(1), Some(0)), "{sesudah}");

    teardown_test_db(&db_name).await;
}

/// The campaign list says how far each campaign has got, inside the caller's scope: pusat sees
/// the whole campaign, an operator only their own satker.
#[tokio::test]
async fn daftar_kegiatan_membawa_progres_menurut_cakupan_pemanggil() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;

    let progres_dari = |data: &Value| -> Value {
        data["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k["id"].as_str() == Some(pengajuan_id.as_str()))
            .unwrap_or_else(|| panic!("kegiatan {pengajuan_id}: {data}"))["progres"]
            .clone()
    };

    let res = get(
        &server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let p = progres_dari(&res.json::<Value>());
    assert_eq!(p["satker_total"].as_i64(), Some(2), "{p}");
    assert_eq!(p["belum_mengajukan"].as_i64(), Some(2), "{p}");
    assert_eq!(p["pegawai_total"].as_i64(), Some(2), "{p}");
    assert_eq!(p["pegawai_terisi"].as_i64(), Some(1), "{p}");

    let res = get(
        &server,
        "/pakaian-dinas/pengajuan",
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let p = progres_dari(&res.json::<Value>());
    assert_eq!(p["satker_total"].as_i64(), Some(1), "{p}");
    assert_eq!(p["pegawai_terisi"].as_i64(), Some(1), "{p}");

    teardown_test_db(&db_name).await;
}

// ============================================================================
// "Keluarkan dari Pengajuan" must be reversible from the screen. Excluding used
// to delete the employee from the list, and the only documented way back was
// "fill their size again" — a form the list no longer offered.
// ============================================================================

fn pegawai_dari_roster(data: &Value, nip: &str) -> Value {
    data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == nip)
        .unwrap_or_else(|| panic!("pegawai {nip} tidak ada di roster: {data}"))
        .clone()
}

#[tokio::test]
async fn pegawai_yang_dikeluarkan_tetap_terdaftar_dan_bisa_dimasukkan_kembali() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    tambah_pegawai(&db, NIP_D, "api-skr001", "Pegawai Uji D", "P").await;
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;

    // D has no size yet: two people owe one.
    let data = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(data["pegawai_belum_diisi"], 1, "{data}");

    let res = delete(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    // Still listed, flagged, and no longer owed a size.
    let data = roster(&server, &pengajuan_id).await;
    let d = pegawai_dari_roster(&data, NIP_D);
    assert_eq!(d["dikeluarkan"], true, "{d}");
    assert_eq!(d["sudah_diisi"], false, "{d}");
    let kesiapan = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(kesiapan["pegawai_belum_diisi"], 0, "{kesiapan}");

    // Put back: listed as active again and counted as not filled.
    let res = delete(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}/keluarkan"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let data = roster(&server, &pengajuan_id).await;
    let d = pegawai_dari_roster(&data, NIP_D);
    assert_eq!(d["dikeluarkan"], false, "{d}");
    let kesiapan = kesiapan_satker(&server, &pengajuan_id).await;
    assert_eq!(kesiapan["pegawai_belum_diisi"], 1, "{kesiapan}");

    // Putting back someone who is not excluded says so instead of pretending.
    let res = delete(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}/keluarkan"
        ),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 404, "{:?}", res.text());

    teardown_test_db(&db_name).await;
}

/// An account that is both operator and wilayah validator reads its region but
/// must only WRITE its own satker's roster. `SatkerScope::from_claims` widens to
/// the wilayah for reads; the write path takes `own_satker_from_claims`.
#[tokio::test]
async fn operator_berperan_ganda_tidak_menulis_roster_satker_lain() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let nip_b = "19850505000000002";
    tambah_pegawai(&db, nip_b, "api-skr002", "Pegawai Uji B", "P").await;
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;

    // SKR002's own operator excludes them, so there is a real exclusion to undo.
    let res = delete(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai/{nip_b}"),
        "operator_satker",
        OPERATOR,
        SATKER_B,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    const DUA_PERAN: &str = "operator_satker,validator_wilayah";

    // Undo it as a dual-role account whose own satker is SKR001.
    let res = delete(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai/{nip_b}/keluarkan"
        ),
        DUA_PERAN,
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "menulis roster satker lain harus 404: {:?}",
        res.text()
    );

    // Saving a size is the same write, so it is refused the same way.
    let res = put(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai/{nip_b}"),
        DUA_PERAN,
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": "L"}]}),
    )
    .await;
    assert_eq!(res.status_code(), 404, "{:?}", res.text());

    // The exclusion is still in force: nothing above changed SKR002's roster.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_B}/pegawai"),
        "validator_wilayah",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let data = res.json::<Value>()["data"].clone();
    let b = data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == nip_b)
        .unwrap_or_else(|| panic!("pegawai {nip_b} tidak ada di roster: {data}"))
        .clone();
    assert_eq!(b["dikeluarkan"], true, "{b}");

    teardown_test_db(&db_name).await;
}

/// A roster write that queues behind a submit must not land: the stage is read
/// **under the row lock**, not before it. Without that, the request would wait
/// for the submit, then put the employee back into a roster that is frozen.
#[tokio::test]
async fn masukkan_kembali_yang_menunggu_pengajuan_ikut_ditolak() {
    let (app, db, db_name) = setup_test_app().await;
    // `Arc` because the re-add request runs in a task of its own while this
    // test holds the row lock.
    let server = std::sync::Arc::new(TestServer::new(app));
    tambah_pegawai(&db, NIP_D, "api-skr001", "Pegawai Uji D", "P").await;
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &pakaian_id, "L").await;

    let res = delete(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    let data = roster(&server, &pengajuan_id).await;
    let row_id: uuid::Uuid = data["pengajuan_satker_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // Hold the row the submit will touch, so the re-add has to wait for it.
    let mut kunci = db.pool().get().await.unwrap();
    let pid_penahan: i32 = kunci
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .unwrap()
        .get(0);
    let tx = kunci.transaction().await.unwrap();
    tx.query_one(
        "SELECT 1 FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE id = $1 FOR UPDATE",
        &[&row_id],
    )
    .await
    .unwrap();

    let jalur = format!(
        "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_D}/keluarkan"
    );
    let srv = server.clone();
    let menunggu =
        tokio::spawn(
            async move { delete(&srv, &jalur, "operator_satker", OPERATOR, SATKER_A).await },
        );

    // Wait until the request is actually queued on the row lock, then submit.
    // The poll runs on its own connection: `kunci` is busy holding the
    // transaction.
    //
    // A row-lock wait is recorded against the holder's TRANSACTION id, not the
    // relation, so filtering `pg_locks.relation` never matches and the test
    // would silently degrade into "run the request after the commit".
    // `pg_blocking_pids` answers "who is this backend waiting for" directly,
    // and the loop ASSERTS it was observed.
    let pemantau = db.pool().get().await.unwrap();
    let mut teramati = false;
    for _ in 0..80 {
        let antre: i64 = pemantau
            .query_one(
                "SELECT count(*) FROM pg_stat_activity
                  WHERE wait_event_type = 'Lock' AND $1 = ANY(pg_blocking_pids(pid))",
                &[&pid_penahan],
            )
            .await
            .unwrap()
            .get(0);
        if antre > 0 {
            teramati = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        teramati,
        "permintaan tidak pernah terlihat mengantre di belakang kunci baris; \
         tanpa itu tes ini tidak membuktikan apa pun"
    );
    tx.execute(
        "UPDATE perlengkapan.pengajuan_pakaian_dinas_satker
            SET aktivitas_id = 1001 WHERE id = $1",
        &[&row_id],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let res = menunggu.await.unwrap();
    assert_eq!(
        res.status_code(),
        400,
        "yang menunggu di belakang pengajuan harus ditolak: {:?}",
        res.text()
    );

    let data = roster(&server, &pengajuan_id).await;
    let d = pegawai_dari_roster(&data, NIP_D);
    assert_eq!(
        d["dikeluarkan"], true,
        "pengecualian tidak boleh hilang setelah pengajuan terkirim: {d}"
    );

    teardown_test_db(&db_name).await;
}

/// The dashboard's size chart counts PEOPLE per kind of garment and size, and never lists free
/// text. It used to group on the size text alone: a shirt "M", trousers "32" and shoes "40" shared
/// one chart, a person with two shirt columns was counted twice under a caption that said
/// "pegawai", and the free text of a name-tag column (a person's NAME) was listed as if it were a
/// size.
#[tokio::test]
async fn distribusi_ukuran_di_dasbor_menghitung_orang_per_jenis_dan_tak_menampilkan_teks_bebas() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    tambah_pegawai(&db, NIP_D, "api-skr001", "Pegawai Uji D", "P").await;
    let (pengajuan_id, baju_id) = buat_kampanye(&server).await;
    simpan_ukuran(&server, &pengajuan_id, NIP_A, &baju_id, "L").await;
    simpan_ukuran(&server, &pengajuan_id, NIP_D, &baju_id, "L").await;

    let baju: uuid::Uuid = baju_id.parse().unwrap();
    let kaos = uuid::Uuid::new_v4();
    let nama_dada = uuid::Uuid::new_v4();
    let khusus = uuid::Uuid::new_v4();
    let client = db.pool().get().await.unwrap();
    // A free-text group, and a second PICK-FROM-A-LIST group whose name differs from "Baju" only in
    // its brackets: both shorten to "Baju", and keyed on the short label one count overwrote the other.
    client
        .execute(
            "INSERT INTO perlengkapan.ms_ukuran_group (kode, nama, tipe, urutan)
             VALUES ('TEKS_UJI', 'Nama dada', 'TEKS', 9),
                    ('BAJU_KHUSUS', 'Baju (khusus)', 'PILIHAN', 8)
             ON CONFLICT (kode) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
    // Two more columns of the same campaign: a second shirt column (BAJU, like the first) and a
    // free-text one. Both are filled for everybody who filled the first.
    for (id, nama, group, ukuran) in [
        (kaos, "Kaos", "BAJU", "L"),
        (khusus, "Baju khusus", "BAJU_KHUSUS", "L"),
        (nama_dada, "Nama dada", "TEKS_UJI", "BUDI SANTOSO"),
    ] {
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_pakaian
                     (id, pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama,
                      spesifikasi_id, spesifikasi_nama, spesifikasi_ukuran_group)
                 SELECT $1, pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama,
                        spesifikasi_id, $3, $4
                   FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE id = $2",
                &[&id, &baju, &nama, &group],
            )
            .await
            .unwrap();
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
                     (pengajuan_satker_id, pegawai_id, pakaian_id, ukuran)
                 SELECT pu.pengajuan_satker_id, pu.pegawai_id, $1, $3
                   FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
                  WHERE pu.pakaian_id = $2",
                &[&id, &baju, &ukuran],
            )
            .await
            .unwrap();
    }

    let res = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026",
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let ukuran = res.json::<Value>()["pakaian_dinas_metrics"]["total_by_ukuran"].clone();
    // Two PEOPLE wear a shirt of size L (A and D, each with two shirt columns), and two a "khusus"
    // one: separate bars, neither overwriting the other. The name is not a size.
    assert_eq!(
        ukuran,
        json!({"Baju L": 2, "Baju (khusus) L": 2}),
        "{ukuran}"
    );

    teardown_test_db(&db_name).await;
}
