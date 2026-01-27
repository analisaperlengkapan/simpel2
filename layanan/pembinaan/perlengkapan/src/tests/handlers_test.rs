#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::{
        extract::{State, Query, Path},
        Json,
    };
    use uuid::Uuid;
    use crate::{
        handlers::*,
        models::*,
        services::PerlengkapanService,
        repository::MockPerlengkapanRepository,
        middleware::Claims,
    };

    fn create_mock_claims() -> Claims {
        Claims {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            permissions: vec![],
        }
    }

    #[tokio::test]
    async fn test_handler_get_all_aset() {
        let mut mock_repo = MockPerlengkapanRepository::new();

        mock_repo.expect_get_all_aset()
            .with(mockall::predicate::eq(1), mockall::predicate::eq(20))
            .times(1)
            .returning(|_, _| Ok((vec![], 0)));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery { page: 1, per_page: 20 });
        let claims = create_mock_claims();

        let result = get_all_aset(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // Check response content if needed (response.0 is the Json wrapper)
        assert_eq!(response.0.page, 1);
        assert_eq!(response.0.data.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_create_aset() {
        let mut mock_repo = MockPerlengkapanRepository::new();

        // Expect validation check
        mock_repo.expect_check_aset_code_exists()
            .returning(|_| Ok(false));

        // Expect creation
        mock_repo.expect_create_aset()
            .times(1)
            .returning(|_, _| Ok(Aset {
                id: Uuid::new_v4(),
                nama: "Test Aset".to_string(),
                kategori: "Kategori".to_string(),
                kode_bmn: "123".to_string(),
                merk: None,
                nup: None,
                kondisi: "baik".to_string(),
                lokasi: "Lokasi".to_string(),
                nilai_perolehan: None,
                tanggal_perolehan: None,
                status: "aktif".to_string(),
                keterangan: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                created_by: None,
                updated_by: None,
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let claims = create_mock_claims();
        let request = Json(CreateAsetRequest {
            nama: "Test Aset".to_string(),
            kategori: "Kategori".to_string(),
            kode_bmn: "123".to_string(),
            merk: None,
            nup: Some("1".to_string()),
            kondisi: "baik".to_string(),
            lokasi: "Lokasi".to_string(),
            nilai_perolehan: None,
            tanggal_perolehan: None,
            keterangan: None,
        });

        let result = create_aset(state, claims, request).await;

        assert!(result.is_ok());
        let (status, _response) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_handler_get_aset_by_id_found() {
        let mut mock_repo = MockPerlengkapanRepository::new();
        let id = Uuid::new_v4();

        mock_repo.expect_get_aset_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(Aset {
                id,
                nama: "Found Aset".to_string(),
                kategori: "Kategori".to_string(),
                kode_bmn: "123".to_string(),
                merk: None,
                nup: None,
                kondisi: "baik".to_string(),
                lokasi: "Lokasi".to_string(),
                nilai_perolehan: None,
                tanggal_perolehan: None,
                status: "aktif".to_string(),
                keterangan: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                created_by: None,
                updated_by: None,
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_aset_by_id(state, Path(id), claims).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.data.nama, "Found Aset");
    }
}
