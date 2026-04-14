//! Unit tests for audit logging module

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
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("test_user"));
    }

    #[test]
    fn test_audit_event_types() {
        let user_id = Uuid::new_v4();

        let events = vec![
            AuditEvent::UserCreated {
                user_id,
                username: "test".to_string(),
            },
            AuditEvent::UserUpdated {
                user_id,
                changes: vec!["email".to_string()],
            },
            AuditEvent::WorkflowTransition {
                entity_id: Uuid::new_v4(),
                from_state: "DRAFT".to_string(),
                to_state: "SUBMITTED".to_string(),
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
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("bulk_approve"));
        assert!(serialized.contains("50"));
    }
}
