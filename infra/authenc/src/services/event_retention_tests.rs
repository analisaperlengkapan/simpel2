#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ColdStorageConfig, EventsConfig};

    #[test]
    fn test_retention_cleanup_result_default() {
        let result = RetentionCleanupResult::default();
        assert_eq!(result.user_events_deleted, 0);
        assert_eq!(result.admin_events_deleted, 0);
        assert_eq!(result.total_events_deleted, 0);
        assert_eq!(result.user_events_archived, 0);
        assert_eq!(result.admin_events_archived, 0);
        assert_eq!(result.total_events_archived, 0);
    }

    #[test]
    fn test_cold_storage_config_default() {
        let config = ColdStorageConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.storage_type, "s3");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.path_prefix, "authenc/events/archive");
        assert!(!config.force_path_style);
    }

    #[test]
    fn test_events_config_with_cold_storage() {
        let cold_storage = ColdStorageConfig {
            enabled: true,
            storage_type: "minio".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            region: "us-east-1".to_string(),
            bucket: "authenc-events".to_string(),
            access_key_id: Some("minioadmin".to_string()),
            secret_access_key: Some("minioadmin".to_string()),
            path_prefix: "archive".to_string(),
            force_path_style: true,
        };

        let config = EventsConfig {
            enabled: true,
            user_event_retention_days: 90,
            admin_event_retention_days: 365,
            max_cleanup_batch_size: 10000,
            cleanup_interval_hours: 24,
            archive_before_delete: true,
            archive_directory: None,
            cold_storage: Some(cold_storage),
        };

        assert!(config.enabled);
        assert!(config.archive_before_delete);
        assert!(config.cold_storage.is_some());

        let cs = config.cold_storage.unwrap();
        assert!(cs.enabled);
        assert_eq!(cs.storage_type, "minio");
        assert_eq!(cs.bucket, "authenc-events");
        assert!(cs.force_path_style);
    }

    #[test]
    fn test_retention_stats_metrics() {
        let stats = RetentionStats {
            total_user_events: 1000,
            total_admin_events: 500,
            expired_user_events: 100,
            expired_admin_events: 50,
            user_retention_days: 90,
            admin_retention_days: 365,
            cleanup_interval_hours: 24,
            last_cleanup_check: Utc::now(),
            cold_storage_enabled: true,
            cold_storage_bucket: Some("test-bucket".to_string()),
        };

        // This should not panic
        stats.update_metrics();

        assert_eq!(stats.total_user_events, 1000);
        assert_eq!(stats.expired_user_events, 100);
        assert!(stats.cold_storage_enabled);
    }
}
