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

use super::models::{DaftarReport, LaporanFilter, RekapBlock, RekapReport, ReportHeader};
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

// ─── Fitting a table to the page ─────────────────────────────────────────
//
// The legacy geometry above is exact for the tables the legacy templates printed: a few size
// columns, eight fixed daftar columns and a handful of garments. Since V019 the content is data: a
// trouser table has 27 sizes (26..52), a PDL campaign has six garments, a chest name is up to 100
// characters. Where the content fits, nothing below changes a single coordinate. Where it does not,
// the table adapts instead of running off the page: tighter cell padding first, then the columns
// that do not fit continue on further pages (the identifying columns repeat), never a figure drawn
// outside its cell.

/// Air on each side of a figure when the legacy 7.5 pt cell padding cannot be afforded.
const COMPACT_PAD: f32 = 2.5;

/// How much of the page the columns' MINIMUM widths may take before another column is moved to a
/// further page. Filling the page to its minimums leaves the text columns (Nama, Jabatan, a chest
/// name) at one word per line, six to eleven lines a row: valid, and nearly unreadable. The share
/// that is left is what those columns grow into.
const FIT_SHARE: f32 = 0.85;

/// Share `total` between columns that each want `want[i]` and can give up to `want[i] - min[i]`.
/// All columns' wishes fit: they are stretched to fill the page, as an auto-layout HTML table settles
/// (the legacy behaviour). Otherwise everyone starts at their minimum and the slack is shared in
/// proportion to what each can still give up. The result always sums to `total` and no column goes
/// below its minimum, so callers must only ask when `sum(min) <= total`.
fn share_width(want: &[f32], min: &[f32], total: f32) -> Vec<f32> {
    let sum_want: f32 = want.iter().sum();
    if sum_want <= total {
        let scale = total / sum_want;
        return want.iter().map(|w| w * scale).collect();
    }
    let sum_min: f32 = min.iter().sum();
    let room: f32 = want.iter().zip(min).map(|(w, m)| (w - m).max(0.0)).sum();
    let slack = (total - sum_min).max(0.0);
    if room <= f32::EPSILON {
        let extra = slack / want.len() as f32;
        return min.iter().map(|m| m + extra).collect();
    }
    let f = (slack / room).min(1.0);
    want.iter()
        .zip(min)
        .map(|(w, m)| m + f * (w - m).max(0.0))
        .collect()
}

/// Which columns of the daftar table go on which page, and how wide each is there.
///
/// `fixed` columns (0..fixed) lead the first part and the first `repeat` of them lead every later
/// part. Garment columns are added to a part while their minimum widths still fit the page; the rest
/// continue on the next part. Everything that fits one part is a single part (the common case).
fn plan_columns(
    want: &[f32],
    min: &[f32],
    fixed: usize,
    repeat: usize,
    total: f32,
) -> Vec<(Vec<usize>, Vec<f32>)> {
    let n = want.len();
    let fits = |cols: &[usize]| cols.iter().map(|&i| min[i]).sum::<f32>() <= total * FIT_SHARE;
    let all: Vec<usize> = (0..n).collect();
    let layout = |cols: Vec<usize>| {
        let w: Vec<f32> = cols.iter().map(|&i| want[i]).collect();
        let m: Vec<f32> = cols.iter().map(|&i| min[i]).collect();
        let widths = share_width(&w, &m, total);
        (cols, widths)
    };
    if fits(&all) {
        return vec![layout(all)];
    }
    let mut parts: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = (0..fixed).collect();
    for i in fixed..n {
        let mut trial = current.clone();
        trial.push(i);
        if fits(&trial) || current.len() <= repeat.min(fixed) {
            current = trial;
        } else {
            parts.push(std::mem::replace(&mut current, (0..repeat).collect()));
            current.push(i);
        }
    }
    parts.push(current);
    parts.into_iter().map(layout).collect()
}

/// `text` cut to one line of at most `max_w`, ending in an ellipsis when it had to be shortened.
fn one_line(c: &Canvas, text: &str, max_w: f32, bold: bool) -> String {
    if c.width(text, BODY_PT, bold) <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{out}{ch}…");
        if c.width(&candidate, BODY_PT, bold) > max_w {
            break;
        }
        out.push(ch);
    }
    format!("{out}…")
}

/// The first of `candidates` that fits `max_w` on one line (the last one, cut, if none does): a column
/// heading that must stay one word-width short ("Jumlah", then "Jml") rather than run past its cell.
fn fit_label(c: &Canvas, candidates: &[&str], max_w: f32, bold: bool) -> String {
    for cand in candidates {
        if c.width(cand, BODY_PT, bold) <= max_w {
            return (*cand).to_string();
        }
    }
    one_line(c, candidates[candidates.len() - 1], max_w, bold)
}

/// The longest unbreakable word of `text`: how narrow a wrapping column can be before it must split a word.
fn longest_word(c: &Canvas, text: &str, bold: bool) -> f32 {
    text.split_whitespace()
        .map(|w| c.width(w, BODY_PT, bold))
        .fold(0.0, f32::max)
}

// ─── Rekap ───────────────────────────────────────────────────────────────

/// How many of a block's size columns one page can hold: all of them (the legacy table) unless the
/// widest figure any of them carries does not fit the column the legacy split would give it.
fn rekap_cols_per_page(c: &Canvas, block: &RekapBlock) -> usize {
    let n = block.ukuran_labels.len();
    if n == 0 {
        return 0;
    }
    let figure = |v: i64, bold: bool| c.width(&v.to_string(), BODY_PT, bold);
    let widest_size = block
        .ukuran_labels
        .iter()
        .map(|l| c.width(l, BODY_PT, true))
        .chain(
            block
                .satkers
                .iter()
                .flat_map(|s| s.counts.iter().map(|&v| figure(v, false))),
        )
        .chain(block.summary.iter().map(|&v| figure(v, true)))
        .fold(0.0, f32::max);
    let widest_total = block
        .satkers
        .iter()
        .map(|s| figure(s.total, true))
        .chain(std::iter::once(figure(block.summary_total, true)))
        .fold(0.0, f32::max);
    // The total column is 1.2 units wide, so its need is shared by 1.2.
    let needed = (widest_size + 2.0 * COMPACT_PAD).max((widest_total + 2.0 * COMPACT_PAD) / 1.2);
    let avail = TABLE_W * 0.75;
    let fit = (avail / needed - 1.2).floor();
    (fit.max(1.0) as usize).min(n)
}

pub fn render_rekap_pdf(report: &RekapReport) -> AppResult<Vec<u8>> {
    let mut c = Canvas::new(&report.header.judul)?;
    let mut y_start = heading(&mut c, &report.header);
    let mut first = true;

    for block in &report.blocks {
        // One page-run per slice of size columns; a block that fits is a single slice.
        let per_page = rekap_cols_per_page(&c, block).max(1);
        let n_all = block.ukuran_labels.len();
        let slices: Vec<std::ops::Range<usize>> = if n_all == 0 {
            std::iter::once(0..0).collect()
        } else {
            (0..n_all)
                .step_by(per_page)
                .map(|s| s..(s + per_page).min(n_all))
                .collect()
        };
        for (si, slice) in slices.iter().enumerate() {
            if !first {
                // Every table starts a page of its own, with no title above it.
                c.new_page();
                y_start = MARGIN;
            }
            first = false;
            draw_rekap_slice(
                &mut c,
                block,
                slice.clone(),
                (si + 1, slices.len()),
                y_start,
            );
        }
    }
    c.finish()
}

/// One table (or one slice of its size columns): No, Satker, the sizes in `cols`, and the satker's
/// Jumlah, which is always the whole row's total, not the slice's.
fn draw_rekap_slice(
    c: &mut Canvas,
    block: &RekapBlock,
    cols: std::ops::Range<usize>,
    (part, parts): (usize, usize),
    y_start: f32,
) {
    let labels = &block.ukuran_labels[cols.clone()];
    let n = labels.len();
    // No = 5 %, Satker = 20 % (the legacy `width` attributes); the rest is
    // shared by the size columns and Jumlah, which is a little wider.
    let w_no = TABLE_W * 0.05;
    let w_satker = TABLE_W * 0.20;
    let unit = TABLE_W * 0.75 / (n as f32 + 1.2);
    let w_size = unit;
    let w_total = unit * 1.2;
    let x_size = MARGIN + w_no + w_satker;

    let title = if parts > 1 {
        format!("{} - {} ({part}/{parts})", block.pakaian_nama, block.gender)
    } else {
        format!("{} - {}", block.pakaian_nama, block.gender)
    };
    let title = one_line(
        c,
        &title,
        (w_size * n as f32 - 2.0 * TH_PAD).max(10.0),
        true,
    );

    // The total column is 1.2 size columns wide: with many sizes "Jumlah" no longer fits it.
    let total_label = fit_label(c, &["Jumlah", "Jml"], w_total - 2.0 * COMPACT_PAD, true);

    let draw_head = |c: &mut Canvas, y: f32| -> f32 {
        let h = TH_REKAP_1 + TH_REKAP_2;
        let at = |x: f32, y: f32, w: f32, h: f32| Rect { x, y, w, h };
        c.cell(at(MARGIN, y, w_no, h), &["No".to_string()], TH);
        c.cell(
            at(MARGIN + w_no, y, w_satker, h),
            &["Satker".to_string()],
            TH,
        );
        c.cell(
            at(x_size, y, w_size * n as f32, TH_REKAP_1),
            std::slice::from_ref(&title),
            TH,
        );
        for (i, label) in labels.iter().enumerate() {
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
            std::slice::from_ref(&total_label),
            TH,
        );
        y + h
    };

    let row_h = LINE_H + 2.0 * TD_PAD;
    let mut y = draw_head(c, y_start);
    for (i, s) in block.satkers.iter().enumerate() {
        let name_lines = c.wrap(&s.satker_nama, w_satker - 2.0 * TD_PAD, false);
        let h = (LINE_H * name_lines.len() as f32 + 2.0 * TD_PAD).max(row_h);
        if y + h > bottom() {
            c.new_page();
            y = draw_head(c, MARGIN);
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
        for (k, count) in s.counts[cols.clone()].iter().enumerate() {
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
        y = draw_head(c, MARGIN);
    }
    // "Jumlah" spans the No and Satker columns.
    let label = Rect {
        x: MARGIN,
        y,
        w: w_no + w_satker,
        h: row_h,
    };
    c.cell(label, &["Jumlah".to_string()], TD_CENTER_BOLD);
    for (k, total) in block.summary[cols].iter().enumerate() {
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
/// The identifying columns (No, NIP, Nama) that lead every page when the garment columns continue.
const DAFTAR_REPEAT: usize = 3;

/// What a daftar column asks for and the least it can live with, at cell padding `pad`: its widest
/// text (heading included, capped at [`MAX_COL_W`]) and its longest unbreakable word.
fn daftar_needs(
    c: &Canvas,
    heads: &[(String, Align)],
    report: &DaftarReport,
    pad: f32,
) -> (Vec<f32>, Vec<f32>) {
    let mut want: Vec<f32> = heads
        .iter()
        .map(|(h, _)| c.width(h, BODY_PT, true) + 2.0 * TH_PAD)
        .collect();
    let mut min: Vec<f32> = heads
        .iter()
        .map(|(h, _)| longest_word(c, h, true) + 2.0 * TH_PAD)
        .collect();
    for g in &report.satkers {
        for r in &g.rows {
            for (i, text) in daftar_cells(r).iter().enumerate() {
                let w = (c.width(text, BODY_PT, false) + 2.0 * pad).min(MAX_COL_W);
                want[i] = want[i].max(w);
                min[i] = min[i].max(longest_word(c, text, false) + 2.0 * pad);
            }
        }
    }
    // The running-number column never narrows below what mPDF gives it (30 pt
    // after scaling), whatever its content: measured, not derived.
    want[0] = want[0].max(23.0);
    min[0] = min[0].max(c.width("000", BODY_PT, false) + 2.0 * pad);
    for (m, w) in min.iter_mut().zip(&want) {
        *m = m.min(*w);
    }
    (want, min)
}

pub fn render_daftar_pdf(report: &DaftarReport) -> AppResult<Vec<u8>> {
    let mut c = Canvas::new(&report.header.judul)?;
    let mut y_start = heading(&mut c, &report.header);

    let heads: Vec<(String, Align)> = DAFTAR_FIXED
        .iter()
        .map(|(h, a)| ((*h).to_string(), *a))
        .chain(report.columns.iter().map(|h| (h.clone(), Align::Center)))
        .collect();

    // Column widths: what the content asks for, scaled so the table fills the
    // page, which is how an auto-layout HTML table settles. When the wishes do not
    // fit the page at the legacy padding, the table is measured again at a tighter
    // padding and, if even the narrowest widths do not fit, the garment columns that
    // do not fit continue on further pages.
    let (mut want, mut min) = daftar_needs(&c, &heads, report, TD_PAD);
    let mut pad = TD_PAD;
    if want.iter().sum::<f32>() > TABLE_W {
        pad = COMPACT_PAD;
        (want, min) = daftar_needs(&c, &heads, report, pad);
    }
    let parts = plan_columns(&want, &min, DAFTAR_FIXED.len(), DAFTAR_REPEAT, TABLE_W);
    let n_parts = parts.len();

    for (si, group) in report.satkers.iter().enumerate() {
        // An empty satker has nothing to continue: one "Data Kosong" table.
        let n_here = if group.rows.is_empty() { 1 } else { n_parts };
        for (pi, (cols, widths)) in parts.iter().enumerate().take(n_here) {
            let xs: Vec<f32> = widths
                .iter()
                .scan(MARGIN, |x, w| {
                    let at = *x;
                    *x += w;
                    Some(at)
                })
                .collect();
            // Head cells may wrap (a garment name in a narrow column): the row grows to its tallest.
            let head_lines: Vec<Vec<String>> = cols
                .iter()
                .enumerate()
                .map(|(k, &i)| {
                    let wrapped = c.wrap(&heads[i].0, widths[k] - 2.0 * TH_PAD, true);
                    if wrapped
                        .iter()
                        .any(|l| c.width(l, BODY_PT, true) > widths[k])
                    {
                        vec![one_line(&c, &heads[i].0, widths[k] - 2.0 * TH_PAD, true)]
                    } else {
                        wrapped
                    }
                })
                .collect();
            let head_h = head_lines
                .iter()
                .map(|l| LINE_H * l.len() as f32 + 2.0 * TH_PAD)
                .fold(TH_DAFTAR, f32::max);
            let draw_head = |c: &mut Canvas, y: f32| -> f32 {
                for (k, lines) in head_lines.iter().enumerate() {
                    c.cell(
                        Rect {
                            x: xs[k],
                            y,
                            w: widths[k],
                            h: head_h,
                        },
                        lines,
                        TH,
                    );
                }
                y + head_h
            };

            // The satker's name sits above its table; each satker (and each continuation of its
            // columns) starts a page.
            let top = if si == 0 && pi == 0 {
                y_start
            } else {
                c.new_page();
                MARGIN
            };
            // On the first page the heading follows the filter lines; on a page of
            // its own it sits where the legacy page puts it (measured).
            let (heading_baseline, table_top) = if si == 0 && pi == 0 {
                (top + H3_PT + 6.5, top + H3_PT + 6.5 + 9.3)
            } else {
                (37.0, 51.4)
            };
            let title = if n_here > 1 {
                format!("{} (kolom {}/{})", group.satker_nama, pi + 1, n_parts)
            } else {
                group.satker_nama.clone()
            };
            c.text(&title, MARGIN, heading_baseline, H3_PT, true);
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
                let wrapped: Vec<Vec<String>> = cols
                    .iter()
                    .enumerate()
                    .map(|(k, &i)| c.wrap(&cells[i], widths[k] - 2.0 * pad, false))
                    .collect();
                let lines = wrapped.iter().map(Vec::len).max().unwrap_or(1);
                let h = LINE_H * lines as f32 + 2.0 * TD_PAD;
                if y + h > bottom() {
                    c.new_page();
                    y = draw_head(&mut c, MARGIN);
                }
                for (k, lines) in wrapped.iter().enumerate() {
                    let style = Style {
                        align: heads[cols[k]].1,
                        bold: false,
                        pad,
                    };
                    c.cell(
                        Rect {
                            x: xs[k],
                            y,
                            w: widths[k],
                            h,
                        },
                        lines,
                        style,
                    );
                }
                y += h;
            }
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

    // ── content that does not fit the legacy geometry ──────────────────────

    /// A rekap block with `n` size columns and counts up to `max_count`.
    fn blok_lebar(n: usize, satkers: usize, max_count: i64) -> RekapReport {
        RekapReport {
            header: header(),
            blocks: vec![RekapBlock {
                pakaian_nama: "Celana Panjang PDL".to_string(),
                ukuran_group: "CELANA".to_string(),
                gender: "L".to_string(),
                ukuran_labels: (0..n).map(|i| (26 + i).to_string()).collect(),
                satkers: (0..satkers)
                    .map(|k| RekapSatkerRow {
                        satker_nama: format!("Satker {k}"),
                        counts: vec![max_count; n],
                        total: max_count * n as i64,
                    })
                    .collect(),
                summary: vec![max_count * satkers as i64; n],
                summary_total: max_count * (n * satkers) as i64,
            }],
        }
    }

    #[test]
    fn a_rekap_that_fits_is_not_split_a_wide_one_continues_on_another_page() {
        let c = Canvas::new("t").unwrap();
        let kecil = &rekap(1, 2).blocks[0];
        assert_eq!(
            rekap_cols_per_page(&c, kecil),
            kecil.ukuran_labels.len(),
            "the legacy table is untouched"
        );
        // 27 trouser sizes (26..52) with ordinary counts still fit one page.
        assert_eq!(
            rekap_cols_per_page(&c, &blok_lebar(27, 3, 120).blocks[0]),
            27
        );
        // 90 columns cannot: the table is cut into slices that each hold their figures.
        let lebar = blok_lebar(90, 3, 120);
        let per_page = rekap_cols_per_page(&c, &lebar.blocks[0]);
        assert!((1..90).contains(&per_page), "split into slices: {per_page}");
        // Every slice's column is wide enough for the widest figure it carries.
        let unit = TABLE_W * 0.75 / (per_page as f32 + 1.2);
        let widest = c
            .width("120", BODY_PT, false)
            .max(c.width("10800", BODY_PT, true) / 1.2);
        assert!(unit >= widest, "{unit} must hold {widest}");

        let sepotong = page_count(&render_rekap_pdf(&lebar).unwrap());
        let utuh = page_count(&render_rekap_pdf(&blok_lebar(27, 3, 120)).unwrap());
        assert_eq!(utuh, 1, "27 sizes, 3 satker: the legacy single page");
        assert!(
            sepotong >= 2,
            "90 sizes continue on further pages: {sepotong}"
        );
    }

    #[test]
    fn a_heading_that_does_not_fit_falls_back_to_its_short_form() {
        let c = Canvas::new("t").unwrap();
        assert_eq!(fit_label(&c, &["Jumlah", "Jml"], 80.0, true), "Jumlah");
        let sempit = c.width("Jml", BODY_PT, true) + 1.0;
        assert_eq!(fit_label(&c, &["Jumlah", "Jml"], sempit, true), "Jml");
        // Nothing fits: cut, never drawn past the cell.
        let cut = fit_label(&c, &["Jumlah", "Jml"], 3.0, true);
        assert!(
            c.width(&cut, BODY_PT, true) <= 3.0 + c.width("…", BODY_PT, true),
            "{cut}"
        );
    }

    #[test]
    fn share_width_fills_the_page_and_never_goes_below_a_minimum() {
        // Wishes that fit are stretched to the page (the legacy behaviour).
        let w = share_width(&[10.0, 30.0], &[5.0, 10.0], 80.0);
        assert!((w.iter().sum::<f32>() - 80.0).abs() < 0.01);
        assert!((w[1] / w[0] - 3.0).abs() < 0.01, "proportional: {w:?}");
        // Wishes that do not fit shrink toward the minimums, which stay respected.
        let w = share_width(&[100.0, 100.0, 100.0], &[20.0, 40.0, 60.0], 200.0);
        assert!((w.iter().sum::<f32>() - 200.0).abs() < 0.01, "{w:?}");
        assert!(w[0] >= 20.0 && w[1] >= 40.0 && w[2] >= 60.0, "{w:?}");
        // Exactly the minimums.
        let w = share_width(&[100.0, 100.0], &[30.0, 70.0], 100.0);
        assert!(
            (w[0] - 30.0).abs() < 0.01 && (w[1] - 70.0).abs() < 0.01,
            "{w:?}"
        );
    }

    #[test]
    fn plan_columns_keeps_a_fitting_table_whole_and_splits_one_that_cannot_fit() {
        let fixed = 8;
        // 8 fixed columns of 40 pt each plus `n` garment columns of 30 pt, on a 785 pt page.
        let plan = |garments: usize| {
            let n = fixed + garments;
            let want: Vec<f32> = (0..n)
                .map(|i| if i < fixed { 60.0 } else { 40.0 })
                .collect();
            let min: Vec<f32> = (0..n)
                .map(|i| if i < fixed { 40.0 } else { 30.0 })
                .collect();
            plan_columns(&want, &min, fixed, 3, 785.0)
        };
        let uno = plan(6);
        assert_eq!(uno.len(), 1, "six garments fit one page");
        assert_eq!(uno[0].0, (0..14).collect::<Vec<_>>());
        assert!((uno[0].1.iter().sum::<f32>() - 785.0).abs() < 0.01);

        let banyak = plan(30);
        assert!(banyak.len() >= 2, "30 garments cannot share one page");
        // The first page carries every fixed column; later pages repeat No, NIP and Nama.
        assert_eq!(&banyak[0].0[..8], &(0..8).collect::<Vec<_>>()[..]);
        for (cols, _) in &banyak[1..] {
            assert_eq!(&cols[..3], &[0, 1, 2], "identifying columns repeat");
        }
        // Each garment column is printed exactly once, and every page's columns fit its minimums.
        let mut digambar: Vec<usize> = banyak
            .iter()
            .flat_map(|(cols, _)| cols.iter().copied().filter(|&i| i >= fixed))
            .collect();
        digambar.sort_unstable();
        assert_eq!(digambar, (fixed..fixed + 30).collect::<Vec<_>>());
        for (cols, widths) in &banyak {
            let min_sum: f32 = cols
                .iter()
                .map(|&i| if i < fixed { 40.0 } else { 30.0 })
                .sum();
            assert!(min_sum <= 785.0, "a part must fit its minimums: {min_sum}");
            assert!((widths.iter().sum::<f32>() - 785.0).abs() < 0.01);
        }
    }

    fn daftar_dengan(garments: usize, nama_dada: &str) -> DaftarReport {
        DaftarReport {
            header: header(),
            columns: (0..garments).map(|i| format!("Pakaian {i}")).collect(),
            satkers: vec![DaftarSatkerGroup {
                satker_nama: "KEJAKSAAN NEGERI A".to_string(),
                rows: (0..3)
                    .map(|k| DaftarRow {
                        nip: format!("20000000000000000{k}"),
                        nama: format!("Pegawai {k}"),
                        jabatan: "Operator Satker".to_string(),
                        golongan: "III/a".to_string(),
                        status: "J".to_string(),
                        gender: "L".to_string(),
                        hijab: "T".to_string(),
                        sizes: (0..garments)
                            .map(|g| {
                                if g == 0 {
                                    nama_dada.to_string()
                                } else {
                                    "2XL".to_string()
                                }
                            })
                            .collect(),
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn a_daftar_with_many_garments_and_a_long_chest_name_stays_on_the_page() {
        // A chest name of 100 characters wraps inside its column instead of widening the table.
        let panjang = "ABCDEFGHIJ ".repeat(9) + "ABCDEFGHI";
        assert_eq!(panjang.chars().count(), 108.min(panjang.chars().count()));
        let satu = page_count(&render_daftar_pdf(&daftar_dengan(2, &panjang)).unwrap());
        assert_eq!(satu, 1, "two garments and a long text still fit one page");

        // Twelve garments do not fit one page of columns: they continue (and still print).
        let banyak = daftar_dengan(40, "AGUS");
        let bytes = render_daftar_pdf(&banyak).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(
            page_count(&bytes) >= 2,
            "40 garment columns continue on further pages"
        );
    }
}
