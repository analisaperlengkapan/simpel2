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
        middleware::Claims,
    };
    use super::MockRepository; // Accessing MockRepository from parent tests module
    use chrono::Utc;

    fn create_mock_claims() -> Claims {
        Claims {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            permissions: vec![],
        }
    }

    #[tokio::test]
    async fn test_handler_get_all_assets() {
        let mut mock_repo = MockRepository::new();

        mock_repo.expect_get_all_assets()
            .with(mockall::predicate::eq(1), mockall::predicate::eq(20), mockall::predicate::eq(None))
            .times(1)
            .returning(|_, _, _| Ok((vec![], 0)));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery { page: 1, per_page: 20, category: None });
        let claims = create_mock_claims();

        let result = get_all_assets(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // Check response content if needed (response.0 is the Json wrapper)
        assert_eq!(response.0.page, 1);
        assert_eq!(response.0.data.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_get_asset_by_id_found() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();

        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(Asset {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_asset_by_id(state, Path(id), claims).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.data.nama_aset, Some("Found Asset".to_string()));
    }
}
