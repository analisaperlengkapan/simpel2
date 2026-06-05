use std::sync::Arc;
use uuid::Uuid;

use crate::export::models::ExportQuery;
use crate::export::repository::MockExportRepository;
use crate::export::services::ExportService;

#[tokio::test]
async fn test_queue_export_job() {
    let mut mock_repo = MockExportRepository::new();
    mock_repo
        .expect_queue_export_job()
        .times(1)
        .returning(|_| Ok(Uuid::new_v4()));

    let service = ExportService::new(Arc::new(mock_repo));
    let query = ExportQuery {
        entity_type: "kebutuhan_bmn".to_string(),
        filters: None,
        limit: Some(10),
        tahun_anggaran: None,
        satker_id: None,
        status: None,
    };
    let job_id = service.queue_export_job(query).await.unwrap();
    assert!(!job_id.is_nil());
}
