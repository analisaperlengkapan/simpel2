//! # Pakaian Dinas Export Module
//!
//! Server-side generation of PDF and XLSX reports for Pakaian Dinas.
//! Matches the output format of simpel_web-main's:
//! - cetakRekapTemplateV.blade.php → Rekap report
//! - cetakDaftarTemplateV.blade.php → Daftar report

use super::models::{LaporanDaftarPegawai, LaporanFilter, LaporanRekapUkuran};
use super::services::PakaianDinasService;
use crate::errors::*;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use uuid::Uuid;

// ============ XLSX Export ============

/// Generate XLSX for Rekap report
/// Layout matches cetakRekapTemplateV: per-spesifikasi section with L/P/Total rows
pub async fn generate_rekap_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    let data = service
        .get_laporan_rekap_ukuran(pengajuan_id, filter.clone())
        .await?;

    let mut workbook = Workbook::new();

    // Formats
    let header_format = Format::new()
        .set_bold()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_font_size(7.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let label_format = Format::new()
        .set_font_size(7.0)
        .set_bold()
        .set_border(FormatBorder::Thin);

    let total_format = Format::new()
        .set_font_size(7.0)
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    // Group by pakaian_nama
    let mut groups: Vec<String> = vec![];
    for r in &data {
        if !groups.contains(&r.pakaian_nama) {
            groups.push(r.pakaian_nama.clone());
        }
    }

    let worksheet = workbook
        .add_worksheet()
        .set_name("Rekap Ukuran")
        .map_err(|e| bad_request(&e.to_string()))?;

    // Title
    let title_format = Format::new().set_bold().set_font_size(12.0);
    worksheet
        .write_string(0, 0, "REKAP UKURAN PAKAIAN DINAS")
        .map_err(|e| bad_request(&e.to_string()))?;
    worksheet
        .set_row_format(0, &title_format)
        .map_err(|e| bad_request(&e.to_string()))?;

    let mut current_row: u32 = 2;

    for group_name in &groups {
        let items: Vec<&LaporanRekapUkuran> = data
            .iter()
            .filter(|r| &r.pakaian_nama == group_name)
            .collect();

        if items.is_empty() {
            continue;
        }

        // Group header
        let group_header_format = Format::new().set_bold().set_font_size(9.0);
        let ukuran_group = &items[0].ukuran_group;
        worksheet
            .write_string_with_format(
                current_row,
                0,
                &format!("{} ({})", group_name, ukuran_group),
                &group_header_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        current_row += 1;

        // Header row: Gender | ukuran1 | ukuran2 | ... | Total
        worksheet
            .write_string_with_format(current_row, 0, "Gender", &header_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        for (i, item) in items.iter().enumerate() {
            worksheet
                .write_string_with_format(current_row, (i + 1) as u16, &item.ukuran, &header_format)
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        worksheet
            .write_string_with_format(
                current_row,
                (items.len() + 1) as u16,
                "Total",
                &header_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        current_row += 1;

        // Row: Laki-laki
        worksheet
            .write_string_with_format(current_row, 0, "Laki-laki (L)", &label_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        let mut total_l: i64 = 0;
        for (i, item) in items.iter().enumerate() {
            total_l += item.jumlah_laki;
            worksheet
                .write_number_with_format(
                    current_row,
                    (i + 1) as u16,
                    item.jumlah_laki as f64,
                    &cell_format,
                )
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        worksheet
            .write_number_with_format(
                current_row,
                (items.len() + 1) as u16,
                total_l as f64,
                &total_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        current_row += 1;

        // Row: Perempuan
        worksheet
            .write_string_with_format(current_row, 0, "Perempuan (P)", &label_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        let mut total_p: i64 = 0;
        for (i, item) in items.iter().enumerate() {
            total_p += item.jumlah_perempuan;
            worksheet
                .write_number_with_format(
                    current_row,
                    (i + 1) as u16,
                    item.jumlah_perempuan as f64,
                    &cell_format,
                )
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        worksheet
            .write_number_with_format(
                current_row,
                (items.len() + 1) as u16,
                total_p as f64,
                &total_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        current_row += 1;

        // Row: Jumlah (Total)
        worksheet
            .write_string_with_format(current_row, 0, "Jumlah", &label_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        let mut total_all: i64 = 0;
        for (i, item) in items.iter().enumerate() {
            total_all += item.jumlah_total;
            worksheet
                .write_number_with_format(
                    current_row,
                    (i + 1) as u16,
                    item.jumlah_total as f64,
                    &total_format,
                )
                .map_err(|e| bad_request(&e.to_string()))?;
        }
        worksheet
            .write_number_with_format(
                current_row,
                (items.len() + 1) as u16,
                total_all as f64,
                &total_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        current_row += 2; // blank row between groups
    }

    // Set column widths
    worksheet
        .set_column_width(0, 20.0)
        .map_err(|e| bad_request(&e.to_string()))?;

    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| bad_request(&e.to_string()))?;
    Ok(buffer)
}

/// Generate XLSX for Daftar Pegawai report
/// Layout matches cetakDaftarTemplateV: No, NIP, Nama, Jabatan, Golongan, Status, Gender, Busana Muslimah, ukuran columns
pub async fn generate_daftar_xlsx(
    service: &PakaianDinasService,
    pengajuan_id: Uuid,
    filter: &LaporanFilter,
) -> AppResult<Vec<u8>> {
    // Fetch all pages
    let mut all_items: Vec<LaporanDaftarPegawai> = Vec::new();
    let mut page = 1;
    let per_page = 500;
    loop {
        let (items, _total) = service
            .get_laporan_daftar_pegawai(pengajuan_id, filter.clone(), page, per_page)
            .await?;
        let len = items.len();
        all_items.extend(items);
        if len < per_page as usize {
            break;
        }
        page += 1;
    }

    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_font_size(8.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_font_size(7.0)
        .set_border(FormatBorder::Thin);

    let cell_center_format = Format::new()
        .set_font_size(7.0)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let worksheet = workbook
        .add_worksheet()
        .set_name("Daftar Pegawai")
        .map_err(|e| bad_request(&e.to_string()))?;

    // Title
    let title_format = Format::new().set_bold().set_font_size(12.0);
    worksheet
        .write_string(0, 0, "DAFTAR PEGAWAI PAKAIAN DINAS")
        .map_err(|e| bad_request(&e.to_string()))?;
    worksheet
        .set_row_format(0, &title_format)
        .map_err(|e| bad_request(&e.to_string()))?;

    // Headers — matching cetakDaftarTemplateV columns
    let headers = [
        "No",
        "NIP",
        "Nama",
        "Jabatan",
        "Golongan",
        "Status",
        "Gender",
        "Busana Muslimah",
        "Baju",
        "Celana",
        "Sepatu",
    ];
    let widths = [5.0, 22.0, 25.0, 20.0, 10.0, 8.0, 8.0, 15.0, 8.0, 8.0, 8.0];

    for (i, (header, width)) in headers.iter().zip(widths.iter()).enumerate() {
        worksheet
            .write_string_with_format(2, i as u16, *header, &header_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .set_column_width(i as u16, *width)
            .map_err(|e| bad_request(&e.to_string()))?;
    }

    // Data rows
    for (idx, item) in all_items.iter().enumerate() {
        let row = (idx + 3) as u32; // start after headers

        let status = match item.jenis.as_deref() {
            Some("0") => "J",
            Some("1") => "T",
            _ => "-",
        };
        let hijab = if item.with_hijab { "Y" } else { "T" };

        worksheet
            .write_number_with_format(row, 0, (idx + 1) as f64, &cell_center_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 1, &item.nip, &cell_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 2, &item.nama, &cell_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 3, item.jabatan.as_deref().unwrap_or("-"), &cell_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(
                row,
                4,
                item.gol_kd.as_deref().unwrap_or("-"),
                &cell_center_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 5, status, &cell_center_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 6, &item.jenis_kelamin, &cell_center_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(row, 7, hijab, &cell_center_format)
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(
                row,
                8,
                item.ukuran_baju.as_deref().unwrap_or("-"),
                &cell_center_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(
                row,
                9,
                item.ukuran_celana.as_deref().unwrap_or("-"),
                &cell_center_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
        worksheet
            .write_string_with_format(
                row,
                10,
                item.ukuran_sepatu.as_deref().unwrap_or("-"),
                &cell_center_format,
            )
            .map_err(|e| bad_request(&e.to_string()))?;
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|e| bad_request(&e.to_string()))?;
    Ok(buffer)
}
