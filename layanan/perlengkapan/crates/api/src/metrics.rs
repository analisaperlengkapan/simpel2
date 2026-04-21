// Prometheus metrics for monitoring and observability
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, register_histogram_vec,
    register_int_counter_vec, register_int_gauge_vec,
};
use std::sync::OnceLock;

/// HTTP request duration histogram (in seconds)
pub fn http_request_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "path", "status"],
            vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
            ]
        )
        .expect("Failed to register http_request_duration_seconds metric")
    })
}

/// HTTP request counter
pub fn http_requests_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "http_requests_total",
            "Total number of HTTP requests",
            &["method", "path", "status"]
        )
        .expect("Failed to register http_requests_total metric")
    })
}

/// Workflow transition counter
pub fn workflow_transitions_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "workflow_transitions_total",
            "Total number of workflow state transitions",
            &["entity_type", "from_state", "to_state", "status"]
        )
        .expect("Failed to register workflow_transitions_total metric")
    })
}

/// Workflow transition duration histogram (in seconds)
pub fn workflow_transition_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "workflow_transition_duration_seconds",
            "Workflow transition duration in seconds",
            &["entity_type", "from_state", "to_state"],
            vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
        )
        .expect("Failed to register workflow_transition_duration_seconds metric")
    })
}

/// Current workflow items by state gauge
pub fn workflow_items_by_state() -> &'static IntGaugeVec {
    static METRIC: OnceLock<IntGaugeVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_gauge_vec!(
            "workflow_items_by_state",
            "Current number of workflow items in each state",
            &["entity_type", "state"]
        )
        .expect("Failed to register workflow_items_by_state metric")
    })
}

/// SLA breach counter
pub fn workflow_sla_breaches_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "workflow_sla_breaches_total",
            "Total number of SLA breaches",
            &["entity_type", "state"]
        )
        .expect("Failed to register workflow_sla_breaches_total metric")
    })
}

/// SLA breach duration histogram (in minutes)
pub fn workflow_sla_breach_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "workflow_sla_breach_duration_minutes",
            "SLA breach duration in minutes",
            &["entity_type", "state"],
            vec![
                10.0, 30.0, 60.0, 120.0, 240.0, 480.0, 720.0, 1440.0, 2880.0, 4320.0
            ]
        )
        .expect("Failed to register workflow_sla_breach_duration_minutes metric")
    })
}

/// SLA escalations counter
pub fn workflow_escalations_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "workflow_escalations_total",
            "Total number of workflow escalations",
            &["entity_type", "state", "status"] // status: success/error
        )
        .expect("Failed to register workflow_escalations_total metric")
    })
}

/// Integration sync duration histogram (in seconds)
pub fn integration_sync_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "integration_sync_duration_seconds",
            "Integration sync duration in seconds",
            &["service", "sync_type"],
            vec![
                1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0
            ]
        )
        .expect("Failed to register integration_sync_duration_seconds metric")
    })
}

/// Integration sync counter
pub fn integration_syncs_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "integration_syncs_total",
            "Total number of integration syncs",
            &["service", "sync_type", "status"]
        )
        .expect("Failed to register integration_syncs_total metric")
    })
}

/// Integration records synced counter
pub fn integration_records_synced() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "integration_records_synced_total",
            "Total number of records synced from external systems",
            &["service", "record_type"]
        )
        .expect("Failed to register integration_records_synced_total metric")
    })
}

/// Database connection pool gauge
pub fn database_connections() -> &'static IntGaugeVec {
    static METRIC: OnceLock<IntGaugeVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_gauge_vec!(
            "database_connections",
            "Current number of database connections",
            &["state"] // active, idle, waiting
        )
        .expect("Failed to register database_connections metric")
    })
}

/// Database query duration histogram (in seconds)
pub fn database_query_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "database_query_duration_seconds",
            "Database query duration in seconds",
            &["operation"],
            vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0
            ]
        )
        .expect("Failed to register database_query_duration_seconds metric")
    })
}

/// Cache hit/miss counter
pub fn cache_operations_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "cache_operations_total",
            "Total number of cache operations",
            &["operation", "result"] // operation: get/set/delete, result: hit/miss/success/error
        )
        .expect("Failed to register cache_operations_total metric")
    })
}

/// Permit expiry reminders sent counter
pub fn permit_expiry_reminders_sent_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "permit_expiry_reminders_sent_total",
            "Total number of permit expiry reminders sent",
            &["days_remaining", "status"] // days_remaining: 30/14/7, status: success/error
        )
        .expect("Failed to register permit_expiry_reminders_sent_total metric")
    })
}

/// Permit expiry notifications sent counter
pub fn permit_expiry_notifications_sent_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "permit_expiry_notifications_sent_total",
            "Total number of permit expiry notifications sent",
            &["status"] // status: success/error
        )
        .expect("Failed to register permit_expiry_notifications_sent_total metric")
    })
}

/// Permit expiry reminder errors counter
pub fn permit_expiry_reminder_errors_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "permit_expiry_reminder_errors_total",
            "Total number of permit expiry reminder errors",
            &["error_type"] // error_type: notification_failed/database_error
        )
        .expect("Failed to register permit_expiry_reminder_errors_total metric")
    })
}

/// SLA check duration histogram (in seconds)
pub fn workflow_sla_check_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "workflow_sla_check_duration_seconds",
            "SLA check duration in seconds",
            &["workflow_type"], // workflow_type: all/kebutuhan_bmn/pemakaian_bmn/penghapusan_bmn
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
        )
        .expect("Failed to register workflow_sla_check_duration_seconds metric")
    })
}

/// SLA check counter
pub fn workflow_sla_check_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            "workflow_sla_check_total",
            "Total number of SLA checks performed",
            &["status"] // status: success/error
        )
        .expect("Failed to register workflow_sla_check_total metric")
    })
}

/// Export metrics in Prometheus text format
pub fn export_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        // Test that all metrics can be accessed without panicking
        let _ = http_request_duration();
        let _ = http_requests_total();
        let _ = workflow_transitions_total();
        let _ = workflow_transition_duration();
        let _ = workflow_items_by_state();
        let _ = workflow_sla_breaches_total();
        let _ = workflow_sla_breach_duration();
        let _ = workflow_escalations_total();
        let _ = workflow_sla_check_duration();
        let _ = workflow_sla_check_total();
        let _ = integration_sync_duration();
        let _ = integration_syncs_total();
        let _ = integration_records_synced();
        let _ = database_connections();
        let _ = database_query_duration();
        let _ = cache_operations_total();
        let _ = permit_expiry_reminders_sent_total();
        let _ = permit_expiry_notifications_sent_total();
        let _ = permit_expiry_reminder_errors_total();
    }

    #[test]
    fn test_export_metrics() {
        // Ensure metrics are registered and have a value
        let metric = http_requests_total();
        metric.with_label_values(&["GET", "/test", "200"]).inc();

        // Test that metrics can be exported
        let result = export_metrics();
        assert!(result.is_ok());
        let metrics_text = result.unwrap();
        assert!(!metrics_text.is_empty());
    }
}
