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
            INSERT INTO api_call_log (
                module, endpoint, kode_kl, kdsatker, full_url,
                request_method, request_headers, request_params,
                response_status, response_time_ms, response_size_bytes, record_count,
                success, error_message, retry_count,
                token_used, token_refreshed, new_token_received,
                storage_strategy, data_saved,
                started_at, completed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
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
                    &self.request_headers,
                    &self.request_params,
                    &self.response_status,
                    &self.response_time_ms,
                    &self.response_size_bytes,
                    &self.record_count,
                    &self.success,
                    &self.error_message,
                    &self.retry_count,
                    &self.token_used,
                    &self.token_refreshed,
                    &self.new_token_received,
                    &self.storage_strategy,
                    &self.data_saved,
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
            INSERT INTO batch_processing_log (
                batch_type, kode_kl, kdsatker, total_satker,
                status, storage_strategy, parallel_mode,
                started_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.batch_type,
                    &self.kode_kl,
                    &self.kdsatker,
                    &self.total_satker,
                    &self.status,
                    &self.storage_strategy,
                    &self.parallel_mode,
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
        total_calls: i32,
        successful_calls: i32,
        failed_calls: i32,
    ) -> Result<(), MonsaktiError> {
        let query = r#"
            UPDATE batch_processing_log
            SET processed_satker = $2,
                failed_satker = $3,
                total_api_calls = $4,
                successful_calls = $5,
                failed_calls = $6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
        "#;

        db.execute(
            query,
            &[
                &batch_id,
                &processed,
                &failed,
                &total_calls,
                &successful_calls,
                &failed_calls,
            ],
        )
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
        let query = r#"
            UPDATE batch_processing_log
            SET status = $2,
                error_message = $3,
                completed_at = CURRENT_TIMESTAMP,
                duration_seconds = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))::INTEGER
            WHERE id = $1
        "#;

        db.execute(query, &[&batch_id, &status, &error_message])
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
        batch_id: Option<i64>,
        api_call_id: Option<i64>,
    ) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO data_sync_log (
                table_name, module, endpoint,
                records_fetched, records_inserted, records_updated, records_failed,
                kode_kl, kdsatker, batch_id, api_call_id,
                success, error_message
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.table_name,
                    &self.module,
                    &self.endpoint,
                    &self.records_fetched,
                    &self.records_inserted,
                    &self.records_updated,
                    &self.records_failed,
                    &self.kode_kl,
                    &self.kdsatker,
                    &batch_id,
                    &api_call_id,
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
    pub async fn save(&self, db: &Client, api_call_id: Option<i64>) -> Result<i64, MonsaktiError> {
        let query = r#"
            INSERT INTO token_reset_log (
                module, kode_kl, reset_reason, reset_method,
                success, error_message, new_token_received, api_call_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
        "#;

        let row = db
            .query_one(
                query,
                &[
                    &self.module,
                    &self.kode_kl,
                    &self.reset_reason,
                    &self.reset_method,
                    &self.success,
                    &self.error_message,
                    &self.new_token_received,
                    &api_call_id,
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
