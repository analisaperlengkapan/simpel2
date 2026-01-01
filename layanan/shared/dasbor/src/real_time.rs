use crate::error::DashboardError;
use crate::models::RealTimeUpdate;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use redis::AsyncCommands;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait RealTimeServiceTrait {
    async fn publish_update(
        &self,
        metric_name: &str,
        value: f64,
        source: &str,
    ) -> Result<(), DashboardError>;
    async fn subscribe_to_metric(&self, metric_name: &str) -> Result<Value, DashboardError>;
    async fn get_recent_updates(
        &self,
        metric_name: &str,
        limit: i64,
    ) -> Result<Vec<RealTimeUpdate>, DashboardError>;
    async fn get_active_subscriptions(&self) -> Result<Value, DashboardError>;
}

pub struct RealTimeService {
    #[allow(dead_code)]
    pool: Pool,
    #[allow(dead_code)]
    redis_url: String,
}

impl RealTimeService {
    pub fn new(pool: Pool, redis_url: String) -> Self {
        Self { pool, redis_url }
    }

    #[allow(dead_code)]
    async fn get_redis_client(&self) -> Result<redis::Client, DashboardError> {
        let client = redis::Client::open(self.redis_url.as_str())?;
        Ok(client)
    }
}

#[async_trait]
impl RealTimeServiceTrait for RealTimeService {
    async fn publish_update(
        &self,
        metric_name: &str,
        value: f64,
        source: &str,
    ) -> Result<(), DashboardError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Store in database
        client.execute(
            "INSERT INTO real_time_updates (id, metric_name, value, timestamp, source) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &metric_name, &value, &now, &source]
        ).await?;

        // Publish to Redis for real-time subscribers
        let redis_client = self.get_redis_client().await?;
        let mut conn = redis_client.get_multiplexed_tokio_connection().await?;
        let update_data = serde_json::json!({
            "id": id,
            "metric_name": metric_name,
            "value": value,
            "timestamp": now,
            "source": source
        });

        conn.publish::<_, _, ()>(format!("metric:{}", metric_name), update_data.to_string())
            .await?;

        Ok(())
    }

    async fn subscribe_to_metric(&self, metric_name: &str) -> Result<Value, DashboardError> {
        let redis_client = self.get_redis_client().await?;

        // For pubsub in redis 0.32+, get async pubsub directly
        let mut pubsub = redis_client.get_async_pubsub().await?;
        pubsub.subscribe(format!("metric:{}", metric_name)).await?;

        Ok(serde_json::json!({
            "subscription": "active",
            "metric": metric_name,
            "channel": format!("metric:{}", metric_name)
        }))
    }

    async fn get_recent_updates(
        &self,
        metric_name: &str,
        limit: i64,
    ) -> Result<Vec<RealTimeUpdate>, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT * FROM real_time_updates WHERE metric_name = $1 ORDER BY timestamp DESC LIMIT $2",
            &[&metric_name, &limit]
        ).await?;

        let updates = rows.iter().map(RealTimeUpdate::from).collect();
        Ok(updates)
    }

    async fn get_active_subscriptions(&self) -> Result<Value, DashboardError> {
        let redis_client = self.get_redis_client().await?;
        let mut conn = redis_client.get_multiplexed_tokio_connection().await?;

        // Get info about active subscriptions (simplified)
        let info: redis::Value = redis::cmd("INFO").query_async(&mut conn).await?;
        let info_str = format!("{:?}", info);

        Ok(serde_json::json!({
            "active_subscriptions": "unknown",
            "status": "active",
            "info": info_str
        }))
    }
}
