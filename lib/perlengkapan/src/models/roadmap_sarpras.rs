//! Predictive Analytics / Forecast models for Roadmap Sarpras
//!
//! Replaces the old CRUD roadmap with a forecasting engine that uses
//! historical kebutuhan_bmn data to predict future BMN needs.

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Forecast method selection
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ForecastMethod {
    /// Simple Moving Average
    #[default]
    Sma,
    /// Weighted Moving Average (recent years weighted more)
    Wma,
    /// Exponential Smoothing (Holt-Winters)
    ExponentialSmoothing,
}

/// Request to generate a forecast
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForecastRequest {
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
    pub horizon_years: Option<i32>,
    pub method: Option<ForecastMethod>,
    /// Confidence level (0.0 – 1.0), default 0.95
    pub confidence_level: Option<f64>,
}

/// A single year of historical data used as input
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct YearlyData {
    pub tahun: i32,
    pub total_kebutuhan: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub jumlah_satker: i64,
    pub estimasi_total_biaya: f64,
}

#[cfg(feature = "backend")]
impl YearlyData {
    pub fn from_row(row: &Row) -> Self {
        Self {
            tahun: row.get("tahun_anggaran"),
            total_kebutuhan: row.get::<_, i64>("total_kebutuhan"),
            total_existing: row.get::<_, i64>("total_existing"),
            total_gap: row.get::<_, i64>("total_gap"),
            jumlah_satker: row.get::<_, i64>("jumlah_satker"),
            estimasi_total_biaya: row
                .get::<_, Option<f64>>("estimasi_total_biaya")
                .unwrap_or(0.0),
        }
    }
}

/// A single predicted year produced by the forecasting engine
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PredictedYear {
    pub tahun: i32,
    pub predicted_kebutuhan: f64,
    pub predicted_gap: f64,
    pub predicted_biaya: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
}

/// Full forecast result returned by the API
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForecastResult {
    pub method: String,
    pub confidence_level: f64,
    pub historical: Vec<YearlyData>,
    pub predictions: Vec<PredictedYear>,
    pub generated_at: DateTime<Utc>,
}

/// Summary statistics for the forecast dashboard
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForecastSummary {
    pub total_historical_years: i32,
    pub total_predicted_years: i32,
    pub avg_yearly_growth_pct: f64,
    pub total_predicted_kebutuhan: f64,
    pub total_predicted_biaya: f64,
    pub trend_direction: String,
}

/// Stored forecast snapshot for compare-with-previous
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForecastSnapshot {
    pub id: Uuid,
    pub method: String,
    pub confidence_level: f64,
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
    pub predictions_json: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "backend")]
impl ForecastSnapshot {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            method: row.get("method"),
            confidence_level: row.get("confidence_level"),
            satker_id: row.get("satker_id"),
            kode_barang: row.get("kode_barang"),
            predictions_json: row.get("predictions_json"),
            created_at: row.get("created_at"),
        }
    }
}
