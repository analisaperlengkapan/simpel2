use chrono::Utc;
use secreton_core::audit::{
    AuditBackend, AuditLog, AuditLogger, AuditQuery, AuditStatus, MemoryBackend,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_audit_query_filtering() {
    let backend = Arc::new(MemoryBackend::default());
    let logger = AuditLogger::new(vec![backend.clone()]);

    // Create some logs
    let log1 = AuditLog {
        id: Uuid::new_v4(),
        timestamp: Utc::now() - chrono::Duration::hours(2),
        action: "login".to_string(),
        actor: Some("user1".to_string()),
        resource_type: "auth".to_string(),
        resource_id: "session".to_string(),
        status: AuditStatus::Success,
        ip: Some("127.0.0.1".to_string()),
        user_agent: None,
        namespace: None,
        metadata: HashMap::new(),
    };

    let log2 = AuditLog {
        id: Uuid::new_v4(),
        timestamp: Utc::now() - chrono::Duration::hours(1),
        action: "read_secret".to_string(),
        actor: Some("user1".to_string()),
        resource_type: "secret".to_string(),
        resource_id: "app/db".to_string(),
        status: AuditStatus::Failure,
        ip: Some("127.0.0.1".to_string()),
        user_agent: None,
        namespace: None,
        metadata: HashMap::new(),
    };

    let log3 = AuditLog {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        action: "login".to_string(),
        actor: Some("user2".to_string()),
        resource_type: "auth".to_string(),
        resource_id: "session".to_string(),
        status: AuditStatus::Success,
        ip: Some("127.0.0.2".to_string()),
        user_agent: None,
        namespace: Some("dev".to_string()),
        metadata: HashMap::new(),
    };

    // Log them directly via backend to ensure they are stored
    backend.log(log1).await.unwrap();
    backend.log(log2).await.unwrap();
    backend.log(log3).await.unwrap();

    // Test 1: Filter by action
    let query = AuditQuery::new().action("login");
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|l| l.action == "login"));

    // Test 2: Filter by actor
    let query = AuditQuery::new().actor("user1");
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|l| l.actor.as_deref() == Some("user1")));

    // Test 3: Filter by status
    let query = AuditQuery::new().status(AuditStatus::Failure);
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action, "read_secret");

    // Test 4: Filter by time (last 1.5 hours)
    let cutoff = Utc::now() - chrono::Duration::minutes(90);
    let query = AuditQuery::new().start_time(Some(cutoff));
    let results = logger.query(&query).await.unwrap();
    // Should match log2 (1h ago) and log3 (now). log1 is 2h ago.
    assert_eq!(results.len(), 2);

    // Test 5: Pagination
    let query = AuditQuery::new().limit(Some(1)).offset(Some(0));
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 1);

    let query = AuditQuery::new().limit(Some(1)).offset(Some(1));
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 1);

    // Test 6: Namespace
    let query = AuditQuery::new().namespace("dev");
    let results = logger.query(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].actor.as_deref(), Some("user2"));
}
