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
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const PUSAT: &str = "00000000-0000-0000-0000-000000000002";
const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
const NIP_A: &str = "19800101000000001";

fn headers(role: &str, user: &str, satker: &str) -> Headers {
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

async fn post(
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

async fn get(
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

async fn put(
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
async fn buat_kampanye(server: &TestServer) -> (String, String) {
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
            "tgl_mulai": "2026-01-01",
            "tgl_selesai": "2026-12-31",
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
