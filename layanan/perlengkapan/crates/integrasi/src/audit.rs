/// Audit logging module untuk tracking API calls dan token management
/// Provides comprehensive logging untuk compliance dan debugging
use crate::error::MonsaktiError;
use serde_json::Value;
use tokio_postgres::Client;
use tracing::info;

/// Log API call untuk audit trail
pub struct ApiCallLog {
    pub module: String,
    pub endpoint: String,
    pub kode_kl: Option<String>,
    pub kdsatker: Option<String>,
    pub full_url: String,
    // Request details (optional - for advanced auditing)
    pub request_method: Option<String>,
    pub request_headers: Option<Value>,
    pub request_params: Option<Value>,
    // Response information
    pub response_status: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub response_size_bytes: Option<i32>,
    pub record_count: Option<i32>,
    // Result
    pub success: bool,
    pub error_message: Option<String>,
    pub retry_count: i32,
    // Token information
    pub token_used: Option<String>,
    pub token_refreshed: bool,
    pub new_token_received: bool,
    // Metadata
    pub storage_strategy: Option<String>,
    pub data_saved: bool,
}

impl ApiCallLog {
    /// Create new API call log entry
    pub fn new(module: &str, endpoint: &str, url: &str) -> Self {
        Self {
            module: module.to_string(),
            endpoint: endpoint.to_string(),
            kode_kl: None,
            kdsatker: None,
            full_url: url.to_string(),
            request_method: None,
            request_headers: None,
            request_params: None,
            response_status: None,
            response_time_ms: None,
            response_size_bytes: None,
            record_count: None,
            success: false,
            error_message: None,
            retry_count: 0,
            token_used: None,
            token_refreshed: false,
            new_token_received: false,
            storage_strategy: None,
            data_saved: false,
        }
    }

    /// Save log to database
    pub async fn save(&self, db: &Client) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO api_log (
                log_type, module, endpoint, kode_kl, kdsatker, full_url,
                request_method,
                response_status, response_time_ms, record_count,
                success, error_message, retry_count,
                started_at, completed_at
            ) VALUES ('api_call', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.module,
                    &self.endpoint,
                    &self.kode_kl,
                    &self.kdsatker,
                    &self.full_url,
                    &self.request_method,
                    &self.response_status,
                    &self.response_time_ms,
                    &self.record_count,
                    &self.success,
                    &self.error_message,
                    &self.retry_count,
                ],
            )
            .await?;

        let id: i64 = row.get(0);
        Ok(id)
    }
}

/// Log batch processing operation
pub struct BatchProcessingLog {
    pub batch_type: String,
    pub kode_kl: Option<String>,
    pub kdsatker: Option<String>,
    pub total_satker: Option<i32>,
    pub status: String,
    pub storage_strategy: Option<String>,
    pub parallel_mode: bool,
}

impl BatchProcessingLog {
    /// Create new batch processing log
    pub fn new(batch_type: &str, kode_kl: Option<&str>) -> Self {
        Self {
            batch_type: batch_type.to_string(),
            kode_kl: kode_kl.map(|s| s.to_string()),
            kdsatker: None,
            total_satker: None,
            status: "running".to_string(),
            storage_strategy: None,
            parallel_mode: false,
        }
    }

    /// Start batch processing log
    pub async fn start(&self, db: &Client) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO api_log (
                log_type, module, endpoint, kode_kl, kdsatker,
                record_count, success, error_message,
                started_at
            ) VALUES ('batch', $1, $2, $3, $4, $5, false, NULL, CURRENT_TIMESTAMP)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.batch_type,
                    &self.kode_kl,
                    &self.kdsatker,
                    &self.kdsatker,
                    &self.total_satker,
                ],
            )
            .await?;

        let id: i64 = row.get(0);
        info!("Started batch processing log: {}", id);
        Ok(id)
    }

    /// Update batch processing log
    pub async fn update(
        db: &Client,
        batch_id: i64,
        processed: i32,
        failed: i32,
        _total_calls: i32,
        _successful_calls: i32,
        _failed_calls: i32,
    ) -> Result<(), MonsaktiError> {
        let query = r#"
            UPDATE api_log
            SET record_count = $2,
                retry_count = $3,
                completed_at = CURRENT_TIMESTAMP
            WHERE id = $1
        "#;

        db.execute(query, &[&batch_id, &processed, &failed])
            .await?;

        Ok(())
    }

    /// Complete batch processing log
    pub async fn complete(
        db: &Client,
        batch_id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), MonsaktiError> {
        let success = status == "completed";
        let query = r#"
            UPDATE api_log
            SET success = $2,
                error_message = $3,
                completed_at = CURRENT_TIMESTAMP
            WHERE id = $1
        "#;

        db.execute(query, &[&batch_id, &success, &error_message])
            .await?;

        info!(
            "Completed batch processing log: {} with status: {}",
            batch_id, status
        );
        Ok(())
    }
}

/// Log data synchronization
pub struct DataSyncLog {
    pub table_name: String,
    pub module: String,
    pub endpoint: String,
    pub records_fetched: i32,
    pub records_inserted: i32,
    pub records_updated: i32,
    pub records_failed: i32,
    pub kode_kl: Option<String>,
    pub kdsatker: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl DataSyncLog {
    /// Create new data sync log
    pub fn new(table_name: &str, module: &str, endpoint: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            module: module.to_string(),
            endpoint: endpoint.to_string(),
            records_fetched: 0,
            records_inserted: 0,
            records_updated: 0,
            records_failed: 0,
            kode_kl: None,
            kdsatker: None,
            success: false,
            error_message: None,
        }
    }

    /// Save sync log to database
    pub async fn save(
        &self,
        db: &Client,
        _batch_id: Option<i64>,
        _api_call_id: Option<i64>,
    ) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO api_log (
                log_type, module, endpoint, kode_kl, kdsatker,
                record_count, success, error_message,
                started_at, completed_at
            ) VALUES ('sync', $1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.module,
                    &self.endpoint,
                    &self.kode_kl,
                    &self.kdsatker,
                    &self.records_inserted,
                    &self.success,
                    &self.error_message,
                ],
            )
            .await?;

        let id: i64 = row.get(0);
        Ok(id)
    }
}

/// Log token reset operation
pub struct TokenResetLog {
    pub module: String,
    pub kode_kl: String,
    pub reset_reason: String,
    pub reset_method: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub new_token_received: bool,
}

impl TokenResetLog {
    /// Create new token reset log
    pub fn new(module: &str, kode_kl: &str, reason: &str, method: &str) -> Self {
        Self {
            module: module.to_string(),
            kode_kl: kode_kl.to_string(),
            reset_reason: reason.to_string(),
            reset_method: method.to_string(),
            success: false,
            error_message: None,
            new_token_received: false,
        }
    }

    /// Save reset log to database
    pub async fn save(&self, db: &Client, _api_call_id: Option<i64>) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO api_log (
                log_type, module, endpoint, kode_kl,
                success, error_message,
                started_at, completed_at
            ) VALUES ('token_reset', $1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.module,
                    &self.reset_reason,
                    &self.kode_kl,
                    &self.success,
                    &self.error_message,
                ],
            )
            .await?;

        let id: i64 = row.get(0);
        info!("Token reset logged: {} for module {}", id, self.module);
        Ok(id)
    }
}

/// Helper functions untuk query audit logs
/// Get recent failed API calls
pub async fn get_recent_failed_calls(db: &Client, limit: i64) -> Result<Vec<Value>, MonsaktiError> {
    let query = r#"
        SELECT * FROM v_recent_failed_calls
        LIMIT $1
    "#;

    let rows = db.query(query, &[&limit]).await?;

    let mut results = Vec::new();
    for row in rows {
        let json = serde_json::json!({
            "id": row.get::<_, i64>("id"),
            "module": row.get::<_, String>("module"),
            "endpoint": row.get::<_, String>("endpoint"),
            "error_message": row.get::<_, Option<String>>("error_message"),
            "started_at": row.get::<_, chrono::DateTime<chrono::Utc>>("started_at"),
        });
        results.push(json);
    }

    Ok(results)
}

/// Get token health status
pub async fn get_token_health(db: &Client) -> Result<Vec<Value>, MonsaktiError> {
    let query = "SELECT * FROM v_token_health ORDER BY health_status, module";

    let rows = db.query(query, &[]).await?;

    let mut results = Vec::new();
    for row in rows {
        let json = serde_json::json!({
            "module": row.get::<_, String>("module"),
            "health_status": row.get::<_, String>("health_status"),
            "is_active": row.get::<_, bool>("is_active"),
            "consecutive_failures": row.get::<_, i32>("consecutive_failures"),
        });
        results.push(json);
    }

    Ok(results)
}

/// Get API statistics by module
pub async fn get_api_stats_by_module(db: &Client, days: i32) -> Result<Vec<Value>, MonsaktiError> {
    let query = r#"
        SELECT * FROM v_api_stats_by_module
        WHERE call_date >= CURRENT_DATE - $1
        ORDER BY call_date DESC, module
    "#;

    let rows = db.query(query, &[&days]).await?;

    let mut results = Vec::new();
    for row in rows {
        let json = serde_json::json!({
            "module": row.get::<_, String>("module"),
            "total_calls": row.get::<_, i64>("total_calls"),
            "successful_calls": row.get::<_, i64>("successful_calls"),
            "failed_calls": row.get::<_, i64>("failed_calls"),
            "avg_response_time_ms": row.get::<_, Option<f64>>("avg_response_time_ms"),
            "call_date": row.get::<_, chrono::NaiveDate>("call_date"),
        });
        results.push(json);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_call_log_creation() {
        let log = ApiCallLog::new("ADM", "refAdmin", "https://example.com/api");
        assert_eq!(log.module, "ADM");
        assert_eq!(log.endpoint, "refAdmin");
        assert!(!log.success);
    }

    #[test]
    fn test_batch_log_creation() {
        let log = BatchProcessingLog::new("complete", Some("006"));
        assert_eq!(log.batch_type, "complete");
        assert_eq!(log.kode_kl, Some("006".to_string()));
        assert_eq!(log.status, "running");
    }
}
