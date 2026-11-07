//! # Pakaian Dinas XLSX Export
//!
//! Renders the shared report view-models ([`DaftarReport`] / [`RekapReport`],
//! built in `services.rs`) to XLSX, in the shape the legacy simpelv1 export has.
//! That export is the Blade template rendered to HTML and converted, so the
//! sheet is plain: one sheet named "Worksheet", the title in bold 18, the
//! instance line in bold 13.5, the period line, one `Label:Value` row per
//! filter, then the tables with no borders and no styling, merged where the
//! HTML spans cells. Rekap prints one table per (pakaian × gender) with a blank
//! row between tables; daftar prints a bold satker heading above each satker's
//! table.
//!
//! Differences from the legacy file, each one a defect of the conversion rather
//! than a design choice: the NIP is written as text (legacy writes the literal
//! characters `&#039;` in front of it), the first two rekap columns get a
//! readable width (legacy sets 0.4 and 1.7 characters, which hides the number
//! column and cuts every satker name), the daftar columns get widths (legacy
//! leaves the default, which cuts a NIP), and the heading reads "Busana
//! Muslimah" (legacy: "Mulsimah").

use super::models::{DaftarReport, LaporanFilter, RekapReport, ReportHeader};
use super::services::PakaianDinasService;
use crate::shared::error::*;
use crate::shared::satker_scope::SatkerScope;
use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet, XlsxError};
use uuid::Uuid;

// ─────────────────────────── async wrappers ───────────────────────────

pub async fn generate_rekap_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
    scope: &SatkerScope,
) -> AppResult<Vec<u8>> {
    let report = service
        .build_rekap_report(pengajuan_id, filter, scope)
        .await?;
    render_rekap_xlsx(&report)
}

pub async fn generate_daftar_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
    scope: &SatkerScope,
) -> AppResult<Vec<u8>> {
    let report = service
        .build_daftar_report(pengajuan_id, filter, scope)
        .await?;
    render_daftar_xlsx(&report)
}

// ─────────────────────────── helpers ───────────────────────────

fn xlsx_err(e: XlsxError) -> AppError {
    AppError::Internal(format!("Gagal membuat Excel: {e}"))
}

/// A table cell as the HTML converter types it: a whole number becomes a
/// number, anything else stays text. (`-` and `L` stay text; `34` is a number.)
fn write_cell(ws: &mut Worksheet, row: u32, col: u16, text: &str) -> Result<(), XlsxError> {
    match text.parse::<i64>() {
        Ok(n) if text == n.to_string() => ws.write_number(row, col, n as f64).map(|_| ()),
        _ => ws.write_string(row, col, text).map(|_| ()),
    }
}

/// Merge `first..=last` on one row when it spans more than one cell; a single
/// cell is just written.
fn merged(
    ws: &mut Worksheet,
    (r1, c1): (u32, u16),
    (r2, c2): (u32, u16),
    text: &str,
    format: &Format,
) -> Result<(), XlsxError> {
    if (r1, c1) == (r2, c2) {
        ws.write_string_with_format(r1, c1, text, format)
            .map(|_| ())
    } else {
        ws.merge_range(r1, c1, r2, c2, text, format).map(|_| ())
    }
}

/// Title, instance and period lines, then one row per filter. Returns the first
/// free row.
fn write_header_block(ws: &mut Worksheet, header: &ReportHeader) -> Result<u32, XlsxError> {
    let h2 = Format::new().set_bold().set_font_size(18.0);
    let h3 = Format::new().set_bold().set_font_size(13.5);
    ws.write_string_with_format(0, 0, &header.judul, &h2)?;
    ws.write_string_with_format(1, 0, &header.inst_nama, &h3)?;
    ws.write_string(2, 0, header.periode_line())?;
    let mut row = 3;
    for (label, value) in &header.filters {
        ws.write_string(row, 0, format!("{label}:{value}"))?;
        row += 1;
    }
    Ok(row)
}

// ─────────────────────────── Daftar ───────────────────────────

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
const DAFTAR_FIXED_WIDTHS: [f64; 8] = [5.0, 21.0, 28.0, 28.0, 10.0, 8.0, 8.0, 17.0];

pub fn render_daftar_xlsx(report: &DaftarReport) -> AppResult<Vec<u8>> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Worksheet").map_err(xlsx_err)?;
    build_daftar(ws, report).map_err(xlsx_err)?;
    wb.save_to_buffer().map_err(xlsx_err)
}

fn build_daftar(ws: &mut Worksheet, report: &DaftarReport) -> Result<(), XlsxError> {
    let h3 = Format::new().set_bold().set_font_size(13.5);
    let centered = Format::new().set_align(FormatAlign::Center);
    let n_cols = DAFTAR_FIXED_HEADERS.len() + report.columns.len();
    let last_col = (n_cols - 1) as u16;

    let mut row = write_header_block(ws, &report.header)?;
    for (i, w) in DAFTAR_FIXED_WIDTHS.iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
    }
    for i in DAFTAR_FIXED_HEADERS.len()..n_cols {
        ws.set_column_width(i as u16, 14.0)?;
    }

    for (si, group) in report.satkers.iter().enumerate() {
        if si > 0 {
            row += 1; // the blank row the legacy export leaves between satker
        }
        ws.write_string_with_format(row, 0, &group.satker_nama, &h3)?;
        row += 1;
        for (i, h) in DAFTAR_FIXED_HEADERS.iter().enumerate() {
            ws.write_string(row, i as u16, *h)?;
        }
        for (i, h) in report.columns.iter().enumerate() {
            ws.write_string(row, (DAFTAR_FIXED_HEADERS.len() + i) as u16, h)?;
        }
        row += 1;

        if group.rows.is_empty() {
            merged(ws, (row, 0), (row, last_col), "Data Kosong", &centered)?;
            row += 1;
            continue;
        }
        for (ri, r) in group.rows.iter().enumerate() {
            ws.write_number(row, 0, (ri + 1) as f64)?;
            // Text, so a NIP is never read as a number and rounded.
            ws.write_string(row, 1, &r.nip)?;
            ws.write_string(row, 2, &r.nama)?;
            ws.write_string(row, 3, &r.jabatan)?;
            ws.write_string(row, 4, &r.golongan)?;
            ws.write_string(row, 5, &r.status)?;
            ws.write_string(row, 6, &r.gender)?;
            ws.write_string(row, 7, &r.hijab)?;
            for (i, size) in r.sizes.iter().enumerate() {
                write_cell(ws, row, (DAFTAR_FIXED_HEADERS.len() + i) as u16, size)?;
            }
            row += 1;
        }
    }
    Ok(())
}

// ─────────────────────────── Rekap ───────────────────────────

pub fn render_rekap_xlsx(report: &RekapReport) -> AppResult<Vec<u8>> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Worksheet").map_err(xlsx_err)?;
    build_rekap(ws, report).map_err(xlsx_err)?;
    wb.save_to_buffer().map_err(xlsx_err)
}

fn build_rekap(ws: &mut Worksheet, report: &RekapReport) -> Result<(), XlsxError> {
    let plain = Format::new();
    let mut row = write_header_block(ws, &report.header)?;
    ws.set_column_width(0, 6.0)?;
    ws.set_column_width(1, 40.0)?;

    for (bi, block) in report.blocks.iter().enumerate() {
        if bi > 0 {
            row += 1; // blank row between tables
        }
        let n = block.ukuran_labels.len() as u16;
        let total_col = 2 + n;

        // Two heading rows: No / Satker / "<pakaian> - <L|P>" / Jumlah, then sizes.
        merged(ws, (row, 0), (row + 1, 0), "No", &plain)?;
        merged(ws, (row, 1), (row + 1, 1), "Satker", &plain)?;
        if n > 0 {
            let title = format!("{} - {}", block.pakaian_nama, block.gender);
            merged(ws, (row, 2), (row, 1 + n), &title, &plain)?;
        }
        merged(ws, (row, total_col), (row + 1, total_col), "Jumlah", &plain)?;
        for (i, label) in block.ukuran_labels.iter().enumerate() {
            write_cell(ws, row + 1, 2 + i as u16, label)?;
        }
        row += 2;

        for (i, s) in block.satkers.iter().enumerate() {
            ws.write_number(row, 0, (i + 1) as f64)?;
            ws.write_string(row, 1, &s.satker_nama)?;
            for (k, count) in s.counts.iter().enumerate() {
                ws.write_number(row, 2 + k as u16, *count as f64)?;
            }
            ws.write_number(row, total_col, s.total as f64)?;
            row += 1;
        }

        merged(ws, (row, 0), (row, 1), "Jumlah", &plain)?;
        for (k, total) in block.summary.iter().enumerate() {
            ws.write_number(row, 2 + k as u16, *total as f64)?;
        }
        ws.write_number(row, total_col, block.summary_total as f64)?;
        row += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::models::{
        DaftarRow, DaftarSatkerGroup, RekapBlock, RekapSatkerRow, ReportHeader,
    };
    use super::*;
    use std::io::Read;

    fn sheet_xml(bytes: &[u8]) -> (String, String) {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip");
        let mut read = |name: &str| {
            let mut s = String::new();
            if let Ok(mut f) = z.by_name(name) {
                f.read_to_string(&mut s).unwrap();
            }
            s
        };
        (
            read("xl/worksheets/sheet1.xml"),
            read("xl/sharedStrings.xml"),
        )
    }

    fn header() -> ReportHeader {
        ReportHeader {
            judul: "Kebutuhan PDH".to_string(),
            inst_nama: "WILAYAH KEJAKSAAN TINGGI DKI JAKARTA".to_string(),
            periode_mulai: chrono::NaiveDate::from_ymd_opt(2026, 11, 1),
            periode_selesai: chrono::NaiveDate::from_ymd_opt(2026, 12, 31),
            filters: vec![("Jenis Kelamin".to_string(), "L".to_string())],
        }
    }

    #[test]
    fn rekap_has_the_legacy_cell_layout() {
        let report = RekapReport {
            header: header(),
            blocks: vec![RekapBlock {
                pakaian_nama: "Celana PDH".to_string(),
                ukuran_group: "CELANA".to_string(),
                gender: "L".to_string(),
                ukuran_labels: vec!["32".to_string(), "34".to_string()],
                satkers: vec![
                    RekapSatkerRow {
                        satker_nama: "KEJARI A".to_string(),
                        counts: vec![0, 1],
                        total: 1,
                    },
                    RekapSatkerRow {
                        satker_nama: "KEJARI B".to_string(),
                        counts: vec![0, 0],
                        total: 0,
                    },
                ],
                summary: vec![0, 1],
                summary_total: 1,
            }],
        };
        let (sheet, strings) = sheet_xml(&render_rekap_xlsx(&report).unwrap());
        // Title, instance, period, one Label:Value row per filter.
        for expected in [
            "Kebutuhan PDH",
            "WILAYAH KEJAKSAAN TINGGI DKI JAKARTA",
            "Periode: 01-Nopember-2026 S/D 31-Desember-2026",
            "Jenis Kelamin:L",
            "Celana PDH - L",
        ] {
            assert!(strings.contains(expected), "missing {expected}: {strings}");
        }
        // Header rows 5-6 (after 4 heading rows): No/Satker/Jumlah span two rows,
        // the pakaian title spans both size columns, "Jumlah" spans No+Satker.
        for merge in ["A5:A6", "B5:B6", "C5:D5", "E5:E6", "A9:B9"] {
            assert!(
                sheet.contains(&format!("ref=\"{merge}\"")),
                "missing merge {merge}: {sheet}"
            );
        }
        // Sizes that are numbers are numbers.
        assert!(
            sheet.contains("<c r=\"C6\"><v>32</v></c>"),
            "size label as a number: {sheet}"
        );
        // Every satker has its row, zeros included.
        assert!(strings.contains("KEJARI B"));
    }

    #[test]
    fn daftar_has_satker_headings_a_text_nip_and_data_kosong() {
        let report = DaftarReport {
            header: header(),
            columns: vec!["Celana PDH".to_string()],
            satkers: vec![
                DaftarSatkerGroup {
                    satker_nama: "KEJARI A".to_string(),
                    rows: vec![DaftarRow {
                        nip: "200000000000000001".to_string(),
                        nama: "Pegawai Satu".to_string(),
                        jabatan: "Operator".to_string(),
                        golongan: "III/a".to_string(),
                        status: "J".to_string(),
                        gender: "L".to_string(),
                        hijab: "T".to_string(),
                        sizes: vec!["34".to_string()],
                    }],
                },
                DaftarSatkerGroup {
                    satker_nama: "KEJARI KOSONG".to_string(),
                    rows: vec![],
                },
            ],
        };
        let (sheet, strings) = sheet_xml(&render_daftar_xlsx(&report).unwrap());
        assert!(
            strings.contains("200000000000000001"),
            "NIP kept whole, as text"
        );
        assert!(
            !strings.contains("&#039;"),
            "no stray entity in front of the NIP"
        );
        assert!(strings.contains("Busana Muslimah"));
        assert!(strings.contains("Data Kosong"));
        assert!(strings.contains("KEJARI KOSONG"));
        // "Data Kosong" spans every column: 8 fixed + 1 size column = A..I.
        assert!(
            sheet.contains("ref=\"A11:I11\""),
            "Data Kosong merged across all columns: {sheet}"
        );
    }
}
