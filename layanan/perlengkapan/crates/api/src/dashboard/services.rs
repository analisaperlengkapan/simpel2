// Dashboard business logic services

use crate::dashboard::models::*;
use crate::dashboard::repository;
use crate::errors::AppError;
use deadpool_postgres::Pool;
use rust_xlsxwriter::*;

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
    pub async fn get_perlengkapan_dashboard_metrics(
        &self,
        params: &DashboardParams,
    ) -> Result<PerlengkapanDashboardMetrics, AppError> {
        // Fetch all metrics concurrently
        let kebutuhan_metrics_future = repository::fetch_kebutuhan_metrics(&self.db_pool, params);
        let gap_analysis_future = repository::fetch_gap_analysis(&self.db_pool, 10);
        let pakaian_dinas_metrics_future =
            repository::fetch_pakaian_dinas_metrics(&self.db_pool, params);
        let workflow_metrics_future = repository::fetch_workflow_metrics(&self.db_pool);
        let asset_utilization_future = repository::fetch_asset_utilization(&self.db_pool);

        // Wait for all futures to complete
        let (
            kebutuhan_metrics,
            gap_analysis,
            pakaian_dinas_metrics,
            workflow_metrics,
            asset_utilization,
        ) = tokio::try_join!(
            kebutuhan_metrics_future,
            gap_analysis_future,
            pakaian_dinas_metrics_future,
            workflow_metrics_future,
            asset_utilization_future,
        )?;

        Ok(PerlengkapanDashboardMetrics {
            kebutuhan_metrics,
            gap_analysis,
            pakaian_dinas_metrics,
            workflow_metrics,
            asset_utilization,
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
    pub async fn export_dashboard_to_pdf(
        &self,
        metrics: &PerlengkapanDashboardMetrics,
        tahun_anggaran: i32,
    ) -> Result<Vec<u8>, AppError> {
        // For PDF, we'll create a simple HTML representation and convert
        // In production, you'd want to use a proper PDF library or service

        let html = self.generate_dashboard_html(metrics, tahun_anggaran)?;

        // For now, return a simple PDF with text
        // In production, integrate with wkhtmltopdf or headless Chrome
        let pdf_bytes = self.html_to_simple_pdf(&html)?;

        Ok(pdf_bytes)
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
        let mut row = 1;
        for (status, count) in &metrics.total_by_status {
            worksheet.write_string(row, 0, status)?;
            worksheet.write_number(row, 1, *count as f64)?;
            row += 1;
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
        let headers = ["Kode Barang",
            "Nama Barang",
            "Standard Qty",
            "Existing Qty",
            "Gap"];

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
        let mut row = 1;
        for (jenis, count) in &metrics.total_by_jenis {
            worksheet.write_string(row, 0, jenis)?;
            worksheet.write_number(row, 1, *count as f64)?;
            row += 1;
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

    fn generate_dashboard_html(
        &self,
        metrics: &PerlengkapanDashboardMetrics,
        tahun_anggaran: i32,
    ) -> Result<String, AppError> {
        let total_kebutuhan: i64 = metrics.kebutuhan_metrics.total_by_status.values().sum();
        let total_pakaian: i64 = metrics.pakaian_dinas_metrics.total_by_jenis.values().sum();

        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Dashboard Perlengkapan {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4472C4; color: white; }}
        .section {{ margin: 30px 0; }}
    </style>
</head>
<body>
    <h1>Dashboard Perlengkapan</h1>
    <p>Tahun Anggaran: {}</p>
    <p>Generated: {}</p>

    <div class="section">
        <h2>Kebutuhan BMN</h2>
        <p>Total: {}</p>
    </div>

    <div class="section">
        <h2>Gap Analysis (Top 10)</h2>
        <table>
            <tr>
                <th>Kode Barang</th>
                <th>Nama Barang</th>
                <th>Gap</th>
            </tr>
            {}
        </table>
    </div>

    <div class="section">
        <h2>Pakaian Dinas</h2>
        <p>Total: {}</p>
    </div>
</body>
</html>
            "#,
            tahun_anggaran,
            tahun_anggaran,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            total_kebutuhan,
            metrics
                .gap_analysis
                .iter()
                .map(|item| format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    item.kode_barang, item.nama_barang, item.gap
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            total_pakaian,
        );

        Ok(html)
    }

    fn html_to_simple_pdf(&self, html: &str) -> Result<Vec<u8>, AppError> {
        // This is a placeholder implementation
        // In production, use wkhtmltopdf, headless Chrome, or a PDF library

        // For now, return a simple text-based PDF
        // You would integrate with printpdf or similar library here

        // Placeholder: return HTML as bytes (not a real PDF)
        // TODO: Implement proper PDF generation
        Ok(html.as_bytes().to_vec())
    }
}
