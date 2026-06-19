// ============================================================================
// Workflow Engine Module
// Description: Core workflow engine for state transitions and approval validation
// Requirements: REQ-W004, REQ-W005
// ============================================================================

use crate::workflow::config::WorkflowConfig;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Workflow engine error types
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Insufficient permissions: required role {required_role}")]
    InsufficientPermissions { required_role: String },

    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),

    #[error("Authenc client error: {0}")]
    AuthencError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("Audit error: {0}")]
    AuditError(String),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Workflow transition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    /// Entity ID (e.g., kebutuhan_bmn.id)
    pub entity_id: Uuid,

    /// Current state
    pub from_state: String,

    /// Target state
    pub to_state: String,

    /// User performing the transition
    pub user_id: Uuid,

    /// Role of the user (from JWT claims)
    pub user_role: String,

    /// Optional notes/comments
    pub catatan: Option<String>,

    /// IP address of the user
    pub ip_address: String,
}

/// Workflow transition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    /// Entity ID
    pub entity_id: Uuid,

    /// New state
    pub new_state: String,

    /// Timestamp of transition
    pub transitioned_at: DateTime<Utc>,

    /// Activity record ID
    pub activity_id: Uuid,
}

/// Core workflow engine
pub struct WorkflowEngine {
    /// Workflow configuration
    config: WorkflowConfig,

    /// Database connection pool
    db_pool: Pool,

    /// Document generator (port). Optional because document generation is
    /// only meaningful for entity types that produce SK/surat after approval.
    docs: Option<Arc<dyn crate::contracts::DocumentGenerator>>,

    /// Notification sender (port).
    notifier: Option<Arc<dyn crate::contracts::NotificationSender>>,

    /// Audit sink (port). Optional — dev/test boleh tanpa, produksi wajib
    /// terinjeksi agar setiap transisi tercatat di `perlengkapan.audit_log`
    /// (BPK-ready).
    audit_sink: Option<Arc<dyn crate::contracts::AuditSink>>,

    /// V1.4: in-process event bus. Optional — bila ter-inject, setiap
    /// transisi sukses mem-publish `DomainEvent::WorkflowTransitioned`
    /// agar subscriber lain (real-time monitoring, search index, dst)
    /// tap-in tanpa men-tightly-couple engine.
    event_bus: Option<Arc<crate::shared::events::EventBus>>,
}

pub mod documents;
pub mod notifications;
pub mod transition;

impl WorkflowEngine {
    /// Create a new workflow engine with the given configuration
    pub fn new(config: WorkflowConfig, db_pool: Pool) -> Self {
        Self {
            config,
            db_pool,
            docs: None,
            notifier: None,
            audit_sink: None,
            event_bus: None,
        }
    }
    /// Inject a document generator (replaces the deleted gRPC client).
    pub fn with_document_generator(
        mut self,
        docs: Arc<dyn crate::contracts::DocumentGenerator>,
    ) -> Self {
        self.docs = Some(docs);
        self
    }
    /// Inject a notification sender (replaces the deleted gRPC client).
    pub fn with_notification_sender(
        mut self,
        notifier: Arc<dyn crate::contracts::NotificationSender>,
    ) -> Self {
        self.notifier = Some(notifier);
        self
    }
    /// Inject the audit sink (typically `PgAuditSink`). Setiap transisi
    /// sukses akan mem-publish AuditEvent ke sink ini.
    pub fn with_audit_sink(mut self, audit_sink: Arc<dyn crate::contracts::AuditSink>) -> Self {
        self.audit_sink = Some(audit_sink);
        self
    }
    /// V1.4: inject in-process event bus. Subscriber lain (notifier
    /// dispatcher, real-time monitoring, dst) dapat tap-in via
    /// `shared::events::spawn_subscriber`. Audit sink lama tetap valid
    /// secara paralel utk back-compat.
    pub fn with_event_bus(mut self, bus: Arc<crate::shared::events::EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }
    /// Create a workflow engine for Kebutuhan BMN
    pub fn for_kebutuhan_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_kebutuhan_bmn(), db_pool)
    }
    /// Create a workflow engine for Pemakaian BMN
    pub fn for_pemakaian_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_pemakaian_bmn(), db_pool)
    }
    /// Create a workflow engine for Penghapusan BMN
    pub fn for_penghapusan_bmn(db_pool: Pool) -> Self {
        Self::new(WorkflowConfig::default_penghapusan_bmn(), db_pool)
    }
    /// Get the current state of an entity
    pub async fn get_current_state(&self, entity_id: Uuid) -> Result<String> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT status
            FROM perlengkapan.kebutuhan_bmn
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&entity_id])
            .await?
            .ok_or_else(|| WorkflowError::EntityNotFound(entity_id))?;

        Ok(row.get("status"))
    }
    /// Get all valid next states from the current state
    pub fn get_next_states(&self, current_state: &str) -> Vec<String> {
        self.config.get_next_states(current_state)
    }
    /// Check if a state is terminal (no outgoing transitions)
    pub fn is_terminal_state(&self, state: &str) -> bool {
        self.config.is_terminal_state(state)
    }
    /// Get the workflow configuration
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }
    /// Compute an RFC3339 deadline for a target state using the config's
    /// `sla_minutes` mapping. Returns None if no SLA is configured for that
    /// state — the notification then omits a deadline rather than inventing
    /// one. Weekend/holiday-aware business-hour math can be layered on top
    /// later; for now we use wall-clock minutes, which matches how the UI
    /// renders countdowns today.
    pub(crate) fn compute_sla_deadline(&self, target_state: &str) -> Option<String> {
        let minutes = self.config.sla_minutes.get(target_state).copied()?;
        if minutes == 0 {
            return None;
        }
        let deadline = Utc::now() + chrono::Duration::minutes(minutes as i64);
        Some(deadline.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_transition() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        // Valid transition
        assert!(engine.validate_transition("DRAFT", "INPUT_BARANG").is_ok());

        // Invalid transition
        assert!(engine.validate_transition("DRAFT", "APPROVED").is_err());
    }

    #[test]
    fn test_get_next_states() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        let next_states = engine.get_next_states("DRAFT");
        assert_eq!(next_states.len(), 2);
        assert!(next_states.contains(&"INPUT_BARANG".to_string()));
        assert!(next_states.contains(&"CANCELLED".to_string()));
    }

    #[test]
    fn test_is_terminal_state() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let engine = WorkflowEngine::new(config, pool);

        assert!(engine.is_terminal_state("REJECTED"));
        assert!(engine.is_terminal_state("CANCELLED"));
        assert!(!engine.is_terminal_state("DRAFT"));
    }
}
