//! Unit tests for audit logging module

#[allow(unused_imports)]
use lib_common::audit::{AuditEvent, AuditLogger};
use uuid::Uuid;

#[cfg(test)]
mod audit_tests {
    use super::*;

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::UserCreated {
            user_id: Uuid::new_v4(),
            username: "test_user".to_string(),
            created_by: Uuid::new_v4(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("test_user"));
    }

    #[test]
    fn test_audit_event_types() {
        let user_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();

        let events = vec![
            AuditEvent::UserCreated {
                user_id,
                username: "test".to_string(),
                created_by: actor_id,
            },
            AuditEvent::UserUpdated {
                user_id,
                username: "test".to_string(),
                updated_by: actor_id,
                changes: vec!["email".to_string()],
            },
            AuditEvent::WorkflowTransition {
                entity_id: Uuid::new_v4(),
                entity_type: "document".to_string(),
                from_state: "DRAFT".to_string(),
                to_state: "SUBMITTED".to_string(),
                user_id,
            },
        ];

        for event in events {
            let serialized = serde_json::to_string(&event);
            assert!(serialized.is_ok());
        }
    }

    #[test]
    fn test_batch_operation_event() {
        let event = AuditEvent::BatchOperation {
            batch_id: Uuid::new_v4(),
            operation: "bulk_approve".to_string(),
            count: 50,
            success_count: 48,
            failure_count: 2,
            user_id: Uuid::new_v4(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("bulk_approve"));
        assert!(serialized.contains("50"));
    }
}
