//! Laporan yang mengikuti isi (V019): kolom per kolom pakaian, isian teks, dan kertas.
//!
//! Sebelum V019 laporan mengumpulkan ukuran per GRUP (BAJU/CELANA/SEPATU): daftar di layar memakai
//! `MAX(CASE WHEN group = 'BAJU' …)`, jadi baju dan kaos (sama-sama BAJU) saling menimpa dengan nilai
//! terbesar menurut abjad, dan topi serta nama dada tak pernah tampil. Rekap menghitung orang per
//! ukuran, sehingga isian teks (nama dada) akan menjadi satu kolom per nama. Dan berkas XLSX tak
//! punya pengaturan kertas sama sekali.
//!
//! Dijalankan lewat HTTP seperti tombol ekspor, di atas fixture bersama: dua satker, satu Selesai.

use super::pakaian_dinas_laporan_cakupan_test::{PUSAT, get, seed};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::Value;
use std::io::Read;
use uuid::Uuid;

const NIP_SELESAI: &str = "198001010000000001";

/// `sheet1.xml` dan string bersama sebuah XLSX.
fn bagian(bytes: &[u8]) -> (String, String) {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("xlsx adalah zip");
    let mut baca = |nama: &str| {
        let mut s = String::new();
        z.by_name(nama)
            .unwrap_or_else(|_| panic!("{nama} tak ada"))
            .read_to_string(&mut s)
            .unwrap();
        s
    };
    (
        baca("xl/worksheets/sheet1.xml"),
        baca("xl/sharedStrings.xml"),
    )
}

/// Tambahkan dua kolom pakaian pada kampanye fixture dan isi untuk pegawai satker yang Selesai:
/// "Kaos PDH" (grup BAJU, sama dengan baju: kasus tabrakan) dan "Nama Dada" (grup TEKS).
async fn tambah_kolom(db: &layanan_perlengkapan::shared::db::Database, pengajuan_id: Uuid) {
    let client = db.pool().get().await.unwrap();
    let baju: Uuid = client
        .query_one(
            "SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE pengajuan_id = $1",
            &[&pengajuan_id],
        )
        .await
        .unwrap()
        .get("id");
    let (pegawai_id, satker_id): (Uuid, Uuid) = {
        let r = client
            .query_one(
                "SELECT id, pengajuan_satker_id FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE nip = $1",
                &[&NIP_SELESAI],
            )
            .await
            .unwrap();
        (r.get("id"), r.get("pengajuan_satker_id"))
    };
    for (nama, grup, ukuran) in [
        ("Kaos PDH", "BAJU", "M"),
        ("Nama Dada", "NAMA_DADA", "AGUS PRATAMA"),
    ] {
        let id: Uuid = client
            .query_one(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_pakaian
                   (pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama, spesifikasi_id,
                    spesifikasi_nama, spesifikasi_ukuran_group)
                 SELECT pengajuan_id, jenis_pakaian_id, jenis_pakaian_nama, spesifikasi_id, $2, $3
                 FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE id = $1
                 RETURNING id",
                &[&baju, &nama, &grup],
            )
            .await
            .unwrap_or_else(|e| panic!("kolom {nama}: {e}"))
            .get("id");
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
                   (pengajuan_satker_id, pegawai_id, pakaian_id, ukuran) VALUES ($1, $2, $3, $4)",
                &[&satker_id, &pegawai_id, &id, &ukuran],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn daftar_di_layar_memisahkan_baju_dan_kaos_dan_memuat_isian_teks() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;
    tambah_kolom(&db, f.pengajuan_id).await;

    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/laporan/daftar-pegawai?pengajuan_id={}",
            f.pengajuan_id
        ),
        "validator_pusat",
        PUSAT,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.json::<Value>();
    let pegawai = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nip"] == NIP_SELESAI)
        .unwrap_or_else(|| panic!("pegawai satker Selesai tak ada: {body}"))
        .clone();
    let ukuran = |nama: &str| -> Option<String> {
        pegawai["ukuran"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["pakaian_nama"] == nama)
            .map(|u| u["ukuran"].as_str().unwrap().to_string())
    };
    // Dulu: MAX('L','M') = 'M' di kolom Baju, kaosnya hilang, nama dada tak tampil.
    assert_eq!(
        ukuran("Baju PDH").as_deref(),
        Some("L"),
        "baju tak ditimpa kaos: {pegawai}"
    );
    assert_eq!(ukuran("Kaos PDH").as_deref(), Some("M"));
    assert_eq!(
        ukuran("Nama Dada").as_deref(),
        Some("AGUS PRATAMA"),
        "isian teks tampil di daftar"
    );

    // Kolomnya sama di setiap halaman: dari pengajuan, urut seperti ekspor daftar.
    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/laporan/daftar-kolom?pengajuan_id={}",
            f.pengajuan_id
        ),
        "validator_pusat",
        PUSAT,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let kolom: Vec<String> = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap().to_string())
        .collect();
    assert_eq!(kolom, ["Baju PDH", "Kaos PDH", "Nama Dada"]);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn rekap_tidak_membuat_kolom_dari_isian_teks_dan_ekspor_mengikuti() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;
    tambah_kolom(&db, f.pengajuan_id).await;

    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/laporan/rekap-ukuran?pengajuan_id={}",
            f.pengajuan_id
        ),
        "validator_pusat",
        PUSAT,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let nama: Vec<String> = res.json::<Value>()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["pakaian_nama"].as_str().unwrap().to_string())
        .collect();
    assert!(
        nama.iter().any(|n| n == "Kaos PDH"),
        "kolom berdaftar tetap direkap: {nama:?}"
    );
    assert!(
        !nama.iter().any(|n| n == "Nama Dada"),
        "isian teks bukan bahan rekap: {nama:?}"
    );

    let cetak = |laporan: &str, berkas: &str| {
        let jalan = format!(
            "/pakaian-dinas/laporan/cetak?jenis_laporan={laporan}&jenis_file={berkas}&pengajuan_id={}",
            f.pengajuan_id
        );
        let server = &server;
        async move { get(server, &jalan, "validator_pusat", PUSAT).await }
    };

    // Rekap Excel: tabel Kaos ada, tak ada tabel (atau kolom) bagi nama dada.
    let res = cetak("rekap", "excel").await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let (sheet, strings) = bagian(res.as_bytes());
    assert!(strings.contains("Kaos PDH - L"), "{strings}");
    assert!(
        !strings.contains("Nama Dada"),
        "tak ada tabel nama dada di rekap: {strings}"
    );
    assert!(!strings.contains("AGUS PRATAMA"));
    // Kertas: landscape, A4 selama muat, skala yang memuat lebarnya, tiap tabel di halamannya.
    assert!(sheet.contains("orientation=\"landscape\""), "{sheet}");
    assert!(
        sheet.contains("paperSize=\"9\""),
        "A4 untuk tabel yang muat: {sheet}"
    );
    // Skala eksplisit, bukan "fit to pages": Excel mengabaikan page break manual dalam mode itu.
    assert!(!sheet.contains("fitToPage"), "{sheet}");
    assert!(
        sheet.contains("<rowBreaks"),
        "tiap tabel mulai di halaman baru: {sheet}"
    );

    // Daftar Excel: nama dada ada di daftar (bukan di rekap), dan kertasnya diatur.
    let res = cetak("daftar", "excel").await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let (sheet, strings) = bagian(res.as_bytes());
    assert!(strings.contains("AGUS PRATAMA"), "{strings}");
    assert!(
        strings.contains("Nama Dada") && strings.contains("Kaos PDH"),
        "{strings}"
    );
    assert!(
        sheet.contains("orientation=\"landscape\"") && sheet.contains("<rowBreaks"),
        "{sheet}"
    );

    // PDF: tetap PDF nyata untuk keduanya.
    for laporan in ["rekap", "daftar"] {
        let res = cetak(laporan, "pdf").await;
        assert_eq!(res.status_code(), 200, "{laporan}: {}", res.text());
        assert!(res.as_bytes().starts_with(b"%PDF"), "{laporan} bukan PDF");
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn ekspor_profil_menamai_baris_menurut_pakaian_dan_memuat_kolom_baru() {
    use layanan_perlengkapan::export::models::ExportQuery;
    use layanan_perlengkapan::shared::satker_scope::SatkerScope;

    let (app, db, db_name) = setup_test_app().await;
    let _server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();
    let nip = "198501012010011009";
    client
        .execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama, kode_satker, status)
             VALUES ($1, 'Pegawai Z', 'SKR001', 'ACTIVE')",
            &[&nip],
        )
        .await
        .unwrap();
    // Sepatu PDH dan sepatu PDL: dua isian, dua baris (dulu satu kolom `ukuran_sepatu`), serta kaos
    // dan nama dada yang tak punya kolom lama sama sekali.
    for (jenis, spek, ukuran) in [
        ("PDH", "Sepatu Dinas", "42"),
        ("PDL", "Sepatu", "43"),
        ("PDL", "Kaos", "M"),
        ("PDL", "Nama Dada", "AGUS PRATAMA"),
    ] {
        client
            .execute(
                "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran)
                 SELECT $1, s.id, $4 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                 JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
                 WHERE upper(j.nama) = $2 AND upper(s.nama) = upper($3)",
                &[&nip, &jenis, &spek, &ukuran],
            )
            .await
            .unwrap_or_else(|e| panic!("{jenis}/{spek}: {e}"));
    }
    let rows = db
        .export_rows(&ExportQuery {
            entity_type: "pakaian_dinas".to_string(),
            filters: None,
            limit: Some(100),
            tahun_anggaran: None,
            satker_id: None,
            status: None,
            scope: SatkerScope::Satker("SKR001".into()),
        })
        .await
        .unwrap();
    let isi: Vec<(String, String)> = rows
        .iter()
        .filter(|r| r["nip"] == nip)
        .map(|r| {
            (
                r["jenis_pakaian"].as_str().unwrap().to_string(),
                r["ukuran"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    for dicari in [
        ("PDH Sepatu Dinas", "42"),
        ("PDL Sepatu", "43"),
        ("PDL Kaos", "M"),
        ("PDL Nama Dada", "AGUS PRATAMA"),
    ] {
        assert!(
            isi.iter().any(|(j, u)| (j.as_str(), u.as_str()) == dicari),
            "{dicari:?} tak ada di ekspor: {isi:?}"
        );
    }

    drop(client);
    teardown_test_db(&db_name).await;
}
