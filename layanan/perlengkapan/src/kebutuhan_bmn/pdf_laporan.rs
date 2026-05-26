//! # Kebutuhan BMN — Laporan Hasil Analisis PDF (Fase 0.8)
//!
//! Render Laporan Hasil Analisis Kebutuhan BMN per-satker ke PDF.
//! Sumber data: [`super::services::KebutuhanBmnService::get_analisis_kelayakan`]
//! yang sudah memuat:
//! - Header satker (nama, tahun pengajuan)
//! - Daftar barang dgn jumlah diusulkan, existing SIMAN, gap, rekomendasi
//! - Rekap pegawai (eselon/non-eselon) jika tersedia
//! - Sync status SIMAN/MySIMKARI
//! - Summary kelayakan (%)
//!
//! Output: A4 portrait, table-driven. Dua endpoint memakai bytes ini:
//! - `/satker/{id}/laporan/preview` (Content-Disposition inline)
//! - `/satker/{id}/laporan/download` (Content-Disposition attachment)
//!
//! Catatan implementasi: ada duplikasi `PdfBuilder` dgn `pakaian_dinas/
//! pdf_export.rs`. Extract ke `shared/pdf.rs` dijadwalkan sbg follow-up
//! refactor (low priority — kedua modul stabil & tidak sering diubah).

use std::io::{BufWriter, Cursor};

use printpdf::{
    BuiltinFont, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference,
    Point,
};
use uuid::Uuid;

use super::models::{AnalisisKelayakanResponse, BarangWithExistingInventory};
use super::services::KebutuhanBmnService;
use crate::shared::error::{AppResult, bad_request};

// ─── Page geometry (A4 portrait) ─────────────────────────────────────────
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN_L: f32 = 15.0;
const MARGIN_R: f32 = 15.0;
const MARGIN_TOP: f32 = 285.0;
const MARGIN_BOTTOM: f32 = 18.0;
const ROW_H: f32 = 5.5;
const TITLE_FONT: f32 = 13.0;
const SECTION_FONT: f32 = 10.0;
const CELL_FONT: f32 = 7.5;
const BODY_FONT: f32 = 9.0;

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

    fn new_page(&mut self) {
        let (p, l) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "L1");
        self.pages.push((p, l));
        self.current_page = self.pages.len() - 1;
        self.y = MARGIN_TOP;
    }

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
        let approx_w = text.chars().count() as f32 * font_size * 0.18;
        let x = (PAGE_W - approx_w) / 2.0;
        self.write_text(text, font_size, bold, x.max(MARGIN_L));
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

    fn row(&mut self, col_widths: &[f32], cells: &[String], bold: bool) {
        self.ensure_space(ROW_H);
        let y_top = self.y + ROW_H * 0.75;
        let y_bot = self.y - ROW_H * 0.25;
        let total_w: f32 = col_widths.iter().sum();
        let prev_y = self.y;
        self.y = y_top;
        self.hline(MARGIN_L, MARGIN_L + total_w);
        self.y = y_bot;
        self.hline(MARGIN_L, MARGIN_L + total_w);
        self.y = prev_y;
        let mut x = MARGIN_L;
        self.vline(x, y_top, y_bot);
        for w in col_widths {
            x += *w;
            self.vline(x, y_top, y_bot);
        }
        let mut x = MARGIN_L + 1.0;
        for (cell, w) in cells.iter().zip(col_widths) {
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

/// Render Laporan Hasil Analisis Kebutuhan BMN per-satker ke PDF bytes.
///
/// Bytes ini dapat di-stream sbg `application/pdf` baik inline (preview
/// di iframe) maupun attachment (download).
pub async fn generate_laporan_analisis_pdf(
    service: &KebutuhanBmnService,
    satker_id: Uuid,
) -> AppResult<Vec<u8>> {
    let analisis = service.get_analisis_kelayakan(satker_id).await?;
    render_pdf(&analisis)
}

fn render_pdf(analisis: &AnalisisKelayakanResponse) -> AppResult<Vec<u8>> {
    let mut pdf = PdfBuilder::new("Laporan Hasil Analisis Kebutuhan BMN")?;

    // ── Title block ──────────────────────────────────────────────────
    pdf.write_centered(
        "LAPORAN HASIL ANALISIS KEBUTUHAN BMN",
        TITLE_FONT,
        true,
    );
    pdf.advance(7.0);
    let satker_label = analisis
        .satker
        .nm_satker
        .clone()
        .unwrap_or_else(|| format!("Satker {}", analisis.satker.ms_satker_id));
    pdf.write_centered(&satker_label, SECTION_FONT, false);
    pdf.advance(10.0);

    // ── Identitas Satker ────────────────────────────────────────────
    pdf.write_text("IDENTITAS SATKER", SECTION_FONT, true, MARGIN_L);
    pdf.advance(7.0);
    let satker_info: Vec<(&str, String)> = vec![
        ("Kode Satker", analisis.satker.ms_satker_id.clone()),
        ("Nama Satker", satker_label.clone()),
        ("Status", analisis.satker.status.label().to_string()),
        ("Prioritas", analisis.satker.prioritas.to_string()),
        (
            "Catatan Satker",
            analisis
                .satker
                .catatan_satker
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    for (label, value) in &satker_info {
        pdf.write_text(&format!("{:20}: {}", label, value), BODY_FONT, false, MARGIN_L);
        pdf.advance(5.5);
    }
    pdf.advance(4.0);

    // ── Tabel Barang Usulan vs Eksisting SIMAN ──────────────────────
    pdf.ensure_space(40.0);
    pdf.write_text(
        "DAFTAR BARANG: USULAN vs EKSISTING SIMAN",
        SECTION_FONT,
        true,
        MARGIN_L,
    );
    pdf.advance(7.0);

    // Kolom: No | Kode | Nama Barang | Jml Usulan | Existing | Gap | Rekomendasi
    let col_widths: Vec<f32> = vec![
        8.0,  // No
        22.0, // Kode
        55.0, // Nama
        15.0, // Jml usulan
        15.0, // Existing
        12.0, // Gap
        53.0, // Rekomendasi
    ];
    let header_cells: Vec<String> = ["No", "Kode", "Nama Barang", "Usulan", "Eksisting", "Gap", "Rekomendasi"]
        .into_iter()
        .map(String::from)
        .collect();
    pdf.row(&col_widths, &header_cells, true);

    if analisis.barang_list.is_empty() {
        pdf.write_text("Tidak ada barang yang diusulkan.", BODY_FONT, false, MARGIN_L);
        pdf.advance(6.0);
    } else {
        for (idx, b) in analisis.barang_list.iter().enumerate() {
            let cells = barang_row_cells(idx + 1, b);
            pdf.row(&col_widths, &cells, false);
        }
    }
    pdf.advance(6.0);

    // ── Summary Kelayakan ────────────────────────────────────────────
    pdf.ensure_space(28.0);
    pdf.write_text("RINGKASAN KELAYAKAN", SECTION_FONT, true, MARGIN_L);
    pdf.advance(7.0);
    let summary = &analisis.summary;
    let summary_lines = vec![
        format!("Total diminta     : {}", summary.total_diminta),
        format!("Total eksisting   : {}", summary.total_existing),
        format!("Gap (kekurangan)  : {}", summary.total_gap),
        format!(
            "Persentase kelayakan: {:.1}%",
            summary.kelayakan_persen
        ),
    ];
    for line in &summary_lines {
        pdf.write_text(line, BODY_FONT, false, MARGIN_L);
        pdf.advance(5.5);
    }
    pdf.advance(4.0);

    // ── Status Integrasi (jika ada) ──────────────────────────────────
    if let Some(integrasi) = &analisis.integrasi_sync {
        pdf.ensure_space(28.0);
        pdf.write_text("STATUS INTEGRASI DATA", SECTION_FONT, true, MARGIN_L);
        pdf.advance(7.0);

        let mut lines: Vec<String> = Vec::new();
        let push = |lines: &mut Vec<String>, label: &str, status: &super::models::IntegrasiSyncStatus| {
            lines.push(format!(
                "{label:8}: {} (records synced: {}, last_sync: {})",
                status.state,
                status.records_synced,
                status.last_sync_at.clone().unwrap_or_else(|| "-".into()),
            ));
        };
        push(&mut lines, "SIMAN", &integrasi.siman);
        push(&mut lines, "MySIMKARI", &integrasi.mysimkari);
        for line in &lines {
            pdf.write_text(line, BODY_FONT, false, MARGIN_L);
            pdf.advance(5.5);
        }
        pdf.advance(4.0);
    }

    // ── Rekap Pegawai (jika MySIMKARI tersedia) ──────────────────────
    if let Some(rekap) = &analisis.data_pegawai {
        pdf.ensure_space(24.0);
        pdf.write_text("REKAP PEGAWAI (MYSIMKARI)", SECTION_FONT, true, MARGIN_L);
        pdf.advance(7.0);
        let total_eselon: i64 = rekap.rekap_eselon.iter().map(|r| r.jumlah).sum();
        let total_non_eselon: i64 = rekap.rekap_non_eselon.iter().map(|r| r.jumlah).sum();
        pdf.write_text(
            &format!("Total pegawai        : {}", rekap.total_pegawai),
            BODY_FONT,
            false,
            MARGIN_L,
        );
        pdf.advance(5.5);
        pdf.write_text(
            &format!("  - Eselon           : {}", total_eselon),
            BODY_FONT,
            false,
            MARGIN_L,
        );
        pdf.advance(5.5);
        pdf.write_text(
            &format!("  - Non-eselon       : {}", total_non_eselon),
            BODY_FONT,
            false,
            MARGIN_L,
        );
        pdf.advance(5.5);
        pdf.advance(4.0);
    }

    // ── Footer ───────────────────────────────────────────────────────
    pdf.ensure_space(12.0);
    pdf.write_text(
        &format!(
            "Dokumen ini di-generate otomatis pada {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ),
        7.0,
        false,
        MARGIN_L,
    );

    pdf.finish()
}

fn barang_row_cells(no: usize, b: &BarangWithExistingInventory) -> Vec<String> {
    // existing = jumlah usulan - gap (jika gap > 0); jika gap negatif berarti
    // existing > usulan → tampilkan jumlah existing apa adanya.
    let existing = (b.barang.jumlah - b.gap).max(0);
    vec![
        no.to_string(),
        b.barang.kode_barang.clone().unwrap_or_else(|| "-".to_string()),
        b.barang.nama.clone(),
        b.barang.jumlah.to_string(),
        existing.to_string(),
        b.gap.to_string(),
        b.recommendation.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_builder_produces_pdf_magic() {
        let mut pdf = PdfBuilder::new("Test").expect("init");
        pdf.write_centered("HALO", 12.0, true);
        pdf.advance(6.0);
        pdf.row(
            &[20.0, 30.0, 30.0],
            &["A".into(), "B".into(), "C".into()],
            false,
        );
        let bytes = pdf.finish().expect("finish");
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..4], b"%PDF");
    }
}
