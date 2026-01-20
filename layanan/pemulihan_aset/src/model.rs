use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use garde::Validate;
use postgres_types::{ToSql, FromSql};

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone, Copy)]
#[postgres(name = "case_status")]
#[allow(dead_code)]
pub enum CaseStatus {
    Draft,
    Process,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone, Copy)]
#[postgres(name = "asset_type")]
#[allow(dead_code)]
pub enum AssetType {
    Money,
    Land,
    Vehicle,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone, Copy)]
#[postgres(name = "asset_status")]
#[allow(dead_code)]
pub enum AssetStatus {
    Identified,
    Seized,
    Recovered,
    Auctioned,
}

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql)]
pub struct Case {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCaseRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(skip)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql)]
pub struct Asset {
    pub id: Uuid,
    pub case_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub estimated_value: Option<f64>, // Decimal in DB, f64 in Rust for simplicity (or rust_decimal if precise)
    pub status: String,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAssetRequest {
    #[garde(skip)]
    pub case_id: Uuid,
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub asset_type: String,
    #[garde(skip)]
    pub estimated_value: Option<f64>,
    #[garde(skip)]
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSql, FromSql)]
#[allow(dead_code)]
pub struct Recovery {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub amount: f64,
    pub recovery_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[allow(dead_code)]
pub struct CreateRecoveryRequest {
    #[garde(skip)]
    pub asset_id: Uuid,
    #[garde(range(min = 0.0))]
    pub amount: f64,
    #[garde(skip)]
    pub notes: Option<String>,
}
