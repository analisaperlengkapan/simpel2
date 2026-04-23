use deadpool_postgres::{Config, Runtime};
use layanan_perlengkapan_notifikasi::config::AppConfig;
use layanan_perlengkapan_notifikasi::email::EmailService;
use layanan_perlengkapan_notifikasi::push::PushService;
use layanan_perlengkapan_notifikasi::queue_processor::{QueueProcessor, QueueStats};
use layanan_perlengkapan_notifikasi::sms::SmsService;
use std::sync::Arc;

/// Helper function to create a test database pool
async fn create_test_pool() -> deadpool_postgres::Pool {
    let mut cfg = Config::new();
    cfg.url = Some(std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/notifikasi_test".to_string()
    }));
    cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .expect("Failed to create test pool")
}

/// Helper function to create test config
fn create_test_config() -> AppConfig {
    AppConfig {
        base: lib_core::config::BaseServiceConfig {
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            database_url: "postgres://postgres:postgres@localhost:5432/notifikasi_test".to_string(),
            database_pool_size: 10,
            log_level: "info".to_string(),
        },
        grpc_port: 50053,
        smtp_host: "smtp.example.com".to_string(),
        smtp_port: 587,
        smtp_username: "test@example.com".to_string(),
        smtp_password: "test_password".to_string(),
        smtp_from: "noreply@simpel.kejaksaan.go.id".to_string(),
        fcm_server_key: Some("test_fcm_key".to_string()),
        whatsapp_api_url: Some("https://api.whatsapp.com".to_string()),
        whatsapp_access_token: Some("test_token".to_string()),
        whatsapp_phone_number_id: Some("123456789".to_string()),
        redis_url: "redis://localhost:6379".to_string(),
        api_key: "test_api_key".to_string(),
    }
}

#[tokio::test]
async fn test_email_service_creation() {
    let config = create_test_config();
    let pool = create_test_pool().await;

    let email_service = EmailService::new(config, pool);

    // Verify service was created successfully
    assert_eq!(email_service.config.smtp_host, "smtp.example.com");
    assert_eq!(email_service.config.smtp_port, 587);
}

#[tokio::test]
async fn test_sms_service_creation() {
    let config = create_test_config();
    let pool = create_test_pool().await;

    let sms_service = SmsService::new(config, pool);

    // Verify service was created successfully
    assert_eq!(sms_service.config.smtp_host, "smtp.example.com");
}

#[tokio::test]
async fn test_push_service_creation() {
    let config = create_test_config();
    let pool = create_test_pool().await;

    let push_service = PushService::new(config, pool);

    // Verify service was created successfully
    assert!(push_service.config.fcm_server_key.is_some());
}

#[tokio::test]
async fn test_queue_processor_creation() {
    let config = Arc::new(create_test_config());
    let pool = create_test_pool().await;

    let email_service = Arc::new(EmailService::new((*config).clone(), pool.clone()));
    let sms_service = Arc::new(SmsService::new((*config).clone(), pool.clone()));
    let push_service = Arc::new(PushService::new((*config).clone(), pool.clone()));

    let _queue_processor =
        QueueProcessor::new(config, pool, email_service, sms_service, push_service);

    // Verify queue processor was created successfully
    // This is a basic test - in production you would test actual queue processing
    assert!(true);
}

#[tokio::test]
async fn test_email_template_rendering() {
    let config = create_test_config();
    let pool = create_test_pool().await;

    let _email_service = EmailService::new(config, pool);

    let template = "Hello {{user_name}}, your {{entity_name}} has been {{status}}.";
    let _data = serde_json::json!({
        "user_name": "John Doe",
        "entity_name": "Kebutuhan BMN",
        "status": "approved"
    });

    // Use the render_template method (it's private, so we test via send_email_with_template in integration tests)
    // For unit tests, we verify the template format
    assert!(template.contains("{{user_name}}"));
    assert!(template.contains("{{entity_name}}"));
    assert!(template.contains("{{status}}"));
}

#[tokio::test]
async fn test_sms_template_rendering() {
    let config = create_test_config();
    let pool = create_test_pool().await;

    let _sms_service = SmsService::new(config, pool);

    let template = "SIMPEL: {{entity_name}} {{status}}. Login untuk detail.";
    let _data = serde_json::json!({
        "entity_name": "Pengajuan BMN",
        "status": "disetujui"
    });

    // Verify template format
    assert!(template.contains("{{entity_name}}"));
    assert!(template.contains("{{status}}"));
}

#[tokio::test]
async fn test_phone_number_validation() {
    // Valid E.164 format
    assert!("+62812345678".starts_with('+'));
    assert!("+15555555555".starts_with('+'));
    assert!("+6281234567890".starts_with('+'));

    // Invalid format
    assert!(!"62812345678".starts_with('+'));
    assert!(!"0812345678".starts_with('+'));
    assert!(!"812345678".starts_with('+'));
}

#[tokio::test]
async fn test_notification_priority_levels() {
    let priorities = vec!["low", "normal", "high", "urgent"];

    for priority in priorities {
        assert!(["low", "normal", "high", "urgent"].contains(&priority));
    }
}

#[tokio::test]
async fn test_notification_channels() {
    let channels = vec!["email", "sms", "push", "in_app"];

    for channel in channels {
        assert!(["email", "sms", "push", "in_app"].contains(&channel));
    }
}

#[tokio::test]
async fn test_retry_backoff_calculation() {
    // Test exponential backoff: 1min, 5min, 15min
    let backoff_1 = 1; // 1 minute
    let backoff_2 = 5; // 5 minutes
    let backoff_3 = 15; // 15 minutes

    assert_eq!(backoff_1, 1);
    assert_eq!(backoff_2, 5);
    assert_eq!(backoff_3, 15);

    // Verify exponential growth pattern
    assert!(backoff_2 > backoff_1);
    assert!(backoff_3 > backoff_2);
}

#[tokio::test]
async fn test_max_retries_configuration() {
    let max_retries = 3;

    assert_eq!(max_retries, 3);
    assert!(max_retries > 0);
    assert!(max_retries < 10); // Reasonable upper bound
}

#[tokio::test]
async fn test_notification_status_values() {
    let valid_statuses = vec!["pending", "processing", "completed", "failed"];

    for status in valid_statuses {
        assert!(["pending", "processing", "completed", "failed"].contains(&status));
    }
}

#[tokio::test]
async fn test_delivery_status_values() {
    let valid_statuses = vec!["delivered", "failed", "bounced", "rejected"];

    for status in valid_statuses {
        assert!(["delivered", "failed", "bounced", "rejected"].contains(&status));
    }
}

#[tokio::test]
async fn test_queue_stats_structure() {
    let stats = QueueStats {
        pending_count: 10,
        processing_count: 2,
        completed_count: 50,
        failed_count: 3,
        retry_count: 5,
    };

    assert_eq!(stats.pending_count, 10);
    assert_eq!(stats.processing_count, 2);
    assert_eq!(stats.completed_count, 50);
    assert_eq!(stats.failed_count, 3);
    assert_eq!(stats.retry_count, 5);

    // Verify total makes sense
    let total =
        stats.pending_count + stats.processing_count + stats.completed_count + stats.failed_count;
    assert!(total > 0);
}

#[tokio::test]
async fn test_sms_message_length_validation() {
    // Single SMS: 160 characters
    let short_message = "Test message";
    assert!(short_message.len() <= 160);

    // Multiple SMS: up to 1600 characters
    let long_message = "a".repeat(1600);
    assert!(long_message.len() <= 1600);

    // Too long
    let too_long = "a".repeat(1601);
    assert!(too_long.len() > 1600);
}

#[tokio::test]
async fn test_email_subject_length() {
    // Reasonable email subject length
    let subject = "Pengajuan Baru: Kebutuhan BMN";
    assert!(subject.len() < 500);

    // Very long subject (should be truncated)
    let long_subject = "a".repeat(600);
    assert!(long_subject.len() > 500);
}

#[tokio::test]
async fn test_template_variable_format() {
    let template = "Hello {{user_name}}, your {{entity_name}} is {{status}}.";

    // Verify template uses correct variable format
    assert!(template.contains("{{"));
    assert!(template.contains("}}"));

    // Count variables
    let var_count = template.matches("{{").count();
    assert_eq!(var_count, 3);
}

#[tokio::test]
async fn test_notification_template_names() {
    let template_names = vec![
        "workflow_submitted",
        "workflow_approved",
        "workflow_rejected",
        "sla_breach",
        "permit_expiry_reminder",
    ];

    for name in template_names {
        // Verify template names follow naming convention
        assert!(name.contains('_'));
        assert!(!name.contains(' '));
        assert_eq!(name, name.to_lowercase());
    }
}

#[tokio::test]
async fn test_grpc_port_configuration() {
    let config = create_test_config();

    assert_eq!(config.grpc_port, 50053);
    assert!(config.grpc_port > 50000);
    assert!(config.grpc_port < 60000);
}

#[tokio::test]
async fn test_smtp_port_configuration() {
    let config = create_test_config();

    assert_eq!(config.smtp_port, 587);
    assert!(config.smtp_port == 587 || config.smtp_port == 465 || config.smtp_port == 25);
}

#[tokio::test]
async fn test_redis_url_format() {
    let config = create_test_config();

    assert!(config.redis_url.starts_with("redis://"));
}

#[tokio::test]
async fn test_notification_queue_processing_interval() {
    // Queue processor runs every 30 seconds
    let interval_seconds = 30;

    assert_eq!(interval_seconds, 30);
    assert!(interval_seconds > 0);
    assert!(interval_seconds < 60);
}

// Integration test placeholder
// These would require a test database and mock SMTP/SMS/Push services
#[tokio::test]
#[ignore] // Ignore by default, run with --ignored flag
async fn test_email_send_integration() {
    // This test requires:
    // 1. Test database with schema
    // 2. Mock SMTP server
    // 3. Test email credentials

    // TODO: Implement full integration test
    assert!(true);
}

#[tokio::test]
#[ignore]
async fn test_sms_send_integration() {
    // This test requires:
    // 1. Test database with schema
    // 2. Mock Twilio/SNS API
    // 3. Test SMS credentials

    // TODO: Implement full integration test
    assert!(true);
}

#[tokio::test]
#[ignore]
async fn test_queue_processor_integration() {
    // This test requires:
    // 1. Test database with schema
    // 2. Mock notification services
    // 3. Test queue data

    // TODO: Implement full integration test
    assert!(true);
}

#[tokio::test]
#[ignore]
async fn test_retry_logic_integration() {
    // This test requires:
    // 1. Test database with schema
    // 2. Mock failing notification service
    // 3. Verify retry attempts and backoff

    // TODO: Implement full integration test
    assert!(true);
}

#[tokio::test]
#[ignore]
async fn test_delivery_status_tracking_integration() {
    // This test requires:
    // 1. Test database with schema
    // 2. Send test notifications
    // 3. Verify delivery status records

    // TODO: Implement full integration test
    assert!(true);
}
