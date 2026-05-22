use async_trait::async_trait;
use uuid::Uuid;

use crate::{models::*};
use crate::shared::error::AppResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PerlengkapanRepository: Send + Sync {
    // Dashboard
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;

    // Asset (Read-Only from Integrasi/SIMAN)
    async fn get_all_assets(
        &self,
        page: i32,
        per_page: i32,
        category: Option<String>,
    ) -> AppResult<(Vec<Asset>, i64)>;
    async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset>;

    // Analisis (Local)
    async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
    async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan>;
    async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan>;

    // Pemakaian (Local)
    async fn get_all_pemakaian(&self, page: i32, per_page: i32)
    -> AppResult<(Vec<Pemakaian>, i64)>;
    async fn get_pemakaian_by_id(&self, id: Uuid) -> AppResult<Pemakaian>;
    async fn create_pemakaian(
        &self,
        request: CreatePemakaianRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Pemakaian>;

    // Penghapusan (Local)
    async fn get_all_penghapusan(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<Penghapusan>, i64)>;
    async fn get_penghapusan_by_id(&self, id: Uuid) -> AppResult<Penghapusan>;
    async fn create_penghapusan(
        &self,
        request: CreatePenghapusanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<Penghapusan>;

    // Export (Local)
    async fn queue_export_job(&self, query: crate::handlers::ExportQuery) -> AppResult<Uuid>;
    async fn export_to_excel_sync(&self, query: crate::handlers::ExportQuery)
    -> AppResult<Vec<u8>>;
    async fn get_export_job_status(
        &self,
        job_id: Uuid,
    ) -> AppResult<crate::handlers::ExportJobStatusResponse>;
    async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)>;
}
