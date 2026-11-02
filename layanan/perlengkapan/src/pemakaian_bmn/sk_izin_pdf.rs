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

use uuid::Uuid;

use super::models::IzinPemakaianBmn;
use super::services::PemakaianBmnService;
use crate::shared::error::AppResult;
use crate::shared::grpc::clients::IntegrasiClient;
use crate::shared::pdf::{PageGeometry, PdfBuilder};
use crate::shared::satker_scope::SatkerScope;

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

fn geom() -> PageGeometry {
    PageGeometry {
        page_w: PAGE_W,
        page_h: PAGE_H,
        margin_l: MARGIN_L,
        margin_r: MARGIN_R,
        margin_top: MARGIN_TOP,
        margin_bottom: 18.0,
        row_h: ROW_H,
        cell_font: CELL_FONT,
        cell_pad_x: 1.5,
        cell_trunc_pad: 3.0,
        auto_paginate: false,
    }
}

// ─── Public API ─────────────────────────────────────────────────────────

pub async fn generate_sk_izin_pdf(
    service: &PemakaianBmnService,
    permit_id: Uuid,
    scope: &SatkerScope,
    integrasi: Option<&IntegrasiClient>,
) -> AppResult<Vec<u8>> {
    let detail = service.get_permit_detail(permit_id, scope).await?;
    let foto = fetch_foto(integrasi, &detail.izin.pegawai_nip).await;
    render_pdf(&detail.izin, foto.as_deref())
}

/// The employee's photo bytes, or `None` when there is no photo to draw.
///
/// Resolved from the NIP through integrasi rather than read off
/// `permit.foto_pegawai`. That column holds a file NAME the frontend sent at
/// creation time, and a name is not a snapshot: it points at a mutable remote
/// object, so it is neither current data nor preserved evidence. The
/// authoritative answer for "what does this NIP look like" lives in
/// `integrasi.mysimkari_pegawai`, and that is where this asks.
///
/// Every failure is `None`. A missing photo must not make the SK undownloadable
/// — the placeholder box is the documented fallback — but it IS logged, because
/// "no photo" and "the media host is down" look identical on the page and only
/// the log can tell an operator which one they are looking at.
async fn fetch_foto(integrasi: Option<&IntegrasiClient>, nip: &str) -> Option<Vec<u8>> {
    let client = integrasi.or_else(|| {
        tracing::warn!("SK izin untuk NIP {nip}: klien integrasi tidak ter-wire, foto dilewati");
        None
    })?;
    match client.get_pegawai_foto(nip).await {
        Ok(Some((bytes, content_type))) => {
            tracing::debug!(
                "SK izin NIP {nip}: foto {} byte {content_type}",
                bytes.len()
            );
            Some(bytes)
        }
        Ok(None) => {
            tracing::info!("SK izin NIP {nip}: tidak ada foto tercatat");
            None
        }
        Err(e) => {
            tracing::warn!("SK izin NIP {nip}: pengambilan foto gagal: {e}");
            None
        }
    }
}

fn render_pdf(permit: &IzinPemakaianBmn, foto: Option<&[u8]>) -> AppResult<Vec<u8>> {
    let mut pdf = PdfBuilder::new("SK Izin Pemakaian BMN", geom())?;

    // ─── Halaman 1: Informasi Pegawai ────────────────────────────────
    render_halaman_pegawai(&mut pdf, permit, foto);

    // ─── Halaman 2: Daftar BMN ───────────────────────────────────────
    pdf.new_page();
    render_halaman_daftar_bmn(&mut pdf, permit);

    pdf.finish()
}

fn render_halaman_pegawai(pdf: &mut PdfBuilder, permit: &IzinPemakaianBmn, foto: Option<&[u8]>) {
    // Title bar
    pdf.write_centered("SURAT KEPUTUSAN IZIN PEMAKAIAN BMN", TITLE_FONT, true);
    pdf.advance(7.0);
    pdf.write_centered(
        &format!(
            "Nomor: {}",
            permit
                .nomor_izin
                .as_deref()
                .unwrap_or("(belum diterbitkan)")
        ),
        SECTION_FONT,
        false,
    );
    pdf.advance(12.0);

    pdf.write_text(
        "HALAMAN 1 — INFORMASI PEGAWAI",
        SECTION_FONT,
        true,
        MARGIN_L,
    );
    pdf.advance(10.0);

    // Foto pegawai — WAJIB per stakeholder, dan sampai sekarang tidak pernah
    // ada: yang tergambar hanyalah kotak kosong berlabel "FOTO" dengan URL
    // sumbernya dicetak sebagai teks di bawahnya. URL internal pada surat
    // keputusan resmi bukan hanya tak berguna bagi pembacanya, ia bocoran.
    //
    // Ukuran 35x45 mm = pasfoto 4x6 cm dipotong ke rasio yang lazim pada
    // dokumen kepegawaian; `PdfBuilder::image` menjaga rasio aslinya dan
    // memusatkan sisanya, jadi foto berbentuk lain tidak melar.
    const FOTO_W: f32 = 35.0;
    const FOTO_H: f32 = 45.0;
    let foto_x = MARGIN_L;
    let prev_y = pdf.y;
    pdf.y = prev_y - FOTO_H * 0.5; // center vertically on rect

    // Bingkainya digambar dalam kedua keadaan: dengan foto ia jadi garis
    // pinggir pasfoto, tanpa foto ia jadi kotak kosong yang sudah dikenal.
    pdf.rect(foto_x, FOTO_W, FOTO_H);
    let tergambar = match foto {
        Some(bytes) => match pdf.image(bytes, foto_x, FOTO_W, FOTO_H) {
            Ok(()) => true,
            Err(e) => {
                // Sampai di sini berarti byte-nya tiba tapi tak bisa didekode.
                // SK tetap terbit dengan kotak kosong; yang tak boleh adalah
                // gagal senyap, karena di halaman keduanya terlihat sama.
                tracing::warn!("SK izin: foto tiba tapi gagal didekode: {e}");
                false
            }
        },
        None => false,
    };
    pdf.y = prev_y;
    if !tergambar {
        pdf.write_text("FOTO", CELL_FONT, true, foto_x + FOTO_W / 2.0 - 4.0);
    }
    pdf.advance(FOTO_H + 3.0);
    if !tergambar {
        pdf.write_text("(foto pegawai tidak tersedia)", CELL_FONT, false, foto_x);
    }
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
        // A permit created through the API always carries a keperluan (the
        // create request requires it); the dash is for legacy/seed rows, so the
        // SK prints a blank line rather than the word "None".
        &format!("Keperluan: {}", permit.keperluan.as_deref().unwrap_or("-")),
        BODY_FONT,
        false,
        MARGIN_L,
    );
}

/// Height, in mm, the closing sentence and the generated-at line need below
/// the last table row (8 + 12 of cursor advance, plus the text itself).
const CLOSING_BLOCK_H: f32 = 30.0;

/// Start a continuation page when fewer than `needed` mm remain above the
/// bottom margin. Returns whether a page was started, so the caller knows to
/// repeat whatever heads a page.
fn ensure_room(pdf: &mut PdfBuilder, needed: f32) -> bool {
    if pdf.y - needed < pdf.geom().margin_bottom {
        pdf.new_page();
        true
    } else {
        false
    }
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
    pdf.row(&col_widths, &header_cells, true, true);

    // One row per BMN. This was a single hard-coded row under a comment that
    // said "V1 model: single BMN per permit" — so a permit over three assets
    // produced a letter granting one, and the other two were held with nothing
    // signed for them. The dates come from the item, which is where a period
    // lives since V014; the header's are only the MIN/MAX of these.
    if permit.bmn_items.is_empty() {
        // Says so, rather than printing an empty table that reads like a
        // rendering fault. A permit with no asset cannot be created through
        // the API; a legacy row can still be one.
        pdf.row(
            &col_widths,
            &[
                "-",
                "-",
                "(tidak ada BMN tercatat)",
                "-",
                "-",
                "-",
                "-",
                "-",
            ]
            .map(String::from),
            false,
            true,
        );
    }
    // The builder is `auto_paginate: false` — page 1 is a fixed layout — so
    // `pdf.row` never starts a page of its own, and a table whose length
    // depends on the data has to. Without this the rows past the bottom margin
    // were still "drawn", at a negative y, off the sheet: the signed letter
    // listed the first ~40 assets and silently omitted the rest, which is the
    // single-asset bug in a new place. Each continuation page repeats the
    // column header so a page torn off the stack still reads on its own.
    let row_h = pdf.geom().row_h;
    let mut halaman = 2;
    for item in &permit.bmn_items {
        if ensure_room(pdf, row_h) {
            halaman += 1;
            pdf.write_centered(
                &format!("HALAMAN {halaman} — DAFTAR BMN (lanjutan)"),
                SECTION_FONT,
                true,
            );
            pdf.advance(10.0);
            pdf.row(&col_widths, &header_cells, true, true);
        }
        let cells: Vec<String> = vec![
            item.urutan.to_string(),
            item.bmn_kode_barang.clone(),
            item.bmn_nama_barang.clone(),
            item.bmn_nup.clone(),
            item.bmn_merk.clone().unwrap_or_else(|| "-".to_string()),
            item.bmn_tipe.clone().unwrap_or_else(|| "-".to_string()),
            item.tanggal_mulai.format("%d-%m-%Y").to_string(),
            item.tanggal_selesai.format("%d-%m-%Y").to_string(),
        ];
        pdf.row(&col_widths, &cells, false, true);
    }

    // The closing sentence and the generated-at line take about 30 mm below
    // the last row. A table that ends flush with the margin must not push
    // them off the sheet either.
    ensure_room(pdf, CLOSING_BLOCK_H);

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
        let mut pdf = PdfBuilder::new("Test", geom()).expect("init");
        pdf.write_centered("TEST", 12.0, true);
        pdf.advance(8.0);
        pdf.row(
            &[20.0, 30.0, 30.0],
            &["A".into(), "B".into(), "C".into()],
            false,
            true,
        );
        pdf.new_page();
        pdf.write_centered("Page 2", 12.0, false);
        let bytes = pdf.finish().expect("finish");
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..4], b"%PDF");
    }

    /// Built through `Deserialize` rather than a struct literal: the entity has
    /// 60-odd fields and serde fills every `Option` with `None`, so this names
    /// only what page 1 actually renders and does not have to be edited every
    /// time an unrelated column is added.
    fn permit() -> IzinPemakaianBmn {
        serde_json::from_value(serde_json::json!({
            "id": "11111111-1111-4111-8111-111111111111",
            "pegawai_nip": "200000000000000001",
            "pegawai_nama": "Budi Santoso",
            "pegawai_satker_id": "22222222-2222-4222-8222-222222222222",
            "pegawai_satker_nama": "KEJAKSAAN NEGERI JAKARTA PUSAT",
            "pegawai_jabatan": "Kasubag Perlengkapan",
            "pegawai_pangkat": "Penata Muda",
            "tanggal_mulai": "2026-01-01",
            "tanggal_selesai": "2026-12-31",
            "is_renewal": false,
            "is_completed": false,
            "status": "approved",
            "version": 1,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("fixture permit")
    }

    /// A 2x3 PNG built in-process, so the test needs no fixture file.
    fn png() -> Vec<u8> {
        crate::shared::pdf::tests_support::png_2x3()
    }

    /// The whole point of this change: the photo must reach the bytes.
    ///
    /// Asserted by SIZE DIFFERENCE against the same SK rendered without one,
    /// not by "the call returned Ok". A `render_pdf` that quietly dropped the
    /// image would still return `Ok` and still produce a valid PDF — that is
    /// precisely the failure mode being fixed, so the assertion has to be able
    /// to see it.
    #[test]
    fn foto_masuk_ke_dalam_pdf() {
        let permit = permit();
        let tanpa = render_pdf(&permit, None).expect("tanpa foto");
        let dengan = render_pdf(&permit, Some(&png())).expect("dengan foto");
        assert!(
            dengan.len() > tanpa.len(),
            "PDF berfoto ({} byte) tidak lebih besar dari yang tanpa ({} byte) — \
             gambarnya tidak tertanam",
            dengan.len(),
            tanpa.len()
        );
    }

    /// The other direction. Without it, a `render_pdf` that always drew a
    /// placeholder would pass nothing above and everything here.
    #[test]
    fn tanpa_foto_tetap_terbit_dengan_kotak_kosong() {
        let bytes = render_pdf(&permit(), None).expect("SK tanpa foto tetap terbit");
        assert!(bytes.starts_with(b"%PDF"), "keluaran bukan PDF");
    }

    /// Bytes that are not an image must not fail the download. An SK that
    /// cannot be issued because a media host served an error page is worse
    /// than one with an empty photo box.
    #[test]
    fn byte_rusak_tidak_menggagalkan_sk() {
        let bytes = render_pdf(&permit(), Some(b"<html>error</html>"))
            .expect("byte rusak harus jatuh ke kotak kosong, bukan menggagalkan SK");
        assert!(bytes.starts_with(b"%PDF"));
    }

    /// The URL used to be printed on the document as body text. It is an
    /// internal address on a formal decision letter, and it is gone.
    #[test]
    fn url_sumber_foto_tidak_lagi_dicetak_di_surat() {
        let mut permit = permit();
        permit.foto_pegawai = Some("Ywv1QHmg RAHASIA.jpg".to_string());
        let bytes = render_pdf(&permit, None).expect("render");
        let teks = String::from_utf8_lossy(&bytes);
        assert!(
            !teks.contains("Sumber foto"),
            "label URL sumber masih tercetak di SK"
        );
        assert!(
            !teks.contains("RAHASIA"),
            "nama berkas masih bocor ke badan surat"
        );
    }

    /// One asset row. Built through `Deserialize` for the same reason as
    /// `permit()`: the entity is wide and only the columns page 2 prints
    /// matter here.
    fn item(urutan: i32) -> crate::pemakaian_bmn::models::PemakaianBmnItem {
        serde_json::from_value(serde_json::json!({
            "id": uuid::Uuid::new_v4(),
            "izin_pemakaian_id": "11111111-1111-4111-8111-111111111111",
            "urutan": urutan,
            "jenis_bmn": "LAPTOP",
            "bmn_nup": urutan.to_string(),
            "bmn_kode_barang": "3050102001",
            "bmn_nama_barang": format!("Laptop-{urutan:03}"),
            "tanggal_mulai": "2026-01-01",
            "tanggal_selesai": "2026-12-31",
            "detail_bmn": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("fixture item")
    }

    /// Page 2 as `render_pdf` starts it: a fresh page, then the table.
    fn render_daftar(n: i32) -> PdfBuilder {
        let mut permit = permit();
        permit.bmn_items = (1..=n).map(item).collect();
        let mut pdf = PdfBuilder::new("SK", geom()).expect("init");
        pdf.new_page();
        render_halaman_daftar_bmn(&mut pdf, &permit);
        pdf
    }

    /// The defect: `auto_paginate: false` meant row 41 onwards was drawn at a
    /// negative y — off the sheet — so a permit over 100 assets produced a
    /// signed letter that listed forty of them. Asserted on what was PLACED,
    /// not on "the call returned Ok": a renderer that dropped the overflow
    /// would also return Ok and also produce a valid PDF.
    #[test]
    fn daftar_bmn_yang_panjang_berlanjut_ke_halaman_berikutnya() {
        let pdf = render_daftar(100);
        let runs = pdf.text_runs();
        let bawah = pdf.geom().margin_bottom;

        for n in 1..=100 {
            let nama = format!("Laptop-{n:03}");
            assert!(
                runs.iter().any(|(_, _, t)| t == &nama),
                "{nama} tidak tercetak di SK — daftar terpotong"
            );
        }

        let terendah = runs.iter().map(|r| r.1).fold(f32::INFINITY, f32::min);
        assert!(
            terendah >= bawah,
            "ada teks di y={terendah} mm, di bawah margin bawah {bawah} mm — \
             tercetak di luar lembar"
        );

        let halaman = runs.iter().map(|r| r.0).max().expect("ada teks") + 1;
        assert!(
            halaman >= 4,
            "100 BMN muat di {halaman} halaman — tidak mungkin; daftar tidak berlanjut"
        );
    }

    /// Continuation pages repeat the column header, so a page torn off the
    /// stack still says what its columns are.
    #[test]
    fn halaman_lanjutan_mengulang_kepala_kolom() {
        let runs = render_daftar(100).text_runs();
        let halaman_dengan_kepala: std::collections::BTreeSet<usize> = runs
            .iter()
            .filter(|(_, _, t)| t == "Selesai")
            .map(|r| r.0)
            .collect();
        let halaman_dengan_baris: std::collections::BTreeSet<usize> = runs
            .iter()
            .filter(|(_, _, t)| t.starts_with("Laptop-"))
            .map(|r| r.0)
            .collect();
        assert_eq!(
            halaman_dengan_kepala, halaman_dengan_baris,
            "setiap halaman yang berisi baris BMN harus berkepala kolom"
        );
    }

    /// The other direction: a short list must NOT grow extra pages, or the
    /// fix is "always paginate" rather than "paginate when needed".
    #[test]
    fn daftar_bmn_pendek_tetap_di_halaman_dua() {
        let runs = render_daftar(3).text_runs();
        let halaman = runs.iter().map(|r| r.0).max().expect("ada teks") + 1;
        assert_eq!(halaman, 2, "3 BMN seharusnya muat di halaman 2");
    }
}
