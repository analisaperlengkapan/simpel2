// ============================================================================
// Penghapusan BMN Services
// Description: Business logic for BMN disposal workflow
// Requirements: REQ-W001, REQ-W004, REQ-D002, REQ-N001
// ============================================================================

use super::models::*;
use super::repository::PenghapusanBmnRepository;
use crate::errors::AppResult;
use crate::workflow::engine::{TransitionRequest, WorkflowEngine};
use deadpool_postgres::Pool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PenghapusanBmnService {
    repository: PenghapusanBmnRepository,
    workflow_engine: Arc<WorkflowEngine>,
}

impl PenghapusanBmnService {
    pub fn new(pool: Pool, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self {
            repository: PenghapusanBmnRepository::new(pool),
            workflow_engine,
        }
    }

    /// Create a new penghapusan BMN record
    pub async fn create(
        &self,
        request: CreatePenghapusanBmnRequest,
        created_by: Uuid,
    ) -> AppResult<PenghapusanBmn> {
        self.repository.create(request, created_by).await
    }

    /// Get penghapusan BMN by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<PenghapusanBmn> {
        self.repository.get_by_id(id).await
    }

    /// List penghapusan BMN with filters and pagination
    pub async fn list(
        &self,
        filters: PenghapusanBmnFilters,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PenghapusanBmn>, i64)> {
        self.repository.list(filters, page, per_page).await
    }

    /// Update penghapusan BMN
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
    ) -> AppResult<PenghapusanBmn> {
        self.repository.update(id, request).await
    }

    /// Delete penghapusan BMN
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repository.delete(id).await
    }

    /// Perform workflow transition
    pub async fn transition(
        &self,
        id: Uuid,
        to_state: String,
        user_id: Uuid,
        catatan: Option<String>,
        ip_address: String,
    ) -> AppResult<PenghapusanBmn> {
        // Get current penghapusan
        let penghapusan = self.repository.get_by_id(id).await?;

        // Perform workflow transition
        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: penghapusan.status.clone(),
            to_state: to_state.clone(),
            user_id,
            catatan,
            ip_address,
        };

        self.workflow_engine.transition(transition_request).await
            .map_err(|e| crate::errors::AppError::WorkflowError(e.to_string()))?;

        // Update status in database
        self.repository.update_status(id, &to_state).await?;

        // Get updated penghapusan
        self.repository.get_by_id(id).await
    }
}
