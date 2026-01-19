//! Audit Integrity Checker Service
//!
//! Provperiodic integrity checks for audit logs to detect tampering.
//! Runs daily checks and alerts on integrity failures.

use crate::database::Database;
use crate::models::events::{AdminEvent, Event};
use crate::services::audit_signature::{AuditSignatureError, AuditSignatureService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::time::{Duration, interval};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Errors related to audit integrity operations
#[derive(Debug, Error)]
pub enum AuditIntegrityError {
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Signature verification error
    #[error("Signature verification error: {0}")]
    SignatureError(#[from] AuditSignatureError),

    /// Integrity check failed
    #[error("Integrity check failed: {0} events failed verification")]
    IntegrityCheckFailed(usize),
}

/// Result of an integrity check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    /// Check ID
    pub id: Uuid,
    /// Check timestamp
    pub check_time: DateTime<Utc>,
    /// Number of events checked
    pub events_checked: usize,
    /// Number of admin events checked
    pub admin_events_checked: usize,
    /// Number of events that failed verification
    pub events_failed: usize,
    /// Number of admin events that failed verification
    pub admin_events_failed: usize,
    /// Check duration in milliseconds
    pub check_duration_ms: u64,
    /// Check status
    pub status: IntegrityCheckStatus,
    /// Error details if any
    pub error_details: Option<String>,
}

/// Status of an integrity check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrityCheckStatus {
    /// All checks passed
    Success,
    /// Some checks failed
    Failed,
    /// Check partially completed
    Partial,
}

impl IntegrityCheckStatus {
/// Fungsi `as_str(`.
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrityCheckStatus::Success => "success",
            IntegrityCheckStatus::Failed => "failed",
            IntegrityCheckStatus::Partial => "partial",
        }
    }
}

/// Details of a failed integrity check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityFailure {
    /// Failure ID
    pub id: Uuid,
    /// Check ID this failure belongs to
    pub check_id: Uuid,
    /// Event type ('event' or 'admin_event')
    pub event_type: String,
    /// Event ID
    pub event_id: Uuid,
    /// Expected signature
    pub expected_signature: String,
    /// Actual signature from database
    pub actual_signature: String,
    /// Event data for forensics
    pub event_data: serde_json::Value,
    /// When the failure was detected
    pub detected_at: DateTime<Utc>,
}

/// Audit integrity checker service
pub struct AuditIntegrityChecker {
    /// Database connection
    database: Arc<Database>,
    /// Signature service
    signature_service: Arc<AuditSignatureService>,
    /// Check interval in hours
    check_interval_hours: u64,
}

impl AuditIntegrityChecker {
    /// Create a new audit integrity checker
    pub fn new(
        database: Arc<Database>,
        signature_service: Arc<AuditSignatureService>,
        check_interval_hours: u64,
    ) -> Self {
        Self {
            database,
            signature_service,
            check_interval_hours,
        }
    }

    /// Start the periodic integrity checker
    ///
    /// Runs integrity checks at the configured interval
    pub async fn start_periodic_checks(self: Arc<Self>) {
        let mut check_interval = interval(Duration::from_secs(self.check_interval_hours * 3600));

        info!(
            "Starting periodic audit integrity checks every {} hours",
            self.check_interval_hours
        );

        loop {
            check_interval.tick().await;

            info!("Running scheduled audit integrity check");

            match self.run_integrity_check().await {
                Ok(result) => {
                    if result.status == IntegrityCheckStatus::Success {
                        info!(
                            "Integrity check completed successfully: {} events, {} admin events checked",
                            result.events_checked, result.admin_events_checked
                        );
                    } else {
                        error!(
                            "Integrity check FAILED: {} events failed, {} admin events failed",
                            result.events_failed, result.admin_events_failed
                        );

                        // Trigger alert
                        self.trigger_integrity_alert(&result).await;
                    }
                }
                Err(e) => {
                    error!("Failed to run integrity check: {}", e);
                }
            }
        }
    }

    /// Run a complete integrity check
    ///
    /// Verifies signatures for all events and admin events
    pub async fn run_integrity_check(&self) -> Result<IntegrityCheckResult, AuditIntegrityError> {
        let start_time = Instant::now();
        let check_id = Uuid::new_v4();
        let check_time = Utc::now();

        info!("Starting integrity check {}", check_id);

        // Check events
        let (events_checked, events_failed, event_failures) = self.check_events(check_id).await?;

        // Check admin events
        let (admin_events_checked, admin_events_failed, admin_event_failures) =
            self.check_admin_events(check_id).await?;

        let check_duration_ms = start_time.elapsed().as_millis() as u64;

        let status = if events_failed == 0 && admin_events_failed == 0 {
            IntegrityCheckStatus::Success
        } else {
            IntegrityCheckStatus::Failed
        };

        let result = IntegrityCheckResult {
            id: check_id,
            check_time,
            events_checked,
            admin_events_checked,
            events_failed,
            admin_events_failed,
            check_duration_ms,
            status: status.clone(),
            error_details: None,
        };

        // Store check result
        self.store_check_result(&result).await?;

        // Store failures
        for failure in event_failures.iter().chain(admin_event_failures.iter()) {
            self.store_integrity_failure(failure).await?;
        }

        info!(
            "Integrity check {} completed in {}ms: status={:?}, events_checked={}, admin_events_checked={}, events_failed={}, admin_events_failed={}",
            check_id,
            check_duration_ms,
            status,
            events_checked,
            admin_events_checked,
            events_failed,
            admin_events_failed
        );

        Ok(result)
    }

    /// Check integrity of all events
    async fn check_events(
        &self,
        check_id: Uuid,
    ) -> Result<(usize, usize, Vec<IntegrityFailure>), AuditIntegrityError> {
        let query = r#"
            SELECT id, time, event_type, realm_id, realm_name, client_id,
                   user_id, session_id, ip_address, error, details, signature
            FROM events
            WHERE signature IS NOT NULL
            ORDER BY time DESC
            LIMIT 10000
        "#;

        let rows = self
            .database
            .query_raw(query, &[])
            .await
            .map_err(|e| AuditIntegrityError::DatabaseError(e.to_string()))?;

        let mut checked = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for row in rows {
            checked += 1;

            let event_id: Uuid = row.get(0);
            let signature: Option<String> = row.get(11);

            if let Some(ref sig) = signature {
                // Reconstruct event from row
                let event = self.reconstruct_event_from_row(&row)?;

                // Verify signature
                if let Err(e) = self.signature_service.verify_event(&event, sig) {
                    failed += 1;
                    warn!("Event {} failed integrity check: {}", event_id, e);

                    let expected_sig = self
                        .signature_service
                        .sign_event(&event)
                        .unwrap_or_default();

                    failures.push(IntegrityFailure {
                        id: Uuid::new_v4(),
                        check_id,
                        event_type: "event".to_string(),
                        event_id,
                        expected_signature: expected_sig,
                        actual_signature: sig.clone(),
                        event_data: serde_json::to_value(&event).unwrap_or_default(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok((checked, failed, failures))
    }

    /// Check integrity of all admin events
    async fn check_admin_events(
        &self,
        check_id: Uuid,
    ) -> Result<(usize, usize, Vec<IntegrityFailure>), AuditIntegrityError> {
        let query = r#"
            SELECT id, time, realm_id, realm_name, auth_user_id, auth_username,
                   auth_ip_address, auth_user_agent, resource_type, operation_type,
                   resource_path, representation, error, signature
            FROM admin_events
            WHERE signature IS NOT NULL
            ORDER BY time DESC
            LIMIT 10000
        "#;

        let rows = self
            .database
            .query_raw(query, &[])
            .await
            .map_err(|e| AuditIntegrityError::DatabaseError(e.to_string()))?;

        let mut checked = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for row in rows {
            checked += 1;

            let event_id: Uuid = row.get(0);
            let signature: Option<String> = row.get(13);

            if let Some(ref sig) = signature {
                // Reconstruct admin event from row
                let event = self.reconstruct_admin_event_from_row(&row)?;

                // Verify signature
                if let Err(e) = self.signature_service.verify_admin_event(&event, sig) {
                    failed += 1;
                    warn!("Admin event {} failed integrity check: {}", event_id, e);

                    let expected_sig = self
                        .signature_service
                        .sign_admin_event(&event)
                        .unwrap_or_default();

                    failures.push(IntegrityFailure {
                        id: Uuid::new_v4(),
                        check_id,
                        event_type: "admin_event".to_string(),
                        event_id,
                        expected_signature: expected_sig,
                        actual_signature: sig.clone(),
                        event_data: serde_json::to_value(&event).unwrap_or_default(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok((checked, failed, failures))
    }

    /// Reconstruct an Event from a database row
    fn reconstruct_event_from_row(
        &self,
        row: &tokio_postgres::Row,
    ) -> Result<Event, AuditIntegrityError> {
        use crate::models::events::EventType;
        use std::collections::HashMap;

        let event_type_str: String = row.get(2);
        let event_type = EventType::from_str(&event_type_str).unwrap_or(EventType::Login);

        let details_json: Option<String> = row.get(10);
        let details: HashMap<String, String> = details_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        Ok(Event {
            id: row.get::<_, Uuid>(0).to_string(),
            time: row.get(1),
            event_type,
            realm_id: row.get(3),
            realm_name: row.get(4),
            client_id: row.get(5),
            user_id: row.get(6),
            session_id: row.get(7),
            ip_address: row.get(8),
            error: row.get(9),
            details,
        })
    }

    /// Reconstruct an AdminEvent from a database row
    fn reconstruct_admin_event_from_row(
        &self,
        row: &tokio_postgres::Row,
    ) -> Result<AdminEvent, AuditIntegrityError> {
        use crate::models::events::{AuthDetails, OperationType, ResourceType};

        let resource_type_str: String = row.get(8);
        let resource_type =
            ResourceType::from_str(&resource_type_str).unwrap_or(ResourceType::Custom);

        let operation_type_str: String = row.get(9);
        let operation_type =
            OperationType::from_str(&operation_type_str).unwrap_or(OperationType::Action);

        let auth_user_id: Option<Uuid> = row.get(4);

        Ok(AdminEvent {
            id: row.get::<_, Uuid>(0).to_string(),
            time: row.get(1),
            realm_id: row.get(2),
            realm_name: row.get(3),
            auth_details: AuthDetails {
                user_id: auth_user_id.map(|id| id.to_string()).unwrap_or_default(),
                username: row.get(5),
                ip_address: row.get(6),
                user_agent: row.get(7),
            },
            resource_type,
            operation_type,
            resource_path: row.get(10),
            representation: row.get(11),
            error: row.get(12),
        })
    }

    /// Store integrity check result
    async fn store_check_result(
        &self,
        result: &IntegrityCheckResult,
    ) -> Result<(), AuditIntegrityError> {
        let query = r#"
            INSERT INTO audit_integrity_checks (
                id, check_time, events_checked, admin_events_checked,
                events_failed, admin_events_failed, check_duration_ms,
                status, error_details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        self.database
            .execute(
                query,
                &[
                    &result.id,
                    &result.check_time,
                    &(result.events_checked as i32),
                    &(result.admin_events_checked as i32),
                    &(result.events_failed as i32),
                    &(result.admin_events_failed as i32),
                    &(result.check_duration_ms as i32),
                    &result.status.as_str(),
                    &result.error_details,
                ],
            )
            .await
            .map_err(|e| AuditIntegrityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store integrity failure
    async fn store_integrity_failure(
        &self,
        failure: &IntegrityFailure,
    ) -> Result<(), AuditIntegrityError> {
        let query = r#"
            INSERT INTO audit_integrity_failures (
                id, check_id, event_type, event_id, expected_signature,
                actual_signature, event_data, detected_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        self.database
            .execute(
                query,
                &[
                    &failure.id,
                    &failure.check_id,
                    &failure.event_type,
                    &failure.event_id,
                    &failure.expected_signature,
                    &failure.actual_signature,
                    &failure.event_data,
                    &failure.detected_at,
                ],
            )
            .await
            .map_err(|e| AuditIntegrityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Trigger an alert for integrity check failure
    async fn trigger_integrity_alert(&self, result: &IntegrityCheckResult) {
        error!(
            "🚨 AUDIT INTEGRITY ALERT 🚨\n\
             Check ID: {}\n\
             Events Failed: {}/{}\n\
             Admin Events Failed: {}/{}\n\
             Status: {:?}\n\
             Duration: {}ms",
            result.id,
            result.events_failed,
            result.events_checked,
            result.admin_events_failed,
            result.admin_events_checked,
            result.status,
            result.check_duration_ms
        );

        // TODO: Integrate with alerting system (Kafka, webhook, email, etc.)
        // For now, we just log the alert
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_check_status() {
        assert_eq!(IntegrityCheckStatus::Success.as_str(), "success");
        assert_eq!(IntegrityCheckStatus::Failed.as_str(), "failed");
        assert_eq!(IntegrityCheckStatus::Partial.as_str(), "partial");
    }
}
