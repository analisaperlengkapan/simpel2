// Dashboard business logic services

use crate::bank_aset::AsetScope;
use crate::dashboard::models::*;
use crate::dashboard::repository;
use crate::shared::error::AppError;
use crate::shared::pdf::{PageGeometry, PdfBuilder};
use crate::shared::satker_scope::SatkerScope;
use deadpool_postgres::Pool;
use rust_xlsxwriter::*;

/// Left margin of the dashboard PDF (mm) — shared by the headings and the table
/// so section titles line up with the column grid.
const DASHBOARD_MARGIN_L: f32 = 15.0;

/// Gap-analysis column widths (mm), in header order; sums to the drawable width
/// of A4 portrait at the margins below (210 - 15 - 15 = 180).
const GAP_COL_W: [f32; 3] = [40.0, 115.0, 25.0];

/// A4 **portrait**: three columns fit comfortably, unlike the eight-column
/// kebutuhan rekap which needs landscape.
fn dashboard_geom() -> PageGeometry {
    PageGeometry {
        page_w: 210.0,
        page_h: 297.0,
        margin_l: DASHBOARD_MARGIN_L,
        margin_r: 15.0,
        margin_top: 280.0,
        margin_bottom: 15.0,
        row_h: 6.0,
        cell_font: 8.0,
        cell_pad_x: 1.5,
        cell_trunc_pad: 2.0,
        auto_paginate: true,
    }
}

/// Service to aggregate all dashboard metrics
#[derive(Clone)]
pub struct DashboardService {
    db_pool: Pool,
}

impl DashboardService {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    /// Get complete perlengkapan dashboard metrics
    ///
    /// Every one of the seven aggregates is scoped. Both scopes are needed and
    /// neither substitutes for the other: perlengkapan's workflow rows are keyed
    /// by the MySIMKARI `kode_satker` the caller's JWT carries, while SIMAN
    /// assets are keyed by the disjoint finance code `kdsatker_keu`. Threading
    /// them through the repository SIGNATURES rather than filtering afterwards
    /// is what makes the coverage checkable — adding an eighth aggregate that
    /// forgets to scope will not compile.
    pub async fn get_perlengkapan_dashboard_metrics(
        &self,
        params: &DashboardParams,
        scope: &SatkerScope,
        aset_scope: &AsetScope,
    ) -> Result<PerlengkapanDashboardMetrics, AppError> {
        // Fetch all metrics concurrently
        let kebutuhan_metrics_future =
            repository::fetch_kebutuhan_metrics(&self.db_pool, params, scope);
        let gap_analysis_future =
            repository::fetch_gap_analysis(&self.db_pool, 10, scope, aset_scope);
        let pakaian_dinas_metrics_future =
            repository::fetch_pakaian_dinas_metrics(&self.db_pool, params, scope);
        let workflow_metrics_future = repository::fetch_workflow_metrics(&self.db_pool, scope);
        let asset_utilization_future =
            repository::fetch_asset_utilization(&self.db_pool, aset_scope);
        let pemakaian_metrics_future =
            repository::fetch_pemakaian_status_metrics(&self.db_pool, scope);
        let penghapusan_metrics_future =
            repository::fetch_penghapusan_status_metrics(&self.db_pool, scope);

        // Wait for all futures to complete
        let (
            kebutuhan_metrics,
            gap_analysis,
            pakaian_dinas_metrics,
            workflow_metrics,
            asset_utilization,
            pemakaian_metrics,
            penghapusan_metrics,
        ) = tokio::try_join!(
            kebutuhan_metrics_future,
            gap_analysis_future,
            pakaian_dinas_metrics_future,
            workflow_metrics_future,
            asset_utilization_future,
            pemakaian_metrics_future,
            penghapusan_metrics_future,
        )?;

        Ok(PerlengkapanDashboardMetrics {
            kebutuhan_metrics,
            gap_analysis,
            pakaian_dinas_metrics,
            workflow_metrics,
            asset_utilization,
            pemakaian_metrics,
            penghapusan_metrics,
        })
    }

    /// Export dashboard metrics to Excel
    pub async fn export_dashboard_to_excel(
        &self,
        metrics: &PerlengkapanDashboardMetrics,
        tahun_anggaran: i32,
    ) -> Result<Vec<u8>, AppError> {
        let mut workbook = Workbook::new();

        // Sheet 1: Summary
        self.add_summary_sheet(&mut workbook, metrics, tahun_anggaran)?;

        // Sheet 2: Kebutuhan Metrics
        self.add_kebutuhan_sheet(&mut workbook, &metrics.kebutuhan_metrics)?;

        // Sheet 3: Gap Analysis
        self.add_gap_analysis_sheet(&mut workbook, &metrics.gap_analysis)?;

        // Sheet 4: Pakaian Dinas
        self.add_pakaian_dinas_sheet(&mut workbook, &metrics.pakaian_dinas_metrics)?;

        // Sheet 5: Workflow Metrics
        self.add_workflow_sheet(&mut workbook, &metrics.workflow_metrics)?;

        // Save to temporary file
        let temp_path = format!("/tmp/dashboard_export_{}.xlsx", uuid::Uuid::new_v4());
        workbook.save(&temp_path)?;

        // Read file as bytes
        let excel_bytes = std::fs::read(&temp_path)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        Ok(excel_bytes)
    }

    /// Export dashboard metrics to PDF
    /// Render the dashboard as a REAL PDF via the shared [`PdfBuilder`].
    ///
    /// This used to build an HTML string and return `html.as_bytes()` while the
    /// handler stamped `Content-Type: application/pdf` and
    /// `filename="dashboard_perlengkapan_<year>.pdf"` — so "Export PDF" handed
    /// the user a .pdf file that no PDF reader can open, with a 200 and no
    /// error anywhere. The e2e test could not see it either: it asserted only
    /// that the downloaded filename ended in `.pdf`.
    ///
    /// The placeholder's own TODO pointed at the `dokumen` template pipeline,
    /// which needs a `dashboard_perlengkapan` template seeded in the database
    /// first. That indirection is unnecessary: `crate::shared::pdf::PdfBuilder`
    /// is the printpdf-based renderer four other reports already use
    /// (kebutuhan_bmn rekap, pakaian_dinas, pemakaian_bmn SK, penghapusan), it
    /// needs no template row, and it paginates tables. Using it here makes the
    /// dashboard export consistent with every other PDF this service emits.
    pub async fn export_dashboard_to_pdf(
        &self,
        metrics: &PerlengkapanDashboardMetrics,
        tahun_anggaran: i32,
    ) -> Result<Vec<u8>, AppError> {
        let total_kebutuhan: i64 = metrics.kebutuhan_metrics.total_by_status.values().sum();
        let total_pakaian: i64 = metrics.pakaian_dinas_metrics.total_by_jenis.values().sum();

        let mut pdf = PdfBuilder::new("Dashboard Perlengkapan", dashboard_geom())?;

        pdf.write_centered("DASHBOARD PERLENGKAPAN", 14.0, true);
        pdf.advance(6.0);
        pdf.write_centered(&format!("Tahun Anggaran {}", tahun_anggaran), 10.0, false);
        pdf.advance(5.0);
        pdf.write_centered(
            &format!(
                "Dibuat: {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ),
            8.0,
            false,
        );
        pdf.advance(9.0);

        pdf.write_text("Kebutuhan BMN", 11.0, true, DASHBOARD_MARGIN_L);
        pdf.advance(5.0);
        pdf.write_text(
            &format!("Total pengajuan: {}", total_kebutuhan),
            9.0,
            false,
            DASHBOARD_MARGIN_L,
        );
        pdf.advance(9.0);

        pdf.write_text("Pakaian Dinas", 11.0, true, DASHBOARD_MARGIN_L);
        pdf.advance(5.0);
        pdf.write_text(
            &format!("Total pengajuan: {}", total_pakaian),
            9.0,
            false,
            DASHBOARD_MARGIN_L,
        );
        pdf.advance(9.0);

        pdf.write_text("Gap Analysis (Top 10)", 11.0, true, DASHBOARD_MARGIN_L);
        pdf.advance(6.0);

        let headers: Vec<String> = ["Kode Barang", "Nama Barang", "Gap"]
            .iter()
            .map(|h| h.to_string())
            .collect();
        pdf.row(&GAP_COL_W, &headers, true, true);

        if metrics.gap_analysis.is_empty() {
            // An empty section must SAY it is empty. A table that just stops
            // is indistinguishable from a table that failed to render.
            pdf.row(
                &GAP_COL_W,
                &[
                    "-".to_string(),
                    "Tidak ada data gap analysis".to_string(),
                    "-".to_string(),
                ],
                false,
                true,
            );
        } else {
            for item in &metrics.gap_analysis {
                pdf.row(
                    &GAP_COL_W,
                    &[
                        item.kode_barang.to_string(),
                        item.nama_barang.to_string(),
                        item.gap.to_string(),
                    ],
                    false,
                    true,
                );
            }
        }

        pdf.finish()
    }

    // Helper methods for Excel export

    fn add_summary_sheet(
        &self,
        workbook: &mut Workbook,
        metrics: &PerlengkapanDashboardMetrics,
        tahun_anggaran: i32,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Summary")?;

        let title_format = Format::new().set_bold().set_font_size(16.0);

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        // Title
        worksheet.write_string_with_format(0, 0, "DASHBOARD PERLENGKAPAN", &title_format)?;

        worksheet.write_string(1, 0, format!("Tahun Anggaran: {}", tahun_anggaran))?;

        worksheet.write_string(
            2,
            0,
            format!(
                "Generated: {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            ),
        )?;

        // Summary metrics
        let mut row = 4;

        worksheet.write_string_with_format(row, 0, "Metric", &header_format)?;
        worksheet.write_string_with_format(row, 1, "Value", &header_format)?;

        row += 1;

        // Kebutuhan metrics
        let total_kebutuhan: i64 = metrics.kebutuhan_metrics.total_by_status.values().sum();
        worksheet.write_string(row, 0, "Total Kebutuhan")?;
        worksheet.write_number(row, 1, total_kebutuhan as f64)?;
        row += 1;

        worksheet.write_string(row, 0, "Kebutuhan Approved")?;
        worksheet.write_number(
            row,
            1,
            *metrics
                .kebutuhan_metrics
                .total_by_status
                .get("APPROVED")
                .unwrap_or(&0) as f64,
        )?;
        row += 1;

        worksheet.write_string(row, 0, "Total Gap Analysis Items")?;
        worksheet.write_number(row, 1, metrics.gap_analysis.len() as f64)?;
        row += 1;

        let total_pakaian: i64 = metrics.pakaian_dinas_metrics.total_by_jenis.values().sum();
        worksheet.write_string(row, 0, "Total Pakaian Dinas")?;
        worksheet.write_number(row, 1, total_pakaian as f64)?;

        worksheet.autofit();

        Ok(())
    }

    fn add_kebutuhan_sheet(
        &self,
        workbook: &mut Workbook,
        metrics: &KebutuhanMetrics,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Kebutuhan")?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        // Headers
        worksheet.write_string_with_format(0, 0, "Status", &header_format)?;
        worksheet.write_string_with_format(0, 1, "Count", &header_format)?;

        // Data
        for (i, (status, count)) in metrics.total_by_status.iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write_string(row, 0, status)?;
            worksheet.write_number(row, 1, *count as f64)?;
        }

        worksheet.autofit();

        Ok(())
    }

    fn add_gap_analysis_sheet(
        &self,
        workbook: &mut Workbook,
        gap_items: &[GapAnalysisResult],
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Gap Analysis")?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xED7D31))
            .set_font_color(Color::White);

        // Headers
        let headers = [
            "Kode Barang",
            "Nama Barang",
            "Standard Qty",
            "Existing Qty",
            "Gap",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)?;
        }

        // Data
        for (row_idx, item) in gap_items.iter().enumerate() {
            let row = (row_idx + 1) as u32;

            worksheet.write_string(row, 0, &item.kode_barang)?;
            worksheet.write_string(row, 1, &item.nama_barang)?;
            worksheet.write_number(row, 2, item.standard_quantity as f64)?;
            worksheet.write_number(row, 3, item.existing_good_quantity as f64)?;
            worksheet.write_number(row, 4, item.gap as f64)?;
        }

        worksheet.autofit();

        Ok(())
    }

    fn add_pakaian_dinas_sheet(
        &self,
        workbook: &mut Workbook,
        metrics: &PakaianDinasMetrics,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Pakaian Dinas")?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x70AD47))
            .set_font_color(Color::White);

        // Headers
        worksheet.write_string_with_format(0, 0, "Jenis", &header_format)?;
        worksheet.write_string_with_format(0, 1, "Count", &header_format)?;

        // Data
        for (i, (jenis, count)) in metrics.total_by_jenis.iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write_string(row, 0, jenis)?;
            worksheet.write_number(row, 1, *count as f64)?;
        }

        worksheet.autofit();

        Ok(())
    }

    fn add_workflow_sheet(
        &self,
        workbook: &mut Workbook,
        metrics: &WorkflowMetrics,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Workflow")?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x5B9BD5))
            .set_font_color(Color::White);

        // Metrics
        let mut row = 0;

        worksheet.write_string_with_format(row, 0, "Metric", &header_format)?;
        worksheet.write_string_with_format(row, 1, "Value", &header_format)?;
        row += 1;

        worksheet.write_string(row, 0, "Avg Processing Time (hours)")?;
        worksheet.write_number(row, 1, metrics.average_processing_time_hours)?;
        row += 1;

        worksheet.write_string(row, 0, "SLA Breaches Today")?;
        worksheet.write_number(row, 1, metrics.sla_breaches_today as f64)?;

        worksheet.autofit();

        Ok(())
    }

    // Helper methods for PDF export
}
