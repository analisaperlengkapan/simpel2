//! Predictive Analytics service: fetches the yearly series and hands it to the pure math in
//! [`super::forecast`].

use crate::shared::error::AppError;
use chrono::Utc;
use lib_perlengkapan::models::{
    ForecastMethod, ForecastRequest, ForecastResult, ForecastSnapshot, ForecastSummary,
};

use super::forecast::{
    catatan as susun_catatan, deret_berurutan, pertumbuhan_majemuk_persen, prediksi,
};
use super::repository::RoadmapRepository;

/// Years the summary projects.
const HORIZON_RINGKASAN: usize = 5;

#[derive(Clone)]
pub struct RoadmapService {
    repository: RoadmapRepository,
}

impl RoadmapService {
    pub fn new(repository: RoadmapRepository) -> Self {
        Self { repository }
    }

    /// Generate a forecast based on historical roadmap data.
    pub async fn generate_forecast(
        &self,
        request: ForecastRequest,
    ) -> Result<ForecastResult, AppError> {
        let historical = self
            .repository
            .get_historical_data(request.satker_id, request.kode_barang.as_deref())
            .await?;

        // The series the math runs on is the latest run of consecutive years: a year with no rows
        // is not a zero, and reading "2019, 2021" as neighbours tilts every trend.
        let (deret, catatan_bolong) = deret_berurutan(&historical);
        if deret.len() < 2 {
            return Err(AppError::BadRequest(
                "Minimal 2 tahun berurutan data historis diperlukan untuk prediksi".into(),
            ));
        }

        let method = request.method.unwrap_or_default();
        let horizon = request.horizon_years.unwrap_or(5).clamp(1, 10) as usize;
        let confidence = request.confidence_level.unwrap_or(0.95).clamp(0.5, 0.99);

        let predictions = prediksi(&method, deret, horizon, confidence);
        let method_name = nama_metode(&method);
        let catatan = susun_catatan(deret.len(), &method, catatan_bolong);

        let result = ForecastResult {
            method: method_name.to_string(),
            confidence_level: confidence,
            historical: deret.to_vec(),
            predictions,
            generated_at: Utc::now(),
            catatan,
        };

        self.simpan_bila_berubah(
            method_name,
            confidence,
            request.satker_id,
            request.kode_barang.as_deref(),
            &result,
        )
        .await;

        Ok(result)
    }

    /// Keep a snapshot for later comparison, but only when it differs from the latest one for the
    /// same question (same method and confidence). Every GET used to write a row, so a dashboard
    /// refreshed ten times left ten identical "previous forecasts", and the failure to write was
    /// swallowed without a trace.
    async fn simpan_bila_berubah(
        &self,
        method: &str,
        confidence: f64,
        satker_id: Option<uuid::Uuid>,
        kode_barang: Option<&str>,
        result: &ForecastResult,
    ) {
        let json = match serde_json::to_string(&result.predictions) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(error = %e, "forecast snapshot not saved: predictions did not serialize");
                return;
            }
        };
        // Against the latest of the SAME method: two methods asked in turn would otherwise each
        // differ from "the latest" and pile up copies of one another.
        match self
            .repository
            .get_previous_snapshots(satker_id, kode_barang, 50)
            .await
        {
            Ok(recent) => {
                let sama = recent.iter().find(|s| {
                    s.method == method && (s.confidence_level - confidence).abs() < f64::EPSILON
                });
                if sama.is_some_and(|s| s.predictions_json == json) {
                    return;
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "forecast snapshot: could not read the latest ones")
            }
        }
        if let Err(e) = self
            .repository
            .save_forecast_snapshot(method, confidence, satker_id, kode_barang, &json)
            .await
        {
            tracing::warn!(error = %e, "forecast snapshot not saved");
        }
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
        let (deret, _) = deret_berurutan(&historical);

        if deret.is_empty() {
            return Ok(ForecastSummary {
                total_historical_years: 0,
                total_predicted_years: 0,
                avg_yearly_growth_pct: 0.0,
                total_predicted_kebutuhan: 0.0,
                total_predicted_biaya: 0.0,
                trend_direction: "no_data".into(),
            });
        }

        // The projection the summary totals is the default one (SMA), over the same series as the
        // forecast endpoint.
        let predictions = prediksi(&ForecastMethod::default(), deret, HORIZON_RINGKASAN, 0.95);
        // Compound growth, or "stable" when it is not defined (one year, or a first year of zero):
        // an unknown rate is not a rate of zero, but it is not a direction either.
        let growth = pertumbuhan_majemuk_persen(deret);
        let trend = match growth {
            Some(g) if g > 1.0 => "increasing",
            Some(g) if g < -1.0 => "decreasing",
            _ => "stable",
        };

        Ok(ForecastSummary {
            total_historical_years: deret.len() as i32,
            total_predicted_years: predictions.len() as i32,
            avg_yearly_growth_pct: growth.unwrap_or(0.0),
            total_predicted_kebutuhan: predictions.iter().map(|p| p.predicted_kebutuhan).sum(),
            total_predicted_biaya: predictions.iter().map(|p| p.predicted_biaya).sum(),
            trend_direction: trend.into(),
        })
    }

    /// The snapshots saved BEFORE now. Read before the current forecast is generated: the comparison
    /// used to generate (and save) the current one first and then list the latest snapshots, so the
    /// first "previous" forecast was the one just made.
    pub async fn compare_forecasts(
        &self,
        satker_id: Option<uuid::Uuid>,
        kode_barang: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ForecastSnapshot>, AppError> {
        self.repository
            .get_previous_snapshots(satker_id, kode_barang, limit.clamp(1, 20))
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
        let (deret, _) = deret_berurutan(&historical);
        let predictions = prediksi(&ForecastMethod::default(), deret, HORIZON_RINGKASAN, 0.95);

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
}

fn nama_metode(method: &ForecastMethod) -> &'static str {
    match method {
        ForecastMethod::Sma => "SMA",
        ForecastMethod::Wma => "WMA",
        ForecastMethod::ExponentialSmoothing => "ExponentialSmoothing",
    }
}
