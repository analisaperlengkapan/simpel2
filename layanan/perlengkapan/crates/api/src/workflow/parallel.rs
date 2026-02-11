// ============================================================================
// Parallel Approval Module
// Description: Parallel approval engine for multi-approver workflows
// Requirements: REQ-W007
// ============================================================================

use crate::workflow::engine::{WorkflowEngine, TransitionRequest};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parallel approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateParallelApprovalRequest {
    /// Entity ID (e.g., kebutuhan_bmn.id)
    pub entity_id: Uuid,

    /// List of approver user IDs
    pub approvers: Vec<Uuid>,

    /// Number of approvals required to proceed
    pub required_approvals: usize,

    /// Current state of the entity
    pub current_state: String,

    /// Target state after approval threshold is reached
    pub target_state: String,
}

/// Parallel approval record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelApproval {
    /// Approval ID
    pub id: Uuid,

    /// Entity ID
    pub entity_id: Uuid,

    /// List of approver user IDs
    pub approvers: Vec<Uuid>,

    /// Number of approvals required
    pub required_approvals: usize,

    /// Current number of approvals
    pub current_approvals: usize,

    /// Current state
    pub current_state: String,

    /// Target state after threshold
    pub target_state: String,

    /// Approval status
    pub status: ParallelApprovalStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Parallel approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParallelApprovalStatus {
    /// Waiting for approvals
    Pending,

    /// Threshold reached, approved
    Approved,

    /// Rejected by one or more approvers
    Rejected,

    /// Cancelled
    Cancelled,
}

/// Approval vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalVote {
    /// Vote ID
    pub id: Uuid,

    /// Approval ID
    pub approval_id: Uuid,

    /// Approver user ID
    pub approver_id: Uuid,

    /// Whether approved (true) or rejected (false)
    pub approved: bool,

    /// Optional notes/comments
    pub catatan: Option<String>,

    /// Vote timestamp
    pub created_at: DateTime<Utc>,
}

/// Record approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordApprovalRequest {
    /// Approval ID
    pub approval_id: Uuid,

    /// Approver user ID
    pub approver_id: Uuid,

    /// Whether approved (true) or rejected (false)
    pub approved: bool,

    /// Optional notes/comments
    pub catatan: Option<String>,

    /// IP address of the approver
    pub ip_address: String,
}

/// Parallel approval engine
pub struct ParallelApprovalEngine {
    /// Database connection pool
    db_pool: Pool,

    /// Workflow engine for automatic transitions
    workflow_engine: WorkflowEngine,
}

impl ParallelApprovalEngine {
    /// Create a new parallel approval engine
    pub fn new(db_pool: Pool, workflow_engine: WorkflowEngine) -> Self {
        Self {
            db_pool,
            workflow_engine,
        }
    }

    /// Create a new parallel approval
    ///
    /// Requirements: REQ-W007
    pub async fn create_parallel_approval(
        &self,
        request: CreateParallelApprovalRequest,
    ) -> Result<ParallelApproval, ParallelApprovalError> {
        // Validate request
        if request.approvers.is_empty() {
            return Err(ParallelApprovalError::InvalidRequest(
                "Approvers list cannot be empty".to_string(),
            ));
        }

        if request.required_approvals == 0 {
            return Err(ParallelApprovalError::InvalidRequest(
                "Required approvals must be greater than 0".to_string(),
            ));
        }

        if request.required_approvals > request.approvers.len() {
            return Err(ParallelApprovalError::InvalidRequest(
                "Required approvals cannot exceed number of approvers".to_string(),
            ));
        }

        let approval_id = Uuid::new_v4();
        let client = self.db_pool.get().await?;

        let query = r#"
            INSERT INTO perlengkapan.parallel_approvals
            (id, entity_id, approvers, required_approvals, current_approvals,
             current_state, target_state, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 0, $5, $6, 'PENDING', NOW(), NOW())
            RETURNING created_at, updated_at
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &approval_id,
                    &request.entity_id,
                    &request.approvers,
                    &(request.required_approvals as i32),
                    &request.current_state,
                    &request.target_state,
                ],
            )
            .await?;

        Ok(ParallelApproval {
            id: approval_id,
            entity_id: request.entity_id,
            approvers: request.approvers,
            required_approvals: request.required_approvals,
            current_approvals: 0,
            current_state: request.current_state,
            target_state: request.target_state,
            status: ParallelApprovalStatus::Pending,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Record an approval vote
    ///
    /// This method records the vote and checks if the threshold has been reached.
    /// If the threshold is reached, it automatically transitions the entity to the target state.
    ///
    /// Requirements: REQ-W007
    pub async fn record_approval(
        &self,
        request: RecordApprovalRequest,
    ) -> Result<ApprovalVote, ParallelApprovalError> {
        let mut client = self.db_pool.get().await?;
        let tx = client.transaction().await?;

        // Get approval record
        let approval_query = r#"
            SELECT entity_id, approvers, required_approvals, current_approvals,
                   current_state, target_state, status
            FROM perlengkapan.parallel_approvals
            WHERE id = $1
            FOR UPDATE
        "#;

        let approval_row = tx
            .query_opt(approval_query, &[&request.approval_id])
            .await?
            .ok_or_else(|| ParallelApprovalError::ApprovalNotFound(request.approval_id))?;

        let entity_id: Uuid = approval_row.get("entity_id");
        let approvers: Vec<Uuid> = approval_row.get("approvers");
        let required_approvals: i32 = approval_row.get("required_approvals");
        let current_approvals: i32 = approval_row.get("current_approvals");
        let current_state: String = approval_row.get("current_state");
        let target_state: String = approval_row.get("target_state");
        let status: String = approval_row.get("status");

        // Check if approval is still pending
        if status != "PENDING" {
            return Err(ParallelApprovalError::ApprovalNotPending);
        }

        // Check if approver is in the list
        if !approvers.contains(&request.approver_id) {
            return Err(ParallelApprovalError::ApproverNotAuthorized(
                request.approver_id,
            ));
        }

        // Check if approver has already voted
        let vote_check_query = r#"
            SELECT COUNT(*) as vote_count
            FROM perlengkapan.parallel_approval_votes
            WHERE approval_id = $1 AND approver_id = $2
        "#;

        let vote_count_row = tx
            .query_one(vote_check_query, &[&request.approval_id, &request.approver_id])
            .await?;
        let vote_count: i64 = vote_count_row.get("vote_count");

        if vote_count > 0 {
            return Err(ParallelApprovalError::AlreadyVoted(request.approver_id));
        }

        // Record the vote
        let vote_id = Uuid::new_v4();
        let insert_vote_query = r#"
            INSERT INTO perlengkapan.parallel_approval_votes
            (id, approval_id, approver_id, approved, catatan, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING created_at
        "#;

        let vote_row = tx
            .query_one(
                insert_vote_query,
                &[
                    &vote_id,
                    &request.approval_id,
                    &request.approver_id,
                    &request.approved,
                    &request.catatan,
                ],
            )
            .await?;

        let vote = ApprovalVote {
            id: vote_id,
            approval_id: request.approval_id,
            approver_id: request.approver_id,
            approved: request.approved,
            catatan: request.catatan.clone(),
            created_at: vote_row.get("created_at"),
        };

        // If rejected, update approval status to REJECTED
        if !request.approved {
            let update_query = r#"
                UPDATE perlengkapan.parallel_approvals
                SET status = 'REJECTED', updated_at = NOW()
                WHERE id = $1
            "#;

            tx.execute(update_query, &[&request.approval_id]).await?;

            tx.commit().await?;

            tracing::info!(
                approval_id = %request.approval_id,
                entity_id = %entity_id,
                approver_id = %request.approver_id,
                "Parallel approval rejected"
            );

            return Ok(vote);
        }

        // Count current approvals (only approved votes)
        let count_query = r#"
            SELECT COUNT(*) as approved_count
            FROM perlengkapan.parallel_approval_votes
            WHERE approval_id = $1 AND approved = true
        "#;

        let count_row = tx.query_one(count_query, &[&request.approval_id]).await?;
        let approved_count: i64 = count_row.get("approved_count");

        // Update current_approvals
        let update_count_query = r#"
            UPDATE perlengkapan.parallel_approvals
            SET current_approvals = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        tx.execute(
            update_count_query,
            &[&(approved_count as i32), &request.approval_id],
        )
        .await?;

        // Check if threshold reached
        if approved_count >= required_approvals as i64 {
            // Update approval status to APPROVED
            let update_status_query = r#"
                UPDATE perlengkapan.parallel_approvals
                SET status = 'APPROVED', updated_at = NOW()
                WHERE id = $1
            "#;

            tx.execute(update_status_query, &[&request.approval_id])
                .await?;

            tx.commit().await?;

            tracing::info!(
                approval_id = %request.approval_id,
                entity_id = %entity_id,
                approved_count = %approved_count,
                required_approvals = %required_approvals,
                "Parallel approval threshold reached - transitioning to target state"
            );

            // Transition entity to target state
            // Use a system user ID (nil UUID) for automatic transitions
            let transition_request = TransitionRequest {
                entity_id,
                from_state: current_state,
                to_state: target_state,
                user_id: Uuid::nil(), // System user
                catatan: Some(format!(
                    "Parallel approval threshold reached ({}/{})",
                    approved_count, required_approvals
                )),
                ip_address: "system".to_string(),
            };

            if let Err(e) = self.workflow_engine.transition(transition_request).await {
                tracing::error!(
                    approval_id = %request.approval_id,
                    entity_id = %entity_id,
                    error = %e,
                    "Failed to transition entity after parallel approval threshold reached"
                );
                // Note: The approval is still marked as APPROVED even if transition fails
                // This allows manual intervention if needed
            }
        } else {
            tx.commit().await?;

            tracing::info!(
                approval_id = %request.approval_id,
                entity_id = %entity_id,
                approver_id = %request.approver_id,
                approved_count = %approved_count,
                required_approvals = %required_approvals,
                "Approval recorded - threshold not yet reached"
            );
        }

        Ok(vote)
    }

    /// Get parallel approval by ID
    pub async fn get_approval(
        &self,
        approval_id: Uuid,
    ) -> Result<ParallelApproval, ParallelApprovalError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, entity_id, approvers, required_approvals, current_approvals,
                   current_state, target_state, status, created_at, updated_at
            FROM perlengkapan.parallel_approvals
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&approval_id])
            .await?
            .ok_or_else(|| ParallelApprovalError::ApprovalNotFound(approval_id))?;

        Ok(ParallelApproval {
            id: row.get("id"),
            entity_id: row.get("entity_id"),
            approvers: row.get("approvers"),
            required_approvals: row.get::<_, i32>("required_approvals") as usize,
            current_approvals: row.get::<_, i32>("current_approvals") as usize,
            current_state: row.get("current_state"),
            target_state: row.get("target_state"),
            status: match row.get::<_, String>("status").as_str() {
                "PENDING" => ParallelApprovalStatus::Pending,
                "APPROVED" => ParallelApprovalStatus::Approved,
                "REJECTED" => ParallelApprovalStatus::Rejected,
                "CANCELLED" => ParallelApprovalStatus::Cancelled,
                _ => ParallelApprovalStatus::Pending,
            },
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Get all votes for an approval
    pub async fn get_votes(
        &self,
        approval_id: Uuid,
    ) -> Result<Vec<ApprovalVote>, ParallelApprovalError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, approval_id, approver_id, approved, catatan, created_at
            FROM perlengkapan.parallel_approval_votes
            WHERE approval_id = $1
            ORDER BY created_at ASC
        "#;

        let rows = client.query(query, &[&approval_id]).await?;

        let votes = rows
            .into_iter()
            .map(|row| ApprovalVote {
                id: row.get("id"),
                approval_id: row.get("approval_id"),
                approver_id: row.get("approver_id"),
                approved: row.get("approved"),
                catatan: row.get("catatan"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(votes)
    }

    /// Cancel a parallel approval
    pub async fn cancel_approval(
        &self,
        approval_id: Uuid,
    ) -> Result<(), ParallelApprovalError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.parallel_approvals
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE id = $1 AND status = 'PENDING'
        "#;

        let rows_affected = client.execute(query, &[&approval_id]).await?;

        if rows_affected == 0 {
            return Err(ParallelApprovalError::ApprovalNotPending);
        }

        tracing::info!(
            approval_id = %approval_id,
            "Parallel approval cancelled"
        );

        Ok(())
    }
}

/// Parallel approval errors
#[derive(Debug, thiserror::Error)]
pub enum ParallelApprovalError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Approval not found: {0}")]
    ApprovalNotFound(Uuid),

    #[error("Approval is not in pending status")]
    ApprovalNotPending,

    #[error("Approver not authorized: {0}")]
    ApproverNotAuthorized(Uuid),

    #[error("Approver has already voted: {0}")]
    AlreadyVoted(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),

    #[error("Workflow error: {0}")]
    WorkflowError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_approval_status_serialization() {
        let status = ParallelApprovalStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"PENDING\"");

        let status = ParallelApprovalStatus::Approved;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"APPROVED\"");
    }
}
