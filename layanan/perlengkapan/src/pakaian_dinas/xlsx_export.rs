//! # Pakaian Dinas XLSX Export
//!
//! Renders the shared report view-models ([`DaftarReport`]/[`RekapReport`],
//! built in `services.rs`) to XLSX. The PDF generator (`pdf_export.rs`) renders
//! the *same* view-models, so the two formats stay in lock-step.
//!
//! Layout parity = legacy simpelv1 `cetakDaftarTemplateV` / `cetakRekapTemplateV`:
//! - Daftar: dynamic clothing columns, grouped per satker, nama/periode/filter
//!   header.
//! - Rekap: one block per pakaian × gender, rows = satker, columns = ukuran,
//!   per-block summary row.
//!
//! Every sheet is set landscape + **fit-to-1-page-wide** so the (potentially
//! wide, dynamic) tables never spill past the printable width.

use super::models::{DaftarReport, LaporanFilter, RekapReport, ReportHeader};
use super::services::PakaianDinasService;
use crate::shared::error::*;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, Worksheet};
use uuid::Uuid;

// ─────────────────────────── async wrappers ───────────────────────────

pub async fn generate_rekap_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    let report = service.build_rekap_report(pengajuan_id, filter).await?;
    render_rekap_xlsx(&report)
}

pub async fn generate_daftar_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    let report = service.build_daftar_report(pengajuan_id, filter).await?;
    render_daftar_xlsx(&report)
}

// ─────────────────────────── helpers ───────────────────────────

fn fmt_date(d: Option<chrono::NaiveDate>) -> String {
    d.map(|d| d.format("%d-%m-%Y").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn gender_label(g: &str) -> String {
    match g {
        "L" => "LAKI-LAKI".to_string(),
        "P" => "PEREMPUAN".to_string(),
        other => other.to_uppercase(),
    }
}

/// Write the title / periode / filter header block; returns the next free row.
fn write_header_block(
    ws: &mut Worksheet,
    header: &ReportHeader,
    title_fmt: &Format,
    meta_fmt: &Format,
) -> AppResult<u32> {
    let mut row = 0u32;
    ws.write_string_with_format(row, 0, &header.judul, title_fmt)
        .map_err(|e| bad_request(&e.to_string()))?;
    row += 1;
    if header.periode_mulai.is_some() || header.periode_selesai.is_some() {
        let periode = format!(
            "Periode: {} s/d {}",
            fmt_date(header.periode_mulai),
            fmt_date(header.periode_selesai)
        );
        ws.write_string_with_format(row, 0, &periode, meta_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 1;
    }
    for (label, value) in &header.filters {
        ws.write_string_with_format(row, 0, format!("{}: {}", label, value), meta_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 1;
    }
    Ok(row + 1) // blank spacer row
}

// ─────────────────────────── Daftar ───────────────────────────

/// Fixed leading columns of the daftar table (before the dynamic clothing cols).
const DAFTAR_FIXED_HEADERS: [&str; 8] = [
    "No",
    "NIP",
    "Nama",
    "Jabatan",
    "Golongan",
    "Status",
    "Gender",
    "Busana Muslimah",
];
const DAFTAR_FIXED_WIDTHS: [f64; 8] = [5.0, 20.0, 25.0, 18.0, 10.0, 8.0, 8.0, 14.0];

pub fn render_daftar_xlsx(report: &DaftarReport) -> AppResult<Vec<u8>> {
    let mut workbook = Workbook::new();
    let ws = workbook.add_worksheet();
    ws.set_landscape();
    // Fit all columns onto a single page width (height grows freely).
    ws.set_print_fit_to_pages(1, 0);

    let title_fmt = Format::new().set_bold().set_font_size(12.0);
    let meta_fmt = Format::new().set_font_size(9.0);
    let satker_fmt = Format::new().set_bold().set_font_size(10.0);
    let header_fmt = Format::new()
        .set_bold()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);
    let cell_fmt = Format::new()
        .set_font_size(8.0)
        .set_border(FormatBorder::Thin);
    let cell_center = Format::new()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    // Column widths: fixed leading cols + one per dynamic clothing column.
    for (i, w) in DAFTAR_FIXED_WIDTHS.iter().enumerate() {
        ws.set_column_width(i as u16, *w)
            .map_err(|e| bad_request(&e.to_string()))?;
    }
    for i in 0..report.columns.len() {
        ws.set_column_width((DAFTAR_FIXED_HEADERS.len() + i) as u16, 10.0)
            .map_err(|e| bad_request(&e.to_string()))?;
    }

    let mut row = write_header_block(ws, &report.header, &title_fmt, &meta_fmt)?;

    if report.satkers.is_empty() {
        ws.write_string_with_format(row, 0, "Tidak ada data.", &meta_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
    }

    for group in &report.satkers {
        // Satker heading.
        ws.write_string_with_format(row, 0, &group.satker_nama, &satker_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 1;

        // Header row: fixed + dynamic columns.
        for (c, h) in DAFTAR_FIXED_HEADERS.iter().enumerate() {
            ws.write_string_with_format(row, c as u16, *h, &header_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        for (i, col) in report.columns.iter().enumerate() {
            ws.write_string_with_format(
                row,
                (DAFTAR_FIXED_HEADERS.len() + i) as u16,
                col,
                &header_fmt,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        }
        row += 1;

        // Data rows.
        for (idx, r) in group.rows.iter().enumerate() {
            ws.write_number_with_format(row, 0, (idx + 1) as f64, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 1, &r.nip, &cell_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 2, &r.nama, &cell_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 3, &r.jabatan, &cell_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 4, &r.golongan, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 5, &r.status, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 6, &r.gender, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 7, &r.hijab, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            for (i, size) in r.sizes.iter().enumerate() {
                ws.write_string_with_format(
                    row,
                    (DAFTAR_FIXED_HEADERS.len() + i) as u16,
                    size,
                    &cell_center,
                )
                .map_err(|e| bad_request(&e.to_string()))?;
            }
            row += 1;
        }
        row += 1; // spacer between satkers
    }

    workbook
        .save_to_buffer()
        .map_err(|e| bad_request(&e.to_string()))
}

// ─────────────────────────── Rekap ───────────────────────────

pub fn render_rekap_xlsx(report: &RekapReport) -> AppResult<Vec<u8>> {
    let mut workbook = Workbook::new();
    let ws = workbook.add_worksheet();
    ws.set_landscape();
    ws.set_print_fit_to_pages(1, 0);

    let title_fmt = Format::new().set_bold().set_font_size(12.0);
    let meta_fmt = Format::new().set_font_size(9.0);
    let block_fmt = Format::new().set_bold().set_font_size(10.0);
    let header_fmt = Format::new()
        .set_bold()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);
    let cell_fmt = Format::new()
        .set_font_size(8.0)
        .set_border(FormatBorder::Thin);
    let cell_center = Format::new()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);
    let total_fmt = Format::new()
        .set_bold()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    ws.set_column_width(0, 5.0)
        .map_err(|e| bad_request(&e.to_string()))?; // No
    ws.set_column_width(1, 30.0)
        .map_err(|e| bad_request(&e.to_string()))?; // Satker

    let mut row = write_header_block(ws, &report.header, &title_fmt, &meta_fmt)?;

    if report.blocks.is_empty() {
        ws.write_string_with_format(row, 0, "Tidak ada data.", &meta_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
    }

    for block in &report.blocks {
        // Block heading: "Pakaian - GENDER".
        let heading = format!("{} - {}", block.pakaian_nama, gender_label(&block.gender));
        ws.write_string_with_format(row, 0, &heading, &block_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 1;

        // Header row: No | Satker | <ukuran...> | Jumlah
        ws.write_string_with_format(row, 0, "No", &header_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        ws.write_string_with_format(row, 1, "Satker", &header_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        for (i, label) in block.ukuran_labels.iter().enumerate() {
            let c = (2 + i) as u16;
            ws.set_column_width(c, 8.0)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, c, label, &header_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        let jumlah_col = (2 + block.ukuran_labels.len()) as u16;
        ws.set_column_width(jumlah_col, 8.0)
            .map_err(|e| bad_request(&e.to_string()))?;
        ws.write_string_with_format(row, jumlah_col, "Jumlah", &header_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 1;

        // Satker rows.
        for (idx, sr) in block.satkers.iter().enumerate() {
            ws.write_number_with_format(row, 0, (idx + 1) as f64, &cell_center)
                .map_err(|e| bad_request(&e.to_string()))?;
            ws.write_string_with_format(row, 1, &sr.satker_nama, &cell_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
            for (i, c) in sr.counts.iter().enumerate() {
                ws.write_number_with_format(row, (2 + i) as u16, *c as f64, &cell_center)
                    .map_err(|e| bad_request(&e.to_string()))?;
            }
            ws.write_number_with_format(row, jumlah_col, sr.total as f64, &total_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
            row += 1;
        }

        // Summary row.
        ws.write_string_with_format(row, 1, "Jumlah", &total_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        for (i, c) in block.summary.iter().enumerate() {
            ws.write_number_with_format(row, (2 + i) as u16, *c as f64, &total_fmt)
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        ws.write_number_with_format(row, jumlah_col, block.summary_total as f64, &total_fmt)
            .map_err(|e| bad_request(&e.to_string()))?;
        row += 2; // spacer between blocks
    }

    workbook
        .save_to_buffer()
        .map_err(|e| bad_request(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pakaian_dinas::models::{DaftarRow, DaftarSatkerGroup, RekapBlock, RekapSatkerRow};

    fn sample_daftar() -> DaftarReport {
        DaftarReport {
            header: ReportHeader {
                judul: "Pengadaan PDL 2026".to_string(),
                ..Default::default()
            },
            columns: vec!["Kaos".to_string(), "Topi".to_string(), "Sepatu".to_string()],
            satkers: vec![DaftarSatkerGroup {
                satker_nama: "Kejari A".to_string(),
                rows: vec![DaftarRow {
                    nip: "199".to_string(),
                    nama: "Budi".to_string(),
                    jabatan: "Jaksa".to_string(),
                    golongan: "III/a".to_string(),
                    status: "J".to_string(),
                    gender: "L".to_string(),
                    hijab: "T".to_string(),
                    sizes: vec!["L".to_string(), "M".to_string(), "42".to_string()],
                }],
            }],
        }
    }

    fn sample_rekap() -> RekapReport {
        RekapReport {
            header: ReportHeader {
                judul: "Pengadaan PDL 2026".to_string(),
                ..Default::default()
            },
            blocks: vec![RekapBlock {
                pakaian_nama: "Kaos".to_string(),
                ukuran_group: "BAJU".to_string(),
                gender: "L".to_string(),
                ukuran_labels: vec!["M".to_string(), "L".to_string()],
                satkers: vec![RekapSatkerRow {
                    satker_nama: "Kejari A".to_string(),
                    counts: vec![3, 2],
                    total: 5,
                }],
                summary: vec![3, 2],
                summary_total: 5,
            }],
        }
    }

    #[test]
    fn daftar_xlsx_produces_valid_workbook_bytes() {
        let bytes = render_daftar_xlsx(&sample_daftar()).expect("render daftar xlsx");
        // XLSX is a zip archive → starts with PK.
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn rekap_xlsx_produces_valid_workbook_bytes() {
        let bytes = render_rekap_xlsx(&sample_rekap()).expect("render rekap xlsx");
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn empty_reports_still_render() {
        let d = render_daftar_xlsx(&DaftarReport::default()).expect("empty daftar");
        let r = render_rekap_xlsx(&RekapReport::default()).expect("empty rekap");
        assert_eq!(&d[0..2], b"PK");
        assert_eq!(&r[0..2], b"PK");
    }
}
