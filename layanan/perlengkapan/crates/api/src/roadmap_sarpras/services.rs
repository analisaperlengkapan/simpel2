//! Predictive Analytics forecasting engine
//!
//! Implements Simple Moving Average (SMA), Weighted Moving Average (WMA),
//! and Exponential Smoothing with confidence intervals.

use crate::errors::AppError;
use chrono::Utc;
use lib_perlengkapan::models::{
    ForecastMethod, ForecastRequest, ForecastResult, ForecastSnapshot, ForecastSummary,
    PredictedYear, YearlyData,
};

use super::repository::RoadmapRepository;

#[derive(Clone)]
pub struct RoadmapService {
    repository: RoadmapRepository,
}

impl RoadmapService {
    pub fn new(repository: RoadmapRepository) -> Self {
        Self { repository }
    }

    /// Generate a forecast based on historical kebutuhan_bmn data.
    pub async fn generate_forecast(
        &self,
        request: ForecastRequest,
    ) -> Result<ForecastResult, AppError> {
        let historical = self
            .repository
            .get_historical_data(request.satker_id, request.kode_barang.as_deref())
            .await?;

        if historical.len() < 2 {
            return Err(AppError::BadRequest(
                "Minimal 2 tahun data historis diperlukan untuk prediksi".into(),
            ));
        }

        let method = request.method.unwrap_or_default();
        let horizon = request.horizon_years.unwrap_or(5).max(1).min(10);
        let confidence = request.confidence_level.unwrap_or(0.95).clamp(0.5, 0.99);

        let predictions = match method {
            ForecastMethod::Sma => Self::forecast_sma(&historical, horizon, confidence),
            ForecastMethod::Wma => Self::forecast_wma(&historical, horizon, confidence),
            ForecastMethod::ExponentialSmoothing => {
                Self::forecast_exponential(&historical, horizon, confidence)
            }
        };

        let method_name = match method {
            ForecastMethod::Sma => "SMA",
            ForecastMethod::Wma => "WMA",
            ForecastMethod::ExponentialSmoothing => "ExponentialSmoothing",
        };

        let result = ForecastResult {
            method: method_name.to_string(),
            confidence_level: confidence,
            historical,
            predictions,
            generated_at: Utc::now(),
        };

        // Store snapshot for future comparison
        let predictions_json = serde_json::to_string(&result.predictions)
            .unwrap_or_default();
        let _ = self
            .repository
            .save_forecast_snapshot(
                method_name,
                confidence,
                request.satker_id,
                request.kode_barang.as_deref(),
                &predictions_json,
            )
            .await;

        Ok(result)
    }

    /// Get summary statistics (no prediction run, just overview).
    pub async fn get_summary(
        &self,
        satker_id: Option<uuid::Uuid>,
        kode_barang: Option<&str>,
    ) -> Result<ForecastSummary, AppError> {
        let historical = self
            .repository
            .get_historical_data(satker_id, kode_barang)
            .await?;

        if historical.is_empty() {
            return Ok(ForecastSummary {
                total_historical_years: 0,
                total_predicted_years: 0,
                avg_yearly_growth_pct: 0.0,
                total_predicted_kebutuhan: 0.0,
                total_predicted_biaya: 0.0,
                trend_direction: "no_data".into(),
            });
        }

        // Quick 5-year SMA forecast for summary numbers
        let predictions = Self::forecast_sma(&historical, 5, 0.95);
        let growth = Self::compute_avg_growth(&historical);

        let trend = if growth > 1.0 {
            "increasing"
        } else if growth < -1.0 {
            "decreasing"
        } else {
            "stable"
        };

        Ok(ForecastSummary {
            total_historical_years: historical.len() as i32,
            total_predicted_years: predictions.len() as i32,
            avg_yearly_growth_pct: growth,
            total_predicted_kebutuhan: predictions.iter().map(|p| p.predicted_kebutuhan).sum(),
            total_predicted_biaya: predictions.iter().map(|p| p.predicted_biaya).sum(),
            trend_direction: trend.into(),
        })
    }

    /// Compare current forecast with previous snapshots.
    pub async fn compare_forecasts(
        &self,
        satker_id: Option<uuid::Uuid>,
        kode_barang: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ForecastSnapshot>, AppError> {
        self.repository
            .get_previous_snapshots(satker_id, kode_barang, limit)
            .await
    }

    /// Export historical + forecast data as CSV bytes.
    pub async fn export_csv(
        &self,
        satker_id: Option<uuid::Uuid>,
        kode_barang: Option<&str>,
    ) -> Result<Vec<u8>, AppError> {
        let historical = self
            .repository
            .get_historical_data(satker_id, kode_barang)
            .await?;
        let predictions = Self::forecast_sma(&historical, 5, 0.95);

        let mut csv = String::from(
            "Tipe,Tahun,Kebutuhan,Existing,Gap,Jumlah Satker,Estimasi Biaya,CI Lower,CI Upper\n",
        );

        for h in &historical {
            csv.push_str(&format!(
                "Historis,{},{},{},{},{},{:.0},,\n",
                h.tahun,
                h.total_kebutuhan,
                h.total_existing,
                h.total_gap,
                h.jumlah_satker,
                h.estimasi_total_biaya
            ));
        }

        for p in &predictions {
            csv.push_str(&format!(
                "Prediksi,{},{:.0},,{:.0},,{:.0},{:.0},{:.0}\n",
                p.tahun,
                p.predicted_kebutuhan,
                p.predicted_gap,
                p.predicted_biaya,
                p.confidence_lower,
                p.confidence_upper
            ));
        }

        Ok(csv.into_bytes())
    }

    // ── Forecasting algorithms ───────────────────────────────────

    /// Simple Moving Average over all available years.
    fn forecast_sma(
        historical: &[YearlyData],
        horizon: i32,
        confidence: f64,
    ) -> Vec<PredictedYear> {
        let n = historical.len() as f64;
        if n < 1.0 {
            return vec![];
        }

        let avg_kebutuhan = historical.iter().map(|h| h.total_kebutuhan as f64).sum::<f64>() / n;
        let avg_gap = historical.iter().map(|h| h.total_gap as f64).sum::<f64>() / n;
        let avg_biaya = historical.iter().map(|h| h.estimasi_total_biaya).sum::<f64>() / n;

        // Standard deviation for CI
        let std_kebutuhan = Self::std_dev(
            &historical.iter().map(|h| h.total_kebutuhan as f64).collect::<Vec<_>>(),
        );

        let z = Self::z_score(confidence);
        let last_year = historical.last().map(|h| h.tahun).unwrap_or(2024);

        (1..=horizon)
            .map(|i| {
                let margin = z * std_kebutuhan * (1.0 + i as f64 * 0.1).sqrt();
                PredictedYear {
                    tahun: last_year + i,
                    predicted_kebutuhan: avg_kebutuhan,
                    predicted_gap: avg_gap,
                    predicted_biaya: avg_biaya,
                    confidence_lower: (avg_kebutuhan - margin).max(0.0),
                    confidence_upper: avg_kebutuhan + margin,
                }
            })
            .collect()
    }

    /// Weighted Moving Average – recent years get linearly increasing weight.
    fn forecast_wma(
        historical: &[YearlyData],
        horizon: i32,
        confidence: f64,
    ) -> Vec<PredictedYear> {
        let n = historical.len();
        if n < 1 {
            return vec![];
        }

        let weight_sum: f64 = (1..=n).map(|w| w as f64).sum();

        let wma_kebutuhan: f64 = historical
            .iter()
            .enumerate()
            .map(|(i, h)| (i + 1) as f64 * h.total_kebutuhan as f64)
            .sum::<f64>()
            / weight_sum;

        let wma_gap: f64 = historical
            .iter()
            .enumerate()
            .map(|(i, h)| (i + 1) as f64 * h.total_gap as f64)
            .sum::<f64>()
            / weight_sum;

        let wma_biaya: f64 = historical
            .iter()
            .enumerate()
            .map(|(i, h)| (i + 1) as f64 * h.estimasi_total_biaya)
            .sum::<f64>()
            / weight_sum;

        let std_kebutuhan = Self::std_dev(
            &historical.iter().map(|h| h.total_kebutuhan as f64).collect::<Vec<_>>(),
        );

        let z = Self::z_score(confidence);
        let last_year = historical.last().map(|h| h.tahun).unwrap_or(2024);

        // Apply trend from last 3 years
        let trend = Self::compute_trend(
            &historical.iter().map(|h| h.total_kebutuhan as f64).collect::<Vec<_>>(),
        );

        (1..=horizon)
            .map(|i| {
                let predicted = wma_kebutuhan + trend * i as f64;
                let margin = z * std_kebutuhan * (1.0 + i as f64 * 0.15).sqrt();
                PredictedYear {
                    tahun: last_year + i,
                    predicted_kebutuhan: predicted.max(0.0),
                    predicted_gap: (wma_gap + trend * 0.7 * i as f64).max(0.0),
                    predicted_biaya: (wma_biaya + wma_biaya * trend / wma_kebutuhan.max(1.0) * i as f64).max(0.0),
                    confidence_lower: (predicted - margin).max(0.0),
                    confidence_upper: predicted + margin,
                }
            })
            .collect()
    }

    /// Exponential Smoothing (single-parameter Holt model with trend).
    fn forecast_exponential(
        historical: &[YearlyData],
        horizon: i32,
        confidence: f64,
    ) -> Vec<PredictedYear> {
        let n = historical.len();
        if n < 2 {
            return Self::forecast_sma(historical, horizon, confidence);
        }

        let alpha = 0.3; // level smoothing
        let beta = 0.1; // trend smoothing

        let values: Vec<f64> = historical.iter().map(|h| h.total_kebutuhan as f64).collect();
        let biaya: Vec<f64> = historical.iter().map(|h| h.estimasi_total_biaya).collect();
        let gaps: Vec<f64> = historical.iter().map(|h| h.total_gap as f64).collect();

        // Initialise
        let mut level = values[0];
        let mut trend = values[1] - values[0];

        for v in values.iter().skip(1) {
            let new_level = alpha * v + (1.0 - alpha) * (level + trend);
            let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
            level = new_level;
            trend = new_trend;
        }

        // Biaya ratio
        let biaya_per_kebutuhan = if values.last().copied().unwrap_or(1.0) > 0.0 {
            biaya.last().copied().unwrap_or(0.0) / values.last().copied().unwrap_or(1.0)
        } else {
            0.0
        };
        let gap_ratio = if values.last().copied().unwrap_or(1.0) > 0.0 {
            gaps.last().copied().unwrap_or(0.0) / values.last().copied().unwrap_or(1.0)
        } else {
            0.0
        };

        let std_kebutuhan = Self::std_dev(&values);
        let z = Self::z_score(confidence);
        let last_year = historical.last().map(|h| h.tahun).unwrap_or(2024);

        (1..=horizon)
            .map(|i| {
                let predicted = level + trend * i as f64;
                let margin = z * std_kebutuhan * (1.0 + i as f64 * 0.2).sqrt();
                PredictedYear {
                    tahun: last_year + i,
                    predicted_kebutuhan: predicted.max(0.0),
                    predicted_gap: (predicted * gap_ratio).max(0.0),
                    predicted_biaya: (predicted * biaya_per_kebutuhan).max(0.0),
                    confidence_lower: (predicted - margin).max(0.0),
                    confidence_upper: predicted + margin,
                }
            })
            .collect()
    }

    // ── Helper math ──────────────────────────────────────────────

    fn std_dev(values: &[f64]) -> f64 {
        let n = values.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt()
    }

    fn compute_trend(values: &[f64]) -> f64 {
        let n = values.len();
        if n < 2 {
            return 0.0;
        }
        // Simple linear regression slope
        let n_f = n as f64;
        let x_mean = (n_f - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n_f;

        let numerator: f64 = values
            .iter()
            .enumerate()
            .map(|(i, y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();
        let denominator: f64 = (0..n).map(|i| (i as f64 - x_mean).powi(2)).sum();

        if denominator.abs() < f64::EPSILON {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn compute_avg_growth(historical: &[YearlyData]) -> f64 {
        if historical.len() < 2 {
            return 0.0;
        }
        let growths: Vec<f64> = historical
            .windows(2)
            .filter_map(|w| {
                if w[0].total_kebutuhan > 0 {
                    Some(
                        ((w[1].total_kebutuhan - w[0].total_kebutuhan) as f64
                            / w[0].total_kebutuhan as f64)
                            * 100.0,
                    )
                } else {
                    None
                }
            })
            .collect();
        if growths.is_empty() {
            0.0
        } else {
            growths.iter().sum::<f64>() / growths.len() as f64
        }
    }

    /// Approximate z-score for common confidence levels.
    fn z_score(confidence: f64) -> f64 {
        match () {
            _ if confidence >= 0.99 => 2.576,
            _ if confidence >= 0.98 => 2.326,
            _ if confidence >= 0.95 => 1.96,
            _ if confidence >= 0.90 => 1.645,
            _ if confidence >= 0.80 => 1.282,
            _ => 1.0,
        }
    }
}
