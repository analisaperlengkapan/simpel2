//! The exported laporan has the shape the legacy simpelv1 report has.
//!
//! The legacy report is the Blade template rendered to PDF and Excel. Its rekap
//! prints EVERY size of a clothing group as a column and EVERY satker of the
//! campaign as a row, zeros included, with one table per (pakaian x gender); its
//! daftar prints a heading per satker with a "Data Kosong" row for one that has
//! nobody in it. The export used to print only what occurred, which gave every
//! page a different shape and hid a satker that had not finished.
//!
//! Driven through HTTP exactly as the export button does. The campaign has two
//! satker, only one of them Selesai, so "every satker" and "only the finished
//! one" give different answers.

use super::pakaian_dinas_laporan_cakupan_test::{PUSAT, get, seed};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use std::io::Read;

/// The two satker of the shared fixture, as the report names them (the test
/// satker master resolves their codes to these names).
const NAMA_A: &str = "KEJAKSAAN NEGERI UJI A";
const NAMA_B: &str = "KEJAKSAAN NEGERI UJI B";

/// `sheet1.xml` and the shared strings of an XLSX.
fn parts(bytes: &[u8]) -> (String, String) {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("an xlsx is a zip");
    let mut read = |name: &str| {
        let mut s = String::new();
        z.by_name(name)
            .unwrap_or_else(|_| panic!("{name} missing"))
            .read_to_string(&mut s)
            .unwrap();
        s
    };
    (
        read("xl/worksheets/sheet1.xml"),
        read("xl/sharedStrings.xml"),
    )
}

async fn export(
    server: &TestServer,
    pengajuan_id: impl std::fmt::Display,
    laporan: &str,
    file: &str,
) -> axum_test::TestResponse {
    get(
        server,
        &format!(
            "/pakaian-dinas/laporan/cetak?jenis_laporan={laporan}&jenis_file={file}&pengajuan_id={pengajuan_id}"
        ),
        "validator_pusat",
        PUSAT,
    )
    .await
}

#[tokio::test]
async fn rekap_excel_prints_every_size_every_satker_and_both_genders() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let res = export(&server, f.pengajuan_id, "rekap", "excel").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let (sheet, strings) = parts(res.as_bytes());

    // The three heading lines, as the legacy template prints them.
    assert!(strings.contains("Pengajuan PDH 2026"), "{strings}");
    assert!(strings.contains("WILAYAH"), "instance line: {strings}");
    assert!(
        strings.contains("Periode: 01-Januari-2026 S/D 31-Desember-2026"),
        "legacy date format: {strings}"
    );

    // "SEMUA" gives both gender tables.
    assert!(strings.contains("Baju PDH - L"), "{strings}");
    assert!(strings.contains("Baju PDH - P"), "{strings}");

    // Every size of the group is a column, picked or not: the extremes of the
    // master list prove the whole list is there, not only the "L" someone chose.
    assert!(
        strings.contains("XS") && strings.contains("4XL"),
        "{strings}"
    );

    // Every satker of the campaign has a row: the unfinished one too, with zeros.
    assert!(strings.contains(NAMA_A), "{strings}");
    assert!(
        strings.contains(NAMA_B),
        "a satker that has not finished still has its row: {strings}"
    );

    // One table per gender, each closed by a "Jumlah" row that spans No+Satker.
    let merges_in_column_a = sheet.matches("<mergeCell ref=\"A").count();
    assert!(
        merges_in_column_a >= 4,
        "No spans + Jumlah spans, per table: {sheet}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn daftar_excel_lists_every_satker_and_marks_the_empty_one() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let res = export(&server, f.pengajuan_id, "daftar", "excel").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let (_sheet, strings) = parts(res.as_bytes());

    assert!(strings.contains(NAMA_A), "{strings}");
    assert!(
        strings.contains(NAMA_B),
        "the unfinished satker has a heading: {strings}"
    );
    assert!(
        strings.contains("Data Kosong"),
        "and says it has nobody counted yet: {strings}"
    );
    assert!(strings.contains("Busana Muslimah"), "{strings}");
    // The NIP is written whole, with nothing in front of it.
    assert!(strings.contains("198001010000000001"), "{strings}");
    assert!(!strings.contains("&#039;"), "{strings}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn both_reports_come_back_as_real_pdfs() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    for laporan in ["rekap", "daftar"] {
        let res = export(&server, f.pengajuan_id, laporan, "pdf").await;
        assert_eq!(res.status_code(), 200, "{laporan}: {:?}", res.text());
        assert!(res.as_bytes().starts_with(b"%PDF-"), "{laporan} is a PDF");
        let mut warnings = Vec::new();
        let doc = printpdf::PdfDocument::parse(
            res.as_bytes(),
            &printpdf::PdfParseOptions::default(),
            &mut warnings,
        )
        .expect("the export parses as a PDF");
        assert!(!doc.pages.is_empty(), "{laporan} has pages");
    }

    teardown_test_db(&db_name).await;
}
