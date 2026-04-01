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
    }

    #[tokio::test]
    async fn test_handler_get_asset_by_id_found() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();
        mock_repo.expect_get_asset_by_id().returning(move |_| {
            Ok(Asset {
                id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found".to_string()),
                kode_barang: None,
                merk: None,
                tipe: None,
                kondisi: None,
                lokasi: None,
                satker: None,
                nilai_perolehan: None,
                tgl_perolehan: None,
                updated_at: Utc::now(),
            })
        });
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = get_asset_by_id(State(service), Path(id), create_mock_claims()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_get_all_penghapusan() {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_get_all_penghapusan()
            .returning(|_, _| Ok((vec![], 0)));
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = get_all_penghapusan(
            State(service),
            Query(PaginationQuery {
                page: 1,
                per_page: 20,
                category: None,
            }),
            create_mock_claims(),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_create_penghapusan() {
        let mut mock_repo = MockRepository::new();
        let asset_id = Uuid::new_v4();
        mock_repo.expect_get_asset_by_id().returning(move |_| {
            Ok(Asset {
                id: asset_id,
                kategori_aset: "".into(),
                no_aset: "".into(),
                nama_aset: None,
                kode_barang: None,
                merk: None,
                tipe: None,
                kondisi: None,
                lokasi: None,
                satker: None,
                nilai_perolehan: None,
                tgl_perolehan: None,
                updated_at: Utc::now(),
            })
        });
        mock_repo
            .expect_create_penghapusan()
            .returning(move |req, _| {
                Ok(Penghapusan {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    tanggal_penghapusan: req.tanggal_penghapusan,
                    alasan: req.alasan,
                    metode_penghapusan: req.metode_penghapusan,
                    status: "usulan".into(),
                    nilai_residu: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: None,
                    updated_by: None,
                })
            });
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let req = CreatePenghapusanRequest {
            asset_id,
            tanggal_penghapusan: Utc::now().date_naive(),
            alasan: "Test".into(),
            metode_penghapusan: "Lelang".into(),
            nilai_residu: None,
        };
        let result = create_penghapusan(State(service), create_mock_claims(), Json(req)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_get_penghapusan_by_id() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();
        mock_repo
            .expect_get_penghapusan_by_id()
            .returning(move |_| {
                Ok(Penghapusan {
                    id,
                    asset_id: Uuid::new_v4(),
                    tanggal_penghapusan: Utc::now().date_naive(),
                    alasan: "Test".into(),
                    metode_penghapusan: "Lelang".into(),
                    status: "usulan".into(),
                    nilai_residu: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: None,
                    updated_by: None,
                })
            });
        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = get_penghapusan_by_id(State(service), Path(id), create_mock_claims()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_get_dashboard_stats() {
        let mut mock_repo = MockRepository::new();
        mock_repo.expect_get_dashboard_stats().returning(|| {
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
        let result = get_dashboard_stats(State(service), create_mock_claims()).await;
        assert!(result.is_ok());
    }
}
