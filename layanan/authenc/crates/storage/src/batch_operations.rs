//! Specialized batch operations for Authenc performance optimization
//!
//! This module provides optimized batch operations for common Authenc use cases:
//! - Batch insert for audit logs (batch size: 1000)
//! - Batch query for user permissions
//! - Batch session validation
//! - Batch user lookup by IDs
//!
//! These operations reduce database round-trips and improve performance for bulk operations.

use crate::Database;
use crate::batch::BatchInsertable;
use authenc_types::{AuthencError, Result};
use authenc_core::models::permission::Permission;
use authenc_core::models::user::User;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, error};
use uuid::Uuid;

/// Batch size for audit log inserts (as per requirement 4.4)
const AUDIT_LOG_BATCH_SIZE: usize = 1000;

/// Audit log entry for batch insertion
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl BatchInsertable for AuditLogEntry {
    fn collect_params<'a>(
        &'a self,
        params: &mut Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>,
    ) {
        params.push(&self.id);
        params.push(&self.event_type);
        params.push(&self.user_id);
        params.push(&self.session_id);
        params.push(&self.ip_address);
        params.push(&self.user_agent);
        params.push(&self.action);
        params.push(&self.resource);
        params.push(&self.success);
        params.push(&self.error_message);
        params.push(&self.metadata);
        params.push(&self.timestamp);
    }
}

/// Batch insert audit logs with optimized performance
/// Inserts multiple audit log entries in batches of 1000 to reduce database round-trips.
/// This is significantly faster than individual inserts for bulk audit logging.
/// # Arguments
/// * `db` - Database connection
/// * `entries` - Vector of audit log entries to insert
/// # Returns
/// Number of rows inserted
/// # Example
/// ```no_run
/// use authenc::database::batch_operations::{batch_insert_audit_logs, AuditLogEntry};
/// use chrono::Utc;
/// use uuid::Uuid;
/// async fn example(db: &Database) {
///     let entries = vec![
///         AuditLogEntry {
///             id: Uuid::new_v4(),
///             event_type: "LOGIN".to_string(),
///             user_id: Some(Uuid::new_v4()),
///             session_id: Some("session123".to_string()),
///             ip_address: Some("192.168.1.1".to_string()),
///             user_agent: Some("Mozilla/5.0".to_string()),
///             action: "authenticate".to_string(),
///             resource: "user".to_string(),
///             success: true,
///             error_message: None,
///             metadata: None,
///             timestamp: Utc::now(),
///         },
///         // ... more entries
///     ];
///     let inserted = batch_insert_audit_logs(db, entries).await.unwrap();
///     println!("Inserted {} audit logs", inserted);
/// }
/// ```
pub async fn batch_insert_audit_logs(db: &Database, entries: Vec<AuditLogEntry>) -> Result<u64> {
    if entries.is_empty() {
        return Ok(0);
    }

    let total_entries = entries.len();
    let mut total_inserted = 0u64;

    // Process in batches of 1000
    for chunk in entries.chunks(AUDIT_LOG_BATCH_SIZE) {
        let inserted = batch_insert_audit_logs_chunk(db, chunk).await?;
        total_inserted += inserted;
    }

    debug!(
        "Batch inserted {} audit logs in {} batch(es)",
        total_inserted,
        (total_entries + AUDIT_LOG_BATCH_SIZE - 1) / AUDIT_LOG_BATCH_SIZE
    );

    Ok(total_inserted)
}

/// Insert a single chunk of audit logs
async fn batch_insert_audit_logs_chunk(db: &Database, entries: &[AuditLogEntry]) -> Result<u64> {
    if entries.is_empty() {
        return Ok(0);
    }

    let num_entries = entries.len();
    let num_cols = 12; // Number of columns in audit_logs table

    // Build VALUES clause: ($1, $2, ..., $12), ($13, $14, ..., $24), ...
    let mut value_placeholders = Vec::with_capacity(num_entries);
    for row_idx in 0..num_entries {
        let start_param = row_idx * num_cols + 1;
        let end_param = start_param + num_cols;
        let params: Vec<String> = (start_param..end_param)
            .map(|i| format!("${}", i))
            .collect();
        value_placeholders.push(format!("({})", params.join(", ")));
    }

    // Build complete INSERT statement
    let sql = format!(
        "INSERT INTO audit_logs (
            id, event_type, user_id, session_id, ip_address, user_agent,
            action, resource, success, error_message, metadata, timestamp
        ) VALUES {}",
        value_placeholders.join(", ")
    );

    // Collect all parameters
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    for entry in entries {
        entry.collect_params(&mut params);
    }

    // Execute batch insert
    db.execute(&sql, &params).await.map_err(|e| {
        error!("Batch insert audit logs failed: {}", e);
        AuthencError::database(format!("Batch insert audit logs failed: {}", e))
    })
}

/// Batch query user permissions for multiple users
/// Retrieves permissions for multiple users in a single query, significantly
/// faster than querying each user individually.
/// # Arguments
/// * `db` - Database connection
/// * `user_ids` - Vector of user IDs to query permissions for
/// # Returns
/// HashMap mapping user_id to their list of permissions
/// # Example
/// ```no_run
/// use authenc::database::batch_operations::batch_query_user_permissions;
/// use uuid::Uuid;
/// async fn example(db: &Database) {
///     let user_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
///     let permissions_map = batch_query_user_permissions(db, user_ids).await.unwrap();
///     for (user_id, permissions) in permissions_map {
///         println!("User {} has {} permissions", user_id, permissions.len());
///     }
/// }
/// ```
pub async fn batch_query_user_permissions(
    db: &Database,
    user_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, Vec<Permission>>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build IN clause with placeholders
    let placeholders: Vec<String> = (1..=user_ids.len()).map(|i| format!("${}", i)).collect();

    let sql = format!(
        "SELECT
            p.id, p.name, p.description, p.resource,
            p.action, p.realm_id, p.created_at, p.updated_at,
            rp.role_id, ur.user_id
        FROM permissions p
        INNER JOIN role_permissions rp ON p.id = rp.permission_id
        INNER JOIN user_roles ur ON rp.role_id = ur.role_id
        WHERE ur.user_id IN ({})
          AND p.deleted_at IS NULL
          AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
        ORDER BY ur.user_id, p.name",
        placeholders.join(", ")
    );

    // Convert user_ids to parameters
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = user_ids
        .iter()
        .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = db.query(&sql, &params).await?;

    // Group permissions by user_id
    let mut permissions_map: HashMap<Uuid, Vec<Permission>> = HashMap::new();

    for row in rows {
        let user_id: Uuid = row.get(9);

        let permission = Permission {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            resource: row.get(3),
            action: row.get(4),
            realm_id: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7),
            deleted_at: None,
        };

        permissions_map
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(permission);
    }

    // Ensure all requested user_ids are in the map (even if they have no permissions)
    for user_id in user_ids {
        permissions_map.entry(user_id).or_insert_with(Vec::new);
    }

    debug!(
        "Batch queried permissions for {} users, found {} total permissions",
        permissions_map.len(),
        permissions_map.values().map(|v| v.len()).sum::<usize>()
    );

    Ok(permissions_map)
}

/// Session validation result
#[derive(Debug, Clone)]
pub struct SessionValidationResult {
    pub session_id: String,
    pub is_valid: bool,
    pub user_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

/// Batch validate multiple sessions
/// Validates multiple sessions in a single query, checking expiration and revocation status.
/// Much faster than validating sessions individually.
/// # Arguments
/// * `db` - Database connection
/// * `session_ids` - Vector of session IDs to validate
/// # Returns
/// Vector of validation results for each session
/// # Example
/// ```no_run
/// use authenc::database::batch_operations::batch_validate_sessions;
/// async fn example(db: &Database) {
///     let session_ids = vec!["session1".to_string(), "session2".to_string()];
///     let results = batch_validate_sessions(db, session_ids).await.unwrap();
///     for result in results {
///         if result.is_valid {
///             println!("Session {} is valid", result.session_id);
///         } else {
///             println!("Session {} is invalid: {:?}", result.session_id, result.reason);
///         }
///     }
/// }
/// ```
pub async fn batch_validate_sessions(
    db: &Database,
    session_ids: Vec<String>,
) -> Result<Vec<SessionValidationResult>> {
    if session_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build IN clause with placeholders
    let placeholders: Vec<String> = (1..=session_ids.len()).map(|i| format!("${}", i)).collect();

    let sql = format!(
        "SELECT
            session_id, user_id, expires_at, revoked, last_accessed,
            mfa_verified, is_temp_session
        FROM sessions
        WHERE session_id IN ({})
        ORDER BY session_id",
        placeholders.join(", ")
    );

    // Convert session_ids to parameters
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = session_ids
        .iter()
        .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = db.query(&sql, &params).await?;

    let now = Utc::now();
    let mut results = Vec::new();
    let mut found_sessions: HashMap<String, SessionValidationResult> = HashMap::new();

    for row in rows {
        let session_id: String = row.get(0);
        let user_id: Uuid = row.get(1);
        let expires_at: DateTime<Utc> = row.get(2);
        let revoked: bool = row.get(3);
        let is_temp_session: bool = row.get(6);

        let (is_valid, reason) = if revoked {
            (false, Some("Session has been revoked".to_string()))
        } else if now >= expires_at {
            (false, Some("Session has expired".to_string()))
        } else if is_temp_session {
            (
                false,
                Some("Temporary session pending MFA verification".to_string()),
            )
        } else {
            (true, None)
        };

        found_sessions.insert(
            session_id.clone(),
            SessionValidationResult {
                session_id,
                is_valid,
                user_id: Some(user_id),
                expires_at: Some(expires_at),
                reason,
            },
        );
    }

    // Include results for all requested session_ids (mark missing ones as invalid)
    for session_id in session_ids {
        if let Some(result) = found_sessions.remove(&session_id) {
            results.push(result);
        } else {
            results.push(SessionValidationResult {
                session_id,
                is_valid: false,
                user_id: None,
                expires_at: None,
                reason: Some("Session not found".to_string()),
            });
        }
    }

    debug!(
        "Batch validated {} sessions, {} valid",
        results.len(),
        results.iter().filter(|r| r.is_valid).count()
    );

    Ok(results)
}

/// Batch lookup users by IDs
/// Retrieves multiple users in a single query, significantly faster than
/// querying each user individually.
/// # Arguments
/// * `db` - Database connection
/// * `user_ids` - Vector of user IDs to lookup
/// # Returns
/// HashMap mapping user_id to User object (only includes found users)
/// # Example
/// ```no_run
/// use authenc::database::batch_operations::batch_lookup_users;
/// use uuid::Uuid;
/// async fn example(db: &Database) {
///     let user_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
///     let users_map = batch_lookup_users(db, user_ids).await.unwrap();
///     for (user_id, user) in users_map {
///         println!("Found user: {} ({})", user.username, user_id);
///     }
/// }
/// ```
pub async fn batch_lookup_users(db: &Database, user_ids: Vec<Uuid>) -> Result<HashMap<Uuid, User>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build IN clause with placeholders
    let placeholders: Vec<String> = (1..=user_ids.len()).map(|i| format!("${}", i)).collect();

    let sql = format!(
        "SELECT
            id, username, email, email_verified, first_name, last_name,
            nip, nama, jabatan, satker_code, phone_number, phone_verified,
            password_hash, totp_secret, totp_backup_codes, mfa_enabled,
            mfa_setup_at, mfa_last_used, webauthn_enabled, account_locked,
            account_locked_until, failed_login_attempts, last_login_at,
            last_failed_login_at, password_changed_at, password_expires_at,
            require_password_change, realm_id, organization_id, session_data,
            attributes, enabled, federated, created_at, updated_at,
            deleted_at, login_count
        FROM users
        WHERE id IN ({})
          AND deleted_at IS NULL
        ORDER BY id",
        placeholders.join(", ")
    );

    // Convert user_ids to parameters
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = user_ids
        .iter()
        .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = db.query(&sql, &params).await?;

    let mut users_map: HashMap<Uuid, User> = HashMap::new();

    for row in rows {
        let user_id: Uuid = row.get(0);

        let user = User {
            id: user_id,
            username: row.get(1),
            email: row.get(2),
            email_verified: row.get(3),
            first_name: row.get(4),
            last_name: row.get(5),
            nip: row.get(6),
            nama: row.get(7),
            jabatan: row.get(8),
            satker_code: row.get(9),
            phone_number: row.get(10),
            phone_verified: row.get(11),
            password_hash: row.get(12),
            totp_secret: row.get(13),
            totp_backup_codes: row.get(14),
            mfa_enabled: row.get(15),
            mfa_setup_at: row.get(16),
            mfa_last_used: row.get(17),
            webauthn_enabled: row.get(18),
            account_locked: row.get(19),
            account_locked_until: row.get(20),
            failed_login_attempts: row.get(21),
            last_login_at: row.get(22),
            last_failed_login_at: row.get(23),
            password_changed_at: row.get(24),
            password_expires_at: row.get(25),
            require_password_change: row.get(26),
            realm_id: row.get(27),
            organization_id: row.get(28),
            roles: Vec::new(),       // Loaded separately if needed
            permissions: Vec::new(), // Loaded separately if needed
            session_data: row.get(29),
            security_context: Default::default(),
            attributes: row.get(30),
            enabled: row.get(31),
            federated: row.get(32),
            created_at: row.get(33),
            updated_at: row.get(34),
            deleted_at: row.get(35),
            login_count: row.get(36),
        };

        users_map.insert(user_id, user);
    }

    debug!(
        "Batch looked up {} users, found {}",
        user_ids.len(),
        users_map.len()
    );

    Ok(users_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_params() {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            event_type: "LOGIN".to_string(),
            user_id: Some(Uuid::new_v4()),
            session_id: Some("session123".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            action: "authenticate".to_string(),
            resource: "user".to_string(),
            success: true,
            error_message: None,
            metadata: None,
            timestamp: Utc::now(),
        };

        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        entry.collect_params(&mut params);

        // Should have 12 parameters
        assert_eq!(params.len(), 12);
    }

    #[test]
    fn test_session_validation_result() {
        let result = SessionValidationResult {
            session_id: "test123".to_string(),
            is_valid: true,
            user_id: Some(Uuid::new_v4()),
            expires_at: Some(Utc::now()),
            reason: None,
        };

        assert!(result.is_valid);
        assert!(result.user_id.is_some());
        assert!(result.reason.is_none());
    }
}
