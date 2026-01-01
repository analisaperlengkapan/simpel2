use crate::error::DashboardError;
use crate::models::{AggregatedData, Metric};
use async_trait::async_trait;
use deadpool_postgres::Pool;

#[async_trait]
#[allow(dead_code)]
pub trait AnalyticsServiceTrait {
    async fn get_metrics(&self) -> Result<Vec<Metric>, DashboardError>;
    async fn get_metric_history(&self, name: &str) -> Result<Vec<Metric>, DashboardError>;
    async fn get_aggregated_data(
        &self,
        metric_name: &str,
        aggregation_type: &str,
    ) -> Result<Vec<AggregatedData>, DashboardError>;
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        unit: &str,
        tags: serde_json::Value,
    ) -> Result<(), DashboardError>;
}

#[allow(dead_code)]
pub struct AnalyticsService {
    #[allow(dead_code)]
    pool: Pool,
}

impl AnalyticsService {
    #[allow(dead_code)]
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnalyticsServiceTrait for AnalyticsService {
    async fn get_metrics(&self) -> Result<Vec<Metric>, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM metrics ORDER BY timestamp DESC LIMIT 100",
                &[],
            )
            .await?;
        let metrics = rows.iter().map(Metric::from).collect();
        Ok(metrics)
    }

    async fn get_metric_history(&self, name: &str) -> Result<Vec<Metric>, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM metrics WHERE name = $1 ORDER BY timestamp DESC LIMIT 1000",
                &[&name],
            )
            .await?;
        let metrics = rows.iter().map(Metric::from).collect();
        Ok(metrics)
    }

    async fn get_aggregated_data(
        &self,
        metric_name: &str,
        aggregation_type: &str,
    ) -> Result<Vec<AggregatedData>, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT * FROM aggregated_data WHERE metric_name = $1 AND aggregation_type = $2 ORDER BY period_end DESC LIMIT 100",
            &[&metric_name, &aggregation_type]
        ).await?;
        let aggregated = rows.iter().map(AggregatedData::from).collect();
        Ok(aggregated)
    }

    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        unit: &str,
        tags: serde_json::Value,
    ) -> Result<(), DashboardError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO metrics (name, value, unit, tags) VALUES ($1, $2, $3, $4)",
                &[&name, &value, &unit, &tags],
            )
            .await?;
        Ok(())
    }
}
