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
        async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
        async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;
        async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan>;
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
    use chrono::Utc;
    use mockall::predicate::*;
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

    #[tokio::test]
    async fn test_create_analisis() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let req = CreateAnalisisRequest {
            judul: "New Analisis".to_string(),
            kategori: "TIK".to_string(),
            deskripsi: Some("Desc".to_string()),
            prioritas: "tinggi".to_string(),
            estimasi_biaya: Some(100000.0),
            justifikasi: None,
        };

        mock_repo
            .expect_create_analisis()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(AnalisisKebutuhan {
                    id: Uuid::new_v4(),
                    judul: req.judul,
                    kategori: req.kategori,
                    deskripsi: req.deskripsi,
                    prioritas: req.prioritas,
                    status: "draft".to_string(),
                    estimasi_biaya: req.estimasi_biaya,
                    justifikasi: req.justifikasi,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_analisis(req, Some(user_id)).await.unwrap();

        assert_eq!(result.judul, "New Analisis");
    }
}

// Register handler tests
#[cfg(test)]
mod handlers_test;
