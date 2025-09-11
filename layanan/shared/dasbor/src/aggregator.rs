use crate::error::DashboardError;
use crate::models::AggregatedData;
use async_trait::async_trait;
use deadpool_postgres::Pool;

#[async_trait]
pub trait AggregatorServiceTrait {
    async fn aggregate_metrics(
        &self,
        metric_name: &str,
        aggregation_type: &str,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<AggregatedData, DashboardError>;
    async fn batch_aggregate(
        &self,
        metrics: Vec<String>,
        aggregation_type: &str,
    ) -> Result<Vec<AggregatedData>, DashboardError>;
    async fn get_aggregation_status(&self) -> Result<serde_json::Value, DashboardError>;
}

pub struct AggregatorService {
    pool: Pool,
}

impl AggregatorService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    fn calculate_aggregation(&self, values: &[f64], aggregation_type: &str) -> f64 {
        match aggregation_type {
            "sum" => values.iter().sum(),
            "avg" => values.iter().sum::<f64>() / values.len() as f64,
            "min" => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            "max" => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            "count" => values.len() as f64,
            _ => values.iter().sum::<f64>() / values.len() as f64,
        }
    }
}

#[async_trait]
impl AggregatorServiceTrait for AggregatorService {
    async fn aggregate_metrics(
        &self,
        metric_name: &str,
        aggregation_type: &str,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<AggregatedData, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT value FROM metrics WHERE name = $1 AND timestamp >= $2 AND timestamp <= $3",
                &[&metric_name, &period_start, &period_end],
            )
            .await?;

        let values: Vec<f64> = rows.iter().map(|row| row.get("value")).collect();
        let aggregated_value = self.calculate_aggregation(&values, aggregation_type);

        let aggregated = AggregatedData {
            id: uuid::Uuid::new_v4(),
            metric_name: metric_name.to_string(),
            aggregation_type: aggregation_type.to_string(),
            value: aggregated_value,
            period_start,
            period_end,
            created_at: chrono::Utc::now(),
        };

        client.execute(
            "INSERT INTO aggregated_data (metric_name, aggregation_type, value, period_start, period_end) VALUES ($1, $2, $3, $4, $5)",
            &[&aggregated.metric_name, &aggregated.aggregation_type, &aggregated.value, &aggregated.period_start, &aggregated.period_end]
        ).await?;

        Ok(aggregated)
    }

    async fn batch_aggregate(
        &self,
        metrics: Vec<String>,
        aggregation_type: &str,
    ) -> Result<Vec<AggregatedData>, DashboardError> {
        let mut results = Vec::new();
        let now = chrono::Utc::now();
        let period_start = now - chrono::Duration::hours(1);
        let period_end = now;

        for metric_name in metrics {
            let result = self
                .aggregate_metrics(&metric_name, aggregation_type, period_start, period_end)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn get_aggregation_status(&self) -> Result<serde_json::Value, DashboardError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT COUNT(*) as total_aggregated FROM aggregated_data",
                &[],
            )
            .await?;
        let total_aggregated: i64 = row.get("total_aggregated");

        Ok(serde_json::json!({
            "status": "active",
            "total_aggregated": total_aggregated,
            "last_updated": chrono::Utc::now()
        }))
    }
}
