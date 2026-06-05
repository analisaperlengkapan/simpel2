//! # Export Services
//!
//! Business logic for the generic Excel export feature. Thin orchestration
//! over [`ExportRepository`]; the heavy lifting (SQL + xlsx generation) lives
//! in the repository layer.

use std::sync::Arc;
use uuid::Uuid;

use crate::export::models::{ExportJobStatusResponse, ExportQuery};
use crate::export::repository::ExportRepository;
use crate::shared::error::*;

#[derive(Clone)]
pub struct ExportService {
    repo: Arc<dyn ExportRepository>,
}

impl ExportService {
    pub fn new(repo: Arc<dyn ExportRepository>) -> Self {
        Self { repo }
    }

    /// Queue an async export job for large datasets
    pub async fn queue_export_job(&self, query: ExportQuery) -> AppResult<Uuid> {
        self.repo.queue_export_job(query).await
    }

    /// Export data to Excel synchronously (for small datasets)
    pub async fn export_to_excel_sync(&self, query: ExportQuery) -> AppResult<Vec<u8>> {
        self.repo.export_to_excel_sync(query).await
    }

    /// Get export job status
    pub async fn get_export_job_status(&self, job_id: Uuid) -> AppResult<ExportJobStatusResponse> {
        self.repo.get_export_job_status(job_id).await
    }

    /// Download completed export job
    pub async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)> {
        self.repo.download_export_job(job_id).await
    }
}
