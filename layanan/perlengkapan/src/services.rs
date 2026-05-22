//! # Services Layer
//!
//! Business logic for the Perlengkapan service

use crate::shared::error::*;
use crate::{models::*, repository::PerlengkapanRepository};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct PerlengkapanService {
    repo: Arc<dyn PerlengkapanRepository>,
}

impl PerlengkapanService {
    pub fn new(repo: Arc<dyn PerlengkapanRepository>) -> Self {
        Self { repo }
    }

    // ============ Dashboard Services ============

    pub async fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        self.repo.get_dashboard_stats().await
    }

    // ============ Asset Services (Read-Only) ============

    pub async fn get_all_assets(
        &self,
        page: i32,
        per_page: i32,
        category: Option<String>,
    ) -> AppResult<(Vec<Asset>, i64)> {
        self.repo.get_all_assets(page, per_page, category).await
    }

    pub async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset> {
        self.repo.get_asset_by_id(id).await
    }

    // ============ Analisis Kebutuhan Services ============

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        self.repo.get_all_analisis(page, per_page).await
    }

    pub async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan> {
        self.repo.get_analisis_by_id(id).await
    }

    pub async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;
        self.repo.create_analisis(request, user_id).await
    }

    // ============ Pemakaian Services ============

    pub async fn get_all_pemakaian(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Pemakaian>, i64)> {
        self.repo.get_all_pemakaian(page, per_page).await
    }

    pub async fn get_pemakaian_by_id(&self, id: Uuid) -> AppResult<Pemakaian> {
        self.repo.get_pemakaian_by_id(id).await
    }

    pub async fn create_pemakaian(
        &self,
        request: CreatePemakaianRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pemakaian> {
        request.validate()?;
        self.repo.create_pemakaian(request, user_id).await
    }

    // ============ Penghapusan Services ============

    pub async fn get_all_penghapusan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Penghapusan>, i64)> {
        self.repo.get_all_penghapusan(page, per_page).await
    }

    pub async fn get_penghapusan_by_id(&self, id: Uuid) -> AppResult<Penghapusan> {
        self.repo.get_penghapusan_by_id(id).await
    }

    pub async fn create_penghapusan(
        &self,
        request: CreatePenghapusanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Penghapusan> {
        request.validate()?;
        self.repo.create_penghapusan(request, user_id).await
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
