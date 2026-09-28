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
    BuiltinFont, Line, LinePoint, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Point, Pt,
    RawImage, TextItem, XObjectId, XObjectTransform,
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
    /// Images awaiting registration, in the order [`image`](PdfBuilder::image)
    /// was called.
    ///
    /// They cannot be registered when they are drawn: `add_image` lives on
    /// `PdfDocument`, and the document is not built until [`finish`]. So the
    /// op is emitted now with a placeholder id, and `finish` swaps in the real
    /// one — see [`PLACEHOLDER_XOBJECT_PREFIX`].
    images: Vec<RawImage>,
}

/// Prefix of the stand-in `XObjectId` written into an op before the document
/// exists. `finish` rewrites every id carrying it; one that survives means an
/// image was drawn and never registered, which `finish` treats as a bug rather
/// than emitting a dangling reference that renders as nothing.
const PLACEHOLDER_XOBJECT_PREFIX: &str = "simpel-pending-image-";

/// Points per millimetre (72 pt per inch, 25.4 mm per inch).
const PT_PER_MM: f32 = 72.0 / 25.4;

/// DPI used when placing an image.
///
/// 72 makes one image pixel exactly one point, so the natural size printpdf
/// derives is numerically the pixel count and the scale factors below are a
/// plain ratio. The value never reaches the output — it only decides which
/// number the scale is relative to.
const IMAGE_PLACEMENT_DPI: f32 = 72.0;

impl PdfBuilder {
    pub fn new(title: &str, geom: PageGeometry) -> AppResult<Self> {
        Ok(Self {
            title: title.to_string(),
            page_ops: vec![Vec::new()],
            current_page: 0,
            y: geom.margin_top,
            geom,
            images: Vec::new(),
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

    /// Draw a decoded image inside the box `w` x `h` mm whose left edge is at
    /// `x` and which is centred vertically on the current `y` cursor — the
    /// same anchoring [`rect`](PdfBuilder::rect) uses, so an image and its
    /// frame drawn at one cursor position land on top of each other.
    ///
    /// The image is scaled to FIT, never to fill: its aspect ratio is kept and
    /// it is centred in whatever slack is left. Pasfoto are not all the same
    /// shape, and stretching one to a fixed 35x45 box distorts the face on the
    /// document that exists to identify the person.
    ///
    /// Returns an error when the bytes are not a decodable image. Callers
    /// decide what that means; the SK izin falls back to its placeholder box.
    pub fn image(&mut self, bytes: &[u8], x: f32, w: f32, h: f32) -> AppResult<()> {
        let mut warnings = Vec::new();
        let raw = RawImage::decode_from_bytes(bytes, &mut warnings).map_err(|e| {
            crate::shared::error::AppError::Internal(format!("gambar tak bisa didekode: {e}"))
        })?;
        if raw.width == 0 || raw.height == 0 {
            return Err(crate::shared::error::AppError::Internal(
                "gambar berukuran nol piksel".to_string(),
            ));
        }

        // Fit inside the box, preserving the aspect ratio, then centre the
        // leftover slack.
        let box_w_pt = w * PT_PER_MM;
        let box_h_pt = h * PT_PER_MM;
        let scale = (box_w_pt / raw.width as f32).min(box_h_pt / raw.height as f32);
        let drawn_w_pt = raw.width as f32 * scale;
        let drawn_h_pt = raw.height as f32 * scale;
        let origin_x_pt = x * PT_PER_MM + (box_w_pt - drawn_w_pt) * 0.5;
        let origin_y_pt = (self.y - h * 0.5) * PT_PER_MM + (box_h_pt - drawn_h_pt) * 0.5;

        let id = XObjectId(format!("{PLACEHOLDER_XOBJECT_PREFIX}{}", self.images.len()));
        self.images.push(raw);
        self.push_op(Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Pt(origin_x_pt)),
                translate_y: Some(Pt(origin_y_pt)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(IMAGE_PLACEMENT_DPI),
                ..Default::default()
            },
        });
        Ok(())
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

        // Register the images now that a document exists, then rewrite the
        // placeholder ids the ops carry. Done before the pages are built so a
        // placeholder that never got an image is caught here rather than
        // silently serialising into a reference to nothing — which renders as
        // a blank space, the exact failure this whole change is removing.
        let real_ids: Vec<XObjectId> = self.images.iter().map(|im| doc.add_image(im)).collect();
        let mut page_ops = self.page_ops;
        for ops in page_ops.iter_mut() {
            for op in ops.iter_mut() {
                if let Op::UseXobject { id, .. } = op
                    && let Some(idx) = id.0.strip_prefix(PLACEHOLDER_XOBJECT_PREFIX)
                {
                    let idx: usize = idx.parse().map_err(|_| {
                        crate::shared::error::AppError::Internal(format!(
                            "id gambar sementara rusak: {}",
                            id.0
                        ))
                    })?;
                    *id = real_ids
                        .get(idx)
                        .ok_or_else(|| {
                            crate::shared::error::AppError::Internal(format!(
                                "gambar {idx} dirujuk tapi tak pernah didaftarkan"
                            ))
                        })?
                        .clone();
                }
            }
        }

        let pages: Vec<PdfPage> = page_ops
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

/// Fixtures shared by the PDF tests in this crate.
///
/// `pub` and `cfg(test)`: `pemakaian_bmn::sk_izin_pdf` asserts the SK actually
/// embeds a photo and needs the same image, and a second hand-rolled encoder
/// there would be a second thing to keep correct.
#[cfg(test)]
pub mod tests_support {
    /// A 2x3 PNG, written by hand so the tests carry no fixture file.
    pub fn png_2x3() -> Vec<u8> {
        fn crc32(data: &[u8]) -> u32 {
            let mut crc = 0xFFFF_FFFFu32;
            for &b in data {
                crc ^= b as u32;
                for _ in 0..8 {
                    crc = if crc & 1 != 0 {
                        (crc >> 1) ^ 0xEDB8_8320
                    } else {
                        crc >> 1
                    };
                }
            }
            !crc
        }
        fn chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
            let mut out = (body.len() as u32).to_be_bytes().to_vec();
            out.extend_from_slice(kind);
            out.extend_from_slice(body);
            let mut crc_input = kind.to_vec();
            crc_input.extend_from_slice(body);
            out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
            out
        }
        // 2x3, 8-bit RGB, no interlace.
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&2u32.to_be_bytes());
        ihdr.extend_from_slice(&3u32.to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
        // Three rows of two RGB pixels, each prefixed by filter byte 0.
        let raw: Vec<u8> = (0..3).flat_map(|_| [0u8, 255, 0, 0, 0, 0, 255]).collect();
        // Stored (uncompressed) deflate blocks inside a zlib wrapper, so no
        // compression dependency is needed to build the fixture.
        let mut z = vec![0x78, 0x01];
        z.push(0x01);
        z.extend_from_slice(&(raw.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(raw.len() as u16)).to_le_bytes());
        z.extend_from_slice(&raw);
        let (mut a, mut b) = (1u32, 0u32);
        for &byte in &raw {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());

        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&chunk(b"IHDR", &ihdr));
        png.extend_from_slice(&chunk(b"IDAT", &z));
        png.extend_from_slice(&chunk(b"IEND", &[]));
        png
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::png_2x3;
    use super::*;

    fn geom() -> PageGeometry {
        PageGeometry {
            page_w: 210.0,
            page_h: 297.0,
            margin_l: 20.0,
            margin_r: 20.0,
            margin_top: 270.0,
            margin_bottom: 25.0,
            row_h: 6.0,
            cell_font: 9.0,
            cell_pad_x: 1.5,
            cell_trunc_pad: 3.0,
            auto_paginate: false,
        }
    }

    /// The tripwire for the `printpdf` feature flags.
    ///
    /// `jpeg` and `png` are not default features. Without them
    /// `RawImage::decode_from_bytes` still COMPILES — it resolves to a stub
    /// that always returns `Err` — so dropping them from `Cargo.toml` would
    /// turn every SK's photo back into an empty box with a green build. This
    /// test is what makes that a failure instead.
    #[test]
    fn png_benar_benar_ter_decode() {
        let mut pdf = PdfBuilder::new("uji", geom()).expect("builder");
        pdf.image(&png_2x3(), 20.0, 35.0, 45.0)
            .expect("PNG harus ter-decode — fitur `png` printpdf hilang?");
    }

    #[test]
    fn byte_bukan_gambar_ditolak_bukan_panik() {
        let mut pdf = PdfBuilder::new("uji", geom()).expect("builder");
        // The media host serves an HTML error page for some paths. Reaching
        // the PDF with that body must be an error the caller can fall back
        // from, not a panic inside a request handler compiled with
        // `panic = "abort"`.
        assert!(
            pdf.image(b"<html>not an image</html>", 20.0, 35.0, 45.0)
                .is_err()
        );
    }

    #[test]
    fn gambar_terdaftar_dan_id_sementara_tak_bocor_ke_keluaran() {
        let mut pdf = PdfBuilder::new("uji", geom()).expect("builder");
        pdf.image(&png_2x3(), 20.0, 35.0, 45.0).expect("gambar");
        let bytes = pdf.finish().expect("finish");
        // The placeholder id is a plain string in the op; if `finish` ever
        // stopped rewriting it, the reference would dangle and the image would
        // render as blank space — visually identical to the empty box this
        // change replaces, which is exactly why it is asserted rather than
        // eyeballed.
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(
            !haystack.contains(PLACEHOLDER_XOBJECT_PREFIX),
            "id sementara bocor ke PDF — `finish` tidak menukarnya"
        );
        assert!(bytes.len() > 500, "PDF terlalu kecil untuk memuat gambar");
    }

    #[test]
    fn aspek_dipertahankan_bukan_diregangkan() {
        // A 2x3 image in a 35x45 mm box: the box is 0.778 wide-per-tall and
        // the image 0.667, so height is the binding dimension and the drawn
        // width must come out NARROWER than the box rather than filling it.
        let mut pdf = PdfBuilder::new("uji", geom()).expect("builder");
        pdf.image(&png_2x3(), 20.0, 35.0, 45.0).expect("gambar");
        let Op::UseXobject { transform, .. } = &pdf.page_ops[0][0] else {
            panic!("op pertama bukan UseXobject");
        };
        let scale = transform.scale_x.expect("scale_x");
        assert!(
            (scale - transform.scale_y.expect("scale_y")).abs() < f32::EPSILON,
            "skala x dan y wajib sama, kalau tidak wajahnya melar"
        );
        let drawn_w_mm = 2.0 * scale / PT_PER_MM;
        let drawn_h_mm = 3.0 * scale / PT_PER_MM;
        assert!(drawn_w_mm < 35.0, "lebar {drawn_w_mm} mestinya < kotak");
        assert!(
            (drawn_h_mm - 45.0).abs() < 0.01,
            "tinggi {drawn_h_mm} mestinya pas mengisi kotak"
        );
    }
}
