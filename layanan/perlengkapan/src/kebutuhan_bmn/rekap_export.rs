//! # Kebutuhan BMN — Laporan Rekap export (XLSX + PDF)
//!
//! Renders the cross-campaign recap ([`RekapLaporanRow`]) that backs the
//! `/kebutuhan-bmn/laporan` page. Both formats render the *same* rows the
//! JSON endpoint returns — and all three go through the same scoped
//! repository query — so an export can never reveal more than the table the
//! user is looking at.
//!
//! Distinct from `pdf_laporan.rs`, which renders the per-satker *Laporan Hasil
//! Analisis* (feasibility, one satker). This one is the flat multi-satker
//! recap: one row per requested item.

use super::models::{RekapLaporanFilter, RekapLaporanRow};
use crate::shared::error::AppResult;
use crate::shared::pdf::{PageGeometry, PdfBuilder};
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};

/// Column headers, shared by both renderers so the formats stay in lock-step.
const HEADERS: [&str; 8] = [
    "Tahun",
    "Satker",
    "Kode Barang",
    "Nama Barang",
    "Satuan",
    "Jumlah Diusulkan",
    "Jumlah Disetujui",
    "Status",
];

fn subtitle(filter: &RekapLaporanFilter) -> String {
    let tahun = filter
        .tahun
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Semua Tahun".to_string());
    match filter.status_kode {
        Some(k) => format!("Tahun: {} — Status: {}", tahun, k),
        None => format!("Tahun: {} — Status: Semua", tahun),
    }
}

fn cells(r: &RekapLaporanRow) -> [String; 8] {
    [
        r.tahun.to_string(),
        r.satker_nama.clone().unwrap_or_else(|| r.satker_id.clone()),
        r.kode_barang.clone().unwrap_or_else(|| "-".to_string()),
        r.nama_barang.clone(),
        r.satuan.clone().unwrap_or_else(|| "-".to_string()),
        r.jumlah.to_string(),
        r.jml_setuju.to_string(),
        r.status_nama
            .clone()
            .unwrap_or_else(|| r.status_kode.to_string()),
    ]
}

// ─────────────────────────── XLSX ───────────────────────────

pub fn render_rekap_xlsx(
    rows: &[RekapLaporanRow],
    filter: &RekapLaporanFilter,
) -> AppResult<Vec<u8>> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Rekap Kebutuhan BMN")?;
    // Landscape + fit-to-1-page-wide, matching `pakaian_dinas/xlsx_export.rs`,
    // so the table never spills past the printable width.
    sheet.set_landscape();
    sheet.set_print_fit_to_pages(1, 0);

    let title_fmt = Format::new().set_bold().set_font_size(14.0);
    let head_fmt = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let cell_fmt = Format::new().set_border(FormatBorder::Thin);
    let num_fmt = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Right);

    sheet.write_with_format(0, 0, "Laporan Kebutuhan BMN", &title_fmt)?;
    sheet.write(1, 0, subtitle(filter))?;

    let head_row = 3;
    for (c, h) in HEADERS.iter().enumerate() {
        sheet.write_with_format(head_row, c as u16, *h, &head_fmt)?;
    }

    for (i, r) in rows.iter().enumerate() {
        let row = head_row + 1 + i as u32;
        let v = cells(r);
        for (c, val) in v.iter().enumerate() {
            // Quantity columns right-align as numbers; the rest are text.
            let is_num = c == 5 || c == 6;
            let fmt = if is_num { &num_fmt } else { &cell_fmt };
            if is_num {
                sheet.write_number_with_format(
                    row,
                    c as u16,
                    val.parse::<f64>().unwrap_or(0.0),
                    fmt,
                )?;
            } else {
                sheet.write_with_format(row, c as u16, val.as_str(), fmt)?;
            }
        }
    }

    // Totals row — the report's reason for existing is the rollup.
    let total_row = head_row + 1 + rows.len() as u32;
    let total_fmt = Format::new().set_bold().set_border(FormatBorder::Thin);
    sheet.write_with_format(total_row, 4, "TOTAL", &total_fmt)?;
    sheet.write_number_with_format(
        total_row,
        5,
        rows.iter().map(|r| r.jumlah as f64).sum::<f64>(),
        &total_fmt,
    )?;
    sheet.write_number_with_format(
        total_row,
        6,
        rows.iter().map(|r| r.jml_setuju as f64).sum::<f64>(),
        &total_fmt,
    )?;

    for (c, w) in [8.0, 28.0, 14.0, 34.0, 10.0, 16.0, 16.0, 22.0]
        .iter()
        .enumerate()
    {
        sheet.set_column_width(c as u16, *w)?;
    }

    Ok(workbook.save_to_buffer()?)
}

// ─────────────────────────── PDF ───────────────────────────

/// A4 **landscape** — eight columns do not fit portrait at a legible size.
fn geom() -> PageGeometry {
    PageGeometry {
        page_w: 297.0,
        page_h: 210.0,
        margin_l: 12.0,
        margin_r: 12.0,
        margin_top: 198.0,
        margin_bottom: 14.0,
        row_h: 5.0,
        cell_font: 7.0,
        cell_pad_x: 1.0,
        cell_trunc_pad: 2.0,
        auto_paginate: true,
    }
}

/// Column widths (mm) in `HEADERS` order; sums to the landscape drawable width.
const COL_W: [f32; 8] = [14.0, 50.0, 26.0, 68.0, 18.0, 26.0, 26.0, 45.0];

pub fn render_rekap_pdf(
    rows: &[RekapLaporanRow],
    filter: &RekapLaporanFilter,
) -> AppResult<Vec<u8>> {
    let mut pdf = PdfBuilder::new("Laporan Kebutuhan BMN", geom())?;
    pdf.write_centered("LAPORAN KEBUTUHAN BMN", 13.0, true);
    pdf.advance(5.0);
    pdf.write_centered(&subtitle(filter), 9.0, false);
    pdf.advance(7.0);

    let headers: Vec<String> = HEADERS.iter().map(|h| h.to_string()).collect();
    pdf.row(&COL_W, &headers, true, true);

    for r in rows {
        pdf.row(&COL_W, &cells(r).to_vec(), false, true);
    }

    // Totals row — the rollup is the report's reason for existing.
    let totals = vec![
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        "TOTAL".to_string(),
        rows.iter().map(|r| r.jumlah).sum::<i32>().to_string(),
        rows.iter().map(|r| r.jml_setuju).sum::<i32>().to_string(),
        format!("{} baris", rows.len()),
    ];
    pdf.row(&COL_W, &totals, true, true);

    pdf.finish()
}
