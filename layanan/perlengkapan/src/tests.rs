use crate::repository::PerlengkapanRepository;
use crate::shared::error::AppResult;
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

// Include pakaian dinas tests
#[cfg(test)]
mod pakaian_dinas_test;

// Define the mock repository at file scope so it's visible to submodules.
// The remaining catch-all trait covers only the generic export concern
// (dashboard-stats moved to `dashboard/`, analisis to `analisis/`).
mock! {
    pub Repository {}
    #[async_trait]
    impl PerlengkapanRepository for Repository {
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
    async fn test_queue_export_job() {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_queue_export_job()
            .times(1)
            .returning(|_| Ok(Uuid::new_v4()));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let query = crate::handlers::ExportQuery {
            entity_type: "kebutuhan_bmn".to_string(),
            filters: None,
            limit: Some(10),
            tahun_anggaran: None,
            satker_id: None,
            status: None,
        };
        let job_id = service.queue_export_job(query).await.unwrap();
        assert!(!job_id.is_nil());
    }
}
