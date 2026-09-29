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
    pub delegator_user_id: Uuid,

    /// User who receives the delegation (delegate)
    pub delegate_user_id: Uuid,

    /// Role being delegated
    pub role: String,

    /// Start date of delegation
    pub valid_from: DateTime<Utc>,

    /// End date of delegation
    pub valid_until: DateTime<Utc>,

    /// Reason for delegation
    pub reason: Option<String>,
}

/// Delegation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Delegation ID
    pub id: Uuid,

    /// User who is delegating (delegator)
    pub delegator_user_id: Uuid,

    /// User who receives the delegation (delegate)
    pub delegate_user_id: Uuid,

    /// Role being delegated
    pub role: String,

    /// Start date of delegation
    pub valid_from: DateTime<Utc>,

    /// End date of delegation
    pub valid_until: DateTime<Utc>,

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

/// Longest a delegation may run. A delegation is for a leave or a business trip,
/// not a permanent re-assignment of authority.
pub const MAX_DELEGATION_DAYS: i64 = 30;
/// Shortest written reason accepted, in characters (after trimming).
pub const MIN_DELEGATION_REASON_CHARS: usize = 10;

/// The checks on a delegation that need no database: whether the delegator may
/// give this role away at all, and whether the window is sane.
///
/// * The delegator must **hold** the role. Before, the role was free text the
///   caller typed, so anyone could "delegate" `validator_pusat`.
/// * Only a known business role may be delegated, and never an administrator
///   role — authority over the system is not something to hand round.
/// * No future-dated start: the table's status check has no `SCHEDULED` value,
///   so a future `valid_from` used to fail the INSERT with a 500. Rather than
///   widen the constraint for a feature that grants nothing yet, it is refused
///   up front.
/// * At most [`MAX_DELEGATION_DAYS`], with a written reason.
pub fn validate_delegation_authority(
    delegator_roles: &lib_core::authz::RoleSet,
    request: &CreateDelegationRequest,
    now: DateTime<Utc>,
) -> Result<(), DelegationError> {
    let role = request.role.trim();
    if lib_core::authz::role_info(role).is_none() {
        return Err(DelegationError::InvalidRequest(format!(
            "Role '{role}' tidak dikenal"
        )));
    }
    if lib_core::authz::is_admin_role(role) {
        return Err(DelegationError::InvalidRequest(
            "Role administrator tidak dapat didelegasikan".to_string(),
        ));
    }
    if !delegator_roles.has(role) {
        return Err(DelegationError::Unauthorized(format!(
            "Anda tidak memegang role '{role}', sehingga tidak dapat mendelegasikannya"
        )));
    }
    if request.valid_from > now + chrono::Duration::minutes(5) {
        return Err(DelegationError::InvalidRequest(
            "Tanggal mulai tidak boleh di masa depan; delegasi berlaku sejak dibuat".to_string(),
        ));
    }
    if request.valid_until - request.valid_from > chrono::Duration::days(MAX_DELEGATION_DAYS) {
        return Err(DelegationError::InvalidRequest(format!(
            "Delegasi maksimal {MAX_DELEGATION_DAYS} hari"
        )));
    }
    let reason_chars = request
        .reason
        .as_deref()
        .map(|r| r.trim().chars().count())
        .unwrap_or(0);
    if reason_chars < MIN_DELEGATION_REASON_CHARS {
        return Err(DelegationError::InvalidRequest(format!(
            "Alasan delegasi wajib diisi minimal {MIN_DELEGATION_REASON_CHARS} karakter"
        )));
    }
    Ok(())
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
        if request.delegator_user_id == request.delegate_user_id {
            return Err(DelegationError::InvalidRequest(
                "Cannot delegate to yourself".to_string(),
            ));
        }

        if request.valid_until <= request.valid_from {
            return Err(DelegationError::InvalidRequest(
                "End date must be after start date".to_string(),
            ));
        }

        if request.valid_until <= Utc::now() {
            return Err(DelegationError::InvalidRequest(
                "End date must be in the future".to_string(),
            ));
        }

        let delegation_id = Uuid::new_v4();
        let client = self.db_pool.get().await?;

        // Determine initial status
        let now = Utc::now();
        let status = if request.valid_from > now {
            DelegationStatus::Scheduled
        } else {
            DelegationStatus::Active
        };

        // The V007 schema declares `reason` as NOT NULL; the API still
        // accepts `Option<String>` from the frontend, so coalesce to "" when
        // the caller didn't supply one rather than failing the INSERT.
        let reason_val = request.reason.clone().unwrap_or_default();

        let query = r#"
            INSERT INTO perlengkapan.workflow_delegations
            (id, delegator_user_id, delegate_user_id, delegator_role,
             valid_from, valid_until, reason, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING created_at, updated_at
        "#;

        let row = client
            .query_one(
                query,
                &[
                    &delegation_id,
                    &request.delegator_user_id,
                    &request.delegate_user_id,
                    &request.role,
                    &request.valid_from,
                    &request.valid_until,
                    &reason_val,
                    &format!("{:?}", status).to_uppercase(),
                ],
            )
            .await?;

        tracing::info!(
            delegation_id = %delegation_id,
            delegator_user_id = %request.delegator_user_id,
            delegate_user_id = %request.delegate_user_id,
            role = %request.role,
            valid_from = %request.valid_from,
            valid_until = %request.valid_until,
            status = ?status,
            "Delegation created"
        );

        Ok(Delegation {
            id: delegation_id,
            delegator_user_id: request.delegator_user_id,
            delegate_user_id: request.delegate_user_id,
            role: request.role,
            valid_from: request.valid_from,
            valid_until: request.valid_until,
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
            SELECT id, delegator_user_id, delegate_user_id, delegator_role AS role, valid_from, valid_until,
                   reason, status, created_at, updated_at
            FROM perlengkapan.workflow_delegations
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
            SELECT id, delegator_user_id, delegate_user_id, delegator_role AS role, valid_from, valid_until,
                   reason, status, created_at, updated_at
            FROM perlengkapan.workflow_delegations
            WHERE delegate_user_id = $1
              AND status = 'ACTIVE'
              AND valid_from <= NOW()
              AND valid_until > NOW()
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
        delegator_user_id: Uuid,
    ) -> Result<Vec<Delegation>, DelegationError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT id, delegator_user_id, delegate_user_id, delegator_role AS role, valid_from, valid_until,
                   reason, status, created_at, updated_at
            FROM perlengkapan.workflow_delegations
            WHERE delegator_user_id = $1
            ORDER BY created_at DESC
        "#;

        let rows = client.query(query, &[&delegator_user_id]).await?;

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
            FROM perlengkapan.workflow_delegations
            WHERE delegate_user_id = $1
              AND delegator_role = $2
              AND status = 'ACTIVE'
              AND valid_from <= NOW()
              AND valid_until > NOW()
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
            SELECT delegator_user_id, status
            FROM perlengkapan.workflow_delegations
            WHERE id = $1
        "#;

        let row = client
            .query_opt(check_query, &[&delegation_id])
            .await?
            .ok_or_else(|| DelegationError::DelegationNotFound(delegation_id))?;

        let delegator_user_id: Uuid = row.get("delegator_user_id");
        let status: String = row.get("status");

        // Only the delegator can revoke
        if delegator_user_id != revoked_by {
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
            UPDATE perlengkapan.workflow_delegations
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
            UPDATE perlengkapan.workflow_delegations
            SET status = 'EXPIRED', updated_at = NOW()
            WHERE status IN ('ACTIVE', 'SCHEDULED')
              AND valid_until <= NOW()
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
            UPDATE perlengkapan.workflow_delegations
            SET status = 'ACTIVE', updated_at = NOW()
            WHERE status = 'SCHEDULED'
              AND valid_from <= NOW()
              AND valid_until > NOW()
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
            delegator_user_id: row.get("delegator_user_id"),
            delegate_user_id: row.get("delegate_user_id"),
            role: row.get("role"),
            valid_from: row.get("valid_from"),
            valid_until: row.get("valid_until"),
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
    use lib_core::authz::RoleSet;

    fn req(role: &str) -> CreateDelegationRequest {
        let now = Utc::now();
        CreateDelegationRequest {
            delegator_user_id: Uuid::new_v4(),
            delegate_user_id: Uuid::new_v4(),
            role: role.to_string(),
            valid_from: now,
            valid_until: now + chrono::Duration::days(7),
            reason: Some("Cuti tahunan 7 hari".to_string()),
        }
    }

    #[test]
    fn a_caller_can_only_delegate_a_role_they_hold() {
        let now = Utc::now();
        let mine = RoleSet::new(["validator_wilayah"]);
        assert!(validate_delegation_authority(&mine, &req("validator_wilayah"), now).is_ok());
        assert!(matches!(
            validate_delegation_authority(&mine, &req("validator_pusat"), now),
            Err(DelegationError::Unauthorized(_))
        ));
    }

    #[test]
    fn administrator_and_unknown_roles_cannot_be_delegated() {
        let now = Utc::now();
        let admin = RoleSet::new(["admin", "superadmin", "admin_pusat"]);
        for role in ["admin", "superadmin", "admin_pusat"] {
            assert!(
                matches!(
                    validate_delegation_authority(&admin, &req(role), now),
                    Err(DelegationError::InvalidRequest(_))
                ),
                "{role} must not be delegable even by an admin"
            );
        }
        let odd = RoleSet::new(["totally_made_up"]);
        assert!(validate_delegation_authority(&odd, &req("totally_made_up"), now).is_err());
    }

    #[test]
    fn the_window_is_bounded_and_may_not_start_in_the_future() {
        let now = Utc::now();
        let mine = RoleSet::new(["validator_wilayah"]);

        let mut long = req("validator_wilayah");
        long.valid_until = long.valid_from + chrono::Duration::days(MAX_DELEGATION_DAYS + 1);
        assert!(validate_delegation_authority(&mine, &long, now).is_err());

        let mut exact = req("validator_wilayah");
        exact.valid_until = exact.valid_from + chrono::Duration::days(MAX_DELEGATION_DAYS);
        assert!(validate_delegation_authority(&mine, &exact, now).is_ok());

        // The `workflow_delegations` status check has no SCHEDULED value.
        let mut future = req("validator_wilayah");
        future.valid_from = now + chrono::Duration::days(2);
        future.valid_until = future.valid_from + chrono::Duration::days(3);
        assert!(validate_delegation_authority(&mine, &future, now).is_err());
    }

    #[test]
    fn a_written_reason_is_required() {
        let now = Utc::now();
        let mine = RoleSet::new(["validator_wilayah"]);
        let mut none = req("validator_wilayah");
        none.reason = None;
        assert!(validate_delegation_authority(&mine, &none, now).is_err());
        let mut short = req("validator_wilayah");
        short.reason = Some("cuti".to_string());
        assert!(validate_delegation_authority(&mine, &short, now).is_err());
        let mut blank = req("validator_wilayah");
        blank.reason = Some("           ".to_string());
        assert!(validate_delegation_authority(&mine, &blank, now).is_err());
    }

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
            delegator_user_id: user1,
            delegate_user_id: user2,
            role: "admin_pusat".to_string(),
            valid_from: now,
            valid_until: future,
            reason: Some("Vacation".to_string()),
        };

        assert_ne!(request.delegator_user_id, request.delegate_user_id);
        assert!(request.valid_until > request.valid_from);
    }
}
