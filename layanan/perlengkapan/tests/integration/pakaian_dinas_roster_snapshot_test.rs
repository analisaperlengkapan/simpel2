//! Daftar pegawai pakaian dinas DIKUNCI per kampanye saat validator pusat
//! membuatnya (V018), dan tanggal pembekuannya tampil ke semua role.
//!
//! Sebelum ini roster dibaca LANGSUNG dari `integrasi.mysimkari_pegawai` tiap
//! halaman dibuka, jadi keanggotaan (siapa yang harus mengisi) tidak pernah
//! dibekukan: pegawai yang mutasi masuk/keluar setelah kampanye dibuat ikut
//! berubah pada kampanye LAMA, dan identitas yang tersimpan saat operator
//! menyimpan ukuran bisa berbeda dari yang dilihatnya saat mengisi.
//!
//! Tiga hal yang ditegaskan, dan masing-masing gagal pada bentuk lama:
//!
//!   1. **Keanggotaan beku.** Pegawai yang ditambahkan ke MySIMKARI SETELAH
//!      kampanye dibuat tidak muncul di roster kampanye itu; yang dihapus dari
//!      MySIMKARI tetap muncul. (Pada bentuk lama, yang pertama muncul dan yang
//!      kedua hilang.)
//!   2. **Tanggal terlihat semua role.** `roster_snapshot_at` hadir dan sama
//!      untuk operator, validator wilayah, dan validator pusat.
//!   3. **Identitas dari snapshot.** Mengubah nama pegawai di MySIMKARI tidak
//!      mengubah baris pengajuan yang ditulis operator setelahnya.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const PUSAT: &str = "00000000-0000-0000-0000-000000000002";
const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
const NIP_A: &str = "19800101000000001";
const NIP_BARU: &str = "19990101000000099";
const NIP_TANPA_NAMA: &str = "19750101000000077";

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

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

/// Kampanye PDH menyasar SKR001 + SKR002, satu kolom BAJU.
/// Mengembalikan `(pengajuan_id, pakaian_id, roster_snapshot_at)`.
async fn buat_kampanye(server: &TestServer) -> (String, String, String) {
    let hari = chrono::Utc::now().date_naive();
    let mulai = (hari - chrono::Duration::days(30)).to_string();
    let selesai = (hari + chrono::Duration::days(300)).to_string();

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
    let data = &res.json::<Value>()["data"];
    let pengajuan_id = data["id"].as_str().unwrap().to_string();
    let snapshot_at = data["roster_snapshot_at"]
        .as_str()
        .expect("kampanye baru harus punya roster_snapshot_at")
        .to_string();

    // Kolomnya dari daftar pengisian, bukan `spesifikasi_id`: yang dipakai
    // menyimpan ukuran adalah `pengajuan_pakaian_dinas_pakaian.id`.
    let roster = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(roster.status_code(), 200, "roster: {:?}", roster.text());
    let pakaian_id = roster.json::<Value>()["data"]["pakaian"][0]["id"]
        .as_str()
        .expect("kampanye harus membawa minimal satu kolom pakaian")
        .to_string();
    (pengajuan_id, pakaian_id, snapshot_at)
}

async fn roster(server: &TestServer, pengajuan_id: &str, satker: &str) -> Value {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{satker}/pegawai"),
        "operator_satker",
        OPERATOR,
        satker,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    res.json::<Value>()["data"].clone()
}

fn nips(data: &Value) -> Vec<String> {
    data["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["nip"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn keanggotaan_dibekukan_saat_kampanye_dibuat() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _, _) = buat_kampanye(&server).await;

    // Awalnya roster berisi pegawai yang ada di MySIMKARI.
    let sebelum = roster(&server, &pengajuan_id, SATKER_A).await;
    assert_eq!(nips(&sebelum), vec![NIP_A.to_string()], "{sebelum}");

    // Kepegawaian berubah SETELAH kampanye: satu pegawai baru masuk, yang lama
    // pindah keluar (dihapus dari satker ini).
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, jk)
             VALUES ($1, 'Pegawai Baru', 'api-skr001', 'KEJAKSAAN NEGERI UJI A', 'Staf', 'L')",
            &[&NIP_BARU],
        )
        .await
        .unwrap();
    client
        .execute(
            "DELETE FROM integrasi.mysimkari_pegawai WHERE nip = $1",
            &[&NIP_A],
        )
        .await
        .unwrap();

    // Roster kampanye LAMA tidak bergerak: yang baru tidak muncul, yang keluar
    // tetap ada. Inilah "dikunci per pengajuan".
    let sesudah = roster(&server, &pengajuan_id, SATKER_A).await;
    let sesudah_nips = nips(&sesudah);
    assert_eq!(
        sesudah_nips,
        vec![NIP_A.to_string()],
        "roster beku tidak boleh ikut perubahan MySIMKARI: {sesudah}"
    );
    assert!(
        !sesudah_nips.contains(&NIP_BARU.to_string()),
        "pegawai yang masuk setelah kampanye tidak boleh muncul: {sesudah}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn tanggal_snapshot_terlihat_untuk_semua_role() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, _, snapshot_at) = buat_kampanye(&server).await;
    let path = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai");

    // Operator, validator wilayah (SKR001 di bawah KJT01), dan validator pusat
    // semuanya melihat tanggal pembekuan yang SAMA.
    for (role, user, satker) in [
        ("operator_satker", OPERATOR, SATKER_A),
        ("validator_wilayah", OPERATOR, "KJT01"),
        ("validator_pusat", PUSAT, SATKER_A),
    ] {
        let res = get(&server, &path, role, user, satker).await;
        assert_eq!(res.status_code(), 200, "{role}: {:?}", res.text());
        let data = &res.json::<Value>()["data"];
        assert_eq!(
            data["roster_snapshot_at"].as_str(),
            Some(snapshot_at.as_str()),
            "role {role} harus melihat tanggal snapshot yang sama: {data}"
        );
    }

    teardown_test_db(&db_name).await;
}

/// MySIMKARI mengizinkan `nama` NULL (dan `storage.rs` memang menulis NULL saat
/// nama tak tersedia), sedangkan kolom snapshot NOT NULL. Menyalin NULL apa
/// adanya menggagalkan SELURUH transaksi pembuatan pengajuan — dan karena
/// snapshot adalah satu INSERT..SELECT untuk semua satker target, satu pegawai
/// tanpa nama pada SATU satker memblokir pembuatan periode untuk SEMUA satker
/// (termasuk yang datanya lengkap). Pegawai itu tetap harus ikut ke daftar.
#[tokio::test]
async fn pegawai_tanpa_nama_tidak_memblokir_pembuatan_pengajuan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Satu pegawai tanpa nama di SKR001 SEBELUM kampanye dibuat.
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, jk)
             VALUES ($1, NULL, 'api-skr001', 'KEJAKSAAN NEGERI UJI A', 'Staf', 'L')",
            &[&NIP_TANPA_NAMA],
        )
        .await
        .unwrap();

    // Pembuatan pengajuan harus berhasil (dulu gagal karena NOT NULL).
    let (pengajuan_id, _, _) = buat_kampanye(&server).await;

    // Pegawai tanpa nama tetap muncul, dilabeli NIP sebagai pengganti nama.
    let data = roster(&server, &pengajuan_id, SATKER_A).await;
    let orang = data["pegawai"].as_array().unwrap();
    let tanpa_nama = orang
        .iter()
        .find(|p| p["nip"] == NIP_TANPA_NAMA)
        .unwrap_or_else(|| panic!("pegawai tanpa nama harus tetap ada di roster: {data}"));
    assert_eq!(
        tanpa_nama["nama"].as_str(),
        Some(NIP_TANPA_NAMA),
        "nama NULL diganti NIP, bukan dikosongkan atau dihilangkan: {tanpa_nama}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn identitas_pegawai_diambil_dari_snapshot_bukan_mysimkari() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id, _) = buat_kampanye(&server).await;

    // Setelah kampanye, nama pegawai berubah di MySIMKARI.
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "UPDATE integrasi.mysimkari_pegawai SET nama = 'Nama Baru Di Mysimkari' WHERE nip = $1",
            &[&NIP_A],
        )
        .await
        .unwrap();

    // Operator menyimpan ukuran. Identitas yang tersimpan harus yang DIBEKUKAN
    // (nama saat kampanye dibuat), bukan yang baru.
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

    let row = client
        .query_one(
            "SELECT psp.nama FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
             JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON ps.id = psp.pengajuan_satker_id
             WHERE ps.pengajuan_id = $1 AND psp.nip = $2",
            &[&uuid::Uuid::parse_str(&pengajuan_id).unwrap(), &NIP_A],
        )
        .await
        .unwrap();
    let nama: String = row.get("nama");
    assert_eq!(
        nama, "Pegawai Uji A",
        "identitas harus dari snapshot beku, bukan MySIMKARI yang berubah"
    );

    teardown_test_db(&db_name).await;
}
