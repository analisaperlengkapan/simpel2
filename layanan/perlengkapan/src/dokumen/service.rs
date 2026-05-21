//! In-process [`DocumentGenerator`] implementation for the dokumen module.
//!
//! Wraps [`TemplateService`] + [`PdfGenerator`] + [`ExcelGenerator`] so the
//! workflow module (and any future caller) can request document generation
//! through the [`lib_perlengkapan::contracts::DocumentGenerator`] trait
//! instead of the dropped internal gRPC client.
//!
//! Storage strategy: files land under `${DOCUMENT_STORAGE_PATH:-/tmp/perlengkapan/docs}/<uuid>.<ext>`
//! and the artifact's `storage_key` carries that relative path. The proper
//! [`DocumentStorage`](lib_perlengkapan::contracts::DocumentStorage)
//! adapter (filesystem now, S3 later) lands in a follow-up commit;
//! `storage_key` is forward-compatible with that swap.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use deadpool_postgres::Pool;
use lib_perlengkapan::ServiceError;
use lib_perlengkapan::contracts::{
    DocumentArtifact, DocumentFormat, DocumentGenerator, DocumentRequest,
};
use uuid::Uuid;

use super::docx_generator::DocxGenerator;
use super::excel_generator::ExcelGenerator;
use super::pdf_generator::PdfGenerator;
use super::template_service::TemplateService;

const DEFAULT_STORAGE_ROOT: &str = "/tmp/perlengkapan/docs";

/// Concrete service that fulfils the [`DocumentGenerator`] contract.
#[derive(Clone)]
pub struct DokumenService {
    pool: Pool,
    template_service: Arc<TemplateService>,
    pdf_generator: Arc<PdfGenerator>,
    excel_generator: Arc<ExcelGenerator>,
    docx_generator: Arc<DocxGenerator>,
}

impl DokumenService {
    pub fn new(
        pool: Pool,
        template_service: Arc<TemplateService>,
        pdf_generator: Arc<PdfGenerator>,
        excel_generator: Arc<ExcelGenerator>,
        docx_generator: Arc<DocxGenerator>,
    ) -> Self {
        Self {
            pool,
            template_service,
            pdf_generator,
            excel_generator,
            docx_generator,
        }
    }

    fn storage_root() -> PathBuf {
        std::env::var("DOCUMENT_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_STORAGE_ROOT))
    }

    fn ext_for(format: DocumentFormat) -> &'static str {
        match format {
            DocumentFormat::Pdf => "pdf",
            DocumentFormat::Excel => "xlsx",
            DocumentFormat::Docx => "docx",
            DocumentFormat::Html => "html",
            DocumentFormat::Csv => "csv",
        }
    }

    fn content_type_for(format: DocumentFormat) -> &'static str {
        match format {
            DocumentFormat::Pdf => "application/pdf",
            DocumentFormat::Excel => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            DocumentFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            DocumentFormat::Html => "text/html; charset=utf-8",
            DocumentFormat::Csv => "text/csv; charset=utf-8",
        }
    }

    /// Render `template_id` against `data` to raw bytes, without persisting.
    async fn render_bytes(
        &self,
        template_id: &str,
        format: DocumentFormat,
        data: &serde_json::Value,
    ) -> Result<bytes::Bytes, ServiceError> {
        let template_uuid = Uuid::parse_str(template_id).map_err(|e| {
            ServiceError::validation(format!("Invalid template_id (expected UUID): {}", e))
        })?;

        let template = self
            .template_service
            .get_template(&self.pool, template_uuid)
            .await
            .map_err(|e| ServiceError::storage(format!("Template fetch failed: {}", e)))?;

        match format {
            DocumentFormat::Html => {
                let html = self
                    .template_service
                    .render_template(&template.content, data)
                    .map_err(|e| {
                        ServiceError::validation(format!("Template render failed: {}", e))
                    })?;
                Ok(bytes::Bytes::from(html.into_bytes()))
            }
            DocumentFormat::Pdf => {
                // pdf_generator writes a temp file on disk as a side effect; use
                // a discardable path under the storage root.
                let tmp_path = Self::storage_root().join(format!("preview-{}.pdf", Uuid::new_v4()));
                tokio::fs::create_dir_all(tmp_path.parent().unwrap_or(&Self::storage_root()))
                    .await
                    .map_err(|e| ServiceError::storage(format!("create_dir_all: {}", e)))?;
                let bytes = self
                    .pdf_generator
                    .generate_pdf(&template, data, tmp_path.to_str().unwrap_or(""))
                    .await
                    .map_err(|e| ServiceError::storage(format!("PDF render failed: {}", e)))?;
                let _ = tokio::fs::remove_file(&tmp_path).await;
                Ok(bytes::Bytes::from(bytes))
            }
            DocumentFormat::Excel => {
                let tmp_path = Self::storage_root().join(format!("preview-{}.xlsx", Uuid::new_v4()));
                tokio::fs::create_dir_all(tmp_path.parent().unwrap_or(&Self::storage_root()))
                    .await
                    .map_err(|e| ServiceError::storage(format!("create_dir_all: {}", e)))?;
                let bytes = self
                    .excel_generator
                    .generate_excel(&template, data, tmp_path.to_str().unwrap_or(""))
                    .await
                    .map_err(|e| ServiceError::storage(format!("Excel render failed: {}", e)))?;
                let _ = tokio::fs::remove_file(&tmp_path).await;
                Ok(bytes::Bytes::from(bytes))
            }
            DocumentFormat::Docx => {
                let tmp_path = Self::storage_root().join(format!("preview-{}.docx", Uuid::new_v4()));
                tokio::fs::create_dir_all(tmp_path.parent().unwrap_or(&Self::storage_root()))
                    .await
                    .map_err(|e| ServiceError::storage(format!("create_dir_all: {}", e)))?;
                let bytes = self
                    .docx_generator
                    .generate_docx(&template, data, tmp_path.to_str().unwrap_or(""))
                    .await
                    .map_err(|e| ServiceError::storage(format!("DOCX render failed: {}", e)))?;
                let _ = tokio::fs::remove_file(&tmp_path).await;
                Ok(bytes::Bytes::from(bytes))
            }
            DocumentFormat::Csv => Err(ServiceError::validation(
                "CSV rendering not yet implemented for the trait path",
            )),
        }
    }
}

#[async_trait]
impl DocumentGenerator for DokumenService {
    async fn generate(
        &self,
        request: DocumentRequest,
    ) -> Result<DocumentArtifact, ServiceError> {
        let document_id = Uuid::new_v4();
        let bytes = self
            .render_bytes(&request.template_id, request.format, &request.data)
            .await?;

        let ext = Self::ext_for(request.format);
        let storage_root = Self::storage_root();
        tokio::fs::create_dir_all(&storage_root)
            .await
            .map_err(|e| ServiceError::storage(format!("create_dir_all: {}", e)))?;

        let filename = format!("{}.{}", document_id, ext);
        let storage_path = storage_root.join(&filename);
        let storage_key = storage_path.to_string_lossy().into_owned();
        tokio::fs::write(&storage_path, &bytes)
            .await
            .map_err(|e| ServiceError::storage(format!("write {:?}: {}", storage_path, e)))?;

        Ok(DocumentArtifact {
            document_id,
            filename,
            content_type: Self::content_type_for(request.format).to_string(),
            size_bytes: bytes.len() as u64,
            storage_key,
            generated_at: Utc::now(),
        })
    }

    async fn preview(
        &self,
        request: DocumentRequest,
    ) -> Result<bytes::Bytes, ServiceError> {
        self.render_bytes(&request.template_id, request.format, &request.data)
            .await
    }
}
