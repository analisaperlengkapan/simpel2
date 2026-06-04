use async_trait::async_trait;
use uuid::Uuid;

use crate::models::*;
use crate::shared::error::AppResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PerlengkapanRepository: Send + Sync {
    // Dashboard
    async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;

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
