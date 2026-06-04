#[cfg(test)]
mod tests {
    use crate::shared::middleware::Claims;
    use crate::tests::MockRepository; // Accessing MockRepository from parent tests module
    use crate::{handlers::*, models::*, services::PerlengkapanService};
    use axum::extract::State;
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
}
