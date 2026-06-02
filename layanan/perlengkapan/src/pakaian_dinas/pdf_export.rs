//! # Pakaian Dinas PDF Export Module (Fase 0.9)
//!
//! PDF generation untuk dua mode laporan:
//! - **Rekap** (`generate_rekap_pdf`): aggregat per-pakaian × per-gender ×
//!   per-ukuran (mirror struktur simpelv1 `cetakRekapTemplateV.blade.php`
//!   yg disederhanakan ke level summary, sama dgn Excel rekap).
//! - **Daftar** (`generate_daftar_pdf`): per-satker (page-break) dgn baris
//!   per pegawai + kolom ukuran (mirror `cetakDaftarTemplateV.blade.php`).
//!
//! Implementasi pakai `printpdf` 0.7 primitives — table cells digambar
//! manual via `Line` shapes + `use_text`. Tidak ada dependency pada
//! TemplateService berbasis DB (template untuk laporan tabular cocok
//! di-render lewat kode terstruktur, bukan Handlebars).

use uuid::Uuid;

use super::models::{LaporanDaftarPegawai, LaporanFilter, LaporanRekapUkuran};
use super::services::PakaianDinasService;
use crate::shared::error::AppResult;
use crate::shared::pdf::{PageGeometry, PdfBuilder};

// ─── Page geometry (A4 landscape) ────────────────────────────────────────
const PAGE_W: f32 = 297.0;
const PAGE_H: f32 = 210.0;
const MARGIN_L: f32 = 12.0;
const MARGIN_R: f32 = 12.0;
const MARGIN_TOP: f32 = 200.0; // start text near top
const MARGIN_BOTTOM: f32 = 15.0;
const ROW_H: f32 = 6.0;
const HEADER_FONT: f32 = 9.0;
const TITLE_FONT: f32 = 14.0;
const CELL_FONT: f32 = 7.5;

fn drawable_w() -> f32 {
    PAGE_W - MARGIN_L - MARGIN_R
}

fn geom() -> PageGeometry {
    PageGeometry {
        page_w: PAGE_W,
        page_h: PAGE_H,
        margin_l: MARGIN_L,
        margin_r: MARGIN_R,
        margin_top: MARGIN_TOP,
        margin_bottom: MARGIN_BOTTOM,
        row_h: ROW_H,
        cell_font: CELL_FONT,
        cell_pad_x: 1.0,
        cell_trunc_pad: 2.0,
        auto_paginate: true,
    }
}

// ─── Rekap PDF ──────────────────────────────────────────────────────────

pub async fn generate_rekap_pdf(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    let data = service
        .get_laporan_rekap_ukuran(pengajuan_id, filter.clone())
        .await?;

    let mut pdf = PdfBuilder::new("Rekap Ukuran Pakaian Dinas", geom())?;
    pdf.write_centered("REKAP UKURAN PAKAIAN DINAS", TITLE_FONT, true);
    pdf.advance(8.0);
    pdf.write_centered(
        &format!("Pengajuan ID: {}", pengajuan_id),
        HEADER_FONT,
        false,
    );
    pdf.advance(10.0);

    if data.is_empty() {
        pdf.write_text("Tidak ada data.", HEADER_FONT, false, MARGIN_L);
        return pdf.finish();
    }

    // Group by pakaian_nama
    let mut groups: Vec<String> = Vec::new();
    for r in &data {
        if !groups.contains(&r.pakaian_nama) {
            groups.push(r.pakaian_nama.clone());
        }
    }

    for group in &groups {
        let items: Vec<&LaporanRekapUkuran> =
            data.iter().filter(|r| &r.pakaian_nama == group).collect();
        if items.is_empty() {
            continue;
        }

        // Group title
        pdf.ensure_space(ROW_H * 5.0);
        pdf.write_text(
            &format!("{} ({})", group, items[0].ukuran_group),
            HEADER_FONT,
            true,
            MARGIN_L,
        );
        pdf.advance(7.0);

        // Table: Gender | <ukuran...> | Total
        let n_uk = items.len();
        let gender_col_w = 22.0;
        let total_col_w = 18.0;
        let remaining = drawable_w() - gender_col_w - total_col_w;
        let uk_col_w = (remaining / n_uk as f32).max(10.0);
        let mut col_widths = vec![gender_col_w];
        col_widths.extend(std::iter::repeat_n(uk_col_w, n_uk));
        col_widths.push(total_col_w);

        // Header row
        let mut header = vec!["Gender".to_string()];
        header.extend(items.iter().map(|i| i.ukuran.clone()));
        header.push("Total".to_string());
        pdf.row(&col_widths, &header, true, true);

        // L row
        let mut total_l: i64 = 0;
        let mut row_l = vec!["L".to_string()];
        for it in &items {
            row_l.push(it.jumlah_laki.to_string());
            total_l += it.jumlah_laki;
        }
        row_l.push(total_l.to_string());
        pdf.row(&col_widths, &row_l, false, true);

        // P row
        let mut total_p: i64 = 0;
        let mut row_p = vec!["P".to_string()];
        for it in &items {
            row_p.push(it.jumlah_perempuan.to_string());
            total_p += it.jumlah_perempuan;
        }
        row_p.push(total_p.to_string());
        pdf.row(&col_widths, &row_p, false, true);

        // Total row
        let mut grand: i64 = 0;
        let mut row_t = vec!["Total".to_string()];
        for it in &items {
            row_t.push(it.jumlah_total.to_string());
            grand += it.jumlah_total;
        }
        row_t.push(grand.to_string());
        pdf.row(&col_widths, &row_t, true, true);
        pdf.advance(4.0);
    }

    pdf.finish()
}

// ─── Daftar PDF ─────────────────────────────────────────────────────────

pub async fn generate_daftar_pdf(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    // Fetch all rows (paginate until exhausted) — same pattern as Excel
    // generator agar konsisten.
    let mut all: Vec<LaporanDaftarPegawai> = Vec::new();
    let mut page = 1;
    let per_page = 500;
    loop {
        let (items, _total) = service
            .get_laporan_daftar_pegawai(pengajuan_id, filter.clone(), page, per_page)
            .await?;
        let n = items.len();
        all.extend(items);
        if n < per_page as usize {
            break;
        }
        page += 1;
    }

    let mut pdf = PdfBuilder::new("Daftar Pegawai Pakaian Dinas", geom())?;
    pdf.write_centered("DAFTAR PEGAWAI PAKAIAN DINAS", TITLE_FONT, true);
    pdf.advance(8.0);
    pdf.write_centered(
        &format!("Pengajuan ID: {}", pengajuan_id),
        HEADER_FONT,
        false,
    );
    pdf.advance(10.0);

    if all.is_empty() {
        pdf.write_text("Tidak ada data.", HEADER_FONT, false, MARGIN_L);
        return pdf.finish();
    }

    // Group by satker_nama (preserve first-seen order).
    let mut satker_order: Vec<String> = Vec::new();
    for it in &all {
        let key = if it.satker_nama.is_empty() {
            "-".to_string()
        } else {
            it.satker_nama.clone()
        };
        if !satker_order.contains(&key) {
            satker_order.push(key);
        }
    }

    // Column widths (sum should ≈ drawable_w() = 273 mm)
    let col_widths: Vec<f32> = vec![
        8.0,  // No
        28.0, // NIP
        45.0, // Nama
        35.0, // Jabatan
        15.0, // Golongan
        12.0, // Status (J/T)
        12.0, // Gender
        18.0, // Hijab
        30.0, // Baju
        30.0, // Celana
        30.0, // Sepatu
    ];
    let header_cells: Vec<String> = vec![
        "No", "NIP", "Nama", "Jabatan", "Golongan", "Status", "Gender", "Hijab", "Baju", "Celana",
        "Sepatu",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut first = true;
    for satker in &satker_order {
        if !first {
            pdf.new_page();
        }
        first = false;

        pdf.write_text(satker, HEADER_FONT, true, MARGIN_L);
        pdf.advance(7.0);

        pdf.row(&col_widths, &header_cells, true, true);

        let mut idx = 0usize;
        for it in all.iter().filter(|p| {
            let pname = if p.satker_nama.is_empty() {
                "-"
            } else {
                p.satker_nama.as_str()
            };
            pname == satker.as_str()
        }) {
            idx += 1;
            let status = match it.jenis.as_deref() {
                Some("0") => "J",
                Some("1") => "T",
                _ => "-",
            };
            let hijab = if it.with_hijab { "Y" } else { "T" };
            let cells = vec![
                idx.to_string(),
                it.nip.clone(),
                it.nama.clone(),
                it.jabatan.clone().unwrap_or_else(|| "-".to_string()),
                it.gol_kd.clone().unwrap_or_else(|| "-".to_string()),
                status.to_string(),
                it.jenis_kelamin.clone(),
                hijab.to_string(),
                it.ukuran_baju.clone().unwrap_or_else(|| "-".to_string()),
                it.ukuran_celana.clone().unwrap_or_else(|| "-".to_string()),
                it.ukuran_sepatu.clone().unwrap_or_else(|| "-".to_string()),
            ];
            pdf.row(&col_widths, &cells, false, true);
        }
    }

    pdf.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: builder produces bytes starting with `%PDF-` magic.
    /// Mengisolasi printpdf wiring (font + page) tanpa butuh DB.
    #[test]
    fn pdf_builder_produces_pdf_magic_bytes() {
        let mut pdf = PdfBuilder::new("Test", geom()).expect("init builder");
        pdf.write_centered("HELLO", 12.0, true);
        pdf.advance(8.0);
        pdf.row(
            &[20.0, 30.0, 30.0],
            &["A".into(), "B".into(), "C".into()],
            false,
            true,
        );
        let bytes = pdf.finish().expect("finish");
        assert!(bytes.len() > 4);
        assert_eq!(
            &bytes[0..4],
            b"%PDF",
            "expected PDF magic at start, got {:?}",
            &bytes[0..bytes.len().min(8)]
        );
    }
}
