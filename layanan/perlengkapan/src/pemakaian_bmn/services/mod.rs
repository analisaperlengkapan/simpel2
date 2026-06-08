//! # Pemakaian BMN Services
//!
//! Business logic layer for BMN usage permit system.
//! Integrates with workflow engine for approval processes.
//!
//! Requirements: REQ-P001 through REQ-P016, REQ-W004, REQ-W005

use std::sync::Arc;

use crate::workflow::engine::WorkflowEngine;

use super::repository::PemakaianBmnRepository;

/// Service for BMN usage permit business logic
#[derive(Clone)]
pub struct PemakaianBmnService {
    repository: Arc<PemakaianBmnRepository>,
    workflow_engine: Arc<WorkflowEngine>,
    notifier: Option<Arc<dyn crate::contracts::NotificationSender>>,
    docs: Option<Arc<dyn crate::contracts::DocumentGenerator>>,
}

impl PemakaianBmnService {
    /// Create a new service instance with workflow engine
    ///
    /// Requirements: REQ-P004
    pub fn new(repository: PemakaianBmnRepository, workflow_engine: WorkflowEngine) -> Self {
        Self {
            repository: Arc::new(repository),
            workflow_engine: Arc::new(workflow_engine),
            notifier: None,
            docs: None,
        }
    }

    /// Inject the notification sender (replaces the deleted gRPC client).
    pub fn with_notification_sender(
        mut self,
        notifier: Arc<dyn crate::contracts::NotificationSender>,
    ) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Inject the document generator (replaces the deleted gRPC client).
    pub fn with_document_generator(
        mut self,
        docs: Arc<dyn crate::contracts::DocumentGenerator>,
    ) -> Self {
        self.docs = Some(docs);
        self
    }
}

mod availability;
mod create_docs;
mod crud;
mod detail_transition;
mod expiry;
mod history;
mod monitoring;
mod revoke_renew;
mod workflow;

#[cfg(test)]
mod tests;
