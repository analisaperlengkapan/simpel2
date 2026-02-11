// Structured logging configuration for Perlengkapan API
// Requirements: NFR-M002

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initialize structured logging with JSON format
///
/// This configures tracing with:
/// - JSON formatted output for log aggregation
/// - Structured fields for filtering and analysis
/// - Environment-based log level filtering
/// - Span tracking for request tracing
pub fn init_structured_logging() {
    // Get log level from environment or default to info
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,layanan_perlengkapan=debug".to_string());

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&log_level));

    // Determine if we should use JSON format (production) or pretty format (development)
    let use_json = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "json".to_string())
        .to_lowercase()
        == "json";

    if use_json {
        // JSON format for production (log aggregation)
        let json_layer = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(env_filter);

        tracing_subscriber::registry()
            .with(json_layer)
            .init();
    } else {
        // Pretty format for development
        let pretty_layer = fmt::layer()
            .pretty()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(env_filter);

        tracing_subscriber::registry()
            .with(pretty_layer)
            .init();
    }

    tracing::info!(
        log_level = %log_level,
        format = if use_json { "json" } else { "pretty" },
        "Structured logging initialized"
    );
}

/// Log retention policy configuration
#[derive(Debug, Clone)]
pub struct LogRetentionPolicy {
    /// Number of days to retain logs
    pub retention_days: u32,

    /// Maximum log file size in MB
    pub max_file_size_mb: u32,

    /// Maximum number of log files to keep
    pub max_files: u32,
}

impl Default for LogRetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 30,
            max_file_size_mb: 100,
            max_files: 10,
        }
    }
}

impl LogRetentionPolicy {
    /// Load retention policy from environment variables
    pub fn from_env() -> Self {
        Self {
            retention_days: std::env::var("LOG_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_file_size_mb: std::env::var("LOG_MAX_FILE_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_files: std::env::var("LOG_MAX_FILES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

/// Structured log event for important operations
#[derive(Debug, serde::Serialize)]
pub struct StructuredLogEvent {
    pub event_type: String,
    pub user_id: Option<uuid::Uuid>,
    pub entity_id: Option<uuid::Uuid>,
    pub entity_type: Option<String>,
    pub action: String,
    pub status: String,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

impl StructuredLogEvent {
    /// Log the event with structured fields
    pub fn log(&self) {
        match self.status.as_str() {
            "success" => {
                tracing::info!(
                    event_type = %self.event_type,
                    user_id = ?self.user_id,
                    entity_id = ?self.entity_id,
                    entity_type = ?self.entity_type,
                    action = %self.action,
                    status = %self.status,
                    duration_ms = ?self.duration_ms,
                    metadata = %self.metadata,
                    "Operation completed successfully"
                );
            }
            "error" | "failed" => {
                tracing::error!(
                    event_type = %self.event_type,
                    user_id = ?self.user_id,
                    entity_id = ?self.entity_id,
                    entity_type = ?self.entity_type,
                    action = %self.action,
                    status = %self.status,
                    duration_ms = ?self.duration_ms,
                    error = ?self.error,
                    metadata = %self.metadata,
                    "Operation failed"
                );
            }
            _ => {
                tracing::warn!(
                    event_type = %self.event_type,
                    user_id = ?self.user_id,
                    entity_id = ?self.entity_id,
                    entity_type = ?self.entity_type,
                    action = %self.action,
                    status = %self.status,
                    duration_ms = ?self.duration_ms,
                    metadata = %self.metadata,
                    "Operation completed with warning"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_retention_policy_default() {
        let policy = LogRetentionPolicy::default();
        assert_eq!(policy.retention_days, 30);
        assert_eq!(policy.max_file_size_mb, 100);
        assert_eq!(policy.max_files, 10);
    }

    #[test]
    fn test_structured_log_event() {
        let event = StructuredLogEvent {
            event_type: "workflow_transition".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            entity_id: Some(uuid::Uuid::new_v4()),
            entity_type: Some("kebutuhan_bmn".to_string()),
            action: "transition".to_string(),
            status: "success".to_string(),
            duration_ms: Some(123.45),
            error: None,
            metadata: serde_json::json!({"from": "DRAFT", "to": "SUBMITTED"}),
        };

        // Just verify it doesn't panic
        event.log();
    }
}
