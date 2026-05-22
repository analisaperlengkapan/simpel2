//! Predictive Analytics API models
//!
//! API-level request/response types for the forecast endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export domain models from lib-perlengkapan
pub use lib_perlengkapan::models::{
    ForecastMethod, ForecastRequest, ForecastResult, ForecastSnapshot,
};

/// Query parameters for GET /forecast
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastQuery {
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
    /// Number of years to predict (default: 5)
    pub horizon: Option<i32>,
    /// Forecasting method: sma | wma | exponential
    pub method: Option<String>,
    /// Confidence level 0.0–1.0 (default: 0.95)
    pub confidence: Option<f64>,
}

impl ForecastQuery {
    pub fn into_request(self) -> ForecastRequest {
        let method = match self.method.as_deref() {
            Some("wma") => Some(ForecastMethod::Wma),
            Some("exponential") => Some(ForecastMethod::ExponentialSmoothing),
            _ => Some(ForecastMethod::Sma),
        };
        ForecastRequest {
            satker_id: self.satker_id,
            kode_barang: self.kode_barang,
            horizon_years: self.horizon,
            method,
            confidence_level: self.confidence,
        }
    }
}

/// Query parameters for GET /forecast/summary
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastSummaryQuery {
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
}

/// Query parameters for GET /forecast/compare
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastCompareQuery {
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
    /// How many previous snapshots to compare (default: 2)
    pub limit: Option<i64>,
}

/// Response for forecast comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastCompareResponse {
    pub current: ForecastResult,
    pub previous_snapshots: Vec<ForecastSnapshot>,
}

/// Query parameters for GET /forecast/export
#[derive(Debug, Clone, Deserialize)]
pub struct ForecastExportQuery {
    pub satker_id: Option<Uuid>,
    pub kode_barang: Option<String>,
    /// Export format: csv | xlsx (default: csv)
    pub format: Option<String>,
}
