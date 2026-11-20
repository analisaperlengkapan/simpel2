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

/// [`write_cell`], with `wrap` on a text value (a number stays a number).
fn write_cell_wrapped(
    ws: &mut Worksheet,
    row: u32,
    col: u16,
    text: &str,
    wrap: &Format,
) -> Result<(), XlsxError> {
    match text.parse::<i64>() {
        Ok(n) if text == n.to_string() => ws.write_number(row, col, n as f64).map(|_| ()),
        _ => ws
            .write_string_with_format(row, col, text, wrap)
            .map(|_| ()),
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

// ─────────────────────────── paper and widths ───────────────────────────

/// The smallest scale a sheet may be printed at before it moves to the next paper size up. 65 % of
/// 11 pt is still 7 pt, which reads.
const MIN_PRINT_SCALE: u16 = 65;
/// Excel's paper size codes, and their sheet width in millimetres (landscape).
const PAPER_A4: u8 = 9;
const PAPER_A3: u8 = 8;
const A4_WIDTH_MM: f64 = 297.0;
const A3_WIDTH_MM: f64 = 420.0;
/// Page margins, left and right, in inches (and the same in millimetres).
const MARGIN_IN: f64 = 0.4;
const MARGIN_MM: f64 = MARGIN_IN * 25.4;
/// Pixels of a column of width `w` characters at Excel's default font (Calibri 11, 7 px a
/// character, 5 px of padding), and millimetres of a pixel at 96 dpi.
const PX_PER_CHAR: f64 = 7.0;
const PX_PADDING: f64 = 5.0;
const MM_PER_PX: f64 = 25.4 / 96.0;
/// What is left of the page width after the scale is rounded down, so a column is never cut by
/// rounding or by a printer that wastes a millimetre.
const SCALE_SAFETY: f64 = 0.97;

/// Printed width of the columns at 100 %, in millimetres.
fn natural_width_mm(widths: &[f64]) -> f64 {
    widths
        .iter()
        .map(|w| w * PX_PER_CHAR + PX_PADDING)
        .sum::<f64>()
        * MM_PER_PX
}

/// The scale (percent) at which `widths` fit one page of `paper_mm` across, never above 100.
fn scale_for(widths: &[f64], paper_mm: f64) -> u16 {
    let printable = paper_mm - 2.0 * MARGIN_MM;
    let pct = (printable / natural_width_mm(widths) * 100.0 * SCALE_SAFETY).floor();
    pct.clamp(10.0, 100.0) as u16
}

/// Paper and print scale for a sheet of these column widths: A4 while it prints at no less than
/// [`MIN_PRINT_SCALE`], A3 once it does not (and then at whatever scale fits A3).
///
/// An EXPLICIT scale, not "fit to one page wide": Excel documents that fit-to-pages ignores every
/// manual page break, and each table (each satker, each rekap block) must start its own page.
fn page_plan(widths: &[f64]) -> (u8, u16) {
    let on_a4 = scale_for(widths, A4_WIDTH_MM);
    if on_a4 >= MIN_PRINT_SCALE {
        (PAPER_A4, on_a4)
    } else {
        (PAPER_A3, scale_for(widths, A3_WIDTH_MM))
    }
}

/// Landscape, the paper and scale for the sheet's width, tight margins, a page number, and a page
/// break before each table (each table starts a page, as in the PDF). Rows continue onto further
/// pages by themselves.
fn setup_page(ws: &mut Worksheet, widths: &[f64], breaks: &[u32]) -> Result<(), XlsxError> {
    let (paper, scale) = page_plan(widths);
    ws.set_landscape();
    ws.set_paper_size(paper);
    ws.set_print_scale(scale);
    ws.set_margins(MARGIN_IN, MARGIN_IN, 0.5, 0.5, 0.3, 0.3);
    ws.set_footer("&C&8Halaman &P dari &N");
    ws.set_page_breaks(breaks)?;
    Ok(())
}

/// A column wide enough for its longest text, no narrower than `min` and no wider than `max`
/// (a longer text wraps inside the cell instead of widening the sheet past the paper).
fn width_for(longest_chars: usize, min: f64, max: f64) -> f64 {
    ((longest_chars as f64) + 2.0).clamp(min, max)
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
/// Narrowest and widest each fixed daftar column may be; the width in between follows its content.
const DAFTAR_FIXED_BOUNDS: [(f64, f64); 8] = [
    (5.0, 5.0),
    (20.0, 20.0),
    (14.0, 36.0),
    (12.0, 36.0),
    (10.0, 12.0),
    (8.0, 8.0),
    (8.0, 8.0),
    (17.0, 17.0),
];
/// Widest a size/garment column may grow before its text wraps (a chest name is up to 100 characters).
const DAFTAR_SIZE_MAX_WIDTH: f64 = 28.0;

/// The column widths of a daftar sheet: the fixed columns and one per garment, each from its own
/// content (header included) within bounds.
fn daftar_widths(report: &DaftarReport) -> Vec<f64> {
    let n_fixed = DAFTAR_FIXED_HEADERS.len();
    let mut longest: Vec<usize> = DAFTAR_FIXED_HEADERS
        .iter()
        .map(|h| h.chars().count())
        .chain(report.columns.iter().map(|h| h.chars().count()))
        .collect();
    for r in report.satkers.iter().flat_map(|g| g.rows.iter()) {
        for (i, text) in [&r.nip, &r.nama, &r.jabatan, &r.golongan]
            .iter()
            .enumerate()
        {
            longest[i + 1] = longest[i + 1].max(text.chars().count());
        }
        for (i, s) in r.sizes.iter().enumerate() {
            longest[n_fixed + i] = longest[n_fixed + i].max(s.chars().count());
        }
    }
    longest
        .iter()
        .enumerate()
        .map(|(i, len)| match DAFTAR_FIXED_BOUNDS.get(i) {
            Some((lo, hi)) => width_for(*len, *lo, *hi),
            None => width_for(*len, 8.0, DAFTAR_SIZE_MAX_WIDTH),
        })
        .collect()
}

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
    let widths = daftar_widths(report);
    for (i, w) in widths.iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
    }
    // Text that outgrows its column wraps and the row grows to hold it, rather than the sheet growing
    // wider than the paper.
    let wrap = Format::new().set_text_wrap();
    let mut breaks: Vec<u32> = Vec::new();

    for (si, group) in report.satkers.iter().enumerate() {
        if si > 0 {
            row += 1; // the blank row the legacy export leaves between satker
            breaks.push(row); // each satker starts a page, as in the PDF
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
            ws.write_string_with_format(row, 2, &r.nama, &wrap)?;
            ws.write_string_with_format(row, 3, &r.jabatan, &wrap)?;
            ws.write_string(row, 4, &r.golongan)?;
            ws.write_string(row, 5, &r.status)?;
            ws.write_string(row, 6, &r.gender)?;
            ws.write_string(row, 7, &r.hijab)?;
            for (i, size) in r.sizes.iter().enumerate() {
                write_cell_wrapped(
                    ws,
                    row,
                    (DAFTAR_FIXED_HEADERS.len() + i) as u16,
                    size,
                    &wrap,
                )?;
            }
            row += 1;
        }
    }
    setup_page(ws, &widths, &breaks)
}

// ─────────────────────────── Rekap ───────────────────────────

pub fn render_rekap_xlsx(report: &RekapReport) -> AppResult<Vec<u8>> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Worksheet").map_err(xlsx_err)?;
    build_rekap(ws, report).map_err(xlsx_err)?;
    wb.save_to_buffer().map_err(xlsx_err)
}

/// Column widths of the rekap sheet. All tables share the columns, so each column is as wide as the
/// widest thing any table puts in it: a size label or count, or the row total (which sits in a
/// different column in a table with a different number of sizes).
fn rekap_widths(report: &RekapReport) -> Vec<f64> {
    let longest_satker = report
        .blocks
        .iter()
        .flat_map(|b| b.satkers.iter().map(|s| s.satker_nama.chars().count()))
        .max()
        .unwrap_or(0);
    let mut widths = vec![6.0, width_for(longest_satker, 24.0, 45.0)];
    for block in &report.blocks {
        let digits = |n: i64| n.to_string().chars().count();
        for (i, label) in block.ukuran_labels.iter().enumerate() {
            let counts = block
                .satkers
                .iter()
                .map(|s| digits(s.counts[i]))
                .chain(std::iter::once(digits(block.summary[i])))
                .max()
                .unwrap_or(1);
            set_max(
                &mut widths,
                2 + i,
                width_for(label.chars().count().max(counts), 5.0, 12.0),
            );
        }
        let totals = block
            .satkers
            .iter()
            .map(|s| digits(s.total))
            .chain(std::iter::once(digits(block.summary_total)))
            .max()
            .unwrap_or(1);
        set_max(
            &mut widths,
            2 + block.ukuran_labels.len(),
            width_for(totals.max("Jumlah".len()), 8.0, 12.0),
        );
    }
    widths
}

fn set_max(widths: &mut Vec<f64>, at: usize, w: f64) {
    if widths.len() <= at {
        widths.resize(at + 1, 0.0);
    }
    if w > widths[at] {
        widths[at] = w;
    }
}

fn build_rekap(ws: &mut Worksheet, report: &RekapReport) -> Result<(), XlsxError> {
    let plain = Format::new();
    let wrap = Format::new().set_text_wrap();
    let mut row = write_header_block(ws, &report.header)?;
    let widths = rekap_widths(report);
    for (i, w) in widths.iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
    }
    let mut breaks: Vec<u32> = Vec::new();

    for (bi, block) in report.blocks.iter().enumerate() {
        if bi > 0 {
            row += 1; // blank row between tables
            breaks.push(row); // each table starts a page, as in the PDF
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
            ws.write_string_with_format(row, 1, &s.satker_nama, &wrap)?;
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
    setup_page(ws, &widths, &breaks)
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

    #[test]
    fn paper_and_scale_follow_the_width_a4_while_it_reads_a3_once_it_does_not() {
        // A sheet that fits A4 at full size prints at 100 %.
        assert_eq!(page_plan(&[10.0, 20.0, 30.0]), (PAPER_A4, 100));
        // Wider: A4 at a reduced scale while that stays legible...
        let (paper, scale) = page_plan(&[8.0; 25]);
        assert_eq!(paper, PAPER_A4);
        assert!((MIN_PRINT_SCALE..100).contains(&scale), "{scale}");
        // ...then A3, whose scale is bigger because the page is wider.
        let wide = [8.0; 50];
        let (paper, scale) = page_plan(&wide);
        assert_eq!(
            paper, PAPER_A3,
            "A4 would print this below {MIN_PRINT_SCALE} %"
        );
        assert!(scale > scale_for(&wide, A4_WIDTH_MM), "{scale}");
        // Whatever the scale, the printed width is inside the page: nothing runs past the edge.
        for w in [vec![10.0; 3], vec![8.0; 25], vec![8.0; 50], vec![8.0; 200]] {
            let (p, s) = page_plan(&w);
            let page = if p == PAPER_A3 {
                A3_WIDTH_MM
            } else {
                A4_WIDTH_MM
            };
            let printed = natural_width_mm(&w) * f64::from(s) / 100.0;
            assert!(
                printed <= page - 2.0 * MARGIN_MM,
                "{printed} mm on {page} mm paper"
            );
        }
    }

    fn pegawai(nama_dada: &str, garments: usize) -> DaftarRow {
        DaftarRow {
            nip: "200000000000000001".to_string(),
            nama: "Pegawai Satu".to_string(),
            jabatan: "Operator".to_string(),
            golongan: "III/a".to_string(),
            status: "J".to_string(),
            gender: "L".to_string(),
            hijab: "T".to_string(),
            sizes: (0..garments)
                .map(|g| {
                    if g == 0 {
                        nama_dada.to_string()
                    } else {
                        "42".to_string()
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn daftar_sheet_is_set_up_for_the_paper_and_wraps_long_text() {
        let report = DaftarReport {
            header: header(),
            columns: vec!["Nama Dada".to_string(), "Sepatu PDH".to_string()],
            satkers: vec![
                DaftarSatkerGroup {
                    satker_nama: "KEJARI A".to_string(),
                    rows: vec![pegawai(&"BUDI SANTOSO ".repeat(8), 2)],
                },
                DaftarSatkerGroup {
                    satker_nama: "KEJARI B".to_string(),
                    rows: vec![pegawai("AGUS", 2)],
                },
            ],
        };
        let (sheet, _) = sheet_xml(&render_daftar_xlsx(&report).unwrap());
        for expected in ["orientation=\"landscape\"", "paperSize=\"9\"", "<rowBreaks"] {
            assert!(sheet.contains(expected), "missing {expected}: {sheet}");
        }
        // An explicit scale, NOT fit-to-pages: Excel ignores manual page breaks in fit-to mode, and
        // each satker must start its own page.
        assert!(
            !sheet.contains("fitToPage"),
            "fit-to-pages would make Excel ignore the page breaks: {sheet}"
        );
        // The chest-name column is capped (28 + the 2 of air within the 28 ceiling), not as wide as the text.
        assert!(
            !sheet.contains("width=\"100"),
            "a 100-character text must wrap, not widen the sheet: {sheet}"
        );
    }

    #[test]
    fn a_very_wide_daftar_moves_to_a3() {
        let n = 25;
        let report = DaftarReport {
            header: header(),
            columns: (0..n).map(|i| format!("Pakaian nomor {i}")).collect(),
            satkers: vec![DaftarSatkerGroup {
                satker_nama: "KEJARI A".to_string(),
                rows: vec![pegawai("M", n)],
            }],
        };
        let (sheet, _) = sheet_xml(&render_daftar_xlsx(&report).unwrap());
        assert!(
            sheet.contains("paperSize=\"8\""),
            "A3 for a sheet this wide: {sheet}"
        );
        assert!(
            sheet.contains("scale=\""),
            "an explicit scale keeps it inside the page: {sheet}"
        );
    }

    #[test]
    fn rekap_columns_are_as_wide_as_their_figures_and_each_table_starts_a_page() {
        let blok = |pakaian: &str, n: usize| RekapBlock {
            pakaian_nama: pakaian.to_string(),
            ukuran_group: "CELANA".to_string(),
            gender: "L".to_string(),
            ukuran_labels: (0..n).map(|i| (26 + i).to_string()).collect(),
            satkers: vec![RekapSatkerRow {
                satker_nama: "KEJARI A".to_string(),
                counts: vec![3; n],
                total: 3 * n as i64,
            }],
            summary: vec![3; n],
            summary_total: 3 * n as i64,
        };
        let report = RekapReport {
            header: header(),
            blocks: vec![blok("Celana PDH", 27), blok("Celana PDL", 10)],
        };
        let widths = rekap_widths(&report);
        // No, Satker, 27 sizes, Jumlah: size columns are narrow (5), not the 14 the old fixed width gave.
        assert_eq!(widths.len(), 2 + 27 + 1);
        assert!(
            (widths[2] - 5.0).abs() < 0.01,
            "a size column holds a two-digit number in 5 characters"
        );
        assert!(widths[1] >= 24.0, "the satker name column keeps room");
        let (sheet, _) = sheet_xml(&render_rekap_xlsx(&report).unwrap());
        assert!(
            sheet.contains("<rowBreaks"),
            "the second table starts a page: {sheet}"
        );
        assert!(sheet.contains("paperSize=\"9\""), "{sheet}");
    }
}
