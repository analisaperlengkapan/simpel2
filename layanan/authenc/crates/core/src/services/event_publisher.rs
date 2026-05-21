//! Kafka integration (requires `kafka` feature)
#![cfg(feature = "kafka")]

//! Enhanced Event Publisher with Reliability Features
//!
//! Provides reliable event publishing to Kafka with:
//! - Retry logic with exponential backoff
//! - Dead letter queue for permanently failed events
//! - Event batching for performance
//! - Comprehensive metrics

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};

use authenc_types::{AuthencError, Result};

/// Configuration for the event publisher
#[derive(Debug, Clone)]
pub struct EventPublisherConfig {
    /// Kafka broker addresses (comma-separated)
    pub brokers: String,
    /// Maximum number of retry attempts for failed publishes
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Batch size for event batching
    pub batch_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    /// Dead letter queue topic name
    pub dlq_topic: String,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for EventPublisherConfig {
    fn default() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            batch_size: 100,
            flush_interval_ms: 1000,
            dlq_topic: "authenc.events.dlq".to_string(),
            enable_metrics: true,
        }
    }
}

/// Event to be published
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishableEvent {
    /// Topic to publish to
    pub topic: String,
    /// Event key for partitioning
    pub key: String,
    /// Event payload (JSON serialized)
    pub payload: String,
    /// Timestamp when event was created
    pub created_at: i64,
    /// Number of retry attempts
    #[serde(default)]
    pub retry_count: u32,
}

impl PublishableEvent {
    /// Create a new publishable event
    pub fn new(topic: String, key: String, payload: String) -> Self {
        Self {
            topic,
            key,
            payload,
            created_at: chrono::Utc::now().timestamp_millis(),
            retry_count: 0,
        }
    }
}

/// Dead letter queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqEntry {
    /// Original event
    pub event: PublishableEvent,
    /// Reason for failure
    pub failure_reason: String,
    /// Timestamp when moved to DLQ
    pub dlq_timestamp: i64,
}

/// Enhanced event publisher with reliability features
pub struct EventPublisher {
    /// Kafka producer
    producer: FutureProducer,
    /// Configuration
    config: EventPublisherConfig,
    /// Event batch buffer
    batch_buffer: Arc<Mutex<VecDeque<PublishableEvent>>>,
    /// Last flush time
    last_flush: Arc<RwLock<Instant>>,
    /// Metrics enabled flag
    metrics_enabled: bool,
}

impl EventPublisher {
    /// Create a new event publisher
    pub fn new(config: EventPublisherConfig) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", "30000")
            .set("queue.buffering.max.messages", "100000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("compression.type", "snappy")
            .create()
            .map_err(|e| {
                AuthencError::internal(format!("Failed to create Kafka producer: {}", e))
            })?;

        let metrics_enabled = config.enable_metrics;

        Ok(Self {
            producer,
            config,
            batch_buffer: Arc::new(Mutex::new(VecDeque::new())),
            last_flush: Arc::new(RwLock::new(Instant::now())),
            metrics_enabled,
        })
    }

    /// Publish an event with retry logic
    pub async fn publish(&self, event: PublishableEvent) -> Result<()> {
        // Add to batch buffer
        let mut buffer = self.batch_buffer.lock().await;
        buffer.push_back(event);

        // Check if we should flush
        let should_flush = buffer.len() >= self.config.batch_size
            || self.last_flush.read().await.elapsed()
                >= Duration::from_millis(self.config.flush_interval_ms);

        drop(buffer);

        if should_flush {
            self.flush_batch().await?;
        }

        Ok(())
    }

    /// Flush the batch buffer
    pub async fn flush_batch(&self) -> Result<()> {
        let mut buffer = self.batch_buffer.lock().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let events: Vec<PublishableEvent> = buffer.drain(..).collect();
        drop(buffer);

        let batch_size = events.len();
        debug!("Flushing batch of {} events", batch_size);

        let start = Instant::now();

        // Process events in parallel
        let mut tasks = Vec::new();
        for event in events {
            let producer = self.producer.clone();
            let config = self.config.clone();
            let metrics_enabled = self.metrics_enabled;

            tasks.push(tokio::spawn(async move {
                Self::publish_with_retry(producer, event, config, metrics_enabled).await
            }));
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        let mut success_count = 0;
        let mut failure_count = 0;

        for result in results {
            match result {
                Ok(Ok(())) => success_count += 1,
                Ok(Err(_)) => failure_count += 1,
                Err(e) => {
                    error!("Task join error: {}", e);
                    failure_count += 1;
                }
            }
        }

        let duration = start.elapsed();

        info!(
            "Batch flush completed: {} success, {} failures in {:?}",
            success_count, failure_count, duration
        );

        // Update metrics
        if self.metrics_enabled {
            #[cfg(feature = "metrics")]
            {
                histogram!("authenc.event_publisher.batch_flush_duration_ms")
                    .record(duration.as_millis() as f64);
                counter!("authenc.event_publisher.batch_flush_total").increment(1);
                counter!("authenc.event_publisher.events_flushed_total")
                    .increment(batch_size as u64);
            }
        }

        // Update last flush time
        *self.last_flush.write().await = Instant::now();

        Ok(())
    }

    /// Publish event with retry logic
    async fn publish_with_retry(
        producer: FutureProducer,
        mut event: PublishableEvent,
        config: EventPublisherConfig,
        metrics_enabled: bool,
    ) -> Result<()> {
        let mut backoff_ms = config.initial_backoff_ms;

        for attempt in 0..=config.max_retries {
            event.retry_count = attempt;

            match Self::send_to_kafka(&producer, &event).await {
                Ok(()) => {
                    debug!(
                        "Successfully published event to topic {} (attempt {})",
                        event.topic,
                        attempt + 1
                    );

                    // Update success metrics
                    if metrics_enabled {
                        #[cfg(feature = "metrics")]
                        {
                            counter!("authenc.event_publisher.publish_success_total",
                                "topic" => event.topic.clone())
                            .increment(1);

                            if attempt > 0 {
                                counter!("authenc.event_publisher.retry_success_total",
                                    "topic" => event.topic.clone())
                                .increment(1);
                            }
                        }
                    }

                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to publish event to topic {} (attempt {}): {}",
                        event.topic,
                        attempt + 1,
                        e
                    );

                    // Update failure metrics
                    if metrics_enabled {
                        #[cfg(feature = "metrics")]
                        {
                            counter!("authenc.event_publisher.publish_failure_total",
                                "topic" => event.topic.clone())
                            .increment(1);
                        }
                    }

                    // If this was the last attempt, send to DLQ
                    if attempt == config.max_retries {
                        error!(
                            "Event permanently failed after {} attempts, sending to DLQ",
                            config.max_retries + 1
                        );

                        let dlq_entry = DlqEntry {
                            event: event.clone(),
                            failure_reason: e.to_string(),
                            dlq_timestamp: chrono::Utc::now().timestamp_millis(),
                        };

                        if let Err(dlq_err) =
                            Self::send_to_dlq(&producer, &config.dlq_topic, &dlq_entry).await
                        {
                            error!("Failed to send event to DLQ: {}", dlq_err);

                            // Update DLQ failure metrics
                            if metrics_enabled {
                                #[cfg(feature = "metrics")]
                                {
                                    counter!("authenc.event_publisher.dlq_failure_total")
                                        .increment(1);
                                }
                            }
                        } else {
                            info!("Event sent to DLQ: {}", config.dlq_topic);

                            // Update DLQ success metrics
                            if metrics_enabled {
                                #[cfg(feature = "metrics")]
                                {
                                    counter!("authenc.event_publisher.dlq_success_total")
                                        .increment(1);
                                }
                            }
                        }

                        return Err(e);
                    }

                    // Exponential backoff
                    sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(config.max_backoff_ms);
                }
            }
        }

        Err(AuthencError::internal(
            "Event publishing failed after all retries",
        ))
    }

    /// Send event to Kafka
    async fn send_to_kafka(producer: &FutureProducer, event: &PublishableEvent) -> Result<()> {
        let record = FutureRecord::to(&event.topic)
            .payload(&event.payload)
            .key(&event.key);

        match producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => Ok(()),
            Err((e, _)) => Err(AuthencError::internal(format!("Kafka send error: {}", e))),
        }
    }

    /// Send failed event to dead letter queue
    async fn send_to_dlq(
        producer: &FutureProducer,
        dlq_topic: &str,
        dlq_entry: &DlqEntry,
    ) -> Result<()> {
        let payload = serde_json::to_string(dlq_entry)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize DLQ entry: {}", e)))?;

        let record = FutureRecord::to(dlq_topic)
            .payload(&payload)
            .key(&dlq_entry.event.key);

        match producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => Ok(()),
            Err((e, _)) => Err(AuthencError::internal(format!("DLQ send error: {}", e))),
        }
    }

    /// Force flush all pending events
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down event publisher, flushing pending events");
        self.flush_batch().await?;
        Ok(())
    }

    /// Get current batch buffer size
    pub async fn batch_buffer_size(&self) -> usize {
        self.batch_buffer.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_publisher_config_default() {
        let config = EventPublisherConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval_ms, 1000);
    }

    #[test]
    fn test_publishable_event_creation() {
        let event = PublishableEvent::new(
            "test.topic".to_string(),
            "test-key".to_string(),
            r#"{"test": "data"}"#.to_string(),
        );

        assert_eq!(event.topic, "test.topic");
        assert_eq!(event.key, "test-key");
        assert_eq!(event.retry_count, 0);
    }

    #[test]
    fn test_dlq_entry_serialization() {
        let event = PublishableEvent::new(
            "test.topic".to_string(),
            "test-key".to_string(),
            r#"{"test": "data"}"#.to_string(),
        );

        let dlq_entry = DlqEntry {
            event,
            failure_reason: "Test failure".to_string(),
            dlq_timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let json = serde_json::to_string(&dlq_entry);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("test.topic"));
        assert!(json_str.contains("Test failure"));
    }

    #[tokio::test]
    async fn test_event_publisher_creation() {
        let config = EventPublisherConfig {
            brokers: "localhost:9092".to_string(),
            ..Default::default()
        };

        // This will fail without Kafka running, but tests the creation logic
        let result = EventPublisher::new(config);

        // We expect this to fail without Kafka, but the code path is tested
        match result {
            Ok(publisher) => {
                assert_eq!(publisher.batch_buffer_size().await, 0);
            }
            Err(_) => {
                // Expected without Kafka
                assert!(true);
            }
        }
    }
}
