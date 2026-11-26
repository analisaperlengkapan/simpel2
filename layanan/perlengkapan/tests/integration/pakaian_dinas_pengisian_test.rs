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
    assert_eq!(k["belum_lengkap"].as_i64(), Some(0), "{k:?}");
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

    // Pusat excludes the row it is about to act on: the remaining satker is
    // still at Input.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=pusat&kecuali={row_id}"),
        "validator_pusat",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan pusat: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["belum_mengajukan"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // Wilayah: SKR001 was submitted but has no sizes, SKR002 has not submitted
    // at all. The two failures are reported separately.
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
    assert_eq!(k["belum_lengkap"].as_i64(), Some(1), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(false), "{k:?}");

    // A bulk move excludes EVERY selected row: with both participation rows
    // named, nothing is left behind, so the bulk dialog stays quiet. This is
    // the comma list the mass-action bar sends.
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
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&kecuali={row_id},{other_id}"
        ),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "kesiapan massal: {:?}", res.text());
    let k = res.json::<Value>()["data"].clone();
    assert_eq!(k["total"].as_i64(), Some(0), "{k:?}");
    assert_eq!(k["lengkap"].as_bool(), Some(true), "{k:?}");

    // A malformed entry is a 400, not a silently dropped row.
    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap=wilayah&kecuali=bukan-uuid"
        ),
        "validator_wilayah",
        PUSAT,
        SATKER_A,
    )
    .await;
    assert_eq!(
        res.status_code(),
        400,
        "kecuali tidak valid: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}
