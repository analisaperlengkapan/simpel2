use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client as S3Client, config::Region};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::time::{Duration as TokioDuration, interval};

use crate::config::{ColdStorageConfig, EventsConfig};
use crate::database::Database;
use crate::error::{AuthencError as Error, Result};
use crate::services::events::EventStoreProvider;

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};

/// Event retention policy service
pub struct EventRetentionService {
    config: EventsConfig,
    database: Arc<Database>,
    event_store: Arc<dyn EventStoreProvider>,
    s3_client: Option<S3Client>,
}

impl EventRetentionService {
    /// Create a new event retention service
    pub async fn new(
        config: EventsConfig,
        database: Arc<Database>,
        event_store: Arc<dyn EventStoreProvider>,
    ) -> Result<Self> {
        let s3_client = if let Some(ref cold_storage) = config.cold_storage {
            if cold_storage.enabled {
                Some(Self::create_s3_client(cold_storage).await?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            config,
            database,
            event_store,
            s3_client,
        })
    }

    async fn create_s3_client(cold_storage: &ColdStorageConfig) -> Result<S3Client> {
        let mut config_builder = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(cold_storage.region.clone()))
            .force_path_style(cold_storage.force_path_style);

        // Set custom endpoint for MinIO or custom S3-compatible storage
        if !cold_storage.endpoint.is_empty() {
            config_builder = config_builder.endpoint_url(&cold_storage.endpoint);
        }

        // Set credentials if provided
        if let (Some(access_key), Some(secret_key)) =
            (&cold_storage.access_key_id, &cold_storage.secret_access_key)
        {
            let credentials =
                Credentials::new(access_key, secret_key, None, None, "authenc-cold-storage");
            config_builder = config_builder.credentials_provider(credentials);
        }

        let config = config_builder.build();
        Ok(S3Client::from_conf(config))
    }

    /// Start the retention cleanup task
    /// Create S3 client from cold storage configuration
    pub fn start_cleanup_task(self: Arc<Self>) {
        if !self.config.enabled {
            tracing::info!("Event retention is disabled, skipping cleanup task");
            return;
        }

        let cleanup_interval =
            TokioDuration::from_secs(self.config.cleanup_interval_hours as u64 * 3600);
        let self_clone = Arc::clone(&self);

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

            loop {
                interval.tick().await;

                if let Err(e) = self_clone.perform_cleanup().await {
                    tracing::error!("Event retention cleanup failed: {}", e);
                }
            }
        });

        tracing::info!(
            "Event retention cleanup task started with interval: {} hours",
            self.config.cleanup_interval_hours
        );
    }

    /// Perform cleanup of old events
    pub async fn perform_cleanup(&self) -> Result<RetentionCleanupResult> {
        if !self.config.enabled {
            return Ok(RetentionCleanupResult::default());
        }

        let start_time = std::time::Instant::now();
        let now = Utc::now();
        let mut total_deleted = 0;
        let mut total_archived = 0;

        // Clean up user events
        let user_cutoff = now - Duration::days(self.config.user_event_retention_days as i64);
        let (user_deleted, user_archived) = self.cleanup_user_events(user_cutoff).await?;
        total_deleted += user_deleted;
        total_archived += user_archived;

        // Clean up admin events
        let admin_cutoff = now - Duration::days(self.config.admin_event_retention_days as i64);
        let (admin_deleted, admin_archived) = self.cleanup_admin_events(admin_cutoff).await?;
        total_deleted += admin_deleted;
        total_archived += admin_archived;

        let duration = start_time.elapsed();

        let result = RetentionCleanupResult {
            user_events_deleted: user_deleted,
            admin_events_deleted: admin_deleted,
            total_events_deleted: total_deleted,
            user_events_archived: user_archived,
            admin_events_archived: admin_archived,
            total_events_archived: total_archived,
            cleanup_time: now,
            duration_ms: duration.as_millis() as u64,
        };

        tracing::info!(
            "Event retention cleanup completed: {} user events deleted, {} admin events deleted, {} total archived in {:?}",
            user_deleted,
            admin_deleted,
            total_archived,
            duration
        );

        // Update metrics
        #[cfg(feature = "metrics")]
        {
            counter!("authenc.event_retention.cleanup_total").increment(1);
            counter!("authenc.event_retention.events_deleted_total")
                .increment(total_deleted as u64);
            counter!("authenc.event_retention.events_archived_total")
                .increment(total_archived as u64);
            histogram!("authenc.event_retention.cleanup_duration_ms")
                .record(duration.as_millis() as f64);
        }

        Ok(result)
    }

    async fn cleanup_user_events(&self, cutoff_date: DateTime<Utc>) -> Result<(usize, usize)> {
        let mut archived = 0;

        // Archive events if cold storage is enabled
        if self.config.archive_before_delete && self.s3_client.is_some() {
            archived = self.archive_events("events", cutoff_date).await?;
        }

        // Delete old events
        let deleted = self.event_store.clear_old_events(cutoff_date).await?;

        Ok((deleted, archived))
    }

    /// Clean up old admin events
    async fn cleanup_admin_events(&self, cutoff_date: DateTime<Utc>) -> Result<(usize, usize)> {
        let mut archived = 0;

        // Archive events if cold storage is enabled
        if self.config.archive_before_delete && self.s3_client.is_some() {
            archived = self.archive_events("admin_events", cutoff_date).await?;
        }

        // Delete old events
        let query = "DELETE FROM admin_events WHERE time < $1";
        let result = self
            .database
            .execute(query, &[&cutoff_date])
            .await
            .map_err(|e| Error::database(e.to_string()))?;

        Ok((result as usize, archived))
    }

    /// Archive events to cold storage (S3/MinIO)
    async fn archive_events(&self, table_name: &str, cutoff_date: DateTime<Utc>) -> Result<usize> {
        let s3_client = match &self.s3_client {
            Some(client) => client,
            None => {
                tracing::warn!("S3 client not configured, skipping archiving");
                return Ok(0);
            }
        };

        let cold_storage = match &self.config.cold_storage {
            Some(config) => config,
            None => {
                tracing::warn!("Cold storage not configured, skipping archiving");
                return Ok(0);
            }
        };

        tracing::info!(
            "Archiving events from {} older than {}",
            table_name,
            cutoff_date
        );

        // Fetch events to archive in batches
        let batch_size = self.config.max_cleanup_batch_size as i64;
        let mut total_archived = 0;
        let mut offset = 0i64;

        loop {
            let query = format!(
                "SELECT * FROM {} WHERE time < $1 ORDER BY time LIMIT $2 OFFSET $3",
                table_name
            );

            let rows = self
                .database
                .query(&query, &[&cutoff_date, &batch_size, &offset])
                .await
                .map_err(|e| Error::database(e.to_string()))?;

            if rows.is_empty() {
                break;
            }

            // Convert rows to JSON
            let events: Vec<serde_json::Value> = rows
                .iter()
                .map(|row: &tokio_postgres::Row| {
                    let mut event = serde_json::Map::new();
                    for (idx, column) in row.columns().iter().enumerate() {
                        let value = match column.type_().name() {
                            "uuid" => match row.try_get::<_, uuid::Uuid>(idx) {
                                Ok(val) => serde_json::Value::String(val.to_string()),
                                Err(_) => serde_json::Value::Null,
                            },
                            "timestamptz" | "timestamp" => {
                                match row.try_get::<_, DateTime<Utc>>(idx) {
                                    Ok(val) => serde_json::Value::String(val.to_rfc3339()),
                                    Err(_) => serde_json::Value::Null,
                                }
                            }
                            "text" | "varchar" => match row.try_get::<_, String>(idx) {
                                Ok(val) => serde_json::Value::String(val),
                                Err(_) => serde_json::Value::Null,
                            },
                            "int4" => match row.try_get::<_, i32>(idx) {
                                Ok(val) => serde_json::Value::Number((val as i64).into()),
                                Err(_) => serde_json::Value::Null,
                            },
                            "int8" => match row.try_get::<_, i64>(idx) {
                                Ok(val) => serde_json::Value::Number(val.into()),
                                Err(_) => serde_json::Value::Null,
                            },
                            "bool" => match row.try_get::<_, bool>(idx) {
                                Ok(val) => serde_json::Value::Bool(val),
                                Err(_) => serde_json::Value::Null,
                            },
                            "jsonb" | "json" => match row.try_get::<_, serde_json::Value>(idx) {
                                Ok(val) => val,
                                Err(_) => serde_json::Value::Null,
                            },
                            _ => serde_json::Value::Null,
                        };
                        event.insert(column.name().to_string(), value);
                    }
                    serde_json::Value::Object(event)
                })
                .collect();

            // Create archive file name with timestamp
            let archive_date = cutoff_date.format("%Y-%m-%d").to_string();
            let archive_key = format!(
                "{}/{}/{}-{}.json",
                cold_storage.path_prefix, table_name, archive_date, offset
            );

            // Serialize events to JSON
            let json_data =
                serde_json::to_vec_pretty(&events).map_err(|e| Error::InternalError {
                    message: format!("Failed to serialize events: {}", e),
                })?;

            // Upload to S3/MinIO
            let put_result = s3_client
                .put_object()
                .bucket(&cold_storage.bucket)
                .key(&archive_key)
                .body(json_data.into())
                .content_type("application/json")
                .send()
                .await;

            match put_result {
                Ok(_) => {
                    let batch_count = events.len();
                    total_archived += batch_count;
                    tracing::info!(
                        "Archived {} events from {} to {}",
                        batch_count,
                        table_name,
                        archive_key
                    );

                    #[cfg(feature = "metrics")]
                    {
                        counter!("authenc.event_retention.archive_batches_total",
                            "table" => table_name.to_string())
                        .increment(1);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to archive events to {}: {}", archive_key, e);
                    return Err(Error::InternalError {
                        message: format!("Failed to archive events: {}", e),
                    });
                }
            }

            offset += batch_size;

            // Prevent infinite loop
            if rows.len() < batch_size as usize {
                break;
            }
        }

        tracing::info!(
            "Successfully archived {} events from {} to cold storage",
            total_archived,
            table_name
        );

        Ok(total_archived)
    }

    /// Get retention statistics
    pub async fn get_retention_stats(&self) -> Result<RetentionStats> {
        let now = Utc::now();

        // Count total events
        let total_user_events = self.count_table_rows("events").await?;
        let total_admin_events = self.count_table_rows("admin_events").await?;

        // Count events older than retention periods
        let user_cutoff = now - Duration::days(self.config.user_event_retention_days as i64);
        let admin_cutoff = now - Duration::days(self.config.admin_event_retention_days as i64);

        let expired_user_events = self.count_expired_events("events", user_cutoff).await?;
        let expired_admin_events = self
            .count_expired_events("admin_events", admin_cutoff)
            .await?;

        let (cold_storage_enabled, cold_storage_bucket) =
            if let Some(ref cs) = self.config.cold_storage {
                (cs.enabled, Some(cs.bucket.clone()))
            } else {
                (false, None)
            };

        let stats = RetentionStats {
            total_user_events,
            total_admin_events,
            expired_user_events,
            expired_admin_events,
            user_retention_days: self.config.user_event_retention_days,
            admin_retention_days: self.config.admin_event_retention_days,
            cleanup_interval_hours: self.config.cleanup_interval_hours,
            last_cleanup_check: now,
            cold_storage_enabled,
            cold_storage_bucket,
        };

        // Update metrics
        stats.update_metrics();

        Ok(stats)
    }

    /// Count rows in a table
    async fn count_table_rows(&self, table_name: &str) -> Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {}", table_name);
        let row: tokio_postgres::Row = self
            .database
            .query_one(&query, &[])
            .await
            .map_err(|e| Error::database(e.to_string()))?;

        let count: i64 = row.get(0);
        Ok(count)
    }

    /// Count expired events in a table
    async fn count_expired_events(
        &self,
        table_name: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {} WHERE time < $1", table_name);
        let row: tokio_postgres::Row = self
            .database
            .query_one(&query, &[&cutoff_date])
            .await
            .map_err(|e| Error::database(e.to_string()))?;

        let count: i64 = row.get(0);
        Ok(count)
    }
}

/// Result of a retention cleanup operation
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
    /// Clean up old user events
pub struct RetentionCleanupResult {
    /// Number of user events deleted
    pub user_events_deleted: usize,
    /// Number of admin events deleted
    pub admin_events_deleted: usize,
    /// Total number of events deleted
    pub total_events_deleted: usize,
    /// Number of user events archived
    pub user_events_archived: usize,
    /// Number of admin events archived
    pub admin_events_archived: usize,
    /// Total number of events archived
    pub total_events_archived: usize,
    /// Time when cleanup was performed
    pub cleanup_time: chrono::DateTime<chrono::Utc>,
    /// Duration of cleanup operation in milliseconds
    pub duration_ms: u64,
}

/// Statistics about event retention
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionStats {
    /// Total number of user events in the system
    pub total_user_events: i64,
    /// Total number of admin events in the system
    pub total_admin_events: i64,
    /// Number of user events older than retention period
    pub expired_user_events: i64,
    /// Number of admin events older than retention period
    pub expired_admin_events: i64,
    /// User event retention period in days
    pub user_retention_days: u32,
    /// Admin event retention period in days
    pub admin_retention_days: u32,
    /// Cleanup interval in hours
    pub cleanup_interval_hours: u32,
    /// Last time retention stats were checked
    pub last_cleanup_check: chrono::DateTime<chrono::Utc>,
    /// Whether cold storage archiving is enabled
    pub cold_storage_enabled: bool,
    /// Cold storage bucket name (if enabled)
    pub cold_storage_bucket: Option<String>,
}

impl RetentionStats {
    /// Update metrics from retention stats
    pub fn update_metrics(&self) {
        #[cfg(feature = "metrics")]
        {
            use metrics::gauge;

            gauge!("authenc.event_retention.total_user_events").set(self.total_user_events as f64);
            gauge!("authenc.event_retention.total_admin_events")
                .set(self.total_admin_events as f64);
            gauge!("authenc.event_retention.expired_user_events")
                .set(self.expired_user_events as f64);
            gauge!("authenc.event_retention.expired_admin_events")
                .set(self.expired_admin_events as f64);
            gauge!("authenc.event_retention.user_retention_days")
                .set(self.user_retention_days as f64);
            gauge!("authenc.event_retention.admin_retention_days")
                .set(self.admin_retention_days as f64);
        }
    }
}
