use std::sync::Arc;
use uuid::Uuid;

use crate::export::models::{ExportCaller, ExportQuery};
use crate::export::repository::MockExportRepository;
use crate::export::services::ExportService;
use crate::shared::satker_scope::SatkerScope;

#[tokio::test]
async fn test_queue_export_job() {
    let mut mock_repo = MockExportRepository::new();
    mock_repo
        .expect_queue_export_job()
        .times(1)
        .returning(|_, _| Ok(Uuid::new_v4()));

    let service = ExportService::new(Arc::new(mock_repo));
    let query = ExportQuery {
        entity_type: "kebutuhan_bmn".to_string(),
        filters: None,
        limit: Some(10),
        tahun_anggaran: None,
        satker_id: None,
        status: None,
        scope: SatkerScope::All,
    };
    let caller = ExportCaller {
        user_id: Uuid::new_v4(),
        is_admin: false,
        scope: SatkerScope::All,
    };
    let job_id = service.queue_export_job(query, &caller).await.unwrap();
    assert!(!job_id.is_nil());
}

fn caller(user: Uuid, admin: bool) -> ExportCaller {
    ExportCaller {
        user_id: user,
        is_admin: admin,
        scope: SatkerScope::Denied,
    }
}

/// A job is its owner's, and an administrator's; nobody else's. A job without a
/// recorded owner (rows from before ownership was stored) is administrators-only.
#[test]
fn a_job_is_visible_to_its_owner_and_to_administrators_only() {
    use crate::export::repository::may_see_job;

    let owner = Uuid::new_v4();
    let stranger = Uuid::new_v4();

    assert!(may_see_job(Some(owner), &caller(owner, false)));
    assert!(!may_see_job(Some(owner), &caller(stranger, false)));
    assert!(may_see_job(Some(owner), &caller(stranger, true)));

    assert!(!may_see_job(None, &caller(stranger, false)));
    assert!(may_see_job(None, &caller(stranger, true)));
}

/// A query that never had a scope stamped on it must export nothing, not
/// everything: the wire format cannot carry one, and the default is `Denied`.
#[test]
fn a_query_without_a_stamped_scope_is_denied() {
    let query: ExportQuery = serde_json::from_str(r#"{"entity_type":"kebutuhan_bmn"}"#).unwrap();
    assert_eq!(query.scope, SatkerScope::Denied);

    // And a client cannot smuggle one in.
    let sneaky: ExportQuery =
        serde_json::from_str(r#"{"entity_type":"kebutuhan_bmn","scope":"All"}"#).unwrap();
    assert_eq!(sneaky.scope, SatkerScope::Denied);
}
