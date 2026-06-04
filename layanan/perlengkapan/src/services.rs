//! # Services Layer
//!
//! Business logic for the Perlengkapan service

use crate::repository::PerlengkapanRepository;
use crate::shared::error::*;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct PerlengkapanService {
    repo: Arc<dyn PerlengkapanRepository>,
}

impl PerlengkapanService {
    pub fn new(repo: Arc<dyn PerlengkapanRepository>) -> Self {
        Self { repo }
    }

    // ============ Export Services ============

    /// Queue an async export job for large datasets
    pub async fn queue_export_job(&self, query: crate::handlers::ExportQuery) -> AppResult<Uuid> {
        self.repo.queue_export_job(query).await
    }

    /// Export data to Excel synchronously (for small datasets)
    pub async fn export_to_excel_sync(
        &self,
        query: crate::handlers::ExportQuery,
    ) -> AppResult<Vec<u8>> {
        self.repo.export_to_excel_sync(query).await
    }

    /// Get export job status
    pub async fn get_export_job_status(
        &self,
        job_id: Uuid,
    ) -> AppResult<crate::handlers::ExportJobStatusResponse> {
        self.repo.get_export_job_status(job_id).await
    }

    /// Download completed export job
    pub async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)> {
        self.repo.download_export_job(job_id).await
    }
}
