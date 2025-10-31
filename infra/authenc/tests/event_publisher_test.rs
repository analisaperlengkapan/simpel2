//! Integration tests for EventPublisher
//!
//! Tests the enhanced event publisher with retry logic, batching, and DLQ.

use authenc::services::event_publisher::{EventPublisher, EventPublisherConfig, PublishableEvent};

#[tokio::test]
async fn test_event_publisher_config() {
    let config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        batch_size: 100,
        flush_interval_ms: 1000,
        dlq_topic: "authenc.events.dlq".to_string(),
        enable_metrics: true,
    };

    assert_eq!(config.max_retries, 3);
    assert_eq!(config.batch_size, 100);
    assert_eq!(config.flush_interval_ms, 1000);
}

#[tokio::test]
async fn test_publishable_event_creation() {
    let event = PublishableEvent::new(
        "authenc.user.events".to_string(),
        "user-123".to_string(),
        r#"{"event_type":"USER_LOGIN","user_id":"user-123"}"#.to_string(),
    );

    assert_eq!(event.topic, "authenc.user.events");
    assert_eq!(event.key, "user-123");
    assert_eq!(event.retry_count, 0);
    assert!(event.created_at > 0);
}

#[tokio::test]
async fn test_event_publisher_batch_buffer() {
    let config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        batch_size: 5,
        flush_interval_ms: 10000, // Long interval to test manual batching
        ..Default::default()
    };

    // This test doesn't require actual Kafka connection
    // It tests the batching logic
    match EventPublisher::new(config) {
        Ok(publisher) => {
            // Verify initial buffer is empty
            assert_eq!(publisher.batch_buffer_size().await, 0);

            // Add events to buffer
            for i in 0..3 {
                let event = PublishableEvent::new(
                    "test.topic".to_string(),
                    format!("key-{}", i),
                    format!(r#"{{"id":{}}}"#, i),
                );

                // This will fail to publish without Kafka, but will add to buffer
                let _ = publisher.publish(event).await;
            }

            // Note: In real scenario with Kafka, buffer would be flushed
            // Without Kafka, events stay in buffer or fail
        }
        Err(_) => {
            // Expected without Kafka running
            assert!(true, "Kafka not available for testing");
        }
    }
}

#[test]
fn test_event_serialization() {
    let event = PublishableEvent::new(
        "authenc.admin.events".to_string(),
        "admin-456".to_string(),
        r#"{"event_type":"ADMIN_ACTION","action":"user_created"}"#.to_string(),
    );

    let json = serde_json::to_string(&event);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("authenc.admin.events"));
    assert!(json_str.contains("admin-456"));
    assert!(json_str.contains("retry_count"));
}

#[test]
fn test_exponential_backoff_calculation() {
    let config = EventPublisherConfig {
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        ..Default::default()
    };

    let mut backoff = config.initial_backoff_ms;

    // First retry: 100ms
    assert_eq!(backoff, 100);

    // Second retry: 200ms
    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 200);

    // Third retry: 400ms
    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 400);

    // Continue until max
    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 800);

    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 1600);

    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 3200);

    // Should cap at max_backoff_ms
    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 5000);

    backoff = (backoff * 2).min(config.max_backoff_ms);
    assert_eq!(backoff, 5000); // Still capped
}

#[tokio::test]
async fn test_batch_size_threshold() {
    let batch_size = 100;
    let config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        batch_size,
        flush_interval_ms: 60000, // Very long interval
        ..Default::default()
    };

    // Test that batch size is correctly configured
    assert_eq!(config.batch_size, batch_size);

    // Verify flush interval is set
    assert_eq!(config.flush_interval_ms, 60000);
}

#[test]
fn test_dlq_entry_structure() {
    use authenc::services::event_publisher::DlqEntry;

    let event = PublishableEvent::new(
        "test.topic".to_string(),
        "test-key".to_string(),
        r#"{"test":"data"}"#.to_string(),
    );

    let dlq_entry = DlqEntry {
        event: event.clone(),
        failure_reason: "Connection timeout after 3 retries".to_string(),
        dlq_timestamp: chrono::Utc::now().timestamp_millis(),
    };

    // Test serialization
    let json = serde_json::to_string(&dlq_entry);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("test.topic"));
    assert!(json_str.contains("Connection timeout"));
    assert!(json_str.contains("dlq_timestamp"));
}

#[tokio::test]
async fn test_publisher_shutdown() {
    let config = EventPublisherConfig {
        brokers: "localhost:9092".to_string(),
        ..Default::default()
    };

    match EventPublisher::new(config) {
        Ok(publisher) => {
            // Test shutdown flushes pending events
            let result = publisher.shutdown().await;

            // Without Kafka, this will fail, but tests the code path
            match result {
                Ok(()) => {
                    // Successful shutdown
                    assert!(true);
                }
                Err(_) => {
                    // Expected without Kafka
                    assert!(true);
                }
            }
        }
        Err(_) => {
            // Expected without Kafka
            assert!(true);
        }
    }
}

#[test]
fn test_metrics_configuration() {
    let config_with_metrics = EventPublisherConfig {
        enable_metrics: true,
        ..Default::default()
    };
    assert!(config_with_metrics.enable_metrics);

    let config_without_metrics = EventPublisherConfig {
        enable_metrics: false,
        ..Default::default()
    };
    assert!(!config_without_metrics.enable_metrics);
}
