//! In-process [`DocumentGenerator`] implementation for the dokumen module.
//!
//! Wraps [`TemplateService`] + [`PdfGenerator`] + [`ExcelGenerator`] +
//! [`DocxGenerator`] so the workflow module (and any future caller) can
//! request document generation through the
//! [`lib_perlengkapan::contracts::DocumentGenerator`] trait instead of
//! the dropped internal gRPC client. Persisted artifacts go through the
//! [`DocumentStorage`] port (filesystem today via
//! [`super::FilesystemStorage`], S3 in a follow-up), so this service
//! never touches `tokio::fs` directly — same code path swaps to any
//! storage backend.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use deadpool_postgres::Pool;
use lib_perlengkapan::ServiceError;
use lib_perlengkapan::contracts::{
    DocumentArtifact, DocumentFormat, DocumentGenerator, DocumentRequest, DocumentStorage,
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
    /// Storage backend used by `generate()`. Optional so older test
    /// harnesses that constructed `DokumenService` without one still
    /// compile; when `None`, `generate()` falls back to a local
    /// [`super::FilesystemStorage`] built from env (same default behaviour
    /// the service had before the trait extraction).
    storage: Option<Arc<dyn DocumentStorage>>,
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
            storage: None,
        }
    }

    /// Inject the document storage backend. The unified `AppState` wires
    /// this to the `Arc<dyn DocumentStorage>` it constructs once at boot.
    pub fn with_storage(mut self, storage: Arc<dyn DocumentStorage>) -> Self {
        self.storage = Some(storage);
        self
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
                let tmp_path =
                    Self::storage_root().join(format!("preview-{}.xlsx", Uuid::new_v4()));
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
                let tmp_path =
                    Self::storage_root().join(format!("preview-{}.docx", Uuid::new_v4()));
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
            // TODO(csv-export): wire CSV as an alternative export format
            // alongside XLSX. Intended use is the same export endpoints that
            // currently emit `.xlsx` (bank-aset dashboard, dashboard
            // perlengkapan export, pakaian dinas laporan) — callers will be
            // able to pick `?format=csv` instead. Implementation steps:
            //   1. Add `CsvGenerator` next to `ExcelGenerator` (each export
            //      template knows its column projection; the renderer can
            //      reuse those projections rather than re-deriving rows).
            //   2. Hold `Arc<CsvGenerator>` on `DokumenService`; dispatch
            //      this branch through it analogously to the Excel branch.
            //   3. Surface a "Cetak CSV" button next to "Cetak Excel" in
            //      `pakaian_dinas_laporan.rs` + `mapping_kodefikasi_
            //      dashboard.rs`.
            // Until that lands, returning a validation error keeps the
            // trait honest — callers see a clear "not yet" rather than a
            // 5xx from the deeper pipeline.
            DocumentFormat::Csv => Err(ServiceError::validation(
                "CSV export not yet implemented — see TODO(csv-export) in dokumen::service",
            )),
        }
    }
}

#[async_trait]
impl DocumentGenerator for DokumenService {
    async fn generate(&self, request: DocumentRequest) -> Result<DocumentArtifact, ServiceError> {
        let document_id = Uuid::new_v4();
        let format = request.format;
        let bytes = self
            .render_bytes(&request.template_id, format, &request.data)
            .await?;

        let ext = Self::ext_for(format);
        let content_type = Self::content_type_for(format);
        // Storage key uses a stable, predictable layout that the matching
        // route handler can later resolve back to the file: per-document
        // folder keyed by the new UUID, plus an extension that reflects
        // the rendered format.
        let storage_key = format!("generated/{}/{}.{}", document_id, document_id, ext);

        // Delegate persistence to the storage port. Without one wired we
        // fall back to a local FilesystemStorage built from env — the
        // adapter writes to the same layout the prior inline code did.
        let handle = match self.storage.as_ref() {
            Some(storage) => {
                storage
                    .put(&storage_key, bytes.clone(), content_type)
                    .await?
            }
            None => {
                let fallback = super::FilesystemStorage::new(Self::storage_root(), None);
                fallback
                    .put(&storage_key, bytes.clone(), content_type)
                    .await?
            }
        };

        Ok(DocumentArtifact {
            document_id,
            filename: format!("{}.{}", document_id, ext),
            content_type: handle.content_type,
            size_bytes: handle.size_bytes,
            storage_key: handle.key,
            generated_at: Utc::now(),
        })
    }

    async fn preview(&self, request: DocumentRequest) -> Result<bytes::Bytes, ServiceError> {
        self.render_bytes(&request.template_id, request.format, &request.data)
            .await
    }
}
