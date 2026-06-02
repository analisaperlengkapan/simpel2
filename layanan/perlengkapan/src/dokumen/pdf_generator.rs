use super::error::AppError;
use super::template_models::DocumentTemplate;
use super::template_service::TemplateService;
use printpdf::*;
use std::fs::File;
use std::io::Write;

pub struct PdfGenerator {
    template_service: TemplateService,
}

impl PdfGenerator {
    pub fn new(template_service: TemplateService) -> Self {
        Self { template_service }
    }

    /// Generate PDF from template with data
    pub async fn generate_pdf(
        &self,
        template: &DocumentTemplate,
        data: &serde_json::Value,
        output_path: &str,
    ) -> Result<Vec<u8>, AppError> {
        // 1. Render HTML from template
        let html = self
            .template_service
            .render_template(&template.content, data)?;

        // 2. Apply letterhead if configured
        let html_with_letterhead = if let Some(letterhead_config) = &template.letterhead_config {
            self.apply_letterhead(&html, letterhead_config)?
        } else {
            html
        };

        // 3. Convert HTML to PDF
        let pdf_bytes = self.html_to_pdf(&html_with_letterhead, output_path)?;

        Ok(pdf_bytes)
    }

    /// Apply official letterhead to HTML content
    fn apply_letterhead(
        &self,
        html: &str,
        letterhead_config: &serde_json::Value,
    ) -> Result<String, AppError> {
        let logo_url = letterhead_config["logo_url"]
            .as_str()
            .unwrap_or("/assets/logo-kejaksaan.png");

        let header_text = letterhead_config["header_text"]
            .as_str()
            .unwrap_or("KEJAKSAAN REPUBLIK INDONESIA");

        let footer_text = letterhead_config["footer_text"].as_str().unwrap_or("");

        let letterhead_html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        @page {{
            size: A4;
            margin: 2cm 2cm 3cm 2cm;
        }}
        body {{
            font-family: 'Times New Roman', serif;
            font-size: 12pt;
            line-height: 1.5;
        }}
        .letterhead {{
            text-align: center;
            border-bottom: 3px solid #000;
            padding-bottom: 10px;
            margin-bottom: 20px;
        }}
        .letterhead img {{
            height: 80px;
            margin-bottom: 10px;
        }}
        .letterhead h1 {{
            font-size: 16pt;
            font-weight: bold;
            margin: 5px 0;
        }}
        .content {{
            margin: 20px 0;
        }}
        .footer {{
            position: fixed;
            bottom: 0;
            left: 0;
            right: 0;
            text-align: center;
            font-size: 10pt;
            padding: 10px;
            border-top: 1px solid #ccc;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 10px 0;
        }}
        table, th, td {{
            border: 1px solid #000;
        }}
        th, td {{
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f0f0f0;
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="letterhead">
        <img src="{}" alt="Logo Kejaksaan RI">
        <h1>{}</h1>
    </div>
    <div class="content">
        {}
    </div>
    <div class="footer">
        {}
    </div>
</body>
</html>
            "#,
            logo_url, header_text, html, footer_text
        );

        Ok(letterhead_html)
    }

    /// Convert HTML to PDF using printpdf
    fn html_to_pdf(&self, html: &str, output_path: &str) -> Result<Vec<u8>, AppError> {
        // Parse HTML and extract text (simplified)
        let text_content = self.extract_text_from_html(html);

        let mut all_page_ops: Vec<Vec<Op>> = Vec::new();
        let mut current_ops: Vec<Op> = Vec::new();
        let mut y_position = Mm(280.0); // Start from top
        let line_height = Mm(5.0);

        // Set font once at the start of each page
        let set_font_op = Op::SetFont {
            font: PdfFontHandle::Builtin(BuiltinFont::TimesRoman),
            size: Pt(12.0),
        };

        current_ops.push(set_font_op.clone());

        for line in text_content.lines() {
            // Write text
            current_ops.push(Op::StartTextSection);
            current_ops.push(Op::SetFont {
                font: PdfFontHandle::Builtin(BuiltinFont::TimesRoman),
                size: Pt(12.0),
            });
            current_ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(20.0), y_position),
            });
            current_ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.to_string())],
            });
            current_ops.push(Op::EndTextSection);

            y_position = Mm(y_position.0 - line_height.0);

            // Create new page if needed
            if y_position.0 < 20.0 {
                all_page_ops.push(current_ops);
                current_ops = Vec::new();
                current_ops.push(set_font_op.clone());
                y_position = Mm(280.0);
            }
        }

        // Push the last page
        all_page_ops.push(current_ops);

        // Build pages
        let pages: Vec<PdfPage> = all_page_ops
            .into_iter()
            .map(|ops| PdfPage::new(Mm(210.0), Mm(297.0), ops))
            .collect();

        // Create document
        let mut doc = PdfDocument::new("Document");
        doc.with_pages(pages);

        // Save to bytes
        let mut warnings = Vec::new();
        let pdf_bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);

        // Save to file
        let mut file = File::create(output_path).map_err(|_| AppError::Internal)?;
        file.write_all(&pdf_bytes).map_err(|_| AppError::Internal)?;

        Ok(pdf_bytes)
    }

    /// Extract text from HTML (simplified parser)
    fn extract_text_from_html(&self, html: &str) -> String {
        // Remove HTML tags (very basic implementation)
        // In production, use a proper HTML parser like scraper or html5ever
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        let text = re.replace_all(html, " ");

        // Clean up whitespace
        let re_whitespace = regex::Regex::new(r"\s+").unwrap();
        let cleaned = re_whitespace.replace_all(&text, " ");

        cleaned.trim().to_string()
    }

    // NOTE: Headless Chrome PDF generation not implemented.
    // Use printpdf for now.
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self::new(TemplateService::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_generation() {
        let generator = PdfGenerator::default();

        let template = DocumentTemplate {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            description: None,
            template_type: "test".to_string(),
            content: "<h1>Test Document</h1><p>This is a test.</p>".to_string(),
            format: "html".to_string(),
            output_format: "pdf".to_string(),
            version: 1,
            is_active: true,
            parent_template_id: None,
            variables: serde_json::json!({}),
            sample_data: None,
            letterhead_config: None,
            created_by: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            updated_by: None,
            updated_at: chrono::Utc::now(),
        };

        let data = serde_json::json!({});
        let result = generator
            .generate_pdf(&template, &data, "/tmp/test.pdf")
            .await;

        assert!(result.is_ok());
    }
}
