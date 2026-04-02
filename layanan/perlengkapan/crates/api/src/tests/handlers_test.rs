#[cfg(test)]
mod tests {
    use crate::tests::MockRepository;
    use crate::{handlers::*, middleware::Claims, models::*, services::PerlengkapanService};
    use axum::Json;
    use axum::extract::{Path, Query, State};
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_mock_claims() -> Claims {
        Claims {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            permissions: vec![],
            nama: Some("Test User".to_string()),
            jabatan: Some("Admin".to_string()),
            name: Some("Test User".to_string()),
            nip: Some("123456789".to_string()),
        }
    }

    #[tokio::test]
    async fn test_handler_get_all_assets() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_all_assets()
            .with(
                mockall::predicate::eq(1),
                mockall::predicate::eq(20),
                mockall::predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _| Ok((vec![], 0)));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery {
            page: 1,
            per_page: 20,
            category: None,
        });
        let claims = create_mock_claims();

        let result = get_all_assets(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.page, 1);
        assert_eq!(response.0.data.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_get_asset_by_id_found() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_get_asset_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| {
                Ok(Asset {
                    id,
                    kategori_aset: "Tanah".to_string(),
                    no_aset: "1".to_string(),
                    nama_aset: Some("Found Asset".to_string()),
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
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_asset_by_id(state, Path(id), claims).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().0.data.nama_aset,
            Some("Found Asset".to_string())
        );
    }

    #[tokio::test]
    async fn test_handler_get_dashboard_stats() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_dashboard_stats()
            .times(1)
            .returning(|| {
                Ok(DashboardStats {
                    total_aset: 100,
                    total_nilai_aset: 5000.0,
                    total_satker: 2,
                    aset_baik: 90,
                    aset_rusak: 10,
                    categories: vec![],
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_dashboard_stats(state, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.total_aset, 100);
        assert_eq!(response.0.data.aset_baik, 90);
    }

    #[tokio::test]
    async fn test_handler_get_analisis_by_id() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_get_analisis_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| {
                Ok(AnalisisKebutuhan {
                    id,
                    judul: "Test Analisis".to_string(),
                    kategori: "TIK".to_string(),
                    deskripsi: None,
                    prioritas: "tinggi".to_string(),
                    status: "draft".to_string(),
                    estimasi_biaya: None,
                    justifikasi: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: None,
                    updated_by: None,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_analisis_by_id(state, Path(id), claims).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.data.judul, "Test Analisis");
    }

    #[tokio::test]
    async fn test_handler_create_pemakaian() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        mock_repo
            .expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| {
                Ok(Asset {
                    id: asset_id,
                    kategori_aset: "Tanah".to_string(),
                    no_aset: "1".to_string(),
                    nama_aset: Some("Found Asset".to_string()),
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

        mock_repo
            .expect_create_pemakaian()
            .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| {
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
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePemakaianRequest {
            asset_id,
            piminjam_nama: "Jane Doe".to_string(),
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            keperluan: None,
        });

        let result = create_pemakaian(state, claims, request).await;

        assert!(result.is_ok());
        let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.piminjam_nama, "Jane Doe");
    }
}
