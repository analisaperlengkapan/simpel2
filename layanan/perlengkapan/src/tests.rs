use crate::shared::error::AppResult;
use crate::{models::*, repository::PerlengkapanRepository};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

// Include pakaian dinas tests
#[cfg(test)]
mod pakaian_dinas_test;

// Define the mock repository at file scope so it's visible to submodules
mock! {
    pub Repository {}
    #[async_trait]
    impl PerlengkapanRepository for Repository {
        async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;
        async fn queue_export_job(&self, query: crate::handlers::ExportQuery) -> AppResult<Uuid>;
        async fn export_to_excel_sync(&self, query: crate::handlers::ExportQuery) -> AppResult<Vec<u8>>;
        async fn get_export_job_status(&self, job_id: Uuid) -> AppResult<crate::handlers::ExportJobStatusResponse>;
        async fn download_export_job(&self, job_id: Uuid) -> AppResult<(String, Vec<u8>)>;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::services::PerlengkapanService;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_dashboard_stats() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_dashboard_stats()
            .times(1)
            .returning(|| {
                Ok(DashboardStats {
                    total_aset: 100,
                    total_nilai_aset: 1000000.0,
                    total_satker: 5,
                    aset_baik: 80,
                    aset_rusak: 20,
                    categories: vec![],
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let stats = service.get_dashboard_stats().await.unwrap();

        assert_eq!(stats.total_aset, 100);
        assert_eq!(stats.aset_baik, 80);
    }
}

// Register handler tests
#[cfg(test)]
mod handlers_test;
