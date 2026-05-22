use crate::{models::*, repository::PerlengkapanRepository};
use crate::shared::error::AppResult;
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
                    1,
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

    #[tokio::test]
    async fn test_create_pemakaian() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePemakaianRequest {
            asset_id,
            piminjam_nama: "John Doe".to_string(),
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            keperluan: Some("Project A".to_string()),
        };

        mock_repo
            .expect_create_pemakaian()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Pemakaian {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    piminjam_nama: req.piminjam_nama,
                    tanggal_mulai: req.tanggal_mulai,
                    tanggal_selesai: req.tanggal_selesai,
                    status: "dipinjam".to_string(),
                    keperluan: req.keperluan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_pemakaian(req, Some(user_id)).await.unwrap();

        assert_eq!(result.piminjam_nama, "John Doe");
        assert_eq!(result.status, "dipinjam");
    }

    #[tokio::test]
    async fn test_create_penghapusan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePenghapusanRequest {
            asset_id,
            tanggal_penghapusan: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            alasan: "Rusak berat".to_string(),
            metode_penghapusan: "Lelang".to_string(),
            nilai_residu: Some(100000.0),
        };

        mock_repo
            .expect_create_penghapusan()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Penghapusan {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    tanggal_penghapusan: req.tanggal_penghapusan,
                    alasan: req.alasan,
                    metode_penghapusan: req.metode_penghapusan,
                    status: "usulan".to_string(),
                    nilai_residu: req.nilai_residu,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service
            .create_penghapusan(req, Some(user_id))
            .await
            .unwrap();

        assert_eq!(result.alasan, "Rusak berat");
        assert_eq!(result.status, "usulan");
    }
}

// Register handler tests
#[cfg(test)]
mod handlers_test;
