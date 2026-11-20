//! Grup ukuran sebagai DATA REFERENSI dan profil ukuran per spesifikasi (V019).
//!
//! Sampai V018, dua hal membuat jenis pakaian baru tak bisa ditambah sebagai data: grup ukuran
//! hanya boleh BAJU/CELANA/SEPATU (dua CHECK), dan profil pegawai hanya punya satu kolom per
//! grup, jadi sepatu PDH dan sepatu PDL, kaos dan baju, saling menimpa. Tes ini menjaga bentuk
//! barunya: admin menambah grup dan ukuran, spesifikasi memakai grup dari referensi, profil
//! tersimpan per spesifikasi dengan aturan grupnya (pilihan dari daftar, atau teks bebas), dan
//! saran pengisian dibaca per spesifikasi.

use super::pakaian_dinas_pengisian_test::{
    NIP_A, OPERATOR, PUSAT, SATKER_A, buat_kampanye, get, headers, periode_berjalan, post, put,
};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";

/// Token dengan klaim NIP: endpoint profil membaca pegawainya dari klaim, bukan dari badan permintaan.
fn bearer(nip: &str) -> reqwest::header::HeaderValue {
    reqwest::header::HeaderValue::from_str(&format!(
        "Bearer mock::operator_satker::{OPERATOR}::{SATKER_A}::{nip}"
    ))
    .unwrap()
}

async fn hapus(server: &TestServer, path: &str, role: &str, user: &str) -> axum_test::TestResponse {
    let mut req = server.delete(path);
    for (k, v) in headers(role, user, SATKER_A) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn admin_post(server: &TestServer, path: &str, body: Value) -> axum_test::TestResponse {
    post(server, path, "admin", ADMIN, SATKER_A, body).await
}

/// Id spesifikasi menurut NAMA jenis dan NAMA spesifikasi, seperti V011 mencocokkannya.
async fn spek_id(
    db: &layanan_perlengkapan::shared::db::Database,
    jenis: &str,
    nama: &str,
) -> String {
    let client = db.pool().get().await.unwrap();
    client
        .query_one(
            "SELECT s.id::text AS id FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
             JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
             WHERE upper(j.nama) = upper($1) AND upper(s.nama) = upper($2) ORDER BY s.created_at LIMIT 1",
            &[&jenis, &nama],
        )
        .await
        .unwrap_or_else(|e| panic!("spesifikasi {jenis}/{nama} tidak ada: {e}"))
        .get("id")
}

async fn jenis_id(db: &layanan_perlengkapan::shared::db::Database, nama: &str) -> String {
    let client = db.pool().get().await.unwrap();
    client
        .query_one(
            "SELECT id::text AS id FROM perlengkapan.ms_jenis_pakaian_dinas
             WHERE upper(nama) = upper($1) ORDER BY created_at LIMIT 1",
            &[&nama],
        )
        .await
        .unwrap()
        .get("id")
}

#[tokio::test]
async fn grup_baru_dibuat_admin_dinormalkan_dan_ditolak_peran_lain() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"kode": "kacamata", "nama": "Kacamata"}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        403,
        "hanya admin mengubah referensi: {}",
        res.text()
    );

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "sepatu pdl", "nama": "Sepatu PDL"}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let data = &res.json::<Value>()["data"];
    assert_eq!(
        data["kode"], "SEPATU_PDL",
        "kode dinormalkan: huruf besar, spasi jadi garis bawah"
    );
    assert_eq!(data["tipe"], "PILIHAN", "bawaannya grup berdaftar");

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "SEPATU_PDL", "nama": "Lagi"}),
    )
    .await;
    assert_ne!(res.status_code(), 201);
    assert!(
        res.text().contains("sudah ada"),
        "pesan yang jelas, bukan galat constraint: {}",
        res.text()
    );

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "x;drop", "nama": "Salah"}),
    )
    .await;
    assert_ne!(
        res.status_code(),
        201,
        "karakter di luar huruf, angka, dan garis bawah ditolak"
    );

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "ANEH", "nama": "Aneh", "tipe": "ANGKA"}),
    )
    .await;
    assert_ne!(res.status_code(), 201, "tipe hanya PILIHAN atau TEKS");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn spesifikasi_memakai_grup_dari_referensi_dan_menolak_yang_tidak_ada() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let pdl = jenis_id(&db, "PDL").await;

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "SEPATU_PDL", "nama": "Sepatu PDL"}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());

    // Grup baru langsung bisa dipakai spesifikasi, dan penulisan huruf kecil dinormalkan.
    let res = admin_post(
        &server,
        "/pakaian-dinas/spesifikasi",
        json!({"jenis_pakaian_dinas_id": pdl, "nama": "Sepatu Lapangan", "gender": "SEMUA",
               "ukuran_group": "sepatu_pdl", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    assert_eq!(res.json::<Value>()["data"]["ukuran_group"], "SEPATU_PDL");

    // Dulu ditolak karena bukan salah satu dari tiga nama; kini ditolak karena TIDAK ADA di referensi.
    let res = admin_post(
        &server,
        "/pakaian-dinas/spesifikasi",
        json!({"jenis_pakaian_dinas_id": pdl, "nama": "Tak Berumah", "gender": "SEMUA",
               "ukuran_group": "TIDAK_ADA", "is_active": true}),
    )
    .await;
    assert_ne!(res.status_code(), 201);
    assert!(res.text().contains("tidak ada"), "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_ditambah_hanya_ke_grup_berdaftar_tanpa_duplikat() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran",
        json!({"group": "TOPI", "ukuran": "68"}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());

    let res = get(
        &server,
        "/pakaian-dinas/ukuran?group=TOPI",
        "admin",
        ADMIN,
        SATKER_A,
    )
    .await;
    let daftar: Vec<String> = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["ukuran"].as_str().unwrap().to_string())
        .collect();
    assert!(
        daftar.contains(&"68".to_string()) && daftar.contains(&"57".to_string()),
        "{daftar:?}"
    );

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran",
        json!({"group": "TOPI", "ukuran": "68"}),
    )
    .await;
    assert_ne!(res.status_code(), 201, "ukuran yang sama dua kali");

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran",
        json!({"group": "NAMA_DADA", "ukuran": "X"}),
    )
    .await;
    assert_ne!(res.status_code(), 201, "grup TEKS tak punya daftar ukuran");
    assert!(res.text().contains("TEKS"), "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_dan_grup_yang_masih_dipakai_tidak_bisa_dihapus() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let baju = spek_id(&db, "PDH", "Pakaian Dinas").await;

    // Satu pegawai memakai 2XL pada kolom BAJU.
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama) VALUES ($1, 'P')",
            &[&NIP_A],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran)
             VALUES ($1, $2::text::uuid, '2XL')",
            &[&NIP_A, &baju],
        )
        .await
        .unwrap();

    let res = hapus(&server, "/pakaian-dinas/ukuran/BAJU/2XL", "admin", ADMIN).await;
    assert_ne!(res.status_code(), 200, "ukuran yang dipakai profil");
    assert!(res.text().contains("masih dipakai"), "{}", res.text());

    let res = hapus(&server, "/pakaian-dinas/ukuran/BAJU/8XL", "admin", ADMIN).await;
    assert_eq!(
        res.status_code(),
        200,
        "yang tak dipakai bisa dihapus: {}",
        res.text()
    );

    let res = hapus(&server, "/pakaian-dinas/ukuran-grup/BAJU", "admin", ADMIN).await;
    assert_ne!(res.status_code(), 200, "grup yang dipakai spesifikasi");
    assert!(res.text().contains("masih dipakai"), "{}", res.text());

    // Grup yang tak dipakai siapa pun bisa dihapus.
    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "SEMENTARA", "nama": "Sementara"}),
    )
    .await;
    assert_eq!(res.status_code(), 201);
    let res = hapus(
        &server,
        "/pakaian-dinas/ukuran-grup/SEMENTARA",
        "admin",
        ADMIN,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn profil_per_spesifikasi_menyimpan_pilihan_dan_teks_dengan_aturan_grupnya() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let nip = "199203142014031001";

    // Semua kolom pakaian aktif ditawarkan, termasuk yang ditambahkan V019, dengan tipenya.
    let res = server
        .get("/pakaian-dinas/ukuran-profil")
        .add_header(reqwest::header::AUTHORIZATION, bearer(nip))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let item = |daftar: &Value, jenis: &str, nama: &str| -> Value {
        daftar["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["jenis_pakaian_nama"] == jenis && i["spesifikasi_nama"] == nama)
            .unwrap_or_else(|| panic!("{jenis}/{nama} tidak ditawarkan: {daftar}"))
            .clone()
    };
    let awal = res.json::<Value>();
    for (j, n, tipe) in [
        ("PDL", "Kaos", "PILIHAN"),
        ("PDL", "Topi", "PILIHAN"),
        ("PDL", "Nama Dada", "TEKS"),
        ("PDH", "Sepatu Dinas", "PILIHAN"),
        ("PDL", "Sepatu", "PILIHAN"),
    ] {
        let i = item(&awal, j, n);
        assert_eq!(i["tipe"], tipe, "{j}/{n}");
        assert!(i["ukuran"].is_null(), "belum ada isian: {i}");
    }
    let kaos = item(&awal, "PDL", "Kaos")["spesifikasi_id"]
        .as_str()
        .unwrap()
        .to_string();
    let topi = item(&awal, "PDL", "Topi")["spesifikasi_id"]
        .as_str()
        .unwrap()
        .to_string();
    let nama_dada = item(&awal, "PDL", "Nama Dada")["spesifikasi_id"]
        .as_str()
        .unwrap()
        .to_string();
    let sepatu_pdh = item(&awal, "PDH", "Sepatu Dinas")["spesifikasi_id"]
        .as_str()
        .unwrap()
        .to_string();
    let sepatu_pdl = item(&awal, "PDL", "Sepatu")["spesifikasi_id"]
        .as_str()
        .unwrap()
        .to_string();

    let simpan = |body: Value| {
        server
            .put("/pakaian-dinas/ukuran-profil")
            .add_header(reqwest::header::AUTHORIZATION, bearer(nip))
            .json(&body)
    };

    // Pilihan dari daftar (ejaan daftar yang tersimpan), teks bebas dirapikan, dan sepatu PDH
    // serta sepatu PDL kini DUA isian yang berbeda.
    let res = simpan(json!({"ukuran": [
        {"spesifikasi_id": kaos, "ukuran": "xl"},
        {"spesifikasi_id": topi, "ukuran": "57"},
        {"spesifikasi_id": nama_dada, "ukuran": "  E. SILALAHI "},
        {"spesifikasi_id": sepatu_pdh, "ukuran": "42"},
        {"spesifikasi_id": sepatu_pdl, "ukuran": "43"},
    ]}))
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let tersimpan = res.json::<Value>();
    assert_eq!(item(&tersimpan, "PDL", "Kaos")["ukuran"], "XL");
    assert_eq!(item(&tersimpan, "PDL", "Topi")["ukuran"], "57");
    assert_eq!(
        item(&tersimpan, "PDL", "Nama Dada")["ukuran"],
        "E. SILALAHI"
    );
    assert_eq!(item(&tersimpan, "PDH", "Sepatu Dinas")["ukuran"], "42");
    assert_eq!(
        item(&tersimpan, "PDL", "Sepatu")["ukuran"],
        "43",
        "sepatu PDL tak menimpa sepatu PDH"
    );

    // Satu nilai di luar daftar menggagalkan SELURUH permintaan: yang lain tak ikut tersimpan.
    let res = simpan(json!({"ukuran": [
        {"spesifikasi_id": topi, "ukuran": "58"},
        {"spesifikasi_id": kaos, "ukuran": "99XL"},
    ]}))
    .await;
    assert_ne!(res.status_code(), 200, "ukuran di luar daftar");
    assert!(res.text().contains("tidak ada di daftar"), "{}", res.text());
    let res = server
        .get("/pakaian-dinas/ukuran-profil")
        .add_header(reqwest::header::AUTHORIZATION, bearer(nip))
        .await;
    assert_eq!(
        item(&res.json::<Value>(), "PDL", "Topi")["ukuran"],
        "57",
        "tak ada yang tersimpan sebagian"
    );

    // Isian teks dibatasi panjangnya; nilai kosong menghapus isian.
    let res =
        simpan(json!({"ukuran": [{"spesifikasi_id": nama_dada, "ukuran": "A".repeat(101)}]})).await;
    assert_ne!(res.status_code(), 200, "teks melebihi 100 karakter");
    let res = simpan(json!({"ukuran": [{"spesifikasi_id": kaos, "ukuran": ""}]})).await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert!(
        item(&res.json::<Value>(), "PDL", "Kaos")["ukuran"].is_null(),
        "kosong = hapus"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn saran_pengisian_dibaca_per_spesifikasi_bukan_per_grup() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Kampanye PDL dengan DUA kolom dalam grup BAJU: baju dan kaos. Dulu keduanya mendapat
    // saran yang sama, karena profil hanya punya satu ukuran per grup.
    let pdl = jenis_id(&db, "PDL").await;
    let baju = spek_id(&db, "PDL", "Baju").await;
    let kaos = spek_id(&db, "PDL", "Kaos").await;
    let (mulai, selesai) = periode_berjalan();
    let res = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({"nama": "PDL 2026", "tgl_mulai": mulai, "tgl_selesai": selesai, "is_reguler": true, "tahun": 2026,
               "pilihan_satker": "sebagian", "jenis_pakaian_dinas_id": pdl,
               "spesifikasi_ids": [baju, kaos], "satker_ids": [SATKER_A]}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let pengajuan = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Profil NIP_A: baju L, kaos M.
    for (spek, ukuran) in [(&baju, "L"), (&kaos, "M")] {
        let res = server
            .put("/pakaian-dinas/ukuran-profil")
            .add_header(reqwest::header::AUTHORIZATION, bearer(NIP_A))
            .json(&json!({"ukuran": [{"spesifikasi_id": spek, "ukuran": ukuran}]}))
            .await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
    }

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.json::<Value>();
    let kolom = |nama: &str| -> String {
        body["data"]["pakaian"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["spesifikasi_nama"] == nama)
            .unwrap_or_else(|| panic!("kolom {nama}: {body}"))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let pegawai = body["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_A)
        .unwrap()
        .clone();
    let saran = |pakaian_id: &str| -> Option<String> {
        pegawai["saran"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["pakaian_id"] == pakaian_id)
            .map(|s| s["ukuran"].as_str().unwrap().to_string())
    };
    assert_eq!(saran(&kolom("Baju")).as_deref(), Some("L"));
    assert_eq!(
        saran(&kolom("Kaos")).as_deref(),
        Some("M"),
        "kaos punya ukurannya sendiri, bukan ukuran baju"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn penulis_lama_tiga_ukuran_tetap_menjadi_saran() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan, pakaian) = buat_kampanye(&server).await;

    // Halaman Ukuran lama (dan impor massal admin) menulis tiga kolom. Saran dibaca per spesifikasi,
    // jadi penulis lama ikut mengisi tabel per spesifikasi untuk setiap spesifikasi dalam grupnya.
    let res = server
        .post("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, bearer(NIP_A))
        .json(&json!({"ukuran_baju": "L", "ukuran_celana": "32", "ukuran_sepatu": "41"}))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let pegawai = res.json::<Value>()["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_A)
        .unwrap()
        .clone();
    assert_eq!(pegawai["saran"][0]["pakaian_id"], pakaian.as_str());
    assert_eq!(pegawai["saran"][0]["ukuran"], "L");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pengisian_kampanye_menolak_ukuran_di_luar_daftar_dan_menyimpan_ejaan_daftar() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan, pakaian) = buat_kampanye(&server).await;
    let jalan = format!("/pakaian-dinas/pengajuan/{pengajuan}/satker/{SATKER_A}/pegawai/{NIP_A}");

    // Dulu API menyimpan nilai apa pun: daftar hanya dijaga oleh pilihan di layar.
    let res = put(
        &server,
        &jalan,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "XXL"}]}),
    )
    .await;
    assert_ne!(
        res.status_code(),
        200,
        "XXL sudah bukan penamaan daftar (2XL)"
    );
    assert!(res.text().contains("tidak ada di daftar"), "{}", res.text());

    let res = put(
        &server,
        &jalan,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "  2xl "}]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let pegawai = res.json::<Value>()["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_A)
        .unwrap()
        .clone();
    assert_eq!(
        pegawai["ukuran"][0]["ukuran"], "2XL",
        "tersimpan dengan ejaan daftar"
    );

    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------------------------
// Temuan review #1121
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn nama_dada_21_sampai_100_karakter_tersimpan_di_kampanye_dan_101_ditolak() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Kolom ukuran kampanye dulu varchar(20) sedangkan isian TEKS boleh 100: nilai yang sah di
    // profil menggagalkan seluruh penyimpanan pegawai di kampanye.
    let pdl = jenis_id(&db, "PDL").await;
    let nama_dada = spek_id(&db, "PDL", "Nama Dada").await;
    let (mulai, selesai) = periode_berjalan();
    let res = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        SATKER_A,
        json!({"nama": "PDL nama dada", "tgl_mulai": mulai, "tgl_selesai": selesai, "is_reguler": true,
               "tahun": 2026, "pilihan_satker": "sebagian", "jenis_pakaian_dinas_id": pdl,
               "spesifikasi_ids": [nama_dada], "satker_ids": [SATKER_A]}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let pengajuan = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let jalan_roster = format!("/pakaian-dinas/pengajuan/{pengajuan}/satker/{SATKER_A}/pegawai");
    let roster = get(
        &server,
        &jalan_roster,
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(roster.status_code(), 200, "{}", roster.text());
    let pakaian = roster.json::<Value>()["data"]["pakaian"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let jalan = format!("{jalan_roster}/{NIP_A}");

    for panjang in [21usize, 22, 100] {
        let teks = "A".repeat(panjang);
        let res = put(
            &server,
            &jalan,
            "operator_satker",
            OPERATOR,
            SATKER_A,
            json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": teks}]}),
        )
        .await;
        assert_eq!(res.status_code(), 200, "{panjang} karakter: {}", res.text());
    }
    let roster = get(
        &server,
        &jalan_roster,
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let pegawai = roster.json::<Value>()["data"]["pegawai"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_A)
        .unwrap()
        .clone();
    assert_eq!(
        pegawai["ukuran"][0]["ukuran"].as_str().unwrap().len(),
        100,
        "tersimpan utuh, tak terpotong"
    );

    let res = put(
        &server,
        &jalan,
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian, "ukuran": "A".repeat(101)}]}),
    )
    .await;
    assert_ne!(res.status_code(), 200, "101 karakter melampaui batas");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn grup_spesifikasi_tak_bisa_diganti_bila_profil_pegawai_sudah_memakainya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let pdl = jenis_id(&db, "PDL").await;

    // Spesifikasi baru di grup BAJU, satu pegawai mengisinya dengan L.
    let res = admin_post(
        &server,
        "/pakaian-dinas/spesifikasi",
        json!({"jenis_pakaian_dinas_id": pdl, "nama": "Rompi", "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let rompi = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = server
        .put("/pakaian-dinas/ukuran-profil")
        .add_header(reqwest::header::AUTHORIZATION, bearer(NIP_A))
        .json(&json!({"ukuran": [{"spesifikasi_id": rompi, "ukuran": "L"}]}))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let ubah = |grup: &str| {
        let jalan = format!("/pakaian-dinas/spesifikasi/{rompi}");
        let badan = json!({"jenis_pakaian_dinas_id": pdl, "nama": "Rompi", "gender": "SEMUA", "ukuran_group": grup, "is_active": true});
        let server = &server;
        async move { put(server, &jalan, "admin", ADMIN, SATKER_A, badan).await }
    };
    // Ukuran "L" milik BAJU akan terbaca sebagai ukuran SEPATU dan tak bisa disimpan ulang.
    let res = ubah("SEPATU").await;
    assert_ne!(
        res.status_code(),
        200,
        "grup diganti padahal ada isian profil"
    );
    assert!(res.text().contains("1 isian ukuran"), "{}", res.text());
    // Menyunting hal lain dengan grup yang SAMA tetap boleh.
    let res = ubah("BAJU").await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    // Spesifikasi tanpa isian bebas berganti grup.
    let res = admin_post(
        &server,
        "/pakaian-dinas/spesifikasi",
        json!({"jenis_pakaian_dinas_id": pdl, "nama": "Kosong", "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    let kosong = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = put(
        &server,
        &format!("/pakaian-dinas/spesifikasi/{kosong}"),
        "admin",
        ADMIN,
        SATKER_A,
        json!({"jenis_pakaian_dinas_id": pdl, "nama": "Kosong", "gender": "SEMUA", "ukuran_group": "SEPATU", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn penghapusan_ukuran_menunggu_penulis_yang_sedang_memvalidasi_lalu_melihat_pemakaiannya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let baju = spek_id(&db, "PDH", "Pakaian Dinas").await;

    // Penulis ukuran: sudah mengambil kunci bersama grup BAJU, memvalidasi 8XL terhadap daftar
    // yang masih memuatnya, menulis, tetapi belum commit. Persis interleaving yang dilaporkan review.
    let penulis = db.pool().get().await.unwrap();
    penulis.execute("BEGIN", &[]).await.unwrap();
    penulis
        .execute(
            "SELECT pg_advisory_xact_lock_shared(hashtext($1::text)::bigint)",
            &[&"ukuran_grup:BAJU"],
        )
        .await
        .unwrap();
    penulis
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama) VALUES ($1, 'P')",
            &[&NIP_A],
        )
        .await
        .unwrap();
    penulis
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran)
             VALUES ($1, $2::text::uuid, '8XL')",
            &[&NIP_A, &baju],
        )
        .await
        .unwrap();

    let pengamat = db.pool().get().await.unwrap();
    let hapus_8xl = hapus(&server, "/pakaian-dinas/ukuran/BAJU/8XL", "admin", ADMIN);
    let lepas = async {
        // Beri waktu penghapusan sampai ke kunci, lalu pastikan ia BENAR-BENAR menunggu.
        let mut menunggu = 0i64;
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            menunggu = pengamat
                .query_one(
                    "SELECT count(*) AS n FROM pg_locks WHERE locktype = 'advisory' AND NOT granted",
                    &[],
                )
                .await
                .unwrap()
                .get("n");
            if menunggu > 0 {
                break;
            }
        }
        penulis.execute("COMMIT", &[]).await.unwrap();
        menunggu
    };
    let (res, menunggu) = tokio::join!(hapus_8xl, lepas);

    assert_eq!(
        menunggu, 1,
        "penghapusan harus menunggu kunci yang dipegang penulis"
    );
    // Setelah commit penghapusan melihat 8XL sudah dipakai, jadi menolak. Tanpa kunci bersama ia
    // membaca nol pemakaian, menghapus 8XL, dan profil tersimpan dengan ukuran yang tak ada di daftar.
    assert_ne!(res.status_code(), 200, "8XL sudah dipakai: {}", res.text());
    assert!(res.text().contains("masih dipakai"), "{}", res.text());
    let masih: i64 = pengamat
        .query_one("SELECT count(*) AS n FROM perlengkapan.ms_ukuran WHERE \"group\" = 'BAJU' AND ukuran = '8XL'", &[])
        .await
        .unwrap()
        .get("n");
    assert_eq!(masih, 1, "ukuran tetap ada di daftar");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn penulis_lama_dua_versi_ke_belakang_tetap_menjadi_saran_lewat_jembatan_basis_data() {
    let (app, db, db_name) = setup_test_app().await;
    let _server = TestServer::new(app);
    let baju = spek_id(&db, "PDH", "Pakaian Dinas").await;
    let kaos = spek_id(&db, "PDL", "Kaos").await;
    let sepatu = spek_id(&db, "PDH", "Sepatu Dinas").await;
    let client = db.pool().get().await.unwrap();
    let ukuran = |spek: String| {
        let client = &client;
        async move {
            client
                .query_opt(
                    "SELECT ukuran FROM perlengkapan.pegawai_pakaian_dinas_ukuran WHERE nip = $1 AND spesifikasi_id = $2::text::uuid",
                    &[&NIP_A, &spek],
                )
                .await
                .unwrap()
                .map(|r| r.get::<_, String>("ukuran"))
        }
    };

    // SQL yang SAMA dengan yang dijalankan versi sebelum V019: hanya tiga kolom profil.
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama, ukuran_baju, ukuran_sepatu) VALUES ($1, 'P', 'L', '42')",
            &[&NIP_A],
        )
        .await
        .unwrap();
    assert_eq!(ukuran(baju.clone()).await.as_deref(), Some("L"));
    assert_eq!(ukuran(sepatu.clone()).await.as_deref(), Some("42"));

    // Rollback, lalu pegawai mengganti ukuran bajunya lewat halaman lama: berhasil di versi lama,
    // dan harus menjadi saran begitu versi baru berjalan lagi.
    client
        .execute("UPDATE perlengkapan.pegawai_pakaian_dinas SET ukuran_baju = 'M', updated_at = now() WHERE nip = $1", &[&NIP_A])
        .await
        .unwrap();
    assert_eq!(
        ukuran(baju.clone()).await.as_deref(),
        Some("M"),
        "perubahan versi lama sampai ke saran"
    );

    // Isian per spesifikasi yang ditulis kode baru tak tertimpa oleh penulisan kolom lain.
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran)
             VALUES ($1, $2::text::uuid, 'S') ON CONFLICT (nip, spesifikasi_id) DO UPDATE SET ukuran = EXCLUDED.ukuran",
            &[&NIP_A, &kaos],
        )
        .await
        .unwrap();
    client
        .execute(
            "UPDATE perlengkapan.pegawai_pakaian_dinas SET ukuran_sepatu = '43' WHERE nip = $1",
            &[&NIP_A],
        )
        .await
        .unwrap();
    assert_eq!(
        ukuran(kaos.clone()).await.as_deref(),
        Some("S"),
        "kolom baju tak berubah, kaos tak disentuh"
    );
    assert_eq!(ukuran(sepatu.clone()).await.as_deref(), Some("43"));

    // Versi lama menulis penamaan lamanya.
    client
        .execute(
            "UPDATE perlengkapan.pegawai_pakaian_dinas SET ukuran_baju = 'XXL' WHERE nip = $1",
            &[&NIP_A],
        )
        .await
        .unwrap();
    assert_eq!(
        ukuran(baju.clone()).await.as_deref(),
        Some("2XL"),
        "XXL diterjemahkan ke penamaan daftar"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ukuran_dan_grup_berisi_spasi_saja_ditolak() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran",
        json!({"group": "TOPI", "ukuran": "   "}),
    )
    .await;
    assert_ne!(
        res.status_code(),
        201,
        "ukuran kosong tak boleh masuk daftar: {}",
        res.text()
    );
    let res = admin_post(
        &server,
        "/pakaian-dinas/ukuran-grup",
        json!({"kode": "SPASI", "nama": "   "}),
    )
    .await;
    assert_ne!(res.status_code(), 201, "grup tanpa nama: {}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn profil_baru_membawa_bendera_hijab_dan_tak_menimpanya_bila_tak_dikirim() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let kaos = spek_id(&db, "PDL", "Kaos").await;
    let nip = "199203142014031002";

    let simpan = |body: Value| {
        server
            .put("/pakaian-dinas/ukuran-profil")
            .add_header(reqwest::header::AUTHORIZATION, bearer(nip))
            .json(&body)
    };
    let hijab = || async {
        let res = server
            .get("/pakaian-dinas/ukuran-pakaian-pegawai")
            .add_header(reqwest::header::AUTHORIZATION, bearer(nip))
            .await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        res.json::<Value>()["data"]["with_hijab"].as_bool()
    };

    // Halaman Ukuran lama menyimpan bendera ini bersama tiga ukuran; halaman baru tak bisa memakai
    // endpoint itu (menimpa ukuran per spesifikasi), jadi bendera ikut di endpoint profil.
    let res =
        simpan(json!({"ukuran": [{"spesifikasi_id": kaos, "ukuran": "M"}], "with_hijab": true}))
            .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert_eq!(hijab().await, Some(true));

    // Tak dikirim = tak diubah.
    let res = simpan(json!({"ukuran": [{"spesifikasi_id": kaos, "ukuran": "L"}]})).await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert_eq!(
        hijab().await,
        Some(true),
        "bendera yang tak dikirim tak boleh terhapus"
    );

    let res = simpan(json!({"ukuran": [], "with_hijab": false})).await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert_eq!(hijab().await, Some(false));

    teardown_test_db(&db_name).await;
}
