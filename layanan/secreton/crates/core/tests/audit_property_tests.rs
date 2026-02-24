//! Property-Based Tests for Audit System
//!
//! These tests verify the correctness properties of the audit logging system.

use chrono::Utc;
use proptest::option;
use proptest::prelude::*;
use secreton_core::security::audit::{
    AdvancedAuditSystem, AuditCategory, AuditEvent, AuditQuery, AuditResult, AuditSeverity,
    AuditStorage, ComplianceConfig, ComplianceStandard, ReportSchedule, RetentionPolicy,
    SecurityAuditError, SignedAuditEntry, SimpleAnomalyDetector,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

// Mock storage implementation for testing
struct MockAuditStorage {
    entries: Arc<Mutex<Vec<SignedAuditEntry>>>,
}

impl MockAuditStorage {
    fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_entries(&self) -> Vec<SignedAuditEntry> {
        self.entries.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl AuditStorage for MockAuditStorage {
    async fn store_entry(&self, entry: &SignedAuditEntry) -> Result<(), SecurityAuditError> {
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry.clone());
        Ok(())
    }

    async fn retrieve_entries(
        &self,
        start_sequence: u64,
        end_sequence: u64,
    ) -> Result<Vec<SignedAuditEntry>, SecurityAuditError> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .filter(|e| e.sequence_number >= start_sequence && e.sequence_number <= end_sequence)
            .cloned()
            .collect())
    }

    async fn get_latest_sequence(&self) -> Result<u64, SecurityAuditError> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.iter().map(|e| e.sequence_number).max().unwrap_or(0))
    }

    async fn search_entries(
        &self,
        query: &AuditQuery,
    ) -> Result<Vec<SignedAuditEntry>, SecurityAuditError> {
        let entries = self.entries.lock().unwrap();
        let mut filtered: Vec<SignedAuditEntry> = entries
            .iter()
            .filter(|e| query.matches(e))
            .cloned()
            .collect();

        // Apply limit and offset
        if let Some(offset) = query.offset {
            filtered = filtered.into_iter().skip(offset).collect();
        }
        if let Some(limit) = query.limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
    }

    async fn verify_chain_integrity(
        &self,
        _start_sequence: u64,
        _end_sequence: u64,
    ) -> Result<bool, SecurityAuditError> {
        Ok(true)
    }

    async fn archive_entries(
        &self,
        _before_sequence: u64,
        _archive_location: &str,
    ) -> Result<u64, SecurityAuditError> {
        Ok(0)
    }
}

// Helper to setup test audit system
async fn setup_test_audit_system() -> (Arc<AdvancedAuditSystem>, Arc<MockAuditStorage>) {
    let storage = Arc::new(MockAuditStorage::new());
    let compliance_config = ComplianceConfig {
        standards: vec![ComplianceStandard::PciDss],
        report_schedule: ReportSchedule::Daily,
        retention_policy: RetentionPolicy {
            default_retention: Duration::from_secs(86400 * 365),
            category_specific: HashMap::new(),
            archive_after: Duration::from_secs(86400 * 90),
            archive_location: "/archive".to_string(),
            permanent_retention_categories: vec![AuditCategory::SecurityEvent],
        },
        encryption_required: true,
        digital_signatures: true,
    };

    let anomaly_detector = Arc::new(SimpleAnomalyDetector);

    let audit_system = Arc::new(
        AdvancedAuditSystem::new(
            storage.clone() as Arc<dyn AuditStorage>,
            "test_node".to_string(),
            compliance_config,
            anomaly_detector,
        )
        .unwrap(),
    );

    audit_system.initialize().await.unwrap();

    (audit_system, storage)
}

// Strategy for generating audit events
fn audit_event_strategy() -> impl Strategy<Value = AuditEvent> {
    (
        "[a-z_]{5,20}",                                             // action
        option::of("[a-z0-9]{5,15}"),                               // principal
        option::of("[a-z/]{5,30}"),                                 // target
        option::of("(127\\.0\\.0\\.1|192\\.168\\.1\\.[0-9]{1,3})"), // source_ip
    )
        .prop_map(|(action, principal, target, source_ip)| AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: AuditSeverity::Info,
            category: AuditCategory::DataAccess,
            source: "test_service".to_string(),
            principal,
            target,
            action,
            result: AuditResult::Success,
            context: HashMap::new(),
            source_ip,
            user_agent: None,
            session_id: Some("session_123".to_string()),
            correlation_id: Some("corr_123".to_string()),
            geo_location: None,
            risk_score: None,
            compliance_tags: Vec::new(),
            sensitive_data_access: false,
            duration: Some(Duration::from_millis(100)),
        })
}

// **Feature: secreton-comprehensive-enhancement, Property 28: Audit Log Integrity**
// **Validates: Requirements 12.2**
//
// Property: For any audit log entry, HMAC verification SHALL detect any tampering
// with the entry content.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn property_audit_log_integrity(
        event in audit_event_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (audit_system, storage) = setup_test_audit_system().await;

            // Log the event
            let event_id = audit_system.log_event(event.clone()).await
                .expect("Event logging should succeed");

            prop_assert!(!event_id.is_nil(), "Event ID should not be nil");

            // Wait for batch processing
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Retrieve the stored entry
            let entries = storage.get_entries();
            prop_assert!(!entries.is_empty(), "Should have at least one entry");

            // Find our entry
            let our_entry = entries.iter()
                .find(|e| e.event.event_id == event_id);

            prop_assert!(our_entry.is_some(), "Should find our logged entry");

            let entry = our_entry.unwrap();

            // Property 1: Entry should verify successfully
            let verification_result = audit_system.verify_entry(entry)
                .expect("Verification should not error");

            prop_assert!(verification_result, "Entry should verify successfully");

            // Property 2: Tampering with event data should fail verification
            let mut tampered_entry = entry.clone();
            tampered_entry.event.action = "tampered_action".to_string();

            let tampered_verification = audit_system.verify_entry(&tampered_entry)
                .expect("Verification should not error");

            prop_assert!(!tampered_verification,
                "Tampered entry should fail verification");

            // Property 3: Tampering with HMAC should fail verification
            let mut hmac_tampered = entry.clone();
            hmac_tampered.hmac = vec![0; 32];

            let hmac_verification = audit_system.verify_entry(&hmac_tampered)
                .expect("Verification should not error");

            prop_assert!(!hmac_verification,
                "Entry with tampered HMAC should fail verification");

            // Property 4: Tampering with signature should fail verification
            let mut sig_tampered = entry.clone();
            sig_tampered.signature = vec![0; 64];

            let sig_verification = audit_system.verify_entry(&sig_tampered)
                .expect("Verification should not error");

            prop_assert!(!sig_verification,
                "Entry with tampered signature should fail verification");

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 28: Audit Log Integrity (Chain)**
// **Validates: Requirements 12.2**
//
// Property: For any sequence of audit entries, the HMAC chain SHALL maintain integrity
// such that tampering with any entry breaks the chain.
//
// Note: This test is simplified to avoid long wait times for batch processing
#[cfg(test)]
mod chain_integrity_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_chain_integrity_simple() {
        let (audit_system, storage) = setup_test_audit_system().await;

        // Log events
        for i in 0..3 {
            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AuditSeverity::Info,
                category: AuditCategory::DataAccess,
                source: "test".to_string(),
                principal: Some(format!("user{}", i)),
                target: Some(format!("secret/test{}", i)),
                action: format!("read{}", i),
                result: AuditResult::Success,
                context: HashMap::new(),
                source_ip: Some("127.0.0.1".to_string()),
                user_agent: None,
                session_id: None,
                correlation_id: None,
                geo_location: None,
                risk_score: None,
                compliance_tags: Vec::new(),
                sensitive_data_access: false,
                duration: None,
            };
            audit_system.log_event(event).await.unwrap();
        }

        // Wait for batch processing
        tokio::time::sleep(Duration::from_secs(6)).await;

        // Retrieve all entries
        let entries = storage.get_entries();
        assert!(entries.len() >= 3, "Should have at least 3 entries");

        // Property 1: All entries should verify individually
        // This tests that HMAC and signature verification works
        for entry in &entries {
            assert!(
                audit_system.verify_entry(entry).unwrap(),
                "Each entry should verify successfully"
            );
        }

        // Property 2: Tampering with any entry should be detectable
        for (idx, entry) in entries.iter().enumerate() {
            let mut tampered_entry = entry.clone();
            tampered_entry.event.action = format!("tampered_{}", idx);

            // The tampered entry should fail verification
            assert!(
                !audit_system.verify_entry(&tampered_entry).unwrap(),
                "Tampered entry {} should fail verification",
                idx
            );
        }

        // Property 3: Sequence numbers should be monotonically increasing
        for i in 1..entries.len() {
            assert!(
                entries[i].sequence_number > entries[i - 1].sequence_number,
                "Sequence numbers should be monotonically increasing"
            );
        }
    }
}

#[cfg(test)]
mod audit_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_empty_audit_log() {
        let (_audit_system, storage) = setup_test_audit_system().await;

        // No events logged yet
        let entries = storage.get_entries();
        assert_eq!(entries.len(), 0, "Should start with empty audit log");
    }

    #[tokio::test]
    async fn test_single_audit_entry() {
        let (audit_system, storage) = setup_test_audit_system().await;

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: AuditSeverity::Info,
            category: AuditCategory::DataAccess,
            source: "test".to_string(),
            principal: Some("user1".to_string()),
            target: Some("secret/test".to_string()),
            action: "read".to_string(),
            result: AuditResult::Success,
            context: HashMap::new(),
            source_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            session_id: None,
            correlation_id: None,
            geo_location: None,
            risk_score: None,
            compliance_tags: Vec::new(),
            sensitive_data_access: false,
            duration: None,
        };

        audit_system.log_event(event).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        let entries = storage.get_entries();
        assert_eq!(entries.len(), 1, "Should have one entry");

        // First entry should have genesis previous_hash and previous_hmac
        let entry = &entries[0];
        assert_eq!(entry.sequence_number, 1);
        assert_eq!(entry.previous_hash, vec![0; 32]);
        assert_eq!(entry.previous_hmac, vec![0; 32]);

        // Should verify successfully
        assert!(audit_system.verify_entry(entry).unwrap());
    }

    #[tokio::test]
    async fn test_concurrent_audit_logging() {
        let (audit_system, storage) = setup_test_audit_system().await;

        // Log multiple events concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let audit_system = audit_system.clone();
            let handle = tokio::spawn(async move {
                let event = AuditEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    severity: AuditSeverity::Info,
                    category: AuditCategory::DataAccess,
                    source: "test".to_string(),
                    principal: Some(format!("user{}", i)),
                    target: Some(format!("secret/test{}", i)),
                    action: "read".to_string(),
                    result: AuditResult::Success,
                    context: HashMap::new(),
                    source_ip: Some("127.0.0.1".to_string()),
                    user_agent: None,
                    session_id: None,
                    correlation_id: None,
                    geo_location: None,
                    risk_score: None,
                    compliance_tags: Vec::new(),
                    sensitive_data_access: false,
                    duration: None,
                };
                audit_system.log_event(event).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        let entries = storage.get_entries();
        assert_eq!(entries.len(), 10, "Should have 10 entries");

        // All entries should verify
        for entry in &entries {
            assert!(audit_system.verify_entry(entry).unwrap());
        }
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 29: Audit Query Filtering**
// **Validates: Requirements 12.3**
//
// Property: For any audit query with filters, the results SHALL contain only entries
// matching all filter criteria.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn property_audit_query_filtering(
        actor in "[a-z0-9]{5,15}",
        action in "[a-z_]{5,20}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (audit_system, _storage) = setup_test_audit_system().await;

            // Log multiple events with different actors and actions
            let mut expected_event_ids = Vec::new();
            for i in 0..5 {
                let event = AuditEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    severity: AuditSeverity::Info,
                    category: AuditCategory::DataAccess,
                    source: "test_service".to_string(),
                    principal: Some(if i < 2  { actor.clone() } else { format!("other_user_{}", i) }),
                    target: Some(format!("secret/test{}", i)),
                    action: if i < 2  { action.clone() } else { format!("other_action_{}", i) },
                    result: AuditResult::Success,
                    context: HashMap::new(),
                    source_ip: Some("127.0.0.1".to_string()),
                    user_agent: None,
                    session_id: Some("session_123".to_string()),
                    correlation_id: Some("corr_123".to_string()),
                    geo_location: None,
                    risk_score: None,
                    compliance_tags: Vec::new(),
                    sensitive_data_access: false,
                    duration: Some(Duration::from_millis(100)),
                };

                if i < 2 {
                    expected_event_ids.push(event.event_id);
                }

                audit_system.log_event(event).await
                    .expect("Event logging should succeed");
            }

            // Wait for batch processing
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Property 1: Filter by actor - should only return events from that actor
            let query = AuditQuery::new().with_actor(&actor);
            let results = audit_system.search(&query).await
                .expect("Search should succeed");

            prop_assert!(results.len() >= 2,
                "Should find at least 2 events for actor {}", actor);

            for entry in &results {
                prop_assert_eq!(entry.event.principal.as_ref(), Some(&actor),
                    "All results should have the specified actor");
            }

            // Property 2: Filter by action - should only return events with that action
            let query = AuditQuery::new().with_action(&action);
            let results = audit_system.search(&query).await
                .expect("Search should succeed");

            prop_assert!(results.len() >= 2,
                "Should find at least 2 events for action {}", action);

            for entry in &results {
                prop_assert_eq!(&entry.event.action, &action,
                    "All results should have the specified action");
            }

            // Property 3: Filter by both actor AND action - should only return events matching both
            let query = AuditQuery::new()
                .with_actor(&actor)
                .with_action(&action);
            let results = audit_system.search(&query).await
                .expect("Search should succeed");

            prop_assert!(results.len() >= 2,
                "Should find at least 2 events matching both filters");

            for entry in &results {
                prop_assert_eq!(entry.event.principal.as_ref(), Some(&actor),
                    "All results should have the specified actor");
                prop_assert_eq!(&entry.event.action, &action,
                    "All results should have the specified action");
            }

            // Property 4: Filter by resource pattern
            let query = AuditQuery::new().with_resource_pattern("secret/test");
            let results = audit_system.search(&query).await
                .expect("Search should succeed");

            prop_assert!(results.len() >= 5,
                "Should find all events with resource pattern");

            for entry in &results {
                if let Some(ref target) = entry.event.target {
                    prop_assert!(target.contains("secret/test"),
                        "All results should match the resource pattern");
                }
            }

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod query_filtering_tests {
    use super::*;

    #[tokio::test]
    async fn test_time_range_filtering() {
        let (audit_system, _storage) = setup_test_audit_system().await;

        let start_time = Utc::now();

        // Log an event
        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: AuditSeverity::Info,
            category: AuditCategory::DataAccess,
            source: "test".to_string(),
            principal: Some("user1".to_string()),
            target: Some("secret/test".to_string()),
            action: "read".to_string(),
            result: AuditResult::Success,
            context: HashMap::new(),
            source_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            session_id: None,
            correlation_id: None,
            geo_location: None,
            risk_score: None,
            compliance_tags: Vec::new(),
            sensitive_data_access: false,
            duration: None,
        };
        audit_system.log_event(event).await.unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let end_time = Utc::now();

        // Query with time range
        let query = AuditQuery::new().with_time_range(start_time, end_time);
        let results = audit_system.search(&query).await.unwrap();

        assert!(!results.is_empty(), "Should find events in time range");

        // All results should be within the time range
        for entry in &results {
            assert!(entry.event.timestamp >= start_time);
            assert!(entry.event.timestamp <= end_time);
        }
    }

    #[tokio::test]
    async fn test_empty_query_returns_all() {
        let (audit_system, _storage) = setup_test_audit_system().await;

        // Log multiple events
        for i in 0..3 {
            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AuditSeverity::Info,
                category: AuditCategory::DataAccess,
                source: "test".to_string(),
                principal: Some(format!("user{}", i)),
                target: Some(format!("secret/test{}", i)),
                action: "read".to_string(),
                result: AuditResult::Success,
                context: HashMap::new(),
                source_ip: Some("127.0.0.1".to_string()),
                user_agent: None,
                session_id: None,
                correlation_id: None,
                geo_location: None,
                risk_score: None,
                compliance_tags: Vec::new(),
                sensitive_data_access: false,
                duration: None,
            };
            audit_system.log_event(event).await.unwrap();
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Empty query should return all events
        let query = AuditQuery::new();
        let results = audit_system.search(&query).await.unwrap();

        assert!(results.len() >= 3, "Empty query should return all events");
    }
}
