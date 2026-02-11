// Prometheus metrics for integration service monitoring
use prometheus::{
    register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec, TextEncoder,
    Encoder,
};
use std::sync::OnceLock;

/// Integration sync duration histogram (in seconds)
pub fn integration_sync_duration() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            "integration_sync_duration_seconds",
            "Integration sync duration in seconds",
            &["service", "sync_type"],
            vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0]
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
        let _ = integration_sync_duration();
        let _ = integration_syncs_total();
        let _ = integration_records_synced();
    }

    #[test]
    fn test_export_metrics() {
        // Test that metrics can be exported
        let result = export_metrics();
        assert!(result.is_ok());
        let metrics_text = result.unwrap();
        assert!(!metrics_text.is_empty());
    }
}
