//! Replication operation types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an operation to be replicated to secondary nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationOperation {
    /// Unique identifier for this operation
    pub id: Uuid,

    /// Timestamp when the operation was created
    pub timestamp: DateTime<Utc>,

    /// The type of operation
    pub operation_type: OperationType,

    /// The data associated with this operation
    pub data: OperationData,

    /// Sequence number for ordering
    pub sequence: u64,
}

/// Type of replication operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Secret created or updated
    SecretWrite,

    /// Secret deleted
    SecretDelete,

    /// Policy created or updated
    PolicyWrite,

    /// Policy deleted
    PolicyDelete,

    /// Configuration updated
    ConfigUpdate,

    /// Token created (DR mode only)
    TokenCreate,

    /// Token revoked (DR mode only)
    TokenRevoke,

    /// Lease created (DR mode only)
    LeaseCreate,

    /// Lease renewed (DR mode only)
    LeaseRenew,

    /// Lease revoked (DR mode only)
    LeaseRevoke,

    /// Audit log entry (DR mode only)
    AuditLog,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::SecretWrite => write!(f, "secret_write"),
            OperationType::SecretDelete => write!(f, "secret_delete"),
            OperationType::PolicyWrite => write!(f, "policy_write"),
            OperationType::PolicyDelete => write!(f, "policy_delete"),
            OperationType::ConfigUpdate => write!(f, "config_update"),
            OperationType::TokenCreate => write!(f, "token_create"),
            OperationType::TokenRevoke => write!(f, "token_revoke"),
            OperationType::LeaseCreate => write!(f, "lease_create"),
            OperationType::LeaseRenew => write!(f, "lease_renew"),
            OperationType::LeaseRevoke => write!(f, "lease_revoke"),
            OperationType::AuditLog => write!(f, "audit_log"),
        }
    }
}

/// Data payload for a replication operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum OperationData {
    /// Secret data
    Secret {
        path: String,
        data: Vec<u8>,
        metadata: serde_json::Value,
    },

    /// Secret deletion
    SecretDeletion {
        path: String,
    },

    /// Policy data
    Policy {
        name: String,
        policy: String,
    },

    /// Policy deletion
    PolicyDeletion {
        name: String,
    },

    /// Configuration update
    Config {
        key: String,
        value: serde_json::Value,
    },

    /// Token data (DR mode only)
    Token {
        token_id: String,
        data: Vec<u8>,
    },

    /// Token revocation (DR mode only)
    TokenRevocation {
        token_id: String,
    },

    /// Lease data (DR mode only)
    Lease {
        lease_id: String,
        data: Vec<u8>,
    },

    /// Lease renewal (DR mode only)
    LeaseRenewal {
        lease_id: String,
        increment: i64,
    },

    /// Lease revocation (DR mode only)
    LeaseRevocation {
        lease_id: String,
    },

    /// Audit log entry (DR mode only)
    Audit {
        entry: Vec<u8>,
    },
}

impl ReplicationOperation {
    /// Create a new replication operation
    pub fn new(operation_type: OperationType, data: OperationData, sequence: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            operation_type,
            data,
            sequence,
        }
    }

    /// Check if this operation should be replicated in Performance mode
    pub fn is_performance_replicable(&self) -> bool {
        matches!(
            self.operation_type,
            OperationType::SecretWrite
                | OperationType::SecretDelete
                | OperationType::PolicyWrite
                | OperationType::PolicyDelete
                | OperationType::ConfigUpdate
        )
    }

    /// Check if this operation requires DR mode
    pub fn requires_dr_mode(&self) -> bool {
        matches!(
            self.operation_type,
            OperationType::TokenCreate
                | OperationType::TokenRevoke
                | OperationType::LeaseCreate
                | OperationType::LeaseRenew
                | OperationType::LeaseRevoke
                | OperationType::AuditLog
        )
    }

    /// Get a human-readable description of this operation
    pub fn description(&self) -> String {
        match &self.data {
            OperationData::Secret { path, .. } => {
                format!("Write secret at {}", path)
            }
            OperationData::SecretDeletion { path } => {
                format!("Delete secret at {}", path)
            }
            OperationData::Policy { name, .. } => {
                format!("Write policy {}", name)
            }
            OperationData::PolicyDeletion { name } => {
                format!("Delete policy {}", name)
            }
            OperationData::Config { key, .. } => {
                format!("Update config {}", key)
            }
            OperationData::Token { token_id, .. } => {
                format!("Create token {}", token_id)
            }
            OperationData::TokenRevocation { token_id } => {
                format!("Revoke token {}", token_id)
            }
            OperationData::Lease { lease_id, .. } => {
                format!("Create lease {}", lease_id)
            }
            OperationData::LeaseRenewal { lease_id, .. } => {
                format!("Renew lease {}", lease_id)
            }
            OperationData::LeaseRevocation { lease_id } => {
                format!("Revoke lease {}", lease_id)
            }
            OperationData::Audit { .. } => {
                "Audit log entry".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_creation() {
        let op = ReplicationOperation::new(
            OperationType::SecretWrite,
            OperationData::Secret {
                path: "/secret/test".to_string(),
                data: vec![1, 2, 3],
                metadata: serde_json::json!({}),
            },
            1,
        );

        assert_eq!(op.sequence, 1);
        assert_eq!(op.operation_type, OperationType::SecretWrite);
        assert!(op.is_performance_replicable());
        assert!(!op.requires_dr_mode());
    }

    #[test]
    fn test_performance_replicable() {
        let secret_op = ReplicationOperation::new(
            OperationType::SecretWrite,
            OperationData::Secret {
                path: "/test".to_string(),
                data: vec![],
                metadata: serde_json::json!({}),
            },
            1,
        );
        assert!(secret_op.is_performance_replicable());

        let token_op = ReplicationOperation::new(
            OperationType::TokenCreate,
            OperationData::Token {
                token_id: "token-123".to_string(),
                data: vec![],
            },
            2,
        );
        assert!(!token_op.is_performance_replicable());
        assert!(token_op.requires_dr_mode());
    }

    #[test]
    fn test_operation_description() {
        let op = ReplicationOperation::new(
            OperationType::SecretWrite,
            OperationData::Secret {
                path: "/secret/db/password".to_string(),
                data: vec![],
                metadata: serde_json::json!({}),
            },
            1,
        );

        assert_eq!(op.description(), "Write secret at /secret/db/password");
    }
}
