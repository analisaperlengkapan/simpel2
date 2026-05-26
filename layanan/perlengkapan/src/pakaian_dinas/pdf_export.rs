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

use std::io::{BufWriter, Cursor};

use printpdf::{
    BuiltinFont, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference,
    Point,
};
use uuid::Uuid;

use super::models::{LaporanDaftarPegawai, LaporanFilter, LaporanRekapUkuran};
use super::services::PakaianDinasService;
use crate::shared::error::{AppResult, bad_request};

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

struct PdfBuilder {
    doc: PdfDocumentReference,
    font: IndirectFontRef,
    font_bold: IndirectFontRef,
    pages: Vec<(printpdf::PdfPageIndex, printpdf::PdfLayerIndex)>,
    current_page: usize,
    /// Y cursor in mm from the bottom of page (printpdf convention).
    y: f32,
}

impl PdfBuilder {
    fn new(title: &str) -> AppResult<Self> {
        let (doc, page1, layer1) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "L1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| bad_request(&format!("font: {e}")))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| bad_request(&format!("font: {e}")))?;
        Ok(Self {
            doc,
            font,
            font_bold,
            pages: vec![(page1, layer1)],
            current_page: 0,
            y: MARGIN_TOP,
        })
    }

    fn layer(&self) -> PdfLayerReference {
        let (p, l) = self.pages[self.current_page];
        self.doc.get_page(p).get_layer(l)
    }

    /// Mulai halaman baru; reset y cursor.
    fn new_page(&mut self) {
        let (p, l) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "L1");
        self.pages.push((p, l));
        self.current_page = self.pages.len() - 1;
        self.y = MARGIN_TOP;
    }

    /// Pastikan ada ruang `needed` mm di bawah cursor; jika tidak, page baru.
    fn ensure_space(&mut self, needed: f32) {
        if self.y - needed < MARGIN_BOTTOM {
            self.new_page();
        }
    }

    fn write_text(&mut self, text: &str, font_size: f32, bold: bool, x: f32) {
        let font = if bold { &self.font_bold } else { &self.font };
        self.layer().use_text(text, font_size, Mm(x), Mm(self.y), font);
    }

    fn write_centered(&mut self, text: &str, font_size: f32, bold: bool) {
        // printpdf doesn't measure text width; approximate via char count.
        let approx_w = text.chars().count() as f32 * font_size * 0.18;
        let x = (PAGE_W - approx_w) / 2.0;
        self.write_text(text, font_size, bold, x.max(MARGIN_L));
    }

    fn advance(&mut self, delta: f32) {
        self.y -= delta;
    }

    /// Draw a horizontal line at current y.
    fn hline(&mut self, x1: f32, x2: f32) {
        let layer = self.layer();
        let line = Line {
            points: vec![
                (Point::new(Mm(x1), Mm(self.y)), false),
                (Point::new(Mm(x2), Mm(self.y)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    /// Draw a vertical line from y_top down to y_bottom at given x.
    fn vline(&mut self, x: f32, y_top: f32, y_bottom: f32) {
        let layer = self.layer();
        let line = Line {
            points: vec![
                (Point::new(Mm(x), Mm(y_top)), false),
                (Point::new(Mm(x), Mm(y_bottom)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    /// Render a table row given column widths (sum should ≤ drawable_w),
    /// cell texts, and styling. Increments y by ROW_H.
    fn row(&mut self, col_widths: &[f32], cells: &[String], bold: bool, draw_border: bool) {
        self.ensure_space(ROW_H);
        // Borders
        let y_top = self.y + ROW_H * 0.75;
        let y_bot = self.y - ROW_H * 0.25;

        if draw_border {
            // Top + bottom lines
            let total_w: f32 = col_widths.iter().sum();
            let prev_y = self.y;
            self.y = y_top;
            self.hline(MARGIN_L, MARGIN_L + total_w);
            self.y = y_bot;
            self.hline(MARGIN_L, MARGIN_L + total_w);
            self.y = prev_y;
            // Vertical lines
            let mut x = MARGIN_L;
            self.vline(x, y_top, y_bot);
            for w in col_widths {
                x += *w;
                self.vline(x, y_top, y_bot);
            }
        }

        // Text in each cell (left-aligned w/ small inset).
        let mut x = MARGIN_L + 1.0;
        for (cell, w) in cells.iter().zip(col_widths) {
            // Truncate text that won't fit using crude char-width estimate.
            let max_chars = ((*w - 2.0) / (CELL_FONT * 0.16)).max(1.0) as usize;
            let display: String = if cell.chars().count() > max_chars {
                let mut s: String = cell.chars().take(max_chars.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                cell.clone()
            };
            self.write_text(&display, CELL_FONT, bold, x);
            x += *w;
        }
        self.advance(ROW_H);
    }

    fn finish(self) -> AppResult<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        {
            let writer = BufWriter::new(Cursor::new(&mut buf));
            self.doc
                .save(&mut BufWriter::new(writer.into_inner().map_err(|e| {
                    bad_request(&format!("pdf writer flush: {e}"))
                })?))
                .map_err(|e| bad_request(&format!("pdf save: {e}")))?;
        }
        Ok(buf)
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

    let mut pdf = PdfBuilder::new("Rekap Ukuran Pakaian Dinas")?;
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

    let mut pdf = PdfBuilder::new("Daftar Pegawai Pakaian Dinas")?;
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
        "No",
        "NIP",
        "Nama",
        "Jabatan",
        "Golongan",
        "Status",
        "Gender",
        "Hijab",
        "Baju",
        "Celana",
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
        let mut pdf = PdfBuilder::new("Test").expect("init builder");
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
