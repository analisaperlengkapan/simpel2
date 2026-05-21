// ============================================================================
// Delegation Module
// Description: Workflow delegation management for temporary role assignment
// Requirements: REQ-W006
// ============================================================================

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Delegation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDelegationRequest {
    /// User who is delegating (delegator)
    pub delegator_id: Uuid,

    /// User who receives the delegation (delegate)
    pub delegate_id: Uuid,

    /// Role being delegated
    pub role: String,

    /// Start date of delegation
    pub start_date: DateTime<Utc>,

    /// End date of delegation
    pub end_date: DateTime<Utc>,

    /// Reason for delegation
    pub reason: Option<String>,
}

/// Delegation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Delegation ID
    pub id: Uuid,

    /// User who is delegating (delegator)
    pub delegator_id: Uuid,

    /// User who receives the delegation (delegate)
    pub delegate_id: Uuid,

    /// Role being delegated
    pub role: String,

    /// Start date of delegation
    pub start_date: DateTime<Utc>,

    /// End date of delegation
    pub end_date: DateTime<Utc>,

    /// Reason for delegation
    pub reason: Option<String>,

    /// Delegation status
    pub status: DelegationStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Delegation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DelegationStatus {
    /// Delegation is active
    Active,

    /// Delegation has expired
    Expired,

    /// Delegation was revoked
    Revoked,

    /// Delegation is scheduled (not yet active)
    Scheduled,
}

/// Delegation manager
pub struct DelegationManager {
    /// Database connection pool
    db_pool: Pool,
}

impl DelegationManager {
    /// Create a new delegation manager
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    /// Create a new delegation
    ///
    /// Requirements: REQ-W006
    pub async fn create_delegation(
        &self,
        request: CreateDelegationRequest,
    ) -> Result<Delegation, DelegationError> {
        // Validate request
        if request.delegator_id == request.delegate_id {
            return Err(DelegationError::InvalidRequest(
                "Cannot delegate to yourself".to_string(),
            ));
        }

        if request.end_date <= request.start_date {
            return Err(DelegationError::InvalidRequest(
                "End date must be after start date".to_string(),
            ));
        }

        if request.end_date <= Utc::now() {
            return Err(DelegationError::InvalidRequest(
                "End date must be in the future".to_string(),
            ));
        }

        let delegation_id = Uuid::new_v4();
        let client = self.db_pool.get().await?;

        // Determine initial status
        let now = Utc::now();
        let status = if request.start_date > now {
            DelegationStatus::Scheduled
        } else {
            DelegationStatus::Active
        };

        let query = r#"
            INSERT INTO perlengkapan.delegations
            (id, delegator_id, delegate_id, role, start_date, end_date,
             reason, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING created_at, updated_at
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &delegation_id,
                    &request.delegator_id,
                    &request.delegate_id,
                    &request.role,
                    &request.start_date,
                    &request.end_date,
                    &request.reason,
                    &format!("{:?}", status).to_uppercase(),
                ],
            )
            .await?;

        tracing::info!(
            delegation_id = %delegation_id,
            delegator_id = %request.delegator_id,
            delegate_id = %request.delegate_id,
            role = %request.role,
            start_date = %request.start_date,
            end_date = %request.end_date,
            status = ?status,
            "Delegation created"
        );

        Ok(Delegation {
            id: delegation_id,
            delegator_id: request.delegator_id,
            delegate_id: request.delegate_id,
            role: request.role,
            start_date: request.start_date,
            end_date: request.end_date,
            reason: request.reason,
            status,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Get delegation by ID
    pub async fn get_delegation(&self, delegation_id: Uuid) -> Result<Delegation, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, delegator_id, delegate_id, role, start_date, end_date,
                   reason, status, created_at, updated_at
            FROM perlengkapan.delegations
            WHERE id = $1
        "#;

        let row = client
            .query_opt(query, &[&delegation_id])
            .await?
            .ok_or_else(|| DelegationError::DelegationNotFound(delegation_id))?;

        Ok(self.row_to_delegation(row))
    }

    /// Get active delegations for a user (as delegate)
    ///
    /// Returns all active delegations where the user is the delegate
    pub async fn get_active_delegations_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Delegation>, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, delegator_id, delegate_id, role, start_date, end_date,
                   reason, status, created_at, updated_at
            FROM perlengkapan.delegations
            WHERE delegate_id = $1
              AND status = 'ACTIVE'
              AND start_date <= NOW()
              AND end_date > NOW()
            ORDER BY created_at DESC
        "#;

        let rows = client.query(query, &[&user_id]).await?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_delegation(row))
            .collect())
    }

    /// Get all delegations created by a user (as delegator)
    pub async fn get_delegations_by_delegator(
        &self,
        delegator_id: Uuid,
    ) -> Result<Vec<Delegation>, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, delegator_id, delegate_id, role, start_date, end_date,
                   reason, status, created_at, updated_at
            FROM perlengkapan.delegations
            WHERE delegator_id = $1
            ORDER BY created_at DESC
        "#;

        let rows = client.query(query, &[&delegator_id]).await?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_delegation(row))
            .collect())
    }

    /// Check if a user has a specific role through delegation
    ///
    /// Returns true if the user has an active delegation for the specified role
    pub async fn has_delegated_role(
        &self,
        user_id: Uuid,
        role: &str,
    ) -> Result<bool, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT COUNT(*) as count
            FROM perlengkapan.delegations
            WHERE delegate_id = $1
              AND role = $2
              AND status = 'ACTIVE'
              AND start_date <= NOW()
              AND end_date > NOW()
        "#;

        let row = client.query_one(query, &[&user_id, &role]).await?;
        let count: i64 = row.get("count");

        Ok(count > 0)
    }

    /// Revoke a delegation
    ///
    /// Requirements: REQ-W006
    pub async fn revoke_delegation(
        &self,
        delegation_id: Uuid,
        revoked_by: Uuid,
    ) -> Result<(), DelegationError> {
        let client = self.db_pool.get().await?;

        // Verify delegation exists and is active
        let check_query = r#"
            SELECT delegator_id, status
            FROM perlengkapan.delegations
            WHERE id = $1
        "#;

        let row = client
            .query_opt(check_query, &[&delegation_id])
            .await?
            .ok_or_else(|| DelegationError::DelegationNotFound(delegation_id))?;

        let delegator_id: Uuid = row.get("delegator_id");
        let status: String = row.get("status");

        // Only the delegator can revoke
        if delegator_id != revoked_by {
            return Err(DelegationError::Unauthorized(
                "Only the delegator can revoke the delegation".to_string(),
            ));
        }

        // Can only revoke active or scheduled delegations
        if status != "ACTIVE" && status != "SCHEDULED" {
            return Err(DelegationError::InvalidStatus(
                "Can only revoke active or scheduled delegations".to_string(),
            ));
        }

        // Revoke the delegation
        let update_query = r#"
            UPDATE perlengkapan.delegations
            SET status = 'REVOKED', updated_at = NOW()
            WHERE id = $1
        "#;

        client.execute(update_query, &[&delegation_id]).await?;

        tracing::info!(
            delegation_id = %delegation_id,
            revoked_by = %revoked_by,
            "Delegation revoked"
        );

        Ok(())
    }

    /// Expire delegations that have passed their end date
    ///
    /// This method should be called periodically (e.g., every hour) by a scheduler
    ///
    /// Requirements: REQ-W006
    pub async fn expire_delegations(&self) -> Result<usize, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.delegations
            SET status = 'EXPIRED', updated_at = NOW()
            WHERE status IN ('ACTIVE', 'SCHEDULED')
              AND end_date <= NOW()
        "#;

        let rows_affected = client.execute(query, &[]).await? as usize;

        if rows_affected > 0 {
            tracing::info!(
                expired_count = %rows_affected,
                "Delegations expired"
            );
        }

        Ok(rows_affected)
    }

    /// Activate scheduled delegations that have reached their start date
    ///
    /// This method should be called periodically (e.g., every hour) by a scheduler
    pub async fn activate_scheduled_delegations(&self) -> Result<usize, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            UPDATE perlengkapan.delegations
            SET status = 'ACTIVE', updated_at = NOW()
            WHERE status = 'SCHEDULED'
              AND start_date <= NOW()
              AND end_date > NOW()
        "#;

        let rows_affected = client.execute(query, &[]).await? as usize;

        if rows_affected > 0 {
            tracing::info!(
                activated_count = %rows_affected,
                "Scheduled delegations activated"
            );
        }

        Ok(rows_affected)
    }

    /// Process delegation lifecycle (expire and activate)
    ///
    /// This is a convenience method that calls both expire_delegations and activate_scheduled_delegations
    pub async fn process_delegation_lifecycle(&self) -> Result<(usize, usize), DelegationError> {
        let expired = self.expire_delegations().await?;
        let activated = self.activate_scheduled_delegations().await?;

        Ok((expired, activated))
    }

    /// Helper method to convert a database row to a Delegation
    fn row_to_delegation(&self, row: tokio_postgres::Row) -> Delegation {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "ACTIVE" => DelegationStatus::Active,
            "EXPIRED" => DelegationStatus::Expired,
            "REVOKED" => DelegationStatus::Revoked,
            "SCHEDULED" => DelegationStatus::Scheduled,
            _ => DelegationStatus::Active,
        };

        Delegation {
            id: row.get("id"),
            delegator_id: row.get("delegator_id"),
            delegate_id: row.get("delegate_id"),
            role: row.get("role"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            reason: row.get("reason"),
            status,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Delegation errors
#[derive(Debug, thiserror::Error)]
pub enum DelegationError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Delegation not found: {0}")]
    DelegationNotFound(Uuid),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_status_serialization() {
        let status = DelegationStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"ACTIVE\"");

        let status = DelegationStatus::Expired;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"EXPIRED\"");
    }

    #[test]
    fn test_create_delegation_request_validation() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let now = Utc::now();
        let future = now + chrono::Duration::days(7);

        let request = CreateDelegationRequest {
            delegator_id: user1,
            delegate_id: user2,
            role: "admin_pusat".to_string(),
            start_date: now,
            end_date: future,
            reason: Some("Vacation".to_string()),
        };

        assert_ne!(request.delegator_id, request.delegate_id);
        assert!(request.end_date > request.start_date);
    }
}
