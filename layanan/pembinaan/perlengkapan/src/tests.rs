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
    use serde_json::json;

    // Define the mock repository
    mock! {
        pub Repository {}
        #[async_trait]
        impl PerlengkapanRepository for Repository {
            async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;
            async fn get_all_aset(&self, page: i32, per_page: i32) -> AppResult<(Vec<Aset>, i64)>;
            async fn get_aset_by_id(&self, id: Uuid) -> AppResult<Aset>;
            async fn create_aset(&self, request: CreateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset>;
            async fn update_aset(&self, id: Uuid, request: UpdateAsetRequest, user_id: Option<Uuid>) -> AppResult<Aset>;
            async fn delete_aset(&self, id: Uuid) -> AppResult<()>;
            async fn check_aset_code_exists(&self, code: &str) -> AppResult<bool>;
            async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)>;
            async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan>;
            async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan>;
            async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
            async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;
            async fn upsert_siman_asset(&self, id: Uuid, nama: String, kategori: String, kode_bmn: String, merk: Option<String>, nup: String, kondisi: String, lokasi: String, nilai_perolehan: Option<f64>) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_create_aset_success() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();

        // Expect check_aset_code_exists to return false
        mock_repo
            .expect_check_aset_code_exists()
            .with(eq("101.1"))
            .times(1)
            .returning(|_| Ok(false));

        // Expect create_aset to be called
        mock_repo
            .expect_create_aset()
            .times(1)
            .returning(move |_, _| {
                Ok(Aset {
                    id: Uuid::new_v4(),
                    nama: "Laptop".to_string(),
                    kategori: "Elektronik".to_string(),
                    kode_bmn: "101.1".to_string(),
                    merk: Some("Dell".to_string()),
                    nup: Some("1".to_string()),
                    kondisi: "baik".to_string(),
                    lokasi: "Gudang".to_string(),
                    nilai_perolehan: Some(1000.0),
                    tanggal_perolehan: None,
                    status: "aktif".to_string(),
                    keterangan: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    created_by: Some(user_id),
                    updated_by: Some(user_id),
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));

        let req = CreateAsetRequest {
            nama: "Laptop".to_string(),
            kategori: "Elektronik".to_string(),
            kode_bmn: "101.1".to_string(),
            merk: Some("Dell".to_string()),
            nup: Some("1".to_string()),
            kondisi: "baik".to_string(),
            lokasi: "Gudang".to_string(),
            nilai_perolehan: Some(1000.0),
            tanggal_perolehan: None,
            keterangan: None,
        };

        let result = service.create_aset(req, Some(user_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kode_bmn, "101.1");
    }

    #[tokio::test]
    async fn test_create_aset_duplicate_code() {
        let mut mock_repo = MockRepository::new();

        // Expect check_aset_code_exists to return true (exists)
        mock_repo
            .expect_check_aset_code_exists()
            .with(eq("101.1"))
            .times(1)
            .returning(|_| Ok(true));

        // Expect create_aset NOT to be called
        mock_repo.expect_create_aset().never();

        let service = PerlengkapanService::new(Arc::new(mock_repo));

        let req = CreateAsetRequest {
            nama: "Laptop".to_string(),
            kategori: "Elektronik".to_string(),
            kode_bmn: "101.1".to_string(),
            merk: Some("Dell".to_string()),
            nup: Some("1".to_string()),
            kondisi: "baik".to_string(),
            lokasi: "Gudang".to_string(),
            nilai_perolehan: Some(1000.0),
            tanggal_perolehan: None,
            keterangan: None,
        };

        let result = service.create_aset(req, None).await;
        assert!(result.is_err());
        // Verify error type is conflict (409) or similar message
        match result {
            Err(AppError::Conflict(msg)) => assert_eq!(msg, "Kode BMN sudah digunakan"),
            _ => panic!("Expected Conflict error"),
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
                    total_aset: 10,
                    total_pengadaan: 5,
                    total_analisis: 2,
                    aset_aktif: 8,
                    pengadaan_berjalan: 3,
                    analisis_pending: 1,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let stats = service.get_dashboard_stats().await.unwrap();

        assert_eq!(stats.total_aset, 10);
        assert_eq!(stats.aset_aktif, 8);
    }

    #[tokio::test]
    async fn test_process_siman_assets() {
        let mut mock_repo = MockRepository::new();

        // Expect upsert to be called once
        mock_repo
            .expect_upsert_siman_asset()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _, _| Ok(()));

        let service = PerlengkapanService::new(Arc::new(mock_repo));

        let siman_data = vec![
            json!({
                "KD_BRG": "101",
                "NO_ASET": "1",
                "NM_BRG": "Laptop",
                "MERK": "Dell",
                "KONDISI": "BAIK",
                "NM_SATKER": "Pusat",
                "RPH_ASET": 15000000
            }),
            json!({
                "NM_BRG": "Invalid Asset" // Missing IDs
            })
        ];

        let result = service.process_siman_assets(siman_data, "Elektronik").await;
        assert!(result.is_ok());
        let (success, errors) = result.unwrap();

        assert_eq!(success, 1);
        assert_eq!(errors, 1);
    }
}

// Register handler tests
mod handlers_test;
