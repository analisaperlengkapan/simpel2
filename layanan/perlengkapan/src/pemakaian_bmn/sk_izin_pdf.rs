//! # Pemakaian BMN — SK Izin Pemakaian BMN 2-halaman PDF (Fase 1.10)
//!
//! Stakeholder eksplisit (plan §5.1):
//! - Halaman 1: informasi pegawai (NIP, nama, **pangkat**, jabatan,
//!   satker, **foto pegawai WAJIB ditampilkan**)
//! - Halaman 2: daftar BMN (kode_barang, nama, NUP, merk, tipe,
//!   tanggal mulai, tanggal berakhir)
//!
//! Sumber data: `IzinPemakaianBmn` entity. Saat ini model single-BMN
//! per permit; daftar BMN di halaman 2 dirender sebagai 1-row table —
//! struktur table generik agar saat multi-BMN ditambahkan (Fase 1.5+
//! refactor lebih luas), tinggal extend baris.
//!
//! Catatan duplikasi: PdfBuilder helper sama dgn pakaian_dinas/pdf_export.rs
//! (0.9) dan kebutuhan_bmn/pdf_laporan.rs (0.8). Refactor ekstraksi ke
//! `shared/pdf.rs` ada di task #17 follow-up.

use std::io::{BufWriter, Cursor};

use printpdf::{
    BuiltinFont, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference,
    Point,
};
use uuid::Uuid;

use super::models::IzinPemakaianBmn;
use super::services::PemakaianBmnService;
use crate::shared::error::{AppResult, bad_request};

// ─── Page geometry (A4 portrait) ─────────────────────────────────────────
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN_L: f32 = 18.0;
const MARGIN_R: f32 = 18.0;
const MARGIN_TOP: f32 = 280.0;
const ROW_H: f32 = 6.0;
const CELL_FONT: f32 = 8.0;
const BODY_FONT: f32 = 10.0;
const SECTION_FONT: f32 = 11.0;
const TITLE_FONT: f32 = 14.0;

fn drawable_w() -> f32 {
    PAGE_W - MARGIN_L - MARGIN_R
}

struct PdfBuilder {
    doc: PdfDocumentReference,
    font: IndirectFontRef,
    font_bold: IndirectFontRef,
    pages: Vec<(printpdf::PdfPageIndex, printpdf::PdfLayerIndex)>,
    current_page: usize,
    y: f32,
}

impl PdfBuilder {
    fn new(title: &str) -> AppResult<Self> {
        let (doc, p, l) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "L1");
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
            pages: vec![(p, l)],
            current_page: 0,
            y: MARGIN_TOP,
        })
    }

    fn layer(&self) -> PdfLayerReference {
        let (p, l) = self.pages[self.current_page];
        self.doc.get_page(p).get_layer(l)
    }

    fn new_page(&mut self) {
        let (p, l) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "L1");
        self.pages.push((p, l));
        self.current_page = self.pages.len() - 1;
        self.y = MARGIN_TOP;
    }

    fn write_text(&mut self, text: &str, font_size: f32, bold: bool, x: f32) {
        let font = if bold { &self.font_bold } else { &self.font };
        self.layer().use_text(text, font_size, Mm(x), Mm(self.y), font);
    }

    fn write_centered(&mut self, text: &str, font_size: f32, bold: bool) {
        let approx_w = text.chars().count() as f32 * font_size * 0.18;
        let x = ((PAGE_W - approx_w) / 2.0).max(MARGIN_L);
        self.write_text(text, font_size, bold, x);
    }

    fn advance(&mut self, delta: f32) {
        self.y -= delta;
    }

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

    fn rect(&mut self, x: f32, w: f32, h: f32) {
        let y_top = self.y + h * 0.5;
        let y_bot = self.y - h * 0.5;
        let prev_y = self.y;
        self.y = y_top;
        self.hline(x, x + w);
        self.y = y_bot;
        self.hline(x, x + w);
        self.vline(x, y_top, y_bot);
        self.vline(x + w, y_top, y_bot);
        self.y = prev_y;
    }

    fn row(&mut self, col_widths: &[f32], cells: &[String], bold: bool) {
        let y_top = self.y + ROW_H * 0.75;
        let y_bot = self.y - ROW_H * 0.25;
        let total_w: f32 = col_widths.iter().sum();
        let prev_y = self.y;
        self.y = y_top;
        self.hline(MARGIN_L, MARGIN_L + total_w);
        self.y = y_bot;
        self.hline(MARGIN_L, MARGIN_L + total_w);
        self.y = prev_y;
        let mut x_cursor = MARGIN_L;
        self.vline(x_cursor, y_top, y_bot);
        for w in col_widths {
            x_cursor += *w;
            self.vline(x_cursor, y_top, y_bot);
        }
        let mut x_cursor = MARGIN_L + 1.5;
        for (cell, w) in cells.iter().zip(col_widths) {
            let max_chars = ((*w - 3.0) / (CELL_FONT * 0.16)).max(1.0) as usize;
            let display: String = if cell.chars().count() > max_chars {
                let mut s: String = cell.chars().take(max_chars.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                cell.clone()
            };
            self.write_text(&display, CELL_FONT, bold, x_cursor);
            x_cursor += *w;
        }
        self.advance(ROW_H);
    }

    fn finish(self) -> AppResult<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = BufWriter::new(cursor);
            self.doc
                .save(&mut writer)
                .map_err(|e| bad_request(&format!("pdf save: {e}")))?;
        }
        Ok(buf)
    }
}

// ─── Public API ─────────────────────────────────────────────────────────

pub async fn generate_sk_izin_pdf(
    service: &PemakaianBmnService,
    permit_id: Uuid,
) -> AppResult<Vec<u8>> {
    let detail = service.get_permit_detail(permit_id).await?;
    render_pdf(&detail.izin)
}

fn render_pdf(permit: &IzinPemakaianBmn) -> AppResult<Vec<u8>> {
    let mut pdf = PdfBuilder::new("SK Izin Pemakaian BMN")?;

    // ─── Halaman 1: Informasi Pegawai ────────────────────────────────
    render_halaman_pegawai(&mut pdf, permit);

    // ─── Halaman 2: Daftar BMN ───────────────────────────────────────
    pdf.new_page();
    render_halaman_daftar_bmn(&mut pdf, permit);

    pdf.finish()
}

fn render_halaman_pegawai(pdf: &mut PdfBuilder, permit: &IzinPemakaianBmn) {
    // Title bar
    pdf.write_centered("SURAT KEPUTUSAN IZIN PEMAKAIAN BMN", TITLE_FONT, true);
    pdf.advance(7.0);
    pdf.write_centered(
        &format!(
            "Nomor: {}",
            permit.nomor_izin.as_deref().unwrap_or("(belum diterbitkan)")
        ),
        SECTION_FONT,
        false,
    );
    pdf.advance(12.0);

    pdf.write_text("HALAMAN 1 — INFORMASI PEGAWAI", SECTION_FONT, true, MARGIN_L);
    pdf.advance(10.0);

    // Foto pegawai placeholder (WAJIB per stakeholder).
    // printpdf 0.7 embedding image dari URL membutuhkan async fetch +
    // decode. Untuk MVP gambar foto sebagai kotak placeholder dgn label
    // — saat foto_pegawai (URL) di-resolve via DocumentStorage, ganti
    // dgn ImageXObject embed.
    let foto_x = MARGIN_L;
    let foto_w = 35.0;
    let foto_h = 45.0;
    let prev_y = pdf.y;
    pdf.y = prev_y - foto_h * 0.5; // center vertically on rect
    pdf.rect(foto_x, foto_w, foto_h);
    pdf.y = prev_y;
    pdf.write_text("FOTO", CELL_FONT, true, foto_x + foto_w / 2.0 - 4.0);
    let foto_url_label = permit
        .foto_pegawai
        .as_deref()
        .unwrap_or("(foto belum di-upload)");
    pdf.advance(foto_h + 3.0);
    pdf.write_text(
        &format!("Sumber foto: {}", foto_url_label),
        CELL_FONT,
        false,
        foto_x,
    );
    pdf.advance(6.0);

    // Data pegawai sebagai key-value list di sebelah foto akan butuh
    // layout 2-kolom. Untuk simplicity & legibility, render full-width
    // setelah area foto.
    pdf.advance(4.0);
    let info: Vec<(&str, String)> = vec![
        ("NIP", permit.pegawai_nip.clone()),
        ("Nama", permit.pegawai_nama.clone()),
        (
            "Pangkat",
            permit
                .pegawai_pangkat
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Jabatan",
            permit
                .pegawai_jabatan
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Satker", permit.pegawai_satker_nama.clone()),
        (
            "Unit Kerja",
            permit
                .pegawai_unit_kerja
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    for (label, value) in &info {
        pdf.write_text(
            &format!("{:14}: {}", label, value),
            BODY_FONT,
            false,
            MARGIN_L,
        );
        pdf.advance(6.5);
    }

    // Catatan: foto pegawai WAJIB sesuai stakeholder. Footer dgn timestamp
    // & SK number.
    pdf.advance(8.0);
    pdf.write_text(
        &format!(
            "Periode izin: {} s.d. {}",
            permit.tanggal_mulai.format("%d %B %Y"),
            permit.tanggal_selesai.format("%d %B %Y")
        ),
        BODY_FONT,
        false,
        MARGIN_L,
    );
    pdf.advance(6.5);
    pdf.write_text(
        &format!("Keperluan: {}", permit.keperluan),
        BODY_FONT,
        false,
        MARGIN_L,
    );
}

fn render_halaman_daftar_bmn(pdf: &mut PdfBuilder, permit: &IzinPemakaianBmn) {
    pdf.write_centered("HALAMAN 2 — DAFTAR BMN", SECTION_FONT, true);
    pdf.advance(10.0);

    // Header table: No | Kode | Nama | NUP | Merk | Tipe | Mulai | Selesai
    let col_widths: Vec<f32> = vec![
        8.0,  // No
        22.0, // Kode
        38.0, // Nama
        15.0, // NUP
        22.0, // Merk
        18.0, // Tipe
        24.0, // Mulai
        24.0, // Selesai
    ];
    // Verify sum ≈ drawable_w
    let _total: f32 = col_widths.iter().sum();
    let header_cells: Vec<String> = [
        "No", "Kode", "Nama", "NUP", "Merk", "Tipe", "Mulai", "Selesai",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    pdf.row(&col_widths, &header_cells, true);

    // V1 model: single BMN per permit. Render sebagai 1 row table.
    // Saat multi-BMN refactor (post-Fase 1.5), loop melalui Vec<BmnItem>.
    let cells: Vec<String> = vec![
        "1".to_string(),
        permit.bmn_kode_barang.clone(),
        permit.bmn_nama_barang.clone(),
        permit.bmn_nup.clone(),
        permit.bmn_merk.clone().unwrap_or_else(|| "-".to_string()),
        // Field `bmn_tipe` belum ada di model main; fallback "-" sampai
        // refactor model BMN multi-field landed.
        "-".to_string(),
        permit.tanggal_mulai.format("%d-%m-%Y").to_string(),
        permit.tanggal_selesai.format("%d-%m-%Y").to_string(),
    ];
    pdf.row(&col_widths, &cells, false);

    // Catatan footer
    pdf.advance(8.0);
    pdf.write_text(
        "Demikian Surat Keputusan ini dibuat untuk dipergunakan sebagaimana mestinya.",
        BODY_FONT,
        false,
        MARGIN_L,
    );
    pdf.advance(12.0);
    pdf.write_text(
        &format!(
            "Dokumen di-generate otomatis pada {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ),
        7.5,
        false,
        MARGIN_L,
    );

    let _ = drawable_w(); // suppress unused warning if drawable_w pure-helper not used
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_builder_produces_pdf_magic() {
        let mut pdf = PdfBuilder::new("Test").expect("init");
        pdf.write_centered("TEST", 12.0, true);
        pdf.advance(8.0);
        pdf.row(
            &[20.0, 30.0, 30.0],
            &["A".into(), "B".into(), "C".into()],
            false,
        );
        pdf.new_page();
        pdf.write_centered("Page 2", 12.0, false);
        let bytes = pdf.finish().expect("finish");
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..4], b"%PDF");
    }
}
