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
//!
//! Updated for printpdf 0.9.x ops-based API.

use printpdf::{
    BuiltinFont, Line, LinePoint, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::shared::error::AppResult;

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
///
/// In printpdf 0.9.x, content is expressed as `Vec<Op>` per page rather than
/// mutable layer references.  This builder collects ops for the current page
/// and finalises them into `PdfPage` objects on [`finish`](PdfBuilder::finish).
pub struct PdfBuilder {
    title: String,
    /// Ops collected for each page so far.
    page_ops: Vec<Vec<Op>>,
    current_page: usize,
    geom: PageGeometry,
    /// Y cursor in mm from the bottom of the page (printpdf convention).
    /// Public so callers can place free-form content (e.g. an image) relative
    /// to the current row.
    pub y: f32,
}

impl PdfBuilder {
    pub fn new(title: &str, geom: PageGeometry) -> AppResult<Self> {
        Ok(Self {
            title: title.to_string(),
            page_ops: vec![Vec::new()],
            current_page: 0,
            y: geom.margin_top,
            geom,
        })
    }

    pub fn geom(&self) -> &PageGeometry {
        &self.geom
    }

    /// Push an op onto the current page's op list.
    fn push_op(&mut self, op: Op) {
        self.page_ops[self.current_page].push(op);
    }

    pub fn new_page(&mut self) {
        self.page_ops.push(Vec::new());
        self.current_page = self.page_ops.len() - 1;
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
        let font = if bold {
            BuiltinFont::HelveticaBold
        } else {
            BuiltinFont::Helvetica
        };
        self.push_op(Op::StartTextSection);
        self.push_op(Op::SetFont {
            font: printpdf::PdfFontHandle::Builtin(font),
            size: Pt(font_size),
        });
        self.push_op(Op::SetTextCursor {
            pos: Point::new(Mm(x), Mm(self.y)),
        });
        self.push_op(Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        });
        self.push_op(Op::EndTextSection);
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
        let line = Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(x1), Mm(self.y)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(x2), Mm(self.y)),
                    bezier: false,
                },
            ],
            is_closed: false,
        };
        self.push_op(Op::DrawLine { line });
    }

    pub fn vline(&mut self, x: f32, y_top: f32, y_bottom: f32) {
        let line = Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(x), Mm(y_top)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(x), Mm(y_bottom)),
                    bezier: false,
                },
            ],
            is_closed: false,
        };
        self.push_op(Op::DrawLine { line });
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
        let mut doc = PdfDocument::new(&self.title);
        let pages: Vec<PdfPage> = self
            .page_ops
            .into_iter()
            .map(|ops| PdfPage::new(Mm(self.geom.page_w), Mm(self.geom.page_h), ops))
            .collect();
        doc.with_pages(pages);
        let mut warnings = Vec::new();
        let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
        if !warnings.is_empty() {
            tracing::warn!("PDF save warnings: {:?}", warnings);
        }
        Ok(bytes)
    }
}
