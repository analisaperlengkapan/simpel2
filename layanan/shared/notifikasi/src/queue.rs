use crate::error::AppError;
use redis::AsyncCommands;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueueJob {
    pub job_type: String,
    pub payload: Value,
}

pub struct QueueService {
    pub redis: redis::Client,
}

impl QueueService {
    pub fn new(redis_url: &str) -> Result<Self, AppError> {
        let redis = redis::Client::open(redis_url).map_err(|_e| AppError::Internal)?;
        Ok(Self { redis })
    }

    pub async fn enqueue_job(&self, queue: &str, job: &QueueJob) -> Result<(), AppError> {
        let mut conn = self.redis.get_async_connection().await.map_err(|_e| AppError::Internal)?;
        let data = serde_json::to_string(job).map_err(|_e| AppError::Internal)?;
        conn.rpush::<_, _, ()>(queue, data).await.map_err(|_e| AppError::Internal)?;
        Ok(())
    }

    pub async fn dequeue_job(&self, queue: &str, timeout_secs: u64) -> Result<Option<QueueJob>, AppError> {
        let mut conn = self.redis.get_async_connection().await.map_err(|_e| AppError::Internal)?;
        let res: Option<(String, String)> = timeout(Duration::from_secs(timeout_secs), conn.blpop(queue, 0.0)).await
            .map_err(|_| AppError::Internal)??;
        if let Some((_, data)) = res {
            let job: QueueJob = serde_json::from_str(&data).map_err(|_e| AppError::Internal)?;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    pub async fn ack_job(&self, _queue: &str, _job: &QueueJob) -> Result<(), AppError> {
        // No-op for simple queue
        Ok(())
    }
}
