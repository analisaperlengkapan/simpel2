// Audit Service - Provides audit log querying and compliance reporting
//
// This service provides:
// - Audit log querying with filtering
// - Compliance report generation
// - Audit trail functionality
// - Log retention policy enforcement

use authenc_types::{AuditLog, ComplianceMetrics, Result};
use authenc_storage::Database;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;

/// Audit service for querying audit logs and generating compliance reports
#[derive(Clone)]
pub struct AuditService {
    db: Arc<Database>,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get audit logs with optional filtering
    ///
    /// # Arguments
    /// * `user_id` - Optional user ID filter
    /// * `action` - Optional action/event type filter
    /// * `start_time` - Optional start time filter (Unix timestamp)
    /// * `end_time` - Optional end time filter (Unix timestamp)
    /// * `limit` - Maximum number of logs to return
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    /// Tuple of (logs, total_count)
    pub async fn get_audit_logs(
        &self,
        user_id: Option<Uuid>,
        action: Option<String>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AuditLog>, i32)> {
        // For simplicity, we'll use separate queries for each filter combination
        // In production, you'd want to use a query builder or dynamic SQL

        let limit_i64 = limit as i64;
        let offset_i64 = offset as i64;

        // Convert timestamps to DateTime
        let start_dt = start_time.and_then(|t| DateTime::from_timestamp(t, 0));
        let end_dt = end_time.and_then(|t| DateTime::from_timestamp(t, 0));

        // Build query based on filters
        let (query, count_query, params) = self.build_audit_query(
            user_id.as_ref(),
            action.as_ref(),
            start_dt.as_ref(),
            end_dt.as_ref(),
            &limit_i64,
            &offset_i64,
        );

        // Get total count
        let count_row: Row = self.db.query_one(&count_query, &params[..params.len()-2]).await?;
        let total: i64 = count_row.get(0);

        // Execute main query
        let rows = self.db.query(&query, &params).await?;

        // Convert rows to AuditLog
        let logs: Vec<AuditLog> = rows
            .into_iter()
            .map(|row| self.row_to_audit_log(row))
            .collect::<Result<Vec<_>>>()?;

        Ok((logs, total as i32))
    }

    /// Build audit query with filters
    fn build_audit_query<'a>(
        &self,
        user_id: Option<&'a Uuid>,
        action: Option<&'a String>,
        start_dt: Option<&'a DateTime<Utc>>,
        end_dt: Option<&'a DateTime<Utc>>,
        limit: &'a i64,
        offset: &'a i64,
    ) -> (String, String, Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>) {
        let mut query = String::from(
            r#"
            SELECT
                id,
                user_id,
                event_type,
                resource_type,
                success,
                COALESCE(ip_address::text, ''),
                COALESCE(user_agent, ''),
                created_at,
                event_data
            FROM event_log
            WHERE 1=1
            "#,
        );

        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut param_count = 1;

        // Add filters
        if let Some(uid) = user_id {
            query.push_str(&format!(" AND user_id = ${}", param_count));
            params.push(uid);
            param_count += 1;
        }

        if let Some(act) = action {
            query.push_str(&format!(" AND event_type = ${}", param_count));
            params.push(act);
            param_count += 1;
        }

        if let Some(start) = start_dt {
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            params.push(start);
            param_count += 1;
        }

        if let Some(end) = end_dt {
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            params.push(end);
            param_count += 1;
        }

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS count_query", query);

        // Add ordering and pagination to main query
        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count, param_count + 1));
        params.push(limit);
        params.push(offset);

        (query, count_query, params)
    }

    /// Generate compliance report for a time period
    ///
    /// # Arguments
    /// * `start_time` - Start time (Unix timestamp)
    /// * `end_time` - End time (Unix timestamp)
    /// * `report_types` - Types of reports to generate
    ///
    /// # Returns
    /// Map of report type to compliance metrics
    pub async fn get_compliance_report(
        &self,
        start_time: i64,
        end_time: i64,
        report_types: Vec<String>,
    ) -> Result<HashMap<String, ComplianceMetrics>> {
        let start_dt = DateTime::from_timestamp(start_time, 0)
            .ok_or_else(|| authenc_types::error::AuthencError::validation("Invalid start_time"))?;
        let end_dt = DateTime::from_timestamp(end_time, 0)
            .ok_or_else(|| authenc_types::error::AuthencError::validation("Invalid end_time"))?;

        let mut metrics_map = HashMap::new();

        // Generate metrics for each requested report type
        for report_type in report_types {
            let metrics = match report_type.as_str() {
                "authentication" => self.get_authentication_metrics(&start_dt, &end_dt).await?,
                "mfa" => self.get_mfa_metrics(&start_dt, &end_dt).await?,
                "sessions" => self.get_session_metrics(&start_dt, &end_dt).await?,
                "security" => self.get_security_metrics(&start_dt, &end_dt).await?,
                _ => {
                    // Unknown report type, return empty metrics
                    ComplianceMetrics {
                        total_authentications: 0,
                        failed_authentications: 0,
                        mfa_enabled_users: 0,
                        active_sessions: 0,
                        additional_metrics: HashMap::new(),
                    }
                }
            };

            metrics_map.insert(report_type, metrics);
        }

        Ok(metrics_map)
    }

    /// Get authentication metrics for compliance reporting
    async fn get_authentication_metrics(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<ComplianceMetrics> {
        // Query total authentications
        let total_auth_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type IN ('LOGIN', 'AUTHENTICATE')
            AND created_at >= $1 AND created_at <= $2
        "#;
        let total_row: Row = self.db.query_one(total_auth_query, &[start_time, end_time]).await?;
        let total_authentications: i64 = total_row.get(0);

        // Query failed authentications
        let failed_auth_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type IN ('LOGIN', 'AUTHENTICATE')
            AND success = false
            AND created_at >= $1 AND created_at <= $2
        "#;
        let failed_row: Row = self.db.query_one(failed_auth_query, &[start_time, end_time]).await?;
        let failed_authentications: i64 = failed_row.get(0);

        // Query MFA-enabled users
        let mfa_users_query = r#"
            SELECT COUNT(*)
            FROM users
            WHERE mfa_enabled = true
        "#;
        let mfa_row: Row = self.db.query_one(mfa_users_query, &[]).await?;
        let mfa_enabled_users: i64 = mfa_row.get(0);

        // Query active sessions
        let active_sessions_query = r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE expires_at > NOW()
        "#;
        let sessions_row: Row = self.db.query_one(active_sessions_query, &[]).await?;
        let active_sessions: i64 = sessions_row.get(0);

        // Additional metrics
        let mut additional_metrics = HashMap::new();

        // Success rate
        let success_rate = if total_authentications > 0 {
            ((total_authentications - failed_authentications) as f64 / total_authentications as f64 * 100.0) as i64
        } else {
            0
        };
        additional_metrics.insert("success_rate_percent".to_string(), success_rate);

        Ok(ComplianceMetrics {
            total_authentications,
            failed_authentications,
            mfa_enabled_users,
            active_sessions,
            additional_metrics,
        })
    }

    /// Get MFA metrics for compliance reporting
    async fn get_mfa_metrics(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<ComplianceMetrics> {
        // Query MFA verifications
        let mfa_verify_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type = 'MFA_VERIFY'
            AND created_at >= $1 AND created_at <= $2
        "#;
        let verify_row: Row = self.db.query_one(mfa_verify_query, &[start_time, end_time]).await?;
        let total_mfa_verifications: i64 = verify_row.get(0);

        // Query failed MFA verifications
        let failed_mfa_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type = 'MFA_VERIFY'
            AND success = false
            AND created_at >= $1 AND created_at <= $2
        "#;
        let failed_row: Row = self.db.query_one(failed_mfa_query, &[start_time, end_time]).await?;
        let failed_mfa_verifications: i64 = failed_row.get(0);

        // Query MFA-enabled users
        let mfa_users_query = r#"
            SELECT COUNT(*)
            FROM users
            WHERE mfa_enabled = true
        "#;
        let mfa_row: Row = self.db.query_one(mfa_users_query, &[]).await?;
        let mfa_enabled_users: i64 = mfa_row.get(0);

        // Additional metrics
        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("total_mfa_verifications".to_string(), total_mfa_verifications);
        additional_metrics.insert("failed_mfa_verifications".to_string(), failed_mfa_verifications);

        Ok(ComplianceMetrics {
            total_authentications: total_mfa_verifications,
            failed_authentications: failed_mfa_verifications,
            mfa_enabled_users,
            active_sessions: 0,
            additional_metrics,
        })
    }

    /// Get session metrics for compliance reporting
    async fn get_session_metrics(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<ComplianceMetrics> {
        // Query sessions created in period
        let sessions_created_query = r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE created_at >= $1 AND created_at <= $2
        "#;
        let created_row: Row = self.db.query_one(sessions_created_query, &[start_time, end_time]).await?;
        let sessions_created: i64 = created_row.get(0);

        // Query active sessions
        let active_sessions_query = r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE expires_at > NOW()
        "#;
        let active_row: Row = self.db.query_one(active_sessions_query, &[]).await?;
        let active_sessions: i64 = active_row.get(0);

        // Query expired sessions in period
        let expired_sessions_query = r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE expires_at >= $1 AND expires_at <= $2
        "#;
        let expired_row: Row = self.db.query_one(expired_sessions_query, &[start_time, end_time]).await?;
        let expired_sessions: i64 = expired_row.get(0);

        // Additional metrics
        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("sessions_created".to_string(), sessions_created);
        additional_metrics.insert("sessions_expired".to_string(), expired_sessions);

        Ok(ComplianceMetrics {
            total_authentications: 0,
            failed_authentications: 0,
            mfa_enabled_users: 0,
            active_sessions,
            additional_metrics,
        })
    }

    /// Get security metrics for compliance reporting
    async fn get_security_metrics(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<ComplianceMetrics> {
        // Query security events (failed logins, lockouts, etc.)
        let security_events_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_category = 'AUTH'
            AND success = false
            AND created_at >= $1 AND created_at <= $2
        "#;
        let security_row: Row = self.db.query_one(security_events_query, &[start_time, end_time]).await?;
        let security_events: i64 = security_row.get(0);

        // Query account lockouts
        let lockouts_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type = 'ACCOUNT_LOCKED'
            AND created_at >= $1 AND created_at <= $2
        "#;
        let lockouts_row: Row = self.db.query_one(lockouts_query, &[start_time, end_time]).await?;
        let account_lockouts: i64 = lockouts_row.get(0);

        // Query password resets
        let password_resets_query = r#"
            SELECT COUNT(*)
            FROM event_log
            WHERE event_type = 'PASSWORD_RESET'
            AND created_at >= $1 AND created_at <= $2
        "#;
        let resets_row: Row = self.db.query_one(password_resets_query, &[start_time, end_time]).await?;
        let password_resets: i64 = resets_row.get(0);

        // Additional metrics
        let mut additional_metrics = HashMap::new();
        additional_metrics.insert("security_events".to_string(), security_events);
        additional_metrics.insert("account_lockouts".to_string(), account_lockouts);
        additional_metrics.insert("password_resets".to_string(), password_resets);

        Ok(ComplianceMetrics {
            total_authentications: 0,
            failed_authentications: security_events,
            mfa_enabled_users: 0,
            active_sessions: 0,
            additional_metrics,
        })
    }

    /// Convert database row to AuditLog
    fn row_to_audit_log(&self, row: Row) -> Result<AuditLog> {
        let id: Uuid = row.get(0);
        let user_id: Option<Uuid> = row.get(1);
        let action: String = row.get(2);
        let resource: Option<String> = row.get(3);
        let success: bool = row.get(4);
        let ip_address: String = row.get(5);
        let user_agent: String = row.get(6);
        let timestamp: DateTime<Utc> = row.get(7);
        let metadata_json: Option<serde_json::Value> = row.get(8);

        // Convert metadata JSON to HashMap
        let metadata = if let Some(json) = metadata_json {
            if let serde_json::Value::Object(map) = json {
                map.into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        Ok(AuditLog {
            id: id.to_string(),
            user_id: user_id.map(|u| u.to_string()).unwrap_or_default(),
            action,
            resource: resource.unwrap_or_default(),
            success,
            ip_address,
            user_agent,
            timestamp: timestamp.timestamp(),
            metadata,
        })
    }

    /// Enforce log retention policy (delete old logs)
    ///
    /// # Arguments
    /// * `retention_days` - Number of days to retain logs
    ///
    /// # Returns
    /// Number of logs deleted
    pub async fn enforce_retention_policy(&self, retention_days: i32) -> Result<i64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        let delete_query = r#"
            DELETE FROM event_log
            WHERE created_at < $1
        "#;

        let result = self.db.execute(delete_query, &[&cutoff_date]).await?;
        Ok(result as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
    // For now, we'll skip tests as they require database setup
}
