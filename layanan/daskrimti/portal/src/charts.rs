use crate::error::DashboardError;
use crate::models::Chart;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait ChartServiceTrait {
    async fn create_chart(
        &self,
        dashboard_id: Uuid,
        name: &str,
        chart_type: &str,
        data_source: &str,
        config: Value,
    ) -> Result<Chart, DashboardError>;
    async fn get_chart(&self, id: Uuid) -> Result<Chart, DashboardError>;
    async fn update_chart(
        &self,
        id: Uuid,
        name: Option<&str>,
        chart_type: Option<&str>,
        data_source: Option<&str>,
        config: Option<Value>,
    ) -> Result<Chart, DashboardError>;
    async fn delete_chart(&self, id: Uuid) -> Result<(), DashboardError>;
    async fn get_charts_by_dashboard(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<Chart>, DashboardError>;
    async fn get_chart_data(&self, id: Uuid) -> Result<Value, DashboardError>;
}

#[allow(dead_code)]
pub struct ChartService {
    #[allow(dead_code)]
    pool: Pool,
}

impl ChartService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChartServiceTrait for ChartService {
    async fn create_chart(
        &self,
        dashboard_id: Uuid,
        name: &str,
        chart_type: &str,
        data_source: &str,
        config: Value,
    ) -> Result<Chart, DashboardError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client.query_one(
            "INSERT INTO charts (id, dashboard_id, name, chart_type, data_source, config, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
            &[&id, &dashboard_id, &name, &chart_type, &data_source, &config, &now, &now]
        ).await?;

        Ok(Chart::from(&row))
    }

    async fn get_chart(&self, id: Uuid) -> Result<Chart, DashboardError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT * FROM charts WHERE id = $1", &[&id])
            .await?;
        Ok(Chart::from(&row))
    }

    async fn update_chart(
        &self,
        id: Uuid,
        name: Option<&str>,
        chart_type: Option<&str>,
        data_source: Option<&str>,
        config: Option<Value>,
    ) -> Result<Chart, DashboardError> {
        let client = self.pool.get().await?;
        let now = chrono::Utc::now();

        let row = client.query_one(
            "UPDATE charts SET name = COALESCE($2, name), chart_type = COALESCE($3, chart_type), data_source = COALESCE($4, data_source), config = COALESCE($5, config), updated_at = $6 WHERE id = $1 RETURNING *",
            &[&id, &name, &chart_type, &data_source, &config, &now]
        ).await?;

        Ok(Chart::from(&row))
    }

    async fn delete_chart(&self, id: Uuid) -> Result<(), DashboardError> {
        let client = self.pool.get().await?;
        client
            .execute("DELETE FROM charts WHERE id = $1", &[&id])
            .await?;
        Ok(())
    }

    async fn get_charts_by_dashboard(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<Chart>, DashboardError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM charts WHERE dashboard_id = $1 ORDER BY created_at",
                &[&dashboard_id],
            )
            .await?;
        let charts = rows.iter().map(Chart::from).collect();
        Ok(charts)
    }

    async fn get_chart_data(&self, id: Uuid) -> Result<Value, DashboardError> {
        let chart = self.get_chart(id).await?;

        // This would typically query the data source and format it for the chart
        // For now, return a placeholder
        Ok(serde_json::json!({
            "chart_type": chart.chart_type,
            "data": [],
            "config": chart.config
        }))
    }
}
