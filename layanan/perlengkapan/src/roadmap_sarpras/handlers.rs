//! Predictive Analytics HTTP handlers
//!
//! All endpoints are GET-only (read-only forecast dashboard).
//!
//! The forecast is computed over `roadmap_sarpras`, which is keyed by a legacy
//! satker *uuid* that no claim carries — so it cannot be confined to a satker.
//! It is the national picture, and is therefore for callers who may see every
//! satker (`Capability::ViewAllSatker`). No frontend calls it, and before this it
//! answered anyone authenticated, with `satker_id` as a client-chosen filter.

use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;

use super::models::{
    ForecastCompareQuery, ForecastCompareResponse, ForecastExportQuery, ForecastQuery,
    ForecastSummaryQuery,
};
use super::services::RoadmapService;

/// Generate a forecast from historical kebutuhan BMN data.
///
/// GET /api/v1/perlengkapan/forecast
pub async fn get_forecast(
    State(service): State<RoadmapService>,
    Query(query): Query<ForecastQuery>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    claims.require_capability(lib_core::authz::Capability::ViewAllSatker)?;
    let request = query.into_request();
    let result = service.generate_forecast(request).await?;
    Ok(Json(result))
}

/// Get forecast summary statistics (quick overview).
///
/// GET /api/v1/perlengkapan/forecast/summary
pub async fn get_forecast_summary(
    State(service): State<RoadmapService>,
    Query(query): Query<ForecastSummaryQuery>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    claims.require_capability(lib_core::authz::Capability::ViewAllSatker)?;
    let summary = service
        .get_summary(query.satker_id, query.kode_barang.as_deref())
        .await?;
    Ok(Json(summary))
}

/// Compare current forecast with previous snapshots.
///
/// GET /api/v1/perlengkapan/forecast/compare
pub async fn get_forecast_compare(
    State(service): State<RoadmapService>,
    Query(query): Query<ForecastCompareQuery>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    claims.require_capability(lib_core::authz::Capability::ViewAllSatker)?;
    let current_request = lib_perlengkapan::models::ForecastRequest {
        satker_id: query.satker_id,
        kode_barang: query.kode_barang.clone(),
        horizon_years: Some(5),
        method: None,
        confidence_level: None,
    };

    let current = service.generate_forecast(current_request).await?;
    let previous_snapshots = service
        .compare_forecasts(
            query.satker_id,
            query.kode_barang.as_deref(),
            query.limit.unwrap_or(2),
        )
        .await?;

    Ok(Json(ForecastCompareResponse {
        current,
        previous_snapshots,
    }))
}

/// Export forecast data as CSV.
///
/// GET /api/v1/perlengkapan/forecast/export
pub async fn export_forecast(
    State(service): State<RoadmapService>,
    Query(query): Query<ForecastExportQuery>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    claims.require_capability(lib_core::authz::Capability::ViewAllSatker)?;
    let csv_bytes = service
        .export_csv(query.satker_id, query.kode_barang.as_deref())
        .await?;

    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"forecast_sarpras.csv\"",
        ),
    ];

    Ok((StatusCode::OK, headers, csv_bytes))
}
