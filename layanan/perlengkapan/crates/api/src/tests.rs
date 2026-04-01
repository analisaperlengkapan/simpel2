use crate::{errors::AppResult, models::*, repository::PerlengkapanRepository};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

#[cfg(test)]
mod pakaian_dinas_test;

mock! {
    pub Repository {}
    #[async_trait]
    impl PerlengkapanRepository for Repository {
        async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;
        async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)>;
        async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset>;
        async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
        async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;
        async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan>;
        async fn get_all_pemakaian(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pemakaian>, i64)>;
        async fn create_pemakaian(&self, request: CreatePemakaianRequest, user_id: Option<Uuid>) -> AppResult<Pemakaian>;
        async fn get_pemakaian_by_id(&self, id: Uuid) -> AppResult<Pemakaian>;
        async fn get_all_penghapusan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Penghapusan>, i64)>;
        async fn create_penghapusan(&self, request: CreatePenghapusanRequest, user_id: Option<Uuid>) -> AppResult<Penghapusan>;
        async fn get_penghapusan_by_id(&self, id: Uuid) -> AppResult<Penghapusan>;
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
                    total_nilai_aset: 0.0,
                    total_satker: 1,
                    aset_baik: 100,
                    aset_rusak: 0,
                    categories: vec![],
                })
            });
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let stats = service.get_dashboard_stats().await.unwrap();
        assert_eq!(stats.total_aset, 100);
    }

    #[tokio::test]
    async fn test_get_all_assets() {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get_all_assets()
            .times(1)
            .returning(|_, _, _| Ok((vec![], 0)));
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let (_, total) = service.get_all_assets(1, 20, None).await.unwrap();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_create_analisis() {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_create_analisis()
            .times(1)
            .returning(|req, _| {
                Ok(AnalisisKebutuhan {
                    id: Uuid::new_v4(),
                    judul: req.judul,
                    kategori: req.kategori,
                    deskripsi: req.deskripsi,
                    prioritas: req.prioritas,
                    status: "draft".into(),
                    estimasi_biaya: req.estimasi_biaya,
                    justifikasi: req.justifikasi,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: None,
                    updated_by: None,
                })
            });
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let req = CreateAnalisisRequest {
            judul: "Test".into(),
            kategori: "TIK".into(),
            deskripsi: None,
            prioritas: "tinggi".into(),
            estimasi_biaya: None,
            justifikasi: None,
        };
        let result = service.create_analisis(req, None).await.unwrap();
        assert_eq!(result.judul, "Test");
    }
}

#[cfg(test)]
mod handlers_test;
