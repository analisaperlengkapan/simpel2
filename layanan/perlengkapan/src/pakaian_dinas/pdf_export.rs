//! # Pakaian Dinas PDF export
//!
//! Renders the shared report view-models ([`DaftarReport`] / [`RekapReport`]) as
//! the legacy simpelv1 PDF: A4 landscape, 10 mm margins, DejaVu Sans Condensed
//! at 7 pt (what mPDF sets `sans-serif` in), a centred three-line heading, then
//! full-width black-bordered tables. Rekap prints one table per
//! (pakaian × gender), each on its own page; daftar prints one table per satker,
//! each on its own page, with a "Data Kosong" row for a satker with nobody in it.
//!
//! The geometry below (column shares, row heights, heading positions) was
//! measured from a PDF rendered by the legacy templates themselves with the real
//! mPDF, not guessed; see `legacy_report_parity` in the integration tests for
//! the structural assertions that keep it from drifting.
//!
//! Font metrics come from the embedded faces, so cell text is measured, wrapped
//! and centred with the real glyph widths.

use printpdf::{
    Color, FontId, Line, LinePoint, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, Rgb, TextItem,
};
use uuid::Uuid;

use super::models::{DaftarReport, LaporanFilter, RekapReport, ReportHeader};
use super::services::PakaianDinasService;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;

static FONT_REGULAR: &[u8] = include_bytes!("fonts/DejaVuSansCondensed.ttf");
static FONT_BOLD: &[u8] = include_bytes!("fonts/DejaVuSansCondensed-Bold.ttf");

// ─── geometry, in points, measured from the legacy output ────────────────

const PAGE_W: f32 = 841.89;
const PAGE_H: f32 = 595.28;
/// 10 mm, mPDF's default page margin.
const MARGIN: f32 = 28.35;
const TABLE_W: f32 = PAGE_W - 2.0 * MARGIN;
const BODY_PT: f32 = 7.0;
/// `h2` and `h3` at the sizes mPDF gives them against a 7 pt body.
const H2_PT: f32 = 10.5;
const H3_PT: f32 = 8.2;
const BORDER_PT: f32 = 0.75;
/// `td { padding: 10px }` and the default `th` padding, in points.
const TD_PAD: f32 = 7.5;
const TH_PAD: f32 = 1.5;
/// One line of body text, and the row height it gives a 10 px-padded cell.
const LINE_H: f32 = 9.4;
/// Heading rows of the rekap table (two rows) and the daftar table (one row).
const TH_REKAP_1: f32 = 9.4;
const TH_REKAP_2: f32 = 11.3;
const TH_DAFTAR: f32 = 10.7;

const HEAD_H2_BASELINE: f32 = 38.4;
const HEAD_H3_BASELINE: f32 = 50.8;
const HEAD_PERIODE_BASELINE: f32 = 67.8;
/// `p { margin: 1em }` between consecutive paragraphs.
const PARA_GAP: f32 = 16.4;

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Left,
    Center,
}

/// Where a cell sits on the page, in points from the top-left of the sheet.
#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// How a cell's text is set.
#[derive(Clone, Copy)]
struct Style {
    align: Align,
    bold: bool,
    pad: f32,
}

/// Column heading: bold, centred, th padding.
const TH: Style = Style {
    align: Align::Center,
    bold: true,
    pad: TH_PAD,
};
const TD_CENTER: Style = Style {
    align: Align::Center,
    bold: false,
    pad: TD_PAD,
};
const TD_CENTER_BOLD: Style = Style {
    align: Align::Center,
    bold: true,
    pad: TD_PAD,
};
const TD_LEFT: Style = Style {
    align: Align::Left,
    bold: false,
    pad: TD_PAD,
};

struct Canvas {
    regular: ParsedFont,
    bold: ParsedFont,
    regular_id: FontId,
    bold_id: FontId,
    doc: PdfDocument,
    pages: Vec<Vec<Op>>,
}

impl Canvas {
    fn new(title: &str) -> AppResult<Self> {
        let mut warnings = Vec::new();
        let regular = ParsedFont::from_bytes(FONT_REGULAR, 0, &mut warnings)
            .ok_or_else(|| AppError::Internal("font laporan tidak terbaca".to_string()))?;
        let bold = ParsedFont::from_bytes(FONT_BOLD, 0, &mut warnings)
            .ok_or_else(|| AppError::Internal("font tebal laporan tidak terbaca".to_string()))?;
        let mut doc = PdfDocument::new(title);
        let regular_id = doc.add_font(&regular);
        let bold_id = doc.add_font(&bold);
        Ok(Self {
            regular,
            bold,
            regular_id,
            bold_id,
            doc,
            pages: vec![Vec::new()],
        })
    }

    fn new_page(&mut self) {
        self.pages.push(Vec::new());
    }

    fn ops(&mut self) -> &mut Vec<Op> {
        self.pages.last_mut().expect("a page always exists")
    }

    /// Advance width of `text` in points, from the font's own glyph widths.
    fn width(&self, text: &str, size: f32, bold: bool) -> f32 {
        let font = if bold { &self.bold } else { &self.regular };
        let upem = f32::from(font.font_metrics.units_per_em.max(1));
        text.chars()
            .map(|c| {
                font.lookup_glyph_index(c as u32)
                    .map(|g| f32::from(font.get_horizontal_advance(g)))
                    .unwrap_or(0.5 * upem)
                    * size
                    / upem
            })
            .sum()
    }

    fn mm(pt: f32) -> Mm {
        Mm(pt * 25.4 / 72.0)
    }

    /// `y` is measured from the TOP of the page, as the layout is described.
    fn point(x: f32, y_top: f32) -> Point {
        Point::new(Self::mm(x), Self::mm(PAGE_H - y_top))
    }

    fn text(&mut self, text: &str, x: f32, baseline: f32, size: f32, bold: bool) {
        if text.is_empty() {
            return;
        }
        let font = if bold {
            self.bold_id.clone()
        } else {
            self.regular_id.clone()
        };
        let pos = Self::point(x, baseline);
        let ops = self.ops();
        ops.push(Op::StartTextSection);
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font),
            size: Pt(size),
        });
        ops.push(Op::SetTextCursor { pos });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        });
        ops.push(Op::EndTextSection);
    }

    fn centered(&mut self, text: &str, baseline: f32, size: f32, bold: bool) {
        let w = self.width(text, size, bold);
        self.text(text, (PAGE_W - w) / 2.0, baseline, size, bold);
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let line = Line {
            points: vec![
                LinePoint {
                    p: Self::point(x1, y1),
                    bezier: false,
                },
                LinePoint {
                    p: Self::point(x2, y2),
                    bezier: false,
                },
            ],
            is_closed: false,
        };
        let ops = self.ops();
        ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(BORDER_PT) });
        ops.push(Op::DrawLine { line });
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.line(x, y, x + w, y);
        self.line(x, y + h, x + w, y + h);
        self.line(x, y, x, y + h);
        self.line(x + w, y, x + w, y + h);
    }

    /// Break `text` into lines no wider than `max_w`, on spaces, hard-breaking a
    /// word that is wider than the line by itself.
    fn wrap(&self, text: &str, max_w: f32, bold: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for word in text.split_whitespace() {
            let candidate = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{cur} {word}")
            };
            if self.width(&candidate, BODY_PT, bold) <= max_w || cur.is_empty() {
                cur = candidate;
                while self.width(&cur, BODY_PT, bold) > max_w && cur.chars().count() > 1 {
                    let mut head = String::new();
                    for ch in cur.chars() {
                        if self.width(&format!("{head}{ch}"), BODY_PT, bold) > max_w {
                            break;
                        }
                        head.push(ch);
                    }
                    if head.is_empty() {
                        break;
                    }
                    let rest: String = cur.chars().skip(head.chars().count()).collect();
                    lines.push(head);
                    cur = rest;
                }
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            }
        }
        if !cur.is_empty() || lines.is_empty() {
            lines.push(cur);
        }
        lines
    }

    /// A bordered cell with `lines` of text, vertically centred.
    fn cell(&mut self, r: Rect, lines: &[String], st: Style) {
        self.rect(r.x, r.y, r.w, r.h);
        let block_h = LINE_H * lines.len() as f32;
        let top = r.y + (r.h - block_h) / 2.0;
        for (i, l) in lines.iter().enumerate() {
            let tw = self.width(l, BODY_PT, st.bold);
            let tx = match st.align {
                Align::Left => r.x + st.pad,
                Align::Center => r.x + (r.w - tw) / 2.0,
            };
            let baseline = top + LINE_H * i as f32 + 0.5 * LINE_H + 0.36 * BODY_PT;
            self.text(l, tx, baseline, BODY_PT, st.bold);
        }
    }

    fn finish(mut self) -> AppResult<Vec<u8>> {
        let pages: Vec<PdfPage> = std::mem::take(&mut self.pages)
            .into_iter()
            .map(|ops| PdfPage::new(Mm(PAGE_W * 25.4 / 72.0), Mm(PAGE_H * 25.4 / 72.0), ops))
            .collect();
        self.doc.with_pages(pages);
        let mut warnings = Vec::new();
        Ok(self.doc.save(&PdfSaveOptions::default(), &mut warnings))
    }
}

/// The centred title block and the filter lines; returns the y where the first
/// table starts (page top, in points).
fn heading(c: &mut Canvas, header: &ReportHeader) -> f32 {
    c.centered(&header.judul, HEAD_H2_BASELINE, H2_PT, true);
    c.centered(&header.inst_nama, HEAD_H3_BASELINE, H3_PT, true);
    c.centered(
        &header.periode_line(),
        HEAD_PERIODE_BASELINE,
        BODY_PT,
        false,
    );
    let mut baseline = HEAD_PERIODE_BASELINE;
    for (label, value) in &header.filters {
        baseline += PARA_GAP;
        c.text(
            &format!("{label} : {value}"),
            MARGIN,
            baseline,
            BODY_PT,
            false,
        );
    }
    baseline + 2.0 + 7.0 + 1.9
}

fn bottom() -> f32 {
    PAGE_H - MARGIN
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

// ─── Rekap ───────────────────────────────────────────────────────────────

pub fn render_rekap_pdf(report: &RekapReport) -> AppResult<Vec<u8>> {
    let mut c = Canvas::new(&report.header.judul)?;
    let mut y_start = heading(&mut c, &report.header);

    for (bi, block) in report.blocks.iter().enumerate() {
        if bi > 0 {
            // Every table starts a page of its own, with no title above it.
            c.new_page();
            y_start = MARGIN;
        }
        let n = block.ukuran_labels.len();
        // No = 5 %, Satker = 20 % (the legacy `width` attributes); the rest is
        // shared by the size columns and Jumlah, which is a little wider.
        let w_no = TABLE_W * 0.05;
        let w_satker = TABLE_W * 0.20;
        let unit = TABLE_W * 0.75 / (n as f32 + 1.2);
        let w_size = unit;
        let w_total = unit * 1.2;
        let x_size = MARGIN + w_no + w_satker;

        let draw_head = |c: &mut Canvas, y: f32| -> f32 {
            let h = TH_REKAP_1 + TH_REKAP_2;
            let at = |x: f32, y: f32, w: f32, h: f32| Rect { x, y, w, h };
            c.cell(at(MARGIN, y, w_no, h), &["No".to_string()], TH);
            c.cell(
                at(MARGIN + w_no, y, w_satker, h),
                &["Satker".to_string()],
                TH,
            );
            let title = format!("{} - {}", block.pakaian_nama, block.gender);
            c.cell(at(x_size, y, w_size * n as f32, TH_REKAP_1), &[title], TH);
            for (i, label) in block.ukuran_labels.iter().enumerate() {
                let cell = at(
                    x_size + w_size * i as f32,
                    y + TH_REKAP_1,
                    w_size,
                    TH_REKAP_2,
                );
                c.cell(cell, std::slice::from_ref(label), TH);
            }
            c.cell(
                at(x_size + w_size * n as f32, y, w_total, h),
                &["Jumlah".to_string()],
                TH,
            );
            y + h
        };

        let row_h = LINE_H + 2.0 * TD_PAD;
        let mut y = draw_head(&mut c, y_start);
        for (i, s) in block.satkers.iter().enumerate() {
            let name_lines = c.wrap(&s.satker_nama, w_satker - 2.0 * TD_PAD, false);
            let h = (LINE_H * name_lines.len() as f32 + 2.0 * TD_PAD).max(row_h);
            if y + h > bottom() {
                c.new_page();
                y = draw_head(&mut c, MARGIN);
            }
            c.cell(
                Rect {
                    x: MARGIN,
                    y,
                    w: w_no,
                    h,
                },
                &[(i + 1).to_string()],
                TD_CENTER,
            );
            c.cell(
                Rect {
                    x: MARGIN + w_no,
                    y,
                    w: w_satker,
                    h,
                },
                &name_lines,
                TD_LEFT,
            );
            for (k, count) in s.counts.iter().enumerate() {
                let cell = Rect {
                    x: x_size + w_size * k as f32,
                    y,
                    w: w_size,
                    h,
                };
                c.cell(cell, &[count.to_string()], TD_CENTER);
            }
            let total = Rect {
                x: x_size + w_size * n as f32,
                y,
                w: w_total,
                h,
            };
            c.cell(total, &[s.total.to_string()], TD_CENTER_BOLD);
            y += h;
        }
        if y + row_h > bottom() {
            c.new_page();
            y = draw_head(&mut c, MARGIN);
        }
        // "Jumlah" spans the No and Satker columns.
        let label = Rect {
            x: MARGIN,
            y,
            w: w_no + w_satker,
            h: row_h,
        };
        c.cell(label, &["Jumlah".to_string()], TD_CENTER_BOLD);
        for (k, total) in block.summary.iter().enumerate() {
            let cell = Rect {
                x: x_size + w_size * k as f32,
                y,
                w: w_size,
                h: row_h,
            };
            c.cell(cell, &[total.to_string()], TD_CENTER_BOLD);
        }
        let grand = Rect {
            x: x_size + w_size * n as f32,
            y,
            w: w_total,
            h: row_h,
        };
        c.cell(grand, &[block.summary_total.to_string()], TD_CENTER_BOLD);
    }
    c.finish()
}

// ─── Daftar ──────────────────────────────────────────────────────────────

/// Column headings and alignment of the fixed part of the daftar table, as the
/// legacy template prints them. The legacy heading reads "Busana Mulsimah";
/// that is a typo, corrected here.
const DAFTAR_FIXED: [(&str, Align); 8] = [
    ("No", Align::Center),
    ("NIP", Align::Left),
    ("Nama", Align::Left),
    ("Jabatan", Align::Left),
    ("Golongan", Align::Center),
    ("Status", Align::Center),
    ("Gender", Align::Center),
    ("Busana Muslimah", Align::Center),
];
/// Widest a text column is allowed to ask for before its text wraps.
const MAX_COL_W: f32 = 150.0;

pub fn render_daftar_pdf(report: &DaftarReport) -> AppResult<Vec<u8>> {
    let mut c = Canvas::new(&report.header.judul)?;
    let mut y_start = heading(&mut c, &report.header);

    let heads: Vec<(String, Align)> = DAFTAR_FIXED
        .iter()
        .map(|(h, a)| ((*h).to_string(), *a))
        .chain(report.columns.iter().map(|h| (h.clone(), Align::Center)))
        .collect();

    // Column widths: what the content asks for, scaled so the table fills the
    // page, which is how an auto-layout HTML table settles.
    let mut want: Vec<f32> = heads
        .iter()
        .map(|(h, _)| c.width(h, BODY_PT, true) + 2.0 * TH_PAD)
        .collect();
    for g in &report.satkers {
        for r in &g.rows {
            let cells = daftar_cells(r);
            for (i, text) in cells.iter().enumerate() {
                let w = (c.width(text, BODY_PT, false) + 2.0 * TD_PAD).min(MAX_COL_W);
                if w > want[i] {
                    want[i] = w;
                }
            }
        }
    }
    // The running-number column never narrows below what mPDF gives it (30 pt
    // after scaling), whatever its content: measured, not derived.
    want[0] = want[0].max(23.0);
    let scale = TABLE_W / want.iter().sum::<f32>();
    let widths: Vec<f32> = want.iter().map(|w| w * scale).collect();
    let xs: Vec<f32> = widths
        .iter()
        .scan(MARGIN, |x, w| {
            let at = *x;
            *x += w;
            Some(at)
        })
        .collect();

    let draw_head = |c: &mut Canvas, y: f32| -> f32 {
        for (i, (h, _)) in heads.iter().enumerate() {
            c.cell(
                Rect {
                    x: xs[i],
                    y,
                    w: widths[i],
                    h: TH_DAFTAR,
                },
                std::slice::from_ref(h),
                TH,
            );
        }
        y + TH_DAFTAR
    };

    for (si, group) in report.satkers.iter().enumerate() {
        // The satker's name sits above its table; each satker starts a page.
        let top = if si == 0 {
            y_start
        } else {
            c.new_page();
            MARGIN
        };
        // On the first page the heading follows the filter lines; on a page of
        // its own it sits where the legacy page puts it (measured).
        let (heading_baseline, table_top) = if si == 0 {
            (top + H3_PT + 6.5, top + H3_PT + 6.5 + 9.3)
        } else {
            (37.0, 51.4)
        };
        c.text(&group.satker_nama, MARGIN, heading_baseline, H3_PT, true);
        y_start = table_top;
        let mut y = draw_head(&mut c, y_start);

        if group.rows.is_empty() {
            let h = LINE_H + 2.0 * TD_PAD;
            let whole = Rect {
                x: MARGIN,
                y,
                w: TABLE_W,
                h,
            };
            c.cell(whole, &["Data Kosong".to_string()], TD_CENTER);
            continue;
        }
        for (ri, r) in group.rows.iter().enumerate() {
            let mut cells = daftar_cells(r);
            cells[0] = (ri + 1).to_string();
            let wrapped: Vec<Vec<String>> = cells
                .iter()
                .enumerate()
                .map(|(i, t)| c.wrap(t, widths[i] - 2.0 * TD_PAD, false))
                .collect();
            let lines = wrapped.iter().map(Vec::len).max().unwrap_or(1);
            let h = LINE_H * lines as f32 + 2.0 * TD_PAD;
            if y + h > bottom() {
                c.new_page();
                y = draw_head(&mut c, MARGIN);
            }
            for (i, lines) in wrapped.iter().enumerate() {
                let style = Style {
                    align: heads[i].1,
                    bold: false,
                    pad: TD_PAD,
                };
                c.cell(
                    Rect {
                        x: xs[i],
                        y,
                        w: widths[i],
                        h,
                    },
                    lines,
                    style,
                );
            }
            y += h;
        }
    }
    c.finish()
}

/// The text of one daftar row, in column order; index 0 (the running number) is
/// filled in by the caller.
fn daftar_cells(r: &super::models::DaftarRow) -> Vec<String> {
    let mut v = vec![
        String::new(),
        r.nip.clone(),
        r.nama.clone(),
        r.jabatan.clone(),
        r.golongan.clone(),
        r.status.clone(),
        r.gender.clone(),
        r.hijab.clone(),
    ];
    v.extend(r.sizes.iter().cloned());
    v
}

#[cfg(test)]
mod tests {
    use super::super::models::{DaftarRow, DaftarSatkerGroup, RekapBlock, RekapSatkerRow};
    use super::*;

    fn header() -> ReportHeader {
        ReportHeader {
            judul: "Kebutuhan PDH".to_string(),
            inst_nama: "WILAYAH KEJAKSAAN TINGGI DKI JAKARTA".to_string(),
            ..Default::default()
        }
    }

    fn rekap(blocks: usize, satkers: usize) -> RekapReport {
        RekapReport {
            header: header(),
            blocks: (0..blocks)
                .map(|i| RekapBlock {
                    pakaian_nama: format!("Pakaian {i}"),
                    ukuran_group: "BAJU".to_string(),
                    gender: if i % 2 == 0 { "L" } else { "P" }.to_string(),
                    ukuran_labels: ["S", "M", "L"].iter().map(|s| s.to_string()).collect(),
                    satkers: (0..satkers)
                        .map(|k| RekapSatkerRow {
                            satker_nama: format!("Satker {k}"),
                            counts: vec![0, 1, 0],
                            total: 1,
                        })
                        .collect(),
                    summary: vec![0, satkers as i64, 0],
                    summary_total: satkers as i64,
                })
                .collect(),
        }
    }

    fn page_count(bytes: &[u8]) -> usize {
        let mut warnings = Vec::new();
        PdfDocument::parse(bytes, &printpdf::PdfParseOptions::default(), &mut warnings)
            .expect("the output parses as a PDF")
            .pages
            .len()
    }

    #[test]
    fn rekap_is_a_valid_pdf_with_one_page_per_block() {
        let bytes = render_rekap_pdf(&rekap(3, 4)).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(
            page_count(&bytes),
            3,
            "one page per (pakaian x gender) table"
        );
    }

    #[test]
    fn a_table_longer_than_a_page_continues_on_the_next() {
        let bytes = render_rekap_pdf(&rekap(1, 60)).unwrap();
        assert!(
            page_count(&bytes) >= 3,
            "60 rows at 24 pt cannot fit one landscape page"
        );
    }

    #[test]
    fn daftar_gives_each_satker_a_page_and_an_empty_one_still_prints() {
        let report = DaftarReport {
            header: header(),
            columns: vec!["Celana PDH".to_string(), "Kemeja PDH".to_string()],
            satkers: vec![
                DaftarSatkerGroup {
                    satker_nama: "KEJAKSAAN NEGERI A".to_string(),
                    rows: vec![DaftarRow {
                        nip: "200000000000000001".to_string(),
                        nama: "Pegawai Satu".to_string(),
                        jabatan: "Operator Satker".to_string(),
                        golongan: "III/a".to_string(),
                        status: "J".to_string(),
                        gender: "L".to_string(),
                        hijab: "T".to_string(),
                        sizes: vec!["34".to_string(), "L".to_string()],
                    }],
                },
                DaftarSatkerGroup {
                    satker_nama: "KEJAKSAAN NEGERI KOSONG".to_string(),
                    rows: vec![],
                },
            ],
        };
        let bytes = render_daftar_pdf(&report).unwrap();
        assert_eq!(page_count(&bytes), 2);
    }

    #[test]
    fn measured_widths_use_the_real_glyphs() {
        let c = Canvas::new("t").unwrap();
        let narrow = c.width("iii", BODY_PT, false);
        let wide = c.width("WWW", BODY_PT, false);
        assert!(
            wide > narrow * 2.0,
            "a proportional face: W is much wider than i"
        );
        assert!(c.width("Jumlah", BODY_PT, true) > c.width("Jumlah", BODY_PT, false));
    }
}
