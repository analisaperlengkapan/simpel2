//! In-process [`DocumentGenerator`] implementation for the dokumen module.
//!
//! Wraps the existing template / generator / storage primitives so that the
//! workflow module (and any future caller) can request document generation
//! through the [`lib_perlengkapan::contracts::DocumentGenerator`] trait
//! instead of the dropped internal gRPC client.

use std::sync::Arc;

use async_trait::async_trait;
use lib_perlengkapan::ServiceError;
use lib_perlengkapan::contracts::{
    DocumentArtifact, DocumentGenerator, DocumentRequest,
};

use super::template_service::TemplateService;

/// Concrete service that fulfils the [`DocumentGenerator`] contract.
///
/// The actual orchestration of [`TemplateService`], the PDF/Excel generators,
/// and the storage layer lives in their existing modules; this struct is the
/// adapter that exposes them as a single trait surface.
#[derive(Clone)]
pub struct DokumenService {
    template_service: Arc<TemplateService>,
}

impl DokumenService {
    pub fn new(template_service: Arc<TemplateService>) -> Self {
        Self { template_service }
    }

    pub fn template_service(&self) -> &Arc<TemplateService> {
        &self.template_service
    }
}

#[async_trait]
impl DocumentGenerator for DokumenService {
    async fn generate(
        &self,
        _request: DocumentRequest,
    ) -> Result<DocumentArtifact, ServiceError> {
        // TODO(perlengkapan-unified): wire to TemplateService::render_document
        // + the PDF/Excel generators + StorageService::store. Returning a
        // placeholder error so the trait can be wired into AppState while the
        // call-site migration in workflow::engine and workflow::sla is in
        // flight.
        Err(ServiceError::internal(
            "DocumentGenerator::generate not yet wired to dokumen pipeline",
        ))
    }

    async fn preview(
        &self,
        _request: DocumentRequest,
    ) -> Result<bytes::Bytes, ServiceError> {
        // TODO(perlengkapan-unified): produce the rendered bytes without
        // persisting them — used by the /admin/templates Preview button.
        Err(ServiceError::internal(
            "DocumentGenerator::preview not yet wired to dokumen pipeline",
        ))
    }
}
