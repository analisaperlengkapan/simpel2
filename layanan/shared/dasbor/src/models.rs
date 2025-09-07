use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Row> for Dashboard {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            user_id: row.get("user_id"),
            config: row.get("config"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chart {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub name: String,
    pub chart_type: String,
    pub data_source: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Row> for Chart {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            dashboard_id: row.get("dashboard_id"),
            name: row.get("name"),
            chart_type: row.get("chart_type"),
            data_source: row.get("data_source"),
            config: row.get("config"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metric {
    pub id: Uuid,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub tags: serde_json::Value,
}

impl From<&Row> for Metric {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            value: row.get("value"),
            unit: row.get("unit"),
            timestamp: row.get("timestamp"),
            tags: row.get("tags"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatedData {
    pub id: Uuid,
    pub metric_name: String,
    pub aggregation_type: String,
    pub value: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for AggregatedData {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            metric_name: row.get("metric_name"),
            aggregation_type: row.get("aggregation_type"),
            value: row.get("value"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealTimeUpdate {
    pub id: Uuid,
    pub metric_name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl From<&Row> for RealTimeUpdate {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            metric_name: row.get("metric_name"),
            value: row.get("value"),
            timestamp: row.get("timestamp"),
            source: row.get("source"),
        }
    }
}
