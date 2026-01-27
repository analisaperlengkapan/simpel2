#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::*,
        repository::PerlengkapanRepository,
        services::PerlengkapanService,
        errors::{AppError, AppResult},
    };
    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;

    // Define the mock repository
    mock! {
        pub Repository {}
        #[async_trait]
        impl PerlengkapanRepository for Repository {
            async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;
            async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)>;
            async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset>;
            async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)>;
            async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan>;
            async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan>;
            async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
            async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;
        }
    }

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
    async fn test_get_all_assets() {
        let mut mock_repo = MockRepository::new();
        let asset_id = Uuid::new_v4();

        mock_repo
            .expect_get_all_assets()
            .with(eq(1), eq(20), eq(None))
            .times(1)
            .returning(move |_, _, _| {
                Ok((
                    vec![Asset {
                        id: asset_id,
                        kategori_aset: "Tanah".to_string(),
                        no_aset: "1".to_string(),
                        nama_aset: Some("Tanah Kantor".to_string()),
                        kode_barang: Some("101".to_string()),
                        merk: None,
                        tipe: None,
                        kondisi: Some("Baik".to_string()),
                        lokasi: Some("Jakarta".to_string()),
                        satker: Some("Pusat".to_string()),
                        nilai_perolehan: Some(1000000.0),
                        tgl_perolehan: Some("2023-01-01".to_string()),
                        updated_at: Utc::now(),
                    }],
                    1
                ))
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let (assets, total) = service.get_all_assets(1, 20, None).await.unwrap();

        assert_eq!(total, 1);
        assert_eq!(assets[0].id, asset_id);
    }

    #[tokio::test]
    async fn test_get_asset_by_id() {
        let mut mock_repo = MockRepository::new();
        let asset_id = Uuid::new_v4();

        mock_repo
            .expect_get_asset_by_id()
            .with(eq(asset_id))
            .times(1)
            .returning(move |_| {
                Ok(Asset {
                    id: asset_id,
                    kategori_aset: "Tanah".to_string(),
                    no_aset: "1".to_string(),
                    nama_aset: Some("Tanah Kantor".to_string()),
                    kode_barang: Some("101".to_string()),
                    merk: None,
                    tipe: None,
                    kondisi: Some("Baik".to_string()),
                    lokasi: Some("Jakarta".to_string()),
                    satker: Some("Pusat".to_string()),
                    nilai_perolehan: Some(1000000.0),
                    tgl_perolehan: Some("2023-01-01".to_string()),
                    updated_at: Utc::now(),
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let asset = service.get_asset_by_id(asset_id).await.unwrap();

        assert_eq!(asset.id, asset_id);
    }
}

// Register handler tests
mod handlers_test;
