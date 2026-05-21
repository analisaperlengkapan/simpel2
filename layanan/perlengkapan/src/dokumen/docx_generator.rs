//! DOCX generator for the dokumen module.
//!
//! Renders a Handlebars template against caller-supplied data into a plain-
//! text body, then packs each line as a paragraph in a `.docx` file using
//! the `docx-rs` writer. Formatting fidelity is intentionally minimal —
//! enough to produce a real Word document that the user can open in
//! Word/LibreOffice, fill out, and sign. Rich formatting (tables, bold,
//! letterhead images) can be layered on once the konsep surat / SK
//! templates have stabilised.

use docx_rs::{Docx, Paragraph, Run};

use super::error::AppError;
use super::template_models::DocumentTemplate;
use super::template_service::TemplateService;

#[derive(Default)]
pub struct DocxGenerator {
    template_service: TemplateService,
}

impl DocxGenerator {
    pub fn new(template_service: TemplateService) -> Self {
        Self { template_service }
    }

    /// Render `template` with `data` and return the resulting `.docx` bytes.
    ///
    /// The output path is also written to disk so callers can return its
    /// URL — same convention used by [`PdfGenerator::generate_pdf`] and
    /// [`ExcelGenerator::generate_excel`].
    pub async fn generate_docx(
        &self,
        template: &DocumentTemplate,
        data: &serde_json::Value,
        output_path: &str,
    ) -> Result<Vec<u8>, AppError> {
        // 1. Render the Handlebars template to text.
        let rendered = self
            .template_service
            .render_template(&template.content, data)?;

        // 2. Strip HTML tags so the .docx body reads as plain text. The
        //    templates ship as Handlebars-flavoured HTML; for now we want
        //    a faithful textual rendering, not visual fidelity.
        let plain = strip_html(&rendered);

        // 3. Build the docx document by adding one paragraph per line.
        let mut docx = Docx::new();
        for line in plain.lines() {
            let trimmed = line.trim_end();
            // Empty paragraphs are fine — preserve vertical spacing.
            docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(trimmed)));
        }

        // 4. Pack to bytes and persist to `output_path` (so callers that
        //    return a URL pointing at the file can also serve it directly
        //    later).
        let mut buf: Vec<u8> = Vec::new();
        docx.build()
            .pack(std::io::Cursor::new(&mut buf))
            .map_err(|e| {
                tracing::error!(error = %e, "docx pack failed");
                AppError::Internal
            })?;

        if !output_path.is_empty() {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(output_path, &buf).await?;
        }

        Ok(buf)
    }
}

/// Strip simple HTML tags and decode the handful of named entities the
/// templates actually use. Good enough for the konsep surat / SK templates
/// — a full HTML-to-OOXML converter would be overkill at this stage.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_basic() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("Pak <b>Budi</b>"), "Pak Budi");
        assert_eq!(strip_html("A&nbsp;B"), "A B");
        assert_eq!(
            strip_html("<h1>Title</h1>\n<p>Line one</p>\n<p>Line two</p>"),
            "Title\nLine one\nLine two"
        );
    }
}
