#[cfg(test)]
mod tests {
    use crate::shared::middleware::Claims;
    use crate::tests::MockRepository; // Accessing MockRepository from parent tests module
    use crate::{handlers::*, models::*, services::PerlengkapanService};
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
            satker_code: Some("001".to_string()),
        }
    }

    #[tokio::test]
    async fn test_handler_get_all_analisis() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_all_analisis()
            .with(mockall::predicate::eq(1), mockall::predicate::eq(20))
            .times(1)
            .returning(|_, _| {
                Ok((
                    vec![AnalisisKebutuhan {
                        id: Uuid::new_v4(),
                        judul: "Analisis Server".to_string(),
                        kategori: "TIK".to_string(),
                        deskripsi: None,
                        prioritas: "tinggi".to_string(),
                        status: "draft".to_string(),
                        estimasi_biaya: Some(50000000.0),
                        justifikasi: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        created_by: None,
                        updated_by: None,
                    }],
                    1,
                ))
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery {
            page: 1,
            per_page: 20,
            category: None,
        });
        let claims = create_mock_claims();

        let result = get_all_analisis(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.len(), 1);
        assert_eq!(response.0.data[0].judul, "Analisis Server");
    }

    #[tokio::test]
    async fn test_handler_create_analisis() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();

        mock_repo
            .expect_create_analisis()
            .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| {
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
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreateAnalisisRequest {
            judul: "Handler Analisis".to_string(),
            kategori: "UMUM".to_string(),
            deskripsi: None,
            prioritas: "sedang".to_string(),
            estimasi_biaya: None,
            justifikasi: None,
        });

        let result = create_analisis(state, claims, request).await;

        assert!(result.is_ok());
        let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.judul, "Handler Analisis");
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
}
