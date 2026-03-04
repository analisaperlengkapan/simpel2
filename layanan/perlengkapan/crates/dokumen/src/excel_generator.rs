use crate::error::AppError;
use crate::template_models::DocumentTemplate;
use chrono::Utc;
use rust_xlsxwriter::*;
use serde_json::Value;

pub struct ExcelGenerator;

impl ExcelGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate Excel file from template data
    pub async fn generate_excel(
        &self,
        template: &DocumentTemplate,
        data: &Value,
        output_path: &str,
    ) -> Result<Vec<u8>, AppError> {
        let mut workbook = Workbook::new();

        // Determine template type and generate accordingly
        match template.template_type.as_str() {
            "rekapitulasi_kebutuhan" => {
                self.generate_rekapitulasi_kebutuhan(&mut workbook, data)?;
            }
            "roadmap_sarpras" => {
                self.generate_roadmap_sarpras(&mut workbook, data)?;
            }
            "gap_analysis" => {
                self.generate_gap_analysis(&mut workbook, data)?;
            }
            _ => {
                // Generic table generation
                self.generate_generic_table(&mut workbook, data)?;
            }
        }

        // Add metadata sheet
        self.add_metadata_sheet(&mut workbook, template, data)?;

        // Save workbook
        workbook.save(output_path).map_err(|_| AppError::Internal)?;

        // Read file back as bytes
        let excel_bytes = std::fs::read(output_path).map_err(|_| AppError::Internal)?;

        Ok(excel_bytes)
    }

    /// Generate rekapitulasi kebutuhan BMN
    fn generate_rekapitulasi_kebutuhan(
        &self,
        workbook: &mut Workbook,
        data: &Value,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name("Rekapitulasi Kebutuhan")
            .map_err(|_| AppError::Internal)?;

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);

        // Title
        worksheet
            .write_string(0, 0, "REKAPITULASI KEBUTUHAN BMN")
            .map_err(|_| AppError::Internal)?;

        let tahun = data["tahun_anggaran"].as_i64().unwrap_or(2024);
        worksheet
            .write_string(1, 0, format!("Tahun Anggaran: {}", tahun))
            .map_err(|_| AppError::Internal)?;

        // Headers
        let headers = vec![
            "No",
            "Satker",
            "Kode Barang",
            "Nama Barang",
            "Jumlah Kebutuhan",
            "Jumlah Existing",
            "Gap",
            "Estimasi Anggaran",
            "Status",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_format)
                .map_err(|_| AppError::Internal)?;
        }

        // Data rows
        if let Some(items) = data["items"].as_array() {
            for (idx, item) in items.iter().enumerate() {
                let row = (idx + 4) as u32;

                worksheet
                    .write_number(row, 0, (idx + 1) as f64)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 1, item["satker_nama"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 2, item["kode_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 3, item["nama_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 4, item["jumlah_kebutuhan"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 5, item["jumlah_existing"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 6, item["gap"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;

                let anggaran = item["estimasi_anggaran"].as_f64().unwrap_or(0.0);
                let currency_format = Format::new().set_num_format("#,##0");
                worksheet
                    .write_number_with_format(row, 7, anggaran, &currency_format)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 8, item["status"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;
            }
        }

        // Auto-fit columns
        worksheet.autofit();

        Ok(())
    }

    /// Generate roadmap sarpras
    fn generate_roadmap_sarpras(
        &self,
        workbook: &mut Workbook,
        data: &Value,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name("Roadmap Sarpras")
            .map_err(|_| AppError::Internal)?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x70AD47))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);

        // Title
        worksheet
            .write_string(0, 0, "ROADMAP SARANA DAN PRASARANA")
            .map_err(|_| AppError::Internal)?;

        let periode = format!(
            "{} - {}",
            data["periode_mulai"].as_i64().unwrap_or(2024),
            data["periode_akhir"].as_i64().unwrap_or(2028)
        );
        worksheet
            .write_string(1, 0, format!("Periode: {}", periode))
            .map_err(|_| AppError::Internal)?;

        // Headers
        let headers = vec![
            "No",
            "Kode Barang",
            "Nama Barang",
            "Tahun 1",
            "Tahun 2",
            "Tahun 3",
            "Tahun 4",
            "Tahun 5",
            "Total",
            "Realisasi",
            "Sisa",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_format)
                .map_err(|_| AppError::Internal)?;
        }

        // Data rows
        if let Some(items) = data["items"].as_array() {
            for (idx, item) in items.iter().enumerate() {
                let row = (idx + 4) as u32;

                worksheet
                    .write_number(row, 0, (idx + 1) as f64)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 1, item["kode_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 2, item["nama_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                // Year columns
                for year in 0..5 {
                    let key = format!("tahun_{}", year + 1);
                    let value = item[&key].as_f64().unwrap_or(0.0);
                    worksheet
                        .write_number(row, (3 + year) as u16, value)
                        .map_err(|_| AppError::Internal)?;
                }

                // Total, Realisasi, Sisa
                worksheet
                    .write_number(row, 8, item["total"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 9, item["realisasi"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 10, item["sisa"].as_f64().unwrap_or(0.0))
                    .map_err(|_| AppError::Internal)?;
            }
        }

        worksheet.autofit();

        Ok(())
    }

    /// Generate gap analysis
    fn generate_gap_analysis(&self, workbook: &mut Workbook, data: &Value) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name("Gap Analysis")
            .map_err(|_| AppError::Internal)?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xED7D31))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);

        // Title
        worksheet
            .write_string(0, 0, "ANALISIS GAP KEBUTUHAN BMN")
            .map_err(|_| AppError::Internal)?;

        // Headers
        let headers = vec![
            "No",
            "Satker",
            "Kode Barang",
            "Nama Barang",
            "Standar Jumlah",
            "Existing (Baik)",
            "Gap",
            "% Gap",
            "Prioritas",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(2, col as u16, *header, &header_format)
                .map_err(|_| AppError::Internal)?;
        }

        // Data rows
        if let Some(items) = data["items"].as_array() {
            for (idx, item) in items.iter().enumerate() {
                let row = (idx + 3) as u32;

                worksheet
                    .write_number(row, 0, (idx + 1) as f64)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 1, item["satker_nama"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 2, item["kode_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 3, item["nama_barang"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;

                let standard = item["standard_quantity"].as_f64().unwrap_or(0.0);
                let existing = item["existing_good_quantity"].as_f64().unwrap_or(0.0);
                let gap = item["gap"].as_f64().unwrap_or(0.0);
                let gap_pct = if standard > 0.0 {
                    (gap / standard) * 100.0
                } else {
                    0.0
                };

                worksheet
                    .write_number(row, 4, standard)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 5, existing)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_number(row, 6, gap)
                    .map_err(|_| AppError::Internal)?;

                let percent_format = Format::new().set_num_format("0.00%");
                worksheet
                    .write_number_with_format(row, 7, gap_pct / 100.0, &percent_format)
                    .map_err(|_| AppError::Internal)?;

                worksheet
                    .write_string(row, 8, item["prioritas"].as_str().unwrap_or(""))
                    .map_err(|_| AppError::Internal)?;
            }
        }

        worksheet.autofit();

        Ok(())
    }

    /// Generate generic table from data
    fn generate_generic_table(
        &self,
        workbook: &mut Workbook,
        data: &Value,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Data").map_err(|_| AppError::Internal)?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        // If data is an array, create table
        if let Some(items) = data.as_array()
            && let Some(first_item) = items.first()
                && let Some(obj) = first_item.as_object() {
                    // Write headers
                    for (col, key) in obj.keys().enumerate() {
                        worksheet
                            .write_string_with_format(0, col as u16, key, &header_format)
                            .map_err(|_| AppError::Internal)?;
                    }

                    // Write data
                    for (row_idx, item) in items.iter().enumerate() {
                        if let Some(obj) = item.as_object() {
                            for (col, (_, value)) in obj.iter().enumerate() {
                                let row = (row_idx + 1) as u32;
                                match value {
                                    Value::String(s) => {
                                        worksheet
                                            .write_string(row, col as u16, s)
                                            .map_err(|_| AppError::Internal)?;
                                    }
                                    Value::Number(n) => {
                                        if let Some(f) = n.as_f64() {
                                            worksheet
                                                .write_number(row, col as u16, f)
                                                .map_err(|_| AppError::Internal)?;
                                        }
                                    }
                                    Value::Bool(b) => {
                                        worksheet
                                            .write_boolean(row, col as u16, *b)
                                            .map_err(|_| AppError::Internal)?;
                                    }
                                    _ => {
                                        worksheet
                                            .write_string(row, col as u16, value.to_string())
                                            .map_err(|_| AppError::Internal)?;
                                    }
                                }
                            }
                        }
                    }
                }

        worksheet.autofit();

        Ok(())
    }

    /// Add metadata sheet
    fn add_metadata_sheet(
        &self,
        workbook: &mut Workbook,
        template: &DocumentTemplate,
        _data: &Value,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name("Metadata")
            .map_err(|_| AppError::Internal)?;

        let bold_format = Format::new().set_bold();

        let generated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let version_str = template.version.to_string();

        let metadata = [("Template Name", template.name.as_str()),
            ("Template Type", template.template_type.as_str()),
            ("Generated At", generated_at.as_str()),
            ("Version", version_str.as_str())];

        for (row, (key, value)) in metadata.iter().enumerate() {
            worksheet
                .write_string_with_format(row as u32, 0, *key, &bold_format)
                .map_err(|_| AppError::Internal)?;
            worksheet
                .write_string(row as u32, 1, *value)
                .map_err(|_| AppError::Internal)?;
        }

        worksheet.autofit();

        Ok(())
    }
}

impl Default for ExcelGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_excel_generation() {
        let generator = ExcelGenerator::new();

        let template = DocumentTemplate {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            description: None,
            template_type: "rekapitulasi_kebutuhan".to_string(),
            content: "".to_string(),
            format: "excel".to_string(),
            output_format: "xlsx".to_string(),
            version: 1,
            is_active: true,
            parent_template_id: None,
            variables: serde_json::json!({}),
            sample_data: None,
            letterhead_config: None,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_by: None,
            updated_at: Utc::now(),
        };

        let data = serde_json::json!({
            "tahun_anggaran": 2024,
            "items": []
        });

        let result = generator
            .generate_excel(&template, &data, "/tmp/test.xlsx")
            .await;
        assert!(result.is_ok());
    }
}
