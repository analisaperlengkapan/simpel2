//! # Pakaian Dinas PDF Export
//!
//! Renders the shared report view-models ([`DaftarReport`]/[`RekapReport`]) to
//! PDF — the exact same models the XLSX generator (`export.rs`) uses, so the
//! two formats never diverge in content.
//!
//! Layout parity = legacy simpelv1 `cetakDaftarTemplateV` (per-satker, dynamic
//! clothing columns) and `cetakRekapTemplateV` (per pakaian × gender, satker
//! rows × ukuran columns + summary). A4 **landscape**; column widths are always
//! scaled so the table fits the printable width ([`scale_to_fit`]), and the
//! cell font shrinks as the column count grows so wide tables stay readable.

use uuid::Uuid;

use super::models::{DaftarReport, LaporanFilter, RekapReport, ReportHeader};
use super::services::PakaianDinasService;
use crate::shared::error::AppResult;
use crate::shared::pdf::{PageGeometry, PdfBuilder};
use crate::shared::satker_scope::SatkerScope;

// ─── Page geometry (A4 landscape) ────────────────────────────────────────
const PAGE_W: f32 = 297.0;
const PAGE_H: f32 = 210.0;
const MARGIN_L: f32 = 12.0;
const MARGIN_R: f32 = 12.0;
const MARGIN_TOP: f32 = 200.0; // text origin near top (printpdf y-down from top)
const MARGIN_BOTTOM: f32 = 15.0;
const ROW_H: f32 = 6.0;
const HEADER_FONT: f32 = 9.0;
const TITLE_FONT: f32 = 13.0;

fn drawable_w() -> f32 {
    PAGE_W - MARGIN_L - MARGIN_R
}

fn geom(cell_font: f32) -> PageGeometry {
    PageGeometry {
        page_w: PAGE_W,
        page_h: PAGE_H,
        margin_l: MARGIN_L,
        margin_r: MARGIN_R,
        margin_top: MARGIN_TOP,
        margin_bottom: MARGIN_BOTTOM,
        row_h: ROW_H,
        cell_font,
        cell_pad_x: 1.0,
        cell_trunc_pad: 2.0,
        auto_paginate: true,
    }
}

/// Shrink the cell font as the table widens so many columns stay legible.
fn cell_font_for(cols: usize) -> f32 {
    match cols {
        0..=12 => 7.5,
        13..=18 => 6.5,
        19..=26 => 5.5,
        _ => 5.0,
    }
}

/// Scale a set of column widths down proportionally so their sum never exceeds
/// the drawable width — guarantees the table fits the page horizontally.
fn scale_to_fit(mut widths: Vec<f32>, total: f32) -> Vec<f32> {
    let sum: f32 = widths.iter().sum();
    if sum > total && sum > 0.0 {
        let s = total / sum;
        for w in &mut widths {
            *w *= s;
        }
    }
    widths
}

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

/// Title + periode + filters block.
fn write_header_block(pdf: &mut PdfBuilder, header: &ReportHeader) {
    let title = if header.judul.is_empty() {
        "Laporan Pakaian Dinas"
    } else {
        header.judul.as_str()
    };
    pdf.write_centered(title, TITLE_FONT, true);
    pdf.advance(7.0);
    if header.periode_mulai.is_some() || header.periode_selesai.is_some() {
        pdf.write_centered(
            &format!(
                "Periode: {} s/d {}",
                fmt_date(header.periode_mulai),
                fmt_date(header.periode_selesai)
            ),
            HEADER_FONT,
            false,
        );
        pdf.advance(6.0);
    }
    for (label, value) in &header.filters {
        pdf.write_text(
            &format!("{}: {}", label, value),
            HEADER_FONT,
            false,
            MARGIN_L,
        );
        pdf.advance(5.0);
    }
    pdf.advance(3.0);
}

// ─── async wrappers ──────────────────────────────────────────────────────

pub async fn generate_rekap_pdf(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
    scope: &SatkerScope,
) -> AppResult<Vec<u8>> {
    let report = service
        .build_rekap_report(pengajuan_id, filter, scope)
        .await?;
    render_rekap_pdf(&report)
}

pub async fn generate_daftar_pdf(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
    scope: &SatkerScope,
) -> AppResult<Vec<u8>> {
    let report = service
        .build_daftar_report(pengajuan_id, filter, scope)
        .await?;
    render_daftar_pdf(&report)
}

// ─── Daftar PDF ──────────────────────────────────────────────────────────

const DAFTAR_FIXED_HEADERS: [&str; 8] = [
    "No", "NIP", "Nama", "Jabatan", "Golongan", "Status", "Gender", "Hijab",
];
const DAFTAR_FIXED_WIDTHS: [f32; 8] = [8.0, 26.0, 40.0, 30.0, 14.0, 12.0, 12.0, 14.0];

pub fn render_daftar_pdf(report: &DaftarReport) -> AppResult<Vec<u8>> {
    let total_cols = DAFTAR_FIXED_HEADERS.len() + report.columns.len();
    let mut pdf = PdfBuilder::new(
        "Daftar Pegawai Pakaian Dinas",
        geom(cell_font_for(total_cols)),
    )?;
    write_header_block(&mut pdf, &report.header);

    if report.satkers.is_empty() {
        pdf.write_text("Tidak ada data.", HEADER_FONT, false, MARGIN_L);
        return pdf.finish();
    }

    // Column widths: fixed leading + dynamic clothing columns, scaled to fit.
    let n_dyn = report.columns.len();
    let mut widths: Vec<f32> = DAFTAR_FIXED_WIDTHS.to_vec();
    if n_dyn > 0 {
        let fixed_sum: f32 = DAFTAR_FIXED_WIDTHS.iter().sum();
        let each = ((drawable_w() - fixed_sum) / n_dyn as f32).max(10.0);
        widths.extend(std::iter::repeat_n(each, n_dyn));
    }
    let widths = scale_to_fit(widths, drawable_w());

    let mut header_cells: Vec<String> =
        DAFTAR_FIXED_HEADERS.iter().map(|s| s.to_string()).collect();
    header_cells.extend(report.columns.iter().cloned());

    let mut first = true;
    for group in &report.satkers {
        if !first {
            pdf.new_page();
        }
        first = false;

        pdf.write_text(&group.satker_nama, HEADER_FONT, true, MARGIN_L);
        pdf.advance(6.0);
        pdf.row(&widths, &header_cells, true, true);

        for (idx, r) in group.rows.iter().enumerate() {
            let mut cells = vec![
                (idx + 1).to_string(),
                r.nip.clone(),
                r.nama.clone(),
                r.jabatan.clone(),
                r.golongan.clone(),
                r.status.clone(),
                r.gender.clone(),
                r.hijab.clone(),
            ];
            cells.extend(r.sizes.iter().cloned());
            pdf.row(&widths, &cells, false, true);
        }
    }

    pdf.finish()
}

// ─── Rekap PDF ───────────────────────────────────────────────────────────

pub fn render_rekap_pdf(report: &RekapReport) -> AppResult<Vec<u8>> {
    let max_cols = report
        .blocks
        .iter()
        .map(|b| b.ukuran_labels.len() + 3)
        .max()
        .unwrap_or(3);
    let mut pdf = PdfBuilder::new("Rekap Ukuran Pakaian Dinas", geom(cell_font_for(max_cols)))?;
    write_header_block(&mut pdf, &report.header);

    if report.blocks.is_empty() {
        pdf.write_text("Tidak ada data.", HEADER_FONT, false, MARGIN_L);
        return pdf.finish();
    }

    let mut first = true;
    for block in &report.blocks {
        if !first {
            pdf.new_page();
        }
        first = false;

        pdf.write_text(
            &format!("{} - {}", block.pakaian_nama, gender_label(&block.gender)),
            HEADER_FONT,
            true,
            MARGIN_L,
        );
        pdf.advance(6.0);

        let n = block.ukuran_labels.len();
        // No(8) | Satker(50) | <ukuran...> | Jumlah(16), scaled to fit.
        let no_w = 8.0;
        let jumlah_w = 16.0;
        let satker_w = 50.0;
        let mut widths = vec![no_w, satker_w];
        if n > 0 {
            let each = ((drawable_w() - no_w - satker_w - jumlah_w) / n as f32).max(8.0);
            widths.extend(std::iter::repeat_n(each, n));
        }
        widths.push(jumlah_w);
        let widths = scale_to_fit(widths, drawable_w());

        let mut header_cells = vec!["No".to_string(), "Satker".to_string()];
        header_cells.extend(block.ukuran_labels.iter().cloned());
        header_cells.push("Jumlah".to_string());
        pdf.row(&widths, &header_cells, true, true);

        for (idx, sr) in block.satkers.iter().enumerate() {
            let mut cells = vec![(idx + 1).to_string(), sr.satker_nama.clone()];
            cells.extend(sr.counts.iter().map(|c| c.to_string()));
            cells.push(sr.total.to_string());
            pdf.row(&widths, &cells, false, true);
        }

        // Summary row.
        let mut summary = vec![String::new(), "Jumlah".to_string()];
        summary.extend(block.summary.iter().map(|c| c.to_string()));
        summary.push(block.summary_total.to_string());
        pdf.row(&widths, &summary, true, true);
    }

    pdf.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pakaian_dinas::models::{DaftarRow, DaftarSatkerGroup, RekapBlock, RekapSatkerRow};

    fn pdf_magic_ok(bytes: &[u8]) {
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..4], b"%PDF");
    }

    #[test]
    fn daftar_pdf_renders_with_dynamic_columns() {
        let report = DaftarReport {
            header: ReportHeader {
                judul: "Pengadaan PDL 2026".to_string(),
                ..Default::default()
            },
            columns: vec!["Kaos".into(), "Topi".into(), "Sepatu".into()],
            satkers: vec![DaftarSatkerGroup {
                satker_nama: "Kejari A".into(),
                rows: vec![DaftarRow {
                    nip: "199".into(),
                    nama: "Budi".into(),
                    jabatan: "Jaksa".into(),
                    golongan: "III/a".into(),
                    status: "J".into(),
                    gender: "L".into(),
                    hijab: "T".into(),
                    sizes: vec!["L".into(), "M".into(), "42".into()],
                }],
            }],
        };
        pdf_magic_ok(&render_daftar_pdf(&report).expect("render daftar pdf"));
    }

    #[test]
    fn rekap_pdf_renders_with_satker_rows() {
        let report = RekapReport {
            header: ReportHeader::default(),
            blocks: vec![RekapBlock {
                pakaian_nama: "Kaos".into(),
                ukuran_group: "BAJU".into(),
                gender: "L".into(),
                ukuran_labels: vec!["M".into(), "L".into()],
                satkers: vec![RekapSatkerRow {
                    satker_nama: "Kejari A".into(),
                    counts: vec![3, 2],
                    total: 5,
                }],
                summary: vec![3, 2],
                summary_total: 5,
            }],
        };
        pdf_magic_ok(&render_rekap_pdf(&report).expect("render rekap pdf"));
    }

    #[test]
    fn many_columns_still_fit_within_drawable_width() {
        // 30 dynamic columns → widths must be scaled so the sum never exceeds
        // the drawable width.
        let n = 30usize;
        let mut widths: Vec<f32> = DAFTAR_FIXED_WIDTHS.to_vec();
        let each = ((drawable_w() - DAFTAR_FIXED_WIDTHS.iter().sum::<f32>()) / n as f32).max(10.0);
        widths.extend(std::iter::repeat_n(each, n));
        let fitted = scale_to_fit(widths, drawable_w());
        let sum: f32 = fitted.iter().sum();
        assert!(sum <= drawable_w() + 0.01, "sum {sum} exceeds drawable");
    }

    #[test]
    fn empty_reports_render() {
        pdf_magic_ok(&render_daftar_pdf(&DaftarReport::default()).expect("empty daftar"));
        pdf_magic_ok(&render_rekap_pdf(&RekapReport::default()).expect("empty rekap"));
    }
}
