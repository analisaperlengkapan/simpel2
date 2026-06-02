//! # Shared low-level PDF builder (#17)
//!
//! `printpdf` is a primitive API (no text measurement, no layout). The three
//! report generators — `kebutuhan_bmn::pdf_laporan` (Fase 0.8),
//! `pakaian_dinas::pdf_export` (0.9), `pemakaian_bmn::sk_izin_pdf` (1.10) —
//! each grew an almost-identical `PdfBuilder` helper (page/font setup, text
//! cursor, table rows, line drawing). This module hoists that helper into one
//! place, parameterized by [`PageGeometry`] so each report keeps its own page
//! size / margins / fonts.
//!
//! Behaviour is preserved exactly: the only per-report variations
//! (portrait/landscape, cell padding, whether tables auto-paginate) are
//! captured as geometry fields, so swapping the local copies for this builder
//! produces the same output.

use std::io::{BufWriter, Cursor};

use printpdf::{
    BuiltinFont, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerIndex,
    PdfLayerReference, PdfPageIndex, Point,
};

use crate::shared::error::{AppResult, bad_request};

/// Per-report page geometry. All dimensions in millimetres.
#[derive(Debug, Clone, Copy)]
pub struct PageGeometry {
    pub page_w: f32,
    pub page_h: f32,
    pub margin_l: f32,
    pub margin_r: f32,
    /// Initial / post-page-break y cursor (mm from page bottom).
    pub margin_top: f32,
    /// Lower bound; tables auto-break to a new page below this when
    /// `auto_paginate` is set.
    pub margin_bottom: f32,
    /// Table row height.
    pub row_h: f32,
    /// Font size for table cell text.
    pub cell_font: f32,
    /// Horizontal inset of cell text from the cell's left border.
    pub cell_pad_x: f32,
    /// Width subtracted from a column when estimating truncation width.
    pub cell_trunc_pad: f32,
    /// Whether [`PdfBuilder::row`]/[`PdfBuilder::ensure_space`] auto-paginate.
    /// `false` for fixed-layout documents (e.g. the 2-page SK izin).
    pub auto_paginate: bool,
}

impl PageGeometry {
    /// Usable horizontal width between the left and right margins.
    pub fn drawable_w(&self) -> f32 {
        self.page_w - self.margin_l - self.margin_r
    }
}

/// Stateful builder over a `printpdf` document: an internal page list, a
/// vertical text cursor (`y`), and table/line primitives.
pub struct PdfBuilder {
    doc: PdfDocumentReference,
    font: IndirectFontRef,
    font_bold: IndirectFontRef,
    pages: Vec<(PdfPageIndex, PdfLayerIndex)>,
    current_page: usize,
    geom: PageGeometry,
    /// Y cursor in mm from the bottom of the page (printpdf convention).
    /// Public so callers can place free-form content (e.g. an image) relative
    /// to the current row.
    pub y: f32,
}

impl PdfBuilder {
    pub fn new(title: &str, geom: PageGeometry) -> AppResult<Self> {
        let (doc, page1, layer1) =
            PdfDocument::new(title, Mm(geom.page_w), Mm(geom.page_h), "L1");
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
            y: geom.margin_top,
            geom,
        })
    }

    pub fn geom(&self) -> &PageGeometry {
        &self.geom
    }

    /// Reference to the document — for callers that embed images or other
    /// objects via the raw `printpdf` API.
    pub fn doc(&self) -> &PdfDocumentReference {
        &self.doc
    }

    pub fn layer(&self) -> PdfLayerReference {
        let (p, l) = self.pages[self.current_page];
        self.doc.get_page(p).get_layer(l)
    }

    pub fn new_page(&mut self) {
        let (p, l) = self
            .doc
            .add_page(Mm(self.geom.page_w), Mm(self.geom.page_h), "L1");
        self.pages.push((p, l));
        self.current_page = self.pages.len() - 1;
        self.y = self.geom.margin_top;
    }

    /// Ensure `needed` mm remain below the cursor; otherwise start a new page.
    /// No-op when `geom.auto_paginate` is false.
    pub fn ensure_space(&mut self, needed: f32) {
        if self.geom.auto_paginate && self.y - needed < self.geom.margin_bottom {
            self.new_page();
        }
    }

    pub fn write_text(&mut self, text: &str, font_size: f32, bold: bool, x: f32) {
        let font = if bold { &self.font_bold } else { &self.font };
        self.layer().use_text(text, font_size, Mm(x), Mm(self.y), font);
    }

    /// Approximate-centered text (printpdf can't measure glyph widths, so this
    /// estimates width from the character count).
    pub fn write_centered(&mut self, text: &str, font_size: f32, bold: bool) {
        let approx_w = text.chars().count() as f32 * font_size * 0.18;
        let x = ((self.geom.page_w - approx_w) / 2.0).max(self.geom.margin_l);
        self.write_text(text, font_size, bold, x);
    }

    pub fn advance(&mut self, delta: f32) {
        self.y -= delta;
    }

    pub fn hline(&mut self, x1: f32, x2: f32) {
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

    pub fn vline(&mut self, x: f32, y_top: f32, y_bottom: f32) {
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

    /// Draw a bordered rectangle of width `w` / height `h` centered on the
    /// current y cursor, left edge at `x`. Restores the cursor afterwards.
    pub fn rect(&mut self, x: f32, w: f32, h: f32) {
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

    /// Render one table row anchored at the left margin. `draw_border` toggles
    /// the cell grid lines; cell text is truncated with an ellipsis to fit its
    /// column. Advances the cursor by `row_h`.
    pub fn row(&mut self, col_widths: &[f32], cells: &[String], bold: bool, draw_border: bool) {
        self.ensure_space(self.geom.row_h);
        let margin_l = self.geom.margin_l;
        let row_h = self.geom.row_h;
        let cell_font = self.geom.cell_font;
        let y_top = self.y + row_h * 0.75;
        let y_bot = self.y - row_h * 0.25;

        if draw_border {
            let total_w: f32 = col_widths.iter().sum();
            let prev_y = self.y;
            self.y = y_top;
            self.hline(margin_l, margin_l + total_w);
            self.y = y_bot;
            self.hline(margin_l, margin_l + total_w);
            self.y = prev_y;
            let mut x = margin_l;
            self.vline(x, y_top, y_bot);
            for w in col_widths {
                x += *w;
                self.vline(x, y_top, y_bot);
            }
        }

        let mut x = margin_l + self.geom.cell_pad_x;
        for (cell, w) in cells.iter().zip(col_widths) {
            let max_chars =
                ((*w - self.geom.cell_trunc_pad) / (cell_font * 0.16)).max(1.0) as usize;
            let display: String = if cell.chars().count() > max_chars {
                let mut s: String = cell.chars().take(max_chars.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                cell.clone()
            };
            self.write_text(&display, cell_font, bold, x);
            x += *w;
        }
        self.advance(row_h);
    }

    pub fn finish(self) -> AppResult<Vec<u8>> {
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
